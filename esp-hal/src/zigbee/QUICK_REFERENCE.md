# Zigbee Radio Integration - Quick Reference

## Getting Started

### Initialize Zigbee with Radio

```rust
use esp_hal::{
    peripherals::Peripherals,
    zigbee::{Zigbee, Config, Role},
};

let peripherals = Peripherals::take();

// Coordinator
let mut coordinator = Zigbee::new(
    peripherals.IEEE802154,
    Config::default()
        .with_role(Role::Coordinator)
        .with_channel(15)              // 2420 MHz
        .with_pan_id(Some(0x1234))
        .with_tx_power(20)             // 20 dBm (max power)
);

// End Device
let mut device = Zigbee::new(
    peripherals.IEEE802154,
    Config::default()
        .with_role(Role::EndDevice)
        .with_channel(15)
        .with_tx_power(10)             // 10 dBm (medium power)
);
```

## Network Operations

### Form Network (Coordinator Only)

```rust
// Form a new Zigbee network
coordinator.form_network()?;

// Network is now operational
// Coordinator address: 0x0000
// PAN ID: As configured (or auto-generated)
```

**What Happens:**
1. Radio configured as coordinator
2. PAN ID assigned
3. Address set to 0x0000
4. Beacon transmission enabled
5. Starts receiving frames
6. `NetworkFormed` event generated

### Join Network (Device/Router)

```rust
// Option 1: Join specific network
let mut device = Zigbee::new(
    peripherals.IEEE802154,
    Config::default()
        .with_role(Role::EndDevice)
        .with_channel(15)
        .with_pan_id(Some(0x1234))     // Specific network
);
device.join_network()?;

// Option 2: Scan and join first available
let mut device = Zigbee::new(
    peripherals.IEEE802154,
    Config::default()
        .with_role(Role::EndDevice)
        .with_channel(15)              // No PAN ID = auto-scan
);
device.join_network()?;                // Scans and joins
```

**What Happens:**
1. Scans for networks (if PAN ID not specified)
2. Configures radio with target PAN ID
3. Sends association request (simplified in current version)
4. Receives short address assignment
5. Starts receiving frames
6. `NetworkJoined` event generated

### Scan for Networks

```rust
// Scan all channels for available networks
let networks = device.scan_networks()?;

for network in &networks {
    println!("Found PAN ID: 0x{:04X} on channel {}",
             network.pan_id, network.channel);
    println!("  Extended PAN ID: 0x{:016X}", network.extended_pan_id);
}

// Use first network
if let Some(first) = networks.first() {
    // Configure with discovered network
    // Then join
}
```

**Scan Details:**
- Scans channels 11-26 (2.4 GHz)
- 100ms per channel
- Total scan time: ~1.6 seconds
- Returns up to 16 networks

## Data Communication

### Send Data

```rust
// Send to specific device
let dest_addr = 0x0001;
let data = b"Hello, device!";
zigbee.send_data(dest_addr, data)?;

// Send to coordinator
zigbee.send_data(0x0000, b"Message to coordinator")?;

// Broadcast to all devices
zigbee.send_data(0xFFFF, b"Broadcast message")?;
```

**Limitations:**
- Max payload: 100 bytes (Zigbee application data)
- Must be joined to network
- No automatic retries (in current version)
- Blocking call (returns when transmitted)

### Receive Data (Event Loop)

```rust
loop {
    if let Some(event) = zigbee.poll() {
        match event {
            ZigbeeEvent::DataReceived { source, data, lqi, rssi } => {
                println!("From: 0x{:04X}", source);
                println!("Data: {:?}", data);
                println!("LQI: {} (0-255)", lqi);
                println!("RSSI: {} dBm", rssi);
                
                // Echo back
                zigbee.send_data(source, &data)?;
            }
            ZigbeeEvent::NetworkFormed { pan_id, channel, .. } => {
                println!("Network formed: PAN 0x{:04X} on channel {}", 
                         pan_id, channel);
            }
            ZigbeeEvent::NetworkJoined { network_address, .. } => {
                println!("Joined as address: 0x{:04X}", network_address);
            }
            ZigbeeEvent::BeaconReceived { pan_id, lqi, rssi } => {
                println!("Beacon from PAN 0x{:04X}, LQI: {}, RSSI: {}", 
                         pan_id, lqi, rssi);
            }
            _ => {}
        }
    }
    
    // Don't burn CPU
    // In real code, use proper delays or async
}
```

