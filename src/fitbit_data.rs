use crate::data_types::{Payload, Token, User, UserData};

use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Error, Response, StatusCode,
};
use serde_json::{from_str, from_value};

use jsonwebtokens::raw::{self, decode_json_token_slice, TokenSlices};

use std::time::{SystemTime, UNIX_EPOCH};

use std::{
    env::{self},
    string::String,
};

// change to async in future
pub async fn get_user_data() -> Result<UserData, String> {
    let mut access_token = retrieve_env_variable("ACCESS_TOKEN")?;

    if is_access_token_expired(&access_token) {
        access_token = refresh_access_token().await?;
    }

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.fitbit.com/1/user/-/profile.json")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;

    let response = handle_http_request(response)?;

    let response_body: String = get_response_body(response).await?;
    let user_data: User = from_str(response_body.as_str()).unwrap();
    Ok(user_data.user)
}

async fn refresh_access_token() -> Result<String, String> {
    let refresh_token = retrieve_env_variable("REFRESH_TOKEN")?;
    let client_id = retrieve_env_variable("CLIENT_ID")?;
    let client_secret = retrieve_env_variable("CLIENT_SECRET")?;

    let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);
    let params = [
        ("refresh_token", refresh_token),
        ("grant_type", String::from("refresh_token")),
    ];
    let url =
        reqwest::Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params).unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .await;

    let response = handle_http_request(response)?;

    let response_body: String = get_response_body(response).await?;
    let token_data: Token = from_str(response_body.as_str()).unwrap();
    save_token_to_env(&token_data);

    Ok(token_data.access_token)
}

pub fn save_token_to_env(_token: &Token) {}
fn is_access_token_expired(access_token: &String) -> bool {
    let TokenSlices { claims, .. } =
        raw::split_token(access_token).expect("Error Slicing the token");
    let raw_claim = decode_json_token_slice(claims).expect("Error getting the claims");
    let final_claim: Payload = from_value(raw_claim.clone()).unwrap();

    let current_time_since_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    current_time_since_unix > final_claim.exp
}

pub async fn retrieve_access_token() -> Result<Token, String> {
    let access_code = retrieve_env_variable("ACCESS_CODE")?;
    let client_id = retrieve_env_variable("CLIENT_ID")?;
    let client_secret = retrieve_env_variable("CLIENT_SECRET")?;

    let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);

    let params = [
        ("grant_type", String::from("authorization_code")),
        ("redirect_uri", String::from("http://127.0.0.1:8080")),
        ("code", access_code),
    ];

    let url =
        reqwest::Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params).unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .await;

    let response = handle_http_request(response)?;
    let response_body: String = get_response_body(response).await?;
    Ok(from_str(&response_body).unwrap())
}

pub async fn update_body_fat(body_fat: f32, datetime: DateTime<Utc>) -> Result<Response, String> {
    let mut access_token = retrieve_env_variable("ACCESS_TOKEN")?;

    if is_access_token_expired(&access_token) {
        access_token = refresh_access_token().await?;
    }

    let params = [
        ("fat", body_fat.to_string()),
        ("date", datetime.format("%Y-%m-%d").to_string()),
        ("time", datetime.format("%H:%M:%S").to_string()),
    ];

    let url = reqwest::Url::parse_with_params(
        "https://api.fitbit.com/1/user/-/body/log/fat.json",
        &params,
    )
    .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;

    handle_http_request(response)
}

pub async fn update_body_weight(
    body_weight: f32,
    datetime: DateTime<Utc>,
) -> Result<Response, String> {
    let mut access_token = retrieve_env_variable("ACCESS_TOKEN")?;

    if is_access_token_expired(&access_token) {
        access_token = refresh_access_token().await?;
    }

    let params = [
        ("weight", body_weight.to_string()),
        ("date", datetime.format("%Y-%m-%d").to_string()),
        ("time", datetime.format("%H:%M:%S").to_string()),
    ];
    let url = reqwest::Url::parse_with_params(
        "https://api.fitbit.com/1/user/-/body/log/weight.json",
        &params,
    )
    .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;

    handle_http_request(response)
}

pub fn retrieve_env_variable(key: &str) -> Result<String, String> {
    println!("{}", key);
    match env::var(key.to_string()) {
        Ok(response) => match response.as_ref() {
            "" => Err(format!("{} is empty", key)),
            _ => Ok(response),
        },
        Err(_) => Err(format!("Failed to find {}", key)),
    }
}

fn handle_http_request(response: Result<Response, Error>) -> Result<Response, String> {
    match response {
        Ok(resp) => match resp.status() {
            StatusCode::OK => Ok(resp),
            status_code => Err(String::from(format!(
                "failed to get data from the request: status code {}",
                status_code
            ))),
        },
        Err(err) => Err(err.to_string()),
    }
}

async fn get_response_body(response: Response) -> Result<String, String> {
    match response.text().await {
        Ok(text) => Ok(text),
        Err(_) => Err(String::from("Failed to retrieve response body")),
    }
}
