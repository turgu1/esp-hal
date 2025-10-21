# ESP32 Master and ESP32-C6 Slave Clock Stretching Compatibility Issue

## Problem Statement

When using an **ESP32-C6 as I2C slave** with **ESP32 (original) as I2C master**, enabling clock stretching on the slave causes bus hangs with both SCL and SDA held low indefinitely.

## Root Cause

**ESP32 Master Limitation**: The ESP32 (original) I2C master peripheral has poor support for I2C clock stretching:
- It expects quick slave responses
- Has limited tolerance for SCL being held low by slave
- May timeout or enter an error state when encountering clock stretching
- Can leave the bus in a locked state requiring power cycle or bus reset

**ESP32-C6 Slave Behavior**: When clock stretching is enabled:
- The slave holds SCL low when FIFO is full or needs processing time
- This is standard I2C behavior per the specification
- However, the ESP32 master doesn't handle this properly

## Symptoms

1. **Bus Hang**: Both SCL and SDA lines stuck low
2. **No Recovery**: Bus remains stuck until power cycle or manual reset
3. **Master Timeout**: ESP32 master reports timeout errors
4. **Transaction Failure**: No successful I2C communication

## Solution

### For ESP32-C6 Slave Configuration

**Disable clock stretching** when ESP32 is used as master:

```rust
use esp_hal::i2c::slave::Config;

let config = Config::default()
    .with_address(0x55.into())
    .with_clock_stretch_enable(false);  // ← Critical for ESP32 master compatibility

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);
```

### Handling Large Packets Without Clock Stretching

For packets ≥ 30 bytes, you have two options:

**Option 1: Keep packets small (< 30 bytes)**
- Most reliable approach
- No special handling needed
- Blocking `read()` works fine

**Option 2: Use interrupt-driven reception**
- Enables handling of larger packets
- FIFO watermark interrupt triggers at 30 bytes
- Read data before FIFO completely fills at 32 bytes
- Requires fast interrupt response

## Master-Slave Compatibility Matrix

| Master Chip | Slave Chip | Clock Stretching | Status |
|-------------|------------|------------------|--------|
| ESP32       | ESP32-C6   | Enabled          | ❌ Bus hangs |
| ESP32       | ESP32-C6   | Disabled         | ✅ Works |
| ESP32-S2/S3 | ESP32-C6   | Enabled          | ✅ Likely OK |
| ESP32-C3    | ESP32-C6   | Enabled          | ✅ Likely OK |
| External IC | ESP32-C6   | Enabled          | ✅ Should work |

## Testing Your Setup

To determine if clock stretching is causing issues:

1. **Disable clock stretching**:
   ```rust
   .with_clock_stretch_enable(false)
   ```

2. **Test communication** with small packets (< 30 bytes)

3. **If working**: The issue was clock stretching compatibility

4. **If still failing**: Look for other issues (wiring, pull-ups, timing, etc.)

## Hardware Considerations

### When Clock Stretching is Disabled

**Advantages**:
- Works with ESP32 master
- No bus hang issues
- Predictable timing

**Limitations**:
- Packets must be ≤ 29 bytes in blocking mode
- Larger packets require interrupts or async
- FIFO can overflow if master sends ≥ 32 bytes quickly

### Pull-up Resistors

Ensure proper I2C pull-up resistors (typically 2.2kΩ - 10kΩ):
- Required on both SCL and SDA
- Too weak: Bus may not pull high fast enough
- Too strong: Excessive current when bus is held low
- 4.7kΩ is a good default for most applications

## Alternative: Use Different Master

If you need clock stretching functionality, consider:
- **ESP32-S2, ESP32-S3, ESP32-C3**: Better clock stretching support
- **ESP32-C6 as master**: Should handle its own slave's clock stretching
- **External I2C masters**: Most proper I2C controllers support clock stretching

## Code Example: Complete Working Setup

```rust
use esp_hal::i2c::slave::{Config, I2c};

// ESP32-C6 slave configuration for ESP32 master compatibility
let config = Config::default()
    .with_address(0x55.into())
    .with_clock_stretch_enable(false)  // ← Disable for ESP32 master
    .with_timeout_ms(2000);

let mut i2c = I2c::new(peripherals.I2C0, config)
    .unwrap()
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// For small packets (< 30 bytes), this works perfectly
let mut buffer = [0u8; 28];
match i2c.read(&mut buffer) {
    Ok(bytes_read) => {
        // Process received data
    }
    Err(e) => {
        // Handle error
    }
}
```

## Conclusion

The clock stretching issue you experienced is **not a bug in the ESP32-C6 slave driver**, but rather a **compatibility issue between ESP32 master and clock stretching in general**.

**Solution**: Disable clock stretching when using ESP32 as master, and design your protocol to use packets < 30 bytes or implement interrupt-driven reception for larger transfers.

This limitation is specific to the ESP32 (original) master peripheral and doesn't affect other master implementations.
