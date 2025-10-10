#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{delay::Delay, main};
use esp_hal::zigbee::{Zigbee, Config, Role, ZigbeeEvent};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_alloc::heap_allocator!(size: 32 * 1024);

    let delay = Delay::new();

    println!("Starting Zigbee Coordinator...");

    // Create coordinator configuration
    let config = Config::coordinator()
        .with_channel(15)
        .with_pan_id(0x1234)
        .with_extended_pan_id(0x0123456789ABCDEF)
        .with_tx_power(10)
        .with_security(true)
        .with_max_children(32);

    // Create Zigbee instance
    let mut zigbee = Zigbee::new(peripherals.IEEE802154, config);

    // Form network
    println!("Forming network...");
    match zigbee.form_network() {
        Ok(_) => println!("Network formed successfully!"),
        Err(e) => {
            println!("Failed to form network: {:?}", e);
            loop {}
        }
    }

    // Get network information
    if let Some(info) = zigbee.network_info() {
        println!("PAN ID: 0x{:04X}", info.pan_id);
        println!("Extended PAN ID: 0x{:016X}", info.extended_pan_id);
        println!("Channel: {}", info.channel);
        println!("Network Address: 0x{:04X}", info.network_address);
    }

    // Permit devices to join for 60 seconds
    println!("Permitting join for 60 seconds...");
    zigbee.permit_join(60).expect("Failed to permit join");

    let mut last_status = 0u32;

    loop {
        // Poll for events
        if let Some(event) = zigbee.poll() {
            match event {
                ZigbeeEvent::NetworkFormed { pan_id, channel, .. } => {
                    println!("✓ Network formed - PAN: 0x{:04X}, Channel: {}", pan_id, channel);
                }

                ZigbeeEvent::DeviceJoined { network_address, ieee_address } => {
                    println!("✓ Device joined:");
                    println!("  Network Address: 0x{:04X}", network_address);
                    println!("  IEEE Address: 0x{:016X}", ieee_address);
                    
                    // Send welcome message
                    let welcome = b"Welcome to the network!";
                    if let Err(e) = zigbee.send_data(network_address, welcome) {
                        println!("  Failed to send welcome message: {:?}", e);
                    }
                }

                ZigbeeEvent::DeviceLeft { network_address } => {
                    println!("✗ Device left: 0x{:04X}", network_address);
                }

                ZigbeeEvent::DataReceived { source, endpoint, cluster, data } => {
                    println!("📨 Data received:");
                    println!("  From: 0x{:04X}", source);
                    println!("  Endpoint: {}", endpoint);
                    println!("  Cluster: 0x{:04X}", cluster);
                    println!("  Data: {:?}", data.as_slice());
                }

                ZigbeeEvent::ZclCommand { source, endpoint, cluster, command, data } => {
                    println!("🔧 ZCL Command:");
                    println!("  From: 0x{:04X}", source);
                    println!("  Endpoint: {}", endpoint);
                    println!("  Cluster: 0x{:04X}", cluster);
                    println!("  Command: 0x{:02X}", command);
                    println!("  Data: {:?}", data.as_slice());
                }

                ZigbeeEvent::LinkQualityUpdate { address, lqi, rssi } => {
                    println!("📶 Link quality update:");
                    println!("  Device: 0x{:04X}", address);
                    println!("  LQI: {}", lqi);
                    println!("  RSSI: {} dBm", rssi);
                }

                ZigbeeEvent::NetworkError(e) => {
                    println!("❌ Network error: {:?}", e);
                }

                _ => {}
            }
        }

        // Print status every 10 seconds
        let now = delay.now().ticks();
        if now - last_status > 10_000_000 {
            last_status = now;
            
            if let Some(info) = zigbee.network_info() {
                println!("Status:");
                println!("  Network: 0x{:04X}", info.pan_id);
                println!("  Channel: {}", info.channel);
            }
        }

        delay.delay_millis(100);
    }
}
