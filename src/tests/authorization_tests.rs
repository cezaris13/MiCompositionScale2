#[cfg(test)]
mod tests {
    use crate::authorization::Authorization;
    use crate::authorization::IAuthorization;
    use crate::authorization::TOKEN_FILE;
    use crate::cli_error::CliError;
    use crate::http_request_handler::MockIHttpRequestHandler;
    use crate::tests::utils::test_data::{
        test_config, test_response, test_stream, test_token, test_token_as_string,
    };
    use crate::tests::utils::vector_logger::LOGGER;
    use crate::utils::MockIUtils;
    use crate::utils::{IUtils, Utils};

    use oauth2::url::Url;
    use oauth2::{AuthorizationCode, CsrfToken};
    use reqwest::StatusCode;
    use serial_test::serial;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[serial]
    fn write_auth_token_invalid_dir_returns_error() {
        let token = test_token(None);

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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

    #[test]
    #[serial]
    fn write_auth_token_writes_token_to_file() {
        let token = test_token(None);

        let utils = Utils::new();

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };
        let result = sut.write_auth_token(token);

        let _ = fs::remove_file(utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn write_auth_token_current_dir_error_returns_error() {
        let error_message = "some error";
        let token = test_token(None);

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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

    #[test]
    #[serial]
    fn file_exists_random_dir_returns_false() {
        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.file_exists();

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    #[serial]
    fn file_exists_get_current_dir_fails_returns_error() {
        let error_message = "some error";

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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
    fn read_auth_token_success() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";

        let utils = Utils::new();
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;

        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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
    fn read_auth_token_file_not_found() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Ok(String::from("/mock/directory")));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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
    fn read_auth_token_invalid_json() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let invalid_token_str = "invalid_token_content";

        let utils = Utils::new();

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), invalid_token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
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
    async fn get_access_token_read_auth_token_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let error_message = "Failed to read auth token";

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_get_current_project_directory()
            .returning(|| Err(CliError::Error(error_message.to_string())));

        let sut = Authorization {
            utils: &mock_utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.get_access_token().await;

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
    async fn get_access_token_token_is_not_expired_returns_token() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();
        let utils = Utils::new();

        let valid_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // Set expiration time 1 hour in the future

        let mock_token = test_token_as_string(valid_time);

        let token_str = format!(
            "{{\"access_token\":\"{mock_token}\", \"refresh_token\":\"mocked_refresh_token\"}}"
        );

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.get_access_token().await;

        let _ = fs::remove_file(token_file_path);

        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), mock_token);

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn get_access_token_token_is_expired_invokes_refresh_token_function() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let utils = Utils::new();

        let valid_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3600; // Set expiration time 1 hour in the past

        let mock_token = test_token_as_string(valid_time);

        let token_str = format!(
            "{{\"access_token\":\"{mock_token}\", \"refresh_token\":\"mocked_refresh_token\"}}"
        );

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

        let result = sut.get_access_token().await;

        let _ = fs::remove_file(token_file_path);

        assert!(result.is_err());
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
    async fn refresh_access_token() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let utils = Utils::new();
        let new_access_token = "new access token";

        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(test_response(StatusCode::OK, "Some body")));

        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| {
                Ok(
                    serde_json::to_string(&test_token(Some(String::from(new_access_token))))
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
            serde_json::to_string(&test_config()).unwrap(),
        );

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &reqwest::Client::new(),
        };

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
    async fn refresh_access_token_fails_to_read_auth_token_returns_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let sut = Authorization {
            utils: &Utils::new(),
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

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
    async fn refresh_access_token_fails_to_read_config_file_returns_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";

        let utils = Utils::new();
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &MockIHttpRequestHandler::new(),
            http_client: &reqwest::Client::new(),
        };

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
    async fn refresh_access_token_fails_to_handle_response_returns_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let error_message = "some error message";

        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";

        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let utils = Utils::new();

        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&test_config()).unwrap(),
        );

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &reqwest::Client::new(),
        };

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
    async fn refresh_access_token_fails_to_get_response_body_returns_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let error_message = "some error message";

        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(test_response(StatusCode::OK, "Some body")));

        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let utils = Utils::new();
        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&test_config()).unwrap(),
        );

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &reqwest::Client::new(),
        };

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
    async fn refresh_access_token_deserialization_fails_returns_error() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mut mock_http_handler = MockIHttpRequestHandler::new();
        mock_http_handler
            .expect_handle_http_request()
            .times(1)
            .returning(|_| Ok(test_response(StatusCode::OK, "Some body")));

        mock_http_handler
            .expect_get_response_body()
            .times(1)
            .returning(move |_| Ok(String::from("hello world")));

        let utils = Utils::new();
        let token_str =
            "{\"access_token\":\"mocked_access_token\",\"refresh_token\":\"mocked_refresh_token\"}";
        let token_file_path = utils.get_current_project_directory().unwrap() + "/" + TOKEN_FILE;
        let _ = fs::write(token_file_path.clone(), token_str);

        let config_file_path = utils.get_current_project_directory().unwrap() + "/variables.json";
        let _ = fs::write(
            config_file_path.clone(),
            serde_json::to_string(&test_config()).unwrap(),
        );

        let sut = Authorization {
            utils: &utils,
            http_request_handler: &mock_http_handler,
            http_client: &reqwest::Client::new(),
        };

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
    fn is_access_token_expired_expired_token() {
        let expired_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 100; // Set expiration time 100 seconds in the past

        let mock_token = test_token_as_string(expired_time);

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
    fn is_access_token_expired_valid_token() {
        let valid_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // Set expiration time 1 hour in the future

        let mock_token = test_token_as_string(valid_time);

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
    fn is_access_token_expired_malformed_token() {
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

    #[test]
    fn get_url_from_stream_success() {
        let request_line = "GET /callback?code=123&state=abc HTTP/1.1\r\n".to_string();
        let mock_stream = test_stream(request_line);

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
    fn get_url_from_stream_empty_request_line() {
        let request_line = "\r\n".to_string();
        let mock_stream = test_stream(request_line);

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

    #[test]
    fn get_key_value_from_url_returns_the_value() {
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
    fn get_key_value_from_url_value_does_not_exist_returns_error() {
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
    #[serial]
    fn get_code_from_tcp_listener_returns_token_and_clients_gets_ok_message() {
        let mock_code = "mock_code";
        let mock_state = "test_csrf_state";

        let csrf_state = CsrfToken::new(mock_state.to_string());
        let result: Arc<Mutex<Option<Result<AuthorizationCode, CliError>>>> =
            Arc::new(Mutex::new(None));

        let data = Arc::clone(&result);
        let server = thread::spawn(move || {
            let sut = Authorization {
                utils: &MockIUtils::new(),
                http_request_handler: &MockIHttpRequestHandler::new(),
                http_client: &reqwest::Client::new(),
            };
            let mut data = data.lock().unwrap();
            *data = Some(sut.get_code_from_tcp_listener(csrf_state));
        });

        thread::sleep(std::time::Duration::from_millis(100));

        let mut buffer = [0; 1024];

        let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");
        let message = format!(
            "GET /path?code={mock_code}&state={mock_state} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"
        );

        stream
            .write_all(message.as_bytes())
            .expect("Failed to write to stream");

        stream
            .read(&mut buffer)
            .expect("Failed to read from stream");

        let data = buffer
            .into_iter()
            .map(|p| char::from(p))
            .filter(|p| *p != '\0')
            .collect::<String>();

        assert_eq!(data, "HTTP/1.1 200 OK\r\ncontent-length: 48\r\n\r\nToken has been retrieved. You may close the tab.");

        let _ = server.join();

        let result = Arc::clone(&result);
        let result = result.lock().unwrap();
        assert!(!result.is_none());
        let result = result.as_ref().unwrap();

        assert!(result.is_ok());
        let code = result.as_ref().unwrap();
        assert_eq!(code.secret(), mock_code);
    }
}
