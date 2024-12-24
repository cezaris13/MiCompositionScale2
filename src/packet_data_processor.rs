use crate::data_types::packet_data::PacketData;
use crate::data_types::user::UserData;
use crate::fitbit_api_manager::IFitbitApiManager;

use async_trait::async_trait;
use log::{info, warn};

#[cfg(test)]
#[path = "./tests/packet_data_processor_tests.rs"]
mod tests;

pub struct PacketDataProcessor<'a> {
    fitbit_api_manager: &'a dyn IFitbitApiManager,
}

impl<'a> PacketDataProcessor<'a> {
    pub fn new(fitbit_api_manager: &'a impl IFitbitApiManager) -> Self {
        Self { fitbit_api_manager }
    }
}

#[async_trait]
pub trait IPacketDataProcessor: Sync {
    async fn update_fitbit_weight_data(&self, packet_data: PacketData);
}

#[async_trait]
impl<'a> IPacketDataProcessor for PacketDataProcessor<'a> {
    async fn update_fitbit_weight_data(&self, packet_data: PacketData) {
        info!("Received data {:?}", packet_data);
        let weight_in_kg: f32 = packet_data.unit_to_kg();
        let user_data: UserData = match self.fitbit_api_manager.get_user_data().await {
            Ok(response) => response,
            Err(error) => {
                warn!("Failed to retrieve user data: {error}");
                return;
            }
        };

        // https://www.healthline.com/health/weight-fluctuation
        // If someone finds more reliable source, create an issue.
        if weight_in_kg <= user_data.weight - 3.0 || weight_in_kg >= user_data.weight + 3.0 {
            warn!(
                "weight is not between {} and {}, skip publishing",
                user_data.weight - 3.0,
                user_data.weight + 3.0
            );
            return; // check this
        }

        if packet_data.has_impedance {
            let body_fat: f32 =
                packet_data.get_fat_percentage(user_data.gender, user_data.age, user_data.height);
            match self
                .fitbit_api_manager
                .update_body_fat(body_fat, packet_data.datetime)
                .await
            {
                Ok(_) => info!("Body fat has been updated successfully!"),
                Err(err) => warn!("Failed to update body fat: {err}"),
            }
        }
        match self
            .fitbit_api_manager
            .update_body_weight(weight_in_kg, packet_data.datetime)
            .await
        {
            Ok(_) => info!("Body weight has been updated successfully!"),
            Err(err) => warn!("Failed to update body weight: {err}"),
        }
    }
}
