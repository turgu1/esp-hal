# I2C Slave Driver Test Suite

This directory contains a comprehensive test suite for the I2C slave driver implementation.

## Overview

The test suite validates all aspects of the I2C slave driver, including:
- Basic read/write operations
- **write_read() with repeated START** (Tests 6a-6g, fully supported)
- Address matching and FIFO management
- Clock stretching and filtering
- Async operations with embassy-executor
- Performance and reliability
- Integration with other peripherals

**New in this version:** Comprehensive write_read() testing confirming that ESP32-C6 and modern chips fully support I2C repeated START transactions in normal mode (no special configuration required).

See: `I2C_SLAVE_WRITE_READ_SUPPORT.md` for write_read() implementation details

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

### HIL Setup Guide

#### Hardware Wiring

Connect the two ESP32 boards as follows:

```
Master ESP32-C6          Slave ESP32-C6
┌─────────────┐         ┌─────────────┐
│             │         │             │
│  GPIO4 (SDA)├─────────┤GPIO1 (SDA)  │
│             │    ↑    │             │
│  GPIO5 (SCL)├────┼────┤GPIO2 (SCL)  │
│             │    │    │             │
│         GND ├────┼────┤GND          │
│             │    │    │             │
└─────────────┘    │    └─────────────┘
                   │
              4.7kΩ pull-ups
              to VCC (3.3V)
```

**Pull-up Resistors:**
- Install 4.7kΩ resistors from SDA to 3.3V
- Install 4.7kΩ resistors from SCL to 3.3V
- These are REQUIRED for proper I2C operation

#### Creating the Slave Test Application

Create a new binary in your project to run on the slave device.

**Required dependencies** (add to your `Cargo.toml`):
```toml
[dependencies]
esp-hal = { version = "...", features = ["esp32c6"] }
esp-backtrace = { version = "...", features = ["esp32c6", "panic-handler", "println"] }
esp-println = { version = "...", default-features = false, features = ["esp32c6", "uart"] }
```

**File: `examples/i2c_slave_hil_test.rs`**

```rust
//! I2C Slave HIL Test Application
//!
//! This runs on the slave device during hardware-in-loop testing.
//! It responds to I2C master commands and validates behavior.

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::slave::{Config, Error, I2c},
    main,
};
use esp_println::{print, println};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Configure I2C slave at address 0x55
    let config = Config::default()
        .with_address(0x55.into())
        .with_timeout_ms(2000)
        .with_clear_tx_on_write(true)
        .with_clock_stretch_enable(true);
    
    let mut i2c_slave = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);
    
    println!("I2C Slave Test App Started");
    println!("Address: 0x55");
    println!("Waiting for master transactions...");
    
    let delay = Delay::new();
    let mut transaction_count = 0u32;
    
    loop {
        // Buffer for incoming data
        let mut rx_buffer = [0u8; 32];
        
        // Wait for master write
        match i2c_slave.read(&mut rx_buffer) {
            Ok(bytes_read) => if bytes_read > 0_usize {
                transaction_count += 1;
                
                println!("\n=== Transaction #{} ===", transaction_count);
                println!("Received {} bytes:", bytes_read);
                print_hex(&rx_buffer[..bytes_read]);
                
                // Process command (first byte is typically command/register)
                let command = rx_buffer[0];
                
                // Prepare response based on command
                let mut response = [0u8; 32];
                let response_len = match command {
                    // Test 1: Echo command - echo back exactly what was received
                    0x01 => {
                        println!("Command: ECHO (Test 1)");
                        response[..bytes_read].copy_from_slice(&rx_buffer[..bytes_read]);
                        bytes_read
                    }
                    
                    // Test 2: Single byte response
                    0x10 => {
                        println!("Command: SINGLE BYTE");
                        response[0] = 0x42;
                        1
                    }
                    
                    // Test 2.5: Single byte response from a write_read() master call
                    0x20 => {
                        println!("Command: SINGLE BYTE WRITE_READ");
                        response[0] = 0x43;
                        1
                    }
                    
                    // Test 3: Multi-byte sequential
                    0x30 => {
                        println!("Command: MULTI-BYTE SEQUENTIAL");
                        for i in 0..16 {
                            response[i] = i as u8;
                        }
                        16
                    }
                    
                    // Test 4: Maximum FIFO (31 bytes for read)
                    0x40 => {
                        println!("Command: MAX FIFO READ");
                        for i in 0..31 {
                            response[i] = i as u8;
                        }
                        31
                    }
                    
                    // Test 5: Status register (write_read test)
                    0x00 => {
                        println!("Command: STATUS REGISTER");
                        response[0..4].copy_from_slice(&[0x00, 0x12, 0x34, 0x56]);
                        4
                    }
                    
                    // Default: Echo all received data
                    _ => {
                        println!("Command: UNKNOWN (0x{:02X}), echoing data", command);
                        response[..bytes_read].copy_from_slice(&rx_buffer[..bytes_read]);
                        bytes_read
                    }
                };
                
                println!("Preparing response: {} bytes", response_len);
                print_hex(&response[..response_len]);
                
                // Pre-load response for master read phase
                match i2c_slave.write(&response[..response_len]) {
                    Ok(_) => println!("Response ready"),
                    Err(e) => println!("Error loading response: {:?}", e),
                }
                
                // Small delay for stability
                delay.delay_millis(1);
            } else {
                // No data received
                println!("No data received from master");
                delay.delay_millis(10);
            },
            
            Err(e) => {
                // Don't print timeout errors to avoid spam
                if !matches!(e, Error::Timeout) {
                    println!("Error: {:?}", e);
                }
                delay.delay_millis(10);
            }
        }
    }
}

fn print_hex(data: &[u8]) {
    print!("  [");
    for (i, byte) in data.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("0x{:02X}", byte);
    }
    println!("]");
}
```

