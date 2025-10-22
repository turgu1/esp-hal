# ESP32-C6 I2C Slave write_read() Support

## Question
**Can the I2C slave driver respond to a write_read() call from the master?**

## Answer: **YES - ESP32-C6 slave handles write_read() correctly in all modes** (as of October 22, 2025)

---

## What is write_read()?

A `write_read()` (also called "combined transaction" or "repeated start transaction") is a common I2C pattern where the master:

1. **Writes** data to the slave
2. Issues a **REPEATED START** (not STOP)
3. **Reads** data from the slave
4. Issues STOP

```
Timeline:
START → SLAVE_ADDR+W → [write bytes] → REPEATED_START → SLAVE_ADDR+R → [read bytes] → STOP
                                              ↑
                                    NO STOP between write and read!
```

### Common Use Case
This is extremely common for reading from I2C devices with registers:
```rust
// Master wants to read register 0x10:
master.write_read(slave_addr, &[0x10], &mut read_buffer)?;
//                                ↑          ↑
//                         write register  read data
//                         address (1 byte)
```

---

## ✅ Tested and Confirmed: ESP32-C6 Handles write_read() Correctly

**Important Discovery**: The ESP32-C6 I2C slave hardware correctly handles write_read() transactions
with repeated START **even without register-based mode enabled**. The hardware properly manages the
transaction phases and the driver works correctly out of the box.

### What This Means

When a master performs `write_read()`, the ESP32-C6 slave:
1. ✅ Receives the write phase data (all bytes go into RX FIFO)
2. ✅ Detects the repeated START condition automatically
3. ✅ Transitions to read phase and transmits pre-loaded TX FIFO data
4. ✅ Completes successfully without software intervention

**No special mode or configuration required!**

---

## Driver Capabilities

The ESP32-C6 I2C slave driver supports write_read() transactions through multiple approaches:

### ✅ Approach 1: Normal Mode (ESP32-C6 - WORKS PERFECTLY)

**Best for**: Any protocol where you want full control over all bytes

The slave receives ALL bytes in the write phase and can process them as needed.
Works perfectly with master's `write_read()` calls.

```rust
// ESP32-C6 Slave - Normal mode (default)
let config = Config::default()
    .with_address(0x55.into())
    .with_timeout_ms(2000);

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

loop {
    // Wait for master to write (could be from write_read())
    let mut write_buf = [0u8; 32];
    match i2c.read(&mut write_buf) {
        Ok(bytes_received) => {
            // Process received data (e.g., first byte is register address)
            let register = write_buf[0];
            let response = get_register_data(register);
            
            // Pre-load response for the read phase
            i2c.write(&response)?;
            // Master's read phase will get this data automatically
        }
        Err(e) => {
            // Handle errors
        }
    }
}

// Master can use write_read() with repeated START:
master.write_read(slave_addr, &[0x10], &mut buffer)?; // ✅ Works perfectly!
```

**How it works**: The ESP32-C6 hardware correctly handles the repeated START transition
from write to read automatically. The slave receives write data via `read()`, pre-loads
response via `write()`, and the hardware handles the read phase when master requests it.

### ✅ Approach 2: Register-Based Mode (ESP32-C6 - OPTIONAL)

**Best for**: When you want hardware to automatically separate register address from data

Hardware feature that treats the first byte specially - useful for sensor emulation.

```rust
// ESP32-C6 Slave - Register-based mode
let config = Config::default()
    .with_address(0x48.into())
    .with_register_based_mode(true);  // ← Optional enhancement

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

loop {
    let mut data = [0u8; 32];
    match i2c.read(&mut data) {
        Ok(bytes_received) => {
            // Hardware automatically separated register address
            let reg_addr = i2c.read_register_address();
            // data[] contains only the data bytes (if any)
            
            let response = get_register_data(reg_addr);
            i2c.write(&response)?;
        }
        Err(e) => {
            // Handle errors
        }
    }
}
```

**Advantage**: Slightly cleaner code when emulating register-based devices, as hardware
automatically separates the register address byte from data bytes.

### ✅ Approach 3: Separate Transactions (ALL VARIANTS)

**Best for**: Maximum compatibility across all ESP32 variants (ESP32, S2, S3, C3, C6, H2)

