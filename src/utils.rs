use crate::cli_error::CliError;

use std::{env, path::PathBuf};

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
