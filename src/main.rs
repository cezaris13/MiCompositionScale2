mod auth;
mod bluetooth;
mod data_types;
mod fitbit_data;
mod scale_metrics;
mod utils;

use auth::{file_exists, get_auth_token};
use bluetooth::start_bluetooth_scanning;
use data_types::{Config, PacketData, UserData};
use fitbit_data::{get_user_data, read_configuration_file, update_body_fat, update_body_weight};
use futures::executor;
use log::{info, warn};
use scale_metrics::{get_fat_percentage, process_packet};
use utils::unit_to_kg;

use btleplug::platform::PeripheralId;
use std::{collections::HashMap, error::Error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let config: Config = read_configuration_file()?;

    if !file_exists() {
        let client_id: String = config.client_id;
        let client_secret: String = config.client_secret;
        get_auth_token(client_id, client_secret).await;
    }

    start_bluetooth_scanning(&process_service_data_advertisement).await;
    Ok(())
}

fn process_service_data_advertisement(
    id: PeripheralId,
    service_data: HashMap<Uuid, Vec<u8>>,
    previous_packet: &mut Vec<u8>,
) {
    let search_str = "181b";
    for (uuid, data) in &service_data {
        // There's only visible mac address in linux (hci0/dev_B4_56_5D_BF_B9_56), on mac os, the id is random guid.
        // Ensuring a bit more security with linux if mac address would not match (some other scales are being used).
        if cfg!(target_os = "linux") {
            let id_in_str = id.to_string();
            let parts: Vec<&str> = id_in_str.split('/').collect();
            if parts.len() > 1 {
                let mac_address = parts[1]
                    .strip_prefix("dev_")
                    .unwrap_or(parts[1])
                    .replace('_', ":");
                if mac_address != read_configuration_file().unwrap().mac_address {
                    continue;
                }
            } else {
                println!("Invalid input format.");
            }
        }

        if uuid.to_string().contains(search_str) {
            if *previous_packet == data.clone() {
                info!("Duplicate data, skipping");
            } else {
                *previous_packet = data.clone();
                info!("Id: {id} with UUID: {uuid} for data: {:?}", data);
                let processed_packet: PacketData = process_packet(data);
                if processed_packet.is_stabilized && !processed_packet.is_weight_removed {
                    executor::block_on(update_fitbit_weight_data(processed_packet));
                }
            }
        }
    }
}

async fn update_fitbit_weight_data(processed_packet: PacketData) {
    info!("received data {:?}", processed_packet);
    let weight_in_kg: f32 = unit_to_kg(processed_packet.weight, processed_packet.unit);
    let user_data: UserData = match get_user_data().await {
        Ok(response) => response,
        Err(error) => {
            warn!("Failed to retrieve user data: {error}");
            return;
        }
    };

    // https://www.healthline.com/health/weight-fluctuation
    // If someone finds more reliable source, create an issue.
    if user_data.weight - 3.0 < weight_in_kg && weight_in_kg < user_data.weight + 3.0 {
        if processed_packet.has_impedance {
            let body_fat: f32 = get_fat_percentage(
                processed_packet.impedance,
                weight_in_kg,
                user_data.gender,
                user_data.age,
                user_data.height,
            );
            match update_body_fat(body_fat, processed_packet.datetime).await {
                Ok(_) => info!("Body fat has been updated successfully!"),
                Err(err) => warn!("Failed to update body fat: {err}"),
            }
        }
        match update_body_weight(weight_in_kg, processed_packet.datetime).await {
            Ok(_) => info!("Body weight has been updated successfully!"),
            Err(err) => warn!("Failed to update body weight: {err}"),
        }
    } else {
        warn!(
            "weight is not between {} and {}, skip publishing",
            user_data.weight - 3.0,
            user_data.weight + 3.0
        )
    }
}
