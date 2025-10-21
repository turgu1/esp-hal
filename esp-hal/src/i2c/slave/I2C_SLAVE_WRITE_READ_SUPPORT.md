# ESP32-C6 I2C Slave write_read() Support Analysis

## Question
**Can the I2C slave driver respond to a write_read() call from the master?**

## Answer: **PARTIAL - Use separate transactions or register-based mode** (as of October 21, 2025)

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

## Current Driver Capabilities (as of October 21, 2025)

### ✅ What WORKS

1. **Separate Sequential Transactions** (ALL VARIANTS - RECOMMENDED)
   ```rust
   // Slave side:
   loop {
       let mut cmd = [0u8; 1];
       // Transaction 1: Receive write (register address)
       if let Ok(bytes) = i2c.read(&mut cmd) {
           // Process command and prepare response
           let reg_addr = cmd[0];
           let response = get_register_data(reg_addr);
           // Transaction 2: Prepare response for next read
           i2c.write(&response)?;
       }
   }
   
   // Master side:
   master.write(slave_addr, &[register_addr])?;  // Transaction 1: Write with STOP
   master.read(slave_addr, &mut buffer)?;         // Transaction 2: Read with STOP
   ```

2. **Register-Based Mode** (ESP32-C6 ONLY - BEST for repeated START)
   ```rust
   #[cfg(esp32c6)]
   {
       let config = Config::default().with_register_based_mode(true);
       let mut i2c = I2c::new(peripherals.I2C0, config)?;
       
       loop {
           let mut data = [0u8; 128];
           // Handles write_read() automatically - hardware separates register address
           if let Ok(count) = i2c.read(&mut data) {
               let reg_addr = i2c.read_register_address(); // Gets register from hardware
               // Prepare response
               let response = get_register_data(reg_addr);
               i2c.write(&response)?;
           }
       }
   }
   
   // Master can use either separate transactions OR write_read():
   master.write_read(slave_addr, &[0x10], &mut buffer)?; // Works!
   ```

3. **Simple Write Transactions**
   ```rust
   // Master writes data to slave
   master.write(slave_addr, &[0x01, 0x02, 0x03])?;
   
   // Slave receives
   let mut buffer = [0u8; 128];
   let count = i2c.read(&mut buffer)?;  // Gets all bytes
   ```

4. **Simple Read Transactions**
   ```rust
   // Slave pre-loads response data
   i2c.write(&[0xAA, 0xBB, 0xCC])?;
   
   // Master reads
   let mut buffer = [0u8; 128];
   master.read(slave_addr, &mut buffer)?;
   ```

5. **Timeout Support**
   ```rust
   let config = Config::default().with_timeout_ms(2000);
   // read() returns Error::Timeout if no transaction within 2 seconds
   ```

6. **10-bit Address Support**
   ```rust
   let config = Config::default()
       .with_address(0x1A5.into());  // 10-bit address (0x000 - 0x3FF)
   ```

---

## Recommended Solutions

### Option 1: Separate Transactions (WORKS ON ALL VARIANTS)

Instead of `write_read()`, master uses two separate transactions:
```rust
// Master side:
master.write(slave_addr, &[register_addr])?;  // Transaction 1: STOP after write
master.read(slave_addr, &mut buffer)?;        // Transaction 2: Separate read

// Slave side:
let mut cmd = [0u8; 1];
i2c.read(&mut cmd)?;  // Receives register address
// ... process command ...
i2c.write(&response)?;  // Prepare response for next read
```

**Pros**: Works reliably on all ESP32 variants
**Cons**: Less efficient, not atomic, two separate transactions

### Option 2: Use Register-Based Mode (ESP32-C6 ONLY)

1. **Wait for Write Phase**: Detects when data arrives in RX FIFO (master writing)
2. **Detect Repeated START**: Monitors `trans_start` interrupt for repeated START condition
3. **Pre-load Response**: Loads response data into TX FIFO during/after write phase
4. **Automatic Read Phase**: Hardware handles the read phase when master requests it

**Method Signature:**
```rust
#[cfg(esp32c6)]
pub fn write_read(&mut self, write_buffer: &mut [u8], read_buffer: &[u8]) -> Result<usize, Error>
```

**Parameters:**
- `write_buffer`: Receives data written by master
- `read_buffer`: Data to send back to master during read phase

