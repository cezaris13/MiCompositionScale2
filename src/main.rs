extern crate dotenv_codegen;

use chrono::{DateTime, Utc};
use dotenv::dotenv;
use fitbit_data::{
    get_user_data, retrieve_access_token, update_body_fat, update_body_weight, UserData,
};
use scale_metrics::get_fat_percentage;
use utils::{unit_to_kg, MassUnit};

mod fitbit_data;
mod scale_metrics;
mod utils;

// #[tokio::main]
fn main() {
    dotenv().ok();
    // println!("{:?}",is_access_token_expired(&(env::var("ACCESS_TOKEN").unwrap())));
    println!("{:?}", get_user_data());
    println!("{:?}", retrieve_access_token());
    // let mut updated_tokens = match env::var("ACCESS_TOKEN") {
    //     Ok(response) => match response.as_ref() {
    //         "" => retrieve_access_token(),
    //         _ => {Ok(Token { access_token: "".to_owned(), refresh_token: "".to_owned()})}, // figure out this place
    //     },
    //     Err(_error) => retrieve_access_token(),
    // };

    // for (key, value) in env::vars() {
    //     println!("{}: {}", key, value);
    //     // if key == "ACCESS_TOKEN" {
    //     //     unsafe {
    //     //         env::set_var(key, "123");
    //     //     }
    //     // }
    // }
}

fn callback(
    weight: f32,
    unit: MassUnit,
    has_impedance: bool,
    impedance: f32,
    datetime: DateTime<Utc>,
) {
    // log info
    let weight_in_kg = unit_to_kg(weight, unit);
    let user_data_response: Result<UserData, String> = get_user_data();

    let user_data: UserData = user_data_response.unwrap_or_else(|error| panic!("{}", error));

    if user_data.weight - 3.0 < weight_in_kg && weight_in_kg < user_data.weight + 3.0 {
        if has_impedance {
            let body_fat: f32 = get_fat_percentage(
                impedance,
                weight_in_kg,
                user_data.gender,
                user_data.age,
                user_data.height,
            );
            match update_body_fat(body_fat, datetime) {
                Ok(_) => (),
                Err(err) => println!("{}", err),
            }
        }
        match update_body_weight(weight_in_kg, datetime) {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }
    } else {
        // log warning
    }
}
