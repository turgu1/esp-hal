# Quick Reference - Device Support

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

## Target Architecture

| Device | Architecture | Target Triple |
|--------|-------------|---------------|
| ESP32-C6 | RISC-V | `riscv32imac-unknown-none-elf` |
| ESP32 | Xtensa | `xtensa-esp32-none-elf` |
| ESP32-C2 | RISC-V | `riscv32imac-unknown-none-elf` |
| ESP32-C3 | RISC-V | `riscv32imac-unknown-none-elf` |
| ESP32-H2 | RISC-V | `riscv32imac-unknown-none-elf` |
| ESP32-S2 | Xtensa | `xtensa-esp32-none-elf` |
| ESP32-S3 | Xtensa | `xtensa-esp32-none-elf` |

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

### Other Devices (C2/C3/H2/S2/S3) Setup
```
Master Board          Slave Board
GPIO 6 (SDA) -------- GPIO 1 (SDA)
GPIO 7 (SCL) -------- GPIO 2 (SCL)
GND ----------------- GND

Pull-ups: 4.7kΩ on SDA and SCL to 3.3V
```

## Complete Workflow Example

### For ESP32-C6 (Default)
```bash
# Terminal 1 - Slave
cd test-examples/async-test/slave
./build.sh
./flash.sh

# Terminal 2 - Master
cd test-examples/async-test/master
./build.sh
./flash.sh
```

### For ESP32-S3
```bash
# Terminal 1 - Slave
cd test-examples/async-test/slave
./build.sh esp32s3
./flash.sh esp32s3

# Terminal 2 - Master
cd test-examples/async-test/master
./build.sh esp32s3
./flash.sh esp32s3
```

## Cargo Feature Flags

All feature flags are defined in `Cargo.toml`:

```toml
[features]
default = ["esp32c6"]
esp32 = ["esp-hal/esp32", "esp-backtrace/esp32", ...]
esp32c2 = ["esp-hal/esp32c2", "esp-backtrace/esp32c2", ...]
esp32c3 = ["esp-hal/esp32c3", "esp-backtrace/esp32c3", ...]
esp32c6 = ["esp-hal/esp32c6", "esp-backtrace/esp32c6", ...]
esp32h2 = ["esp-hal/esp32h2", "esp-backtrace/esp32h2", ...]
esp32s2 = ["esp-hal/esp32s2", "esp-backtrace/esp32s2", ...]
esp32s3 = ["esp-hal/esp32s3", "esp-backtrace/esp32s3", ...]
```

## Troubleshooting

### Wrong device selected
**Error**: Build fails or wrong target used
**Solution**: Ensure you pass the correct device name to build.sh/flash.sh

### GPIO conflicts
**Error**: Hardware not responding
**Solution**: Verify you're using the correct GPIO pins for your device (see table above)

### Target not installed
**Error**: `error: toolchain '...' is not installed`
**Solution**: 
```bash
# For RISC-V targets
rustup target add riscv32imac-unknown-none-elf

# For Xtensa targets (requires espup)
espup install
```

## Device-Specific Notes

### ESP32-C6
- ✅ Fully tested with all 6 tests passing
- Clock stretch explicitly configured
- Default device for the test suite

### ESP32 Classic
- ⚠️ Different GPIO pins (21/22 for slave, 18/19 for master)
- Xtensa architecture requires espup installation

### Other Devices
- ⚠️ Code structure in place but not hardware-tested
- Should work with same GPIO configuration as ESP32-C6
- May require device-specific tuning