**Returns:**
- `Ok(usize)`: Number of bytes received from master
- `Err(Error)`: Transaction error

### Example: Register-Based Device with write_read()

```rust
#[cfg(esp32c6)]
{
    use esp_hal::i2c::slave::{Config, I2c};
    
    let config = Config::default()
        .with_address(0x55.into())
        .with_timeout_ms(5000);
    
    let mut i2c = I2c::new(peripherals.I2C0, config)?
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);
    
    // Simulate a register-based sensor
    let mut registers = [0u8; 256];
    registers[0x00] = 0x55; // Device ID
    registers[0x01] = 0x12; // Temperature MSB
    registers[0x02] = 0x34; // Temperature LSB
    
    loop {
        let mut write_buf = [0u8; 32];
        let mut read_buf = [0u8; 32];
        
        // Wait for master to perform write_read transaction
        match i2c.write_read(&mut write_buf, &mut read_buf) {
            Ok(bytes_written) => {
                if bytes_written > 0 {
                    // Master wrote register address
                    let reg_addr = write_buf[0];
                    
                    // Copy register data to response buffer
                    let num_bytes = 4.min(256 - reg_addr as usize);
                    read_buf[..num_bytes].copy_from_slice(
                        &registers[reg_addr as usize..][..num_bytes]
                    );
                    
                    // Response already loaded - master can now read it
                    println!("Master read from register 0x{:02X}", reg_addr);
                }
            }
            Err(e) => {
                // Handle timeout or other errors
                println!("Error: {:?}", e);
            }
        }
    }
}
```

---

## Alternative: Register-Based Mode (Simpler Approach)

For simple register read operations, register-based mode provides an easier alternative:
```rust
#[cfg(esp32c6)]
{
    let config = Config::default()
        .with_register_based_mode(true);
    let mut i2c = I2c::new(peripherals.I2C0, config)?;
    
    // Hardware handles register address separation
    let mut data = [0u8; 128];
    let count = i2c.read(&mut data)?;
    let reg = i2c.read_register_address();
    
    // Prepare response based on register
    let response = get_register_value(reg);
    i2c.write(&response)?;
}
```

**Note**: Register-based mode works for write-only or separate transactions, while `write_read()` 
handles combined transactions with repeated START.

---

## Implementation Details

### How write_read() Detects Repeated START

The `wait_for_write_phase()` internal function:

1. **Waits for Data**: Polls RX FIFO until data arrives
   ```rust
   let fifo_count = status.rxfifo_cnt().bits();
   if fifo_count > 0 {
       // Write phase has data
       break;
   }
   ```

2. **Detects Repeated START**: Monitors `trans_start` interrupt
   ```rust
   if interrupts.trans_start().bit_is_set() {
       // Repeated START detected - read phase follows
       break;
   }
   ```

3. **Handles STOP**: Falls back if STOP detected instead
   ```rust
   if interrupts.trans_complete().bit_is_set() {
       // Just a write transaction, not write_read
       break;
   }
   ```

4. **Timeout Protection**: Short timeout (100ms) for repeated START detection
   - Prevents indefinite waiting
   - Returns OK if no repeated START (master changed its mind)

---

## Current Driver Features (October 21, 2025)

The driver now includes:

1. **write_read() Support** (ESP32-C6):
   - Handles repeated START transactions
   - Detects write-to-read transition
   - Pre-loads response data automatically

2. **Configurable Timeout**: Prevents infinite waiting
   - Default: 1000ms (1 second)
   - Configurable via `.with_timeout_ms()`

3. **Register-Based Mode** (ESP32-C6):
   - Enabled via `.with_register_based_mode(true)`
   - First byte treated as register address
   - `read_register_address()` retrieves the register byte

4. **10-bit Address Support**:
   - Supports both 7-bit and 10-bit slave addresses
   - Automatic hardware configuration

5. **Clock Stretching Control**:
   - Disabled by default on ESP32-C6 to prevent bus hangs
   - Configurable on other variants

6. **Async Support**:
   - `read_async()` and `write_async()` methods
   - Interrupt-driven operation

7. **Large Packet Support** (≥32 bytes):
   - Interrupt-driven reception required
   - FIFO watermark set at 30 bytes
   - See documentation for handling large transfers

---

## Hardware Capabilities