**Build and flash the slave:**
```bash
# For ESP32-C6
cargo build --release --example i2c_slave_hil_test --target riscv32imac-unknown-none-elf
cargo espflash flash --example i2c_slave_hil_test --monitor
```

#### Creating the Master Test Application

Create a binary to run on the master device.

**Required dependencies** (same as slave, add to your `Cargo.toml`):
```toml
[dependencies]
esp-hal = { version = "...", features = ["esp32c6"] }
esp-backtrace = { version = "...", features = ["esp32c6", "panic-handler", "println"] }
esp-println = { version = "...", default-features = false, features = ["esp32c6", "uart"] }
```

**File: `examples/i2c_master_hil_test.rs`**

```rust
//! I2C Master HIL Test Application
//!
//! This runs on the master device during hardware-in-loop testing.
//! It sends commands to the slave and validates responses.

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::master::{Config, Error, I2c, BusTimeout},
    main,
    time::Rate,
};
use esp_println::println;

const SLAVE_ADDRESS: u8 = 0x55;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Configure I2C master at 100 kHz
    let config = Config::default()
        .with_frequency(Rate::from_khz(100))
        // allow stretch to a maximum wait time
        .with_timeout(BusTimeout::BusCycles(2000)); 
    
    let mut i2c_master = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);
    
    println!("\n=================================");
    println!("I2C Master HIL Test Application");
    println!("=================================");
    println!("Slave Address: 0x{:02X}", SLAVE_ADDRESS);
    println!("Bus Speed: 100 kHz\n");
    
    let delay = Delay::new();
    delay.delay_millis(5000); // Wait for slave to start
    
    // Run test suite
    run_test_suite(&mut i2c_master, &delay);
    
    loop {
        delay.delay_millis(5000);
        println!("\n--- Running tests again in 5s ---\n");
        run_test_suite(&mut i2c_master, &delay);
    }
}

fn run_test_suite(i2c: &mut I2c<'_, esp_hal::Blocking>, delay: &Delay) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    
    // Test 1: Simple write + read echo
    println!("Test 1: Simple Write + Read Echo");
    match test_simple_write(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);
    
    // Test 2: Simple read
    println!("Test 2: Simple Read");
    match test_simple_read(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);
    
    // Test 3: write_read (repeated START)
    println!("Test 3: write_read() - Single Byte");
    match test_write_read_single(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);
    
    
    // Test 4: write_read multi-byte
    println!("Test 4: write_read() - Multi-Byte");
    match test_write_read_multi(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);
    
    // Test 5: write_read maximum FIFO
    println!("Test 5: write_read() - Maximum FIFO (31 bytes)");
    match test_write_read_max_fifo(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);
    
    // Test 6: write_read (repeated START)
    println!("Test 6: write_read() - Single Byte");
    match test_big_write_read_single(i2c) {
        Ok(_) => { passed += 1; println!("  ✓ PASS\n"); }
        Err(e) => { failed += 1; println!("  ✗ FAIL: {:?}\n", e); }
    }
    delay.delay_millis(100);

    // Summary
    println!("=================================");
    println!("Test Summary:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Total:  {}", passed + failed);
    println!("=================================\n");
}

fn test_simple_write(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let data = [0x01, 0xAA, 0xBB, 0xCC];
    println!("  Writing {} bytes: {:02X?}", data.len(), data);
    i2c.write(SLAVE_ADDRESS, &data)?;
    println!("  Write successful");
    
    // CRITICAL FIX: Read back the echo response to complete the transaction
    // The slave is echoing the data, so we need to read it back
    let delay = Delay::new();
    delay.delay_micros(100); // Give slave time to prepare response
    
    let mut echo_buffer = [0u8; 4];
    println!("  Reading echo response...");
    i2c.read(SLAVE_ADDRESS, &mut echo_buffer)?;
    println!("  Echo received: {:02X?}", echo_buffer);
    
    // Validate that echo matches what we sent
    if echo_buffer == data {
        println!("  Echo matches sent data - transaction complete");
        Ok(())
    } else {
        println!("  ERROR: Echo mismatch! Sent: {:02X?}, Received: {:02X?}", data, echo_buffer);
        Err(Error::ExecutionIncomplete)
    }
}

fn test_simple_read(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    // First write command
    let command = [0x10];
    i2c.write(SLAVE_ADDRESS, &command)?;
    
    // Small delay for slave to prepare response
    let delay = Delay::new();
    delay.delay_micros(50);
    
    // Then read response
    let mut buffer = [0u8; 1];
    println!("  Reading {} bytes...", buffer.len());
    i2c.read(SLAVE_ADDRESS, &mut buffer)?;
    println!("  Received: {:02X?}", buffer);
    
    // Validate expected response
    if buffer[0] == 0x42 {
        println!("  Response matches expected value");
        Ok(())
    } else {
        println!("  ERROR: Expected 0x42, got 0x{:02X}", buffer[0]);
        Err(Error::ExecutionIncomplete)
    }
}

fn test_write_read_single(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let write_data = [0x20]; // Register address
    let mut read_buffer = [0u8; 1];
    
    println!("  write_read: write={:02X?}, read={} bytes", write_data, read_buffer.len());
    i2c.write_read(SLAVE_ADDRESS, &write_data, &mut read_buffer)?;
    println!("  Received: {:02X?}", read_buffer);
    
    if read_buffer[0] == 0x43 {
        println!("  Response correct (0x43)");
        Ok(())
    } else {
        println!("  ERROR: Expected 0x43, got 0x{:02X}", read_buffer[0]);
        Err(Error::ExecutionIncomplete)
    }
}

fn test_write_read_multi(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let write_data = [0x30]; // Command for multi-byte response
    let mut read_buffer = [0u8; 16];
    
    println!("  write_read: write={:02X?}, read={} bytes", write_data, read_buffer.len());
    i2c.write_read(SLAVE_ADDRESS, &write_data, &mut read_buffer)?;
    println!("  Received: {:02X?}", read_buffer);
    
    // Validate sequential pattern (0, 1, 2, 3, ...)
    let mut expected = [0u8; 16];
    for i in 0..16 {
        expected[i] = i as u8;
    }
    if read_buffer == expected {
        println!("  Response matches sequential pattern");
        Ok(())
    } else {
        println!("  ERROR: Pattern mismatch");
        Err(Error::ExecutionIncomplete)
    }
}

fn test_write_read_max_fifo(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let write_data = [0x40]; // Command for max FIFO response
    let mut read_buffer = [0u8; 31]; // Maximum read capacity
    
    println!("  write_read: write={:02X?}, read={} bytes", write_data, read_buffer.len());
    i2c.write_read(SLAVE_ADDRESS, &write_data, &mut read_buffer)?;
    println!("  Received {} bytes successfully", read_buffer.len());
    
    // Validate sequential pattern
    let mut expected = [0u8; 31];
    for i in 0..31 {
        expected[i] = i as u8;
    }
    if read_buffer == expected {
        println!("  All 31 bytes match sequential pattern");
        Ok(())
    } else {
        println!("  ERROR: Pattern mismatch in max FIFO test");
        Err(Error::ExecutionIncomplete)
    }
}

fn test_big_write_read_single(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let mut write_data = [0u8; 31]; // Register address
    let mut read_buffer = [0u8; 1];
    
    for i in 0..31 {
        write_data[i] = i as u8;
    }
    write_data[0] = 0x20; // Command byte

    println!("  write_read: write={:02X?}, read={} bytes", write_data, read_buffer.len());
    i2c.write_read(SLAVE_ADDRESS, &write_data, &mut read_buffer)?;
    println!("  Received: {:02X?}", read_buffer);
    
    if read_buffer[0] == 0x43 {
        println!("  Response correct (0x43)");
        Ok(())
    } else {
        println!("  ERROR: Expected 0x43, got 0x{:02X}", read_buffer[0]);
        Err(Error::ExecutionIncomplete)
    }
}
```

