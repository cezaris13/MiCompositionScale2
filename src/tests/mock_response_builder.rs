use http::Response as HttpResponse;
use mockall::predicate::*;
use reqwest::{Response, StatusCode};

pub fn mock_response(status: StatusCode, body: &str) -> Response {
    let response = HttpResponse::builder()
        .header("Foo", "Bar")
        .status(status)
        .body(String::from(body))
        .unwrap();
    Response::from(response)
}