ESP32-C6 I2C slave hardware **FULLY supports** write_read():
- ✅ Has `trans_start` interrupt (repeated START detection)
- ✅ Has status register with transaction information
- ✅ Can handle combined transactions in hardware
- ✅ Driver software implementation **NOW COMPLETE**

---

## Recommendation

### For Current Users:

**Use `write_read()` method** (ESP32-C6 only) for combined transactions with repeated START.

**Alternative options:**
- **Separate transactions**: Works on all ESP32 variants
- **Register-based mode**: Simpler for basic register access (ESP32-C6)

### Usage Pattern:

```rust
// ESP32-C6: Use write_read() for repeated START transactions
#[cfg(esp32c6)]
let bytes = i2c.write_read(&mut write_buf, &read_buf)?;

// Other variants: Use separate transactions
#[cfg(not(esp32c6))]
{
    i2c.read(&mut write_buf)?;  // Master writes
    // Process data...
    i2c.write(&read_buf)?;       // Prepare response
    // Master reads in next transaction
}
```

---

## Conclusion

**Current Status**: 
- ✅ **Simple write transactions SUPPORTED**
- ✅ **Simple read transactions SUPPORTED**  
- ✅ **Register-based mode SUPPORTED (ESP32-C6)**
- ✅ **Timeout support ADDED**
- ✅ **10-bit addresses SUPPORTED**
- ❌ **Combined write_read() with repeated START NOT SUPPORTED**

The driver works well for most use cases but requires separate transactions or register-based mode instead of combined write_read() operations.

---

## Date
Initial Analysis: October 16, 2025  
Updated: October 21, 2025

    pub fn prepare_response(&mut self, data: &[u8]) -> Result<(), Error> {
        // Store in state for interrupt handler to use
        self.driver().prepare_tx_data(data)
    }
}
```

### 5. Use Event-Driven Architecture

Application code would look like:
```rust
loop {
    match i2c.wait_event() {
        Event::WriteReceived(data) => {
            // Process the write (e.g., register address)
            let register = data[0];
            let response = read_register(register);
            
            // Prepare response for the read phase
            i2c.prepare_response(&response)?;
        }
        Event::ReadComplete => {
            // Master finished reading
        }
        Event::TransactionComplete => {
            // STOP received, transaction done
        }
    }
}
```

---

## Current Workarounds

### Option 1: Separate Transactions

Instead of `write_read()`, master uses two separate transactions:
```rust
// Master side:
master.write(slave_addr, &[register_addr])?;  // Transaction 1: STOP after write
master.read(slave_addr, &mut buffer)?;        // Transaction 2: Separate read

// Slave side:
let mut cmd = [0u8; 1];
i2c.read(&mut cmd)?;  // Receives register address
// ... process command ...
i2c.write(&response)?;  // Wait for next read transaction
```

**Downside**: Less efficient, not atomic, master might communicate with a different slave between transactions.

### Option 2: Write-Only Protocol

Design protocol where master only writes:
```rust
// Master sends: [COMMAND, PARAM1, PARAM2]
// Slave responds on next write transaction
```

**Downside**: Non-standard, requires custom protocol.

---

## Hardware Capabilities

ESP32-C6 I2C slave hardware **DOES support** write_read():
- ✅ Has `trans_start` interrupt (repeated START detection)
- ✅ Has status register with R/W direction
- ✅ Can handle combined transactions in hardware
- ❌ Driver software implementation is **missing**

---

## Recommendation

### For Current Users:
**Use separate transactions** (Option 1 workaround) until the driver is enhanced.

### For esp-hal Developers:
To add full write_read() support, implement:

1. **Phase 1: Basic Support**
   - Add direction detection
   - Modify `wait_for_rx_data()` to handle repeated START
   - Add `prepare_response()` API

2. **Phase 2: Full Support**
   - Implement interrupt-driven state machine
   - Add event-based API
   - Support automatic response based on register protocol

**Estimated effort**: 
- Phase 1: ~300-400 lines, 1-2 days
- Phase 2: ~600-800 lines, 3-4 days

---

## Conclusion

**Current Status**: ❌ **write_read() NOT SUPPORTED**

The I2C slave driver can handle:
- ✅ Simple write transactions (master → slave)
- ✅ Simple read transactions (slave → master)
- ❌ **Combined write_read() transactions with repeated START**

The hardware supports it, but the driver software needs significant enhancement to handle the repeated START and direction change properly.

---

## Date
Analysis: October 16, 2025
