# Master Support for write_read() Testing

## Overview

This document describes the master-side test infrastructure added to support comprehensive write_read() (repeated START) testing.

## Date: 2024-10-22

## New Test Master Structs

### 1. WriteReadTestMaster (functional.rs)

Blocking I2C master for write_read() testing with repeated START transactions.

**Methods:**

- `test_single_byte_write_read()` - Test 6a: Single byte register read
- `test_multi_byte_write_read()` - Test 6b: Multi-byte register read
- `test_register_mode_compatibility()` - Test 6c: Register-based mode compatibility
- `test_maximum_fifo_write_read()` - Test 6d: Maximum FIFO (32 bytes)
- `test_normal_mode_write_read()` - Test 6e: Normal mode verification
- `test_atomic_vs_separate()` - Test 6f: Atomicity comparison
- `test_esp32_compatible_write_read()` - Test 6g: ESP32 master compatibility
- `test_sequential_register_reads()` - Sequential register access
- `test_write_read_with_data()` - Register + data write, then read
- `test_repeated_start_verification()` - Repeated START behavior validation

**Usage Example:**

```rust
let mut master = WriteReadTestMaster::new(
    peripherals.I2C1,
    peripherals.GPIO1,
    peripherals.GPIO2,
)?;

// Test single byte write_read
let data = master.test_single_byte_write_read(0x10)?;
assert_eq!(data, 0xAA);

// Test multi-byte write_read
let data = master.test_multi_byte_write_read(0x20, 4)?;
assert_eq!(data.len(), 4);
```

### 2. AsyncWriteReadTestMaster (async_support.rs)

Async I2C master for write_read() testing with embassy-executor support.

**Methods:**

- `test_single_byte_write_read()` - Async single byte read
- `test_multi_byte_write_read()` - Async multi-byte read
- `test_register_mode_compatibility()` - Async register mode test
- `test_maximum_fifo_write_read()` - Async max FIFO test
- `test_write_read_with_timeout()` - write_read with timeout
- `test_concurrent_write_read()` - Multiple concurrent operations
- `test_write_read_with_progress()` - Progress monitoring
- `test_atomic_vs_separate()` - Async atomicity comparison
- `test_rapid_sequential_reads()` - Rapid sequential access
- `test_write_read_with_retry()` - Error recovery with retries

**Usage Example:**

```rust
let mut master = AsyncWriteReadTestMaster::new(
    peripherals.I2C1,
    peripherals.GPIO1,
    peripherals.GPIO2,
)?;

// Test with timeout
let data = master.test_write_read_with_timeout(0x10, 100).await?;

// Test concurrent operations
let results = master.test_concurrent_write_read(5).await?;
assert_eq!(results.len(), 5);
```

## New Utility Functions (common.rs)

### Assertions Module Extensions

**`assert_write_read_response()`**
- Validates write_read response matches expected data
- Provides detailed error messages with register address

**`assert_repeated_start_used()`**
- Placeholder for logic analyzer verification
- Documents expectation of repeated START (no STOP)

**`assert_atomic_behavior()`**
- Compares write_read vs separate transactions
- Validates atomicity in single-master scenarios

### write_read Module (NEW)

Specialized utilities for write_read() testing.

#### Register Addresses

Standard test register addresses:
```rust
pub const STATUS: u8 = 0x00;
pub const CONFIG: u8 = 0x01;
pub const DATA: u8 = 0x10;
pub const VERSION: u8 = 0xFE;
pub const ID: u8 = 0xFF;
```

#### Key Functions

**`generate_register_response(register, size)`**
- Generates expected response for register reads
- Simulates sensor/device behavior
- Useful for slave emulation testing

**`validate_register_response(register, response)`**
- Validates actual response matches expected pattern
- Returns true if correct

**`create_register_write(register, data)`**
- Creates register write command (register + data)
- For testing write-then-read patterns

**`extract_register(write_data)`**
- Extracts register address from write phase
- Returns `Option<u8>`

**`expected_timing_us(frequency, write_bytes, read_bytes)`**
- Calculates expected timing for write_read
- Returns `(min_us, max_us)` bounds
- Useful for performance validation

#### Usage Example

```rust
use common::write_read;

// Generate expected response for STATUS register
let expected = write_read::generate_register_response(
    write_read::registers::STATUS,
    4
);

// Perform write_read
let mut actual = vec![0u8; 4];
master.write_read(&[write_read::registers::STATUS], &mut actual)?;

// Validate
assert!(write_read::validate_register_response(
    write_read::registers::STATUS,
    &actual
));
```

## Module Documentation Updates

### functional.rs

Added comprehensive module documentation listing all master types:

- BasicCommMaster
- AddressTestMaster
- FifoTestMaster
- ClockStretchMaster
- FilterTestMaster
- InterruptTestMaster
- ErrorTestMaster
- **WriteReadTestMaster (NEW)**

