# ESP32 I2C Slave - Async Test

This folder contains the I2C slave async test implementation for multiple ESP32 devices.

## Supported Devices

- ESP32
- ESP32-C2
- ESP32-C3
- ESP32-C6 (default)
- ESP32-H2
- ESP32-S2
- ESP32-S3

## Hardware Setup

### ESP32-C6 (default)
- **SDA**: GPIO 1 (connect to master GPIO 6)
- **SCL**: GPIO 2 (connect to master GPIO 7)

### ESP32
- **SDA**: GPIO 21 (connect to master GPIO 18)
- **SCL**: GPIO 22 (connect to master GPIO 19)

### Other devices (ESP32-C2/C3/H2/S2/S3)
- **SDA**: GPIO 1 (connect to master GPIO 6)
- **SCL**: GPIO 2 (connect to master GPIO 7)

**Common requirements:**
- **Pull-ups**: 4.7kΩ on both SDA and SCL to 3.3V
- **Ground**: Connect GND to master GND

## Building

For ESP32-C6 (default):
```bash
chmod +x build.sh
./build.sh
```

For other devices:
```bash
./build.sh esp32      # For ESP32
./build.sh esp32c3    # For ESP32-C3
./build.sh esp32s3    # For ESP32-S3
```

Or manually:
```bash
# For RISC-V devices (C2, C3, C6, H2)
cargo build --release --features esp32c6 --target riscv32imac-unknown-none-elf

# For Xtensa devices (ESP32, S2, S3)
cargo build --release --features esp32 --target xtensa-esp32-none-elf
```

## Flashing

For ESP32-C6 (default):
```bash
chmod +x flash.sh
./flash.sh
```

For other devices:
```bash
./flash.sh esp32      # For ESP32
./flash.sh esp32c3    # For ESP32-C3
./flash.sh esp32s3    # For ESP32-S3
```

Or manually:
```bash
cargo espflash flash --release --features esp32c6 --target riscv32imac-unknown-none-elf --monitor
```

## Configuration

The slave is configured with:
- **Address**: 0x55
- **Auto TX FIFO clear**: Enabled
- **Clock stretching**: Enabled (ESP32-C6 specific)
- **GPIO pins**: Device-specific (see Hardware Setup)

## Expected Output

```
ESP32-C6 I2C Slave (Async Mode) - Starting...
I2C Slave initialized at address 0x55
Waiting for master transactions...

[Test 1] Echo: 4 bytes received
  ✓ Echoed 4 bytes
[Test 2] Simple command received
  ✓ Sent response: 0x42
[Test 3] write_read command: 1 bytes
  ✓ Sent response: 0x43 (clock stretch: ~10-12ms)
[Test 4] Multi-byte command received
  ✓ Sent 4-byte response
[Test 5] Status query received
  ✓ Sent status: 0xFF
[Test 6] write_read command: 31 bytes
  ✓ Sent response: 0x43 (clock stretch: ~10-12ms)

=== All tests completed! ===
Tests passed: 6/6
```

## Notes

- Start the slave BEFORE starting the master
- Monitor the serial output to see test results
- All tests should pass if the hardware is connected correctly
