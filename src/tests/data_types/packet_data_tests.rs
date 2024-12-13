#[cfg(test)]
mod tests {
    use crate::data_types::gender::Gender;
    use crate::data_types::mass_unit::MassUnit;
    use crate::data_types::packet_data::PacketData;

    use chrono::TimeZone;
    use chrono::Utc;

    #[test]
    fn test_from_vec_u8() {
        let raw_data: Vec<u8> = vec![
            0b00000001, // is_lbs = true
            0b10100010, // has_impedance = true, is_stabilized = true, is_weight_removed = true
            0xE5, 0x07, // year = 2021
            0x06, // month = 6
            0x15, // day = 21
            0x0F, // hour = 15
            0x2A, // minutes = 42
            0x10, // seconds = 16
            0x2C, 0x01, // impedance = 300
            0xE8, 0x03, // weight = 1000 (in raw format)
        ];

        let packet_data = PacketData::from(&raw_data);

        assert_eq!(packet_data.unit, MassUnit::Lbs);
        assert_eq!(packet_data.has_impedance, true);
        assert_eq!(packet_data.is_stabilized, true);
        assert_eq!(packet_data.is_weight_removed, true);
        assert_eq!(packet_data.impedance, 300);
        assert_eq!(packet_data.weight, 10.0); // 1000 / 100
        assert_eq!(
            packet_data.datetime,
            Utc.with_ymd_and_hms(2021, 6, 21, 15, 42, 16).unwrap()
        );
    }

    #[test]
    fn test_from_vec_u8_unit_jin() {
        let raw_data: Vec<u8> = vec![
            0b00000000, // is_lbs = false
            0b01000000, // is_jin = true
            0xE5, 0x07, // year = 2021
            0x06, // month = 6
            0x15, // day = 21
            0x0F, // hour = 15
            0x2A, // minutes = 42
            0x10, // seconds = 16
            0x01, 0x2C, // impedance = 300
            0xE8, 0x03, // weight = 1000 (in raw format)
        ];

        let packet_data = PacketData::from(&raw_data);

        assert_eq!(packet_data.unit, MassUnit::Jin);
        assert_eq!(packet_data.weight, 10.0); // 1000 / 100
    }

    #[test]
    fn test_from_vec_u8_unit_kg() {
        let raw_data: Vec<u8> = vec![
            0b00000000, // is_lbs = false
            0b00000000, // is_jin = false
            0xE5, 0x07, // year = 2021
            0x06, // month = 6
            0x15, // day = 21
            0x0F, // hour = 15
            0x2A, // minutes = 42
            0x10, // seconds = 16
            0x01, 0x2C, // impedance = 300
            0xE8, 0x03, // weight = 1000 (in raw format)
        ];

        let packet_data = PacketData::from(&raw_data);

        assert_eq!(packet_data.unit, MassUnit::Kg);
        assert_eq!(packet_data.weight, 5.0); // 1000 / 100 / 2 (adjusted for kg)
    }

    #[test]
    fn test_get_fat_percentage_male() {
        let packet_data = PacketData {
            weight: 70.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 500,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Male, 30, 175.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_female() {
        let packet_data = PacketData {
            weight: 60.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 450,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Female, 25, 160.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_female_young() {
        let packet_data = PacketData {
            weight: 55.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 450,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Female, 20, 165.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_female_old() {
        let packet_data = PacketData {
            weight: 55.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 450,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Female, 50, 165.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_male_low_weight() {
        let packet_data = PacketData {
            weight: 50.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 400,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Male, 30, 175.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_high_impedance() {
        let packet_data = PacketData {
            weight: 80.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 700,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage = packet_data.get_fat_percentage(Gender::Male, 40, 180.0);
        assert!(fat_percentage >= 5.0 && fat_percentage <= 75.0);
    }

    #[test]
    fn test_get_fat_percentage_extreme_height() {
        let packet_data = PacketData {
            weight: 75.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 500,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let fat_percentage_tall = packet_data.get_fat_percentage(Gender::Male, 30, 200.0);
        let fat_percentage_short = packet_data.get_fat_percentage(Gender::Male, 30, 140.0);
        assert!(fat_percentage_tall >= 5.0 && fat_percentage_tall <= 75.0);
        assert!(fat_percentage_short >= 5.0 && fat_percentage_short <= 75.0);
    }

    #[test]
    fn test_unit_to_kg() {
        let packet_data_jin = PacketData {
            weight: 10.0,
            unit: MassUnit::Jin,
            has_impedance: false,
            impedance: 0,
            is_stabilized: false,
            is_weight_removed: false,
            datetime: Utc::now(),
        };
        assert!((packet_data_jin.unit_to_kg() - 6.0).abs() < 0.01);

        let packet_data_lbs = PacketData {
            weight: 22.05,
            unit: MassUnit::Lbs,
            has_impedance: false,
            impedance: 0,
            is_stabilized: false,
            is_weight_removed: false,
            datetime: Utc::now(),
        };
        assert!((packet_data_lbs.unit_to_kg() - 10.0).abs() < 0.01);

        let packet_data_kg = PacketData {
            weight: 10.0,
            unit: MassUnit::Kg,
            has_impedance: false,
            impedance: 0,
            is_stabilized: false,
            is_weight_removed: false,
            datetime: Utc::now(),
        };
        assert_eq!(packet_data_kg.unit_to_kg(), 10.0);
    }

    #[test]
    fn test_check_value_overflow() {
        assert_eq!(PacketData::check_value_overflow(4.0, 5.0, 75.0), 5.0);
        assert_eq!(PacketData::check_value_overflow(80.0, 5.0, 75.0), 75.0);
        assert_eq!(PacketData::check_value_overflow(50.0, 5.0, 75.0), 50.0);
    }

    #[test]
    fn test_get_lbm_coefficient() {
        let packet_data = PacketData {
            weight: 70.0,
            unit: MassUnit::Kg,
            has_impedance: true,
            impedance: 500,
            is_stabilized: true,
            is_weight_removed: false,
            datetime: Utc::now(),
        };

        let lbm = packet_data.get_lbm_coefficient(175.0, 30);
        assert!(lbm > 0.0);
    }
}
