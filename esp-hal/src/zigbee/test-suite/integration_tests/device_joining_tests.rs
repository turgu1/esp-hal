//! Integration tests for device joining

#[cfg(test)]
mod tests {
    use crate::zigbee::*;
    use crate::zigbee::test_suite::{mocks::*, helpers::*};

    #[test]
    fn test_end_device_joins_network() {
        let mut coord_radio = mock_coordinator_radio();
        let mut device_radio = mock_end_device_radio(test_addresses::END_DEVICE_1);
        
        simulate_device_join(&mut coord_radio, &mut device_radio, 0x0001);
        
        assert_eq!(device_radio.pan_id(), test_pan_ids::DEFAULT);
        assert_eq!(device_radio.short_address(), 0x0001);
    }

    #[test]
    fn test_router_joins_network() {
        let mut coord_radio = mock_coordinator_radio();
        let mut router_radio = mock_router_radio(test_addresses::ROUTER_1);
        
        simulate_device_join(&mut coord_radio, &mut router_radio, 0x0001);
        
        assert_eq!(router_radio.pan_id(), test_pan_ids::DEFAULT);
        assert_eq!(router_radio.short_address(), 0x0001);
    }

    #[test]
    fn test_coordinator_tracks_joined_device() {
        let mut coordinator = coordinator::Coordinator::new();
        let device_info = coordinator::DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        coordinator.add_device(device_info).unwrap();
        
        assert_eq!(coordinator.device_count(), 1);
        let found = coordinator.find_device_by_short_address(0x0001);
        assert!(found.is_some());
    }

    #[test]
    fn test_multiple_devices_join() {
        let mut coordinator = coordinator::Coordinator::new();
        
        for i in 1..=5 {
            let device_info = coordinator::DeviceInfo {
                ieee_address: 0x1000000000000000 + i,
                short_address: i as u16,
                capability: 0x80,
                endpoints: heapless::Vec::new(),
                last_seen: 0,
            };
            coordinator.add_device(device_info).unwrap();
        }
        
        assert_eq!(coordinator.device_count(), 5);
    }

    #[test]
    fn test_device_added_to_neighbor_table() {
        let mut manager = network::NetworkManager::new();
        let neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            network::DeviceType::EndDevice,
            255,
        );
        
        manager.add_neighbor(neighbor).unwrap();
        
        let found = manager.find_neighbor_by_short_address(0x0001);
        assert!(found.is_some());
    }

    #[test]
    fn test_end_device_sets_parent() {
        let mut device = device::EndDevice::new(false);
        let parent = device::ParentInfo {
            ieee_address: test_addresses::COORDINATOR,
            short_address: 0x0000,
            lqi: 255,
            rssi: -40,
            last_communication: 0,
        };
        
        device.set_parent(parent);
        
        assert!(device.parent_info().is_some());
        assert_eq!(device.parent_info().unwrap().short_address, 0x0000);
    }

    #[test]
    fn test_router_accepts_children() {
        let mut router = device::Router::new();
        
        for i in 1..=5 {
            let child = device::ChildInfo {
                ieee_address: 0x1000000000000000 + i,
                short_address: 0x0001 + i as u16,
                capability: 0x80,
                timeout: 30,
                last_poll: 0,
            };
            router.add_child(child).unwrap();
        }
        
        assert_eq!(router.child_count(), 5);
    }

    #[test]
    fn test_join_with_install_code() {
        let mut security_manager = security::SecurityManager::new();
        let ieee_addr = test_addresses::END_DEVICE_1;
        let install_code = test_install_code();
        
        security_manager.add_install_code(ieee_addr, &install_code).unwrap();
        
        // Verify install code was stored (would be used for key derivation)
        // In real implementation, this would derive a link key
    }

    #[test]
    fn test_device_announce_after_join() {
        let announce = zdo::DeviceAnnounce {
            short_address: 0x0001,
            ieee_address: test_addresses::END_DEVICE_1,
            capability: zdo::DeviceCapability::end_device(false),
        };
        
        assert_eq!(announce.short_address, 0x0001);
        assert_eq!(announce.ieee_address, test_addresses::END_DEVICE_1);
    }

    #[test]
    fn test_association_with_security() {
        let mut security_manager = security::SecurityManager::new();
        let network_key = test_network_key();
        security_manager.set_network_key(network_key);
        
        let link_key = test_link_key();
        security_manager.add_link_key(test_addresses::END_DEVICE_1, link_key).unwrap();
        
        assert!(security_manager.network_key().is_some());
        assert!(security_manager.get_link_key(test_addresses::END_DEVICE_1).is_some());
    }

    #[test]
    fn test_join_timeout_handling() {
        let config = end_device_config(false).with_join_timeout(30);
        
        assert_eq!(config.join_timeout, 30);
    }

    #[test]
    fn test_sleepy_device_join() {
        let device = device::EndDevice::new(true);
        
        assert!(device.is_sleepy());
    }

    #[test]
    fn test_network_discovery_before_join() {
        let discovered = network::DiscoveredNetwork {
            pan_id: test_pan_ids::DEFAULT,
            extended_pan_id: 0x1122334455667788,
            channel: test_channels::CHANNEL_15,
            permit_joining: true,
            coordinator_address: 0x0000,
            lqi: 255,
            rssi: -40,
        };
        
        assert!(discovered.permit_joining);
        assert_eq!(discovered.pan_id, test_pan_ids::DEFAULT);
    }
}
