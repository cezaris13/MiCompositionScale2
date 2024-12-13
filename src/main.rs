mod authorization;
mod bluetooth_scanner;
mod cli_error;
mod data_types;
mod fitbit_api_manager;
mod http_request_handler;
mod packet_data_processor;
mod utils;

use authorization::{Authorization, IAuthorization};
use bluetooth_scanner::{BluetoothScanner, IBluetoothScanner};
use data_types::config::Config;
use fitbit_api_manager::FitbitApiManager;
use http_request_handler::HttpRequestHandler;
use packet_data_processor::PacketDataProcessor;
use reqwest::Client;
use std::error::Error;
use utils::{IUtils, Utils};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let utils = Utils::new();
    let http_request_handler = HttpRequestHandler::new();
    let http_client: Client = Client::new();

    let authorization = Authorization::new(&utils, &http_request_handler, &http_client);

    let fitbit_api_manager =
        FitbitApiManager::new(&authorization, &http_request_handler, &http_client);

    let packet_data_processor = PacketDataProcessor::new(&fitbit_api_manager);
    let bluetooth_scanner = BluetoothScanner::new(&packet_data_processor, &utils);

    let config: Config = utils.read_configuration_file()?;

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
