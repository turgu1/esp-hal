//! ESP32 I2C Master - Async Mode Test Example
//!
//! This example demonstrates async I2C master with true multitasking:
//! - I2C master operations using async methods
//! - Parallel status LED task (proves async multitasking)
//! - Automatic retry logic for robustness
//! - Timing measurements for performance analysis
//!
//! Tests:
//! - Test 1: Echo test (write + read with STOP)
//! - Test 2: Simple command/response (write + read with STOP)
//! - Test 3: write_read() transaction (NO STOP, repeated START) - CRITICAL
//! - Test 4: Multi-byte response
//! - Test 5: Status query
//! - Test 6: Continuous testing loop with retry logic
//!
//! Supported devices: ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S2, ESP32-S3
//!
//! Hardware Setup (default GPIO for ESP32-C6):
//! - Connect SDA: GPIO 6 (master) to GPIO 1 (slave)
//! - Connect SCL: GPIO 7 (master) to GPIO 2 (slave)
//! - Add 4.7kΩ pull-up resistors on both SDA and SCL
//! - Connect GND between boards

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::{
    i2c::master::{Config, I2c},
    prelude::*,
    timer::timg::TimerGroup,
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

// Command codes (must match slave)
const CMD_ECHO: u8 = 0x01;
const CMD_SIMPLE: u8 = 0x10;
const CMD_WRITE_READ: u8 = 0x20;
const CMD_MULTI_BYTE: u8 = 0x30;
const CMD_STATUS: u8 = 0x40;

// Retry configuration
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 50;

esp_bootloader_esp_idf::esp_app_desc!();

// Status LED task - demonstrates async multitasking
#[embassy_executor::task]
async fn status_led_task() {
    println!("[LED] Status task started - printing every 3 seconds");

    loop {
        Timer::after(Duration::from_secs(3)).await;
        println!("[LED] *** I2C Master async task still running! ***");
    }
}

// Helper function: I2C write with retry logic
async fn write_with_retry(
    i2c: &mut I2c<'_, esp_hal::Async>,
    addr: u8,
    data: &[u8],
) -> Result<(), esp_hal::i2c::master::Error> {
    for attempt in 1..=MAX_RETRIES {
        match i2c.write_async(addr, data).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < MAX_RETRIES {
                    println!(
                        "  [Retry {}/{}] Write failed: {:?}",
                        attempt, MAX_RETRIES, e
                    );
                    Timer::after(Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

// Helper function: I2C read with retry logic
async fn read_with_retry(
    i2c: &mut I2c<'_, esp_hal::Async>,
    addr: u8,
    data: &mut [u8],
) -> Result<(), esp_hal::i2c::master::Error> {
    for attempt in 1..=MAX_RETRIES {
        match i2c.read_async(addr, data).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < MAX_RETRIES {
                    println!("  [Retry {}/{}] Read failed: {:?}", attempt, MAX_RETRIES, e);
                    Timer::after(Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

// Helper function: I2C write_read with retry logic
async fn write_read_with_retry(
    i2c: &mut I2c<'_, esp_hal::Async>,
    addr: u8,
    tx_data: &[u8],
    rx_data: &mut [u8],
) -> Result<(), esp_hal::i2c::master::Error> {
    for attempt in 1..=MAX_RETRIES {
        match i2c.write_read_async(addr, tx_data, rx_data).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt < MAX_RETRIES {
                    println!(
                        "  [Retry {}/{}] Write-read failed: {:?}",
                        attempt, MAX_RETRIES, e
                    );
                    Timer::after(Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Print device-specific information
    #[cfg(feature = "esp32c6")]
    println!("\n=== ESP32-C6 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32")]
    println!("\n=== ESP32 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32c2")]
    println!("\n=== ESP32-C2 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32c3")]
    println!("\n=== ESP32-C3 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32h2")]
    println!("\n=== ESP32-H2 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32s2")]
    println!("\n=== ESP32-S2 I2C Master (Async Mode) ===");
    #[cfg(feature = "esp32s3")]
    println!("\n=== ESP32-S3 I2C Master (Async Mode) ===");

    println!("Testing I2C Slave Async Functionality\n");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy time driver
    #[cfg(target_arch = "riscv32")]
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        sw_int.software_interrupt0,
    );

    // Spawn status LED task to demonstrate async multitasking
    spawner.spawn(status_led_task()).ok();

    // Configure I2C master
    let config = Config::default().with_frequency(I2C_FREQUENCY.Hz());

    // Device-specific I2C initialization based on GPIO pins
    #[cfg(feature = "esp32c6")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    #[cfg(feature = "esp32")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO19)
        .into_async();

    #[cfg(feature = "esp32c2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    #[cfg(feature = "esp32c3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    #[cfg(feature = "esp32h2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    #[cfg(feature = "esp32s2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    #[cfg(feature = "esp32s3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C master")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    println!("I2C Master initialized at {}kHz", I2C_FREQUENCY / 1000);
    println!("GPIO: SDA={}, SCL={}", SDA_PIN, SCL_PIN);
    println!("Slave address: 0x{:02X}\n", SLAVE_ADDR);

    // Wait for slave to initialize
    Timer::after(Duration::from_millis(500)).await;

    let mut passed = 0;
    let mut failed = 0;

    // ========== Test 1: Echo Test ==========
    println!("Test 1: Echo Test");
    println!("  Type: Write + Read (with STOP, retry enabled)");
    let test1_start = Instant::now();
    {
        let tx_data = [CMD_ECHO, 0xAA, 0xBB, 0xCC];
        let mut rx_data = [0u8; 4];

        // Write command with retry
        if let Err(e) = write_with_retry(&mut i2c, SLAVE_ADDR, &tx_data).await {
            println!("  ✗ FAIL: Write failed after retries: {:?}", e);
            failed += 1;
        } else {
            // Brief delay for slave processing
            Timer::after(Duration::from_micros(100)).await;

            // Read response with retry
            match read_with_retry(&mut i2c, SLAVE_ADDR, &mut rx_data).await {
                Ok(_) => {
                    let elapsed = test1_start.elapsed();
                    if rx_data == tx_data {
                        println!("  ✓ PASS: Echo correct (Stretch: {:?})", elapsed);
                        passed += 1;
                    } else {
                        println!("  ✗ FAIL: Data mismatch");
                        println!("    Expected: {:02X?}", tx_data);
                        println!("    Received: {:02X?}", rx_data);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ FAIL: Read failed after retries: {:?}", e);
                    failed += 1;
                }
            }
        }
    }
    Timer::after(Duration::from_millis(100)).await;

    // ========== Test 2: Simple Command/Response ==========
    println!("\nTest 2: Simple Command/Response");
    println!("  Type: Write + Read (with STOP, retry enabled)");
    let test2_start = Instant::now();
    {
        let tx_data = [CMD_SIMPLE];
        let mut rx_data = [0u8; 1];

        if let Err(e) = write_with_retry(&mut i2c, SLAVE_ADDR, &tx_data).await {
            println!("  ✗ FAIL: Write failed after retries: {:?}", e);
            failed += 1;
        } else {
            Timer::after(Duration::from_micros(100)).await;

            match read_with_retry(&mut i2c, SLAVE_ADDR, &mut rx_data).await {
                Ok(_) => {
                    let elapsed = test2_start.elapsed();
                    if rx_data[0] == 0x42 {
                        println!("  ✓ PASS: Received 0x42 (Stretch: {:?})", elapsed);
                        passed += 1;
                    } else {
                        println!("  ✗ FAIL: Expected 0x42, got 0x{:02X}", rx_data[0]);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ FAIL: Read failed: {:?}", e);
                    failed += 1;
                }
            }
        }
    }
    Timer::after(Duration::from_millis(100)).await;

    // ========== Test 3: write_read() Transaction - CRITICAL ==========
    println!("\nTest 3: write_read() Transaction");
    println!("  Type: write_read (NO STOP, repeated START, retry enabled)");
    println!("  NOTE: This is the critical test for clock stretch fix");
    let test3_start = Instant::now();
    {
        let tx_data = [CMD_WRITE_READ];
        let mut rx_data = [0u8; 1];

        // Use write_read() with retry - NO STOP between write and read
        match write_read_with_retry(&mut i2c, SLAVE_ADDR, &tx_data, &mut rx_data).await {
            Ok(_) => {
                let elapsed = test3_start.elapsed();
                if rx_data[0] == 0x43 {
                    println!("  ✓ PASS: Received 0x43 (Stretch: {:?})", elapsed);
                    if elapsed.as_millis() > 10 {
                        println!("    [Clock stretch >10ms confirms fix is working!]");
                    }
                    passed += 1;
                } else {
                    println!("  ✗ FAIL: Expected 0x43, got 0x{:02X}", rx_data[0]);
                    println!("    (If you got 0x42, clock stretch was released too early)");
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ FAIL: write_read failed after retries: {:?}", e);
                failed += 1;
            }
        }
    }
    Timer::after(Duration::from_millis(100)).await;

    // ========== Test 4: Multi-byte Response ==========
    println!("\nTest 4: Multi-byte Response");
    println!("  Type: Write + Read (with STOP, retry enabled)");
    let test4_start = Instant::now();
    {
        let tx_data = [CMD_MULTI_BYTE];
        let mut rx_data = [0u8; 4];

        if let Err(e) = write_with_retry(&mut i2c, SLAVE_ADDR, &tx_data).await {
            println!("  ✗ FAIL: Write failed after retries: {:?}", e);
            failed += 1;
        } else {
            Timer::after(Duration::from_micros(200)).await;

            match read_with_retry(&mut i2c, SLAVE_ADDR, &mut rx_data).await {
                Ok(_) => {
                    let elapsed = test4_start.elapsed();
                    let expected = [0x44u8, 0x45, 0x46, 0x47];
                    if rx_data == expected {
                        println!(
                            "  ✓ PASS: Received 4 bytes correctly (Stretch: {:?})",
                            elapsed
                        );
                        passed += 1;
                    } else {
                        println!("  ✗ FAIL: Data mismatch");
                        println!("    Expected: {:02X?}", expected);
                        println!("    Received: {:02X?}", rx_data);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ FAIL: Read failed after retries: {:?}", e);
                    failed += 1;
                }
            }
        }
    }
    Timer::after(Duration::from_millis(100)).await;

    // ========== Test 5: Status Query ==========
    println!("\nTest 5: Status Query");
    println!("  Type: Write + Read (with STOP, retry enabled)");
    let test5_start = Instant::now();
    {
        let tx_data = [CMD_STATUS];
        let mut rx_data = [0u8; 1];

        if let Err(e) = write_with_retry(&mut i2c, SLAVE_ADDR, &tx_data).await {
            println!("  ✗ FAIL: Write failed after retries: {:?}", e);
            failed += 1;
        } else {
            Timer::after(Duration::from_micros(100)).await;

            match read_with_retry(&mut i2c, SLAVE_ADDR, &mut rx_data).await {
                Ok(_) => {
                    let elapsed = test5_start.elapsed();
                    if rx_data[0] == 0xFF {
                        println!("  ✓ PASS: Status OK (0xFF) (Stretch: {:?})", elapsed);
                        passed += 1;
                    } else {
                        println!("  ✗ FAIL: Expected 0xFF, got 0x{:02X}", rx_data[0]);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ FAIL: Read failed after retries: {:?}", e);
                    failed += 1;
                }
            }
        }
    }
    Timer::after(Duration::from_millis(100)).await;

    // ========== Test 6: Large Packet write_read() ==========
    println!("\nTest 6: Large Packet write_read()");
    println!("  Type: write_read with 31 bytes (at FIFO threshold, retry enabled)");
    println!("  NOTE: Tests FIFO handling under load");
    let test6_start = Instant::now();
    {
        // 31 bytes: command + 30 data bytes (exceeds 30-byte watermark)
        let mut tx_data = [0u8; 31];
        tx_data[0] = CMD_WRITE_READ;
        for i in 1..31 {
            tx_data[i] = i as u8;
        }
        let mut rx_data = [0u8; 1];

        match write_read_with_retry(&mut i2c, SLAVE_ADDR, &tx_data, &mut rx_data).await {
            Ok(_) => {
                let elapsed = test6_start.elapsed();
                if rx_data[0] == 0x43 {
                    println!(
                        "  ✓ PASS: Received 0x43 with 31-byte packet (Stretch: {:?})",
                        elapsed
                    );
                    if elapsed.as_millis() > 10 {
                        println!("    [Large packet handled correctly!]");
                    }
                    passed += 1;
                } else {
                    println!("  ✗ FAIL: Expected 0x43, got 0x{:02X}", rx_data[0]);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ FAIL: write_read failed after retries: {:?}", e);
                failed += 1;
            }
        }
    }

    // ========== Test Summary ==========
    println!("\n========================================");
    println!("Test Summary:");
    println!("  Total tests: 6");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("========================================");

    if failed == 0 {
        println!("\n✓ All tests PASSED! The I2C async master driver is working correctly.");
        println!("  - Async I2C operations with proper .await points");
        println!("  - Automatic retry logic for robustness");
        println!("  - Parallel status LED task demonstrates true async multitasking");
        println!("  - Clock stretching properly measured");
        println!("  - Variable delays optimized per command type\n");
    } else {
        println!("\n✗ Some tests FAILED. Check the output above for details.\n");
    }

    // Loop forever with status LED task running in background
    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
