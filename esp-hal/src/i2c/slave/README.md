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
- ESP32-C6
- ESP32-H2

Note: ESP32-P4 and ESP32-C2 do not have I2C slave support listed in the metadata.

## Features

### Blocking Mode
- Basic read/write operations
- Configurable slave address (7-bit)
- Clock stretching support
- SDA/SCL signal filtering
- Interrupt support

### Async Mode
- Asynchronous read/write operations
- Future-based API
- Compatible with async executors like Embassy

## Configuration Options

The driver can be configured with:
- **Slave Address**: 7-bit I2C address (default: 0x55)
- **Clock Stretching**: Enable/disable clock stretching (default: enabled)
- **SDA Filter**: Enable/disable and configure threshold (default: enabled, threshold 7)
- **SCL Filter**: Enable/disable and configure threshold (default: enabled, threshold 7)

## Usage Examples

### Basic Blocking Mode

```rust
use esp_hal::i2c::slave::{Config, I2c};

let config = Config::default().with_address(0x55.into());

let mut i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);

// Read data from master
let mut buffer = [0u8; 128];
let bytes_read = i2c.read(&mut buffer)?;

// Write data to master
i2c.write(&[0xAA, 0xBB, 0xCC])?;
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
- **ESP32-S3/C3/C6/H2**: Modern register layout with enhanced features

### Clock Stretching

Clock stretching allows the slave to hold the SCL line low to slow down the master:
- Not available on ESP32
- Configurable on newer chips (S2, S3, C3, C6, H2)

### Filters

Signal filters help with noise rejection:
- SDA and SCL filters configurable independently
- Filter threshold set in APB clock cycles
- Default threshold: 7 cycles

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

- SDA and SCL configured as open-drain with pull-up
- Pins configured for both input and output
- Signal routing through GPIO matrix

### Interrupt Management

- Interrupts disabled by default
- User can enable specific events via `listen()`
- Async mode uses interrupts automatically
- Interrupt handler registered per core (multi-core support)

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
3. **FIFO Limits**: Test with data sizes at and beyond FIFO capacity
4. **Clock Stretching**: Test with and without clock stretching enabled
5. **Filtering**: Test in noisy environments with different filter settings
6. **Multi-byte Operations**: Test data transfers of various sizes
7. **Error Conditions**: Test timeout, bus busy, and other error scenarios
8. **Async Operations**: Test with async executor (Embassy)

## Future Enhancements

Potential improvements for future versions:

1. **10-bit Addressing**: Add support for 10-bit I2C addresses
2. **DMA Support**: Integrate with DMA for larger data transfers
3. **General Call**: Support I2C general call address (0x00)
4. **Multi-master**: Handle multi-master scenarios more robustly
5. **Protocol Helpers**: Add helper methods for common I2C slave protocols
6. **Buffer Management**: Add built-in circular buffer support
7. **Hot-plug Detection**: Detect bus events more reliably

## References

- ESP32 Technical Reference Manual - I2C Chapter
- I2C Bus Specification (NXP)
- Master driver implementation: `../master/mod.rs`
- ESP-IDF I2C slave implementation (for hardware behavior reference)

## License

Same as the parent esp-hal project (MIT OR Apache-2.0)
