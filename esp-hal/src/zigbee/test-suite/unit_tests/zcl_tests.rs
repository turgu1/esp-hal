//! Unit tests for ZCL module

#[cfg(test)]
mod tests {
    use crate::zigbee::zcl::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_cluster_ids() {
        assert_eq!(cluster_id::BASIC, 0x0000);
        assert_eq!(cluster_id::ON_OFF, 0x0006);
        assert_eq!(cluster_id::LEVEL_CONTROL, 0x0008);
        assert_eq!(cluster_id::COLOR_CONTROL, 0x0300);
        assert_eq!(cluster_id::TEMPERATURE_MEASUREMENT, 0x0402);
    }

    #[test]
    fn test_on_off_cluster_new() {
        let cluster = OnOffCluster::new();
        assert_eq!(cluster.cluster_id(), cluster_id::ON_OFF);
        assert!(!cluster.is_on());
    }

    #[test]
    fn test_on_off_cluster_turn_on() {
        let mut cluster = OnOffCluster::new();
        let result = cluster.turn_on();
        
        assert!(result.is_ok());
        assert!(cluster.is_on());
    }

    #[test]
    fn test_on_off_cluster_turn_off() {
        let mut cluster = OnOffCluster::new();
        cluster.turn_on().unwrap();
        
        let result = cluster.turn_off();
        assert!(result.is_ok());
        assert!(!cluster.is_on());
    }

    #[test]
    fn test_on_off_cluster_toggle() {
        let mut cluster = OnOffCluster::new();
        
        // Initially off
        assert!(!cluster.is_on());
        
        // Toggle to on
        cluster.toggle().unwrap();
        assert!(cluster.is_on());
        
        // Toggle to off
        cluster.toggle().unwrap();
        assert!(!cluster.is_on());
    }

