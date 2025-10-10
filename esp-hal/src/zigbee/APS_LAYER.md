# APS (Application Support Sublayer) Layer Documentation

## Overview

The APS (Application Support Sublayer) layer is a critical component of the Zigbee protocol stack, sitting between the Network Layer (NWK) and the Application Layer (including ZCL and ZDO). It provides essential services for application data transfer, device binding, group management, and message fragmentation.

## Architecture Position

```
┌──────────────────────────────────┐
│  Application Layer (ZCL, ZDO)    │
├──────────────────────────────────┤
│  APS (Application Support)       │ ← This Layer
├──────────────────────────────────┤
│  NWK (Network Layer)             │
├──────────────────────────────────┤
│  MAC (IEEE 802.15.4 MAC)         │
├──────────────────────────────────┤
│  PHY (IEEE 802.15.4 PHY)         │
└──────────────────────────────────┘
```

## Key Features

### 1. Data Transfer Services

**Delivery Modes:**
- **Unicast**: Point-to-point communication
- **Broadcast**: Send to all devices in network
- **Group**: Send to devices in a specific group

**Reliability:**
- Optional acknowledgment (ACK) mechanism
- Retransmission on failure (when ACKs enabled)
- Duplicate rejection using APS counters

### 2. Fragmentation and Reassembly

**Purpose:** Handle messages larger than the maximum frame size

**Specifications:**
- Maximum payload before fragmentation: 82 bytes
- Maximum fragments per message: 16
- Automatic reassembly on receiver
- Fragment timeout and cleanup

**Extended Header:**
- Fragment number (0-15)
- Total fragment count (0-15)
- Block number for very large messages

### 3. Binding Management

**What is Binding?**
Binding creates a logical connection between two devices for a specific cluster, allowing direct communication without knowing network addresses.

**Binding Table:**
- Source endpoint
- Cluster ID
- Destination address (64-bit IEEE address)
- Destination endpoint

**Use Cases:**
- Light switch bound to light bulb
- Sensor bound to controller
- Automatic device pairing

### 4. Group Management

**What are Groups?**
Groups allow multiple devices to be addressed with a single message, enabling efficient multicast communication.

**Group Membership:**
- Group address (16-bit identifier)
- Endpoint membership
- Multiple groups per endpoint

**Use Cases:**
- Control all lights in a room
- Broadcast to all sensors
- Scene control

### 5. Security Integration

**APS-Level Security:**
- Frame counter for replay protection
- Key negotiation and transport
- Security commands (SKKE, Transport Key, Update Device)

## Frame Format

### APS Data Frame

```
┌──────────────────┬─────────────────┬──────────────┐
│  Frame Control   │  Addressing     │   Payload    │
│    (1 byte)      │  (variable)     │  (variable)  │
└──────────────────┴─────────────────┴──────────────┘

Frame Control:
  Bits 0-1: Frame Type (Data/Command/Ack)
  Bits 2-3: Delivery Mode (Unicast/Broadcast/Group)
  Bit 4: ACK Format
  Bit 5: Security
  Bit 6: ACK Request
  Bit 7: Extended Header Present

Addressing:
  - Destination Endpoint (1 byte)
  - Group Address (2 bytes, if Group mode)
  - Cluster ID (2 bytes)
  - Profile ID (2 bytes)
  - Source Endpoint (1 byte)
  - APS Counter (1 byte)
  - Extended Header (2 bytes, if present)

Payload:
  - Application data (0-82 bytes typically)
```

### APS Acknowledgment Frame

```
┌──────────────────┬─────────────────┐
│  Frame Control   │  Addressing     │
│    (1 byte)      │  (8 bytes)      │
└──────────────────┴─────────────────┘

Acknowledgment contains:
  - Destination Endpoint
  - Cluster ID
  - Profile ID
  - Source Endpoint
  - APS Counter (being acknowledged)
```

### APS Extended Header (Fragmentation)

```
┌─────────────────────┬─────────────────┐
│  Fragment Info      │  Block Number   │
│    (1 byte)         │    (1 byte)     │
└─────────────────────┴─────────────────┘

Fragment Info:
  Bits 0-3: Fragment Number (0-15)
  Bits 4-7: Fragment Count (1-15)

Block Number:
  - For very large messages requiring multiple fragment sets
```

## API Reference

### Core Types

#### `ApsFrameType`
```rust
pub enum ApsFrameType {
    Data = 0,          // Application data
    Command = 1,       // APS command
    Acknowledgment = 2,// Acknowledgment frame
}
```

#### `ApsDeliveryMode`
```rust
pub enum ApsDeliveryMode {
    Unicast = 0,       // Point-to-point
    Broadcast = 2,     // To all devices
    Group = 3,         // To group members
}
```

#### `ApsFrameControl`
Configuration for APS frame transmission.

#### `ApsDataFrame`
Complete APS data frame with addressing and payload.

#### `ApsBinding`
Binding table entry linking source/destination for a cluster.

