# I2C Slave Driver for ESP32 Architecture

This directory contains the I2C slave driver implementation for ESP32 chips.

## Overview

The I2C slave driver allows ESP32 devices to act as I2C slave peripherals that respond to requests from an I2C master. This implementation follows the same design patterns as the I2C master driver in `../master/` for consistency and maintainability.

## Supported Chips

The I2C slave mode is available on the following ESP32 chips:
- ESP32 (limited support - marked as "not_supported" in metadata but hardware capable)
- ESP32-S2
- ESP32-S3
- ESP32-C3
- ESP32-C6 (full support with enhanced features)
- ESP32-H2

Note: ESP32-P4 and ESP32-C2 do not have I2C slave support listed in the metadata.

## Features

### Blocking Mode
- Basic read/write operations
- Configurable slave address (7-bit and 10-bit)
- Configurable timeout for blocking reads
- Clock stretching support (configurable, disabled by default on ESP32-C6)
- SDA/SCL signal filtering
- Interrupt support
- Register-based mode (ESP32-C6 only)

### Async Mode
- Asynchronous read/write operations
- Future-based API
- Compatible with async executors like Embassy

### ESP32-C6 Specific Features
- **Register-Based Mode**: Emulate register-based I2C devices (sensors, etc.)
- **10-bit Address Support**: Full hardware support for 10-bit addressing
- **Clock Stretching Disabled**: Prevents bus hangs during TX operations
- **Enhanced FIFO Configuration**: Automatic handling of FIFO modes

## Configuration Options

The driver can be configured with:
- **Slave Address**: 7-bit (0x00..=0x7F) or 10-bit (0x000..=0x3FF) I2C address (default: 7-bit 0x55)
- **Timeout**: Blocking read timeout in milliseconds (default: 1000ms)
- **Clock Stretching**: Enable/disable clock stretching (default: true, except ESP32-C6 where it's disabled)
- **SDA Filter**: Enable/disable and configure threshold (default: enabled, threshold 7)
- **SCL Filter**: Enable/disable and configure threshold (default: enabled, threshold 7)
- **Register-Based Mode**: Enable register addressing mode (ESP32-C6 only, default: false)

## Usage Examples

### Basic Blocking Mode

```rust
use esp_hal::i2c::slave::{Config, I2c};

let config = Config::default()
    .with_address(0x55.into())
    .with_timeout_ms(2000);  // 2 second timeout

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// Read data from master
let mut buffer = [0u8; 128];
match i2c.read(&mut buffer) {
    Ok(bytes_read) => {
        // Process received data
    }
    Err(Error::Timeout) => {
        // No data received within timeout
    }
    Err(e) => {
        // Handle other errors
    }
}

// Write data to master (preload before master reads)
i2c.write(&[0xAA, 0xBB, 0xCC])?;
```

### 10-bit Address Support

```rust
use esp_hal::i2c::slave::{Config, I2c, I2cAddress};

// Using automatic conversion (u16 -> 10-bit if > 0x7F)
let config = Config::default()
    .with_address(0x1A5.into());  // 10-bit address

// Or explicit 10-bit address
let config = Config::default()
    .with_address(I2cAddress::TenBit(0x2F3));

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);
```

### Register-Based Mode (ESP32-C6 Only)

Emulate a sensor or other register-based I2C device:

```rust
#[cfg(esp32c6)]
{
    use esp_hal::i2c::slave::{Config, I2c};
    
    let config = Config::default()
        .with_address(0x48.into())  // Sensor address
        .with_register_based_mode(true);
    
    let mut i2c = I2c::new(peripherals.I2C0, config)?
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);
    
    // Simulate sensor registers
    let mut registers = [0u8; 256];
    registers[0x00] = 0x48; // Device ID
    registers[0x01] = 0x25; // Temperature high byte
    registers[0x02] = 0x60; // Temperature low byte
    
    loop {
        let mut rx_buffer = [0u8; 32];
        if let Ok(bytes_read) = i2c.read(&mut rx_buffer) {
            // Get the register address that the master specified
            let register_addr = i2c.read_register_address();
            
            if bytes_read > 0 {
                // Master wrote data to register
                for (i, &byte) in rx_buffer[..bytes_read].iter().enumerate() {
                    registers[(register_addr as usize + i) & 0xFF] = byte;
                }
            }
            
            // Prepare response for potential master read
            let response = &registers[register_addr as usize..][..4];
            i2c.write(response)?;
        }
    }
}
```

### Async Mode

```rust
use esp_hal::i2c::slave::{Config, I2c};

let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2)
    .into_async();

// Asynchronous read
let mut buffer = [0u8; 128];
let bytes_read = i2c.read_async(&mut buffer).await?;

// Asynchronous write
i2c.write_async(&[0xAA, 0xBB]).await?;
```

### Complete Async Example with Embassy

Here's a complete example showing how to use the I2C slave driver with async/await and Embassy executor:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    i2c::slave::{Config, I2c},
    prelude::*,
    timer::systimer::SystemTimer,
};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize system timer for Embassy
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    // Configure I2C slave with address 0x55
    let config = Config::default()
        .with_address(0x55.into())
        .with_timeout_ms(5000);

    let mut i2c = I2c::new(peripherals.I2C0, config)
        .unwrap()
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2)
        .into_async();

    // Buffer for received data
    let mut rx_buffer = [0u8; 64];
    
    // Simulate device state
    let mut device_register = [0u8; 16];
    device_register[0] = 0x55; // Device ID
    device_register[1] = 0x01; // Version
    
    loop {
        // Wait for master to send data (async, non-blocking)
        match i2c.read_async(&mut rx_buffer).await {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    // Process received command
                    let command = rx_buffer[0];
                    
                    match command {
                        0x00 => {
                            // Read device ID - prepare response
                            let response = &device_register[0..2];
                            let _ = i2c.write_async(response).await;
                        }
                        0x01 => {
                            // Write data to device
                            if bytes_read > 1 {
                                // Store data (skip command byte)
                                for i in 1..bytes_read {
                                    device_register[i] = rx_buffer[i];
                                }
                            }
                        }
                        0x02 => {
                            // Read all registers
                            let _ = i2c.write_async(&device_register).await;
                        }
                        _ => {
                            // Unknown command - send error code
                            let _ = i2c.write_async(&[0xFF]).await;
                        }
                    }
                }
            }
            Err(e) => {
                // Handle timeout or other errors
                // In async mode, this typically means timeout
                Timer::after(Duration::from_millis(10)).await;
            }
        }
        
        // Small delay to prevent tight loop
        Timer::after(Duration::from_millis(1)).await;
    }
}
```

**Key Features of Async Mode:**
- Non-blocking operations - other tasks can run while waiting for I2C transactions
- Uses Embassy's async runtime for efficient task scheduling
- Interrupts are handled automatically by the async implementation
- `await` points allow the executor to switch to other tasks
- Ideal for applications that need to handle multiple peripherals concurrently

**Required Dependencies (Cargo.toml):**
```toml
[dependencies]
esp-hal = { version = "1.0", features = ["esp32c6"] }
esp-backtrace = { version = "0.14", features = ["esp32c6", "exception-handler", "panic-handler", "println"] }
esp-hal-embassy = { version = "1.0", features = ["esp32c6"] }
embassy-executor = { version = "0.6", features = ["arch-riscv32", "executor-thread"] }
embassy-time = "0.3"
```

### Interrupt Handling

```rust
use esp_hal::i2c::slave::{Config, I2c, Event};

