use datetime::LocalDateTime;

use crate::scale_metrics::Gender;
use std::string::String;

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

pub fn get_user_data() -> UserData {
    todo!()
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
