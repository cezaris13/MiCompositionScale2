use crate::cli_error::CliError;
use crate::data_types::packet_data::PacketData;
use crate::packet_data_processor::IPacketDataProcessor;
use crate::utils::IUtils;

use async_trait::async_trait;
use btleplug::api::{Central, CentralEvent, Manager as _};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::StreamExt;
use log::info;
use std::collections::HashMap;
use std::string::String;
use uuid::Uuid;

#[cfg(test)]
#[path = "./tests/bluetooth_scanner_tests.rs"]
mod tests;

pub struct BluetoothScanner<'a> {
    packet_data_processor: &'a dyn IPacketDataProcessor,
    utils: &'a dyn IUtils,
}

impl<'a> BluetoothScanner<'a> {
    pub fn new(
        packet_data_processor: &'a impl IPacketDataProcessor,
        utils: &'a impl IUtils,
    ) -> Self {
        Self {
            packet_data_processor,
            utils,
        }
    }
}

#[async_trait]
pub trait IBluetoothScanner: Sync {
    async fn start_bluetooth_scanning(&self) -> Result<(), CliError>;
    async fn process_service_data_advertisement(
        &self,
        id: PeripheralId,
        service_data: HashMap<Uuid, Vec<u8>>,
        previous_packet: &mut Vec<u8>,
    ) -> Result<(), CliError>;
    async fn get_adapter(&self) -> Result<Adapter, CliError>;
    fn are_mac_addresses_equal(&self, id: &PeripheralId) -> Result<bool, CliError>;
}

#[async_trait]
impl<'a> IBluetoothScanner for BluetoothScanner<'a> {
    async fn start_bluetooth_scanning(&self) -> Result<(), CliError> {
        let adapter = self.get_adapter().await?;
        let mut events = adapter.events().await?;

        adapter
            .start_scan(btleplug::api::ScanFilter::default())
            .await?;

        let mut previous_packet: Vec<u8> = vec![];
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::ServiceDataAdvertisement {
                    id, service_data, ..
                } => {
                    self.process_service_data_advertisement(id, service_data, &mut previous_packet)
                        .await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn process_service_data_advertisement(
        &self,
        id: PeripheralId,
        service_data: HashMap<Uuid, Vec<u8>>,
        previous_packet: &mut Vec<u8>,
    ) -> Result<(), CliError> {
        let search_str = "181b";
        for (uuid, data) in &service_data {
            if !self.are_mac_addresses_equal(&id)? {
                continue;
            }

            if uuid.to_string().contains(search_str) {
                if previous_packet == data {
                    info!("Duplicate data, skipping");
                    continue;
                }

                *previous_packet = data.to_vec();
                info!("Id: {id} with UUID: {uuid} for data: {:?}", data);
                let processed_packet = PacketData::from(data);
                if processed_packet.is_stabilized && !processed_packet.is_weight_removed {
                    self.packet_data_processor
                        .update_fitbit_weight_data(processed_packet)
                        .await;
                }
            }
        }
        Ok(())
    }

    fn are_mac_addresses_equal(&self, id: &PeripheralId) -> Result<bool, CliError> {
        // There's only visible mac address in linux (hci0/dev_B4_56_5D_BF_B9_56), on macOS, the id is random guid.
        // Ensuring a bit more security with linux if mac address would not match (some other scales are being used).
        if cfg!(target_os = "linux") {
            let id_in_str = id.to_string();
            let parts: Vec<&str> = id_in_str.split('/').collect();
            if parts.len() <= 1 {
                println!("Invalid input format.");
                return Ok(true);
            }

            let mac_address = if let Some(stripped) = parts[1].strip_prefix("dev_") {
                stripped
            } else {
                parts[1]
            }
            .replace('_', ":");

            if mac_address != self.utils.read_configuration_file()?.mac_address {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn get_adapter(&self) -> Result<Adapter, CliError> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;

        match adapters.into_iter().nth(0) {
            Some(adapter) => Ok(adapter),
            None => Err(CliError::Error(String::from("Could not get adapter"))),
        }
    }
}
