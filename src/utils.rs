use crate::cli_error::CliError;
use crate::data_types::config::Config;

use serde_json::from_str;
use std::{env, path::PathBuf};

const CONFIG_FILE: &str = "variables.json";

pub struct Utils;

impl Utils {
    pub fn new() -> Self {
        Self {}
    }
}

pub trait IUtils: Sync {
    fn get_current_project_directory(&self) -> Result<String, CliError>;
    fn read_configuration_file(&self) -> Result<Config, CliError>;
}

impl IUtils for Utils {
    fn get_current_project_directory(&self) -> Result<String, CliError> {
        let mut current_project_path: PathBuf = env::current_exe()?;

        println!("{:?}", current_project_path);
        current_project_path.pop(); // MiCompositionScale2
        current_project_path.pop(); // debug
        current_project_path.pop(); // target
        Ok(current_project_path
            .into_os_string()
            .to_string_lossy()
            .to_string())
    }

    fn read_configuration_file(&self) -> Result<Config, CliError> {
        let config_file: String = self.get_current_project_directory()? + "/" + CONFIG_FILE;
        match std::fs::read_to_string(config_file) {
            Ok(config) => Ok(from_str(&config)?),
            Err(error) => {
                log::error!("Failed to read the config file {}", error);
                Err(CliError::Error(error.to_string()))
            }
        }
    }
}
