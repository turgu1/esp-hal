# I2C Slave Driver Test Suite

This directory contains a comprehensive test suite for the I2C slave driver implementation.

## Test Organization

The tests are organized into modules matching the categories in `TESTING.md`:

```
test-suite/
├── README.md                    (this file)
├── mod.rs                       (test suite entry point)
├── unit/
│   ├── mod.rs                  (unit tests)
│   ├── config_tests.rs         (configuration tests)
│   ├── driver_tests.rs         (driver instantiation tests)
│   └── error_tests.rs          (error handling tests)
├── functional/
│   ├── mod.rs                  (functional tests)
│   ├── basic_comm.rs           (basic communication tests)
│   ├── address_tests.rs        (address matching tests)
│   ├── fifo_tests.rs           (FIFO management tests)
│   ├── clock_stretch_tests.rs  (clock stretching tests)
│   ├── filter_tests.rs         (filtering tests)
│   ├── interrupt_tests.rs      (interrupt tests)
│   └── error_condition_tests.rs (error condition tests)
├── async_tests/
│   ├── mod.rs                  (async mode tests)
│   ├── async_operations.rs     (async read/write tests)
│   ├── concurrent_tests.rs     (concurrent operation tests)
│   └── future_tests.rs         (future cancellation tests)
├── performance/
│   ├── mod.rs                  (performance tests)
│   ├── speed_tests.rs          (bus speed tests)
│   └── throughput_tests.rs     (throughput tests)
├── reliability/
│   ├── mod.rs                  (reliability tests)
│   ├── stress_tests.rs         (stress tests)
│   └── recovery_tests.rs       (error recovery tests)
├── integration/
│   ├── mod.rs                  (integration tests)
│   ├── peripheral_tests.rs     (with other peripherals)
│   └── os_tests.rs            (with OS/frameworks)
├── helpers/
│   ├── mod.rs                  (test helpers)
│   ├── mock_master.rs          (mock I2C master)
│   └── test_utils.rs           (utility functions)
└── master-support/
    ├── mod.rs                  (master support entry)
    ├── common.rs               (common utilities)
    ├── functional.rs           (functional test masters)
    ├── async_support.rs        (async test masters)
    ├── performance.rs          (performance masters)
    ├── reliability.rs          (stress/recovery masters)
    └── integration.rs          (integration test masters)
```

## Running Tests

### All Tests

```bash
cargo test --package esp-hal --lib i2c::slave::test_suite
```

### Specific Test Module

```bash
cargo test --package esp-hal --lib i2c::slave::test_suite::unit
cargo test --package esp-hal --lib i2c::slave::test_suite::functional
```

### Single Test

```bash
cargo test --package esp-hal --lib i2c::slave::test_suite::unit::config_tests::test_default_config
```

### With Output

```bash
cargo test --package esp-hal --lib i2c::slave::test_suite -- --nocapture
```

## Hardware-in-Loop (HIL) Tests

Some tests require actual hardware and are marked with `#[cfg(feature = "hil-test")]`.

To run HIL tests:

```bash
cargo test --package esp-hal --lib i2c::slave::test_suite --features hil-test
```

### HIL Test Requirements

- Two ESP32 devices (one as master, one as slave)
- Or one ESP32 as slave with external I2C master
- Proper wiring with pull-up resistors (4.7kΩ recommended)
- Serial connection for test output

### Master Support for HIL Tests

The `master-support/` module provides I2C master implementations specifically designed for testing the slave driver:

**Common utilities** (`common.rs`):
- `TestMaster` wrapper around esp_hal I2C master
- Pattern generators (sequential, constant, alternating, pseudo-random)
- Timing utilities (Timer, delays)
- Assertions for buffer comparison and timing validation

**Test-specific masters**:
- `functional.rs`: BasicCommMaster, AddressTestMaster, FifoTestMaster, ClockStretchMaster, FilterTestMaster, InterruptTestMaster, ErrorTestMaster
- `async_support.rs`: AsyncTestMaster, AsyncOperationsMaster, ConcurrentTestMaster, FutureTestMaster
- `performance.rs`: SpeedTestMaster, ThroughputTestMaster with result tracking
- `reliability.rs`: StressTestMaster, RecoveryTestMaster with statistics
- `integration.rs`: PeripheralIntegrationMaster, OsIntegrationMaster, AsyncFrameworkMaster