## Configuration

### Channels

```rust
// Set channel (11-26)
Config::default().with_channel(15)     // 2420 MHz
Config::default().with_channel(11)     // 2405 MHz (lowest)
Config::default().with_channel(26)     // 2480 MHz (highest)
```

**Channel to Frequency:**
- Channel = 11 + n
- Frequency = 2405 + (n × 5) MHz
- Example: Channel 15 = 2405 + (4 × 5) = 2425 MHz

### TX Power

```rust
// Set transmit power (-40 to +20 dBm)
Config::default().with_tx_power(20)    // Max power (100 mW)
Config::default().with_tx_power(10)    // Medium (10 mW)
Config::default().with_tx_power(0)     // Low (1 mW)
Config::default().with_tx_power(-10)   // Very low (0.1 mW)

// Or adjust dynamically
zigbee.set_tx_power(15)?;
```

**Power Guidelines:**
- Higher power = longer range, more energy use
- Lower power = shorter range, less energy use
- 20 dBm: ~100m indoor, 300m+ outdoor
- 10 dBm: ~50m indoor, 150m outdoor
- 0 dBm: ~20m indoor, 50m outdoor

### Addressing

```rust
// Short address (assigned during join)
// Coordinator: 0x0000
// Devices: 0x0001-0xFFF7
// Broadcast: 0xFFFF

// Extended address (IEEE MAC)
Config::default()
    .with_ieee_address(Some(0x0011223344556677))

// Get current address
if let Some(addr) = zigbee.network_address() {
    println!("My address: 0x{:04X}", addr);
}
```

## Advanced Usage

### Direct Radio Access

```rust
use esp_hal::zigbee::{Radio, RadioFrame, FrameType, Address};

// If you need direct radio control
// (Usually not needed, use Zigbee API instead)
let mut radio = Radio::new(
    peripherals.IEEE802154,
    15,                           // Channel
    Some(0x1234),                 // PAN ID
    Some(0x0001),                 // Short address
);

// Transmit custom frame
radio.transmit_data(
    0x1234,                       // Dest PAN ID
    Address::Short(0x0000),       // Dest address
    Address::Short(0x0001),       // Src address
    b"Custom data",               // Payload
    42,                           // Sequence number
)?;

// Receive frame
if let Ok(Some(frame)) = radio.receive() {
    println!("Received frame type: {:?}", frame.frame_type);
    println!("Payload: {:?}", frame.payload);
}
```

### Energy Detection Scan

```rust
use esp_hal::zigbee::radio::perform_energy_detection;

// Find quietest channel
let mut best_channel = 11;
let mut best_energy = 255;

for channel in 11..=26 {
    let energy = perform_energy_detection(&mut radio, channel)?;
    if energy < best_energy {
        best_energy = energy;
        best_channel = channel;
    }
}

println!("Best channel: {} (energy: {})", best_channel, best_energy);
```

## Error Handling

```rust
use esp_hal::zigbee::NetworkError;

match zigbee.form_network() {
    Ok(()) => println!("Network formed"),
    Err(NetworkError::FormFailed) => println!("Failed to form"),
    Err(NetworkError::InvalidParameter) => println!("Bad config"),
    Err(e) => println!("Error: {:?}", e),
}
```

**Common Errors:**
- `FormFailed`: Network formation failed
- `JoinFailed`: Unable to join network
- `NoNetworkFound`: No networks discovered during scan
- `TransmissionFailed`: Frame transmission failed
- `InvalidParameter`: Invalid configuration or parameter
- `NotJoined`: Tried to send data before joining

## Performance Tips

### Polling Frequency

```rust
// Don't poll too fast (wastes CPU)
// Don't poll too slow (misses events)

// Good: 10-100 Hz
loop {
    if let Some(event) = zigbee.poll() {
        handle_event(event);
    }
    delay_ms(10);  // 100 Hz
}

// Better: Use async with proper delay
async {
    loop {
        if let Some(event) = zigbee.wait_event().await {
            handle_event(event);
        }
    }
}
```

### Network Scanning

```rust
// Scan only once, cache results
let networks = zigbee.scan_networks()?;

// Don't scan repeatedly
// BAD:
loop {
    let networks = zigbee.scan_networks()?;  // Slow!
    // ...
}
```

