//! Integration tests for ZCL clusters

#[cfg(test)]
mod tests {
    use crate::zigbee::*;
    use crate::zigbee::test_suite::{mocks::*, helpers::*};

    #[test]
    fn test_on_off_cluster_communication() {
        let mut light_cluster = zcl::OnOffCluster::new();
        let mut switch_cluster = zcl::OnOffCluster::new();
        
        // Switch turns on
        switch_cluster.turn_on().unwrap();
        
        // Simulate command sent to light
        light_cluster.handle_command(0x01, &[]).unwrap(); // ON command
        
        assert!(light_cluster.is_on());
    }

    #[test]
    fn test_level_control_dimming() {
        let mut cluster = zcl::LevelControlCluster::new();
        
        // Set to 50%
        cluster.set_level(128, 0).unwrap();
        assert_eq!(cluster.current_level(), 128);
        
        // Dim up
        cluster.step(zcl::StepMode::Up, 20, 0).unwrap();
        assert_eq!(cluster.current_level(), 148);
        
        // Dim down
        cluster.step(zcl::StepMode::Down, 30, 0).unwrap();
        assert_eq!(cluster.current_level(), 118);
    }

    #[test]
    fn test_temperature_sensor_reading() {
        let mut sensor_cluster = zcl::TemperatureMeasurementCluster::new();
        
        // Set temperature to 23.5°C (2350 in 0.01°C units)
        sensor_cluster.set_temperature(2350);
        
        // Read attribute
        let value = sensor_cluster.read_attribute(0x0000).unwrap();
        assert!(matches!(value, zcl::AttributeValue::Int16(2350)));
    }

    #[test]
    fn test_binding_with_clusters() {
        let mut manager = network::NetworkManager::new();
        
        // Bind switch to light for On/Off cluster
        let binding = network::Binding {
            source_ieee: test_addresses::END_DEVICE_1, // Switch
            source_endpoint: 1,
            cluster_id: zcl::cluster_id::ON_OFF,
            destination: network::BindingDestination::ShortAddress {
                address: 0x0002, // Light
                endpoint: 1,
            },
        };
        
        manager.add_binding(binding).unwrap();
    }

    #[test]
    fn test_cluster_command_handling() {
        let mut cluster = zcl::OnOffCluster::new();
        
        // Test all commands
        cluster.handle_command(0x00, &[]).unwrap(); // OFF
        assert!(!cluster.is_on());
        
        cluster.handle_command(0x01, &[]).unwrap(); // ON
        assert!(cluster.is_on());
        
        cluster.handle_command(0x02, &[]).unwrap(); // TOGGLE
        assert!(!cluster.is_on());
    }

    #[test]
    fn test_attribute_read_write() {
        let mut cluster = zcl::OnOffCluster::new();
        
        // Write attribute
        cluster.write_attribute(0x0000, zcl::AttributeValue::Boolean(true)).unwrap();
        
        // Read attribute
        let value = cluster.read_attribute(0x0000).unwrap();
        assert!(matches!(value, zcl::AttributeValue::Boolean(true)));
    }

    #[test]
    fn test_multiple_endpoints() {
        let mut endpoint1 = config::EndpointConfig::default()
            .with_endpoint_id(1)
            .with_device_type(config::DeviceType::OnOffLight);
        
        let mut endpoint2 = config::EndpointConfig::default()
            .with_endpoint_id(2)
            .with_device_type(config::DeviceType::DimmableLight);
        
        endpoint1.add_input_cluster(zcl::cluster_id::ON_OFF);
        endpoint2.add_input_cluster(zcl::cluster_id::ON_OFF);
        endpoint2.add_input_cluster(zcl::cluster_id::LEVEL_CONTROL);
        
        assert_eq!(endpoint1.input_clusters.len(), 1);
        assert_eq!(endpoint2.input_clusters.len(), 2);
    }

    #[test]
    fn test_cluster_discovery() {
        let simple_desc = zdo::SimpleDescriptor {
            endpoint: 1,
            profile_id: 0x0104, // Home Automation
            device_id: 0x0100,  // On/Off Light
            device_version: 1,
            input_clusters: {
                let mut clusters = heapless::Vec::new();
                clusters.push(zcl::cluster_id::BASIC).ok();
                clusters.push(zcl::cluster_id::ON_OFF).ok();
                clusters
            },
            output_clusters: heapless::Vec::new(),
        };
        
        assert_eq!(simple_desc.input_clusters.len(), 2);
    }

    #[test]
    fn test_zcl_frame_format() {
        let frame = test_zcl_frame(zcl::cluster_id::ON_OFF, 0x01, &[]);
        
        // Frame should have: frame control, transaction seq, command ID
        assert!(frame.len() >= 3);
        assert_eq!(frame[2], 0x01); // Command ID
    }

    #[test]
    fn test_group_cluster_commands() {
        let mut manager = network::NetworkManager::new();
        
        // Add binding to group
        let binding = network::Binding {
            source_ieee: test_addresses::END_DEVICE_1,
            source_endpoint: 1,
            cluster_id: zcl::cluster_id::ON_OFF,
            destination: network::BindingDestination::Group(0x0001),
        };
        
        manager.add_binding(binding).unwrap();
    }

    #[test]
    fn test_level_control_with_transition() {
        let mut cluster = zcl::LevelControlCluster::new();
        
        // Move to level with 1 second transition (10 = 100ms)
        cluster.move_to_level(200, 10).unwrap();
        assert_eq!(cluster.current_level(), 200);
    }

    #[test]
    fn test_temperature_range_validation() {
        let mut cluster = zcl::TemperatureMeasurementCluster::new();
        
        // Valid range: -273.15°C to 327.67°C
        cluster.set_temperature(-27315); // -273.15°C
        assert_eq!(cluster.measured_value(), Some(-27315));
        
        cluster.set_temperature(32767); // 327.67°C
        assert_eq!(cluster.measured_value(), Some(32767));
    }

    #[test]
    fn test_cluster_error_handling() {
        let cluster = zcl::OnOffCluster::new();
        
        // Try to read non-existent attribute
        let result = cluster.read_attribute(0xFFFF);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), zcl::ZclError::UnsupportedAttribute));
    }

    #[test]
    fn test_level_control_boundaries() {
        let mut cluster = zcl::LevelControlCluster::new();
        
        // Set to max
        cluster.set_level(255, 0).unwrap();
        assert_eq!(cluster.current_level(), 255);
        
        // Attempt to exceed max
        cluster.step(zcl::StepMode::Up, 10, 0).unwrap();
        assert_eq!(cluster.current_level(), 255); // Should clamp
        
        // Set to min
        cluster.set_level(0, 0).unwrap();
        assert_eq!(cluster.current_level(), 0);
        
        // Attempt to go below min
        cluster.step(zcl::StepMode::Down, 10, 0).unwrap();
        assert_eq!(cluster.current_level(), 0); // Should clamp
    }
}
