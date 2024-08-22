use datetime::LocalDateTime;
use fitbit_data::{get_user_data, update_body_fat, update_body_weight, UserData};
use scale_metrics::get_fat_percentage;
use utils::{unit_to_kg, MassUnit};

mod fitbit_data;
mod scale_metrics;
mod utils;

fn main() {
    println!(
        "{}",
        scale_metrics::get_fat_percentage(
            450 as f32,
            84 as f32,
            scale_metrics::Gender::Male,
            23,
            184 as f32
        )
    );
}

fn callback(
    weight: f32,
    unit: MassUnit,
    has_impedance: bool,
    impedance: f32,
    datetime: LocalDateTime,
) {
    // log info
    let weight_in_kg = unit_to_kg(weight, unit);
    let user_data: UserData = get_user_data();

    if user_data.weight - 3.0 < weight_in_kg && weight_in_kg < user_data.weight + 3.0 {
        if has_impedance {
            let body_fat: f32 = get_fat_percentage(
                impedance,
                weight_in_kg,
                user_data.gender,
                user_data.age,
                user_data.height,
            );
            update_body_fat(body_fat, datetime);
        }
        update_body_weight(weight_in_kg, datetime);
    } else {
        // log warning
    }
}
