//! Unit tests for coordinator module

#[cfg(test)]
mod tests {
    use crate::zigbee::coordinator::*;
    use crate::zigbee::test_suite::helpers::*;

    #[test]
    fn test_coordinator_new() {
        let coord = Coordinator::new();
        assert_eq!(coord.device_count(), 0);
        assert!(!coord.is_permit_joining());
    }

    #[test]
    fn test_add_device() {
        let mut coord = Coordinator::new();
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        let result = coord.add_device(device);
        assert!(result.is_ok());
        assert_eq!(coord.device_count(), 1);
    }

    #[test]
    fn test_device_capacity() {
        let mut coord = Coordinator::new();
        
        // Add maximum devices
        for i in 0..64 {
            let device = DeviceInfo {
                ieee_address: 0x1000000000000000 + i,
                short_address: 0x0001 + i as u16,
                capability: 0x80,
                endpoints: heapless::Vec::new(),
                last_seen: 0,
            };
            let result = coord.add_device(device);
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let device = DeviceInfo {
            ieee_address: 0x9999999999999999,
            short_address: 0x9999,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        let result = coord.add_device(device);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_device() {
        let mut coord = Coordinator::new();
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        coord.add_device(device).unwrap();
        assert_eq!(coord.device_count(), 1);
        
        coord.remove_device(0x0001);
        assert_eq!(coord.device_count(), 0);
    }

    #[test]
    fn test_find_device_by_short_addr() {
        let mut coord = Coordinator::new();
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        coord.add_device(device).unwrap();
        
        let found = coord.find_device_by_short_address(0x0001);
        assert!(found.is_some());
        assert_eq!(found.unwrap().short_address, 0x0001);
    }

    #[test]
    fn test_find_device_by_ieee_addr() {
        let mut coord = Coordinator::new();
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        coord.add_device(device).unwrap();
        
        let found = coord.find_device_by_ieee_address(test_addresses::END_DEVICE_1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().ieee_address, test_addresses::END_DEVICE_1);
    }

    #[test]
    fn test_permit_join() {
        let mut coord = Coordinator::new();
        
        assert!(!coord.is_permit_joining());
        
        coord.permit_join(60).unwrap();
        assert!(coord.is_permit_joining());
    }

    #[test]
    fn test_permit_join_indefinite() {
        let mut coord = Coordinator::new();
        
        coord.permit_join(255).unwrap(); // 255 = indefinite
        assert!(coord.is_permit_joining());
    }

    #[test]
    fn test_stop_permit_join() {
        let mut coord = Coordinator::new();
        
        coord.permit_join(60).unwrap();
        assert!(coord.is_permit_joining());
        
        coord.stop_permit_join();
        assert!(!coord.is_permit_joining());
    }

    #[test]
    fn test_allocate_short_address() {
        let mut coord = Coordinator::new();
        
        let addr1 = coord.allocate_short_address();
        let addr2 = coord.allocate_short_address();
        
        assert_ne!(addr1, addr2);
        assert!(is_valid_short_address(addr1));
        assert!(is_valid_short_address(addr2));
    }

    #[test]
    fn test_allocate_address_range() {
        let mut coord = Coordinator::new();
        
        for _ in 0..100 {
            let addr = coord.allocate_short_address();
            assert!(addr >= 0x0001 && addr <= 0xFFF7);
        }
    }

    #[test]
    fn test_trust_center_key_management() {
        let mut coord = Coordinator::new();
        let ieee_addr = test_addresses::END_DEVICE_1;
        let key = test_link_key();
        
        coord.add_trust_center_key(ieee_addr, key).unwrap();
        
        let retrieved = coord.get_trust_center_key(ieee_addr);
        assert!(retrieved.is_some());
        assert!(keys_equal(retrieved.unwrap(), &key));
    }

    #[test]
    fn test_trust_center_key_capacity() {
        let mut coord = Coordinator::new();
        
        // Add maximum keys
        for i in 0..32 {
            let ieee_addr = 0x1000000000000000 + i;
            let result = coord.add_trust_center_key(ieee_addr, test_link_key());
            assert!(result.is_ok());
        }
        
        // Adding one more should fail
        let result = coord.add_trust_center_key(0x9999999999999999, test_link_key());
        assert!(result.is_err());
    }

    #[test]
    fn test_device_info_with_endpoints() {
        let mut endpoints = heapless::Vec::new();
        endpoints.push(1).ok();
        endpoints.push(2).ok();
        
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints,
            last_seen: 1000,
        };
        
        assert_eq!(device.endpoints.len(), 2);
        assert_eq!(device.endpoints[0], 1);
        assert_eq!(device.endpoints[1], 2);
    }

    #[test]
    fn test_update_device_last_seen() {
        let mut coord = Coordinator::new();
        let device = DeviceInfo {
            ieee_address: test_addresses::END_DEVICE_1,
            short_address: 0x0001,
            capability: 0x80,
            endpoints: heapless::Vec::new(),
            last_seen: 0,
        };
        
        coord.add_device(device).unwrap();
        
        coord.update_device_last_seen(0x0001, 5000);
        let updated = coord.find_device_by_short_address(0x0001);
        assert_eq!(updated.unwrap().last_seen, 5000);
    }
}
