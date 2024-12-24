#[cfg(test)]
mod tests {
    use crate::cli_error::CliError;
    use crate::data_types::gender::Gender;
    use crate::data_types::mass_unit::MassUnit;
    use crate::data_types::packet_data::PacketData;
    use crate::data_types::user::UserData;
    use crate::fitbit_api_manager::MockIFitbitApiManager;
    use crate::packet_data_processor::IPacketDataProcessor;
    use crate::tests::vector_logger::VectorLogger;
    use crate::PacketDataProcessor;

    use chrono::Utc;
    use log::LevelFilter;
    use once_cell::sync::Lazy;
    use reqwest::Response;
    use serial_test::serial;
    use std::sync::Arc;

    // Define a static logger that will be initialized once.
    static LOGGER: Lazy<Arc<VectorLogger>> = Lazy::new(|| {
        let logger = VectorLogger::new();
        let logger_ref = Arc::new(logger);
        log::set_boxed_logger(Box::new(logger_ref.clone())).unwrap();
        log::set_max_level(LevelFilter::Info);
        logger_ref
    });

    // Test cases can now use this static logger
    #[tokio::test]
    #[serial]
    async fn test_update_body_data() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let user_data = get_test_user_data(None);
        let packet_data = get_test_packet_data(None);

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        mock_fitbit_api_manager
            .expect_update_body_fat()
            .times(1)
            .returning(move |_, _| {
                return Ok(Response::from(http::Response::new("body text")));
            });

        mock_fitbit_api_manager
            .expect_update_body_weight()
            .times(1)
            .returning(move |_, _| {
                return Ok(Response::from(http::Response::new("body text")));
            });

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("Received data"));
        assert_eq!(logs[1], "[INFO] Body fat has been updated successfully!");
        assert_eq!(logs[2], "[INFO] Body weight has been updated successfully!");
    }

    #[tokio::test]
    #[serial]
    async fn test_no_impedance_does_not_update_body_fat() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone();

        let user_data = get_test_user_data(None);
        let packet_data = get_test_packet_data(Some(false));

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        mock_fitbit_api_manager
            .expect_update_body_weight()
            .times(1)
            .returning(move |_, _| {
                return Ok(Response::from(http::Response::new("body text")));
            });

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Received data"));
        assert_eq!(logs[1], "[INFO] Body weight has been updated successfully!");
    }

    #[tokio::test]
    #[serial]
    async fn test_weight_is_bigger_than_3_kg_does_not_update() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone(); // Use the global logger

        let test_weight = 100.0;
        let user_data = get_test_user_data(Some(test_weight));
        let packet_data = get_test_packet_data(None);

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();

        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Received data"));
        assert_eq!(
            logs[1],
            format!(
                "[WARN] weight is not between {} and {}, skip publishing",
                test_weight - 3.0,
                test_weight + 3.0
            )
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_weight_is_smaller_than_3_kg_does_not_update() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone(); // Use the global logger

        let test_weight = 50.0;
        let user_data = get_test_user_data(Some(test_weight));
        let packet_data = get_test_packet_data(None);

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Received data"));
        assert_eq!(
            logs[1],
            format!(
                "[WARN] weight is not between {} and {}, skip publishing",
                test_weight - 3.0,
                test_weight + 3.0
            )
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_api_manager_fails_to_get_user_data_returns_warnings() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone(); // Use the global logger

        let packet_data = get_test_packet_data(None);

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        let error = "Some error";
        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Err(CliError::Error(String::from(error))));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Received data"));
        assert_eq!(
            logs[1],
            format!("[WARN] Failed to retrieve user data: Error in program: {error}")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_api_manager_fails_update_body_data_returns_warnings() {
        LOGGER.clear_logs();
        let logger_ref = LOGGER.clone(); // Use the global logger

        let packet_data = get_test_packet_data(None);
        let user_data = get_test_user_data(None);

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        let error = "Some error";

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        mock_fitbit_api_manager
            .expect_update_body_fat()
            .times(1)
            .returning(move |_, _| {
                return Err(CliError::Error(String::from(error)));
            });

        mock_fitbit_api_manager
            .expect_update_body_weight()
            .times(1)
            .returning(move |_, _| {
                return Err(CliError::Error(String::from(error)));
            });

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;

        // Check logs
        let logs = logger_ref.get_logs();
        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("Received data"));
        assert_eq!(
            logs[1],
            format!("[WARN] Failed to update body fat: Error in program: {error}")
        );
        assert_eq!(
            logs[2],
            format!("[WARN] Failed to update body weight: Error in program: {error}")
        );
    }

    fn get_test_packet_data(has_impedance: Option<bool>) -> PacketData {
        let has_impedance = has_impedance.unwrap_or_else(|| true);

        PacketData {
            weight: 70.0,
            unit: MassUnit::Kg,
            has_impedance,
            impedance: 500,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        }
    }

    fn get_test_user_data(sample_weight: Option<f32>) -> UserData {
        let weight = sample_weight.unwrap_or_else(|| 70.2);

        UserData {
            gender: Gender::Male,
            age: 30,
            height: 175.5,
            weight,
            time_zone: String::from("UTC+1"),
        }
    }
}
