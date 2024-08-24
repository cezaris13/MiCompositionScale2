extern crate dotenv_codegen;

use fitbit_data::{
    get_user_data, update_body_fat, update_body_weight, UserData,
};
use scale_metrics::{get_fat_percentage, process_packet, PacketData};
use utils::unit_to_kg;

mod fitbit_data;
mod scale_metrics;
mod utils;

use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let manager = Manager::new().await?;
    let central = get_central(&manager).await;

    let mut events = central.events().await?;

    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ServiceDataAdvertisement { service_data, .. } => {
                let search_str = "181b";
                for (uuid, data) in &service_data {
                    if uuid.to_string().contains(search_str) {
                        println!("Found UUID: {} for data: {:?}", uuid, data);
                        let processed_packet: PacketData = process_packet(data);
                        println!("{:?}", processed_packet);
                        if processed_packet.is_stabilized && !processed_packet.is_weight_removed {
                            callback(processed_packet);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn callback(processed_packet: PacketData) {
    let weight_in_kg = unit_to_kg(processed_packet.weight, processed_packet.unit);
    let user_data_response: Result<UserData, String> = get_user_data();

    let user_data: UserData = user_data_response.unwrap_or_else(|error| panic!("{}", error));

    if user_data.weight - 3.0 < weight_in_kg && weight_in_kg < user_data.weight + 3.0 {
        if processed_packet.has_impedance {
            let body_fat: f32 = get_fat_percentage(
                processed_packet.impedance,
                weight_in_kg,
                user_data.gender,
                user_data.age,
                user_data.height,
            );
            match update_body_fat(body_fat, processed_packet.datetime) {
                Ok(_) => (),
                Err(err) => println!("{}", err),
            }
        }
        match update_body_weight(weight_in_kg, processed_packet.datetime) {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }
    } else {
        // log warning
    }
}