let mut i2c = I2c::new(peripherals.I2C0, Config::default())?;

// Listen for specific events
i2c.listen(Event::RxFifoFull | Event::TxFifoEmpty);

// Check for interrupts
let events = i2c.interrupts();

// Clear interrupts
i2c.clear_interrupts(events);
```

### Handling Large Packets (> 32 bytes)

Due to the 32-byte FIFO limitation, packets larger than 32 bytes require interrupt-driven reception:

```rust
use esp_hal::i2c::slave::{Config, I2c, Event};

let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// Enable FIFO and transaction complete interrupts
i2c.listen(Event::RxFifoFull | Event::TransComplete);

// Buffer for large packet
let mut large_buffer = [0u8; 128];
let mut offset = 0;

loop {
    let events = i2c.interrupts();
    
    if events.contains(Event::RxFifoFull) {
        // FIFO has data (up to 32 bytes) - read it
        match i2c.read(&mut large_buffer[offset..]) {
            Ok(chunk_size) => {
                offset += chunk_size;
            }
            Err(e) => {
                // Handle error
                offset = 0;
            }
        }
        i2c.clear_interrupts(Event::RxFifoFull);
    }
    
    if events.contains(Event::TransComplete) {
        // Transaction finished - process complete packet
        if offset > 0 {
            // Process data in large_buffer[..offset]
            println!("Received {} bytes", offset);
            
            // Reset for next packet
            offset = 0;
        }
        i2c.clear_interrupts(Event::TransComplete);
    }
}
```

**Important Notes:**
- RxFifoFull triggers when FIFO reaches threshold (default: 1 byte, configurable)
- Read data promptly to avoid FIFO overflow
- TransComplete indicates the master has finished the transaction
- Without interrupt handling, only first 32 bytes will be received

## Architecture

The driver follows the same architectural patterns as the I2C master driver:

### Key Components

1. **`I2c<'d, Dm>`**: Main driver struct with lifetime and driver mode parameters
2. **`Config`**: Configuration structure with builder pattern support
3. **`Driver<'a>`**: Internal driver implementation
4. **`Info`**: Static peripheral information (register block, signals, etc.)
5. **`State`**: Runtime state (wakers for async operations)
6. **`Instance`**: Trait for peripheral instances

