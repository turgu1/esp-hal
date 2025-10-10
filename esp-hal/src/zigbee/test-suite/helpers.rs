//! Test helper functions and utilities

use crate::zigbee::*;
use super::mocks::*;

/// Test IEEE addresses
pub mod test_addresses {
    pub const COORDINATOR: u64 = 0x0011223344556677;
    pub const ROUTER_1: u64 = 0x1122334455667788;
    pub const ROUTER_2: u64 = 0x2233445566778899;
    pub const END_DEVICE_1: u64 = 0x33445566778899AA;
    pub const END_DEVICE_2: u64 = 0x445566778899AABB;
    pub const END_DEVICE_3: u64 = 0x556677889900CCDD;
}

/// Test PAN IDs
pub mod test_pan_ids {
    pub const DEFAULT: u16 = 0x1234;
    pub const ALTERNATIVE: u16 = 0x5678;
    pub const BROADCAST: u16 = 0xFFFF;
}

/// Test channels
pub mod test_channels {
    pub const CHANNEL_11: u8 = 11;
    pub const CHANNEL_15: u8 = 15;
    pub const CHANNEL_20: u8 = 20;
    pub const CHANNEL_25: u8 = 25;
}

/// Create a default coordinator config
pub fn coordinator_config() -> Config {
    Config::default()
        .with_role(Role::Coordinator)
        .with_channel(test_channels::CHANNEL_15)
        .with_pan_id(test_pan_ids::DEFAULT)
}

/// Create a default router config
pub fn router_config() -> Config {
    Config::default()
        .with_role(Role::Router)
        .with_channel(test_channels::CHANNEL_15)
}

/// Create a default end device config
pub fn end_device_config(sleepy: bool) -> Config {
    Config::default()
        .with_role(Role::EndDevice { sleepy })
        .with_channel(test_channels::CHANNEL_15)
}

/// Create a secure config
pub fn secure_config(role: Role) -> Config {
    Config::default()
        .with_role(role)
        .with_security(SecurityLevel::High)
        .with_channel(test_channels::CHANNEL_15)
}

/// Create a mock radio with coordinator setup
pub fn mock_coordinator_radio() -> MockRadio {
    let mut radio = MockRadio::new(test_addresses::COORDINATOR);
    radio.set_coordinator(true);
    radio.set_pan_id(test_pan_ids::DEFAULT);
    radio.set_short_address(0x0000);
    radio.set_channel(test_channels::CHANNEL_15);
    radio
}

/// Create a mock radio with router setup
pub fn mock_router_radio(ieee_addr: u64) -> MockRadio {
    let mut radio = MockRadio::new(ieee_addr);
    radio.set_coordinator(false);
    radio.set_channel(test_channels::CHANNEL_15);
    radio
}

/// Create a mock radio with end device setup
pub fn mock_end_device_radio(ieee_addr: u64) -> MockRadio {
    let mut radio = MockRadio::new(ieee_addr);
    radio.set_coordinator(false);
    radio.set_channel(test_channels::CHANNEL_15);
    radio
}

/// Generate test network key
pub fn test_network_key() -> [u8; 16] {
    [0x01, 0x03, 0x05, 0x07, 0x09, 0x0B, 0x0D, 0x0F,
     0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E]
}

/// Generate test link key
pub fn test_link_key() -> [u8; 16] {
    [0x5A, 0x69, 0x67, 0x42, 0x65, 0x65, 0x41, 0x6C,
     0x6C, 0x69, 0x61, 0x6E, 0x63, 0x65, 0x30, 0x39]
}

/// Generate test install code
pub fn test_install_code() -> [u8; 18] {
    let mut code = [0u8; 18];
    // Fill with pattern
    for (i, byte) in code[..16].iter_mut().enumerate() {
        *byte = (i as u8) * 0x11;
    }
    // CRC-16 (simplified for testing)
    code[16] = 0x12;
    code[17] = 0x34;
    code
}

/// Create test ZCL frame
pub fn test_zcl_frame(cluster_id: u16, command: u8, payload: &[u8]) -> Vec<u8, 128> {
    let mut frame = Vec::new();
    
    // Frame control
    frame.push(0x00).ok();
    
    // Transaction sequence number
    frame.push(0x01).ok();
    
    // Command ID
    frame.push(command).ok();
    
    // Payload
    frame.extend_from_slice(payload).ok();
    
    frame
}

/// Simulate network formation
pub fn simulate_network_formation(radio: &mut MockRadio) {
    radio.set_pan_id(test_pan_ids::DEFAULT);
    radio.set_short_address(0x0000);
    radio.set_coordinator(true);
}

