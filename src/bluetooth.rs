use std::collections::HashMap;

use btleplug::api::{Central, CentralEvent, Manager as _};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::StreamExt;
use uuid::Uuid;

async fn get_adapter() -> Adapter {
    let manager = Manager::new().await.unwrap();
    get_central(&manager).await
}

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

pub async fn start_bluetooth_scanning(
    process_service_data_advertisement: &dyn Fn(PeripheralId, HashMap<Uuid, Vec<u8>>, &mut Vec<u8>),
) {
    let central = get_adapter().await;

    let mut events = central.events().await.unwrap();

    central
        .start_scan(btleplug::api::ScanFilter::default())
        .await
        .unwrap();
    let mut previous_packet: Vec<u8> = vec![];
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ServiceDataAdvertisement {
                id, service_data, ..
            } => process_service_data_advertisement(id, service_data, &mut previous_packet),
            _ => {}
        }
    }
}
