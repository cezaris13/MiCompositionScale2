mod authorization;
mod bluetooth_scanner;
mod cli_error;
mod data_types;
mod fitbit_api_manager;
mod http_request_handler;
mod packet_data;
mod utils;

use authorization::Authorization;
use bluetooth_scanner::BluetoothScanner;
use data_types::Config;
use utils::Utils;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let bluetooth_scanner = BluetoothScanner::new().await?;
    let authorization = Authorization::new();

    let config: Config = Utils::read_configuration_file()?;

    if !authorization.file_exists()? {
        let client_id: String = config.client_id;
        let client_secret: String = config.client_secret;
        authorization
            .get_auth_token(client_id, client_secret)
            .await?;
    }

    bluetooth_scanner.start_bluetooth_scanning().await?;

    Ok(())
}