/// Simulate device join
pub fn simulate_device_join(
    coordinator_radio: &mut MockRadio,
    device_radio: &mut MockRadio,
    device_short_addr: u16,
) {
    // Device sends association request
    let assoc_req = MockFrame::data(
        0xFFFF,
        0x0000,
        test_pan_ids::DEFAULT,
        &[0x01], // Association request
    );
    coordinator_radio.inject_frame(assoc_req).ok();
    
    // Coordinator responds with short address
    device_radio.set_pan_id(test_pan_ids::DEFAULT);
    device_radio.set_short_address(device_short_addr);
    
    let assoc_resp = MockFrame::data(
        0x0000,
        device_short_addr,
        test_pan_ids::DEFAULT,
        &[0x02], // Association response
    );
    device_radio.inject_frame(assoc_resp).ok();
}

/// Assert frame was transmitted
pub fn assert_frame_transmitted(radio: &mut MockRadio, expected_dst: u16) {
    let frame = radio.get_transmitted().expect("No frame transmitted");
    assert_eq!(frame.dst_addr, expected_dst);
}

/// Assert frame was received
pub fn assert_frame_received(radio: &mut MockRadio, expected_src: u16) {
    let frame = radio.receive().expect("No frame received");
    assert_eq!(frame.src_addr, expected_src);
}

/// Create test neighbor entry
pub fn test_neighbor(
    ieee_addr: u64,
    short_addr: u16,
    device_type: network::DeviceType,
    lqi: u8,
) -> network::Neighbor {
    network::Neighbor {
        ieee_address: ieee_addr,
        short_address: short_addr,
        device_type,
        lqi,
        rssi: -40,
        depth: 1,
        relationship: network::Relationship::Child,
        permit_joining: false,
        last_seen: 0,
    }
}

/// Create test route entry
pub fn test_route(
    destination: u16,
    next_hop: u16,
    cost: u8,
) -> network::Route {
    network::Route {
        destination,
        next_hop,
        status: network::RouteStatus::Active,
        hop_count: 1,
        cost,
        last_used: 0,
    }
}

/// Verify security header
pub fn verify_security_header(header: &security::SecurityHeader) -> bool {
    header.level != security::SecurityLevel::None
        && header.key_identifier != 0
        && header.frame_counter > 0
}

/// Generate test beacon payload
pub fn test_beacon_payload(permit_joining: bool) -> Vec<u8, 32> {
    let mut payload = Vec::new();
    
    // Protocol ID
    payload.push(0x00).ok();
    
    // Stack profile and protocol version
    payload.push(0x02).ok(); // ZigBee PRO
    
    // Capacity information
    let capacity = if permit_joining { 0x80 } else { 0x00 };
    payload.push(capacity).ok();
    
    // Extended PAN ID
    for i in 0..8 {
        payload.push(i).ok();
    }
    
    payload
}

/// Compare network keys
pub fn keys_equal(key1: &[u8; 16], key2: &[u8; 16]) -> bool {
    key1.iter().zip(key2.iter()).all(|(a, b)| a == b)
}

/// Validate channel is in valid range
pub fn is_valid_channel(channel: u8) -> bool {
    (11..=26).contains(&channel)
}

/// Validate PAN ID
pub fn is_valid_pan_id(pan_id: u16) -> bool {
    pan_id != 0xFFFF // Broadcast PAN ID is invalid for assignment
}

/// Validate short address
pub fn is_valid_short_address(addr: u16) -> bool {
    addr != 0xFFFF && addr != 0xFFFE // Broadcast and reserved
}

/// Calculate expected frame size
pub fn calculate_frame_size(payload_len: usize, security: bool) -> usize {
    const MAC_HEADER_SIZE: usize = 9; // Minimum MAC header
    const MAC_FOOTER_SIZE: usize = 2; // FCS
    const NWK_HEADER_SIZE: usize = 8; // Minimum NWK header
    const SECURITY_HEADER_SIZE: usize = 14; // AES-128 CCM*
    const SECURITY_MIC_SIZE: usize = 4; // MIC-32
    
    let mut size = MAC_HEADER_SIZE + NWK_HEADER_SIZE + payload_len + MAC_FOOTER_SIZE;
    
    if security {
        size += SECURITY_HEADER_SIZE + SECURITY_MIC_SIZE;
    }
    
    size
}

/// Create test attribute value
pub fn test_attribute_value(attr_type: AttributeType) -> zcl::AttributeValue {
    match attr_type {
        AttributeType::Boolean => zcl::AttributeValue::Boolean(true),
        AttributeType::Uint8 => zcl::AttributeValue::Uint8(42),
        AttributeType::Uint16 => zcl::AttributeValue::Uint16(1234),
        AttributeType::Int16 => zcl::AttributeValue::Int16(-100),
        AttributeType::String => {
            let mut s = heapless::String::new();
            s.push_str("Test").ok();
            zcl::AttributeValue::String(s)
        }
    }
}

/// Attribute type enum for helper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Boolean,
    Uint8,
    Uint16,
    Int16,
    String,
}
