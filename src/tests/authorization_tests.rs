#[cfg(test)]
mod tests {
    use crate::authorization::Authorization;
    use crate::authorization::IAuthorization;
    use crate::authorization::TOKEN_FILE;
    use crate::cli_error::CliError;
    use crate::http_request_handler::MockIHttpRequestHandler;
    use crate::tests::test_utils::{
        create_mock_stream, create_mock_token, get_mock_config, get_mock_response, get_mock_token,
    };
    use crate::tests::vector_logger::LOGGER;
    use crate::utils::MockIUtils;
    use crate::utils::{IUtils, Utils};

    use oauth2::url::Url;
    use reqwest::StatusCode;
    use serial_test::serial;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    #[serial]
    async fn test_write_auth_token_invalid_dir_returns_error() {
        let mut mock_utils = MockIUtils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let token = get_mock_token(None);

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.write_auth_token(token);

        assert!(result.is_err());

        if let Err(CliError::IOError(ref error)) = result {
            assert_eq!(
                format!("{:?}", error),
                String::from(
                    "Os { code: 2, kind: NotFound, message: \"No such file or directory\" }"
                )
            );
        } else {
            assert!(
                false,
                "Expected error: CliError::IOError(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_write_auth_token_writes_token_to_file() {
        let mock_utils = Utils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let token = get_mock_token(None);

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.write_auth_token(token);

        let _ =
            fs::remove_file(mock_utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE);
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_write_auth_token_current_dir_error_returns_error() {
        let mut mock_utils = MockIUtils::new();

        let error_message = "some error";
        mock_utils
            .expect_get_current_project_directory()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();
        let token = get_mock_token(None);

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.write_auth_token(token);

        assert!(result.is_err());

        if let Err(CliError::Error(ref message)) = result {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    #[serial]
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

    #[tokio::test]
    #[serial]
    async fn test_file_exists_get_current_dir_fails_returns_error() {
        let mut mock_utils = MockIUtils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let error_message = "some error";
        mock_utils
            .expect_get_current_project_directory()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.file_exists();

        assert!(result.is_err());

        if let Err(CliError::Error(ref message)) = result {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[test]
    #[serial]
    fn test_read_auth_token_success() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();
        let utils = Utils::new();

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;

        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };

        let result = sut.read_auth_token();

        let _ = fs::remove_file(token_file_path);

        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap().access_token, "mocked_access_token");
        assert_eq!(result.unwrap().refresh_token, "mocked_refresh_token");

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 0);
    }

    #[test]
    #[serial]
    fn test_read_auth_token_file_not_found() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };

        let result = sut.read_auth_token();

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Error in program: Failed to read the auth token.\nHave you run the `auth` command?"
        );

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 1);

