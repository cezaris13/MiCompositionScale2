use crate::cli_error::CliError;
use crate::data_types::Config;

use serde_json::from_str;
use std::{env, path::PathBuf};

const CONFIG_FILE: &str = "variables.json";

pub struct Utils;

impl Utils {
    pub fn get_current_project_directory() -> Result<String, CliError> {
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

    pub fn read_configuration_file() -> Result<Config, CliError> {
        let config_file: String = Self::get_current_project_directory()? + "/" + CONFIG_FILE;
        match std::fs::read_to_string(config_file) {
            Ok(config) => Ok(from_str(&config)?),
            Err(error) => {
                log::error!("Failed to read the config file {}", error);
                Err(CliError::Error(error.to_string()))
            }
        }
    }
}