### Driver Modes

- **`Blocking`**: Synchronous operations with polling
- **`Async`**: Asynchronous operations using Futures

### Event System

The driver supports the following events:
- `RxFifoFull`: RX FIFO is full
- `TxFifoEmpty`: TX FIFO is empty
- `ByteReceived`: A byte has been received
- `ByteTransmitted`: A byte has been transmitted
- `TransComplete`: Transaction is complete
- `SlaveAddressed`: Slave address matched
- `StopDetected`: STOP condition detected
- `StartDetected`: START condition detected

## Hardware Specifics

### Register Differences

The implementation handles chip-specific register differences using conditional compilation:

- **ESP32**: Uses legacy FIFO access methods and different register layouts
- **ESP32-S2**: Similar to ESP32 with some improvements
- **ESP32-S3/C3/H2**: Modern register layout with enhanced features
- **ESP32-C6**: Latest architecture with additional features:
  - Register-based mode (`fifo_addr_cfg_en`)
  - Enhanced slave TX auto-start
  - Improved FIFO watermark interrupts
  - Configuration update mechanism (`conf_upgate`)

### Clock Stretching

Clock stretching allows the slave to hold the SCL line low to slow down the master:
- Not available on ESP32
- Configurable on newer chips (S2, S3, C3, C6, H2)
- **ESP32-C6 Special Note**: Clock stretching is **disabled by default** to prevent bus hangs during TX operations. The hardware can hold the bus indefinitely if TX FIFO is empty, which causes communication failures.
- **Impact on 32-byte packets**: Without clock stretching, the slave will NACK if the RX FIFO fills completely (32 bytes). Use interrupt-driven reception for packets at or near FIFO capacity.

### Filters

Signal filters help with noise rejection:
- SDA and SCL filters configurable independently
- Filter threshold set in APB clock cycles
- Default threshold: 7 cycles

### Timeout Mechanism

The driver includes a software timeout to prevent infinite waiting:
- Default: 1000ms (1 second)
- Configurable via `with_timeout_ms()`
- Returns `Error::Timeout` if no transaction occurs within timeout period
- Prevents blocking forever when master doesn't communicate

### Register-Based Mode (ESP32-C6)

The ESP32-C6 hardware supports a special mode for emulating register-based I2C devices:

**How it works:**
- When enabled, the first byte after slave address is treated as a "register address"
- This byte is stored separately (not in main RX FIFO)
- Subsequent bytes go into normal RX FIFO as data
- Call `read_register_address()` to retrieve the register byte

**Use cases:**
- Emulating sensors (e.g., temperature sensor with register map)
- Implementing I2C EEPROM-like devices
- Any device with register-based protocol

## Error Handling

The driver provides detailed error types:

- `FifoExceeded`: Buffer size exceeds FIFO capacity
- `AcknowledgeCheckFailed`: ACK/NACK error
- `Timeout`: Operation timeout
- `ArbitrationLost`: Bus arbitration lost
- `ExecutionIncomplete`: Command execution incomplete
- `ZeroLengthInvalid`: Zero-length operation attempted
- `AddressInvalid`: Invalid I2C address
- `BusBusy`: Bus is busy
- `TxFifoOverflow`: TX FIFO overflow
- `RxFifoUnderflow`: RX FIFO underflow

## Implementation Notes

### FIFO Management

- FIFO size: 32 bytes (property-based, may vary by chip)
- Automatic FIFO reset on initialization and errors
- Read operations poll FIFO status
- Write operations check for FIFO space

### Pin Configuration

- SDA and SCL configured as open-drain
- **No internal pull-ups enabled** (external pull-ups required for I2C bus)
- Pins configured for both input and output
- Signal routing through GPIO matrix

### Interrupt Management

- Interrupts disabled by default
- User can enable specific events via `listen()`
- Async mode uses interrupts automatically
- Interrupt handler registered per core (multi-core support)
- Interrupts cleared at start of `wait_for_rx_data()` to prevent stale data

## Comparison with Master Driver

The slave driver shares many design patterns with the master driver:

### Similarities
- Same file structure and organization
- Same driver mode system (Blocking/Async)
- Similar configuration patterns
- Same pin connection methods
- Shared error handling philosophy
- Same interrupt management API

