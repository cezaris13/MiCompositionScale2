use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use log::{error, info};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::url::Url;
use oauth2::TokenResponse;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, Scope, TokenUrl};

use crate::cli_error::CliError;
use crate::data_types::Token;
use crate::utils::get_current_project_directory;

const TOKEN_FILE: &str = "auth_token.json";

/// Get a token via the OAuth 2.0 Implicit Grant Flow
pub async fn get_token(client_id: String, client_secret: String) -> Result<Token, CliError> {
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
    opener::open(authorize_url.to_string()).expect("failed to open authorize URL");
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code: AuthorizationCode;

            let state: CsrfToken;
            {
                let mut reader: BufReader<&std::net::TcpStream> = BufReader::new(&stream);

                let mut request_line: String = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = match request_line.split_whitespace().nth(1) {
                    Some(element) => element,
                    None => {
                        return Err(CliError::Error(String::from(
                            "No element has been provided",
                        )))
                    }
                };

                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

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
            stream.write_all(response.as_bytes()).unwrap();

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

            return Ok(Token {
                access_token: token.access_token().secret().clone(),
                refresh_token: token.refresh_token().expect("REASON").secret().clone(),
            });
        }
    }

    unreachable!();
}

pub async fn get_auth_token(id: String, secret: String) -> Result<(), CliError> {
    let token: Token = get_token(id, secret).await?;
    write_auth_token(token)?;
    info!("Success! OAuth2 token recorded to {}.", TOKEN_FILE);
    Ok(())
}

pub fn write_auth_token(token: Token) -> Result<(), CliError> {
    let json_token = serde_json::to_string(&token)?;
    let token_file: String = get_current_project_directory()? + "/" + TOKEN_FILE;
    let mut file: File = File::create(token_file)?;
    file.write_all(json_token.as_bytes())?;

    Ok(())
}

pub fn read_auth_token() -> Result<Token, CliError> {
    let token_file: String = get_current_project_directory()? + "/" + TOKEN_FILE;
    match fs::read_to_string(token_file) {
        Ok(token) => Ok(serde_json::from_str(&token)?),
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

pub fn file_exists() -> Result<bool, CliError> {
    let token_file: String = get_current_project_directory()? + "/" + TOKEN_FILE;
    Ok(fs::metadata(token_file).is_ok())
}