Each master provides methods to trigger specific slave behaviors and validate responses, making HIL tests easier to write and maintain.

## Test Categories

### Unit Tests (`unit/`)

Tests that don't require hardware:
- Configuration validation
- Error type behavior
- Builder pattern functionality
- Address validation

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::unit
```

### Functional Tests (`functional/`)

Tests requiring hardware (HIL):
- Basic read/write operations
- Address matching
- FIFO operations
- Clock stretching
- Filtering
- Interrupts

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::functional --features hil-test
```

### Async Tests (`async_tests/`)

Tests for async operations:
- Async read/write
- Concurrent operations
- Future cancellation

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::async_tests --features hil-test
```

### Performance Tests (`performance/`)

Tests measuring performance:
- Bus speed tests (100kHz, 400kHz, 1MHz)
- Throughput measurements
- Latency tests

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::performance --features hil-test -- --nocapture
```

### Reliability Tests (`reliability/`)

Long-running tests:
- Stress tests
- Error recovery
- Extended duration tests

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::reliability --features hil-test -- --test-threads=1
```

### Integration Tests (`integration/`)

Tests with other components:
- With GPIO
- With UART
- With timers
- With Embassy
- With RTIC

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::integration --features hil-test
```

## Test Helpers

The `helpers/` module provides:
- **MockMaster**: Software-based I2C master for testing
- **TestUtils**: Common test utilities
- **Assertions**: Custom assertion macros

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let config = Config::default();
        
        // Act
        let result = config.with_address(0x55.into());
        
        // Assert
        assert_eq!(result.address, I2cAddress::SevenBit(0x55));
    }
}
```

### HIL Test Template

```rust
#[cfg(all(test, feature = "hil-test"))]
mod hil_tests {
    use super::*;
    use crate::i2c::slave::test_suite::helpers::MockMaster;

    #[test]
    fn test_feature_with_hardware() {
        // Setup hardware
        let peripherals = unsafe { Peripherals::steal() };
        let mut slave = I2c::new(peripherals.I2C0, Config::default())
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Create mock master
        let mut master = MockMaster::new(0x55);
        
        // Test interaction
        master.write(&[0xAA]);
        let mut buffer = [0u8; 1];
        let bytes = slave.read(&mut buffer).unwrap();
        
        assert_eq!(bytes, 1);
        assert_eq!(buffer[0], 0xAA);
    }
}
```

## Continuous Integration

These tests can be integrated into CI/CD pipelines:

### GitHub Actions Example

```yaml
name: I2C Slave Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test --lib i2c::slave::test_suite::unit

  hil-tests:
    runs-on: self-hosted-with-esp32
    steps:
      - uses: actions/checkout@v2
      - name: Run HIL tests
        run: cargo test --lib i2c::slave::test_suite --features hil-test
```

## Test Coverage

Track test coverage using `tarpaulin`:

```bash
cargo tarpaulin --lib --packages esp-hal -- i2c::slave::test_suite
```

## Benchmarking

Performance tests can be run as benchmarks:

```bash
cargo bench --bench i2c_slave_bench
```

## Test Status

| Category | Tests | Status |
|----------|-------|--------|
| Unit Tests | 53+ | ✅ Implemented |
| Functional Tests | 44+ HIL + 33+ unit | ✅ Implemented |
| Async Tests | 15+ HIL + 8+ unit | ✅ Implemented |
| Performance Tests | 12+ HIL + 3+ unit | ✅ Implemented |
| Reliability Tests | 16+ HIL + 2+ unit | ✅ Implemented |
| Integration Tests | 18+ HIL + 3+ unit | ✅ Implemented |

**Total Tests Implemented:** 207+  
- HIL (Hardware-in-Loop) tests: 105+
- Unit/Documentation tests: 102+

## Known Issues

None currently.

## Contributing

When adding new tests:

1. Choose the appropriate category
2. Follow the existing test structure
3. Add documentation for HIL requirements
4. Update this README with test count
5. Ensure tests are deterministic
6. Add to CI if applicable

## References

- Parent test checklist: `../TESTING.md`
- Driver implementation: `../mod.rs`
- Design documentation: `../DESIGN.md`