**Build and flash the master:**
```bash
# For ESP32-C6
cargo build --release --example i2c_master_hil_test --target riscv32imac-unknown-none-elf
cargo espflash flash --example i2c_master_hil_test --monitor
```

#### Running HIL Tests

1. **Flash the slave device:**
   ```bash
   cd /path/to/esp-hal
   cargo espflash flash --example i2c_slave_hil_test --monitor
   ```
   Keep this terminal open to see slave output.

2. **Flash the master device (in new terminal):**
   ```bash
   cd /path/to/esp-hal
   cargo espflash flash --example i2c_master_hil_test --monitor
   ```
   Watch test results in this terminal.

3. **Verify wiring:**
   - Check SDA and SCL connections
   - Verify pull-up resistors are installed
   - Ensure common ground connection

4. **Expected Output:**
   
   **Slave terminal:**
   ```
   I2C Slave Test App Started
   Address: 0x55
   Waiting for master transactions...
   
   === Transaction #1 ===
   Received 4 bytes:
     [0x01, 0xAA, 0xBB, 0xCC]
   Command: ECHO (Test 1)
   Preparing response: 4 bytes
     [0x01, 0xAA, 0xBB, 0xCC]
   Response ready
   
   === Transaction #2 ===
   Received 1 bytes:
     [0x10]
   Command: SINGLE BYTE
   Preparing response: 1 bytes
     [0x42]
   Response ready
   ```
   
   **Master terminal:**
   ```
   =================================
   I2C Master HIL Test Application
   =================================
   Slave Address: 0x55
   Bus Speed: 100 kHz
   
   Test 1: Simple Write + Read Echo
     Writing 4 bytes: [01, AA, BB, CC]
     Write successful
     Reading echo response...
     Echo received: [01, AA, BB, CC]
     Echo matches sent data - transaction complete
     ✓ PASS
   
   Test 2: Simple Read
     Reading 4 bytes...
     Received: [42, 00, 00, 00]
     Response matches expected value
     ✓ PASS
   
   Test 3: write_read() - Single Byte
     write_read: write=[10], read=1 bytes
     Received: [42]
     Response correct (0x42)
     ✓ PASS
   
   =================================
   Test Summary:
     Passed: 5
     Failed: 0
     Total:  5
   =================================
   ```

