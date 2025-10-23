# ESP32 I2C Slave/Master Async Test Suite

Complete test suite for validating ESP32 I2C slave async functionality with clock stretching and write_read() transaction support.

## 🎯 Supported Devices

All ESP32 variants with I2C slave support:
- **ESP32** (Xtensa)
- **ESP32-C2** (RISC-V)
- **ESP32-C3** (RISC-V)
- **ESP32-C6** (RISC-V) - **Default**
- **ESP32-H2** (RISC-V)
- **ESP32-S2** (Xtensa)
- **ESP32-S3** (Xtensa)

## 📁 Structure

```
async-test/
├── slave/              # I2C Slave async implementation
│   ├── src/
│   │   └── main.rs     # Slave application (multi-device support)
│   ├── Cargo.toml      # Slave dependencies with feature flags
│   ├── build.sh        # Build script (accepts device parameter)
│   ├── flash.sh        # Flash script (accepts device parameter)
│   └── README.md       # Slave documentation
│
├── master/             # I2C Master test suite
│   ├── src/
│   │   └── main.rs     # Master test application (multi-device support)
│   ├── Cargo.toml      # Master dependencies with feature flags
│   ├── build.sh        # Build script (accepts device parameter)
│   ├── flash.sh        # Flash script (accepts device parameter)
│   └── README.md       # Master documentation
│
└── README.md           # This file
```

## 🔌 Hardware Setup

### Required Components
- 2x ESP32 development boards (same model for both)
- 2x 4.7kΩ resistors (for pull-ups)
- Breadboard and jumper wires

### Connections by Device

#### ESP32-C6 (default)
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ---------- GND
```

#### ESP32 (classic)
```
Master Board          Slave Board
GPIO 18 (SDA) ------- GPIO 21 (SDA)
GPIO 19 (SCL) ------- GPIO 22 (SCL)
GND ---------- GND
```

#### Other devices (C2/C3/H2/S2/S3)
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ---------- GND
```

**Pull-ups required for all configurations:**
```
SDA ----[4.7kΩ]---- 3.3V
SCL ----[4.7kΩ]---- 3.3V
```

## 🚀 Quick Start

### Using ESP32-C6 (Default)

#### 1. Build and Flash Slave

```bash
cd slave
chmod +x build.sh flash.sh
./build.sh        # Builds for ESP32-C6 by default
./flash.sh        # Flashes to ESP32-C6
```

### Using Other Devices

#### For ESP32 (classic)
```bash
cd slave
./build.sh esp32
./flash.sh esp32

cd ../master
./build.sh esp32
./flash.sh esp32
```

#### For ESP32-C3
```bash
cd slave
./build.sh esp32c3
./flash.sh esp32c3

cd ../master
./build.sh esp32c3
./flash.sh esp32c3
```

The build scripts automatically select the correct target architecture:
- **RISC-V** (C2, C3, C6, H2): `riscv32imac-unknown-none-elf`
- **Xtensa** (ESP32, S2, S3): `xtensa-esp32-none-elf`

### 2. Build and Flash Master

```bash
cd master
chmod +x build.sh flash.sh
./build.sh
./flash.sh
```

### 3. Monitor Results

Both boards will output test results via serial. The master will show PASS/FAIL for each test.

## 📋 Test Suite

### Test 1: Echo Test
- **Type**: Write + Read (with STOP)
- **Data**: 4 bytes [0x01, 0xAA, 0xBB, 0xCC]
- **Purpose**: Validate basic read/write with echo

### Test 2: Simple Command/Response
- **Type**: Write + Read (with STOP)
- **Command**: 0x10
- **Response**: 0x42
- **Purpose**: Validate command/response protocol

### Test 3: write_read() Transaction ⚠️ CRITICAL
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x20
- **Response**: 0x43
- **Purpose**: Validate clock stretch during write_read
- **Expected stretch**: >10ms

### Test 4: Multi-byte Response
- **Type**: Write + Read (with STOP)
- **Command**: 0x30
- **Response**: [0x44, 0x45, 0x46, 0x47]
- **Purpose**: Validate multi-byte responses

