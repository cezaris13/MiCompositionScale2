use crate::data_types::config::Config;
use crate::data_types::gender::Gender;
use crate::data_types::mass_unit::MassUnit;
use crate::data_types::packet_data::PacketData;
use crate::data_types::token::Token;
use crate::data_types::user::{User, UserData};

use chrono::Utc;
use http::Response as HttpResponse;
use mockall::predicate::*;
use reqwest::{Response, StatusCode};

pub fn get_mock_response(status: StatusCode, body: &str) -> Response {
    let response = HttpResponse::builder()
        .header("Foo", "Bar")
        .status(status)
        .body(String::from(body))
        .unwrap();
    Response::from(response)
}

pub fn get_mock_token(access_token: Option<String>) -> Token {
    let access_token = access_token.unwrap_or(String::from("Some access token"));
    Token {
        access_token,
        refresh_token: String::from("some refresh token"),
    }
}

pub fn get_mock_config() -> Config {
    Config {
        mac_address: String::from("B4:56:5D:BF:B9:56"),
        client_id: String::from("test id"),
        client_secret: String::from("test secret"),
    }
}

pub fn get_packet_raw_data(is_weight_removed: Option<bool>) -> Vec<u8> {
    let is_weight_removed = is_weight_removed.unwrap_or(false);

    vec![
        0b00000001, // is_lbs = true
        0b00100010 | ((is_weight_removed as u8) << 7),
        0xE5,
        0x07, // year = 2021
        0x06, // month = 6
        0x15, // day = 21
        0x0F, // hour = 15
        0x2A, // minutes = 42
        0x10, // seconds = 16
        0x2C,
        0x01, // impedance = 300
        0xE8,
        0x03, // weight = 1000 (in raw format)
    ]
}

pub fn get_test_user_data_as_string() -> String {
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

pub fn get_test_packet_data(has_impedance: Option<bool>) -> PacketData {
    let has_impedance = has_impedance.unwrap_or_else(|| true);

    PacketData {
        weight: 70.0,
        unit: MassUnit::Kg,
        has_impedance,
        impedance: 500,
        is_stabilized: true,
        is_weight_removed: false,
        datetime: Utc::now(),
    }
}

pub fn get_test_user_data(sample_weight: Option<f32>) -> UserData {
    let weight = sample_weight.unwrap_or_else(|| 70.2);

    UserData {
        gender: Gender::Male,
        age: 30,
        height: 175.5,
        weight,
        time_zone: String::from("UTC+1"),
    }
}
