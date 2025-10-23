# ESP32 I2C Slave/Master Blocking Test Suite

Complete test suite for validating ESP32 I2C slave blocking functionality with clock stretching and write_read() transaction support.

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
blocking-test/
├── slave/              # I2C Slave blocking implementation
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
./build.sh
./flash.sh
```

#### 2. Build and Flash Master (in separate terminal)

```bash
cd master
chmod +x build.sh flash.sh
./build.sh
./flash.sh
```

### Using Other Devices

Replace `esp32c6` with your device (e.g., `esp32s3`):

```bash
# Slave
cd slave
./build.sh esp32s3
./flash.sh esp32s3

# Master (separate terminal)
cd master
./build.sh esp32s3
./flash.sh esp32s3
```

## 🧪 Test Suite Overview

The master runs 6 comprehensive tests:

### Test 1: Simple Write with Echo
- **Type**: Write + Read (with STOP)
- **Data**: [0x01, 0xAA, 0xBB, 0xCC]
- **Purpose**: Validate basic communication and echo

### Test 2: Simple Read
- **Type**: Write + Read (with STOP)
- **Command**: 0x10
- **Response**: 0x42
- **Purpose**: Validate command/response protocol

### Test 3: write_read() Single Byte ⚠️ CRITICAL
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x20
- **Response**: 0x43
- **Purpose**: Validate write_read() and clock stretch
- **Note**: This is the critical test validating the driver fix

### Test 4: write_read() Multi-Byte
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x30
- **Response**: 16 bytes (sequential 0x00-0x0F)
- **Purpose**: Validate multi-byte responses

### Test 5: write_read() Maximum FIFO
- **Type**: write_read (NO STOP, repeated START)
- **Command**: 0x40
- **Response**: 31 bytes (sequential 0x00-0x1E)
- **Purpose**: Validate FIFO handling at capacity

### Test 6: write_read() Large Write
- **Type**: write_read (NO STOP, repeated START)
- **Data**: 31 bytes (command + 30 data bytes)
- **Response**: 0x43
- **Purpose**: Validate large write + small read

## ✅ Success Criteria

All tests should **PASS** with the following characteristics:

- ✓ **Test 1**: Echo matches sent data
- ✓ **Test 2**: Receives 0x42
- ✓ **Test 3**: Receives 0x43 (confirms fix)
- ✓ **Test 4**: Receives correct sequential pattern
- ✓ **Test 5**: All 31 bytes received correctly
- ✓ **Test 6**: Receives 0x43 with large write
- ✓ **No timeouts or errors**

## 🔍 What This Tests

### Blocking I/O
- Standard synchronous read/write operations
- No async/await overhead
- Simple request/response protocols

### Clock Stretch Management
- Validates that clock stretch holds SCL during processing
- Confirms proper timing in blocking mode
- Tests transaction completion

### write_read() Transactions
- Tests the critical case: NO STOP between write and read
- Master immediately enters read phase after write
- Slave must be ready with response

### TX FIFO Auto-Clear
- Validates `clear_tx_on_write = true` functionality
- Ensures no stale response data from previous transactions
- Critical for request/response protocols

### FIFO Threshold Handling
- Tests 5 and 6 validate FIFO at capacity (31 bytes)
- Confirms hardware interrupt handling
- Validates overflow prevention

## 🐛 Troubleshooting

### All Tests Fail
**Problem**: Master cannot communicate with slave  
**Solutions**:
- Verify hardware connections
- Check pull-up resistors (4.7kΩ)
- Ensure slave is running first
- Check serial monitor for error messages

### Test 3 Fails with Wrong Data (0x42 instead of 0x43)
**Problem**: write_read() not distinguishing from separate write+read  
**This indicates the driver is not handling write_read correctly**  
**Solutions**:
- Verify slave command handling logic
- Check that write_read uses repeated START (no STOP)
- Review driver implementation
- **Note**: This fix is specifically validated on ESP32-C6

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
- Install target: `rustup target add riscv32imac-unknown-none-elf`
- For Xtensa: Install espup and run `espup install`
- Check esp-hal path in Cargo.toml is correct

### Timeout Errors
**Problem**: Slave not responding within timeout  
**Solutions**:
- Increase timeout in slave config (default 2000ms)
- Check clock speed (100kHz standard)
- Verify pull-ups are present

## 📊 Differences from Async Tests

| Feature | Blocking Mode | Async Mode |
|---------|--------------|------------|
| Executor | Not required | Requires embassy-executor |
| await | Not used | Required for operations |
| Complexity | Simpler | More complex setup |
| Concurrency | Sequential only | Can run concurrent tasks |
| Use Case | Simple protocols | Complex applications |
| Performance | Blocks thread | Non-blocking |

## 🔧 Advanced Usage

### Custom I2C Speed

Edit `master/src/main.rs`:
```rust
const I2C_FREQUENCY: u32 = 400_000; // 400kHz fast mode
```

### Custom Slave Address

Edit both files:
```rust
const SLAVE_ADDR: u8 = 0x42; // Custom address
```

### Custom GPIO Pins

Modify the device-specific constants in both `main.rs` files:
```rust
#[cfg(feature = "esp32c6")]
const SDA_PIN: u8 = 5; // Custom SDA pin
#[cfg(feature = "esp32c6")]
const SCL_PIN: u8 = 6; // Custom SCL pin
```

## 📝 Notes

- **Blocking I/O**: All operations block until complete
- **No Async Runtime**: Simpler than async examples, no executor needed
- **Sequential Execution**: Tests run one after another
- **Clock Stretching**: Slave holds SCL while processing
- **Auto FIFO Clear**: Prevents stale data between transactions
- **Timeout Protection**: 2-second timeout prevents hangs

## 🚦 Testing Status

- ✅ **ESP32-C6**: Reference implementation, fully tested
- ⚠️ **Other devices**: Code structure in place, requires hardware validation

## 📚 Related Documentation

- See `../async-test/` for async/await examples
- See `../../test-suite/README.md` for comprehensive test documentation
- See esp-hal documentation for I2C slave API reference

## 💡 Tips

1. **Always start slave first** - Master will timeout if slave not ready
2. **Check serial monitors** - Both devices provide detailed output
3. **Verify pull-ups** - Most common cause of communication failures
4. **Use logic analyzer** - For debugging timing issues
5. **Test at 100kHz first** - Before trying faster speeds