```rust
// Works on any ESP32 variant
let config = Config::default().with_address(0x55.into());
let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

loop {
    let mut cmd = [0u8; 1];
    // Transaction 1: Master writes
    if let Ok(_) = i2c.read(&mut cmd) {
        let register = cmd[0];
        let response = get_register_data(register);
        
        // Transaction 2: Prepare for master read (separate transaction)
        i2c.write(&response)?;
    }
}

// Master uses separate transactions (NOT write_read):
master.write(slave_addr, &[0x10])?;     // Transaction 1 with STOP
master.read(slave_addr, &mut buffer)?;  // Transaction 2
       .with_address(0x1A5.into());  // 10-bit address (0x000 - 0x3FF)
   ```

---

## Complete Example: Register-Based Device

This example shows how to implement a sensor-like device that responds to register read requests:

```rust
#[cfg(esp32c6)]
{
    use esp_hal::i2c::slave::{Config, I2c};
    
    // Enable register-based mode
    let config = Config::default()
        .with_address(0x48.into())  // Sensor address
        .with_register_based_mode(true);  // ← Key setting
    
    let mut i2c = I2c::new(peripherals.I2C0, config)?
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);
    
    // Simulate sensor registers
    let mut registers = [0u8; 256];
    registers[0x00] = 0x48; // Device ID
    registers[0x01] = 0x25; // Temperature high byte
    registers[0x02] = 0x60; // Temperature low byte
    
    loop {
        // Wait for transaction (works with both write_read() and separate transactions)
        let mut data = [0u8; 32];
        match i2c.read(&mut data) {
            Ok(bytes_read) => {
                // Get the register address (automatically separated by hardware)
                let reg_addr = i2c.read_register_address();
                
                if bytes_read > 0 {
                    // Master wrote data to registers
                    for (i, &byte) in data[..bytes_read].iter().enumerate() {
                        registers[(reg_addr as usize + i) & 0xFF] = byte;
                    }
                }
                
                // Prepare response for read (if master does read phase)
                let num_bytes = 4.min(256 - reg_addr as usize);
                let response = &registers[reg_addr as usize..][..num_bytes];
                i2c.write(response)?;
            }
            Err(e) => {
                // Handle timeout or other errors
            }
        }
    }
}
```

**Master can use either approach:**
```rust
// Approach 1: write_read() with repeated START (atomic)
master.write_read(0x48, &[0x01], &mut temp_buffer)?;

// Approach 2: Separate transactions (also works)
master.write(0x48, &[0x01])?;
master.read(0x48, &mut temp_buffer)?;
```

Both approaches work identically from the slave's perspective when register-based mode is enabled.

---

## How It Works - Hardware Behavior

### ESP32-C6 Hardware Handling of write_read()

**Testing confirms**: The ESP32-C6 I2C slave peripheral correctly handles repeated START
conditions automatically in hardware:

1. **Write Phase**: Master writes data → Slave receives via `read()` → Data goes to RX FIFO
2. **Repeated START**: Master sends repeated START → Hardware detects automatically
3. **Read Phase**: Master reads → Slave transmits pre-loaded TX FIFO data
4. **No Software Detection Required**: The hardware manages the transition seamlessly

This works in **both normal mode and register-based mode**.

### Optional: Register-Based Mode Enhancement

When `register_based_mode` is enabled (optional), the ESP32-C6 hardware adds:

1. **Automatic Separation**: The hardware's `fifo_addr_cfg_en` bit is set
2. **First Byte Special**: After slave address match, the first byte is stored separately
3. **Register Address Storage**: This byte is accessible via `read_register_address()`
4. **Data in FIFO**: Remaining bytes (if any) go to the normal RX FIFO

**Key Insight**: Register-based mode is purely for convenience - it's NOT required for
write_read() to work. Normal mode handles repeated START perfectly fine.

---

## Driver Features Summary

Current driver capabilities include:

1. **✅ write_read() Support** (ESP32-C6 in ANY mode):
   - ✅ **Tested and confirmed working** in normal mode
   - Hardware automatically handles repeated START transitions
   - No special configuration required
   - Optional: Register-based mode for automatic register address separation

2. **✅ Separate Transaction Support** (All variants):
   - Works on all ESP32 variants (ESP32, S2, S3, C3, C6, H2)
   - Reliable fallback for maximum compatibility

3. **✅ Configurable Timeout**:
   - Default: 1000ms (1 second)
   - Configurable via `.with_timeout_ms()`
   - Prevents infinite blocking

4. **✅ 10-bit Address Support**:
   - Supports both 7-bit and 10-bit slave addresses
   - Automatic hardware configuration
   - Conversions from `u8`, `u16`, and `i32`

