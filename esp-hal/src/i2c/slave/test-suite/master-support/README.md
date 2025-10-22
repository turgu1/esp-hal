# I2C Master Support for HIL Testing

This module provides I2C master implementations specifically designed to support Hardware-in-Loop (HIL) testing of the I2C slave driver.

## Overview

When testing an I2C slave device, you need an I2C master to initiate transactions. This module provides specialized master implementations that:

1. **Simplify test setup** - Pre-configured masters for specific test scenarios
2. **Provide clear APIs** - Test-oriented methods that express intent clearly
3. **Track results** - Built-in statistics and performance measurement
4. **Handle complexity** - Abstract away low-level master configuration details
5. **Support write_read()** - Comprehensive repeated START transaction testing

**New in this version:** Extensive write_read() (repeated START) support confirming that ESP32-C6 and modern chips fully handle I2C write_read() operations in normal mode.

See: `WRITE_READ_SUPPORT.md` for detailed write_read() implementation

## Module Structure

```
master-support/
├── README.md           - This file
├── WRITE_READ_SUPPORT.md - write_read() implementation details
├── mod.rs              - Module entry point
├── common.rs           - Reusable utilities (includes write_read module)
├── functional.rs       - Masters for functional tests (includes WriteReadTestMaster)
├── async_support.rs    - Masters for async tests (includes AsyncWriteReadTestMaster)
├── performance.rs      - Masters for performance measurement
├── reliability.rs      - Masters for stress and recovery tests
└── integration.rs      - Masters for integration tests
```

## Common Utilities (`common.rs`)

### TestMaster Wrapper

Wraps `esp_hal::i2c::Master` with test-friendly configuration:

```rust
use esp_hal::i2c::slave::test_suite::master_support::common::TestMaster;

let mut master = TestMaster::new(i2c0_peripheral, sda_pin, scl_pin, config)?;
master.write(&[0x01, 0x02, 0x03])?;
```

### Pattern Generators

Generate test data patterns:

```rust
use esp_hal::i2c::slave::test_suite::master_support::common::patterns;

let mut buffer = [0u8; 32];
patterns::sequential(&mut buffer, 0);      // [0, 1, 2, 3, ...]
patterns::constant(&mut buffer, 0xAA);     // [0xAA, 0xAA, ...]
patterns::alternating(&mut buffer);         // [0xAA, 0x55, 0xAA, ...]
patterns::pseudo_random(&mut buffer, 42);  // Deterministic random
```

### Timing Utilities

Measure operation time and add delays:

```rust
use esp_hal::i2c::slave::test_suite::master_support::common::timing;

let timer = timing::Timer::new();
// ... operation ...
let elapsed_us = timer.elapsed_us();

timing::delay_ms(10);
timing::delay_us(500);
```

### Assertions

Test-specific assertions:

```rust
use esp_hal::i2c::slave::test_suite::master_support::common::assertions;

assertions::assert_buffers_equal(&expected, &actual, "Test name")?;
assertions::assert_within_timeout(elapsed_us, max_timeout_us, "Operation")?;
assertions::assert_data_rate(bytes, time_us, min_rate_bps, max_rate_bps)?;

// write_read() specific assertions
assertions::assert_write_read_response(register, &expected, &actual)?;
assertions::assert_repeated_start_used(timing_data, "Test name")?;
assertions::assert_atomic_behavior(&wr_result, &sep_result, "Atomicity test")?;
```

### write_read() Utilities (NEW)

Specialized utilities for write_read() testing:

```rust
use esp_hal::i2c::slave::test_suite::master_support::common::write_read;

// Standard register addresses
let status_reg = write_read::registers::STATUS;    // 0x00
let config_reg = write_read::registers::CONFIG;    // 0x01
let data_reg = write_read::registers::DATA;        // 0x10

// Generate expected register response
let expected = write_read::generate_register_response(status_reg, 4);

// Validate response
assert!(write_read::validate_register_response(status_reg, &response));

// Create register write command
let cmd = write_read::create_register_write(0x10, &[0xAA, 0xBB]);

// Extract register from write data
let reg = write_read::extract_register(&write_data);

// Calculate expected timing
let (min_us, max_us) = write_read::expected_timing_us(100_000, 1, 4);
```

## Functional Test Masters (`functional.rs`)

### BasicCommMaster

For basic communication tests (TESTING.md tests 1-6):

