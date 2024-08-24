use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user: UserData,
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
pub struct Payload {
    pub exp: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Gender {
    #[serde(rename = "MALE")]
    Male,
    #[serde(rename = "FEMALE")]
    Female,
}

#[derive(Debug)]
pub struct PacketData {
    pub weight: f32,
    pub unit: MassUnit,
    pub has_impedance: bool,
    pub impedance: u16,
    pub is_stabilized: bool,
    pub is_weight_removed: bool,
    pub datetime: DateTime<Utc>,
}

#[derive(Debug)]
pub enum MassUnit {
    Kg,
    Lbs,
    Jin,
}
