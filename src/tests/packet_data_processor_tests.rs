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
    use log::{info, warn};
    use reqwest::Response;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_body_data() {
        let user_data = get_test_user_data(None);
        let packet_data = get_test_packet_data();

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
    }

    #[tokio::test]
    async fn test_weight_is_bigger_than_3_kg_does_not_update() {
        let user_data = get_test_user_data(Some(100.0));
        let packet_data = get_test_packet_data();

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;
    }

    #[tokio::test]
    async fn test_weight_is_smalled_than_3_kg_does_not_update() {
        let user_data = get_test_user_data(Some(50.0));
        let packet_data = get_test_packet_data();

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;
    }

    #[tokio::test]
    async fn test_api_manager_fails_to_get_user_data_returns_warnings() {
        let packet_data = get_test_packet_data();

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Err(CliError::Error(String::from("Some error"))));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;
    }

    #[tokio::test]
    async fn test_api_manager_fails_to_update_body_data_returns_warnings() {
        let user_data = get_test_user_data(None);
        let packet_data = get_test_packet_data();

        let mut mock_fitbit_api_manager = MockIFitbitApiManager::new();

        mock_fitbit_api_manager
            .expect_get_user_data()
            .times(1)
            .returning(move || Ok(user_data.clone()));

        mock_fitbit_api_manager
            .expect_update_body_fat()
            .times(1)
            .returning(move |_, _| Err(CliError::Error(String::from("some error"))));

        mock_fitbit_api_manager
            .expect_update_body_weight()
            .times(1)
            .returning(move |_, _| Err(CliError::Error(String::from("some error"))));

        let sut = PacketDataProcessor::new(&mock_fitbit_api_manager);

        sut.update_fitbit_weight_data(packet_data).await;
    }

    #[tokio::test]
    async fn test() {
        // Create and set the logger
        let logger = VectorLogger::new();
        let logger_ref = Arc::new(logger);
        log::set_boxed_logger(Box::new(logger_ref.clone())).unwrap();
        log::set_max_level(LevelFilter::Info);

        // Use log macros
        info!("This is an info message");
        warn!("This is a warning message");

        // Retrieve logs
        let logs = logger_ref.get_logs();
        for log in logs {
            println!("{}", log);
        }
    }

    fn get_test_packet_data() -> PacketData {
        PacketData {
            weight: 70.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 500,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        }
    }

    fn get_test_user_data(sample_weight: Option<f32>) -> UserData {
        let weight = match sample_weight {
            Some(weight) => weight,
            None => 70.2,
        };

        UserData {
            gender: Gender::Male,
            age: 30,
            height: 175.5,
            weight,
            time_zone: String::from("UTC+1"),
        }
    }
}