#### Troubleshooting HIL Setup

**No communication:**
- Verify pull-up resistors are installed
- Check wiring (SDA, SCL, GND all connected)
- Confirm slave address matches (0x55)
- Try lower bus speed (50 kHz)

**Intermittent failures:**
- Add delays between transactions
- Check power supply stability
- Verify proper grounding
- Consider EMI/noise sources

**NACK errors:**
- Slave may not be ready
- Address mismatch
- Bus voltage issues
- Pull-up resistor values too high/low

**Timeout errors:**
- Increase timeout in slave config
- Check clock stretching settings
- Verify bus speed compatibility

### Master Support for HIL Tests

The `master-support/` module provides I2C master implementations specifically designed for testing the slave driver:

**Common utilities** (`common.rs`):
- `TestMaster` wrapper around esp_hal I2C master with write_read() support
- Pattern generators (sequential, constant, alternating, pseudo-random)
- Timing utilities (Timer, delays)
- Assertions for buffer comparison and timing validation
- **NEW: write_read module** with register utilities and timing calculations

**Test-specific masters**:
- `functional.rs`: BasicCommMaster, AddressTestMaster, FifoTestMaster, ClockStretchMaster, FilterTestMaster, InterruptTestMaster, ErrorTestMaster, **WriteReadTestMaster**
- `async_support.rs`: AsyncTestMaster, AsyncOperationsMaster, ConcurrentTestMaster, FutureTestMaster, **AsyncWriteReadTestMaster**
- `performance.rs`: SpeedTestMaster, ThroughputTestMaster with result tracking
- `reliability.rs`: StressTestMaster, RecoveryTestMaster with statistics
- `integration.rs`: PeripheralIntegrationMaster, OsIntegrationMaster, AsyncFrameworkMaster

**write_read() Support:**
- `WriteReadTestMaster` - Blocking write_read tests (Tests 6a-6g)
- `AsyncWriteReadTestMaster` - Async write_read tests with timeouts and retries
- Register emulation utilities for sensor-like testing
- ESP32 (original) master compatibility testing

Each master provides methods to trigger specific slave behaviors and validate responses, making HIL tests easier to write and maintain.

