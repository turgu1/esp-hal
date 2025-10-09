# I2C Slave Driver - Quick Start Guide

This guide will help you get started with the I2C slave driver in 5 minutes.

## 1. Prerequisites

- ESP32 development board (any variant)
- Rust toolchain installed
- esp-hal project set up
- Basic knowledge of I2C protocol

## 2. Minimal Example

```rust
#![no_std]
#![no_main]

use esp_hal::{
    i2c::slave::{Config, I2c},
    prelude::*,
};
use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Create I2C slave at address 0x55
    let mut i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to create I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    let mut buffer = [0u8; 64];

    loop {
        // Wait for master to send data
        if let Ok(bytes) = i2c.read(&mut buffer) {
            if bytes > 0 {
                // Echo data back to master
                let _ = i2c.write(&buffer[..bytes]);
            }
        }
    }
}
```

## 3. Hardware Setup

```
Master Device          ESP32 (Slave)
    VCC ─────┬──────────── VCC
             │
            4.7kΩ (pull-up)
             │
    SDA ─────┴──────────── GPIO1
    
    VCC ─────┬
             │
            4.7kΩ (pull-up)
             │
    SCL ─────┴──────────── GPIO2
    
    GND ──────────────────── GND
```

**Important:** Don't forget pull-up resistors on SDA and SCL!

## 4. Build and Flash

```bash
# Build
cargo build --release

# Flash (adjust target and port)
probe-rs run --chip esp32c3

# Or using espflash
espflash flash target/riscv32imc-unknown-none-elf/release/your-project
```

## 5. Configuration Options

### Change Address

```rust
let config = Config::default().with_address(0x42.into());
```

### Enable Clock Stretching

```rust
let config = Config::default().with_clock_stretch_enable(true);
```

### Configure Filters

```rust
let config = Config::default()
    .with_sda_filter_enable(true)
    .with_sda_filter_threshold(7)
    .with_scl_filter_enable(true)
    .with_scl_filter_threshold(7);
```

## 6. Common Patterns

### Echo Server

```rust
loop {
    let mut buffer = [0u8; 64];
    if let Ok(bytes) = i2c.read(&mut buffer) {
        if bytes > 0 {
            let _ = i2c.write(&buffer[..bytes]);
        }
    }
}
```

### Register-Based Interface

```rust
let mut register_addr = 0u8;
let mut registers = [0u8; 256];

loop {
    // First read is register address
    if let Ok(1) = i2c.read(core::slice::from_mut(&mut register_addr)) {
        // Next operation depends on master
        // For write: read data into register
        // For read: write register data
        let _ = i2c.write(&[registers[register_addr as usize]]);
    }
}
```

### Using Async

```rust
let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2)
    .into_async();

loop {
    let mut buffer = [0u8; 64];
    if let Ok(bytes) = i2c.read_async(&mut buffer).await {
        if bytes > 0 {
            let _ = i2c.write_async(&buffer[..bytes]).await;
        }
    }
}
```

## 7. Troubleshooting

### Slave Not Responding

**Problem:** Master reports no ACK from slave

**Solutions:**
1. Check pull-up resistors are installed
2. Verify address matches (remember: 7-bit, right-aligned)
3. Check pin connections
4. Enable clock stretching if timing is tight

**Test:**
```rust
// Print when slave is addressed
i2c.listen(Event::SlaveAddressed);
loop {
    let events = i2c.interrupts();
    if events.contains(Event::SlaveAddressed) {
        esp_println::println!("Addressed!");
        i2c.clear_interrupts(events);
    }
}
```

### Data Corruption

**Problem:** Received data is incorrect

**Solutions:**
1. Check signal integrity with oscilloscope/logic analyzer
2. Increase filter thresholds if noisy environment
3. Enable clock stretching
4. Reduce bus speed at master

**Test:**
```rust
// Verify each byte
let mut buffer = [0u8; 64];
let bytes = i2c.read(&mut buffer)?;
for i in 0..bytes {
    esp_println::println!("Byte {}: 0x{:02X}", i, buffer[i]);
}
```

