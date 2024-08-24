extern crate dotenv;
extern crate dotenv_codegen;

mod data_types;
mod fitbit_data;
mod scale_metrics;
mod utils;

use data_types::{PacketData, UserData};
use fitbit_data::{
    get_user_data, retrieve_access_token, retrieve_env_variable, save_token_to_env,
    update_body_fat, update_body_weight,
};
use log::{info, warn};
use scale_metrics::{get_fat_percentage, process_packet};
use utils::unit_to_kg;

use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use dotenv::dotenv;
use futures::stream::StreamExt;
use std::error::Error;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    env_logger::init();
    match check_if_tokens_exist() {
        Ok(_) => {}
        Err(_) => match retrieve_access_token().await {
            Ok(token) => save_token_to_env(&token),
            Err(err) => {
                println!("{}", err);
            }
        },
    };

    let manager = Manager::new().await?;
    let central = get_central(&manager).await;

    let mut events = central.events().await?;

    central.start_scan(ScanFilter::default()).await?;
    let mut previous_packet: Vec<u8> = vec![];
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ServiceDataAdvertisement { service_data, .. } => {
                // add here mac address check
                let search_str = "181b";
                for (uuid, data) in &service_data {
                    if uuid.to_string().contains(search_str) {
                        if previous_packet == data.clone() {
                            info!("Duplicate data, skipping");
                        } else {
                            previous_packet = data.clone();
                            info!("Found UUID: {uuid} for data: {:?}", data);
                            let processed_packet: PacketData = process_packet(data);
                            if processed_packet.is_stabilized && !processed_packet.is_weight_removed
                            {
                                callback(processed_packet).await;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_if_tokens_exist() -> Result<(), String> {
    retrieve_env_variable("ACCESS_TOKEN")?;
    retrieve_env_variable("REFRESH_TOKEN")?;
    Ok(())
}

async fn callback(processed_packet: PacketData) {
    info!("received data {:?}", processed_packet);
    let weight_in_kg = unit_to_kg(processed_packet.weight, processed_packet.unit);
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
                Ok(_) => info!("Body fat has been updated successfully"),
                Err(err) => warn!("Failed to update body fat: {err}"),
            }
        }
        match update_body_weight(weight_in_kg, processed_packet.datetime).await {
            Ok(_) => info!("Body weight has been updated successfully"),
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
