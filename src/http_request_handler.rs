use crate::cli_error::CliError;

use reqwest::{Error, Response, StatusCode};

pub struct HttpRequestHandler;

impl HttpRequestHandler {
    pub fn handle_http_request(response: Result<Response, Error>) -> Result<Response, CliError> {
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

    pub async fn get_response_body(response: Response) -> Result<String, CliError> {
        match response.text().await {
            Ok(text) => Ok(text),
            Err(_) => Err(CliError::Error(String::from(
                "Failed to retrieve response body",
            ))),
        }
    }
}
