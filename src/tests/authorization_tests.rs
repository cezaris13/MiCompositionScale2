#[cfg(test)]
mod tests {
    use crate::authorization::Authorization;
    use crate::authorization::IAuthorization;
    // use crate::data_types::config::Config;
    use crate::data_types::token::Token;
    use crate::http_request_handler::MockIHttpRequestHandler;
    // use crate::tests::mock_response_builder::mock_response;
    use crate::utils::MockIUtils;
    use crate::utils::Utils;

    #[tokio::test]
    async fn test_get_auth_token() {
        // let mut mock_utils = MockIUtils::new();
        // let mut mock_http_handler = MockIHttpRequestHandler::new();
        // let mock_client = reqwest::Client::new();

        // // Define the behavior of the mocked methods
        // mock_utils
        //     .expect_get_current_project_directory()
        //     .returning(|| Ok(String::from("/mock/directory")));
        // mock_utils.expect_read_configuration_file().returning(|| {
        //     Ok(Config {
        //         mac_address: String::from("Some mac address"),
        //         client_id: String::from("mock_client_id"),
        //         client_secret: String::from("mock_secret"),
        //     })
        // });
        // mock_http_handler
        //     .expect_handle_http_request()
        //     .returning(|_| Ok(mock_response(reqwest::StatusCode::OK, "some body")));

        // let sut = Authorization {
        //     utils: &mock_utils,
        //     http_request_handler: &mock_http_handler,
        //     http_client: &mock_client,
        // };

        // // Create test data
        // let client_id = String::from("mock_client_id");
        // let secret = String::from("mock_secret");

        // Call the method
        // let result = sut.get_auth_token(client_id, secret).await;

        // // Assertions
        // assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_auth_token_invalid_dir_returns_error() {
        let mut mock_utils = MockIUtils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let token = Token {
            access_token: String::from("mock_access_token"),
            refresh_token: String::from("mock_refresh_token"),
        };

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.write_auth_token(token);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_auth_token_writes_token_to_file() {
        let mock_utils = Utils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let token = Token {
            access_token: String::from("mock_access_token"),
            refresh_token: String::from("mock_refresh_token"),
        };

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.write_auth_token(token);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_exists_random_dir_returns_false() {
        let mut mock_utils = MockIUtils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.file_exists();
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // #[tokio::test]
    // async fn test_is_access_token_expired() {
    //     let mock_utils = MockIUtils::new();
    //     let mock_client = reqwest::Client::new();
    //     let mock_http_handler = MockIHttpRequestHandler::new();
    //     let authorization = Authorization {
    //         utils: &mock_utils,
    //         http_request_handler: &mock_http_handler,
    //         http_client: &mock_client,
    //     };

    //     // Define mock token
    //     let mock_token = String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMjM0NTY3ODkwLCJleHBpcnkiOiIyMDI0LTAxLTAxVDEyOjAwOjAwWiJ9.H1Jnt1e8RJ-1SxLVqIs9gL2O9MwK8V78VzNEsaJlVHg");

    //     // Mock behavior for token expiration check
    //     let expired = authorization.is_access_token_expired(&mock_token).unwrap();

    //     assert_eq!(expired, true);
    // }
    //
    //

    // #[tokio::test]
    // async fn test_refresh_access_token() {
    //     // Mock the HTTP request
    //     let mock_http_request_handler = MockIHttpRequestHandler::new();
    //     // Mock utils functions
    //     let utils = MockIUtils::new();

    //     // Mock client
    //     let client = reqwest::Client::new();

    //     // Create an instance of your struct
    //     let sut = Authorization::new(&utils, &mock_http_request_handler, &client);

    //     // Call the method being tested
    //     let result = sut.refresh_access_token().await;

    //     // Assert the result
    //     match result {
    //         Ok(access_token) => {
    //             assert_eq!(access_token, "new_access_token");
    //         }
    //         Err(e) => panic!("Test failed: {}", e),
    //     }
    // }
}
