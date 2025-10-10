#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{delay::Delay, main};
use esp_hal::zigbee::{Zigbee, Config, ZigbeeEvent};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::heap_allocator!(size: 24 * 1024);

    let delay = Delay::new();

    println!("Starting Zigbee End Device...");

    // Create end device configuration (non-sleepy)
    let config = Config::end_device(false)
        .with_channel(15)
        .with_tx_power(10)
        .with_security(true);

    // Create Zigbee instance
    let mut zigbee = Zigbee::new(peripherals.IEEE802154, config);

    // Join network
    println!("Joining network...");
    match zigbee.join_network() {
        Ok(_) => println!("Joined network successfully!"),
        Err(e) => {
            println!("Failed to join network: {:?}", e);
            println!("Retrying...");
            delay.delay_millis(5000);
            loop {}
        }
    }

    // Get network information
    if let Some(info) = zigbee.network_info() {
        println!("Network Info:");
        println!("  PAN ID: 0x{:04X}", info.pan_id);
        println!("  Extended PAN ID: 0x{:016X}", info.extended_pan_id);
        println!("  Channel: {}", info.channel);
        println!("  My Address: 0x{:04X}", info.network_address);
        if let Some(parent) = info.parent_address {
            println!("  Parent: 0x{:04X}", parent);
        }
    }

    let mut message_count = 0u32;
    let mut last_send = 0u32;

    loop {
        // Poll for events
        if let Some(event) = zigbee.poll() {
            match event {
                ZigbeeEvent::NetworkJoined { network_address, parent_address } => {
                    println!("✓ Joined network:");
                    println!("  My Address: 0x{:04X}", network_address);
                    println!("  Parent: 0x{:04X}", parent_address);
                }

                ZigbeeEvent::DataReceived { source, endpoint, cluster, data } => {
                    println!("📨 Data received:");
                    println!("  From: 0x{:04X}", source);
                    println!("  Endpoint: {}", endpoint);
                    println!("  Cluster: 0x{:04X}", cluster);
                    println!("  Data: {:?}", data.as_slice());
                    
                    // Echo back
                    if let Err(e) = zigbee.send_data(source, data.as_slice()) {
                        println!("  Failed to echo: {:?}", e);
                    } else {
                        println!("  Echoed back");
                    }
                }

                ZigbeeEvent::ZclCommand { source, endpoint, cluster, command, data } => {
                    println!("🔧 ZCL Command:");
                    println!("  From: 0x{:04X}", source);
                    println!("  Endpoint: {}", endpoint);
                    println!("  Cluster: 0x{:04X}", cluster);
                    println!("  Command: 0x{:02X}", command);
                }

                ZigbeeEvent::NetworkError(e) => {
                    println!("❌ Network error: {:?}", e);
                    println!("  Attempting to rejoin...");
                    let _ = zigbee.join_network();
                }

                _ => {}
            }
        }

        // Send periodic message to coordinator every 10 seconds
        let now = delay.now().ticks();
        if now - last_send > 10_000_000 {
            last_send = now;
            message_count += 1;

            let message = format!("Hello from end device! Message #{}", message_count);
            
            println!("Sending message to coordinator...");
            match zigbee.send_data(0x0000, message.as_bytes()) {
                Ok(_) => println!("  ✓ Message sent"),
                Err(e) => println!("  ✗ Failed to send: {:?}", e),
            }

            // Get link quality
            if let Some(lqi) = zigbee.get_lqi(0x0000) {
                println!("  Link Quality: {}/255", lqi);
            }
            if let Some(rssi) = zigbee.get_rssi(0x0000) {
                println!("  RSSI: {} dBm", rssi);
            }
        }

        delay.delay_millis(100);
    }
}
