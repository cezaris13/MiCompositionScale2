use crate::data_types::config::Config;
use crate::data_types::gender::Gender;
use crate::data_types::mass_unit::MassUnit;
use crate::data_types::packet_data::PacketData;
use crate::data_types::token::Token;
use crate::data_types::user::{User, UserData};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use http::Response as HttpResponse;
use mockall::predicate::*;
use reqwest::{Response, StatusCode};
use serde_json::{json, to_vec};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn test_response(status: StatusCode, body: &str) -> Response {
    let response = HttpResponse::builder()
        .header("Foo", "Bar")
        .status(status)
        .body(String::from(body))
        .unwrap();
    Response::from(response)
}

pub fn test_token(access_token: Option<String>) -> Token {
    let access_token = access_token.unwrap_or(String::from("Some access token"));
    Token {
        access_token,
        refresh_token: String::from("some refresh token"),
    }
}

pub fn test_config() -> Config {
    Config {
        mac_address: String::from("B4:56:5D:BF:B9:56"),
        client_id: String::from("test id"),
        client_secret: String::from("test secret"),
    }
}

pub fn test_packet_raw_data(is_weight_removed: Option<bool>) -> Vec<u8> {
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

pub fn test_user_data_as_string() -> String {
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

pub fn test_packet_data(has_impedance: Option<bool>) -> PacketData {
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

pub fn test_user_data(sample_weight: Option<f32>) -> UserData {
    let weight = sample_weight.unwrap_or_else(|| 70.2);

    UserData {
        gender: Gender::Male,
        age: 30,
        height: 175.5,
        weight,
        time_zone: String::from("UTC+1"),
    }
}

pub fn test_stream(request_line: String) -> TcpStream {
    // Create a listener to simulate a TCP stream
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Spawn a thread to act as a mock client
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream.write_all(request_line.as_bytes()).unwrap();
    });

    listener.accept().unwrap().0
}

pub fn test_token_as_string(exp: u64) -> String {
    let claims = json!({
        "exp": exp,
    });
    let claims_bytes = to_vec(&claims).unwrap();
    let encoded_claims = URL_SAFE_NO_PAD.encode(claims_bytes);

    // Assemble a mock JWT token (header.payload.signature)
    let header = URL_SAFE_NO_PAD.encode(b"{}"); // Empty JSON header
    let signature = "signature"; // Signature can be any placeholder
    format!("{header}.{encoded_claims}.{signature}")
}

pub fn test_tcp_client_get_data(message: String) -> String {
    let mut buffer = [0; 1024];
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");

    stream
        .write_all(message.as_bytes())
        .expect("Failed to write to stream");

    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    buffer
        .into_iter()
        .map(|p| char::from(p))
        .filter(|p| *p != '\0')
        .collect::<String>()
}