```rust
use esp_hal::i2c::slave::test_suite::master_support::functional::BasicCommMaster;

let mut master = BasicCommMaster::new(i2c0, sda, scl)?;

// Test simple write
master.test_simple_write(&[0x01, 0x02])?;

// Test simple read
let data = master.test_simple_read(4)?;

// Test multi-byte operations
master.test_multi_byte_write(&[0x01, 0x02, 0x03, 0x04])?;
let data = master.test_multi_byte_read(16)?;

// Test FIFO capacity
master.test_maximum_fifo()?;
master.test_beyond_fifo_capacity()?;
```

### AddressTestMaster

For address handling tests (tests 7-9):

```rust
use esp_hal::i2c::slave::test_suite::master_support::functional::AddressTestMaster;

let mut master = AddressTestMaster::new(i2c0, sda, scl)?;

// Test correct address
master.test_correct_address(0x55, &[0x01])?;

// Test wrong address (should get NACK)
let result = master.test_wrong_address(0x66, &[0x01]);
assert!(result.is_err());

// Change address and test
master.change_address(0x77);
master.test_correct_address(0x77, &[0x02])?;

// Test general call
master.test_general_call(&[0x03])?;
```

### ClockStretchMaster

For clock stretching tests (tests 10-11):

```rust
use esp_hal::i2c::slave::test_suite::master_support::functional::ClockStretchMaster;

let mut master = ClockStretchMaster::new(i2c0, sda, scl)?;

// Test with clock stretching enabled
master.test_with_clock_stretch()?;

// Test without clock stretching
master.test_without_clock_stretch()?;

// Rapid writes to trigger stretching
master.rapid_write(100)?;
```

### Other Functional Masters

- **FifoTestMaster**: FIFO operations and overflow testing
- **FilterTestMaster**: Noise filtering tests (tests 12-13)
- **InterruptTestMaster**: Interrupt triggering (tests 14-17)
- **ErrorTestMaster**: Error conditions (tests 18-20)
- **WriteReadTestMaster**: write_read() with repeated START (tests 6a-6g) ✨ NEW

### WriteReadTestMaster (NEW)

For testing write_read() operations with repeated START:

```rust
use esp_hal::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;

let mut master = WriteReadTestMaster::new(i2c0, sda, scl)?;

// Test 6a: Single byte write_read
let data = master.test_single_byte_write_read(0x10)?;
assert_eq!(data, 0xAA);

// Test 6b: Multi-byte write_read
let data = master.test_multi_byte_write_read(0x20, 4)?;
assert_eq!(data.len(), 4);

// Test 6c: Register-based mode compatibility
let data = master.test_register_mode_compatibility(0x30)?;

// Test 6d: Maximum FIFO write_read
let data = master.test_maximum_fifo_write_read(0x40)?;
assert_eq!(data.len(), 32);

// Test 6e: Normal mode (confirms no special config needed)
let data = master.test_normal_mode_write_read(0x50, 4)?;

// Test 6f: Atomic vs separate transactions
let (atomic, separate) = master.test_atomic_vs_separate(0x60, 4)?;

// Test 6g: ESP32 master compatibility
let data = master.test_esp32_compatible_write_read(0x70)?;

// Sequential register reads
let results = master.test_sequential_register_reads(0x10, 5)?;

// Register + data write, then read
let data = master.test_write_read_with_data(0x80, &[0x11, 0x22], 4)?;

// Verify repeated START (no STOP between phases)
let data = master.test_repeated_start_verification(0x90)?;
```

**Key Features:**
- All tests use repeated START (no STOP between write and read)
- ESP32 (original) compatibility testing
- Register emulation support
- Timing validation
- Normal mode and register-based mode testing

## Async Support Masters (`async_support.rs`)

### AsyncTestMaster

For async I2C operations:

```rust
use esp_hal::i2c::slave::test_suite::master_support::async_support::AsyncTestMaster;

let mut master = AsyncTestMaster::new(i2c0, sda, scl, config)?;

// Async write
master.write(&[0x01, 0x02]).await?;

// Async read
let mut buffer = [0u8; 4];
master.read(&mut buffer).await?;

// Async write_read (repeated START)
master.write_read(&[0x01], &mut buffer).await?;
```

### AsyncWriteReadTestMaster (NEW)

For async write_read() testing with embassy-executor:

