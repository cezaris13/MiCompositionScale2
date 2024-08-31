use std::{env, path::PathBuf};

use crate::data_types::MassUnit;

pub fn unit_to_kg(value: f32, unit: MassUnit) -> f32 {
    match unit {
        MassUnit::Jin => value / 1.66667,
        MassUnit::Lbs => value / 2.205,
        MassUnit::Kg => value,
    }
}

pub fn get_current_project_directory() -> String {
    let mut current_project_path: PathBuf = env::current_exe().unwrap();
    current_project_path.pop(); // MiCompositionScale2
    current_project_path.pop(); // debug
    current_project_path.pop(); // target
    current_project_path.into_os_string().into_string().unwrap()
}