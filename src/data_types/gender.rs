use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum Gender {
    #[serde(rename = "MALE")]
    Male,
    #[serde(rename = "FEMALE")]
    Female,
}
