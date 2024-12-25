#[cfg(test)]
mod tests {
    use crate::bluetooth_scanner::BluetoothScanner;
    use crate::bluetooth_scanner::IBluetoothScanner;
    use crate::data_types::config::Config;
    use crate::packet_data_processor::MockIPacketDataProcessor;
    use crate::tests::vector_logger::LOGGER;
    use crate::utils::MockIUtils;

    use btleplug::platform::PeripheralId;
    use serial_test::serial;
    use std::collections::HashMap;
    use uuid::Uuid;

    // #[test]
    // fn test_are_mac_addresses_equal_linux() {
    //     let mut mock_utils = MockIUtils::new();
    //     mock_utils
    //         .expect_read_configuration_file()
    //         .returning(|| Ok(get_mock_config()));

    //     let mock_packet_processor = MockIPacketDataProcessor::new();
    //     let scanner = BluetoothScanner::new(&mock_packet_processor, &mock_utils);

    //     let matching_uuid = Uuid::parse_str("b4565dbf-b956-1234-5678-abcdef123456").unwrap();
    //     let non_matching_uuid = Uuid::parse_str("12345678-1234-5678-1234-abcdef123456").unwrap();

    //     let matching_id = PeripheralId::from(matching_uuid);
    //     let non_matching_id = PeripheralId::from(non_matching_uuid);

    //     // assert!(scanner.are_mac_addresses_equal(&matching_id).unwrap());

    //     // assert!(!scanner.are_mac_addresses_equal(&non_matching_id).unwrap());
    // }

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

        let matching_uuid = Uuid::parse_str("b4565dbf-b956-1234-5678-abcdef123456").unwrap();
        let id = PeripheralId::from(matching_uuid);

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
        assert_eq!(
            logs[0],
            format!(
                "[INFO] Id: {matching_uuid} with UUID: {uuid} for data: {:?}",
                raw_data
            )
        );

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

        let matching_uuid = Uuid::parse_str("b4565dbf-b956-1234-5678-abcdef123456").unwrap();
        let id = PeripheralId::from(matching_uuid);

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
        assert_eq!(
            logs[0],
            format!(
                "[INFO] Id: {matching_uuid} with UUID: {uuid} for data: {:?}",
                raw_data
            )
        );
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

        let matching_uuid = Uuid::parse_str("b4565dbf-b956-1234-5678-abcdef123456").unwrap();
        let id = PeripheralId::from(matching_uuid);

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

    fn get_mock_config() -> Config {
        Config {
            mac_address: String::from("B4:56:5D:BF:B9:56"),
            client_id: String::from("test id"),
            client_secret: String::from("test secret"),
        }
    }

    fn get_packet_raw_data(is_weight_removed: Option<bool>) -> Vec<u8> {
        let is_weight_removed = is_weight_removed.unwrap_or(false);

        vec![
            0b00000001, // is_lbs = true
            0b00100010 | ((is_weight_removed as u8) << 7),
            0xE5,
            0x07, // year = 2021
            0x06, // month = 6
            0x15, // day = 21
            0x0F, // hour = 15
            0x2A, // minutes = 42
            0x10, // seconds = 16
            0x2C,
            0x01, // impedance = 300
            0xE8,
            0x03, // weight = 1000 (in raw format)
        ]
    }
}
