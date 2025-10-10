# Zigbee Driver for ESP32-C6 and ESP32-H2

A comprehensive Zigbee protocol stack implementation for ESP32-C6 and ESP32-H2 microcontrollers.

## Overview

This driver provides a complete Zigbee implementation supporting:
- **Coordinator** - Form and manage Zigbee networks
- **Router** - Route packets and extend network coverage  
- **End Device** - Join networks as sleepy or non-sleepy leaf nodes

## Features

✅ **Multiple Device Roles**
- Coordinator (trust center, network formation)
- Router (packet routing, child management)
- End Device (sleepy and non-sleepy modes)

✅ **Zigbee Cluster Library (ZCL)**
- On/Off cluster
- Level Control cluster
- Temperature Measurement cluster
- Extensible architecture for custom clusters

✅ **Network Management**
- Network formation and joining
- Device discovery and commissioning
- Binding and group management
- Neighbor and routing tables

✅ **Security**
- AES-128 encryption
- Network and link keys
- Install code support
- Trust center functionality

✅ **Programming Models**
- Blocking API for simple applications
- Async API for concurrent operations

✅ **Chip Support**
- ESP32-C6 (2.4 GHz IEEE 802.15.4)
- ESP32-H2 (2.4 GHz IEEE 802.15.4)

## Quick Start

### Coordinator Example

```rust
use esp_hal::zigbee::{Zigbee, Config, Role};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Create coordinator
    let mut zigbee = Zigbee::new(
        peripherals.IEEE802154,
        Config::coordinator()
            .with_channel(15)
            .with_pan_id(0x1234)
    );
    
    // Form network
    zigbee.form_network().expect("Failed to form network");
    
    // Permit devices to join for 60 seconds
    zigbee.permit_join(60).expect("Failed to permit join");
    
    // Main loop
    loop {
        if let Some(event) = zigbee.poll() {
            match event {
                ZigbeeEvent::DeviceJoined { network_address, ieee_address } => {
                    println!("Device joined: 0x{:04X} (IEEE: 0x{:016X})", 
                        network_address, ieee_address);
                }
                ZigbeeEvent::DataReceived { source, data, .. } => {
                    println!("Data from 0x{:04X}: {:?}", source, data);
                }
                _ => {}
            }
        }
    }
}
```

### End Device Example

```rust
use esp_hal::zigbee::{Zigbee, Config, Role};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Create end device (non-sleepy)
    let mut zigbee = Zigbee::new(
        peripherals.IEEE802154,
        Config::end_device(false)
            .with_channel(15)
    );
    
    // Join network
    zigbee.join_network().expect("Failed to join");
    
    // Send data to coordinator
    let data = [0x01, 0x02, 0x03];
    zigbee.send_data(0x0000, &data).expect("Failed to send");
    
    loop {
        if let Some(event) = zigbee.poll() {
            // Handle events
        }
    }
}
```

### Router Example

```rust
use esp_hal::zigbee::{Zigbee, Config, Role};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Create router
    let mut zigbee = Zigbee::new(
        peripherals.IEEE802154,
        Config::router()
            .with_channel(15)
            .with_max_children(20)
    );
    
    // Join network
    zigbee.join_network().expect("Failed to join");
    
    // Permit children to join
    zigbee.permit_join(255).expect("Failed to permit join");
    
    loop {
        if let Some(event) = zigbee.poll() {
            // Handle events
        }
    }
}
```

### Async Example

```rust
use esp_hal::zigbee::{Zigbee, Config, Role};
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    let mut zigbee = Zigbee::new_async(
        peripherals.IEEE802154,
        Config::coordinator()
    );
    
    // Form network asynchronously
    zigbee.form_network().await.expect("Failed to form network");
    
    loop {
        let event = zigbee.wait_event().await;
        // Handle event
    }
}
```

## Architecture

```text
┌──────────────────────────────────────────┐
│       Application Layer                   │
│  (User Code, ZCL Clusters)               │
├──────────────────────────────────────────┤
│  Zigbee Cluster Library (ZCL)            │
│  - OnOffCluster                          │
│  - LevelControlCluster                   │
│  - TemperatureMeasurementCluster         │
│  - Custom clusters                       │
├──────────────────────────────────────────┤
│  Zigbee Device Objects (ZDO)             │
│  - Device & Service Discovery            │
│  - Binding                               │
│  - Network Management                    │
├──────────────────────────────────────────┤
│  Application Support (APS) Layer         │
│  - Group Management                      │
│  - Binding Table                         │
├──────────────────────────────────────────┤
│  Network (NWK) Layer                     │
│  - Routing                               │
│  - Network Formation/Joining             │
│  - Security                              │
├──────────────────────────────────────────┤
│  IEEE 802.15.4 MAC Layer                 │
│  (esp-radio)                             │
├──────────────────────────────────────────┤
│  IEEE 802.15.4 PHY Layer                 │
│  (Hardware Radio)                        │
└──────────────────────────────────────────┘
```

