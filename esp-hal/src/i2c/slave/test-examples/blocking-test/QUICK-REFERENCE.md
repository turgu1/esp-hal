# Quick Reference - Blocking Test Suite

## Build Commands

### ESP32-C6 (Default)
```bash
./build.sh
# or explicitly:
./build.sh esp32c6
```

### ESP32 Classic
```bash
./build.sh esp32
```

### ESP32-C2
```bash
./build.sh esp32c2
```

### ESP32-C3
```bash
./build.sh esp32c3
```

### ESP32-H2
```bash
./build.sh esp32h2
```

### ESP32-S2
```bash
./build.sh esp32s2
```

### ESP32-S3
```bash
./build.sh esp32s3
```

## GPIO Pin Mapping

| Device | Slave SDA | Slave SCL | Master SDA | Master SCL |
|--------|-----------|-----------|------------|------------|
| ESP32-C6 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |
| ESP32 | GPIO 21 | GPIO 22 | GPIO 18 | GPIO 19 |
| ESP32-C2 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |
| ESP32-C3 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |
| ESP32-H2 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |
| ESP32-S2 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |
| ESP32-S3 | GPIO 1 | GPIO 2 | GPIO 6 | GPIO 7 |

## Hardware Connections

### ESP32-C6 Setup
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ----------------- GND

Pull-ups: 4.7kΩ on SDA and SCL to 3.3V
```

### ESP32 Classic Setup
```
Master Board          Slave Board
GPIO 18 (SDA) ------- GPIO 21 (SDA)
GPIO 19 (SCL) ------- GPIO 22 (SCL)
GND ----------------- GND

Pull-ups: 4.7kΩ on SDA and SCL to 3.3V
```

### Other Devices Setup
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ----------------- GND

Pull-ups: 4.7kΩ on SDA and SCL to 3.3V
```

## Test Suite

### All 6 Tests

| Test | Type | Purpose | Expected Result |
|------|------|---------|-----------------|
| 1 | Write + Read | Echo test | Data matches |
| 2 | Write + Read | Simple command | Receives 0x42 |
| 3 | write_read() | Single byte **[CRITICAL]** | Receives 0x43 |
| 4 | write_read() | Multi-byte (16) | Sequential pattern |
| 5 | write_read() | Max FIFO (31) | All bytes correct |
| 6 | write_read() | Large write (31) | Receives 0x43 |

### Success Criteria
- ✅ All 6 tests PASS
- ✅ No timeouts
- ✅ No communication errors

## Complete Workflow Example

### For ESP32-C6 (Default)
```bash
# Terminal 1 - Slave
cd test-examples/blocking-test/slave
./build.sh
./flash.sh

# Terminal 2 - Master
cd test-examples/blocking-test/master
./build.sh
./flash.sh
```

### For ESP32-S3
```bash
# Terminal 1 - Slave
cd test-examples/blocking-test/slave
./build.sh esp32s3
./flash.sh esp32s3

# Terminal 2 - Master
cd test-examples/blocking-test/master
./build.sh esp32s3
./flash.sh esp32s3
```

## Configuration

### Slave Configuration
```rust
Config::default()
    .with_address(0x55)
    .with_clear_tx_on_write(true)
    .with_timeout_ms(2000)
    .with_clock_stretch_enable(true) // ESP32-C6
```

### Master Configuration
```rust
Config::default()
    .with_frequency(100_000.Hz()) // 100kHz
```

## Command Protocol

| Command | Description | Response |
|---------|-------------|----------|
| 0x01 | Echo | Echoes received data |
| 0x10 | Simple byte | 0x42 |
| 0x20 | write_read test | 0x43 |
| 0x30 | Multi-byte | [0x00..0x0F] (16 bytes) |
| 0x40 | Max FIFO | [0x00..0x1E] (31 bytes) |
| 0x00 | Status | [0x00, 0x12, 0x34, 0x56] |
| Other | Unknown | Echoes data |

## Troubleshooting

### All Tests Fail
- Check hardware connections
- Verify pull-up resistors present
- Ensure slave started first
- Check device matches feature flag

### Test 3 Fails (Gets 0x42 instead of 0x43)
- write_read() not working correctly
- Check driver implementation
- Verify ESP32-C6 specific fix applied

### Build Errors
```bash
# Install RISC-V target
rustup target add riscv32imac-unknown-none-elf

# Install Xtensa support (for ESP32, S2, S3)
espup install
```

### Flash Errors
```bash
# Install cargo-espflash
cargo install cargo-espflash

# Check device connected
cargo espflash board-info
```

## Blocking vs Async

| Feature | Blocking | Async |
|---------|----------|-------|
| Dependencies | 3 crates | 6 crates |
| Executor | None | embassy-executor |
| Syntax | Standard | async/await |
| Complexity | Simple | Complex |
| Concurrency | Sequential | Concurrent tasks |
| Code Size | Smaller | Larger |

## Tips

1. **Always start slave first** - Master will timeout if slave not ready
2. **Check both serial monitors** - Slave and master provide detailed output
3. **Verify pull-ups** - Most common cause of failures
4. **Test at 100kHz first** - Before trying higher speeds
5. **Use logic analyzer** - For debugging timing issues

## Performance Expectations

At 100kHz I2C speed:
- Single byte transaction: ~1-2ms
- 4-byte echo: ~2-3ms
- 16-byte read: ~3-4ms
- 31-byte read: ~5-6ms

Times include clock stretching overhead.

## Expected Output

### Slave
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
```

### Master
```
=== ESP32-C6 I2C Master (Blocking Mode) ===
Testing I2C Slave Blocking Functionality

I2C Master initialized at 100kHz
GPIO: SDA=6, SCL=7
Slave address: 0x55

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

[... all tests ...]

=================================
Test Summary:
  Passed: 6
  Failed: 0
  Total:  6

  🎉 ALL TESTS PASSED! 🎉
=================================
```
