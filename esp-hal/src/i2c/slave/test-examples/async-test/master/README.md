# ESP32 I2C Master - Async Test

This folder contains the I2C master async test suite that validates the slave implementation for multiple ESP32 devices.

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
- **SDA**: GPIO 6 (connect to slave GPIO 1)
- **SCL**: GPIO 7 (connect to slave GPIO 2)

### ESP32
- **SDA**: GPIO 18 (connect to slave GPIO 21)
- **SCL**: GPIO 19 (connect to slave GPIO 22)

### Other devices (ESP32-C2/C3/H2/S2/S3)
- **SDA**: GPIO 6 (connect to slave GPIO 1)
- **SCL**: GPIO 7 (connect to slave GPIO 2)

**Common requirements:**
- **Pull-ups**: 4.7kΩ on both SDA and SCL to 3.3V
- **Ground**: Connect GND to slave GND

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

The master is configured with:
- **I2C Speed**: 100kHz
- **Slave Address**: 0x55
- **GPIO pins**: Device-specific (see Hardware Setup)

## Test Suite

The master runs 6 comprehensive tests:

1. **Echo Test** - Write 4 bytes, read back echo
2. **Simple Command** - Send command, receive 0x42
3. **write_read() Transaction** - CRITICAL test for clock stretch fix
4. **Multi-byte Response** - Request 4-byte response
5. **Status Query** - Query slave status (0xFF)
6. **Large Packet write_read()** - 31 bytes at FIFO threshold

## Expected Output

```
=== ESP32-C6 I2C Master (Async Mode) ===
Testing I2C Slave Async Functionality

I2C Master initialized at 100kHz
Slave address: 0x55

Test 1: Echo Test
  Type: Write + Read (with STOP)
  ✓ PASS: Echo correct (Stretch: 8ms)

Test 2: Simple Command/Response
  Type: Write + Read (with STOP)
  ✓ PASS: Received 0x42 (Stretch: 8ms)

Test 3: write_read() Transaction
  Type: write_read (NO STOP, repeated START)
  NOTE: This is the critical test for clock stretch fix
  ✓ PASS: Received 0x43 (Stretch: 11ms)
    [Clock stretch >10ms confirms fix is working!]

Test 4: Multi-byte Response
  Type: Write + Read (with STOP)
  ✓ PASS: Received 4 bytes correctly (Stretch: 9ms)

Test 5: Status Query
  Type: Write + Read (with STOP)
  ✓ PASS: Status OK (0xFF) (Stretch: 8ms)

Test 6: Large Packet write_read()
  Type: write_read with 31 bytes (at FIFO threshold)
  NOTE: Tests FIFO handling under load
  ✓ PASS: Received 0x43 with 31-byte packet (Stretch: 12ms)
    [Large packet handled correctly!]

========================================
Test Summary:
  Total tests: 6
  Passed: 6
  Failed: 0
========================================

✓ All tests PASSED! The I2C slave async driver is working correctly.
  - Clock stretching is properly managed
  - write_read() transactions work correctly
  - TX FIFO auto-clear prevents stale data
  - FIFO threshold handling is robust
```

## Important Notes

- Start the SLAVE first, then the master
- Test 3 and 6 are critical - they validate the clock stretch fix
- Clock stretch >10ms indicates the fix is working correctly
- If Test 3 shows 0x42 instead of 0x43, clock stretch is being released too early

## Troubleshooting

### All tests fail with timeout
- Check hardware connections
- Verify pull-up resistors are installed
- Ensure slave is running and initialized

### Test 3 or 6 fails with wrong data
- Clock stretch issue - check driver implementation
- Verify TX FIFO auto-clear is enabled in slave

### Build fails
- Check Rust toolchain is installed
- Verify esp-hal path in Cargo.toml is correct
