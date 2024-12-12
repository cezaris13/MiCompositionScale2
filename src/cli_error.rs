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
    JsonWebTokenError(jsonwebtokens::error::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BluetoothError(ref err) => write!(f, "Bluetooth error: {}", err),
            Self::ParseError(ref err) => write!(f, "Parse error: {}", err),
            Self::Error(err) => write!(f, "Error in program: {}", err),
            Self::OAuthError(err) => write!(f, "OAuth error in program: {}", err),
            Self::IOError(err) => write!(f, "IO error in program: {}", err),
            Self::JsonWebTokenError(ref err) => write!(f, "JsonWebToken error: {}", err),
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
            Self::JsonWebTokenError(err) => Some(err),
            Self::Error(_) => None,
        }
    }
}
macro_rules! from_error {
    ($source_error:ty, $target_error:ident::$variant:ident) => {
        impl From<$source_error> for $target_error {
            fn from(err: $source_error) -> $target_error {
                $target_error::$variant(err)
            }
        }
    };
}

from_error!(serde_json::Error, CliError::ParseError);
from_error!(btleplug::Error, CliError::BluetoothError);
from_error!(std::string::String, CliError::Error);
from_error!(oauth2::url::ParseError, CliError::OAuthError);
from_error!(std::io::Error, CliError::IOError);
from_error!(jsonwebtokens::error::Error, CliError::JsonWebTokenError);
