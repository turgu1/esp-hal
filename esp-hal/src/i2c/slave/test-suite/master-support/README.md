# I2C Master Support for HIL Testing

This module provides I2C master implementations specifically designed to support Hardware-in-Loop (HIL) testing of the I2C slave driver.

## Overview

When testing an I2C slave device, you need an I2C master to initiate transactions. This module provides specialized master implementations that:

1. **Simplify test setup** - Pre-configured masters for specific test scenarios
2. **Provide clear APIs** - Test-oriented methods that express intent clearly
3. **Track results** - Built-in statistics and performance measurement
4. **Handle complexity** - Abstract away low-level master configuration details

## Module Structure

```
master-support/
├── mod.rs              - Module entry point
├── common.rs           - Reusable utilities for all masters
├── functional.rs       - Masters for functional tests
├── async_support.rs    - Masters for async tests
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

assertions::assert_buffers_equal(&expected, &actual)?;
assertions::assert_within_timeout(elapsed_us, max_timeout_us)?;
assertions::assert_data_rate(bytes, time_us, min_rate_bps)?;
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

## Async Support Masters (`async_support.rs`)

### AsyncTestMaster

For async I2C operations:

```rust
use esp_hal::i2c::slave::test_suite::master_support::async_support::AsyncTestMaster;

let mut master = AsyncTestMaster::new(i2c0, sda, scl)?;

// Async write
master.async_write(&[0x01, 0x02]).await?;

// Async read
let mut buffer = [0u8; 4];
master.async_read(&mut buffer).await?;

// Async write-read
master.async_write_read(&[0x01], &mut buffer).await?;
```

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

## Design Principles

1. **Test-Oriented API**: Methods named after what they test, not low-level operations
2. **Reusable Components**: Common utilities shared across all master types
3. **Built-in Validation**: Result types include statistics and validation helpers
4. **Clear Intent**: Each master type corresponds to a test category
5. **Minimal Setup**: Pre-configured for typical test scenarios
6. **Comprehensive Coverage**: Masters for all test categories in TESTING.md

## Hardware Setup

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
- **esp-hal I2C Master**: `esp_hal::i2c::master`
