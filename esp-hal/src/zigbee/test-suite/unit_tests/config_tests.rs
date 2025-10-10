//! Unit tests for configuration module

#[cfg(test)]
mod tests {
    use crate::zigbee::config::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.channel, 15);
        assert_eq!(config.tx_power, 10);
        assert!(config.security_enabled);
        assert_eq!(config.max_children, 20);
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.stack_profile, 2);
        assert!(config.auto_discovery);
    }

    #[test]
    fn test_coordinator_config() {
        let config = Config::coordinator();
        
        assert!(matches!(config.role, Role::Coordinator));
        assert!(config.security_enabled);
        assert_eq!(config.max_children, 20);
    }

    #[test]
    fn test_router_config() {
        let config = Config::router();
        
        assert!(matches!(config.role, Role::Router));
        assert_eq!(config.max_children, 20);
    }

    #[test]
    fn test_end_device_config() {
        let config_sleepy = Config::end_device(true);
        let config_non_sleepy = Config::end_device(false);
        
        assert!(matches!(config_sleepy.role, Role::EndDevice { sleepy: true }));
        assert!(matches!(config_non_sleepy.role, Role::EndDevice { sleepy: false }));
    }

    #[test]
    fn test_config_builder_channel() {
        let config = Config::default().with_channel(20);
        assert_eq!(config.channel, 20);
    }

    #[test]
    fn test_config_builder_pan_id() {
        let config = Config::default().with_pan_id(0x1234);
        assert_eq!(config.pan_id, Some(0x1234));
    }

    #[test]
    fn test_config_builder_extended_pan_id() {
        let config = Config::default().with_extended_pan_id(0x1122334455667788);
        assert_eq!(config.extended_pan_id, Some(0x1122334455667788));
    }

    #[test]
    fn test_config_builder_tx_power() {
        let config = Config::default().with_tx_power(15);
        assert_eq!(config.tx_power, 15);
    }

    #[test]
    fn test_config_builder_security() {
        let config = Config::default().with_security(SecurityLevel::High);
        assert!(config.security_enabled);
        assert_eq!(config.security_level, SecurityLevel::High);
    }

    #[test]
    fn test_config_builder_max_children() {
        let config = Config::default().with_max_children(50);
        assert_eq!(config.max_children, 50);
    }

    #[test]
    fn test_config_builder_scan_duration() {
        let config = Config::default().with_scan_duration(5);
        assert_eq!(config.scan_duration, 5);
    }

    #[test]
    fn test_config_builder_poll_rate() {
        let config = Config::default().with_poll_rate(500);
        assert_eq!(config.poll_rate_ms, 500);
    }

    #[test]
    fn test_role_is_coordinator() {
        let role = Role::Coordinator;
        assert!(matches!(role, Role::Coordinator));
    }

    #[test]
    fn test_role_is_router() {
        let role = Role::Router;
        assert!(matches!(role, Role::Router));
    }

    #[test]
    fn test_role_is_end_device() {
        let role_sleepy = Role::EndDevice { sleepy: true };
        let role_non_sleepy = Role::EndDevice { sleepy: false };
        
        assert!(matches!(role_sleepy, Role::EndDevice { sleepy: true }));
        assert!(matches!(role_non_sleepy, Role::EndDevice { sleepy: false }));
    }

    #[test]
    fn test_security_level_none() {
        let level = SecurityLevel::None;
        assert!(matches!(level, SecurityLevel::None));
    }

    #[test]
    fn test_security_level_standard() {
        let level = SecurityLevel::Standard;
        assert!(matches!(level, SecurityLevel::Standard));
    }

    #[test]
    fn test_security_level_high() {
        let level = SecurityLevel::High;
        assert!(matches!(level, SecurityLevel::High));
    }

    #[test]
    fn test_channel_mask_default() {
        let mask = ChannelMask::default();
        assert!(mask.is_channel_enabled(15));
    }

    #[test]
    fn test_channel_mask_single_channel() {
        let mask = ChannelMask::single_channel(20);
        assert!(mask.is_channel_enabled(20));
        assert!(!mask.is_channel_enabled(15));
    }

    #[test]
    fn test_channel_mask_all_channels() {
        let mask = ChannelMask::all_channels();
        for channel in 11..=26 {
            assert!(mask.is_channel_enabled(channel));
        }
    }

    #[test]
    fn test_channel_mask_enable_disable() {
        let mut mask = ChannelMask::default();
        mask.enable_channel(25);
        assert!(mask.is_channel_enabled(25));
        
        mask.disable_channel(25);
        assert!(!mask.is_channel_enabled(25));
    }

    #[test]
    fn test_device_type_variants() {
        let types = [
            DeviceType::OnOffLight,
            DeviceType::DimmableLight,
            DeviceType::ColorLight,
            DeviceType::OnOffSwitch,
            DeviceType::DimmerSwitch,
            DeviceType::TemperatureSensor,
            DeviceType::OccupancySensor,
            DeviceType::DoorLockController,
            DeviceType::Thermostat,
            DeviceType::Custom(0x1234),
        ];
        
        for device_type in &types {
            let id = device_type.device_id();
            assert!(id > 0 || matches!(device_type, DeviceType::Custom(_)));
        }
    }

    #[test]
    fn test_endpoint_config_default() {
        let endpoint = EndpointConfig::default();
        assert_eq!(endpoint.endpoint_id, 1);
        assert_eq!(endpoint.profile_id, 0x0104); // Home Automation
    }

    #[test]
    fn test_endpoint_config_builder() {
        let endpoint = EndpointConfig::default()
            .with_endpoint_id(5)
            .with_device_type(DeviceType::TemperatureSensor)
            .with_profile_id(0x0104);
        
        assert_eq!(endpoint.endpoint_id, 5);
        assert_eq!(endpoint.profile_id, 0x0104);
        assert!(matches!(endpoint.device_type, DeviceType::TemperatureSensor));
    }

    #[test]
    fn test_endpoint_config_add_input_cluster() {
        let mut endpoint = EndpointConfig::default();
        endpoint.add_input_cluster(0x0006); // On/Off
        assert!(endpoint.input_clusters.contains(&0x0006));
    }

    #[test]
    fn test_endpoint_config_add_output_cluster() {
        let mut endpoint = EndpointConfig::default();
        endpoint.add_output_cluster(0x0008); // Level Control
        assert!(endpoint.output_clusters.contains(&0x0008));
    }

    #[test]
    fn test_config_chaining() {
        let config = Config::default()
            .with_role(Role::Coordinator)
            .with_channel(20)
            .with_pan_id(0x5678)
            .with_tx_power(15)
            .with_max_children(30);
        
        assert!(matches!(config.role, Role::Coordinator));
        assert_eq!(config.channel, 20);
        assert_eq!(config.pan_id, Some(0x5678));
        assert_eq!(config.tx_power, 15);
        assert_eq!(config.max_children, 30);
    }

    #[test]
    fn test_channel_validation() {
        for channel in 11..=26 {
            assert!(is_valid_channel(channel));
        }
        
        assert!(!is_valid_channel(10));
        assert!(!is_valid_channel(27));
    }
}
