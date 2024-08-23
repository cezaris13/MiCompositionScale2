use datetime::LocalDateTime;
use json::JsonValue;
use reqwest::{header::AUTHORIZATION, StatusCode};
use std::{os::macos::raw::stat, str::FromStr};

use crate::scale_metrics::Gender;
use std::{
    env::{self, VarError},
    error,
    future::IntoFuture,
    string::String,
};

#[derive(Debug)]
pub struct UserData {
    pub gender: Gender,
    pub age: i8,
    pub height: f32,
    pub weight: f32,
    pub time_zone: String,
}

struct Token {
    access_token: String,
    refresh_token: String,
}

// change to async in future
pub fn get_user_data() -> Result<UserData, String> {
    let access_token = match env::var("ACCESS_TOKEN") {
        Ok(response) => match response.as_ref() {
            "" => return Err(String::from("AccessToken is empty")),
            _ => response,
        },
        Err(error) => return Err(error.to_string()), // to_string is ineficcient
    };

    // https://stackoverflow.com/questions/47570580/is-there-a-shortcut-for-as-ref-unwrapdd
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

    let response_text = response_unwrapped.text();

    let unwrapped_response_text: String = match response_text {
        Ok(text) => text,
        Err(_) => return Err(String::from("Failed to retrieve response message")),
    };

    let response_json = match json::parse(&unwrapped_response_text.to_owned()) {
        Ok(json) => json,
        Err(_) => return Err(String::from("failed to parse json"))
    };

    let user: &JsonValue = &response_json["user"];
    Ok(UserData {
        gender: Gender::from_str(&user["gender"].as_str().unwrap()).unwrap(),
        age: user["age"].as_i8().unwrap(),
        height: user["height"].as_f32().unwrap(),
        weight: user["weight"].as_f32().unwrap(),
        time_zone: String::from(user["timezone"].as_str().unwrap()),
    })
}

fn refresh_access_token() {}

fn is_access_token_expired(access_token: String) -> bool {
    false
}

fn retrieve_access_token() -> Token {
    todo!()
}

pub fn update_body_fat(body_fat: f32, datetime: LocalDateTime) -> bool {
    true
}

pub fn update_body_weight(body_weight: f32, datetime: LocalDateTime) -> bool {
    true
}
