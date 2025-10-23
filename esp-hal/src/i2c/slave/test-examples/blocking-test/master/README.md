# ESP32 I2C Master - Blocking Test

This folder contains the I2C master blocking test suite that validates the slave implementation for multiple ESP32 devices.

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

1. **Simple Write with Echo** - Write 4 bytes, read back echo
2. **Simple Read** - Send command 0x10, receive 0x42
3. **write_read() Single Byte** - CRITICAL test for write_read() with command 0x20, expects 0x43
4. **write_read() Multi-Byte** - Request 16-byte sequential response
5. **write_read() Maximum FIFO** - Request 31-byte response at FIFO limit
6. **write_read() Large Write** - Send 31 bytes, read 1 byte response (0x43)

## Expected Output

```
=== ESP32-C6 I2C Master (Blocking Mode) ===
Testing I2C Slave Blocking Functionality

I2C Master initialized at 100kHz
GPIO: SDA=6, SCL=7
Slave address: 0x55

Waiting for slave to initialize...
=================================
Starting I2C Slave Test Suite
=================================

Test 1: Simple Write with Echo
  Writing 4 bytes: [01, AA, BB, CC]
  Write successful
  Reading echo response...
  Echo received: [01, AA, BB, CC]
  Echo matches sent data
  ✓ PASS

Test 2: Simple Read
  Writing command: [10]
  Reading 1 bytes...
  Received: [42]
  Response matches expected value (0x42)
  ✓ PASS

Test 3: write_read() - Single Byte (CRITICAL)
  write_read: write=[20], read=1 bytes
  Received: [43]
  Response correct (0x43)
  ✓ PASS

Test 4: write_read() - Multi-Byte (16 bytes)
  write_read: write=[30], read=16 bytes
  Received first 8 bytes: [00, 01, 02, 03, 04, 05, 06, 07]
  Response matches sequential pattern
  ✓ PASS

Test 5: write_read() - Maximum FIFO (31 bytes)
  write_read: write=[40], read=31 bytes
  Received 31 bytes successfully
  All 31 bytes match sequential pattern
  ✓ PASS

Test 6: write_read() - Large Write (31 bytes) + Single Read
  write_read: write=31 bytes (first 8: [20, 01, 02, 03, 04, 05, 06, 07]...), read=1 bytes
  Received: [43]
  Response correct (0x43)
  ✓ PASS

=================================
Test Summary:
  Passed: 6
  Failed: 0
  Total:  6

  🎉 ALL TESTS PASSED! 🎉
=================================
```

## Test Details

### Test 1: Simple Write with Echo
- **Type**: Separate write + read (with STOP between)
- **Data**: [0x01, 0xAA, 0xBB, 0xCC]
- **Purpose**: Validate basic write and echo functionality

### Test 2: Simple Read
- **Type**: Write command + read response (with STOP between)
- **Command**: 0x10
- **Response**: 0x42
- **Purpose**: Validate command/response protocol

### Test 3: write_read() Single Byte ⚠️ CRITICAL
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x20
- **Response**: 0x43
- **Purpose**: Validate write_read() implementation and clock stretch

### Test 4: write_read() Multi-Byte
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x30
- **Response**: 16 bytes (0x00-0x0F)
- **Purpose**: Validate multi-byte read in write_read()

### Test 5: write_read() Maximum FIFO
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x40
- **Response**: 31 bytes (0x00-0x1E)
- **Purpose**: Validate FIFO handling at maximum capacity

### Test 6: write_read() Large Write
- **Type**: write_read (NO STOP, repeated START)
- **Data**: 31 bytes (command 0x20 + 30 data bytes)
- **Response**: 0x43
- **Purpose**: Validate large write + small read in write_read()

## Success Criteria

All tests should **PASS**:
- ✓ **Test 1**: Echo matches sent data
- ✓ **Test 2**: Receives 0x42
- ✓ **Test 3**: Receives 0x43 (confirms write_read works)
- ✓ **Test 4**: Receives sequential pattern
- ✓ **Test 5**: Receives 31 bytes correctly
- ✓ **Test 6**: Receives 0x43 with large write

## Troubleshooting

### Test 3 Fails (Receives 0x42 instead of 0x43)
This indicates the slave's write_read() handling is not working correctly. The slave should distinguish between:
- Separate write + read (command 0x10 → response 0x42)
- write_read() transaction (command 0x20 → response 0x43)

### All Tests Fail
- Verify hardware connections
- Check pull-up resistors (4.7kΩ required)
- Ensure slave is running first
- Check serial monitor for error messages

### Intermittent Failures
- May indicate timing issues
- Check power supply stability
- Verify I2C bus capacitance is low
