use std::{env, path::PathBuf};

pub fn get_current_project_directory() -> Result<String, String> {
    let mut current_project_path: PathBuf = match env::current_exe() {
        Ok(path) => path,
        Err(e) => return Err(format!("Failed to get current executable path: {}", e)),
    };
    println!("{:?}", current_project_path);
    current_project_path.pop(); // MiCompositionScale2
    current_project_path.pop(); // debug
    current_project_path.pop(); // target
    Ok(current_project_path.into_os_string().to_string_lossy().to_string())
}