```rust
use esp_hal::i2c::slave::test_suite::master_support::async_support::AsyncWriteReadTestMaster;

let mut master = AsyncWriteReadTestMaster::new(i2c0, sda, scl)?;

// Basic async write_read
let data = master.test_single_byte_write_read(0x10).await?;

// Multi-byte async write_read
let data = master.test_multi_byte_write_read(0x20, 4).await?;

// Async with timeout
let data = master.test_write_read_with_timeout(0x30, 100).await?;

// Concurrent write_read operations
let results = master.test_concurrent_write_read(5).await?;

// Progress monitoring
let data = master.test_write_read_with_progress(0x40, |elapsed| {
    println!("Elapsed: {} us", elapsed);
}).await?;

// Atomic vs separate (async)
let (atomic, separate) = master.test_atomic_vs_separate(0x50, 4).await?;

// Rapid sequential reads
let results = master.test_rapid_sequential_reads(0x60, 10).await?;

// Error recovery with retry
let data = master.test_write_read_with_retry(0x70, 3).await?;
```

**Key Features:**
- All async/await operations
- Timeout support with embassy_time
- Concurrent operation testing
- Progress monitoring
- Automatic retry with exponential backoff
- Error recovery testing

### ConcurrentTestMaster

For testing concurrent async operations:

```rust
use esp_hal::i2c::slave::test_suite::master_support::async_support::ConcurrentTestMaster;

let mut master = ConcurrentTestMaster::new(i2c0, sda, scl)?;

// Continuous write operations
master.continuous_write(100, 10).await?;

// Interleaved read/write
master.interleaved_operations(50).await?;
```

## Performance Masters (`performance.rs`)

### SpeedTestMaster

For testing different bus speeds:

```rust
use esp_hal::i2c::slave::test_suite::master_support::performance::SpeedTestMaster;

// Standard mode (100kHz)
let mut master = SpeedTestMaster::new_standard_mode(i2c0, sda, scl)?;

// Fast mode (400kHz)
let mut master = SpeedTestMaster::new_fast_mode(i2c0, sda, scl)?;

// Fast mode plus (1MHz)
let mut master = SpeedTestMaster::new_fast_plus_mode(i2c0, sda, scl)?;

// Test reliability at current speed
let results = master.test_reliability(100)?;
println!("Success rate: {:.2}%", results.success_rate());
println!("Avg time: {} us", results.average_time());

// Measure transaction time
let time_us = master.measure_transaction_time(&[0x01, 0x02])?;

// Test maximum rate
let rate = master.test_maximum_rate()?;
println!("Max rate: {:.2} transactions/sec", rate.transactions_per_second);
```

### ThroughputTestMaster

For measuring data throughput:

```rust
use esp_hal::i2c::slave::test_suite::master_support::performance::ThroughputTestMaster;

let mut master = ThroughputTestMaster::new(i2c0, sda, scl)?;

// Single-byte throughput
let result = master.test_single_byte_throughput(1000)?;
println!("Throughput: {} bytes/sec", result.bytes_per_second());

// Bulk transfer throughput
let result = master.test_bulk_throughput(100, 32)?;
println!("Throughput: {} KB/sec", result.bytes_per_second() / 1024);

// FIFO-optimal throughput
let result = master.test_fifo_optimal_throughput()?;
println!("Efficiency: {:.2}%", result.efficiency_percent());

// Sustained throughput
let result = master.test_sustained_throughput(5_000)?;
println!("Sustained: {} Mbps", result.bits_per_second() / 1_000_000);
```

## Reliability Masters (`reliability.rs`)

### StressTestMaster

For stress testing:

```rust
use esp_hal::i2c::slave::test_suite::master_support::reliability::StressTestMaster;

let mut master = StressTestMaster::new(i2c0, sda, scl)?;

// Continuous stress test
let stats = master.run_continuous_stress(10_000)?;
println!("Success rate: {:.2}%", stats.success_rate());
println!("Transactions/sec: {:.2}", stats.transactions_per_second());

// Burst stress test
let stats = master.run_burst_stress(100, 10)?;

// Variable size stress
let stats = master.run_variable_size_stress(1000)?;

// Random pattern stress
let stats = master.run_random_pattern_stress(1000)?;

// Maximum throughput stress
let stats = master.run_maximum_throughput_stress(5000)?;
```

### RecoveryTestMaster

For testing error recovery:

```rust
use esp_hal::i2c::slave::test_suite::master_support::reliability::RecoveryTestMaster;

let mut master = RecoveryTestMaster::new(i2c0, sda, scl)?;

// Test bus error recovery
let result = master.test_bus_error_recovery()?;
assert!(result.recovered());

// Test timeout recovery
let result = master.test_timeout_recovery()?;

// Test FIFO overflow recovery
let result = master.test_fifo_overflow_recovery()?;

// Test repeated recovery cycles
let result = master.test_repeated_recovery(10)?;
```

