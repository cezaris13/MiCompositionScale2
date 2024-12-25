#[cfg(test)]
mod tests {
    use crate::http_request_handler::HttpRequestHandler;
    use crate::http_request_handler::IHttpRequestHandler;
    use crate::tests::mock_response_builder::mock_response;

    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_handle_http_request_ok_status() {
        let response_body = "OK response";
        let response = mock_response(StatusCode::OK, response_body);

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(Ok(response));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_handle_http_request_created_status() {
        let response_body = "Created response";
        let response = mock_response(StatusCode::CREATED, response_body);

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(Ok(response));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_handle_http_request_error_status() {
        let handler = HttpRequestHandler::new();
        let response_body = "Error response";
        let response = mock_response(StatusCode::BAD_REQUEST, response_body);

        let result = handler.handle_http_request(Ok(response));

        assert!(result.is_err());
    }

    #[test]
    fn test_handle_http_request_err_response() {
        let response = mock_response(StatusCode::BAD_REQUEST, "response body").error_for_status();

        let sut = HttpRequestHandler::new();
        let result = sut.handle_http_request(response);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_response_body_ok() {
        let response = mock_response(StatusCode::OK, "response body");

        let sut = HttpRequestHandler::new();
        let result = sut.get_response_body(response).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "response body");
    }
}
