# ESP32-C6 I2C Slave First Byte Loss Issue - RESOLVED

## Problem Description

When using the ESP32-C6 I2C slave driver, the first byte of data sent by the master was being lost. For example, when the master sends 14 bytes, only 13 bytes would be received (bytes 2-14), with the first byte missing.

## Root Cause

The issue was caused by the **`fifo_addr_cfg_en`** bit in the `FIFO_CONF` register being set to `1`.

### What `fifo_addr_cfg_en` Does

On the ESP32-C6, the I2C slave peripheral has two operating modes:

1. **Register-Based Mode** (`fifo_addr_cfg_en = 1`):
   - Designed for implementing I2C slaves that emulate register-based devices (like sensors with register addresses)
   - When enabled, the **first byte** received after the slave address is treated as a "register address"
   - This first byte is stored in a separate internal register (not the main FIFO)
   - Subsequent bytes go into the normal RX FIFO
   - **This is why the first byte was "lost"** - it was stored elsewhere!

2. **Raw Data Stream Mode** (`fifo_addr_cfg_en = 0`):
   - All received bytes are stored sequentially in the RX FIFO
   - No special treatment of the first byte
   - **This is the default mode for general-purpose I2C slave communication**

## The Solution

The driver now supports **both modes** as a configurable option:

### Default Behavior (Raw Data Stream Mode)

```rust
let config = Config::default(); // fifo_addr_cfg_en = 0 by default
let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// All bytes received go into the RX FIFO
let mut buffer = [0u8; 128];
let count = i2c.read(&mut buffer)?;  // Gets all bytes
```

### Register-Based Mode (Optional)

For applications that need to emulate register-based I2C devices:

```rust
#[cfg(esp32c6)]
let config = Config::default()
    .with_address(0x48.into())
    .with_register_based_mode(true);  // Enable register mode

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// First byte is register address, rest is data
let mut buffer = [0u8; 128];
let count = i2c.read(&mut buffer)?;           // Gets data bytes
let reg_addr = i2c.read_register_address();   // Gets register address
```

## Code Location

File: `esp-hal/src/i2c/slave/mod.rs`  
Functions:
- `Driver::init_slave()` - Sets `fifo_addr_cfg_en` based on config (~line 1280)
- `Driver::prepare_slave_tx()` - Maintains consistent setting for TX (~line 1650)
- `I2c::read_register_address()` - Retrieves register address in register mode (~line 820)

## Configuration

The mode is now controlled by the `register_based_mode` field in `Config`:

```rust
pub struct Config {
    // ... other fields ...
    
    #[cfg(esp32c6)]
    register_based_mode: bool,  // Default: false
}
```

## Verification

After this update:

**Raw Data Stream Mode (default)**:
- Master sends 14 bytes → Slave receives all 14 bytes in `read()`
- No data loss
- All bytes appear in the RX FIFO sequentially

**Register-Based Mode (when enabled)**:
- Master sends 1 register address + 13 data bytes
- `read()` returns 13 data bytes
- `read_register_address()` returns the register address byte
- Perfect for emulating sensors with register maps

## Additional Features

The driver now also includes:

1. **Timeout Support**: Configurable timeout prevents infinite waiting
   ```rust
   let config = Config::default().with_timeout_ms(2000); // 2 second timeout
   ```

2. **10-bit Address Support**: Both 7-bit and 10-bit slave addresses
   ```rust
   let config = Config::default()
       .with_address(0x1A5.into());  // 10-bit address
   ```

3. **Clock Stretching Control**: Disabled by default on ESP32-C6 to prevent bus hangs

## References

- ESP32-C6 Technical Reference Manual, I2C Chapter, FIFO Configuration section
- The `fifo_addr_cfg_en` bit is documented as "FIFO address configuration enable"

## Date

Initial Fix: October 16, 2025  
Feature Enhancement: October 21, 2025