5. **✅ Clock Stretching Control**:
   - Configurable on supported variants
   - Important: May cause issues with ESP32 (original) as master
   - See ESP32_MASTER_COMPATIBILITY.md for details

6. **✅ Async Support**:
   - `read_async()` and `write_async()` methods
   - Embassy executor compatible
   - Future-based API

7. **✅ FIFO Management**:
   - 32-byte hardware FIFO
   - Watermark at 30 bytes for interrupt triggering
   - Timeout protection against stale interrupts

---

## Recommendations

### For ESP32-C6 Users - write_read() Support

**Good News**: write_read() works perfectly in normal mode! You don't need register-based mode
unless you specifically want automatic register address separation.

**Option 1: Normal Mode (Simpler, Recommended)**
```rust
// Default configuration - works great with write_read()
let config = Config::default().with_address(0x55.into());

// Your app receives all bytes and decides how to process them
let mut data = [0u8; 32];
i2c.read(&mut data)?;  // Get all write phase bytes
let register = data[0];  // You decide which byte is the register
let response = get_register_data(register);
i2c.write(&response)?;  // Pre-load for read phase
```

**Option 2: Register-Based Mode (Optional Convenience)**
```rust
// Enable if you want hardware to separate register address automatically
let config = Config::default()
    .with_address(0x48.into())
    .with_register_based_mode(true);  // ← Optional enhancement

// Hardware separates first byte for you
let mut data = [0u8; 32];
i2c.read(&mut data)?;  // Data bytes only
let reg_addr = i2c.read_register_address();  // Register byte (separated by HW)
```

**Both work perfectly with master's `write_read()`!**

### For All Variants (Maximum Compatibility)

**Use separate transactions** when you need to support all ESP32 variants:

```rust
// Works on ESP32, S2, S3, C3, C6, H2
let mut cmd = [0u8; 32];
i2c.read(&mut cmd)?;
let response = process_command(&cmd);
i2c.write(&response)?;

// Master uses separate transactions (NOT write_read)
master.write(slave_addr, &[cmd])?;
master.read(slave_addr, &mut buffer)?;
```

### Choosing the Right Approach

| Use Case | ESP32-C6 | Other Variants | Recommendation |
|----------|----------|----------------|----------------|
| Master uses write_read() | ✅ Normal mode | Separate transactions | Normal mode on C6 |
| Register-based device (sensor) | ✅ Either mode | Separate transactions | Reg mode for convenience |
| Custom protocol | ✅ Normal mode | Separate transactions | Normal mode |
| Maximum compatibility | Separate trans | Separate transactions | Separate transactions |

---

## Conclusion

**write_read() Support Status**: ✅ **FULLY SUPPORTED AND TESTED on ESP32-C6**

### Key Findings from Testing

**✅ CONFIRMED**: The ESP32-C6 I2C slave hardware correctly handles master's `write_read()` 
calls with repeated START **in normal mode** - no special configuration required!

The driver provides complete write_read() support:

- ✅ **write_read() with repeated START** - ESP32-C6 (works in normal mode!)
  - Tested and confirmed working
  - Hardware automatically handles the repeated START transition
  - No special software detection needed
  
- ✅ **Register-based mode** - ESP32-C6 only (optional enhancement)
  - Provides automatic register address separation
  - Convenience feature, not required for write_read()
  
- ✅ **Simple write transactions** - All variants
- ✅ **Simple read transactions** - All variants  
- ✅ **Separate transactions** - All variants (maximum compatibility)
- ✅ **Timeout support** - All variants
- ✅ **10-bit addresses** - All variants
- ✅ **Async operation** - All variants

### Bottom Line

The ESP32-C6 I2C slave driver works perfectly with master's `write_read()` operations out of
the box. The hardware handles repeated START conditions correctly, and register-based mode is
simply an optional convenience feature for register-addressed protocols.

---

## Additional Resources

- **ESP32_MASTER_COMPATIBILITY.md** - Important information about clock stretching compatibility with ESP32 master
- **README.md** - Complete driver documentation with configuration examples
- **Module documentation** (`src/i2c/slave/mod.rs`) - API reference and detailed examples

---

## Document History

- **October 16, 2025**: Initial analysis - write_read() support unclear
- **October 21, 2025**: Register-based mode implemented (ESP32-C6)
- **October 22, 2025**: **Testing confirmed** - write_read() works perfectly in normal mode!
  - ESP32-C6 hardware handles repeated START correctly without special configuration
  - Register-based mode is optional convenience feature, not required for write_read()
  - Document updated to reflect testing results
