use crate::cli_error::CliError;
use crate::data_types::token::{Payload, Token};
use crate::http_request_handler::IHttpRequestHandler;
use crate::utils::IUtils;
use std::borrow::Cow;

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use basic::{BasicErrorResponseType, BasicTokenType};
use jsonwebtokens::raw::{self, decode_json_token_slice, TokenSlices};
use log::{error, info};
use mockall::{automock, predicate::*};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::{
    basic, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::{from_str, from_value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

type OAuth2Client = oauth2::Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

#[cfg(test)]
#[path = "./tests/authorization_tests.rs"]
mod tests;

const TOKEN_FILE: &str = "auth_token.json";

pub struct Authorization<'a> {
    utils: &'a dyn IUtils,
    http_request_handler: &'a dyn IHttpRequestHandler,
    http_client: &'a Client,
}

impl<'a> Authorization<'a> {
    pub fn new(
        utils: &'a impl IUtils,
        http_request_handler: &'a impl IHttpRequestHandler,
        http_client: &'a Client,
    ) -> Self {
        Self {
            utils,
            http_request_handler,
            http_client,
        }
    }
}

#[automock]
#[async_trait]
pub trait IAuthorization: Sync {
    async fn get_auth_token(&self, client_id: String, secret: String) -> Result<(), CliError>;
    fn write_auth_token(&self, token: Token) -> Result<(), CliError>;
    fn file_exists(&self) -> Result<bool, CliError>;
    fn read_auth_token(&self) -> Result<Token, CliError>;
    async fn get_access_token(&self) -> Result<String, CliError>;
    async fn refresh_access_token(&self) -> Result<String, CliError>;
    fn is_access_token_expired(&self, access_token: &String) -> Result<bool, CliError>;
    async fn get_token(&self, client_id: String, client_secret: String) -> Result<Token, CliError>;
    fn get_url_from_stream(&self, stream: &TcpStream) -> Result<Url, CliError>;
    fn get_key_value_from_url(&self, url: &Url, key_parameter: &str) -> Result<String, CliError>;
    fn get_code_from_tcp_listener(
        &self,
        csrf_state: CsrfToken,
    ) -> Result<AuthorizationCode, CliError>;
    async fn exchange_code_with_token(
        &self,
        client: OAuth2Client,
        code: AuthorizationCode,
    ) -> Result<Token, CliError>;
}

#[async_trait]
impl<'a> IAuthorization for Authorization<'a> {
    async fn get_auth_token(&self, id: String, secret: String) -> Result<(), CliError> {
        let token: Token = self.get_token(id, secret).await?;
        self.write_auth_token(token)?;
        info!("Success! OAuth2 token recorded to {}.", TOKEN_FILE);
        Ok(())
    }

    fn write_auth_token(&self, token: Token) -> Result<(), CliError> {
        let json_token = serde_json::to_string(&token)?;
        let token_file: String = self.utils.get_current_project_directory()? + "/" + TOKEN_FILE;
        let mut file: File = File::create(token_file)?;
        file.write_all(json_token.as_bytes())?;

        Ok(())
    }

    fn file_exists(&self) -> Result<bool, CliError> {
        let token_file: String = self.utils.get_current_project_directory()? + "/" + TOKEN_FILE;
        Ok(fs::metadata(token_file).is_ok())
    }

    fn read_auth_token(&self) -> Result<Token, CliError> {
        let token_file: String = self.utils.get_current_project_directory()? + "/" + TOKEN_FILE;
        match fs::read_to_string(token_file) {
            Ok(token) => Ok(from_str(&token)?),
            Err(error) => {
                log::error!(
                    "Failed to read the auth token ({})\nHave you run the `auth` command?",
                    error
                );
                Err(CliError::Error(String::from(
                    "Failed to read the auth token.\nHave you run the `auth` command?",
                )))
            }
        }
    }

    async fn get_access_token(&self) -> Result<String, CliError> {
        let access_token = self.read_auth_token()?.access_token;

        if self.is_access_token_expired(&access_token)? {
            return self.refresh_access_token().await;
        }

        Ok(access_token)
    }

    async fn refresh_access_token(&self) -> Result<String, CliError> {
        let refresh_token: String = self.read_auth_token()?.refresh_token;
        // let client_id: String = self.utils.read_configuration_file()?.client_id;
        let client_id: String = String::from("");
        let client_secret: String = self.utils.read_configuration_file()?.client_secret;

        let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);
        let params = [
            ("refresh_token", refresh_token),
            ("grant_type", String::from("refresh_token")),
        ];
        let url: Url = Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params)?;

        let response = self
            .http_client
            .post(url)
            .header(AUTHORIZATION, format!("Basic {encoded_client_data}"))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await;

        let response = self.http_request_handler.handle_http_request(response)?;

        let response_body: String = self
            .http_request_handler
            .get_response_body(response)
            .await?;
        let token_data: Token = from_str(response_body.as_str())?;
        self.write_auth_token(token_data.clone())?;

        Ok(token_data.access_token)
    }

    fn is_access_token_expired(&self, access_token: &String) -> Result<bool, CliError> {
        let TokenSlices { claims, .. } = raw::split_token(access_token)?;
        let raw_claim = decode_json_token_slice(claims)?;
        let final_claim: Payload = from_value(raw_claim.clone())?;

        let current_time_since_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(current_time_since_unix > final_claim.exp)
    }

    /// Get a token via the OAuth 2.0 Implicit Grant Flow
    async fn get_token(&self, client_id: String, client_secret: String) -> Result<Token, CliError> {
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://www.fitbit.com/oauth2/authorize".to_string())?,
            Some(TokenUrl::new(
                "https://api.fitbit.com/oauth2/token".to_string(),
            )?),
        );

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("weight".to_string()))
            .url();

        opener::open(authorize_url.to_string())?;

        let code = self.get_code_from_tcp_listener(csrf_state)?;

        self.exchange_code_with_token(client, code).await
    }

    fn get_url_from_stream(&self, stream: &TcpStream) -> Result<Url, CliError> {
        let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);

        let mut request_line: String = String::new();
        reader.read_line(&mut request_line)?;

        let redirect_url = match request_line.split_whitespace().nth(1) {
            Some(element) => element,
            None => {
                return Err(CliError::Error(String::from(
                    "No element has been provided",
                )))
            }
        };

        Ok(Url::parse(
            &("http://localhost".to_string() + redirect_url),
        )?)
    }

    fn get_key_value_from_url(&self, url: &Url, key_parameter: &str) -> Result<String, CliError> {
        let pair = url
            .query_pairs()
            .find(|pair: &(Cow<'_, str>, Cow<'_, str>)| {
                let &(ref key, _) = pair;
                key == key_parameter
            });

        match pair {
            Some((_, value)) => Ok(value.into_owned()),
            None => Err(CliError::Error(format!(
                "Value for key {key_parameter} was not found"
            ))),
        }
    }

    fn get_code_from_tcp_listener(
        &self,
        csrf_state: CsrfToken,
    ) -> Result<AuthorizationCode, CliError> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        let mut code: Option<AuthorizationCode> = None;

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let url = self.get_url_from_stream(&stream)?;

                let value = self.get_key_value_from_url(&url, "code")?;
                code = Some(AuthorizationCode::new(value));

                let value = self.get_key_value_from_url(&url, "state")?;
                let state = CsrfToken::new(value);

                let message = "Token has been retrieved. You may close the tab.";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes())?;

                // Verify that the state we generated matches the one the server sent us.
                if csrf_state.secret() != state.secret() {
                    return Err(CliError::Error(String::from(
                        "CSRF state mismatch. Malicious actor?",
                    )));
                }

                break;
            }
        }
        match code {
            Some(code) => Ok(code),
            None => Err(CliError::Error(String::from(
                "No code has been retrieved. Aborting.",
            ))),
        }
    }

    async fn exchange_code_with_token(
        &self,
        client: OAuth2Client,
        code: AuthorizationCode,
    ) -> Result<Token, CliError> {
        let token = match client
            .exchange_code(code)
            .request_async(async_http_client)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                error!("OAuth2: {}", e);
                return Err(CliError::Error(e.to_string()));
            }
        };

        let refresh_token = match token.refresh_token() {
            Some(token) => token.secret(),
            None => "",
        };

        Ok(Token {
            access_token: token.access_token().secret().to_string(),
            refresh_token: refresh_token.to_string(),
        })
    }
}
