//! I2C Slave Example
//!
//! This example demonstrates how to use the I2C slave driver.
//! 
//! # Hardware Setup
//! 
//! Connect an I2C master device:
//! - SDA: GPIO1
//! - SCL: GPIO2
//! 
//! The slave will respond to address 0x55.
//!
//! # Example Behavior
//!
//! The slave will:
//! 1. Wait for data from the master
//! 2. Echo the received data back to the master
//! 3. Repeat indefinitely

#![no_std]
#![no_main]

use esp_hal::{
    delay::Delay,
    i2c::slave::{Config, I2c},
    prelude::*,
};
use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Create I2C slave with address 0x55
    let config = Config::default()
        .with_address(0x55.into())
        .with_clock_stretch_enable(true);

    let mut i2c_slave = I2c::new(peripherals.I2C0, config)
        .expect("Failed to initialize I2C slave")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    esp_println::println!("I2C Slave initialized at address 0x55");
    esp_println::println!("Waiting for master communication...");

    let mut rx_buffer = [0u8; 64];
    let delay = Delay::new();

    loop {
        // Wait for data from master
        match i2c_slave.read(&mut rx_buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    esp_println::println!("Received {} bytes from master", bytes_read);
                    
                    // Echo the data back to master
                    if let Err(e) = i2c_slave.write(&rx_buffer[..bytes_read]) {
                        esp_println::println!("Error writing response: {:?}", e);
                    } else {
                        esp_println::println!("Sent response back to master");
                    }
                }
            }
            Err(e) => {
                esp_println::println!("Error reading from master: {:?}", e);
            }
        }

        delay.delay_millis(10);
    }
}
