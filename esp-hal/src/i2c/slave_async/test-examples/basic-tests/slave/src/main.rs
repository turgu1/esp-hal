//! ESP32 I2C Slave Async - Basic Test Example
//!
//! This example demonstrates the SlaveAsync driver responding to the blocking master test
//! while simultaneously running other async tasks (LED blinker, counter).
//!
//! This proves that the async I2C slave does NOT block other concurrent tasks!
//!
//! Protocol (matching master test):
//! - Command 0x01: Echo test - echoes back received data
//! - Command 0x10: Simple read - responds with 0x42
//! - Command 0x20: write_read single - responds with 0x43
//! - Command 0x30: write_read multi - responds with 16 sequential bytes (0..15)
//! - Command 0x40: write_read max FIFO - responds with 31 sequential bytes (0..30)
//!
//! Supported devices: ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S2, ESP32-S3
//!
//! Hardware Setup (default GPIO for ESP32-C6):
//! - Connect SDA: GPIO 1 (slave) to GPIO 6 (master)
//! - Connect SCL: GPIO 2 (slave) to GPIO 7 (master)
//! - Connect LED: GPIO 8 (for visual concurrent task demonstration)
//! - Add 4.7kΩ pull-up resistors on both SDA and SCL
//! - Connect GND between boards
//!
//! Expected Results:
//! - Master tests should ALL PASS
//! - LED should blink smoothly at 500ms intervals (proving non-blocking)
//! - Counter should increment every second (proving concurrent execution)
//! - Console should show I2C transactions + LED blinks + counter updates interleaved

#![no_std]
#![no_main]

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Level, Output},
    i2c::slave_async::{Config, SlaveAsync},
    main,
};
use esp_println::{println, print};
use static_cell::StaticCell;

const SLAVE_ADDR: u8 = 0x55;

// Device-specific GPIO configuration
#[cfg(feature = "esp32c6")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c6")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32c6")]
const LED_PIN: u8 = 8;

#[cfg(feature = "esp32")]
const SDA_PIN: u8 = 21;
#[cfg(feature = "esp32")]
const SCL_PIN: u8 = 22;
#[cfg(feature = "esp32")]
const LED_PIN: u8 = 2;

#[cfg(feature = "esp32c2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c2")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32c2")]
const LED_PIN: u8 = 8;

#[cfg(feature = "esp32c3")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c3")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32c3")]
const LED_PIN: u8 = 8;

#[cfg(feature = "esp32h2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32h2")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32h2")]
const LED_PIN: u8 = 8;

#[cfg(feature = "esp32s2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32s2")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32s2")]
const LED_PIN: u8 = 18;

#[cfg(feature = "esp32s3")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32s3")]
const SCL_PIN: u8 = 2;
#[cfg(feature = "esp32s3")]
const LED_PIN: u8 = 48;

esp_bootloader_esp_idf::esp_app_desc!();

