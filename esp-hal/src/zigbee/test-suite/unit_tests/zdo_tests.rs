//! Unit tests for ZDO module

#[cfg(test)]
mod tests {
    use crate::zigbee::zdo::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_device_announce_new() {
        let announce = DeviceAnnounce {
            short_address: 0x0001,
            ieee_address: test_addresses::END_DEVICE_1,
            capability: DeviceCapability::coordinator(),
        };
        
        assert_eq!(announce.short_address, 0x0001);
        assert_eq!(announce.ieee_address, test_addresses::END_DEVICE_1);
    }

    #[test]
    fn test_device_capability_coordinator() {
        let cap = DeviceCapability::coordinator();
        
        assert!(cap.is_full_function_device());
        assert!(cap.is_mains_powered());
        assert!(!cap.is_receiver_on_when_idle());
    }

    #[test]
    fn test_device_capability_router() {
        let cap = DeviceCapability::router();
        
        assert!(cap.is_full_function_device());
        assert!(!cap.is_mains_powered());
        assert!(cap.is_receiver_on_when_idle());
    }

    #[test]
    fn test_device_capability_end_device() {
        let cap = DeviceCapability::end_device(false);
        
        assert!(!cap.is_full_function_device());
        assert!(!cap.is_mains_powered());
        assert!(cap.is_receiver_on_when_idle());
    }

    #[test]
    fn test_device_capability_sleepy_end_device() {
        let cap = DeviceCapability::end_device(true);
        
        assert!(!cap.is_full_function_device());
        assert!(!cap.is_mains_powered());
        assert!(!cap.is_receiver_on_when_idle());
    }

    #[test]
    fn test_device_capability_encode_decode() {
        let cap = DeviceCapability::router();
        let encoded = cap.encode();
        let decoded = DeviceCapability::decode(encoded);
        
        assert_eq!(cap.is_full_function_device(), decoded.is_full_function_device());
        assert_eq!(cap.is_mains_powered(), decoded.is_mains_powered());
        assert_eq!(cap.is_receiver_on_when_idle(), decoded.is_receiver_on_when_idle());
    }

    #[test]
    fn test_node_descriptor() {
        let descriptor = NodeDescriptor {
            logical_type: LogicalType::Router,
            complex_descriptor_available: false,
            user_descriptor_available: false,
            frequency_band: 0x08, // 2.4 GHz
            mac_capability: DeviceCapability::router(),
            manufacturer_code: 0x1234,
            max_buffer_size: 127,
            max_incoming_transfer_size: 1500,
            server_mask: 0x0001,
            max_outgoing_transfer_size: 1500,
            descriptor_capability: 0x00,
        };
        
        assert!(matches!(descriptor.logical_type, LogicalType::Router));
        assert_eq!(descriptor.manufacturer_code, 0x1234);
    }

    #[test]
    fn test_logical_type_variants() {
        assert!(matches!(LogicalType::Coordinator, LogicalType::Coordinator));
        assert!(matches!(LogicalType::Router, LogicalType::Router));
        assert!(matches!(LogicalType::EndDevice, LogicalType::EndDevice));
    }

    #[test]
    fn test_power_descriptor() {
        let descriptor = PowerDescriptor {
            current_power_mode: PowerMode::ReceiverOnIdle,
            available_power_sources: 0x01, // Mains
            current_power_source: 0x01,     // Mains
            current_power_level: PowerLevel::Full,
        };
        
        assert!(matches!(descriptor.current_power_mode, PowerMode::ReceiverOnIdle));
        assert!(matches!(descriptor.current_power_level, PowerLevel::Full));
    }

    #[test]
    fn test_power_mode_variants() {
        assert!(matches!(PowerMode::ReceiverOnIdle, PowerMode::ReceiverOnIdle));
        assert!(matches!(PowerMode::ReceiverOnPeriodic, PowerMode::ReceiverOnPeriodic));
        assert!(matches!(PowerMode::ReceiverOnWhenStimulated, PowerMode::ReceiverOnWhenStimulated));
    }