### Differences
- **Address**: Slave has its own address, master addresses targets
- **Initiation**: Slave responds to master requests, doesn't initiate
- **Clock**: Slave doesn't control clock, master does
- **Commands**: Slave doesn't use command queue like master
- **Transactions**: Slave handles individual bytes, master handles complete transactions
- **Timing**: Slave doesn't configure timing parameters

## Testing Recommendations

When testing the slave driver, consider:

1. **Basic Communication**: Test read and write with a master device
2. **Address Matching**: Verify slave responds only to its configured address
   - Test both 7-bit and 10-bit addresses
3. **FIFO Limits**: Test with data sizes at and beyond FIFO capacity (32 bytes)
4. **Clock Stretching**: Test with and without clock stretching enabled
   - On ESP32-C6, verify it stays disabled to prevent bus hangs
5. **Filtering**: Test in noisy environments with different filter settings
6. **Multi-byte Operations**: Test data transfers of various sizes
7. **Error Conditions**: Test timeout, bus busy, and other error scenarios
   - Verify timeout returns error after configured period
8. **Async Operations**: Test with async executor (Embassy)
9. **Register-Based Mode** (ESP32-C6): 
   - Test register address separation
   - Verify `read_register_address()` returns correct value
10. **Separate Transactions**: Verify slave can handle sequential write/read transactions
    - Note: Combined write_read() with repeated START is not yet supported

## Known Limitations

1. **FIFO Size Limit (32 bytes)**:
   - Hardware FIFO is limited to 32 bytes
   - Single `read()` call can only retrieve up to 32 bytes from FIFO
   - Single `write()` call can only load up to 32 bytes into TX FIFO
   - **For packets ≥ 32 bytes**: Use interrupt-driven reception with `RxFifoFull` event
   - **Critical**: In blocking mode without interrupts, a 32-byte packet will be NACKed because the FIFO fills completely before software can read it
   - **Solution**: Enable `Event::RxFifoFull` interrupt to read data as FIFO fills (threshold set at 30 bytes)
   - Example: See "Handling Large Packets" section above
   - Without interrupt handling, packets of 32 bytes or more will fail with NACK

2. **Combined write_read() Transactions**: 
   - Transactions with repeated START (no STOP between write and read) are **not supported**
   - Use separate write and read transactions instead
   - See `I2C_SLAVE_WRITE_READ_SUPPORT.md` for details and workarounds

3. **ESP32-C6 TX FIFO Requirement**:
   - Data must be preloaded into TX FIFO **before** master initiates read
   - If TX FIFO is empty during master read, communication may fail
   - Call `write()` to prepare response data in advance

4. **External Pull-ups Required**:
   - I2C slave devices must NOT enable internal pull-ups
   - Master device or external resistors must provide pull-ups
   - Typical values: 4.7kΩ or 10kΩ depending on bus capacitance

## Future Enhancements

Potential improvements for future versions:

1. **Combined write_read() Support**: Handle repeated START transactions properly
2. **DMA Support**: Integrate with DMA for larger data transfers
3. **General Call**: Support I2C general call address (0x00)
4. **Multi-master**: Handle multi-master scenarios more robustly
5. **Protocol Helpers**: Add helper methods for common I2C slave protocols
6. **Buffer Management**: Add built-in circular buffer support
7. **Hot-plug Detection**: Detect bus events more reliably
8. **Extended Register Mode**: Support for ESP32-S3 and other variants

## Recent Changes (October 2025)

- ✅ Added 10-bit address support for all variants
- ✅ Added configurable timeout for blocking reads
- ✅ Added register-based mode for ESP32-C6
- ✅ Fixed ESP32-C6 slave acknowledgment issues
- ✅ Fixed ESP32-C6 first byte loss in raw data mode
- ✅ Disabled clock stretching on ESP32-C6 to prevent bus hangs
- ✅ Improved interrupt handling with proper clearing
- ✅ Added comprehensive error handling with timeout detection
- ✅ Added `read_register_address()` API for register mode

## References

- ESP32 Technical Reference Manual - I2C Chapter
- ESP32-C6 Technical Reference Manual - I2C Chapter
- I2C Bus Specification (NXP)
- Master driver implementation: `../master/mod.rs`
- ESP-IDF I2C slave implementation (for hardware behavior reference)
- Documentation files:
  - `ESP32C6_I2C_SLAVE_FIRST_BYTE_ISSUE.md` - Register-based mode explanation
  - `I2C_SLAVE_WRITE_READ_SUPPORT.md` - write_read() limitation details

## License

Same as the parent esp-hal project (MIT OR Apache-2.0)
