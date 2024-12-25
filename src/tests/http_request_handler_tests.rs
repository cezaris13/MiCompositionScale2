#[cfg(test)]
mod tests {
    use crate::cli_error::CliError;
    use crate::http_request_handler::HttpRequestHandler;
    use crate::http_request_handler::IHttpRequestHandler;
    use crate::tests::test_utils::get_mock_response;

    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_handle_http_request_ok_status() {
        let response_body = "OK response";
        let response = get_mock_response(StatusCode::OK, response_body);

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(Ok(response));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_handle_http_request_created_status() {
        let response_body = "Created response";
        let response = get_mock_response(StatusCode::CREATED, response_body);

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(Ok(response));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_handle_http_request_error_status() {
        let handler = HttpRequestHandler::new();
        let response_body = "Error response";
        let response = get_mock_response(StatusCode::BAD_REQUEST, response_body);

        let result = handler.handle_http_request(Ok(response));

        assert!(result.is_err());
        if let Err(CliError::Error(ref message)) = result {
            assert_eq!(
                message,
                "Failed to get data from the request: status code 400 Bad Request"
            );
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[test]
    fn test_handle_http_request_err_response() {
        let response =
            get_mock_response(StatusCode::BAD_REQUEST, "response body").error_for_status();

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(response);

        assert!(result.is_err());
        if let Err(CliError::Error(ref message)) = result {
            assert_eq!(
                message,
                "HTTP status client error (400 Bad Request) for url (http://no.url.provided.local/)"
            );
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_get_response_body_ok() {
        let response = get_mock_response(StatusCode::OK, "response body");

        let sut = HttpRequestHandler::new();
        let result = sut.get_response_body(response).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "response body");
    }
}
