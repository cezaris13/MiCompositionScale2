#[cfg(test)]
mod tests {
    use crate::authorization::MockIAuthorization;
    use crate::fitbit_api_manager::{FitbitApiManager, IFitbitApiManager};
    use crate::http_request_handler::MockIHttpRequestHandler;
    use chrono::{TimeZone, Utc};
    use reqwest_mock::{Method, StubClient, StubDefault, StubSettings, StubStrictness, Url};

    #[test]
    async fn test_get_user_data_success() {
        let mut mock_auth = MockIAuthorization::new();
        let mut mock_handler = MockIHttpRequestHandler::new();
        // let mut mock_client = StubClient::new(StubSettings {
        //     // If a request without a corresponding stub is made we want an error
        //     // to be returned when our code executes the request.
        //     default: StubDefault::Error,

        //     // We want the `StubClient` to compare actual requests and provided
        //     // mocks by their method and their url.
        //     strictness: StubStrictness::MethodUrl,
        // });

        // // Mock a request.
        // mock_client
        //     .stub(Url::parse("http://example.com/mocking").unwrap())
        //     .method(Method::GET)
        //     .response()
        //     .body("Mocking is fun!")
        //     .mock();

        // mock_auth
        //     .expect_get_access_token()
        //     .returning(|| Ok("test".to_string()));

        // mock_handler
        //     .expect_handle_http_request()
        //     .returning(|response| Ok(response.unwrap()));

        // let datetime = Utc.with_ymd_and_hms(2023, 12, 31, 12, 0, 0).unwrap();
        // let fitbit_api_manager = FitbitApiManager::new(&mock_auth, &mock_handler, &mock_client);

        // let result = fitbit_api_manager.update_body_weight(75.0, datetime).await;

        // assert!(result.is_ok());
    }
}
