//! ESP32 I2C Slave - Blocking Mode Test Example
//!
//! This example demonstrates blocking I2C slave functionality with:
//! - Command/response protocol
//! - Auto TX FIFO clearing
//! - Clock stretching
//! - write_read() transaction handling
//!
//! Supported devices: ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S2, ESP32-S3
//!
//! Hardware Setup (default GPIO for ESP32-C6):
//! - Connect SDA: GPIO 1 (slave) to GPIO 6 (master)
//! - Connect SCL: GPIO 2 (slave) to GPIO 7 (master)
//! - Add 4.7kΩ pull-up resistors on both SDA and SCL
//! - Connect GND between boards
//!
//! Expected behavior:
//! - Slave responds to various commands from master
//! - Clock stretching holds SCL during processing
//! - Proper TX FIFO management prevents stale data

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::slave::{Config, Error, I2c},
    main,
};
use esp_println::{print, println};

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
const CMD_MAX_FIFO: u8 = 0x40;
const CMD_STATUS: u8 = 0x00;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Print device-specific information
    #[cfg(feature = "esp32c6")]
    println!("ESP32-C6 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32")]
    println!("ESP32 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32c2")]
    println!("ESP32-C2 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32c3")]
    println!("ESP32-C3 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32h2")]
    println!("ESP32-H2 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32s2")]
    println!("ESP32-S2 I2C Slave (Blocking Mode) - Starting...");
    #[cfg(feature = "esp32s3")]
    println!("ESP32-S3 I2C Slave (Blocking Mode) - Starting...");

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Configure I2C slave with:
    // - Auto TX FIFO clearing enabled (prevents stale response data)
    // - Clock stretching enabled (allows time for command processing)
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
    #[cfg(feature = "esp32c6")]
    let mut i2c = I2c::new(peripherals.I2C0, config)
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
    println!("Waiting for master transactions...\n");

    let delay = Delay::new();
    let mut transaction_count = 0u32;

    loop {
        // Buffer for received commands/data
        let mut rx_buffer = [0u8; 32];

        // Wait for master to send data
        match i2c.read(&mut rx_buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    continue; // No data received
                }

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
                    CMD_ECHO => {
                        println!("Command: ECHO (Test 1)");
                        response[..bytes_read].copy_from_slice(&rx_buffer[..bytes_read]);
                        bytes_read
                    }

                    // Test 2: Single byte response (separate write + read)
                    CMD_SIMPLE => {
                        println!("Command: SINGLE BYTE");
                        response[0] = 0x42;
                        1
                    }

                    // Test 3: Single byte response from write_read() master call
                    CMD_WRITE_READ => {
                        println!("Command: SINGLE BYTE WRITE_READ");
                        response[0] = 0x43;
                        1
                    }

                    // Test 4: Multi-byte sequential
                    CMD_MULTI_BYTE => {
                        println!("Command: MULTI-BYTE SEQUENTIAL");
                        for i in 0..16 {
                            response[i] = i as u8;
                        }
                        16
                    }

                    // Test 5: Maximum FIFO (31 bytes for read)
                    CMD_MAX_FIFO => {
                        println!("Command: MAX FIFO READ");
                        for i in 0..31 {
                            response[i] = i as u8;
                        }
                        31
                    }

                    // Test 6: Status register (write_read test)
                    CMD_STATUS => {
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
                match i2c.write(&response[..response_len]) {
                    Ok(_) => println!("Response ready"),
                    Err(e) => println!("Error loading response: {:?}", e),
                }

                // Small delay for stability
                delay.delay_millis(1);
            }

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

// Helper function to print hex data
fn print_hex(data: &[u8]) {
    print!("  [");
    for (i, byte) in data.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{:02X}", byte);
    }
    println!("]");
}
