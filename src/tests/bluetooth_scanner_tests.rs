#[cfg(test)]
mod tests {
    use crate::bluetooth_scanner::BluetoothScanner;
    use crate::bluetooth_scanner::IBluetoothScanner;
    use crate::packet_data_processor::MockIPacketDataProcessor;
    use crate::tests::test_utils::{get_mock_config, get_packet_raw_data};
    use crate::tests::vector_logger::LOGGER;
    use crate::utils::MockIUtils;

    #[cfg(target_os = "linux")]
    use bluez_async::DeviceId;

    use btleplug::platform::PeripheralId;
    use serial_test::serial;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    #[serial]
    async fn test_process_service_data_advertisement() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_read_configuration_file()
            .returning(|| Ok(get_mock_config()));

        let mut mock_packet_processor = MockIPacketDataProcessor::new();
        mock_packet_processor
            .expect_update_fitbit_weight_data()
            .returning(|_| {});

        let mut previous_packet = vec![];
        let mut service_data = HashMap::new();

        let id = get_peripheral_id();

        let uuid = Uuid::parse_str("0000181b-0000-1000-8000-00805f9b34fb").unwrap();

        let raw_data: Vec<u8> = get_packet_raw_data(None);

        service_data.insert(uuid, raw_data.clone());

        let sut = BluetoothScanner::new(&mock_packet_processor, &mock_utils);
        let result = sut
            .process_service_data_advertisement(
                id.clone(),
                service_data.clone(),
                &mut previous_packet,
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(previous_packet, raw_data);
        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 1);
        // assert_eq!(
        //     logs[0],
        //     format!(
        //         "[INFO] Id: {matching_uuid} with UUID: {uuid} for data: {:?}",
        //         raw_data
        //     )
        // );

        let result = sut
            .process_service_data_advertisement(id, service_data, &mut previous_packet)
            .await;
        assert!(result.is_ok());

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[1], String::from("[INFO] Duplicate data, skipping"));
    }

    #[tokio::test]
    #[serial]
    async fn test_process_service_data_advertisement_weight_removed_do_not_update() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_read_configuration_file()
            .returning(|| Ok(get_mock_config()));

        let mock_packet_processor = MockIPacketDataProcessor::new();

        let mut previous_packet = vec![];
        let mut service_data = HashMap::new();

        let id = get_peripheral_id();

        let uuid = Uuid::parse_str("0000181b-0000-1000-8000-00805f9b34fb").unwrap();

        let raw_data: Vec<u8> = get_packet_raw_data(Some(true));

        service_data.insert(uuid, raw_data.clone());

        let sut = BluetoothScanner::new(&mock_packet_processor, &mock_utils);
        let result = sut
            .process_service_data_advertisement(id, service_data.clone(), &mut previous_packet)
            .await;

        assert!(result.is_ok());
        assert_eq!(previous_packet, raw_data);

        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 1);
        // assert_eq!(
        //     logs[0],
        //     format!(
        //         "[INFO] Id: {matching_uuid} with UUID: {uuid} for data: {:?}",
        //         raw_data
        //     )
        // );
    }

    #[tokio::test]
    #[serial]
    async fn test_process_service_data_advertisement_uuid_different_does_not_update() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let mut mock_utils = MockIUtils::new();
        mock_utils
            .expect_read_configuration_file()
            .returning(|| Ok(get_mock_config()));

        let mock_packet_processor = MockIPacketDataProcessor::new();

        let mut previous_packet = vec![];
        let mut service_data = HashMap::new();

        let id = get_peripheral_id();

        let uuid = Uuid::parse_str("0000181c-0000-1000-8000-00805f9b34fb").unwrap();

        let raw_data: Vec<u8> = get_packet_raw_data(None);

        service_data.insert(uuid, raw_data.clone());

        let sut = BluetoothScanner::new(&mock_packet_processor, &mock_utils);
        let result = sut
            .process_service_data_advertisement(id, service_data.clone(), &mut previous_packet)
            .await;

        assert!(result.is_ok());
        assert_eq!(previous_packet, Vec::<u8>::new());

        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 0);
    }

    fn get_peripheral_id() -> PeripheralId {
        #[cfg(target_os = "linux")]
        {
            let device_id = DeviceId::new("/org/bluez/hci0/dev_11_22_33_44_55_66");
            return PeripheralId::from(device_id);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let matching_uuid = Uuid::parse_str("b4565dbf-b956-1234-5678-abcdef123456").unwrap();
            return PeripheralId::from(matching_uuid);
        }
    }
}