## Integration Masters (`integration.rs`)

### PeripheralIntegrationMaster

For testing with other peripherals active:

```rust
use esp_hal::i2c::slave::test_suite::master_support::integration::PeripheralIntegrationMaster;

let mut master = PeripheralIntegrationMaster::new(i2c0, sda, scl)?;

// Test with SPI active
let result = master.test_with_spi_active()?;

// Test with UART active
let result = master.test_with_uart_active()?;

// Test with GPIO interrupts
let result = master.test_with_gpio_interrupts()?;

// Test with WiFi active
let result = master.test_with_wifi_active()?;

println!("Success rate with WiFi: {:.2}%", result.success_rate());
println!("Timing variance: {} us", result.timing_variance());
```

### OsIntegrationMaster

For testing with OS/frameworks:

```rust
use esp_hal::i2c::slave::test_suite::master_support::integration::OsIntegrationMaster;

let mut master = OsIntegrationMaster::new(i2c0, sda, scl)?;

// Test blocking operation (for RTOS)
master.test_blocking_operation()?;

// Test message passing
let messages = master.test_message_passing(10)?;

// Test synchronization
master.test_synchronization()?;

// Test with shared resource (mutex)
master.test_shared_resource()?;
```

## Usage Examples

### Basic HIL Test

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_basic_write_read() {
    use esp_hal::i2c::slave::test_suite::master_support::functional::BasicCommMaster;
    
    // Setup master
    let peripherals = Peripherals::take();
    let mut master = BasicCommMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Setup slave (on another device or different I2C peripheral)
    let slave = I2c::new(
        peripherals.I2C1,
        peripherals.GPIO18,
        peripherals.GPIO19,
        Config::default().with_address(0x55),
    );
    
    // Master writes data
    let test_data = [0x01, 0x02, 0x03, 0x04];
    master.test_simple_write(&test_data).unwrap();
    
    // Verify slave received the data
    assert_eq!(slave.read_buffer(), &test_data);
}
```

### Performance Measurement Test

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_throughput() {
    use esp_hal::i2c::slave::test_suite::master_support::performance::ThroughputTestMaster;
    
    let peripherals = Peripherals::take();
    let mut master = ThroughputTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Measure bulk throughput
    let result = master.test_bulk_throughput(100, 32).unwrap();
    
    println!("Throughput: {} bytes/sec", result.bytes_per_second());
    println!("Efficiency: {:.2}%", result.efficiency_percent());
    
    // Assert minimum performance
    assert!(result.bytes_per_second() > 10_000); // At least 10 KB/sec
}
```

### Async Test

```rust
#[embassy_executor::test]
async fn test_async_communication() {
    use esp_hal::i2c::slave::test_suite::master_support::async_support::AsyncTestMaster;
    
    let peripherals = Peripherals::take();
    let mut master = AsyncTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Async write operation
    master.async_write(&[0xAA, 0xBB, 0xCC]).await.unwrap();
    
    // Async read operation
    let mut buffer = [0u8; 3];
    master.async_read(&mut buffer).await.unwrap();
    
    assert_eq!(buffer, [0xAA, 0xBB, 0xCC]);
}
```

## write_read() Testing Examples (NEW)

### Basic write_read() Test

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_write_read_single_byte() {
    use esp_hal::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;
    
    // Setup master
    let peripherals = Peripherals::take();
    let mut master = WriteReadTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Setup slave (on another device)
    // Slave should respond to register 0x10 with 0xAA
    
    // Test write_read: write [0x10], read 1 byte
    let data = master.test_single_byte_write_read(0x10).unwrap();
    
    assert_eq!(data, 0xAA);
}
```

### Register Emulation Test

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_register_emulation() {
    use esp_hal::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;
    use esp_hal::i2c::slave::test_suite::master_support::common::write_read;
    
    let peripherals = Peripherals::take();
    let mut master = WriteReadTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Test reading STATUS register
    let data = master.test_single_byte_write_read(write_read::registers::STATUS).unwrap();
    
    // Validate response
    let expected = write_read::generate_register_response(
        write_read::registers::STATUS,
        1
    );
    assert_eq!(data, expected[0]);
    
    // Test reading DATA register (multi-byte)
    let data = master.test_multi_byte_write_read(
        write_read::registers::DATA,
        4
    ).unwrap();
    
    assert!(write_read::validate_register_response(
        write_read::registers::DATA,
        &data
    ));
}
```

