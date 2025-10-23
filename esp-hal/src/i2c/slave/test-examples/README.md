# I2C Slave Test Examples

This folder contains comprehensive test examples for ESP32 I2C slave driver with multi-device support.

## 🎯 Supported Devices

All test suites support the following ESP32 devices:
- **ESP32** (Xtensa)
- **ESP32-C2** (RISC-V)
- **ESP32-C3** (RISC-V)
- **ESP32-C6** (RISC-V) - **Default**
- **ESP32-H2** (RISC-V)
- **ESP32-S2** (Xtensa)
- **ESP32-S3** (Xtensa)

## Test Suites

### async-test/
Complete async test suite with separate slave and master implementations.
- **Location**: `async-test/`
- **Mode**: Async/await with embassy-executor
- **Components**: Slave and Master in separate folders
- **Purpose**: Validate async I2C operations, clock stretching, and write_read() transactions
- **Features**: Multi-device support via Cargo features
- **See**: [async-test/README.md](async-test/README.md) for details

### blocking-test/
Complete blocking test suite with separate slave and master implementations.
- **Location**: `blocking-test/`
- **Mode**: Blocking (synchronous) I/O
- **Components**: Slave and Master in separate folders
- **Purpose**: Validate blocking I2C operations and write_read() transactions
- **Features**: Multi-device support via Cargo features
- **See**: [blocking-test/README.md](blocking-test/README.md) for details

## Overview

These examples demonstrate:
- Async and blocking read/write operations
- write_read() transaction handling (repeated START)
- Clock stretching behavior
- FIFO management under load
- Proper TX FIFO clearing strategies
- Multi-device GPIO configuration
- Conditional compilation for different ESP32 variants

## Quick Start

### For ESP32-C6 (Default)

```bash
# Terminal 1 - Async Slave
cd async-test/slave
./build.sh
./flash.sh

# Terminal 2 - Async Master
cd async-test/master
./build.sh
./flash.sh
```

### For Other Devices (e.g., ESP32-S3)

```bash
# Terminal 1 - Slave
cd async-test/slave
./build.sh esp32s3
./flash.sh esp32s3

# Terminal 2 - Master
cd async-test/master
./build.sh esp32s3
./flash.sh esp32s3
```

## Hardware Setup

### Pin Connections by Device

#### ESP32-C6 (default)
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ----------------- GND
```

#### ESP32 (classic)
```
Master Board          Slave Board
GPIO 18 (SDA) ------- GPIO 21 (SDA)
GPIO 19 (SCL) ------- GPIO 22 (SCL)
GND ----------------- GND
```

#### Other devices (C2/C3/H2/S2/S3)
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ----------------- GND
```

**Important**: Use external pull-up resistors (4.7kΩ) on both SDA and SCL lines for all configurations.

## Test Suite Structure

Both async and blocking test suites include:

### Slave Implementation
- Command/response protocol
- Auto TX FIFO clearing enabled
- Clock stretching enabled (device-specific)
- Supports 6 test commands

### Master Implementation
- Comprehensive test suite with 6 tests
- Tests normal write/read and write_read() transactions
- Validates clock stretching and FIFO management
- Automated PASS/FAIL reporting

## Test Scenarios

### Test 1: Echo Test
- **Type**: Write + Read (with STOP)
- **Master sends**: 4 bytes [0x01, 0xAA, 0xBB, 0xCC]
- **Slave echoes**: Same 4 bytes
- **Expected stretch**: ~8-10ms

### Test 2: Simple Command/Response
- **Type**: Write + Read (with STOP)
- **Master sends**: Command 0x10
- **Slave responds**: 0x42
- **Expected stretch**: ~8-10ms

### Test 3: write_read() Transaction
- **Type**: write_read (NO STOP, repeated START)
- **Master sends**: Command 0x20
- **Slave responds**: 0x43
- **Expected stretch**: >10ms (critical test)

### Test 4: Multi-byte Response
- **Type**: Write + Read (with STOP)
- **Master sends**: Command 0x30
- **Slave responds**: [0x44, 0x45, 0x46, 0x47]
- **Expected stretch**: ~8-10ms

### Test 5: Status Query
- **Type**: Write + Read (with STOP)
- **Master sends**: Command 0x40
- **Slave responds**: 0xFF (status byte)
- **Expected stretch**: ~8-10ms

### Test 6: Large Packet write_read()
- **Type**: write_read (NO STOP, repeated START)
- **Master sends**: Command 0x20 + 30 data bytes (31 total, at FIFO threshold)
- **Slave responds**: 0x43
- **Expected stretch**: >10ms
- **Purpose**: Tests FIFO handling under load

## Running the Tests

### Build the Slave
```bash
cd slave_folder
cargo build --release --target riscv32imac-unknown-none-elf --example async_slave
cargo espflash flash --release --example async_slave --monitor
```

### Build the Master
```bash
cd master_folder
cargo build --release --target riscv32imac-unknown-none-elf --example async_master
cargo espflash flash --release --example async_master --monitor
```

## Expected Output

### Slave Output
```
ESP32-C6 I2C Slave (Async Mode) - Starting...
Waiting for master transactions...
[Command 0x01] Echo request: 4 bytes
[Command 0x10] Simple response: 0x42
[Command 0x20] write_read response: 0x43
[Command 0x30] Multi-byte response: 4 bytes
[Command 0x40] Status response: 0xFF
[Command 0x20] Large packet write_read: 31 bytes -> 0x43
All tests completed successfully!
```

### Master Output
```
ESP32-C6 I2C Master (Async Mode) - Starting...

Test 1: Echo Test... PASS (Stretch: ~8ms)
Test 2: Simple Command... PASS (Stretch: ~8ms)
Test 3: write_read Transaction... PASS (Stretch: ~11ms)
Test 4: Multi-byte Response... PASS (Stretch: ~9ms)
Test 5: Status Query... PASS (Stretch: ~8ms)
Test 6: Large write_read... PASS (Stretch: ~12ms)

All tests PASSED! ✓
```

## Key Features Demonstrated

### 1. Clock Stretch Management
- Slave holds SCL during command processing
- Proper release after TX FIFO loaded
- Stretch duration indicates processing time

### 2. write_read() Handling
- No STOP between write and read phases
- Clock stretch held throughout: read → process → write
- Critical for Tests 3 and 6

### 3. TX FIFO Auto-Clear
- `clear_tx_on_write = true` enabled
- Prevents stale response data
- Essential for request/response protocols

### 4. FIFO Threshold Testing
- Test 6 uses 31 bytes (exceeds 30-byte watermark)
- Validates FIFO overflow prevention
- Confirms hardware interrupt handling

## Troubleshooting

### Problem: Test 3 or 6 fails with wrong data
**Cause**: Clock stretch not held long enough  
**Solution**: Verify clock stretching is enabled in slave config

### Problem: Stretch duration too short (<1ms)
**Cause**: Clock stretch released prematurely  
**Solution**: Check that `read_fifo()` doesn't release stretch

### Problem: Master timeout
**Cause**: Slave not responding or SCL stuck low  
**Solution**: 
- Verify external pull-up resistors
- Check slave initialization sequence
- Monitor SCL/SDA with logic analyzer

### Problem: Incorrect echo in Test 1
**Cause**: TX FIFO not cleared properly  
**Solution**: Ensure `clear_tx_on_write = true` in slave config

## Notes

- These tests specifically validate the ESP32-C6 clock stretch fix
- The >10ms stretch in Tests 3 and 6 confirms the fix is working
- All tests should pass without data corruption
- Logic analyzer recommended for detailed timing analysis

## License

Same as esp-hal project.
