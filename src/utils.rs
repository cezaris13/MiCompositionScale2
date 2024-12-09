use std::{env, path::PathBuf};

pub fn get_current_project_directory() -> String {
    let mut current_project_path: PathBuf = env::current_exe().unwrap();
    println!("{:?}", current_project_path);
    current_project_path.pop(); // MiCompositionScale2
    current_project_path.pop(); // debug
    current_project_path.pop(); // target
    current_project_path.into_os_string().into_string().unwrap()
}
