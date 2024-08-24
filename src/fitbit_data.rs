use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use reqwest::{
    blocking::Response,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value};

use jsonwebtokens::raw::{self, decode_json_token_slice, TokenSlices};

use std::time::{SystemTime, UNIX_EPOCH};

use std::{
    env::{self},
    string::String,
};

use crate::scale_metrics::Gender;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    user: UserData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub gender: Gender,
    pub age: i8,
    pub height: f32,
    pub weight: f32,
    #[serde(rename = "timezone")]
    pub time_zone: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    exp: u64,
}

// change to async in future
pub fn get_user_data() -> Result<UserData, String> {
    let mut access_token = match env::var("ACCESS_TOKEN") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("ACCESS_TOKEN is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    if is_access_token_expired(&access_token) {
        access_token = match refresh_access_token() {
            Ok(token) => token,
            Err(err) => return Err(err),
        }
    }

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.fitbit.com/1/user/-/profile.json")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send();

    let response_unwrapped = match response {
        Ok(resp) => match resp.status().as_u16() {
            200 => resp, // change to status code?
            status_code => {
                return Err(String::from(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                )))
            }
        },
        Err(err) => return Err(err.to_string()),
    };

    let unwrapped_response_text: String = match response_unwrapped.text() {
        Ok(text) => text,
        Err(_) => return Err(String::from("Failed to retrieve response message")),
    };

    let user_data: User = from_str(unwrapped_response_text.as_str()).unwrap();
    Ok(user_data.user)
}

fn refresh_access_token() -> Result<String, String> {
    let refresh_token = match env::var("REFRESH_TOKEN") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("REFRRESH_TOKEN is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let client_id = match env::var("CLIENT_ID") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("CLIENT_ID is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let client_secret = match env::var("CLIENT_SECRET") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("CLIENT_SECRET is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);
    let params = [
        ("refresh_token", refresh_token),
        ("grant_type", String::from("refresh_token")),
    ];
    let url =
        reqwest::Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params).unwrap();

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send();

    let response_unwrapped = match response {
        Ok(resp) => match resp.status().as_u16() {
            200 => resp, // change to status code?
            status_code => {
                return Err(String::from(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                )))
            }
        },
        Err(err) => return Err(err.to_string()),
    };

    let unwrapped_response_text: String = match response_unwrapped.text() {
        Ok(text) => text,
        Err(_) => return Err(String::from("Failed to retrieve response message")),
    };

    let response: Token = from_str(unwrapped_response_text.as_str()).unwrap();
    println!("{:?}", response);

    // // add writing to .env
    Ok(response.access_token)
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

pub fn retrieve_access_token() -> Result<Token, String> {
    let access_code = match env::var("ACCESS_CODE") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("ACCESS_CODE is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let client_id = match env::var("CLIENT_ID") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("CLIENT_ID is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let client_secret = match env::var("CLIENT_SECRET") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("CLIENT_SECRET is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    let encoded_client_data = BASE64_STANDARD.encode(client_id + ":" + &client_secret);

    let params = [
        ("grant_type", String::from("authorization_code")),
        ("redirect_uri", String::from("http://127.0.0.1:8080")),
        ("code", access_code),
    ];

    let url =
        reqwest::Url::parse_with_params("https://api.fitbit.com/oauth2/token", &params).unwrap();

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Basic {}", encoded_client_data))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send();

    let response_unwrapped = match response {
        Ok(resp) => match resp.status().as_u16() {
            200 => resp,
            status_code => {
                return Err(String::from(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                )))
            }
        },
        Err(err) => return Err(err.to_string()),
    };

    let unwrapped_response_text: String = match response_unwrapped.text() {
        Ok(text) => text,
        Err(_) => return Err(String::from("Failed to retrieve response message")),
    };
    Ok(from_str(&unwrapped_response_text).unwrap())
}

pub fn update_body_fat(body_fat: f32, datetime: DateTime<Utc>) -> Result<Response, String> {
    let mut access_token = match env::var("ACCESS_TOKEN") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("AccessToken is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    if is_access_token_expired(&access_token) {
        access_token = match refresh_access_token() {
            Ok(token) => token,
            Err(err) => return Err(err),
        }
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

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send();

    match response {
        Ok(resp) => match resp.status().as_u16() {
            200 => Ok(resp), // change to status code?
            status_code => {
                return Err(String::from(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                )))
            }
        },
        Err(err) => return Err(err.to_string()),
    }
}

pub fn update_body_weight(body_weight: f32, datetime: DateTime<Utc>) -> Result<Response, String> {
    let mut access_token = match env::var("ACCESS_TOKEN") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("AccessToken is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()),
    };

    if is_access_token_expired(&access_token) {
        access_token = match refresh_access_token() {
            Ok(token) => token,
            Err(err) => return Err(err),
        }
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

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send();

    match response {
        Ok(resp) => match resp.status().as_u16() {
            200 => Ok(resp), // change to status code?
            status_code => {
                return Err(String::from(format!(
                    "failed to get data from the request: status code {}",
                    status_code
                )))
            }
        },
        Err(err) => return Err(err.to_string()),
    }
}
