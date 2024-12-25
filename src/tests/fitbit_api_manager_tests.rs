#[cfg(test)]
mod tests {
    use crate::authorization::MockIAuthorization;
    use crate::cli_error::CliError;
    use crate::data_types::gender::Gender;
    use crate::data_types::user::{User, UserData};
    use crate::fitbit_api_manager::FitbitApiManager;
    use crate::fitbit_api_manager::IFitbitApiManager;
    use crate::http_request_handler::MockIHttpRequestHandler;
    use crate::tests::mock_response_builder::mock_response;

    use chrono::Utc;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_get_user_data_success() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(|_| Ok(test_user_data()));

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
        let mut mock_authorization = MockIAuthorization::new();
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Err(CliError::Error(String::from("test_token"))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
    }

    #[tokio::test]
    async fn test_get_user_data_bad_response_returns_error() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Err(CliError::Error(String::from("some error"))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
    }

    #[tokio::test]
    async fn test_get_user_data_bad_response_body_returns_error() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(|_| Err(CliError::Error(String::from("some error"))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
    }

    #[tokio::test]
    async fn test_get_user_data_wrong_type_of_data_is_received_returns_error() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(mock_response(StatusCode::OK, "some data")));

        mock_http_request_handler
            .expect_get_response_body()
            .returning(|_| Ok(String::from("random user data")));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let user_data = sut.get_user_data().await;

        assert!(user_data.is_err());
    }

    #[tokio::test]
    async fn test_update_user_body_fat_success() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let response_body = "some data";
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(mock_response(StatusCode::OK, response_body)));

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
        let mut mock_authorization = MockIAuthorization::new();
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Err(CliError::Error(String::from("test_token"))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_fat(20.0, Utc::now()).await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_update_user_body_weight_success() {
        let mut mock_authorization = MockIAuthorization::new();
        let mut mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Ok("test_token".to_string()));

        let response_body = "some data";
        mock_http_request_handler
            .expect_handle_http_request()
            .returning(|_| Ok(mock_response(StatusCode::OK, response_body)));

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
        let mut mock_authorization = MockIAuthorization::new();
        let mock_http_request_handler = MockIHttpRequestHandler::new();
        let mock_client = reqwest::Client::new();

        mock_authorization
            .expect_get_access_token()
            .returning(|| Err(CliError::Error(String::from("test_token"))));

        let sut = FitbitApiManager::new(
            &mock_authorization,
            &mock_http_request_handler,
            &mock_client,
        );

        let response = sut.update_body_weight(80.0, Utc::now()).await;

        assert!(response.is_err());
    }

    fn test_user_data() -> String {
        let user_data = UserData {
            gender: Gender::Male,
            age: 30,
            height: 175.5,
            weight: 70.2,
            time_zone: "America/New_York".to_string(),
        };

        let user = User { user: user_data };

        serde_json::to_string(&user).unwrap()
    }
}
