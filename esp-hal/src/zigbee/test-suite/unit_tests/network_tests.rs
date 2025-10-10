//! Unit tests for network module

#[cfg(test)]
mod tests {
    use crate::zigbee::network::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_network_info_new() {
        let info = NetworkInfo {
            short_address: 0x0001,
            ieee_address: test_addresses::END_DEVICE_1,
            pan_id: test_pan_ids::DEFAULT,
            extended_pan_id: 0x1122334455667788,
            channel: test_channels::CHANNEL_15,
            depth: 1,
            parent_address: Some(0x0000),
            lqi: 255,
            rssi: -40,
        };
        
        assert_eq!(info.short_address, 0x0001);
        assert_eq!(info.pan_id, test_pan_ids::DEFAULT);
        assert_eq!(info.channel, test_channels::CHANNEL_15);
    }

    #[test]
    fn test_network_manager_new() {
        let manager = NetworkManager::new();
        assert_eq!(manager.neighbor_count(), 0);
        assert_eq!(manager.route_count(), 0);
    }

    #[test]
    fn test_add_neighbor() {
        let mut manager = NetworkManager::new();
        let neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            DeviceType::EndDevice,
            255,
        );
        
        let result = manager.add_neighbor(neighbor);
        assert!(result.is_ok());
        assert_eq!(manager.neighbor_count(), 1);
    }

    #[test]
    fn test_neighbor_capacity() {
        let mut manager = NetworkManager::new();
        
        // Add maximum neighbors
        for i in 0..32 {
            let neighbor = test_neighbor(
                0x1000000000000000 + i,
                0x0001 + i as u16,
                DeviceType::EndDevice,
                255,
            );
            let result = manager.add_neighbor(neighbor);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let neighbor = test_neighbor(
            0x9999999999999999,
            0x9999,
            DeviceType::EndDevice,
            255,
        );
        let result = manager.add_neighbor(neighbor);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_neighbor() {
        let mut manager = NetworkManager::new();
        let neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            DeviceType::EndDevice,
            255,
        );
        
        manager.add_neighbor(neighbor).unwrap();
        assert_eq!(manager.neighbor_count(), 1);
        
        manager.remove_neighbor(0x0001);
        assert_eq!(manager.neighbor_count(), 0);
    }

    #[test]
    fn test_find_neighbor_by_short_addr() {
        let mut manager = NetworkManager::new();
        let neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            DeviceType::EndDevice,
            255,
        );
        
        manager.add_neighbor(neighbor).unwrap();
        
        let found = manager.find_neighbor_by_short_address(0x0001);
        assert!(found.is_some());
        assert_eq!(found.unwrap().short_address, 0x0001);
    }

    #[test]
    fn test_find_neighbor_by_ieee_addr() {
        let mut manager = NetworkManager::new();
        let neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            DeviceType::EndDevice,
            255,
        );
        
        manager.add_neighbor(neighbor).unwrap();
        
        let found = manager.find_neighbor_by_ieee_address(test_addresses::END_DEVICE_1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().ieee_address, test_addresses::END_DEVICE_1);
    }

    #[test]
    fn test_add_route() {
        let mut manager = NetworkManager::new();
        let route = test_route(0x0001, 0x0002, 5);
        
        let result = manager.add_route(route);
        assert!(result.is_ok());
        assert_eq!(manager.route_count(), 1);
    }

    #[test]
    fn test_route_capacity() {
        let mut manager = NetworkManager::new();
        
        // Add maximum routes
        for i in 0..16 {
            let route = test_route(0x0001 + i as u16, 0x0002, 5);
            let result = manager.add_route(route);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let route = test_route(0x9999, 0x0002, 5);
        let result = manager.add_route(route);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_route() {
        let mut manager = NetworkManager::new();
        let route = test_route(0x0001, 0x0002, 5);
        
        manager.add_route(route).unwrap();
        assert_eq!(manager.route_count(), 1);
        
        manager.remove_route(0x0001);
        assert_eq!(manager.route_count(), 0);
    }

    #[test]
    fn test_find_route() {
        let mut manager = NetworkManager::new();
        let route = test_route(0x0001, 0x0002, 5);
        
        manager.add_route(route).unwrap();
        
        let found = manager.find_route(0x0001);
        assert!(found.is_some());
        assert_eq!(found.unwrap().destination, 0x0001);
        assert_eq!(found.unwrap().next_hop, 0x0002);
    }

    #[test]
    fn test_add_binding() {
        let mut manager = NetworkManager::new();
        let binding = Binding {
            source_ieee: test_addresses::END_DEVICE_1,
            source_endpoint: 1,
            cluster_id: 0x0006, // On/Off
            destination: BindingDestination::ShortAddress {
                address: 0x0000,
                endpoint: 1,
            },
        };
        
        let result = manager.add_binding(binding);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binding_capacity() {
        let mut manager = NetworkManager::new();
        
        // Add maximum bindings
        for i in 0..16 {
            let binding = Binding {
                source_ieee: 0x1000000000000000 + i,
                source_endpoint: 1,
                cluster_id: 0x0006,
                destination: BindingDestination::ShortAddress {
                    address: 0x0000,
                    endpoint: 1,
                },
            };
            let result = manager.add_binding(binding);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let binding = Binding {
            source_ieee: 0x9999999999999999,
            source_endpoint: 1,
            cluster_id: 0x0006,
            destination: BindingDestination::ShortAddress {
                address: 0x0000,
                endpoint: 1,
            },
        };
        let result = manager.add_binding(binding);
        assert!(result.is_err());
    }

    #[test]
    fn test_binding_destination_short_address() {
        let dest = BindingDestination::ShortAddress {
            address: 0x0001,
            endpoint: 2,
        };
        
        assert!(matches!(dest, BindingDestination::ShortAddress { .. }));
    }

    #[test]
    fn test_binding_destination_ieee_address() {
        let dest = BindingDestination::IeeeAddress {
            address: test_addresses::END_DEVICE_1,
            endpoint: 2,
        };
        
        assert!(matches!(dest, BindingDestination::IeeeAddress { .. }));
    }

    #[test]
    fn test_binding_destination_group() {
        let dest = BindingDestination::Group(0x0001);
        assert!(matches!(dest, BindingDestination::Group(_)));
    }

    #[test]
    fn test_discovered_network() {
        let network = DiscoveredNetwork {
            pan_id: test_pan_ids::DEFAULT,
            extended_pan_id: 0x1122334455667788,
            channel: test_channels::CHANNEL_15,
            permit_joining: true,
            coordinator_address: 0x0000,
            lqi: 255,
            rssi: -40,
        };
        
        assert_eq!(network.pan_id, test_pan_ids::DEFAULT);
        assert!(network.permit_joining);
    }

    #[test]
    fn test_device_type_variants() {
        assert!(matches!(DeviceType::Coordinator, DeviceType::Coordinator));
        assert!(matches!(DeviceType::Router, DeviceType::Router));
        assert!(matches!(DeviceType::EndDevice, DeviceType::EndDevice));
    }

    #[test]
    fn test_relationship_variants() {
        assert!(matches!(Relationship::Parent, Relationship::Parent));
        assert!(matches!(Relationship::Child, Relationship::Child));
        assert!(matches!(Relationship::Sibling, Relationship::Sibling));
        assert!(matches!(Relationship::None, Relationship::None));
    }

    #[test]
    fn test_route_status_variants() {
        assert!(matches!(RouteStatus::Active, RouteStatus::Active));
        assert!(matches!(RouteStatus::Discovery, RouteStatus::Discovery));
        assert!(matches!(RouteStatus::ValidationUnderway, RouteStatus::ValidationUnderway));
        assert!(matches!(RouteStatus::Inactive, RouteStatus::Inactive));
    }

    #[test]
    fn test_form_network_params() {
        let params = FormNetworkParams {
            channel: test_channels::CHANNEL_15,
            pan_id: Some(test_pan_ids::DEFAULT),
            extended_pan_id: None,
            network_key: None,
        };
        
        assert_eq!(params.channel, test_channels::CHANNEL_15);
        assert_eq!(params.pan_id, Some(test_pan_ids::DEFAULT));
    }

    #[test]
    fn test_join_network_params() {
        let params = JoinNetworkParams {
            channel: Some(test_channels::CHANNEL_15),
            pan_id: Some(test_pan_ids::DEFAULT),
            extended_pan_id: None,
            rejoin: false,
            install_code: None,
        };
        
        assert_eq!(params.channel, Some(test_channels::CHANNEL_15));
        assert!(!params.rejoin);
    }

    #[test]
    fn test_neighbor_lqi_update() {
        let mut neighbor = test_neighbor(
            test_addresses::END_DEVICE_1,
            0x0001,
            DeviceType::EndDevice,
            255,
        );
        
        neighbor.lqi = 200;
        assert_eq!(neighbor.lqi, 200);
    }

    #[test]
    fn test_route_cost_comparison() {
        let route1 = test_route(0x0001, 0x0002, 5);
        let route2 = test_route(0x0001, 0x0003, 10);
        
        assert!(route1.cost < route2.cost);
    }
}
