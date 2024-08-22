use crate::utils::MassUnit;

use bytes::Bytes;
use datetime::{LocalDate, LocalDateTime};

#[derive(PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

pub struct PacketData {
    weight: f32,
    unit: MassUnit,
    has_impedance: bool,
    impedance: f32,
    is_stabilized: bool,
    is_weight_removed: bool,
    datetime: LocalDateTime,
}

pub fn process_packet(raw_data: Bytes) -> PacketData {
    let mut weight: f32 = 12.0;
    let is_jin: bool = true;
    let is_lbs: bool = false;

    let unit: MassUnit;
    if is_jin {
        unit = MassUnit::Jin;
    } else if is_lbs {
        unit = MassUnit::Lbs;
    } else {
        unit = MassUnit::Kg;
        weight /= 2.0;
    }

    PacketData {
        weight,
        unit,
        has_impedance: true,
        impedance: 430.0,
        is_stabilized: true,
        is_weight_removed: false,
        datetime: todo!(),
    }
}

pub fn get_fat_percentage(
    impedance: f32,
    weight: f32,
    gender: Gender,
    age: i8,
    height: f32,
) -> f32 {
    let mut constant: f32 = 0.8;
    if gender == Gender::Female {
        constant = if age <= 49 { 9.25 } else { 7.25 };
    }

    let lbm = get_lbm_coefficient(height, weight, impedance, age);

    let mut coefficient: f32 = 1.0;

    if gender == Gender::Male && weight < 61.0 {
        coefficient = 0.98;
    } else if gender == Gender::Female {
        let multiplier: f32 = if height > 160.0 { 1.03 } else { 1.0 };

        if weight > 60.0 {
            coefficient = 0.96 * multiplier;
        } else if weight < 50.0 {
            coefficient = 1.02 * multiplier;
        }
    }

    let mut fat_percentage: f32 = (1.0 - (((lbm - constant) * coefficient) / weight)) * 100.0;

    if fat_percentage > 63.0 {
        fat_percentage = 75.0;
    }

    check_value_overflow(fat_percentage, 5.0, 75.0)
}

pub fn get_lbm_coefficient(height: f32, weight: f32, impedance: f32, age: i8) -> f32 {
    let mut lbm: f32 = (height * 9.058 / 100.0) * (height / 100.0);
    lbm += weight * 0.32 + 12.226;
    lbm -= impedance * 0.0068;
    lbm -= age as f32 * 0.0542;
    lbm
}

pub fn check_value_overflow(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value < minimum {
        return minimum;
    } else if value > maximum {
        return maximum;
    }
    value
}