### FIFO Overflow

**Problem:** `Error::FifoExceeded` error

**Solutions:**
1. Process data faster
2. Use clock stretching
3. Limit transfer size to 32 bytes
4. Use multiple transactions

**Test:**
```rust
// Never try to write more than FIFO size
const MAX_FIFO: usize = 32;
if data.len() > MAX_FIFO {
    // Split into chunks
    for chunk in data.chunks(MAX_FIFO) {
        i2c.write(chunk)?;
    }
}
```

## 8. Testing with Arduino (as Master)

```cpp
// Arduino code to test ESP32 slave
#include <Wire.h>

#define SLAVE_ADDR 0x55

void setup() {
  Wire.begin();
  Serial.begin(115200);
}

void loop() {
  // Write test
  Wire.beginTransmission(SLAVE_ADDR);
  Wire.write("Hello");
  Wire.endTransmission();
  
  delay(10);
  
  // Read test
  Wire.requestFrom(SLAVE_ADDR, 5);
  while(Wire.available()) {
    char c = Wire.read();
    Serial.print(c);
  }
  Serial.println();
  
  delay(1000);
}
```

## 9. Next Steps

Once you have basic communication working:

1. **Read the full documentation:**
   - `README.md` - Comprehensive user guide
   - `DESIGN.md` - Understanding the implementation
   - `EXAMPLE.md` - More complex examples

2. **Try advanced features:**
   - Interrupt-driven I/O
   - Async operations with Embassy
   - Error handling and recovery

3. **Implement your protocol:**
   - Register-based access
   - Command/response patterns
   - Custom protocols

4. **Optimize performance:**
   - Adjust filter settings
   - Tune clock stretching
   - Measure and optimize throughput

## 10. Getting Help

If you're stuck:

1. **Check the troubleshooting section** in `DESIGN.md`
2. **Review the test checklist** in `TESTING.md`
3. **Compare with working examples** in `EXAMPLE.md`
4. **Use debug output:**
   ```rust
   esp_println::println!("Debug: bytes read = {}", bytes);
   ```
5. **Use a logic analyzer** to see actual bus signals
6. **Ask for help** with detailed information:
   - Chip variant
   - Pin configuration
   - Master device type
   - Error messages
   - Logic analyzer captures if available

## Quick Reference Card

| Task | Code |
|------|------|
| Create slave | `I2c::new(i2c, config)?` |
| Set address | `.with_address(0x55.into())` |
| Connect SDA | `.with_sda(gpio)` |
| Connect SCL | `.with_scl(gpio)` |
| Read data | `i2c.read(&mut buf)?` |
| Write data | `i2c.write(&data)?` |
| To async mode | `.into_async()` |
| Listen events | `.listen(Event::RxFifoFull)` |
| Check events | `.interrupts()` |
| Clear events | `.clear_interrupts(events)` |

## Default Configuration

```rust
Config {
    address: 0x55,              // 7-bit address
    clock_stretch_enable: true, // If supported by chip
    sda_filter_enable: true,    // Noise filtering
    sda_filter_threshold: 7,    // Clock cycles
    scl_filter_enable: true,    // Noise filtering
    scl_filter_threshold: 7,    // Clock cycles
}
```

## Pin Recommendations

| Chip | Recommended SDA | Recommended SCL | Notes |
|------|-----------------|-----------------|-------|
| ESP32 | GPIO21 | GPIO22 | I2C0 default |
| ESP32-S2 | GPIO8 | GPIO9 | |
| ESP32-S3 | GPIO1 | GPIO2 | |
| ESP32-C3 | GPIO5 | GPIO6 | |
| ESP32-C6 | GPIO6 | GPIO7 | |
| ESP32-H2 | GPIO1 | GPIO0 | |

*Note: Most GPIO pins can be used due to GPIO matrix*

---

**You're ready to go!** Start with the minimal example above and expand from there. Happy coding! 🚀
