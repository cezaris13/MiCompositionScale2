use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mac_address: String,
    pub client_id: String,
    pub client_secret: String,
}
