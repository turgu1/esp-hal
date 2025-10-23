# ESP32 I2C Slave - Blocking Test

This folder contains the I2C slave blocking test implementation for multiple ESP32 devices.

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
- **Timeout**: 2000ms
- **GPIO pins**: Device-specific (see Hardware Setup)

## Expected Output

```
ESP32-C6 I2C Slave (Blocking Mode) - Starting...
I2C Slave initialized at address 0x55
GPIO: SDA=1, SCL=2
Waiting for master transactions...

=== Transaction #1 ===
Received 4 bytes:
  [01 AA BB CC]
Command: ECHO (Test 1)
Preparing response: 4 bytes
  [01 AA BB CC]
Response ready

=== Transaction #2 ===
Received 1 bytes:
  [10]
Command: SINGLE BYTE
Preparing response: 1 bytes
  [42]
Response ready

=== Transaction #3 ===
Received 1 bytes:
  [20]
Command: SINGLE BYTE WRITE_READ
Preparing response: 1 bytes
  [43]
Response ready

[... continues for all tests ...]
```

## Command Protocol

The slave implements the following command protocol:

| Command | Description | Response |
|---------|-------------|----------|
| 0x01 | Echo | Echoes received data |
| 0x10 | Simple byte | Returns 0x42 |
| 0x20 | write_read test | Returns 0x43 |
| 0x30 | Multi-byte | Returns 16 sequential bytes (0x00-0x0F) |
| 0x40 | Max FIFO | Returns 31 sequential bytes (0x00-0x1E) |
| 0x00 | Status | Returns [0x00, 0x12, 0x34, 0x56] |
| Other | Unknown | Echoes received data |

## Notes

- Uses blocking I/O (no async/await)
- Suitable for simple request/response protocols
- Clock stretching provides processing time
- Auto TX FIFO clear prevents stale data
