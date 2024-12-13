use crate::cli_error::CliError;

use async_trait::async_trait;
use mockall::{automock, predicate::*};
use reqwest::{Error, Response, StatusCode};

#[cfg(test)]
#[path = "./tests/http_request_handler_tests.rs"]
mod tests;

pub struct HttpRequestHandler;

impl HttpRequestHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[automock]
#[async_trait]
pub trait IHttpRequestHandler: Sync {
    fn handle_http_request(&self, response: Result<Response, Error>) -> Result<Response, CliError>;
    async fn get_response_body(&self, response: Response) -> Result<String, CliError>;
}

#[async_trait]
impl IHttpRequestHandler for HttpRequestHandler {
    fn handle_http_request(&self, response: Result<Response, Error>) -> Result<Response, CliError> {
        match response {
            // fix this
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(resp),
                StatusCode::CREATED => Ok(resp),
                status_code => Err(CliError::Error(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                ))),
            },
            Err(err) => Err(CliError::Error(err.to_string())),
        }
    }

    async fn get_response_body(&self, response: Response) -> Result<String, CliError> {
        match response.text().await {
            Ok(text) => Ok(text),
            Err(_) => Err(CliError::Error(String::from(
                "Failed to retrieve response body",
            ))),
        }
    }
}