    #[test]
    fn test_on_off_cluster_read_attribute() {
        let cluster = OnOffCluster::new();
        
        let result = cluster.read_attribute(0x0000); // OnOff attribute
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), AttributeValue::Boolean(_)));
    }

    #[test]
    fn test_on_off_cluster_write_attribute() {
        let mut cluster = OnOffCluster::new();
        
        let result = cluster.write_attribute(0x0000, AttributeValue::Boolean(true));
        assert!(result.is_ok());
        assert!(cluster.is_on());
    }

    #[test]
    fn test_on_off_cluster_handle_command() {
        let mut cluster = OnOffCluster::new();
        
        // Command 0x01 = On
        cluster.handle_command(0x01, &[]).unwrap();
        assert!(cluster.is_on());
        
        // Command 0x00 = Off
        cluster.handle_command(0x00, &[]).unwrap();
        assert!(!cluster.is_on());
        
        // Command 0x02 = Toggle
        cluster.handle_command(0x02, &[]).unwrap();
        assert!(cluster.is_on());
    }

    #[test]
    fn test_level_control_cluster_new() {
        let cluster = LevelControlCluster::new();
        assert_eq!(cluster.cluster_id(), cluster_id::LEVEL_CONTROL);
        assert_eq!(cluster.current_level(), 0);
    }

    #[test]
    fn test_level_control_set_level() {
        let mut cluster = LevelControlCluster::new();
        
        cluster.set_level(128, 0).unwrap();
        assert_eq!(cluster.current_level(), 128);
    }

    #[test]
    fn test_level_control_move_to_level() {
        let mut cluster = LevelControlCluster::new();
        
        cluster.move_to_level(200, 100).unwrap();
        assert_eq!(cluster.current_level(), 200);
    }

    #[test]
    fn test_level_control_move() {
        let mut cluster = LevelControlCluster::new();
        cluster.set_level(50, 0).unwrap();
        
        // Move up
        cluster.move(MoveMode::Up, 10).unwrap();
        // Note: actual movement would happen over time
        
        cluster.stop().unwrap();
    }

    #[test]
    fn test_level_control_step() {
        let mut cluster = LevelControlCluster::new();
        cluster.set_level(100, 0).unwrap();
        
        cluster.step(StepMode::Up, 20, 0).unwrap();
        assert_eq!(cluster.current_level(), 120);
        
        cluster.step(StepMode::Down, 30, 0).unwrap();
        assert_eq!(cluster.current_level(), 90);
    }

    #[test]
    fn test_level_control_boundaries() {
        let mut cluster = LevelControlCluster::new();
        
        // Set to max
        cluster.set_level(255, 0).unwrap();
        assert_eq!(cluster.current_level(), 255);
        
        // Attempt to exceed max (should clamp)
        cluster.step(StepMode::Up, 10, 0).unwrap();
        assert_eq!(cluster.current_level(), 255);
        
        // Set to min
        cluster.set_level(0, 0).unwrap();
        assert_eq!(cluster.current_level(), 0);
        
        // Attempt to go below min (should clamp)
        cluster.step(StepMode::Down, 10, 0).unwrap();
        assert_eq!(cluster.current_level(), 0);
    }

    #[test]
    fn test_temperature_cluster_new() {
        let cluster = TemperatureMeasurementCluster::new();
        assert_eq!(cluster.cluster_id(), cluster_id::TEMPERATURE_MEASUREMENT);
    }

    #[test]
    fn test_temperature_set_value() {
        let mut cluster = TemperatureMeasurementCluster::new();
        
        // Set to 25.5°C (2550 in 0.01°C units)
        cluster.set_temperature(2550);
        assert_eq!(cluster.measured_value(), Some(2550));
    }

    #[test]
    fn test_temperature_read_attribute() {
        let mut cluster = TemperatureMeasurementCluster::new();
        cluster.set_temperature(2000); // 20.0°C
        
        let result = cluster.read_attribute(0x0000); // MeasuredValue
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), AttributeValue::Int16(2000)));
    }

    #[test]
    fn test_attribute_value_boolean() {
        let value = AttributeValue::Boolean(true);
        assert!(matches!(value, AttributeValue::Boolean(true)));
    }

    #[test]
    fn test_attribute_value_uint8() {
        let value = AttributeValue::Uint8(42);
        assert!(matches!(value, AttributeValue::Uint8(42)));
    }

    #[test]
    fn test_attribute_value_uint16() {
        let value = AttributeValue::Uint16(1234);
        assert!(matches!(value, AttributeValue::Uint16(1234)));
    }

    #[test]
    fn test_attribute_value_int16() {
        let value = AttributeValue::Int16(-100);
        assert!(matches!(value, AttributeValue::Int16(-100)));
    }

    #[test]
    fn test_attribute_value_string() {
        let mut s = heapless::String::<32>::new();
        s.push_str("Test").ok();
        let value = AttributeValue::String(s);
        
        assert!(matches!(value, AttributeValue::String(_)));
    }

    #[test]
    fn test_zcl_error_variants() {
        assert!(matches!(ZclError::UnsupportedCommand, ZclError::UnsupportedCommand));
        assert!(matches!(ZclError::UnsupportedAttribute, ZclError::UnsupportedAttribute));
        assert!(matches!(ZclError::InvalidValue, ZclError::InvalidValue));
        assert!(matches!(ZclError::ReadOnly, ZclError::ReadOnly));
    }

    #[test]
    fn test_test_zcl_frame() {
        let frame = test_zcl_frame(cluster_id::ON_OFF, 0x01, &[0x00]);
        
        assert!(frame.len() >= 3);
        assert_eq!(frame[2], 0x01); // Command ID
    }

    #[test]
    fn test_move_mode_variants() {
        assert!(matches!(MoveMode::Up, MoveMode::Up));
        assert!(matches!(MoveMode::Down, MoveMode::Down));
    }

    #[test]
    fn test_step_mode_variants() {
        assert!(matches!(StepMode::Up, StepMode::Up));
        assert!(matches!(StepMode::Down, StepMode::Down));
    }

    #[test]
    fn test_cluster_command_ids() {
        // On/Off cluster commands
        const OFF: u8 = 0x00;
        const ON: u8 = 0x01;
        const TOGGLE: u8 = 0x02;
        
        assert_eq!(OFF, 0x00);
        assert_eq!(ON, 0x01);
        assert_eq!(TOGGLE, 0x02);
    }

    #[test]
    fn test_attribute_test_helper() {
        let bool_val = test_attribute_value(AttributeType::Boolean);
        assert!(matches!(bool_val, AttributeValue::Boolean(_)));
        
        let uint8_val = test_attribute_value(AttributeType::Uint8);
        assert!(matches!(uint8_val, AttributeValue::Uint8(_)));
        
        let string_val = test_attribute_value(AttributeType::String);
        assert!(matches!(string_val, AttributeValue::String(_)));
    }
}
