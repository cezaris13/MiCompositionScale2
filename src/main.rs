mod auth;
mod bluetooth_scanner;
mod data_types;
mod fitbit_data;
mod packet_data;
mod utils;

use auth::{file_exists, get_auth_token};
use bluetooth_scanner::BluetoothScanner;
use data_types::Config;
use fitbit_data::read_configuration_file;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let bluetooth_scanner = BluetoothScanner::new().await?;

    let config: Config = read_configuration_file()?;

    if !file_exists()? {
        let client_id: String = config.client_id;
        let client_secret: String = config.client_secret;
        get_auth_token(client_id, client_secret).await?;
    }

    bluetooth_scanner.start_bluetooth_scanning().await?;

    Ok(())
}