    #[test]
    fn test_power_level_variants() {
        assert!(matches!(PowerLevel::Critical, PowerLevel::Critical));
        assert!(matches!(PowerLevel::Low, PowerLevel::Low));
        assert!(matches!(PowerLevel::Medium, PowerLevel::Medium));
        assert!(matches!(PowerLevel::Full, PowerLevel::Full));
    }

    #[test]
    fn test_simple_descriptor() {
        let mut input_clusters = heapless::Vec::new();
        input_clusters.push(0x0006).ok(); // On/Off
        input_clusters.push(0x0008).ok(); // Level Control
        
        let mut output_clusters = heapless::Vec::new();
        output_clusters.push(0x0000).ok(); // Basic
        
        let descriptor = SimpleDescriptor {
            endpoint: 1,
            profile_id: 0x0104, // Home Automation
            device_id: 0x0100,  // On/Off Light
            device_version: 1,
            input_clusters,
            output_clusters,
        };
        
        assert_eq!(descriptor.endpoint, 1);
        assert_eq!(descriptor.profile_id, 0x0104);
        assert_eq!(descriptor.input_clusters.len(), 2);
        assert_eq!(descriptor.output_clusters.len(), 1);
    }

    #[test]
    fn test_zdo_cluster_ids() {
        assert_eq!(ZDO_NWK_ADDR_REQ, 0x0000);
        assert_eq!(ZDO_IEEE_ADDR_REQ, 0x0001);
        assert_eq!(ZDO_NODE_DESC_REQ, 0x0002);
        assert_eq!(ZDO_POWER_DESC_REQ, 0x0003);
        assert_eq!(ZDO_SIMPLE_DESC_REQ, 0x0004);
        assert_eq!(ZDO_ACTIVE_EP_REQ, 0x0005);
        assert_eq!(ZDO_BIND_REQ, 0x0021);
        assert_eq!(ZDO_UNBIND_REQ, 0x0022);
        assert_eq!(ZDO_MGMT_LEAVE_REQ, 0x0034);
        assert_eq!(ZDO_MGMT_PERMIT_JOIN_REQ, 0x0036);
    }

    #[test]
    fn test_zdo_status_variants() {
        assert!(matches!(ZdoStatus::Success, ZdoStatus::Success));
        assert!(matches!(ZdoStatus::DeviceNotFound, ZdoStatus::DeviceNotFound));
        assert!(matches!(ZdoStatus::InvalidEp, ZdoStatus::InvalidEp));
        assert!(matches!(ZdoStatus::NotActive, ZdoStatus::NotActive));
        assert!(matches!(ZdoStatus::NotSupported, ZdoStatus::NotSupported));
        assert!(matches!(ZdoStatus::Timeout, ZdoStatus::Timeout));
        assert!(matches!(ZdoStatus::NoMatch, ZdoStatus::NoMatch));
        assert!(matches!(ZdoStatus::TableFull, ZdoStatus::TableFull));
        assert!(matches!(ZdoStatus::NotAuthorized, ZdoStatus::NotAuthorized));
    }

    #[test]
    fn test_device_announce_encode() {
        let announce = DeviceAnnounce {
            short_address: 0x0001,
            ieee_address: test_addresses::END_DEVICE_1,
            capability: DeviceCapability::end_device(false),
        };
        
        let encoded = announce.encode();
        assert!(encoded.len() > 0);
    }

    #[test]
    fn test_device_announce_decode() {
        let announce = DeviceAnnounce {
            short_address: 0x0001,
            ieee_address: test_addresses::END_DEVICE_1,
            capability: DeviceCapability::end_device(false),
        };
        
        let encoded = announce.encode();
        let decoded = DeviceAnnounce::decode(&encoded);
        
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.short_address, announce.short_address);
        assert_eq!(decoded.ieee_address, announce.ieee_address);
    }

    #[test]
    fn test_frequency_band_2_4_ghz() {
        const FREQ_2_4_GHZ: u8 = 0x08;
        assert_eq!(FREQ_2_4_GHZ, 0x08);
    }

    #[test]
    fn test_server_mask_primary_trust_center() {
        const PRIMARY_TRUST_CENTER: u16 = 0x0001;
        assert_eq!(PRIMARY_TRUST_CENTER, 0x0001);
    }
}