### Data Transmission

```rust
// Keep payloads small
// Max: 100 bytes for Zigbee data
// Optimal: 40-60 bytes

// BAD: Too large
let data = [0u8; 200];  // Will fail!

// GOOD: Reasonable size
let data = b"Short message";
```

## Debugging

### Enable Logging

```rust
// Check network status
if let Some(info) = zigbee.network_info() {
    println!("Network address: 0x{:04X}", info.network_address);
    println!("PAN ID: 0x{:04X}", info.pan_id);
    println!("Channel: {}", info.channel);
}

// Check IEEE address
if let Some(addr) = zigbee.ieee_address() {
    println!("IEEE address: 0x{:016X}", addr);
}
```

### Monitor Link Quality

```rust
// Track LQI and RSSI per frame
match zigbee.poll() {
    Some(ZigbeeEvent::DataReceived { source, lqi, rssi, .. }) => {
        println!("Device 0x{:04X}: LQI={}, RSSI={}dBm", 
                 source, lqi, rssi);
        
        if rssi < -80 {
            println!("WARNING: Weak signal from 0x{:04X}", source);
        }
    }
    _ => {}
}
```

### Raw Frame Inspection

```rust
// Use radio directly for debugging
if let Ok(Some(raw)) = radio.raw_received() {
    println!("Raw frame: {:?}", raw);
}
```

## Common Patterns

### Simple Coordinator

```rust
// Form network and accept devices
let mut coord = Zigbee::new(peripherals.IEEE802154, 
                             Config::coordinator().with_channel(15));
coord.form_network()?;

loop {
    if let Some(event) = coord.poll() {
        match event {
            ZigbeeEvent::DeviceJoined { network_address, .. } => {
                println!("Device 0x{:04X} joined", network_address);
            }
            ZigbeeEvent::DataReceived { source, data, .. } => {
                // Echo back
                coord.send_data(source, &data)?;
            }
            _ => {}
        }
    }
}
```

### Simple End Device

```rust
// Join and send periodic data
let mut device = Zigbee::new(peripherals.IEEE802154,
                              Config::end_device().with_channel(15));
device.join_network()?;

let mut counter = 0u32;
loop {
    // Send to coordinator every 5 seconds
    let msg = format!("Counter: {}", counter);
    device.send_data(0x0000, msg.as_bytes())?;
    counter += 1;
    
    // Process incoming
    while let Some(event) = device.poll() {
        if let ZigbeeEvent::DataReceived { data, .. } = event {
            println!("Received: {:?}", data);
        }
    }
    
    delay_ms(5000);
}
```

### Network Coordinator with Multiple Devices

```rust
use heapless::FnvIndexMap;

let mut coord = Zigbee::new(peripherals.IEEE802154,
                             Config::coordinator().with_channel(15));
coord.form_network()?;

// Track devices
let mut devices: FnvIndexMap<u16, (u8, i8), 16> = FnvIndexMap::new();

loop {
    if let Some(event) = coord.poll() {
        match event {
            ZigbeeEvent::DeviceJoined { network_address, .. } => {
                println!("Device 0x{:04X} joined", network_address);
                devices.insert(network_address, (0, -128)).ok();
            }
            ZigbeeEvent::DataReceived { source, data, lqi, rssi } => {
                // Update device info
                devices.insert(source, (lqi, rssi)).ok();
                
                // Broadcast to all devices
                for (&addr, _) in devices.iter() {
                    if addr != source {
                        coord.send_data(addr, &data).ok();
                    }
                }
            }
            _ => {}
        }
    }
}
```

## Next Steps

1. **Read Full Documentation**: See `RADIO_INTEGRATION.md` for complete details
2. **Run Examples**: Test with actual ESP32-C6/H2 hardware
3. **Implement Security**: Add frame encryption when needed
4. **Add Routing**: Implement multi-hop for larger networks
5. **Optimize Power**: Implement sleep modes for battery devices

## Reference

- **Channels**: 11-26 (2.4 GHz)
- **Max Payload**: 100 bytes (application data)
- **TX Power Range**: -40 to +20 dBm
- **Coordinator Address**: Always 0x0000
- **Broadcast Address**: 0xFFFF
- **Scan Time**: ~1.6 seconds (all channels)
- **Throughput**: ~80-120 kbps (practical)
