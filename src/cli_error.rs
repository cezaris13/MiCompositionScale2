use btleplug;
use serde_json;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Error(String),
    ParseError(serde_json::Error),
    BluetoothError(btleplug::Error),
    OAuthError(oauth2::url::ParseError),
    IOError(std::io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::BluetoothError(ref err) => write!(f, "Bluetooth error: {}", err),
            CliError::ParseError(ref err) => write!(f, "Parse error: {}", err),
            CliError::Error(err) => write!(f, "Error in program: {}", err),
            CliError::OAuthError(err) => write!(f, "OAuth error in program: {}", err),
            CliError::IOError(err) => write!(f, "IO error in program: {}", err),
        }
    }
}

impl error::Error for CliError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ParseError(err) => Some(err),
            Self::BluetoothError(err) => Some(err),
            Self::OAuthError(err) => Some(err),
            Self::IOError(err) => Some(err),
            Self::Error(_) => None,
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> CliError {
        CliError::ParseError(err)
    }
}

impl From<btleplug::Error> for CliError {
    fn from(err: btleplug::Error) -> CliError {
        CliError::BluetoothError(err)
    }
}

impl From<std::string::String> for CliError {
    fn from(err: std::string::String) -> CliError {
        CliError::Error(err)
    }
}

impl From<oauth2::url::ParseError> for CliError {
    fn from(err: oauth2::url::ParseError) -> CliError {
        CliError::OAuthError(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::IOError(err)
    }
}