## Device Roles

### Coordinator

The coordinator is responsible for:
- **Forming the network** - Selects channel and PAN ID
- **Trust center** - Manages security keys
- **Permitting joins** - Controls which devices can join
- **Network management** - Allocates addresses

Only one coordinator per network.

### Router

Routers provide:
- **Packet routing** - Forward packets between devices
- **Network extension** - Expand network coverage
- **Child support** - Allow end devices to join as children
- **Always on** - Must be mains powered

### End Device

End devices are leaf nodes that:
- **Cannot route** - Don't forward packets
- **Cannot have children** - Leaf nodes only
- **Can sleep** - Battery powered devices
- **Simplest role** - Minimal resource requirements

## Configuration

### Role Configuration

```rust
// Coordinator
let config = Config::coordinator()
    .with_channel(15)
    .with_pan_id(0x1234);

// Router
let config = Config::router()
    .with_channel(15)
    .with_max_children(20);

// End Device (sleepy)
let config = Config::end_device(true)
    .with_channel(15)
    .with_poll_rate(1000); // Poll every 1 second

// End Device (non-sleepy)
let config = Config::end_device(false)
    .with_channel(15);
```

### Security Configuration

```rust
let config = Config::default()
    .with_security(true)
    .with_security_level(SecurityLevel::Standard)
    .with_network_key([0x01, 0x02, ..., 0x10]);
```

### Advanced Configuration

```rust
let config = Config::default()
    .with_role(Role::Coordinator)
    .with_channel(15)
    .with_pan_id(0x1234)
    .with_extended_pan_id(0x0123456789ABCDEF)
    .with_tx_power(10) // dBm
    .with_max_children(32)
    .with_security(true);
```

## Zigbee Cluster Library (ZCL)

### Using Built-in Clusters

```rust
use esp_hal::zigbee::zcl::{OnOffCluster, Cluster};

let mut on_off = OnOffCluster::new();

// Turn on
on_off.turn_on();

// Check state
if on_off.is_on() {
    println!("Light is on");
}

// Handle command
on_off.handle_command(0x02, &[]).unwrap(); // Toggle
```

### Implementing Custom Clusters

```rust
use esp_hal::zigbee::zcl::{Cluster, ClusterId, AttributeId, AttributeValue};

struct MyCustomCluster {
    value: u16,
}

impl Cluster for MyCustomCluster {
    fn cluster_id(&self) -> ClusterId {
        0xFF00 // Custom cluster ID
    }
    
    fn handle_command(&mut self, command: u8, data: &[u8]) -> Result<(), ZclError> {
        // Handle custom commands
        Ok(())
    }
    
    fn read_attribute(&self, attr_id: AttributeId) -> Result<AttributeValue, ZclError> {
        match attr_id {
            0x0000 => Ok(AttributeValue::Uint16(self.value)),
            _ => Err(ZclError::InvalidAttribute),
        }
    }
    
    fn write_attribute(&mut self, attr_id: AttributeId, value: AttributeValue) -> Result<(), ZclError> {
        // Handle attribute writes
        Ok(())
    }
}
```

## Network Management

### Scanning for Networks

```rust
let networks = zigbee.scan_networks().expect("Scan failed");

for network in networks.iter() {
    println!("PAN ID: 0x{:04X}, Channel: {}, LQI: {}", 
        network.pan_id, network.channel, network.lqi);
}
```

### Device Management (Coordinator)

```rust
// Get all devices
let device_count = zigbee.device_count();

// Permit joining
zigbee.permit_join(60).expect("Failed");

// Close network
zigbee.permit_join(0).expect("Failed");
```

### Binding

```rust
// Bind two devices
zigbee.bind(
    1, // source endpoint
    0x0006, // On/Off cluster
    0x0123456789ABCDEF, // destination IEEE address
    1, // destination endpoint
).expect("Bind failed");
```

### Link Quality

```rust
// Get LQI for a device
if let Some(lqi) = zigbee.get_lqi(0x1234) {
    println!("LQI: {}", lqi);
}

// Get RSSI
if let Some(rssi) = zigbee.get_rssi(0x1234) {
    println!("RSSI: {} dBm", rssi);
}
```

## Security

### Network Keys

```rust
// Set network key
let network_key = [0x01, 0x02, /* ... */, 0x10];
config = config.with_network_key(network_key);

// Or generate random key (coordinator)
// Key is automatically generated during network formation
```

### Install Codes

```rust
// Add install code for a joining device
let install_code = [0xAA, 0xBB, /* ... */, 0xFF]; // 16 bytes + 2 byte CRC
zigbee.add_install_code(0x0123456789ABCDEF, install_code)
    .expect("Failed to add install code");
```

