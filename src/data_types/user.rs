use crate::data_types::gender::Gender;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub user: UserData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    pub gender: Gender,
    pub age: i8,
    pub height: f32,
    pub weight: f32,
    #[serde(rename = "timezone")]
    pub time_zone: String,
}
