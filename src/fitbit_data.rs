use datetime::LocalDateTime;

use crate::scale_metrics::Gender;
use std::{env::{self, VarError}, string::String};

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

pub fn get_user_data() -> Result<UserData, String> {
    let access_token: Result<String, VarError> = env::var("ACCESS_TOKEN");

    if access_token.is_err() || access_token.unwrap().is_empty() {
        return Err(String::from("AccessToken is empty"));
    }

    // let response = reqwest::get("https://api.fitbit.com/1/user/-/profile.json");

    Ok(UserData {
        gender: Gender::Male,
        age: 23,
        height: 184.0,
        weight: 84.0,
        time_zone: String::from("Europe/Vilnius")
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