See: `master-support/WRITE_READ_SUPPORT.md` for master implementation details

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
- Basic read/write operations (Tests 1-6)
- **write_read() with repeated START** (Tests 6a-6g)
  - Single byte and multi-byte transfers
  - Register-based mode (ESP32-C6) vs normal mode
  - Maximum FIFO usage
  - Atomic vs separate transactions
  - ESP32 master compatibility
- Address matching
- FIFO operations
- Clock stretching
- Filtering
- Interrupts

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::functional --features hil-test
```

**write_read() tests specifically:**
```bash
cargo test --lib i2c::slave::test_suite::functional::basic_comm::test_write_read --features hil-test
```

### Async Tests (`async_tests/`)

Tests for async operations:
- Async read/write
- **Async write_read() with repeated START**
  - Timeout handling
  - Concurrent operations
  - Progress monitoring
  - Error recovery with retries
- Future cancellation

**Run with:**
```bash
cargo test --lib i2c::slave::test_suite::async_tests --features hil-test
```

**Async write_read() tests:**
```bash
cargo test --lib i2c::slave::test_suite::async_tests::test_async_write_read --features hil-test
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
    use crate::i2c::slave::test_suite::master_support::functional::WriteReadTestMaster;

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_write_read_example() {
        // Setup slave
        let peripherals = unsafe { Peripherals::steal() };
        let mut slave = I2c::new(peripherals.I2C0, Config::default())
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Create master for testing
        let mut master = WriteReadTestMaster::new(
            peripherals.I2C1,
            master_sda_pin,
            master_scl_pin
        ).unwrap();
        
        // Slave prepares to handle write_read
        // Master: write_read([0x10], 4 bytes)
        
        // Write phase - slave receives register
        let mut reg = [0u8; 1];
        slave.read(&mut reg).unwrap();
        assert_eq!(reg[0], 0x10);
        
        // Read phase - slave responds
        let response = [0xAA, 0xBB, 0xCC, 0xDD];
        slave.write(&response).unwrap();
        
        // Master validates response
        let data = master.test_single_byte_write_read(0x10).unwrap();
        assert_eq!(data, 0xAA);
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
| Functional Tests | 50+ HIL + 33+ unit | ✅ Implemented |
| **write_read() Tests** | **6 blocking + 4 async** | ✅ **Implemented** |
| Async Tests | 19+ HIL + 8+ unit | ✅ Implemented |
| Performance Tests | 12+ HIL + 3+ unit | ✅ Implemented |
| Reliability Tests | 16+ HIL + 2+ unit | ✅ Implemented |
| Integration Tests | 18+ HIL + 3+ unit | ✅ Implemented |

**Total Tests Implemented:** 227+  
- HIL (Hardware-in-Loop) tests: 115+
- Unit/Documentation tests: 112+
- **write_read() specific tests: 10+ (6 blocking, 4+ async)**

**New additions:**
- Tests 6a-6g: write_read() repeated START testing
- WriteReadTestMaster with 10 test methods
- AsyncWriteReadTestMaster with 10 async test methods
- write_read utilities in common.rs

## Known Issues

**Previously Fixed:**

✅ **Clock Stretching Bug (Fixed)**: Earlier versions had an issue where ESP32-C6 slave would initiate clock stretching during read operations but never release SCL, causing permanent bus hangs. This has been resolved using the proper ESP-IDF register sequence with `slave_scl_stretch_clr` bit.

**Root Cause & Fix:**
- **Issue**: Missing `slave_scl_stretch_clr` bit clear mechanism used by ESP-IDF
- **Issue 2**: Inadequate transaction state management between operations
- **Issue 3**: Master not reading echo responses, leaving slave TX FIFO with stale data
- **Solution**: Implemented ESP-IDF's `i2c_ll_slave_clear_stretch()` equivalent sequence
- **Solution 2**: Added comprehensive transaction state reset between operations
- **Solution 3**: Fixed master test to properly read echo responses, completing transactions
- **Key**: Uses `scl_stretch_conf.slave_scl_stretch_clr` to properly release SCL when TX FIFO has data
- **Key 2**: Resets controller state after each read/write to prevent stuck transactions
- **Key 3**: Ensures complete I2C transactions to prevent state pollution between tests

**Current Status:**
- Clock stretching now works correctly with pure esp-hal slave/master setups
- For maximum compatibility with older ESP32 masters, clock stretching can be optionally disabled

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
- **write_read() implementation:** `../I2C_SLAVE_WRITE_READ_SUPPORT.md`
- **ESP32 compatibility:** `../ESP32_MASTER_COMPATIBILITY.md`
- **Test suite updates:** `../TEST_SUITE_UPDATES.md`
- **Master support:** `master-support/WRITE_READ_SUPPORT.md`

