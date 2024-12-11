use crate::packet_data::PacketData;
use crate::read_configuration_file;

use btleplug::api::{Central, CentralEvent, Manager as _};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::StreamExt;
use log::info;
use std::collections::HashMap;
use std::string::String;
use uuid::Uuid;

pub struct BluetoothScanner {
    adapter: Adapter,
}

impl BluetoothScanner {
    pub async fn new() -> Result<Self, String> {
        let adapter = Self::get_adapter().await?;
        Ok(Self { adapter })
    }

    pub async fn start_bluetooth_scanning(&self) -> Result<(), String> {
        let mut events = match self.adapter.events().await {
            Ok(result) => result,
            Err(error) => {
                return Err(String::from(format!(
                    "{}",
                    BluetoothScanner::error_to_string(error)
                )))
            }
        };

        if let Err(error) = self
            .adapter
            .start_scan(btleplug::api::ScanFilter::default())
            .await
        {
            return Err(String::from(format!(
                "{}",
                BluetoothScanner::error_to_string(error)
            )));
        }

        let mut previous_packet: Vec<u8> = vec![];
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::ServiceDataAdvertisement {
                    id, service_data, ..
                } => {
                    Self::process_service_data_advertisement(
                        id,
                        service_data,
                        &mut previous_packet,
                    )
                    .await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn process_service_data_advertisement(
        id: PeripheralId,
        service_data: HashMap<Uuid, Vec<u8>>,
        previous_packet: &mut Vec<u8>,
    ) -> Result<(), String> {
        let search_str = "181b";
        for (uuid, data) in &service_data {
            // There's only visible mac address in linux (hci0/dev_B4_56_5D_BF_B9_56), on mac os, the id is random guid.
            // Ensuring a bit more security with linux if mac address would not match (some other scales are being used).
            if cfg!(target_os = "linux") {
                let id_in_str = id.to_string();
                let parts: Vec<&str> = id_in_str.split('/').collect();
                if parts.len() > 1 {
                    let mac_address = if let Some(stripped) = parts[1].strip_prefix("dev_") {
                        stripped
                    } else {
                        parts[1]
                    }
                    .replace('_', ":");

                    if mac_address != read_configuration_file()?.mac_address {
                        continue;
                    }
                } else {
                    println!("Invalid input format.");
                }
            }

            if uuid.to_string().contains(search_str) {
                if previous_packet == data {
                    info!("Duplicate data, skipping");
                } else {
                    *previous_packet = data.to_vec();
                    info!("Id: {id} with UUID: {uuid} for data: {:?}", data);
                    let processed_packet = PacketData::from(data);
                    if processed_packet.is_stabilized && !processed_packet.is_weight_removed {
                        processed_packet.update_fitbit_weight_data().await;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn error_to_string(error: btleplug::Error) -> String {
        format!("{:?}", error)
    }

    async fn get_adapter() -> Result<Adapter, String> {
        let manager = match Manager::new().await {
            Ok(result) => result,
            Err(error) => {
                return Err(String::from(format!(
                    "{}",
                    BluetoothScanner::error_to_string(error)
                )))
            }
        };
        Self::get_central(&manager).await
    }

    async fn get_central(manager: &Manager) -> Result<Adapter, String> {
        let adapters = match manager.adapters().await {
            Ok(result) => result,
            Err(error) => return Err(format!("{}", BluetoothScanner::error_to_string(error))),
        };

        match adapters.into_iter().nth(0) {
            Some(adapter) => Ok(adapter),
            None => Err(String::from("Could not get adapter")),
        }
    }
}
