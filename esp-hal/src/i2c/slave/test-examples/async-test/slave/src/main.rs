//! ESP32 I2C Slave - Embassy Async Integration Example
//!
//! This example demonstrates the RECOMMENDED pattern for integrating I2C slave
//! with async applications:
//!
//! Pattern:
//! - I2C slave runs in dedicated task with blocking operations
//! - Communication via embassy_sync::channel for async integration
//! - Complex processing deferred to separate async task
//! - LED blinker demonstrates true async multitasking
//!
//! Architecture:
//! ```
//! I2C Slave Task (blocking)  -->  Channel  -->  Command Processor Task (async)
//!                                                          |
//!                                                          v
//!                                                    Complex async work
//!                                                    (network, delays, etc)
//! ```
//!
//! This allows:
//! - Fast I2C response (slave responds immediately with basic ack/status)
//! - Complex processing happens asynchronously after I2C transaction completes
//! - Other async tasks run in parallel (LED, network, etc)
//!
//! Supported devices: ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S2, ESP32-S3
//!
//! Hardware Setup (default GPIO for ESP32-C6):
//! - Connect SDA: GPIO 1 (slave) to GPIO 6 (master)
//! - Connect SCL: GPIO 2 (slave) to GPIO 7 (master)
//! - Add 4.7kΩ pull-up resistors on both SDA and SCL
//! - Connect GND between boards

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::{
    i2c::slave::{Config, I2c},
    timer::timg::TimerGroup,
};
use esp_println::println;

const SLAVE_ADDR: u8 = 0x55;

// Device-specific GPIO configuration (For printing purposes)
#[cfg(feature = "esp32c6")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c6")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32")]
const SDA_PIN: u8 = 21;
#[cfg(feature = "esp32")]
const SCL_PIN: u8 = 22;

#[cfg(feature = "esp32c2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c2")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32c3")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c3")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32h2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32h2")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32s2")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32s2")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32s3")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32s3")]
const SCL_PIN: u8 = 2;

// Command codes
const CMD_ECHO: u8 = 0x01;
const CMD_SIMPLE: u8 = 0x10;
const CMD_WRITE_READ: u8 = 0x20;
const CMD_MULTI_BYTE: u8 = 0x30;
const CMD_STATUS: u8 = 0x40;

// Message types for inter-task communication
#[derive(Clone, Copy)]
struct I2cCommand {
    cmd: u8,
    data: [u8; 64],
    len: usize,
}

// Global channel for I2C -> Command Processor communication
static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, I2cCommand, 4> = Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

// Independent async task: LED blinker (demonstrates true async multitasking)
#[embassy_executor::task]
async fn status_led_task() {
    // For this demo, just print periodic status instead of trying to blink LED
    // RGB LEDs require special handling that varies by board
    println!("[LED] Status task started - will print every 2 seconds");

    loop {
        Timer::after(Duration::from_secs(2)).await;
        println!("[LED] *** ASYNC TASK ALIVE *** (this proves async multitasking works!)");
    }
}

// Async task: Command processor (handles complex async work)
#[embassy_executor::task]
async fn command_processor_task() {
    println!("[PROC] Command processor task started");

    loop {
        // Wait for command from I2C slave task (this is async!)
        let cmd = COMMAND_CHANNEL.receive().await;

        println!(
            "[PROC] Processing command 0x{:02X} (can do async work here)",
            cmd.cmd
        );

        // Simulate complex async processing
        // This could be: network requests, database queries, long calculations, etc.
        match cmd.cmd {
            CMD_ECHO => {
                println!("[PROC] Echo - simulating async work...");
                Timer::after(Duration::from_millis(100)).await;
                println!("[PROC] Echo processing complete!");
            }

            CMD_WRITE_READ => {
                println!("[PROC] Write-read - doing complex async analysis...");
                Timer::after(Duration::from_millis(200)).await;
                println!("[PROC] Analysis complete!");
            }

            CMD_MULTI_BYTE => {
                println!("[PROC] Multi-byte - fetching data asynchronously...");
                Timer::after(Duration::from_millis(150)).await;
                println!("[PROC] Data fetch complete!");
            }

            _ => {
                Timer::after(Duration::from_millis(50)).await;
                println!("[PROC] Command processed");
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Print device-specific information
    #[cfg(feature = "esp32c6")]
    println!("ESP32-C6 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32")]
    println!("ESP32 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32c2")]
    println!("ESP32-C2 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32c3")]
    println!("ESP32-C3 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32h2")]
    println!("ESP32-H2 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32s2")]
    println!("ESP32-S2 I2C Slave (Async Mode) - Starting...");
    #[cfg(feature = "esp32s3")]
    println!("ESP32-S3 I2C Slave (Async Mode) - Starting...");

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

    // Configure I2C slave with same settings as blocking test
    // - Auto TX FIFO clearing enabled (prevents stale response data)
    // - Clock stretching enabled (allows time for command processing)
    // - 2 second timeout to handle write_read detection properly
    let mut config = Config::default()
        .with_address(SLAVE_ADDR.into())
        .with_clear_tx_on_write(true)
        .with_timeout_ms(2000);

    // ESP32-C6 requires explicit clock stretch configuration
    #[cfg(feature = "esp32c6")]
    {
        config = config.with_clock_stretch_enable(true);
    }

    // Device-specific I2C initialization based on GPIO pins
    // Note: Using blocking I2C driver, not async mode
    // I2C slave transactions must be handled synchronously (master controls timing)
    // We use async/await only for the event loop between transactions
    #[cfg(feature = "esp32c6")]
    let i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    #[cfg(feature = "esp32c2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32c3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32h2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32s2")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    #[cfg(feature = "esp32s3")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    println!("I2C Slave initialized at address 0x{:02X}", SLAVE_ADDR);
    println!("GPIO: SDA={}, SCL={}", SDA_PIN, SCL_PIN);
    println!("Architecture: I2C Slave Task -> Channel -> Command Processor");
    println!("Waiting for master transactions...\n");

    // Spawn async tasks
    spawner.spawn(status_led_task()).ok();
    spawner.spawn(command_processor_task()).ok();

    // Run I2C slave handler (blocking, but other tasks run in parallel)
    i2c_slave_handler(i2c).await;
}