### Async write_read() Test

```rust
#[embassy_executor::test]
async fn test_async_write_read() {
    use esp_hal::i2c::slave::test_suite::master_support::async_support::AsyncWriteReadTestMaster;
    
    let peripherals = Peripherals::take();
    let mut master = AsyncWriteReadTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Test with timeout
    let data = master.test_write_read_with_timeout(0x10, 100).await.unwrap();
    assert_eq!(data.len(), 4);
    
    // Test concurrent operations
    let results = master.test_concurrent_write_read(5).await.unwrap();
    assert_eq!(results.len(), 5);
    
    // Test with retry on error
    let data = master.test_write_read_with_retry(0x20, 3).await.unwrap();
    assert!(!data.is_empty());
}
```

### ESP32 Compatibility Test

```rust
#[test]
#[ignore = "Requires HIL setup with ESP32 master"]
fn test_esp32_master_compatibility() {
    use esp_hal::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;
    
    let peripherals = Peripherals::take();
    let mut master = WriteReadTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Test with ESP32 (original) as master
    // Slave must respond quickly (<10us) without clock stretching
    let data = master.test_esp32_compatible_write_read(0x10).unwrap();
    
    // Should complete successfully despite ESP32's poor clock stretch support
    assert_eq!(data.len(), 4);
}
```

### Atomicity Comparison Test

```rust
#[test]
#[ignore = "Requires HIL setup"]
fn test_write_read_atomicity() {
    use esp_hal::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;
    use esp_hal::i2c::slave::test_suite::master_support::common::assertions;
    
    let peripherals = Peripherals::take();
    let mut master = WriteReadTestMaster::new(
        peripherals.I2C0,
        peripherals.GPIO21,
        peripherals.GPIO22,
    ).unwrap();
    
    // Compare atomic write_read vs separate transactions
    let (atomic_result, separate_result) = master
        .test_atomic_vs_separate(0x30, 4)
        .unwrap();
    
    // In single-master scenario, both should produce same data
    assertions::assert_atomic_behavior(
        &atomic_result,
        &separate_result,
        "write_read atomicity"
    );
    
    // write_read has no STOP between phases (verify with logic analyzer)
    println!("Atomic result: {:?}", atomic_result);
    println!("Separate result: {:?}", separate_result);
}
```

## Design Principles

1. **Test-Oriented API**: Methods named after what they test, not low-level operations
2. **Reusable Components**: Common utilities shared across all master types
3. **Built-in Validation**: Result types include statistics and validation helpers
4. **Clear Intent**: Each master type corresponds to a test category
5. **Minimal Setup**: Pre-configured for typical test scenarios
6. **Comprehensive Coverage**: Masters for all test categories in TESTING.md
7. **write_read() Support**: Dedicated masters for repeated START testing ✨ NEW

## References

For HIL tests using these masters:

```
Master Device (ESP32 #1)          Slave Device (ESP32 #2)
┌─────────────────────┐          ┌─────────────────────┐
│                     │          │                     │
│  GPIO21 (SDA) ──────┼──────────┼────── GPIO18 (SDA) │
│                     │    4.7k  │                     │
│  GPIO22 (SCL) ──────┼────/\/\──┼────── GPIO19 (SCL) │
│                     │    4.7k  │                     │
│  GND ───────────────┼──────────┼─────────────── GND │
│                     │          │                     │
└─────────────────────┘          └─────────────────────┘
        │                                  │
        └──────── USB Serial ──────────────┘
              (for test output)
```

## Contributing

When adding new master implementations:

1. Choose the appropriate module (functional, async, performance, etc.)
2. Follow the naming pattern: `XxxTestMaster` or `XxxMaster`
3. Provide clear method names that express test intent
4. Include result types with statistics when appropriate
5. Add usage examples in comments
6. Update this README with the new master type

## References

- **Test Suite**: `../` (parent directory)
- **TESTING.md**: `../../TESTING.md` (test checklist)
- **Driver Implementation**: `../../mod.rs`
- **write_read() Implementation**: `../../I2C_SLAVE_WRITE_READ_SUPPORT.md`
- **ESP32 Compatibility**: `../../ESP32_MASTER_COMPATIBILITY.md`
- **write_read() Master Details**: `WRITE_READ_SUPPORT.md`
- **esp-hal I2C Master**: `esp_hal::i2c::master`

