//! Integration tests for data transmission

#[cfg(test)]
mod tests {
    use crate::zigbee::*;
    use crate::zigbee::test_suite::{mocks::*, helpers::*};

    #[test]
    fn test_send_data_coordinator_to_device() {
        let mut coord_radio = mock_coordinator_radio();
        let device_radio = mock_end_device_radio(test_addresses::END_DEVICE_1);
        
        let frame = MockFrame::data(
            0x0000, // Coordinator
            0x0001, // End device
            test_pan_ids::DEFAULT,
            b"Hello",
        );
        
        coord_radio.transmit(frame).unwrap();
        
        let stats = coord_radio.statistics();
        assert_eq!(stats.tx_count, 1);
    }

    #[test]
    fn test_send_data_device_to_coordinator() {
        let coord_radio = mock_coordinator_radio();
        let mut device_radio = mock_end_device_radio(test_addresses::END_DEVICE_1);
        device_radio.set_short_address(0x0001);
        
        let frame = MockFrame::data(
            0x0001, // End device
            0x0000, // Coordinator
            test_pan_ids::DEFAULT,
            b"Response",
        );
        
        device_radio.transmit(frame).unwrap();
        
        let stats = device_radio.statistics();
        assert_eq!(stats.tx_count, 1);
    }

    #[test]
    fn test_route_through_router() {
        let mut manager = network::NetworkManager::new();
        
        // Add route: destination 0x0002 via router 0x0001
        let route = test_route(0x0002, 0x0001, 5);
        manager.add_route(route).unwrap();
        
        let found_route = manager.find_route(0x0002);
        assert!(found_route.is_some());
        assert_eq!(found_route.unwrap().next_hop, 0x0001);
    }

    #[test]
    fn test_multi_hop_routing() {
        let mut router = device::Router::new();
        
        // Router has multiple routes
        router.add_route(device::RoutingEntry {
            destination: 0x0003,
            next_hop: 0x0002,
            status: device::RouteStatus::Active,
            cost: 2,
            last_used: 0,
        }).unwrap();
        
        router.add_route(device::RoutingEntry {
            destination: 0x0004,
            next_hop: 0x0002,
            status: device::RouteStatus::Active,
            cost: 3,
            last_used: 0,
        }).unwrap();
        
        assert_eq!(router.route_count(), 2);
    }

    #[test]
    fn test_broadcast_transmission() {
        let mut radio = mock_coordinator_radio();
        
        let broadcast = MockFrame::data(
            0x0000,
            0xFFFF, // Broadcast address
            test_pan_ids::DEFAULT,
            b"Broadcast",
        );
        
        radio.transmit(broadcast).unwrap();
        
        let transmitted = radio.get_transmitted();
        assert!(transmitted.is_some());
        assert_eq!(transmitted.unwrap().dst_addr, 0xFFFF);
    }

    #[test]
    fn test_acknowledgment_handling() {
        let mut radio = mock_end_device_radio(test_addresses::END_DEVICE_1);
        
        let ack = MockFrame::ack(42); // Sequence 42
        radio.inject_frame(ack).unwrap();
        
        let received = radio.receive();
        assert!(received.is_some());
        assert_eq!(received.unwrap().sequence, 42);
    }

    #[test]
    fn test_data_with_security() {
        let mut security_manager = security::SecurityManager::new();
        security_manager.set_network_key(test_network_key());
        
        let data = b"Secure data";
        let header = security::SecurityHeader::new(
            SecurityLevel::EncMic32,
            0x01,
            1,
            test_addresses::COORDINATOR,
        );
        
        let encrypted = security_manager.encrypt_network(data, &header);
        assert!(encrypted.is_some());
    }

    #[test]
    fn test_payload_size_limits() {
        let mut radio = mock_coordinator_radio();
        
        // Maximum payload for IEEE 802.15.4
        let large_payload = vec![0u8; 100];
        let frame = MockFrame::data(
            0x0000,
            0x0001,
            test_pan_ids::DEFAULT,
            &large_payload,
        );
        
        let result = radio.transmit(frame);
        assert!(result.is_ok());
    }

    #[test]
    fn test_link_quality_tracking() {
        let mut device = device::EndDevice::new(false);
        let mut parent = device::ParentInfo {
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
    fn test_frame_counter_increment() {
        let mut security_manager = security::SecurityManager::new();
        
        let counter1 = security_manager.get_outgoing_frame_counter();
        security_manager.increment_frame_counter();
        let counter2 = security_manager.get_outgoing_frame_counter();
        
        assert_eq!(counter2, counter1 + 1);
    }

    #[test]
    fn test_retransmission_on_error() {
        let mut radio = mock_end_device_radio(test_addresses::END_DEVICE_1);
        radio.inject_errors(true, false);
        
        let frame = MockFrame::data(0x0001, 0x0000, test_pan_ids::DEFAULT, b"Test");
        let result = radio.transmit(frame);
        
        assert!(result.is_err());
        assert_eq!(radio.statistics().tx_errors, 1);
    }

    #[test]
    fn test_binding_for_unicast() {
        let mut manager = network::NetworkManager::new();
        let binding = network::Binding {
            source_ieee: test_addresses::END_DEVICE_1,
            source_endpoint: 1,
            cluster_id: 0x0006, // On/Off
            destination: network::BindingDestination::ShortAddress {
                address: 0x0000,
                endpoint: 1,
            },
        };
        
        manager.add_binding(binding).unwrap();
    }

    #[test]
    fn test_group_addressing() {
        let mut manager = network::NetworkManager::new();
        let binding = network::Binding {
            source_ieee: test_addresses::END_DEVICE_1,
            source_endpoint: 1,
            cluster_id: 0x0006,
            destination: network::BindingDestination::Group(0x0001),
        };
        
        manager.add_binding(binding).unwrap();
    }
}