### Test 5: Status Query
- **Type**: Write + Read (with STOP)
- **Command**: 0x40
- **Response**: 0xFF
- **Purpose**: Validate status queries

### Test 6: Large Packet write_read() ⚠️ CRITICAL
- **Type**: write_read (NO STOP, repeated START)
- **Data**: 31 bytes (command + 30 data bytes)
- **Response**: 0x43
- **Purpose**: Validate FIFO handling at threshold (30 bytes)
- **Expected stretch**: >10ms

## ✅ Success Criteria

All tests should **PASS** with the following characteristics:

- ✓ **Tests 1-6**: All receive correct data
- ✓ **Test 3**: Clock stretch >10ms (confirms fix)
- ✓ **Test 6**: Clock stretch >10ms with 31-byte packet
- ✓ **No timeouts or errors**

## 🔍 What This Tests

### Clock Stretch Management
- Validates that clock stretch is held during: read → process → write
- Confirms stretch is NOT released prematurely in `read_fifo()`
- Verifies stretch is released properly after TX FIFO loaded

### write_read() Transactions
- Tests the critical case: NO STOP between write and read
- Master immediately enters read phase after write
- Slave must hold SCL until response ready

### TX FIFO Auto-Clear
- Validates `clear_tx_on_write = true` functionality
- Ensures no stale response data from previous transactions
- Critical for request/response protocols

### FIFO Threshold Handling
- Test 6 uses 31 bytes (exceeds 30-byte watermark)
- Validates FIFO overflow prevention
- Confirms hardware interrupt handling

## 🐛 Troubleshooting

### All Tests Fail
**Problem**: Master cannot communicate with slave  
**Solutions**:
- Verify hardware connections
- Check pull-up resistors (4.7kΩ)
- Ensure slave is running first
- Check serial monitor for error messages

### Test 3 or 6 Fails with Wrong Data (0x42 instead of 0x43)
**Problem**: Clock stretch released too early  
**This indicates the driver fix is NOT applied correctly**  
**Solutions**:
- Verify `read_fifo()` does NOT release clock stretch
- Check that stretch is only released in `write_fifo()`
- Review recent driver changes
- **Note**: This fix is specifically validated on ESP32-C6

### Clock Stretch Duration Too Short (<1ms)
**Problem**: Clock stretch not enabled or released prematurely  
**Solutions**:
- Verify `clock_stretch_enable = true` in slave config (ESP32-C6 specific)
- Check hardware doesn't have bypass capacitors on I2C lines
- Use logic analyzer to inspect SCL behavior
- **Note**: Clock stretch behavior may vary by device

### Wrong GPIO Pins / Hardware Not Working
**Problem**: Using incorrect GPIOs for your device  
**Solutions**:
- Check device-specific GPIO assignments in README
- ESP32 uses different pins (21/22) than ESP32-C6 (1/2)
- Verify your device matches the feature flag used in build

### Build Errors
**Problem**: Dependencies or paths incorrect  
**Solutions**:
- Verify Rust toolchain installed: `rustup show`
- Check esp-hal path in Cargo.toml
- Run `cargo clean` and rebuild

## 📊 Expected Timing

At 100kHz I2C:
- 1 byte: ~90µs (including ACK)
- Test 1 (4 bytes): ~8ms stretch
- Test 2 (1 byte): ~8ms stretch
- Test 3 (1 byte + processing): ~11ms stretch ✓
- Test 4 (4 bytes): ~9ms stretch
- Test 5 (1 byte): ~8ms stretch
- Test 6 (31 bytes + processing): ~12ms stretch ✓

The longer stretch in Tests 3 and 6 confirms the slave is holding clock during the entire command processing cycle.

## 📝 Notes

- These tests specifically validate the ESP32-C6 clock stretch fix
- Tests 3 and 6 are the most critical - they exercise write_read()
- The >10ms stretch is definitive proof the fix works correctly
- Logic analyzer highly recommended for detailed timing analysis
- All tests use embassy-executor for async/await functionality

## 🔗 Related Documentation

- See individual slave/master README.md files for detailed instructions
- Check esp-hal I2C slave driver documentation
- Review CHANGELOG for recent driver updates

## 📄 License

Same as esp-hal project (MIT OR Apache-2.0)