#### `ApsGroupMembership`
Group membership entry for an endpoint.

### ApsManager API

#### Creating a Manager

```rust
let mut aps_manager = ApsManager::new();
```

#### Sending Data

**Simple Unicast:**
```rust
let mut frame = ApsDataFrame::new(
    dst_endpoint: 1,
    src_endpoint: 1,
    cluster_id: 0x0006,  // On/Off cluster
    profile_id: 0x0104,   // Home Automation profile
    payload: &[0x01, 0x00], // Command data
)?;

frame.aps_counter = aps_manager.next_counter();
let encoded = frame.encode()?;
// Send via network layer
```

**With Acknowledgment Request:**
```rust
let frame = ApsDataFrame::new(...)?.with_ack_request();
```

**Broadcast:**
```rust
let frame = ApsDataFrame::new_broadcast(
    dst_endpoint: 255,  // Broadcast endpoint
    src_endpoint: 1,
    cluster_id: 0x0006,
    profile_id: 0x0104,
    payload: data,
)?;
```

**Group:**
```rust
let frame = ApsDataFrame::new_group(
    group_address: 0x0001,
    dst_endpoint: 1,
    src_endpoint: 1,
    cluster_id: 0x0006,
    profile_id: 0x0104,
    payload: data,
)?;
```

#### Binding Management

**Create Binding:**
```rust
let binding = ApsBinding::new(
    src_endpoint: 1,
    cluster_id: 0x0006,
    dst_address: 0x0011223344556677,  // IEEE address
    dst_endpoint: 1,
);

aps_manager.add_binding(binding)?;
```

**Get Bindings:**
```rust
let bindings = aps_manager.get_bindings(src_endpoint: 1, cluster_id: 0x0006);
for binding in bindings {
    println!("Bound to: {:016X}", binding.dst_address);
}
```

**Remove Binding:**
```rust
aps_manager.remove_binding(
    src_endpoint: 1,
    cluster_id: 0x0006,
    dst_address: 0x0011223344556677,
    dst_endpoint: 1,
)?;
```

#### Group Management

**Add to Group:**
```rust
aps_manager.add_group(group_address: 0x0001, endpoint: 1)?;
```

**Check Membership:**
```rust
if aps_manager.is_group_member(0x0001, 1) {
    println!("Member of group 0x0001");
}
```

**Remove from Group:**
```rust
aps_manager.remove_group(0x0001, 1)?;
```

#### Fragmentation

**Fragment Large Payload:**
```rust
let large_payload = [0u8; 200];
let fragments = aps_manager.fragment_payload(&large_payload, max_size: 82)?;

for (i, fragment) in fragments.iter().enumerate() {
    let mut frame = ApsDataFrame::new(...)?;
    frame.extended_header = Some(ApsExtendedHeader {
        fragment_number: i as u8,
        fragment_count: fragments.len() as u8,
        block_number: 0,
    });
    frame.frame_control.extended_header = true;
    // Send fragment
}
```

**Process Received Fragment:**
```rust
if let Ok(Some(complete_payload)) = aps_manager.process_fragment(
    src_address: 0x0001,
    frame: &aps_frame,
    timestamp: get_current_time(),
) {
    println!("Message reassembled: {} bytes", complete_payload.len());
}
```

**Cleanup Old Fragments:**
```rust
// Call periodically (e.g., every second)
aps_manager.cleanup_fragments(current_time, timeout_ms: 5000);
```

#### Acknowledgment Tracking

**Track Pending ACK:**
```rust
aps_manager.add_pending_ack(dst_addr: 0x0001, aps_counter: 42)?;
```

**Check if ACK Pending:**
```rust
if aps_manager.is_ack_pending(0x0001, 42) {
    // Retransmit or timeout
}
```

**Remove Pending ACK:**
```rust
aps_manager.remove_pending_ack(0x0001, 42);
```

## Integration with Zigbee Driver

The APS layer is integrated into the main Zigbee driver:

### Sending APS Data

```rust
// Using the Zigbee driver's APS API
zigbee.send_aps_data(
    dest: 0x0001,
    dst_endpoint: 1,
    src_endpoint: 1,
    cluster_id: 0x0006,
    profile_id: 0x0104,
    data: &[0x01, 0x00],
    ack_request: true,
)?;
```

### Binding via Zigbee Driver

```rust
// Bind switch (endpoint 1) to light (0x1122334455667788, endpoint 1)
zigbee.bind(
    source_endpoint: 1,
    cluster: 0x0006,  // On/Off cluster
    dest_address: 0x1122334455667788,
    dest_endpoint: 1,
)?;
```

### Group Management via Zigbee Driver

```rust
// Add endpoint to group
zigbee.add_group(group_address: 0x0001, endpoint: 1)?;

// Send to group
zigbee.send_group_message(
    group_address: 0x0001,
    dst_endpoint: 1,
    src_endpoint: 1,
    cluster_id: 0x0006,
    profile_id: 0x0104,
    data: &[0x01],  // Turn on
)?;
```

