//! ESP32 I2C Master - Blocking Mode Test Example
//!
//! This example tests the I2C slave blocking implementation with various scenarios:
//! - Test 1: Echo test (write + read with STOP)
//! - Test 2: Simple command/response (write + read with STOP)
//! - Test 3: write_read() single byte (NO STOP, repeated START) - CRITICAL
//! - Test 4: write_read() multi-byte (16 bytes)
//! - Test 5: write_read() maximum FIFO (31 bytes)
//! - Test 6: write_read() large write + single read (31 bytes write)
//!
//! Supported devices: ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S2, ESP32-S3
//!
//! Hardware Setup (default GPIO for ESP32-C6):
//! - Connect SDA: GPIO 6 (master) to GPIO 1 (slave)
//! - Connect SCL: GPIO 7 (master) to GPIO 2 (slave)
//! - Add 4.7kΩ pull-up resistors on both SDA and SCL
//! - Connect GND between boards
//!
//! Expected Results:
//! - All tests should PASS
//! - Test 3 validates write_read() with clock stretch
//! - Test 6 validates large write + read transactions

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

const SLAVE_ADDR: u8 = 0x55;
const I2C_FREQUENCY: u32 = 100_000; // 100kHz

// Device-specific GPIO configuration (For printing purposes)
#[cfg(feature = "esp32c6")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32c6")]
const SCL_PIN: u8 = 7;

#[cfg(feature = "esp32")]
const SDA_PIN: u8 = 18;
#[cfg(feature = "esp32")]
const SCL_PIN: u8 = 19;

#[cfg(feature = "esp32c2")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32c2")]
const SCL_PIN: u8 = 7;

#[cfg(feature = "esp32c3")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32c3")]
const SCL_PIN: u8 = 7;

#[cfg(feature = "esp32h2")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32h2")]
const SCL_PIN: u8 = 7;

#[cfg(feature = "esp32s2")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32s2")]
const SCL_PIN: u8 = 7;

#[cfg(feature = "esp32s3")]
const SDA_PIN: u8 = 6;
#[cfg(feature = "esp32s3")]
const SCL_PIN: u8 = 7;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Print device-specific information
    #[cfg(feature = "esp32c6")]
    println!("\n=== ESP32-C6 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32")]
    println!("\n=== ESP32 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32c2")]
    println!("\n=== ESP32-C2 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32c3")]
    println!("\n=== ESP32-C3 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32h2")]
    println!("\n=== ESP32-H2 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32s2")]
    println!("\n=== ESP32-S2 I2C Master (Blocking Mode) ===");
    #[cfg(feature = "esp32s3")]
    println!("\n=== ESP32-S3 I2C Master (Blocking Mode) ===");
    
    println!("Testing I2C Slave Blocking Functionality\n");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure I2C master
    let config = Config::default()
        .with_frequency(Rate::from_khz(I2C_FREQUENCY / 1000))
        // allow stretch to a maximum wait time
        .with_timeout(BusTimeout::BusCycles(2000));

    // Device-specific I2C initialization based on GPIO pins
    #[cfg(feature = "esp32c6")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    #[cfg(feature = "esp32")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO19);

    #[cfg(feature = "esp32c2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    #[cfg(feature = "esp32c3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    #[cfg(feature = "esp32h2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    #[cfg(feature = "esp32s2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    #[cfg(feature = "esp32s3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);

    println!("I2C Master initialized at {}kHz", I2C_FREQUENCY / 1000);
    println!("GPIO: SDA={}, SCL={}", SDA_PIN, SCL_PIN);
    println!("Slave address: 0x{:02X}\n", SLAVE_ADDR);

    // Wait for slave to initialize
    let delay = Delay::new();
    println!("Waiting for slave to initialize...");
    delay.delay_millis(2000);

    loop {
        // Run test suite
        run_test_suite(&mut i2c, &delay);

        println!("\nAll tests complete. Looping...\n");

        delay.delay_millis(5000);
    }
}

