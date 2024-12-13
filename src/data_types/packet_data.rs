use crate::data_types::gender::Gender;
use crate::data_types::mass_unit::MassUnit;

#[cfg(test)]
#[path = "../tests/data_types/packet_data_tests.rs"]
mod tests;

use chrono::{DateTime, LocalResult, TimeZone, Utc};

#[derive(Debug)]
pub struct PacketData {
    pub weight: f32,
    pub unit: MassUnit,
    pub has_impedance: bool,
    pub impedance: u16,
    pub is_stabilized: bool,
    pub is_weight_removed: bool,
    pub datetime: DateTime<Utc>,
}

impl From<&Vec<u8>> for PacketData {
    fn from(raw_data: &Vec<u8>) -> Self {
        let is_lbs: bool = (raw_data[0] & 1) != 0;
        let has_impedance: bool = (raw_data[1] & (1 << 1)) != 0;
        let is_stabilized: bool = (raw_data[1] & (1 << 5)) != 0;
        let is_jin: bool = (raw_data[1] & (1 << 6)) != 0;
        let is_weight_removed: bool = (raw_data[1] & (1 << 7)) != 0;
        let mut weight: f32 = (((raw_data[12] as u16) << 8) | raw_data[11] as u16) as f32 / 100.0;
        let impedance: u16 = ((raw_data[10] as u16) << 8) | raw_data[9] as u16;
        let year: i32 = (((raw_data[3] as u16) << 8) | raw_data[2] as u16).into();
        let month: u32 = raw_data[4].into();
        let day: u32 = raw_data[5].into();
        let hour: u32 = raw_data[6].into();
        let minutes: u32 = raw_data[7].into();
        let seconds: u32 = raw_data[8].into();

        let unit: MassUnit;
        if is_jin {
            unit = MassUnit::Jin;
        } else if is_lbs {
            unit = MassUnit::Lbs;
        } else {
            unit = MassUnit::Kg;
            weight /= 2.0;
        }

        // Safely handle datetime creation
        let datetime = match Utc.with_ymd_and_hms(year, month, day, hour, minutes, seconds) {
            LocalResult::Single(dt) => dt,
            _ => Utc::now(), // Fallback to current time if invalid
        };

        Self {
            weight,
            unit,
            has_impedance,
            impedance,
            is_stabilized,
            is_weight_removed,
            datetime,
        }
    }
}

impl PacketData {
    pub fn get_fat_percentage(&self, gender: Gender, age: i8, height: f32) -> f32 {
        let mut constant: f32 = 0.8;
        if gender == Gender::Female {
            constant = if age <= 49 { 9.25 } else { 7.25 };
        }

        let lbm = self.get_lbm_coefficient(height, age);

        let mut coefficient: f32 = 1.0;

        if gender == Gender::Male && self.weight < 61.0 {
            coefficient = 0.98;
        } else if gender == Gender::Female {
            let multiplier: f32 = if height > 160.0 { 1.03 } else { 1.0 };

            if self.weight > 60.0 {
                coefficient = 0.96 * multiplier;
            } else if self.weight < 50.0 {
                coefficient = 1.02 * multiplier;
            }
        }

        let mut fat_percentage: f32 =
            (1.0 - (((lbm - constant) * coefficient) / self.weight)) * 100.0;

        if fat_percentage > 63.0 {
            fat_percentage = 75.0;
        }

        Self::check_value_overflow(fat_percentage, 5.0, 75.0)
    }

    pub fn unit_to_kg(&self) -> f32 {
        match self.unit {
            MassUnit::Jin => self.weight / 1.66667,
            MassUnit::Lbs => self.weight / 2.205,
            MassUnit::Kg => self.weight,
        }
    }

    fn check_value_overflow(value: f32, minimum: f32, maximum: f32) -> f32 {
        if value < minimum {
            return minimum;
        } else if value > maximum {
            return maximum;
        }
        value
    }

    fn get_lbm_coefficient(&self, height: f32, age: i8) -> f32 {
        let mut lbm: f32 = (height * 9.058 / 100.0) * (height / 100.0);
        lbm += self.weight * 0.32 + 12.226;
        lbm -= self.impedance as f32 * 0.0068;
        lbm -= age as f32 * 0.0542;
        lbm
    }
}
