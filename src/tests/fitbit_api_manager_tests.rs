#[cfg(test)]
mod tests {
    use crate::authorization::MockIAuthorization;
    use crate::cli_error::CliError;
    use crate::data_types::gender::Gender;
    use crate::fitbit_api_manager::FitbitApiManager;
    use crate::fitbit_api_manager::IFitbitApiManager;
    use crate::http_request_handler::MockIHttpRequestHandler;
    use crate::tests::test_utils::{get_mock_response, get_test_user_data_as_string};

    use chrono::Utc;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_get_user_data_success() {
        let mock_client = reqwest::Client::new();

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(|_| Ok(get_test_user_data_as_string()));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_ok());
        let user_data = user_data.unwrap();

        assert_eq!(user_data.gender, Gender::Male);
        assert_eq!(user_data.age, 30);
        assert_eq!(user_data.weight, 70.2);
    }

    #[tokio::test]
    async fn test_get_user_data_bad_fails_to_get_access_token_returns_error() {
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        let error_message = "some error";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
        if let Err(CliError::Error(ref message)) = user_data {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_get_user_data_bad_response_returns_error() {
        let mock_client = reqwest::Client::new();

        let error_message = "some error";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
        if let Err(CliError::Error(ref message)) = user_data {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_get_user_data_bad_response_body_returns_error() {
        let mock_client = reqwest::Client::new();

        let error_message = "some error";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(move |_| Err(CliError::Error(String::from(error_message))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
        if let Err(CliError::Error(ref message)) = user_data {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_get_user_data_wrong_type_of_data_is_received_returns_error() {
        let mock_client = reqwest::Client::new();

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(get_mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(|_| Ok(String::from("random user data")));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        println!("{:?}", user_data);
        assert!(user_data.is_err());
        if let Err(CliError::ParseError(ref message)) = user_data {
            assert_eq!(format!("{message}"), "expected value at line 1 column 1");
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_update_user_body_fat_success() {
        let mock_client = reqwest::Client::new();

        let response_body = "some data";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(get_mock_response(StatusCode::OK, response_body)));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_fat(20.0, Utc::now()).await;

        assert!(response.is_ok());
        let response = response.unwrap();

        assert_eq!(response.text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_update_user_body_fat_authorization_token_retrieval_fails_returns_error() {
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        let error_message = "some error";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_fat(20.0, Utc::now()).await;

        assert!(response.is_err());
        if let Err(CliError::Error(ref message)) = response {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }

    #[tokio::test]
    async fn test_update_user_body_weight_success() {
        let mock_client = reqwest::Client::new();

        let response_body = "some data";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(get_mock_response(StatusCode::OK, response_body)));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_weight(80.0, Utc::now()).await;

        assert!(response.is_ok());
        let response = response.unwrap();

        assert_eq!(response.text().await.unwrap(), response_body);
    }

    #[tokio::test]
    async fn test_update_user_body_weight_authorization_token_retrieval_fails_returns_error() {
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        let error_message = "some error";

        let mut mock_authorization = MockIAuthorization::new();
        mock_authorization
            .expect_get_access_token()
            .returning(move || Err(CliError::Error(String::from(error_message))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_weight(80.0, Utc::now()).await;

        assert!(response.is_err());
        if let Err(CliError::Error(ref message)) = response {
            assert_eq!(message, error_message);
        } else {
            assert!(
                false,
                "Expected error: CliError::Error(\"some error\"), but got a different result"
            );
        }
    }
}
