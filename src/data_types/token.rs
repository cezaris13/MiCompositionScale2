use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub exp: u64,
}