fn run_test_suite(i2c: &mut I2c<'_, esp_hal::Blocking>, delay: &Delay) {
    let mut passed = 0;
    let mut failed = 0;

    println!("=================================");
    println!("Starting I2C Slave Test Suite");
    println!("=================================\n");

    // Test 1: Simple write with echo
    println!("Test 1: Simple Write with Echo");
    match test_simple_write(i2c, delay) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);

    /*
    // Test 2: Simple read
    println!("Test 2: Simple Read");
    match test_simple_read(i2c, delay) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);

    // Test 3: write_read (repeated START) - CRITICAL
    println!("Test 3: write_read() - Single Byte (CRITICAL)");
    match test_write_read_single(i2c) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);

    // Test 4: write_read multi-byte
    println!("Test 4: write_read() - Multi-Byte (16 bytes)");
    match test_write_read_multi(i2c) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);

    // Test 5: write_read maximum FIFO
    println!("Test 5: write_read() - Maximum FIFO (31 bytes)");
    match test_write_read_max_fifo(i2c) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);

    // Test 6: write_read large write
    println!("Test 6: write_read() - Large Write (31 bytes) + Single Read");
    match test_big_write_read_single(i2c) {
        Ok(_) => {
            passed += 1;
            println!("  ✓ PASS\n");
        }
        Err(e) => {
            failed += 1;
            println!("  ✗ FAIL: {:?}\n", e);
        }
    }
    delay.delay_millis(100);
*/

    // Summary
    println!("=================================");
    println!("Test Summary:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Total:  {}", passed + failed);
    if failed == 0 {
        println!("\n  🎉 ALL TESTS PASSED! 🎉");
    } else {
        println!("\n  ⚠️  SOME TESTS FAILED");
    }
    println!("=================================\n");
}

fn test_simple_write(i2c: &mut I2c<'_, esp_hal::Blocking>, delay: &Delay) -> Result<(), Error> {
    let data = [0x01, 0xAA, 0xBB, 0xCC];
    println!("  Writing {} bytes: {:02X?}", data.len(), data);
    i2c.write(SLAVE_ADDR, &data)?;
    println!("  Write successful");

    // Give slave time to prepare echo response
    delay.delay_micros(100);

    // Read back the echo response
    let mut echo_buffer = [0u8; 4];
    println!("  Reading echo response...");
    i2c.read(SLAVE_ADDR, &mut echo_buffer)?;
    println!("  Echo received: {:02X?}", echo_buffer);

    // Validate that echo matches what we sent
    if echo_buffer == data {
        println!("  Echo matches sent data");
        Ok(())
    } else {
        println!(
            "  ERROR: Echo mismatch! Sent: {:02X?}, Received: {:02X?}",
            data, echo_buffer
        );
        Err(Error::ExecutionIncomplete)
    }
}

fn test_simple_read(i2c: &mut I2c<'_, esp_hal::Blocking>, delay: &Delay) -> Result<(), Error> {
    // First write command
    let command = [0x10];
    println!("  Writing command: {:02X?}", command);
    i2c.write(SLAVE_ADDR, &command)?;

    // Small delay for slave to prepare response
    delay.delay_micros(50);

    // Then read response
    let mut buffer = [0u8; 1];
    println!("  Reading {} bytes...", buffer.len());
    i2c.read(SLAVE_ADDR, &mut buffer)?;
    println!("  Received: {:02X?}", buffer);

    // Validate expected response
    if buffer[0] == 0x42 {
        println!("  Response matches expected value (0x42)");
        Ok(())
    } else {
        println!("  ERROR: Expected 0x42, got 0x{:02X}", buffer[0]);
        Err(Error::ExecutionIncomplete)
    }
}

fn test_write_read_single(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<(), Error> {
    let write_data = [0x20]; // Command byte
    let mut read_buffer = [0u8; 1];

    println!(
        "  write_read: write={:02X?}, read={} bytes",
        write_data,
        read_buffer.len()
    );
    i2c.write_read(SLAVE_ADDR, &write_data, &mut read_buffer)?;
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

    println!(
        "  write_read: write={:02X?}, read={} bytes",
        write_data,
        read_buffer.len()
    );
    i2c.write_read(SLAVE_ADDR, &write_data, &mut read_buffer)?;
    println!("  Received first 8 bytes: {:02X?}", &read_buffer[..8]);

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

    println!(
        "  write_read: write={:02X?}, read={} bytes",
        write_data,
        read_buffer.len()
    );
    i2c.write_read(SLAVE_ADDR, &write_data, &mut read_buffer)?;
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
    let mut write_data = [0u8; 31];
    let mut read_buffer = [0u8; 1];

    // Fill with sequential data
    for i in 0..31 {
        write_data[i] = i as u8;
    }
    write_data[0] = 0x20; // Command byte

    println!(
        "  write_read: write={} bytes (first 8: {:02X?}...), read={} bytes",
        write_data.len(),
        &write_data[..8],
        read_buffer.len()
    );
    i2c.write_read(SLAVE_ADDR, &write_data, &mut read_buffer)?;
    println!("  Received: {:02X?}", read_buffer);

    if read_buffer[0] == 0x43 {
        println!("  Response correct (0x43)");
        Ok(())
    } else {
        println!("  ERROR: Expected 0x43, got 0x{:02X}", read_buffer[0]);
        Err(Error::ExecutionIncomplete)
    }
}