// Shared state for echo buffer (simple approach)
static ECHO_BUFFER: StaticCell<RefCell<[u8; 32]>> = StaticCell::new();
static ECHO_LEN: StaticCell<RefCell<usize>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Print device-specific information
    #[cfg(feature = "esp32c6")]
    println!("\n=== ESP32-C6 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32")]
    println!("\n=== ESP32 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32c2")]
    println!("\n=== ESP32-C2 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32c3")]
    println!("\n=== ESP32-C3 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32h2")]
    println!("\n=== ESP32-H2 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32s2")]
    println!("\n=== ESP32-S2 I2C Slave Async (Basic Test) ===");
    #[cfg(feature = "esp32s3")]
    println!("\n=== ESP32-S3 I2C Slave Async (Basic Test) ===");

    println!("Async I2C Slave with Concurrent Tasks Demo\n");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize embassy timer
    #[cfg(target_arch = "riscv32")]
    let sw_int =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        sw_int.software_interrupt0,
    );

    // Initialize shared state
    let echo_buffer = ECHO_BUFFER.init(RefCell::new([0u8; 32]));
    let echo_len = ECHO_LEN.init(RefCell::new(0));

    // Configure I2C slave with interrupt-driven async driver
    let config = Config::default()
        .with_address(SLAVE_ADDR.into())
        .with_clock_stretch_enable(true) // Enable for compatibility
        .with_rx_fifo_threshold(8) // Interrupt when 8+ bytes in RX FIFO
        .with_tx_fifo_threshold(24); // Interrupt when <24 bytes in TX FIFO

    // Device-specific I2C initialization
    #[cfg(feature = "esp32c6")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    #[cfg(feature = "esp32c2")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32c3")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32h2")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32s2")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32s3")]
    let i2c = SlaveAsync::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    println!("I2C Slave initialized (Async/Interrupt-Driven)");
    println!("GPIO: SDA={}, SCL={}", SDA_PIN, SCL_PIN);
    println!("Slave address: 0x{:02X}", SLAVE_ADDR);
    println!("LED pin: GPIO {}", LED_PIN);
    println!("\nWaiting for master...\n");

    // Configure LED
    #[cfg(feature = "esp32c6")]
    let led = Output::new(peripherals.GPIO8, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32")]
    let led = Output::new(peripherals.GPIO2, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32c2")]
    let led = Output::new(peripherals.GPIO8, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32c3")]
    let led = Output::new(peripherals.GPIO8, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32h2")]
    let led = Output::new(peripherals.GPIO8, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32s2")]
    let led = Output::new(peripherals.GPIO18, Level::Low, esp_hal::gpio::OutputConfig::default());
    #[cfg(feature = "esp32s3")]
    let led = Output::new(peripherals.GPIO48, Level::Low, esp_hal::gpio::OutputConfig::default());

    // Spawn concurrent tasks
    spawner
        .spawn(i2c_slave_task(i2c, echo_buffer, echo_len))
        .ok();
    spawner.spawn(led_blinker_task(led)).ok();
    spawner.spawn(every_second_task()).ok();

    println!("✓ All tasks spawned successfully");
    println!("✓ System is now fully concurrent - I2C + LED + Counter\n");
}

/// Main I2C slave task - handles master requests asynchronously
#[embassy_executor::task]
async fn i2c_slave_task(
    mut i2c: SlaveAsync<'static>,
    echo_buffer: &'static RefCell<[u8; 32]>,
    echo_len: &'static RefCell<usize>,
) {
    let mut rx_buffer = [0u8; 32];
    let mut command_count = 0u32;

    loop {
        // Wait for master to write command - THIS DOES NOT BLOCK OTHER TASKS!
        match i2c.read_async(&mut rx_buffer).await {
            Ok(len) => {
                command_count += 1;
                println!(
                    "[I2C #{:03}] Received {} bytes: {:02X?}",
                    command_count,
                    len,
                    &rx_buffer[..len]
                );

                // Process command and prepare response immediately
                let command = rx_buffer[0];
                match command {
                    0x01 => {
                        // Test 1: Echo test - send back received data immediately
                        println!(
                            "[I2C #{:03}] Echo test: sending back {} bytes",
                            command_count, len
                        );
                        if let Err(e) = i2c.write_async(&rx_buffer[..len]).await {
                            println!("[I2C #{:03}] Echo write error: {:?}", command_count, e);
                        } else {
                            println!("[I2C #{:03}] Echo sent successfully", command_count);
                        }
                    }

                    0x10 => {
                        // Test 2: Simple read - respond with 0x42
                        println!(
                            "[I2C #{:03}] Simple read command: sending 0x42",
                            command_count
                        );
                        let response = [0x42];
                        if let Err(e) = i2c.write_async(&response).await {
                            println!("[I2C #{:03}] Write error (0x10): {:?}", command_count, e);
                        }
                    }

                    0x20 => {
                        // Test 3: write_read single - respond with 0x43
                        println!(
                            "[I2C #{:03}] write_read single: sending 0x43",
                            command_count
                        );
                        let response = [0x43];
                        if let Err(e) = i2c.write_async(&response).await {
                            println!("[I2C #{:03}] Write error (0x20): {:?}", command_count, e);
                        }
                    }

                    0x30 => {
                        // Test 4: write_read multi - respond with 16 sequential bytes
                        println!(
                            "[I2C #{:03}] write_read multi: sending 16 bytes (0..15)",
                            command_count
                        );
                        let mut response = [0u8; 16];
                        for i in 0..16 {
                            response[i] = i as u8;
                        }
                        if let Err(e) = i2c.write_async(&response).await {
                            println!("[I2C #{:03}] Write error (0x30): {:?}", command_count, e);
                        }
                    }

                    0x40 => {
                        // Test 5: write_read max FIFO - respond with 31 sequential bytes
                        println!(
                            "[I2C #{:03}] write_read max FIFO: sending 31 bytes (0..30)",
                            command_count
                        );
                        let mut response = [0u8; 31];
                        for i in 0..31 {
                            response[i] = i as u8;
                        }
                        if let Err(e) = i2c.write_async(&response).await {
                            println!("[I2C #{:03}] Write error (0x40): {:?}", command_count, e);
                        }
                    }

                    _ => {
                        println!(
                            "[I2C #{:03}] Unknown command: 0x{:02X}",
                            command_count, command
                        );
                    }
                }
            }

            Err(e) => {
                println!("[I2C] Read error: {:?}", e);
                // Small delay before retry to avoid tight error loop
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}

/// LED blinker task - demonstrates that I2C operations don't block this!
#[embassy_executor::task]
async fn led_blinker_task(mut led: Output<'static>) {
    let mut count = 0u16;
    println!("[LED] Blinker task started - 500ms interval");

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
        
        count += 1;
        if count % 10 == 0 {
            print!("💡");
        }
    }
}

/// Counter task - another concurrent task demonstrating true async behavior
#[embassy_executor::task]
async fn every_second_task() {
    let mut count = 0u32;
    println!("[EVERY_SECOND] Task started - 1 second interval\n");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        count += 1;

        // Access interrupt counter using the global function
        #[cfg(feature = "esp32c6")]
        {
            let interrupt_count = esp_hal::i2c::slave_async::get_i2c0_interrupt_count();
            println!(
                "[SEC:{:03}] Time: {}s | I2C Interrupts: {}",
                count, count, interrupt_count
            );
        }

        #[cfg(not(feature = "esp32c6"))]
        {
            println!("[SEC:{:03}] Time: {}s | I2C Interrupts: N/A (not esp32c6)", count, count);
        }
    }
}
