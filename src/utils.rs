use crate::data_types::MassUnit;

pub fn unit_to_kg(value: f32, unit: MassUnit) -> f32 {
    match unit {
        MassUnit::Jin => value / 1.66667,
        MassUnit::Lbs => value / 2.205,
        MassUnit::Kg => value,
    }
}
