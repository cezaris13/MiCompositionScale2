use crate::cli_error::CliError;
use crate::data_types::token::{Payload, Token};
use crate::http_request_handler::IHttpRequestHandler;
use crate::utils::IUtils;

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use jsonwebtokens::raw::{self, decode_json_token_slice, TokenSlices};
use log::{error, info};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Scope, TokenResponse, TokenUrl,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::{from_str, from_value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

const TOKEN_FILE: &str = "auth_token.json";

pub struct Authorization<'a> {
    utils: &'a dyn IUtils,
    http_request_handler: &'a dyn IHttpRequestHandler,
}

impl<'a> Authorization<'a> {
    pub fn new(utils: &'a impl IUtils, http_request_handler: &'a impl IHttpRequestHandler) -> Self {
        Self {
            utils,
            http_request_handler,
        }
    }
}

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

        let client: Client = Client::new();
        let response = client
            .post(url)
            .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
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
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let code: AuthorizationCode;

                let state: CsrfToken;
                {
                    let mut reader: BufReader<&std::net::TcpStream> = BufReader::new(&stream);

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

                    let url = Url::parse(&("http://localhost".to_string() + redirect_url))?;

                    let code_pair: (std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>) = url
                        .query_pairs()
                        .find(
                            |pair: &(std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>)| {
                                let &(ref key, _) = pair;
                                key == "code"
                            },
                        )
                        .unwrap();

                    let (_, value) = code_pair;
                    code = AuthorizationCode::new(value.into_owned());

                    let state_pair: (std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>) = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "state"
                        })
                        .unwrap();

                    let (_, value) = state_pair;
                    state = CsrfToken::new(value.into_owned());
                }

                let message = "Token has been retrieved. You may close the tab.";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes())?;

                // Verify that the state we generated matches the one the server sent us.
                assert_eq!(
                    csrf_state.secret(),
                    state.secret(),
                    "CSRF state mismatch. Malicious actor?"
                );

                // Exchange the code with a token.
                let token = match client
                    .exchange_code(code)
                    .request_async(async_http_client)
                    .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        error!("OAuth2: {}", e);
                        eprintln!("Failed to exchange the code for a valid access_token.\nIncorrect client secret?");
                        return Err(CliError::Error(e.to_string()));
                    }
                };

                let refresh_token = match token.refresh_token() {
                    Some(token) => token.secret(),
                    None => "",
                };

                return Ok(Token {
                    access_token: token.access_token().secret().to_string(),
                    refresh_token: refresh_token.to_string(),
                });
            }
        }

        unreachable!();
    }
}