### async_support.rs

Updated to include async write_read support:

- AsyncTestMaster
- AsyncOperationsMaster
- ConcurrentTestMaster
- **AsyncWriteReadTestMaster (NEW)**

### mod.rs

Added overview of write_read testing support with references to:
- `I2C_SLAVE_WRITE_READ_SUPPORT.md`
- `ESP32_MASTER_COMPATIBILITY.md`

## Test Coverage

### Blocking Tests (WriteReadTestMaster)

- ✅ Single byte write_read
- ✅ Multi-byte write_read
- ✅ Register-based mode compatibility
- ✅ Maximum FIFO capacity
- ✅ Normal mode verification
- ✅ Atomic vs separate transactions
- ✅ ESP32 master compatibility
- ✅ Sequential register reads
- ✅ Register + data writes
- ✅ Repeated START verification

### Async Tests (AsyncWriteReadTestMaster)

- ✅ Basic async write_read
- ✅ Timeout handling
- ✅ Concurrent operations
- ✅ Progress monitoring
- ✅ Atomicity comparison
- ✅ Rapid sequential access
- ✅ Error recovery with retries

### Utility Tests

- ✅ Register response generation
- ✅ Timing calculations
- ✅ Register write commands
- ✅ Register extraction
- ✅ Pattern validation

## Integration with Test Suite

The master support integrates with:

1. **Functional Tests** (`test-suite/functional/basic_comm.rs`)
   - Uses `WriteReadTestMaster` for HIL tests
   - Tests 6a-6g reference these masters

2. **Async Tests** (`test-suite/async_tests/async_operations.rs`)
   - Uses `AsyncWriteReadTestMaster`
   - Embassy-executor integration

3. **Test Documentation** (`TESTING.md`)
   - References write_read master capabilities
   - Test procedures use master methods

## ESP32 Master Compatibility

Both master types include ESP32-specific testing:

- Quick response requirements (<10us)
- No clock stretching assumptions
- Compatibility verification methods

See: `ESP32_MASTER_COMPATIBILITY.md` for details

## Usage in HIL Tests

### Blocking Example

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_write_read_single_byte() {
    // Setup slave
    let slave = setup_slave();
    
    // Setup master
    let mut master = WriteReadTestMaster::new(
        peripherals.I2C1,
        master_sda_pin,
        master_scl_pin,
    ).unwrap();
    
    // Test write_read
    let data = master.test_single_byte_write_read(0x10).unwrap();
    
    // Validate
    assert_eq!(data, expected_value);
}
```

### Async Example

```rust
#[embassy_executor::test]
#[ignore = "Requires HIL setup"]
async fn test_async_write_read() {
    // Setup
    let slave = setup_async_slave();
    let mut master = AsyncWriteReadTestMaster::new(
        peripherals.I2C1,
        master_sda_pin,
        master_scl_pin,
    ).unwrap();
    
    // Test with timeout
    let data = master.test_write_read_with_timeout(0x10, 100)
        .await
        .unwrap();
    
    // Validate
    assert_eq!(data, expected);
}
```

## Dependencies

The master support requires:

- `esp-hal` I2C master driver
- `embassy-executor` (for async tests)
- `embassy-time` (for async timing)
- Logic analyzer (optional, for repeated START verification)

## Testing Checklist

When using the master support:

- [ ] Hardware connected (SDA, SCL, GND)
- [ ] Pull-up resistors installed (4.7kΩ)
- [ ] Slave address configured correctly
- [ ] Bus frequency compatible with slave
- [ ] Power supplies stable
- [ ] Logic analyzer connected (if verifying repeated START)
- [ ] ESP32 compatibility mode if needed

## Future Enhancements

Possible additions:

1. **Multi-master testing**
   - Test atomicity with competing masters
   - Bus arbitration scenarios

2. **Performance profiling**
   - Detailed timing measurements
   - Throughput benchmarks

3. **Error injection**
   - Simulated NACK conditions
   - Bus errors during write_read

4. **Protocol emulation**
   - SMBus block read/write
   - PMBus commands
   - Sensor-specific protocols

## References

- `I2C_SLAVE_WRITE_READ_SUPPORT.md` - Slave implementation details
- `ESP32_MASTER_COMPATIBILITY.md` - ESP32 master issues
- `TESTING.md` - Complete test checklist
- `TEST_SUITE_UPDATES.md` - Test suite changes

## Conclusion

The master support infrastructure provides comprehensive tools for testing write_read() functionality in both blocking and async modes. With dedicated test masters, utility functions, and integration with the test suite, developers can thoroughly validate I2C slave write_read() behavior.

All features support the confirmed capability that **write_read() is FULLY SUPPORTED** on ESP32-C6 and other modern chips in normal mode.
