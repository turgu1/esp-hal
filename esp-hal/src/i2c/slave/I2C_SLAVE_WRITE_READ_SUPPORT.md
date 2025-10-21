# ESP32-C6 I2C Slave write_read() Support Analysis

## Question
**Can the I2C slave driver respond to a write_read() call from the master?**

## Answer: **PARTIALLY - Simple transactions work, but combined write_read() is NOT supported**

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

1. **Simple Write Transactions**
   ```rust
   // Master writes data to slave
   master.write(slave_addr, &[0x01, 0x02, 0x03])?;
   
   // Slave receives
   let mut buffer = [0u8; 128];
   let count = i2c.read(&mut buffer)?;  // Gets all bytes
   ```

2. **Simple Read Transactions**
   ```rust
   // Slave pre-loads response data
   i2c.write(&[0xAA, 0xBB, 0xCC])?;
   
   // Master reads
   let mut buffer = [0u8; 128];
   master.read(slave_addr, &mut buffer)?;
   ```

3. **Separate Sequential Transactions**
   ```rust
   // Transaction 1: Write with STOP
   master.write(slave_addr, &[register_addr])?;
   
   // Transaction 2: Read with STOP
   master.read(slave_addr, &mut buffer)?;
   ```

4. **Register-Based Mode (ESP32-C6)**
   ```rust
   #[cfg(esp32c6)]
   let config = Config::default().with_register_based_mode(true);
   let mut i2c = I2c::new(peripherals.I2C0, config)?;
   
   // Hardware automatically separates register address from data
   let mut buffer = [0u8; 128];
   let count = i2c.read(&mut buffer)?;           // Gets data bytes
   let reg_addr = i2c.read_register_address();   // Gets register address
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

### ❌ What DOES NOT WORK

**Combined write_read() transactions with repeated START**

---

## Why write_read() Still Fails

### Problem: `read()` waits for STOP

**Current code in `wait_for_rx_data()`:**
```rust
fn wait_for_rx_data(&self) -> Result<(), Error> {
    // Clears old interrupts, then waits...
    loop {
        let interrupts = self.regs().int_raw().read();
        
        // ❌ PROBLEM: Waits for trans_complete (STOP condition)
        if interrupts.trans_complete().bit_is_set() {
            break;
        }
        
        // Timeout after configured period
        if Instant::now() > start + timeout {
            return Err(Error::Timeout);
        }
    }
    Ok(())
}
```

**What happens in write_read():**
1. Master sends START + SLAVE_ADDR + W + data bytes
2. Master sends **REPEATED START** (NO STOP!)
3. Slave's `read()` is **blocked** waiting for `trans_complete`
4. Master times out or slave times out
5. **Transaction fails**

---

## Workarounds

### Option 1: Separate Transactions (RECOMMENDED)

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

**Downside**: Less efficient, not atomic, but works reliably.

### Option 2: Use Register-Based Mode (ESP32-C6 only)

For simple register read operations:
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

**Limitation**: Only works for the specific pattern where first byte is register address.

---

## What Would Be Needed for Full write_read() Support

### 1. Detect Repeated START

Monitor `trans_start` interrupt for repeated START conditions:
```rust
if ints.trans_start().bit_is_set() && !ints.trans_complete().bit_is_set() {
    // Repeated START detected
}
```

### 2. Detect Read/Write Direction

Check R/W bit after each START to know the direction.

### 3. State Machine

Implement proper state tracking:
- IDLE → WRITE_IN_PROGRESS → READ_IN_PROGRESS → COMPLETE

### 4. Pre-loaded Response Buffer

Allow application to prepare response data before master requests it.

---

## Current Driver Features (October 21, 2025)

The driver now includes:

1. **Configurable Timeout**: Prevents infinite waiting
   - Default: 1000ms (1 second)
   - Configurable via `.with_timeout_ms()`

2. **Register-Based Mode** (ESP32-C6):
   - Enabled via `.with_register_based_mode(true)`
   - First byte treated as register address
   - `read_register_address()` retrieves the register byte

3. **10-bit Address Support**:
   - Supports both 7-bit and 10-bit slave addresses
   - Automatic hardware configuration

4. **Clock Stretching Control**:
   - Disabled by default on ESP32-C6 to prevent bus hangs
   - Configurable on other variants

5. **Async Support**:
   - `read_async()` and `write_async()` methods
   - Interrupt-driven operation

---

## Hardware Capabilities

ESP32-C6 I2C slave hardware **DOES support** write_read():
- ✅ Has `trans_start` interrupt (repeated START detection)
- ✅ Has status register with transaction information
- ✅ Can handle combined transactions in hardware
- ❌ Driver software implementation for repeated START is **missing**

---

## Recommendation

### For Current Users:

**Use separate transactions** (Option 1) or **register-based mode** (Option 2 for ESP32-C6) until full write_read() support is implemented.

### For esp-hal Developers:

To add full write_read() support:

1. Modify `wait_for_rx_data()` to detect repeated START vs STOP
2. Add state tracking for transaction phases
3. Implement direction detection
4. Add API for pre-loading response data

**Estimated effort**: ~400-600 lines, 2-3 days

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