## Events

```rust
match event {
    ZigbeeEvent::NetworkFormed { pan_id, channel, .. } => {
        println!("Network formed on channel {}", channel);
    }
    
    ZigbeeEvent::NetworkJoined { network_address, .. } => {
        println!("Joined as 0x{:04X}", network_address);
    }
    
    ZigbeeEvent::DeviceJoined { network_address, ieee_address } => {
        println!("Device 0x{:04X} joined", network_address);
    }
    
    ZigbeeEvent::DeviceLeft { network_address } => {
        println!("Device 0x{:04X} left", network_address);
    }
    
    ZigbeeEvent::DataReceived { source, endpoint, cluster, data } => {
        println!("Data from 0x{:04X}", source);
    }
    
    ZigbeeEvent::ZclCommand { source, cluster, command, .. } => {
        println!("ZCL command {} from 0x{:04X}", command, source);
    }
    
    ZigbeeEvent::NetworkError(err) => {
        println!("Error: {:?}", err);
    }
    
    ZigbeeEvent::LinkQualityUpdate { address, lqi, rssi } => {
        println!("0x{:04X}: LQI={}, RSSI={}", address, lqi, rssi);
    }
}
```

## Hardware Setup

### ESP32-C6

- **IEEE 802.15.4 Radio**: Built-in 2.4 GHz
- **TX Power**: Up to +20 dBm
- **RX Sensitivity**: -102 dBm
- **Operating Channels**: 11-26

### ESP32-H2

- **IEEE 802.15.4 Radio**: Built-in 2.4 GHz
- **TX Power**: Up to +20 dBm  
- **RX Sensitivity**: -105 dBm
- **Operating Channels**: 11-26

### Antenna Considerations

- Use appropriate 2.4 GHz antenna
- PCB trace antennas work for short range
- External antennas recommended for extended range
- Ensure proper RF matching

## Channel Selection

Zigbee uses IEEE 802.15.4 channels 11-26 (2.4 GHz band):

| Channel | Frequency (MHz) | WiFi Overlap |
|---------|----------------|--------------|
| 11 | 2405 | WiFi 1 |
| 15 | 2425 | WiFi 3-4 |
| 20 | 2450 | WiFi 7-8 |
| 25 | 2475 | WiFi 11-12 |
| 26 | 2480 | WiFi 13 |

**Recommendation**: Use channels 15, 20, or 25 to minimize WiFi interference.

## Power Consumption

### Coordinator/Router
- Always on: ~60-80 mA (active RX)
- Cannot sleep

### End Device (Non-Sleepy)
- Always on: ~60-80 mA (active RX)
- Suitable for mains powered devices

### End Device (Sleepy)
- Active: ~60-80 mA
- Sleep: ~20 µA (deep sleep)
- Average depends on poll rate
- Suitable for battery powered devices

## Limitations

- **Coexistence**: Cannot run simultaneously with WiFi or Bluetooth on same chip
- **Range**: Typical 10-100m depending on environment
- **Data Rate**: 250 kbps (IEEE 802.15.4)
- **Network Size**: Up to 65,000 devices (practical limit ~100-200 per coordinator)

## Troubleshooting

### Cannot Form Network

- Check channel is clear (use network scanner)
- Ensure no other coordinator on same PAN ID
- Verify antenna connection

### Cannot Join Network

- Ensure coordinator permits joining
- Check channel matches coordinator
- Verify security settings match
- Check signal strength (RSSI)

### Poor Link Quality

- Reduce distance between devices
- Add routers to extend network
- Change channel to avoid interference
- Check antenna

### Device Keeps Dropping

- Check power supply (brownout)
- Verify poll rate for sleepy devices
- Check for RF interference
- Verify child timeout settings

## Examples

See the `examples/zigbee/` directory for:
- Coordinator setup
- End device (sleepy and non-sleepy)
- Router configuration
- ZCL cluster usage
- Custom clusters
- Binding examples
- Security configuration

## API Reference

Full API documentation is available in the source files:
- `mod.rs` - Main driver API
- `config.rs` - Configuration structures
- `coordinator.rs` - Coordinator functionality
- `device.rs` - End device and router functionality
- `network.rs` - Network management
- `security.rs` - Security and encryption
- `zcl.rs` - Zigbee Cluster Library
- `zdo.rs` - Zigbee Device Objects

## References

- [Zigbee Specification](https://zigbeealliance.org/)
- [IEEE 802.15.4 Standard](https://standards.ieee.org/standard/802_15_4-2020.html)
- [ESP32-C6 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c6_datasheet_en.pdf)
- [ESP32-H2 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-h2_datasheet_en.pdf)

## License

This driver is part of esp-hal and follows the same licensing.
