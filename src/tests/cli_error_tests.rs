#[cfg(test)]
mod tests {
    use crate::cli_error::CliError;
    use crate::data_types::user::User;

    use std::error::Error;
    use std::io;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_display_messages() {
        assert_eq!(
            format!(
                "{}",
                CliError::BluetoothError(btleplug::Error::DeviceNotFound)
            ),
            "Bluetooth error: Device not found"
        );

        let json_error = serde_json::from_str::<User>("some random").err().unwrap();
        let json_error1 = serde_json::from_str::<User>("some random").err().unwrap();
        assert_eq!(
            format!("{}", CliError::ParseError(json_error)),
            format!("Parse error: {json_error1}")
        );

        assert_eq!(
            format!("{}", CliError::Error("Test error".to_string())),
            "Error in program: Test error"
        );

        let oauth_error = oauth2::url::ParseError::RelativeUrlWithoutBase;
        assert_eq!(
            format!("{}", CliError::OAuthError(oauth_error.clone())),
            format!("OAuth error in program: {oauth_error}")
        );

        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let io_error1 = io::Error::new(io::ErrorKind::Other, "IO test error");
        assert_eq!(
            format!("{}", CliError::IOError(io_error)),
            format!("IO error in program: {io_error1}")
        );

        let jwt_error = jsonwebtokens::error::Error::InvalidSignature();
        let jwt_error1 = jsonwebtokens::error::Error::InvalidSignature();
        assert_eq!(
            format!("{}", CliError::JsonWebTokenError(jwt_error)),
            format!("JsonWebToken error: {jwt_error1}")
        );

        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let io_error1 = io::Error::new(io::ErrorKind::Other, "IO test error");
        let opener_error = opener::OpenError::Io(io_error);
        let opener_error1 = opener::OpenError::Io(io_error1);
        assert_eq!(
            format!("{}", CliError::OpenerError(opener_error)),
            format!("Opener error: {opener_error1}")
        );

        let now = SystemTime::now();

        // Create a future time (e.g., 5 seconds from now)
        let future_time = now + Duration::new(5, 0);

        let time_error = now.duration_since(future_time).err().unwrap();
        assert_eq!(
            format!("{}", CliError::SystemTimeError(time_error)),
            "SystemTimeError error: second time provided was later than self"
        );
    }

    #[test]
    fn test_source() {
        // Test the source method for each error type
        let json_error = serde_json::from_str::<User>("some random").err().unwrap();
        let json_error1 = serde_json::from_str::<User>("some random").err().unwrap();
        let cli_error = CliError::ParseError(json_error);
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            json_error1.to_string()
        );

        let btle_error = btleplug::Error::DeviceNotFound;
        let btle_error1 = btleplug::Error::DeviceNotFound;
        let cli_error = CliError::BluetoothError(btle_error);
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            btle_error1.to_string()
        );

        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let io_error1 = io::Error::new(io::ErrorKind::Other, "IO test error");
        let cli_error = CliError::IOError(io_error);
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            io_error1.to_string()
        );

        let now = SystemTime::now();
        let future_time = now + Duration::new(5, 0);
        let system_time_error = now.duration_since(future_time).err().unwrap();
        let cli_error = CliError::SystemTimeError(system_time_error.clone());
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            system_time_error.to_string()
        );

        let oauth_error = oauth2::url::ParseError::RelativeUrlWithoutBase;
        let cli_error = CliError::OAuthError(oauth_error.clone());
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            oauth_error.to_string()
        );

        // Check source for JsonWebTokenError
        let jwt_error = jsonwebtokens::error::Error::InvalidSignature();
        let jwt_error1 = jsonwebtokens::error::Error::InvalidSignature();

        let cli_error = CliError::JsonWebTokenError(jwt_error);
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            jwt_error1.to_string()
        );

        // Check source for OpenerError
        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let opener_error = opener::OpenError::Io(io_error);

        let io_error1 = io::Error::new(io::ErrorKind::Other, "IO test error");
        let opener_error1 = opener::OpenError::Io(io_error1);

        let cli_error = CliError::OpenerError(opener_error);
        assert_eq!(
            cli_error.source().unwrap().to_string(),
            opener_error1.to_string()
        );

        let cli_error = CliError::Error("Test error".to_string());
        assert!(cli_error.source().is_none());
    }

    #[test]
    fn test_from_error_macro() {
        let json_error = serde_json::from_str::<User>("some random").err().unwrap();
        let cli_error: CliError = json_error.into();
        match cli_error {
            CliError::ParseError(err) => {
                assert_eq!(err.to_string(), "expected value at line 1 column 1")
            }
            _ => panic!("Expected CliError::ParseError"),
        }

        let btle_error = btleplug::Error::DeviceNotFound;
        let cli_error: CliError = btle_error.into();
        match cli_error {
            CliError::BluetoothError(err) => assert_eq!(err.to_string(), "Device not found"),
            _ => panic!("Expected CliError::BluetoothError"),
        }

        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let cli_error: CliError = io_error.into();
        match cli_error {
            CliError::IOError(err) => assert_eq!(err.to_string(), "IO test error"),
            _ => panic!("Expected CliError::IOError"),
        }
        let io_error = io::Error::new(io::ErrorKind::Other, "IO test error");
        let opener_error = opener::OpenError::Io(io_error);
        let cli_error: CliError = opener_error.into();
        match cli_error {
            CliError::OpenerError(err) => assert_eq!(err.to_string(), "IO error"),
            _ => panic!("Expected CliError::OpenerError"),
        }
    }
}
