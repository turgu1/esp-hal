//! Unit tests for device module (EndDevice and Router)

#[cfg(test)]
mod tests {
    use crate::zigbee::device::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_end_device_new() {
        let device = EndDevice::new(false);
        assert!(!device.is_sleepy());
        assert!(device.parent_info().is_none());
    }

    #[test]
    fn test_end_device_sleepy() {
        let device = EndDevice::new(true);
        assert!(device.is_sleepy());
    }

    #[test]
    fn test_end_device_set_parent() {
        let mut device = EndDevice::new(false);
        let parent = ParentInfo {
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
    fn test_end_device_clear_parent() {
        let mut device = EndDevice::new(false);
        let parent = ParentInfo {
            ieee_address: test_addresses::COORDINATOR,
            short_address: 0x0000,
            lqi: 255,
            rssi: -40,
            last_communication: 0,
        };
        
        device.set_parent(parent);
        assert!(device.parent_info().is_some());
        
        device.clear_parent();
        assert!(device.parent_info().is_none());
    }

    #[test]
    fn test_end_device_poll_rate() {
        let mut device = EndDevice::new(true);
        
        device.set_poll_rate(500);
        assert_eq!(device.poll_rate_ms(), 500);
    }

    #[test]
    fn test_end_device_update_parent_lqi() {
        let mut device = EndDevice::new(false);
        let parent = ParentInfo {
            ieee_address: test_addresses::COORDINATOR,
            short_address: 0x0000,
            lqi: 255,
            rssi: -40,
            last_communication: 0,
        };
        
        device.set_parent(parent);
        device.update_parent_lqi(200, -50);
        
        let updated = device.parent_info().unwrap();
        assert_eq!(updated.lqi, 200);
        assert_eq!(updated.rssi, -50);
    }

    #[test]
    fn test_router_new() {
        let router = Router::new();
        assert_eq!(router.child_count(), 0);
        assert_eq!(router.route_count(), 0);
    }

    #[test]
    fn test_router_add_child() {
        let mut router = Router::new();
        let child = ChildInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            timeout: 30,
            last_poll: 0,
        };
        
        let result = router.add_child(child);
        assert!(result.is_ok());
        assert_eq!(router.child_count(), 1);
    }

    #[test]
    fn test_router_child_capacity() {
        let mut router = Router::new();
        
        // Add maximum children
        for i in 0..32 {
            let child = ChildInfo {
                ieee_address: 0x1000000000000000 + i,
                short_address: 0x0001 + i as u16,
                capability: 0x80,
                timeout: 30,
                last_poll: 0,
            };
            let result = router.add_child(child);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let child = ChildInfo {
            ieee_address: 0x9999999999999999,
            short_address: 0x9999,
            capability: 0x80,
            timeout: 30,
            last_poll: 0,
        };
        let result = router.add_child(child);
        assert!(result.is_err());
    }

    #[test]
    fn test_router_remove_child() {
        let mut router = Router::new();
        let child = ChildInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            timeout: 30,
            last_poll: 0,
        };
        
        router.add_child(child).unwrap();
        assert_eq!(router.child_count(), 1);
        
        router.remove_child(0x0001);
        assert_eq!(router.child_count(), 0);
    }

    #[test]
    fn test_router_find_child() {
        let mut router = Router::new();
        let child = ChildInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            timeout: 30,
            last_poll: 0,
        };
        
        router.add_child(child).unwrap();
        
        let found = router.find_child(0x0001);
        assert!(found.is_some());
        assert_eq!(found.unwrap().short_address, 0x0001);
    }

    #[test]
    fn test_router_add_route() {
        let mut router = Router::new();
        let route = RoutingEntry {
            destination: 0x0002,
            next_hop: 0x0001,
            status: RouteStatus::Active,
            cost: 5,
            last_used: 0,
        };
        
        let result = router.add_route(route);
        assert!(result.is_ok());
        assert_eq!(router.route_count(), 1);
    }

    #[test]
    fn test_router_route_capacity() {
        let mut router = Router::new();
        
        // Add maximum routes
        for i in 0..16 {
            let route = RoutingEntry {
                destination: 0x0001 + i as u16,
                next_hop: 0x0000,
                status: RouteStatus::Active,
                cost: 5,
                last_used: 0,
            };
            let result = router.add_route(route);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let route = RoutingEntry {
            destination: 0x9999,
            next_hop: 0x0000,
            status: RouteStatus::Active,
            cost: 5,
            last_used: 0,
        };
        let result = router.add_route(route);
        assert!(result.is_err());
    }

    #[test]
    fn test_router_find_route() {
        let mut router = Router::new();
        let route = RoutingEntry {
            destination: 0x0002,
            next_hop: 0x0001,
            status: RouteStatus::Active,
            cost: 5,
            last_used: 0,
        };
        
        router.add_route(route).unwrap();
        
        let found = router.find_route(0x0002);
        assert!(found.is_some());
        assert_eq!(found.unwrap().next_hop, 0x0001);
    }

    #[test]
    fn test_router_update_route_cost() {
        let mut router = Router::new();
        let route = RoutingEntry {
            destination: 0x0002,
            next_hop: 0x0001,
            status: RouteStatus::Active,
            cost: 5,
            last_used: 0,
        };
        
        router.add_route(route).unwrap();
        router.update_route_cost(0x0002, 10);
        
        let updated = router.find_route(0x0002);
        assert_eq!(updated.unwrap().cost, 10);
    }

    #[test]
    fn test_child_info_timeout() {
        let child = ChildInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            timeout: 60,
            last_poll: 1000,
        };
        
        assert_eq!(child.timeout, 60);
        assert_eq!(child.last_poll, 1000);
    }

    #[test]
    fn test_parent_info_link_quality() {
        let parent = ParentInfo {
            ieee_address: test_addresses::COORDINATOR,
            short_address: 0x0000,
            lqi: 200,
            rssi: -50,
            last_communication: 5000,
        };
        
        assert_eq!(parent.lqi, 200);
        assert_eq!(parent.rssi, -50);
    }

    #[test]
    fn test_routing_entry_status() {
        let route = RoutingEntry {
            destination: 0x0002,
            next_hop: 0x0001,
            status: RouteStatus::Active,
            cost: 5,
            last_used: 0,
        };
        
        assert!(matches!(route.status, RouteStatus::Active));
    }

    #[test]
    fn test_route_status_variants() {
        let active = RouteStatus::Active;
        let discovery = RouteStatus::Discovery;
        let validation = RouteStatus::ValidationUnderway;
        let inactive = RouteStatus::Inactive;
        
        assert!(matches!(active, RouteStatus::Active));
        assert!(matches!(discovery, RouteStatus::Discovery));
        assert!(matches!(validation, RouteStatus::ValidationUnderway));
        assert!(matches!(inactive, RouteStatus::Inactive));
    }
}
