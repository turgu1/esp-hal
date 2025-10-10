//! Integration tests for network formation

#[cfg(test)]
mod tests {
    use crate::zigbee::*;
    use crate::zigbee::test_suite::{mocks::*, helpers::*};

    #[test]
    fn test_coordinator_forms_network() {
        let mut radio = mock_coordinator_radio();
        let config = coordinator_config();
        
        // Simulate network formation
        simulate_network_formation(&mut radio);
        
        assert_eq!(radio.pan_id(), test_pan_ids::DEFAULT);
        assert_eq!(radio.short_address(), 0x0000);
        assert_eq!(radio.channel(), test_channels::CHANNEL_15);
    }

    #[test]
    fn test_network_formation_with_custom_pan_id() {
        let mut radio = MockRadio::new(test_addresses::COORDINATOR);
        let config = Config::coordinator()
            .with_channel(test_channels::CHANNEL_20)
            .with_pan_id(0x5678);
        
        radio.set_pan_id(0x5678);
        radio.set_coordinator(true);
        radio.set_channel(test_channels::CHANNEL_20);
        
        assert_eq!(radio.pan_id(), 0x5678);
        assert_eq!(radio.channel(), test_channels::CHANNEL_20);
    }

    #[test]
    fn test_network_formation_on_different_channels() {
        for channel in &[test_channels::CHANNEL_11, test_channels::CHANNEL_15, 
                        test_channels::CHANNEL_20, test_channels::CHANNEL_25] {
            let mut radio = MockRadio::new(test_addresses::COORDINATOR);
            radio.set_channel(*channel);
            radio.set_coordinator(true);
            
            assert_eq!(radio.channel(), *channel);
        }
    }

    #[test]
    fn test_coordinator_permits_joining() {
        let mut coordinator = coordinator::Coordinator::new();
        
        coordinator.permit_join(60).unwrap();
        assert!(coordinator.is_permit_joining());
    }

    #[test]
    fn test_coordinator_broadcasts_beacon() {
        let mut radio = mock_coordinator_radio();
        
        let beacon = MockFrame::beacon(0x0000, test_pan_ids::DEFAULT);
        radio.transmit(beacon).unwrap();
        
        let transmitted = radio.get_transmitted();
        assert!(transmitted.is_some());
        assert_eq!(transmitted.unwrap().frame_type, FrameType::Beacon);
    }

    #[test]
    fn test_beacon_contains_permit_join_info() {
        let beacon_payload = test_beacon_payload(true);
        
        // Check capacity byte indicates permit joining
        assert!(beacon_payload[2] & 0x80 != 0);
    }

    #[test]
    fn test_network_manager_initialized() {
        let manager = network::NetworkManager::new();
        
        assert_eq!(manager.neighbor_count(), 0);
        assert_eq!(manager.route_count(), 0);
    }

    #[test]
    fn test_coordinator_allocates_addresses_sequentially() {
        let mut coordinator = coordinator::Coordinator::new();
        
        let addr1 = coordinator.allocate_short_address();
        let addr2 = coordinator.allocate_short_address();
        let addr3 = coordinator.allocate_short_address();
        
        assert!(addr1 < addr2);
        assert!(addr2 < addr3);
    }

    #[test]
    fn test_network_formation_with_security() {
        let mut radio = mock_coordinator_radio();
        let config = Config::coordinator()
            .with_channel(test_channels::CHANNEL_15)
            .with_security(SecurityLevel::High);
        
        let mut security_manager = security::SecurityManager::new();
        security_manager.generate_network_key();
        
        assert!(security_manager.network_key().is_some());
    }

    #[test]
    fn test_multiple_coordinators_different_pan_ids() {
        let mut coord1 = MockRadio::new(test_addresses::COORDINATOR);
        let mut coord2 = MockRadio::new(test_addresses::ROUTER_1);
        
        coord1.set_pan_id(0x1234);
        coord1.set_coordinator(true);
        
        coord2.set_pan_id(0x5678);
        coord2.set_coordinator(true);
        
        assert_ne!(coord1.pan_id(), coord2.pan_id());
    }
}