// I2C Slave handler - runs blocking I2C operations
// Responds immediately to master, defers complex work to command processor
async fn i2c_slave_handler(mut i2c: I2c<'_, esp_hal::Blocking>) {
    let mut rx_buffer = [0u8; 64];
    let mut test_nbr = 0;

    println!("[I2C] Slave handler started\n");

    loop {
        // Block waiting for I2C transaction (up to 2 seconds timeout)
        match i2c.read(&mut rx_buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    continue;
                }

                let command = rx_buffer[0];

                // Prepare immediate response for master (must be fast!)
                let (response_data, response_len, test_name) = match command {
                    CMD_ECHO => {
                        test_nbr = 1;
                        let mut response = [0u8; 64];
                        response[..bytes_read].copy_from_slice(&rx_buffer[..bytes_read]);
                        (response, bytes_read, "Echo")
                    }

                    CMD_SIMPLE => {
                        test_nbr = 2;
                        let mut response = [0u8; 64];
                        response[0] = 0x42;
                        (response, 1, "Simple")
                    }

                    CMD_WRITE_READ => {
                        test_nbr = 3;
                        let mut response = [0u8; 64];
                        response[0] = 0x43;
                        (response, 1, "Write-Read")
                    }

                    CMD_MULTI_BYTE => {
                        test_nbr = 4;
                        let mut response = [0u8; 64];
                        response[0..4].copy_from_slice(&[0x44, 0x45, 0x46, 0x47]);
                        (response, 4, "Multi-byte")
                    }

                    CMD_STATUS => {
                        test_nbr = 5;
                        let mut response = [0u8; 64];
                        response[0] = 0xFF;
                        (response, 1, "Status")
                    }

                    _ => {
                        let mut response = [0u8; 64];
                        response[..bytes_read].copy_from_slice(&rx_buffer[..bytes_read]);
                        (response, bytes_read, "Unknown")
                    }
                };

                println!(
                    "[I2C] Test {} - {}: {} bytes",
                    test_nbr, test_name, bytes_read
                );

                // Respond to master immediately (synchronous, fast!)
                if let Err(e) = i2c.write(&response_data[..response_len]) {
                    println!("[I2C] ERROR: Response write failed: {:?}", e);
                } else {
                    println!("[I2C] Response sent: {} bytes", response_len);

                    // WORKAROUND: Manual stretch release needed in async context
                    // (blocking example doesn't need this - potential driver issue)
                    #[cfg(feature = "esp32c6")]
                    {
                        for _ in 0..700000 {
                            unsafe { core::arch::asm!("nop") };
                        }
                        i2c.release_scl_stretch();
                        println!("[I2C] Clock stretch released");
                    }
                }

                // NOW defer complex processing to async task via channel
                // This happens AFTER I2C transaction is complete
                let cmd_msg = I2cCommand {
                    cmd: command,
                    data: rx_buffer,
                    len: bytes_read,
                };

                // Send to processor (non-blocking - uses try_send)
                match COMMAND_CHANNEL.try_send(cmd_msg) {
                    Ok(_) => println!("[I2C] Command queued for async processing"),
                    Err(_) => println!("[I2C] Warning: Command queue full"),
                }

                // Yield briefly to let other tasks run
                embassy_futures::yield_now().await;
            }

            Err(e) => {
                // Timeout or other error - this is normal when no I2C activity
                // Don't print timeout errors to avoid spam
                if !matches!(e, esp_hal::i2c::slave::Error::Timeout) {
                    println!("ERROR: Read failed: {:?}", e);
                }

                // Yield on timeout to allow LED task to run
                embassy_futures::yield_now().await;
            }
        }
    }
}