## Common Patterns

### Pattern 1: Reliable Unicast

```rust
// Send with ACK request
let mut frame = ApsDataFrame::new(...)?.with_ack_request();
frame.aps_counter = aps_manager.next_counter();

// Track for retransmission
aps_manager.add_pending_ack(dest_addr, frame.aps_counter)?;

// Send
let encoded = frame.encode()?;
// ... transmit via network layer

// On ACK received:
aps_manager.remove_pending_ack(dest_addr, aps_counter);

// On timeout:
if aps_manager.is_ack_pending(dest_addr, aps_counter) {
    // Retransmit up to 3 times
}
```

### Pattern 2: Group Broadcast

```rust
// Add devices to group
aps_manager.add_group(0x0001, endpoint: 1)?;

// Send to entire group
let frame = ApsDataFrame::new_group(
    group_address: 0x0001,
    dst_endpoint: 1,
    src_endpoint: 1,
    cluster_id: 0x0006,
    profile_id: 0x0104,
    payload: &[0x01],  // On command
)?;

// No ACK for group messages
```

### Pattern 3: Binding-Based Communication

```rust
// Setup: Create binding on sender
aps_manager.add_binding(ApsBinding::new(
    src_endpoint: 1,
    cluster_id: 0x0006,
    dst_address: light_ieee_addr,
    dst_endpoint: 1,
))?;

// Send: Get bindings and send to each
let bindings = aps_manager.get_bindings(1, 0x0006);
for binding in bindings {
    let frame = ApsDataFrame::new(
        dst_endpoint: binding.dst_endpoint,
        src_endpoint: 1,
        cluster_id: 0x0006,
        profile_id: 0x0104,
        payload: data,
    )?;
    // Send to binding.dst_address
}
```

### Pattern 4: Large Message with Fragmentation

```rust
let large_data = [0u8; 200];

// Fragment
let fragments = aps_manager.fragment_payload(&large_data, 82)?;

// Send each fragment
for (i, fragment_data) in fragments.iter().enumerate() {
    let mut frame = ApsDataFrame::new(
        dst_endpoint: 1,
        src_endpoint: 1,
        cluster_id: 0x0006,
        profile_id: 0x0104,
        payload: fragment_data,
    )?;
    
    frame.extended_header = Some(ApsExtendedHeader {
        fragment_number: i as u8,
        fragment_count: fragments.len() as u8,
        block_number: 0,
    });
    frame.frame_control.extended_header = true;
    
    // Send fragment
}

// Receiver automatically reassembles via process_fragment()
```

## Performance Considerations

### Frame Overhead

**APS Frame Header:**
- Minimum: 8 bytes (unicast, no fragmentation)
- With group: 10 bytes
- With fragmentation: 10 bytes
- Total overhead: 8-10 bytes per frame

**Maximum Payload:**
- Without fragmentation: 82 bytes (typical)
- With fragmentation: Limited by implementation (256 bytes in current)

### Fragmentation Trade-offs

**Benefits:**
- Support for large messages
- Transparent to application

**Costs:**
- Multiple radio transmissions
- Reassembly buffer memory
- Timeout management overhead
- Potential for partial message loss

**Recommendations:**
- Keep messages ≤82 bytes when possible
- Use fragmentation only when necessary
- Set appropriate timeout values
- Cleanup old fragments regularly

### Binding Table Size

**Current Limits:**
- Max bindings: 16
- Max groups: 16

**Optimization:**
- Use groups instead of multiple bindings when possible
- Remove unused bindings
- Prioritize critical bindings

## Testing

Comprehensive tests are included in `aps.rs`:

```rust
#[test]
fn test_frame_control_encode_decode()
#[test]
fn test_data_frame_encode_decode()
#[test]
fn test_fragmentation()
#[test]
fn test_binding_management()
#[test]
fn test_group_management()
```

Run tests:
```bash
cargo test --lib zigbee::aps
```

## Future Enhancements

1. **Security Integration**: APS encryption/decryption
2. **Retry Logic**: Automatic retransmission with exponential backoff
3. **Flow Control**: Throttle transmission rate
4. **Priority Queues**: High/low priority message queues
5. **Persistent Binding Table**: Store bindings in flash
6. **Enhanced Fragmentation**: Block acknowledgments for large messages

## References

- Zigbee Specification (Revision 22): Section 2.2 (APS Layer)
- Zigbee PRO Feature Set: APS Security
- IEEE 802.15.4: MAC layer integration

## Summary

The APS layer provides essential application support services:
- ✅ Data transfer with multiple delivery modes
- ✅ Fragmentation for large messages
- ✅ Binding management for device pairing
- ✅ Group management for multicast
- ✅ Acknowledgment tracking for reliability
- ✅ Integration with Zigbee driver

This implementation provides a solid foundation for Zigbee application development on ESP32-C6/H2 platforms.