        assert_eq!(logs[0], "[ERROR] Failed to read the auth token (No such file or directory (os error 2))\nHave you run the `auth` command?");
    }

    #[test]
    #[serial]
    fn test_read_auth_token_invalid_json() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();
        let utils = Utils::new();

        let invalid_token_str = "invalid_token_content";

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), invalid_token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };
        let result = sut.read_auth_token();

        let _ = fs::remove_file(token_file_path);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Parse error: expected value at line 1 column 1"
        );

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_auth_token() {
        // let mut mock_utils = MockIUtils::new();
        // let mut mock_http_handler = MockIHttpRequestHandler::new();
        // let mock_client = reqwest::Client::new();

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

        // let client_id = String::from("mock_client_id");
        // let secret = String::from("mock_secret");

        // let result = sut.get_auth_token(client_id, secret).await;

        // assert!(result.is_ok());
    }

    #[test]
    fn test_is_access_token_expired_expired_token() {
        let expired_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 100; // Set expiration time 100 seconds in the past
        let mock_token = create_mock_token(expired_time);

        let sut = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.is_access_token_expired(&mock_token);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_access_token_expired_valid_token() {
        let valid_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // Set expiration time 1 hour in the future
        let mock_token = create_mock_token(valid_time);

        let sut = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.is_access_token_expired(&mock_token);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_access_token_expired_malformed_token() {
        let malformed_token = "invalid.token.structure".to_string();

        let sut = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.is_access_token_expired(&malformed_token);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "JsonWebToken error: Invalid Input: base64 decode failure"
        );
    }

    // #[tokio::test]
    // #[serial]
    // async fn test_get_access_token_valid_token() {
    //     let mock_utils = Utils::new();
    //     let mock_client = reqwest::Client::new();
    //     let mock_http_handler = MockIHttpRequestHandler::new();

    //     let token = get_mock_token(None);

    //     let sut = Authorization {
    //         utils: &mock_utils,
    //         http_request_handler: &mock_http_handler,
    //         http_client: &mock_client,
    //     };

    //     sut.write_auth_token(token.clone()).unwrap();

    //     let result = sut.get_access_token().await;

    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), token.access_token);
    // }

    // #[tokio::test]
    // #[serial]
    // async fn test_get_access_token_expired_token() {
    //     let mut mock_utils = MockIUtils::new();
    //     let mock_client = reqwest::Client::new();
    //     let mut mock_http_handler = MockIHttpRequestHandler::new();
    //     mock_http_handler
    //         .expect_handle_http_request()
    //         .times(1)
    //         .returning(|_| Ok(mock_response(StatusCode::OK, "new token")));

    //     let expired_token = "expired_token";

    //     mock_utils
    //         .expect_get_current_project_directory()
    //         .returning(|| Ok(String::from("/mock/directory")));

    //     let sut = Authorization {
    //         utils: &mock_utils,
    //         http_request_handler: &mock_http_handler,
    //         http_client: &mock_client,
    //     };

    //     let result = sut.get_access_token().await;

    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), expired_token);
    // }

    #[tokio::test]
    #[serial]
    async fn test_get_access_token_read_auth_token_error() {
        let mut mock_utils = MockIUtils::new();
        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let error_message = "Failed to read auth token";

        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Err(CliError::Error(error_message.to_string())));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &mock_http_handler,
            http_client: &mock_client,
        };

        let result = sut.get_access_token().await;

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Error in program: {error_message}")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "Some body")));

        let new_access_token = "new access token";
        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| {
                Ok(
                    serde_json::to_string(&get_mock_token(Some(String::from(new_access_token))))
                        .unwrap(),
                )
            });

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&get_mock_config()).unwrap(),
        );

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);

        let result = sut.refresh_access_token().await;

        let _ = fs::remove_file(token_file_path);
        let _ = fs::remove_file(config_file_path);

        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), new_access_token);

        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token_fails_to_read_auth_token_returns_error() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);
        let result = sut.refresh_access_token().await;

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Error in program: Failed to read the auth token.\nHave you run the `auth` command?"
        );

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 1);

        assert_eq!(logs[0], "[ERROR] Failed to read the auth token (No such file or directory (os error 2))\nHave you run the `auth` command?");
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token_fails_to_read_config_file_returns_error() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mock_http_handler = MockIHttpRequestHandler::new();

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);

        let result = sut.refresh_access_token().await;

        let _ = fs::remove_file(token_file_path);

        assert_eq!(
            result.err().unwrap().to_string(),
            "Error in program: No such file or directory (os error 2)"
        );

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 1);

        assert_eq!(
            logs[0],
            "[ERROR] Failed to read the config file No such file or directory (os error 2)"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token_fails_to_handle_response_returns_error() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mut mock_http_handler = MockIHttpRequestHandler::new();
        let error_message = "some error message";
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&get_mock_config()).unwrap(),
        );

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);

        let result = sut.refresh_access_token().await;

        let _ = fs::remove_file(token_file_path);
        let _ = fs::remove_file(config_file_path);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Error in program: {error_message}")
        );

        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token_fails_to_get_response_body_returns_error() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "Some body")));

        let error_message = "some error message";
        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&get_mock_config()).unwrap(),
        );

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);

        let result = sut.refresh_access_token().await;

        let _ = fs::remove_file(token_file_path);
        let _ = fs::remove_file(config_file_path);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            format!("Error in program: {error_message}")
        );

        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_refresh_access_token_deserialization_fails_returns_error() {
        let utils = Utils::new();

        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mock_client = reqwest::Client::new();
        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "Some body")));

        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| Ok(String::from("hello world")));

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&get_mock_config()).unwrap(),
        );

        let sut = Authorization::new(&utils, &mock_http_handler, &mock_client);

        let result = sut.refresh_access_token().await;

        let _ = fs::remove_file(token_file_path);
        let _ = fs::remove_file(config_file_path);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Parse error: expected value at line 1 column 1"
        );
        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_get_key_value_from_url_returns_the_value() {
        let url = Url::parse("http://localhost/callback?code=123&state=abc").unwrap();

        let sut = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.get_key_value_from_url(&url, "code");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "123");
    }

    #[test]
    fn test_get_key_value_from_url_value_does_not_exist_returns_error() {
        let url = Url::parse("http://localhost/callback?code=123&state=abc").unwrap();
        let nonexistent_value = "nonexistent";

        let sut = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.get_key_value_from_url(&url, nonexistent_value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("Error in program: Value for key {nonexistent_value} was not found")
        );
    }

    #[test]
    fn test_get_url_from_stream_success() {
        let request_line = "GET /callback?code=123&state=abc HTTP/1.1\r\n".to_string();
        let mock_stream = create_mock_stream(request_line);

        let authorization = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = authorization.get_url_from_stream(&mock_stream);
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.as_str(), "http://localhost/callback?code=123&state=abc");
    }

    #[test]
    fn test_get_url_from_stream_empty_request_line() {
        let request_line = "\r\n".to_string();
        let mock_stream = create_mock_stream(request_line);

        let authorization = Authorization {
            utils: &MockIUtils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = authorization.get_url_from_stream(&mock_stream);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Error in program: No element has been provided"
        );
    }
}
