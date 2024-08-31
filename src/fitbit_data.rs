use crate::{
    auth::{read_auth_token, write_auth_token},
    data_types::{Config, Payload, Token, User, UserData},
    utils::get_current_project_directory,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use jsonwebtokens::raw::{self, decode_json_token_slice, TokenSlices};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client, Error, Response, StatusCode, Url,
};
use serde_json::{from_str, from_value};
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

const CONFIG_FILE: &str = "variables.json";

pub async fn get_user_data() -> Result<UserData, String> {
    let access_token = get_access_token().await?;

    let client: Client = Client::new();
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
    let refresh_token: String = read_auth_token().refresh_token;
    // let client_id: String = read_configuration_file()?.client_id;
    let client_id: String = String::from("");
    let client_secret: String = read_configuration_file()?.client_secret;

    let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);
    let params = [
        ("refresh_token", refresh_token),
        ("grant_type", String::from("refresh_token")),
    ];
    let url: Url = Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params).unwrap();

    let client: Client = Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .await;

    let response = handle_http_request(response)?;

    let response_body: String = get_response_body(response).await?;
    let token_data: Token = from_str(response_body.as_str()).unwrap();
    write_auth_token(token_data.clone());

    Ok(token_data.access_token)
}

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

pub async fn update_body_fat(body_fat: f32, datetime: DateTime<Utc>) -> Result<Response, String> {
    let access_token = get_access_token().await?;

    let params = [
        ("fat", body_fat.to_string()),
        ("date", datetime.format("%Y-%m-%d").to_string()),
        ("time", datetime.format("%H:%M:%S").to_string()),
    ];

    let url: Url =
        Url::parse_with_params("https://api.fitbit.com/1/user/-/body/log/fat.json", &params)
            .unwrap();

    let client: Client = Client::new();
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
    let access_token = get_access_token().await?;

    let params = [
        ("weight", body_weight.to_string()),
        ("date", datetime.format("%Y-%m-%d").to_string()),
        ("time", datetime.format("%H:%M:%S").to_string()),
    ];
    let url: Url = Url::parse_with_params(
        "https://api.fitbit.com/1/user/-/body/log/weight.json",
        &params,
    )
    .unwrap();

    let client: Client = Client::new();
    let response: Result<Response, Error> = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await;

    handle_http_request(response)
}

pub fn read_configuration_file() -> Result<Config, String> {
    let config_file: String = get_current_project_directory() + "/" + CONFIG_FILE;
    match std::fs::read_to_string(config_file) {
        Ok(config) => Ok(serde_json::from_str(&config).unwrap()),
        Err(e) => {
            log::error!("Failed to read the config file {}", e);
            Err(e.to_string())
        }
    }
}

fn handle_http_request(response: Result<Response, Error>) -> Result<Response, String> {
    match response {
        Ok(resp) => match resp.status() {
            StatusCode::OK => Ok(resp),
            StatusCode::CREATED => Ok(resp),
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

async fn get_access_token() -> Result<String, String> {
    let access_token = read_auth_token().access_token;

    if is_access_token_expired(&access_token) {
        return refresh_access_token().await;
    }

    Ok(access_token)
}
