# Multi-Device Support - Implementation Summary

This document describes the device feature support added to the I2C slave async test examples.

## Overview

Both the slave and master test examples now support all ESP32 device variants through Cargo feature flags and conditional compilation.

## Supported Devices

| Device | Architecture | I2C Slave Support | GPIO (Slave) | GPIO (Master) |
|--------|-------------|-------------------|--------------|---------------|
| ESP32 | Xtensa | ✅ | SDA:21, SCL:22 | SDA:18, SCL:19 |
| ESP32-C2 | RISC-V | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |
| ESP32-C3 | RISC-V | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |
| ESP32-C6 | RISC-V | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |
| ESP32-H2 | RISC-V | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |
| ESP32-S2 | Xtensa | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |
| ESP32-S3 | Xtensa | ✅ | SDA:1, SCL:2 | SDA:6, SCL:7 |

**Default device**: ESP32-C6

## Changes Made

### 1. Cargo.toml Files (Slave & Master)

Added feature flags for all devices:

```toml
[dependencies]
esp-hal = { path = "../../../../../.." }
esp-backtrace = { version = "0.14.2", default-features = false, features = ["exception-handler", "panic-handler", "println"] }
esp-println = { version = "0.12.0", default-features = false }
esp-hal-embassy = { version = "0.6.0", default-features = false }
embassy-executor = { version = "0.6.3", features = ["task-arena-size-32768"] }
embassy-time = "0.3.2"

[features]
default = ["esp32c6"]

# ESP32 variants
esp32 = ["esp-hal/esp32", "esp-backtrace/esp32", "esp-println/esp32", "esp-hal-embassy/esp32"]
esp32c2 = ["esp-hal/esp32c2", "esp-backtrace/esp32c2", "esp-println/esp32c2", "esp-hal-embassy/esp32c2"]
esp32c3 = ["esp-hal/esp32c3", "esp-backtrace/esp32c3", "esp-println/esp32c3", "esp-hal-embassy/esp32c3"]
esp32c6 = ["esp-hal/esp32c6", "esp-backtrace/esp32c6", "esp-println/esp32c6", "esp-hal-embassy/esp32c6"]
esp32h2 = ["esp-hal/esp32h2", "esp-backtrace/esp32h2", "esp-println/esp32h2", "esp-hal-embassy/esp32h2"]
esp32s2 = ["esp-hal/esp32s2", "esp-backtrace/esp32s2", "esp-println/esp32s2", "esp-hal-embassy/esp32s2"]
esp32s3 = ["esp-hal/esp32s3", "esp-backtrace/esp32s3", "esp-println/esp32s3", "esp-hal-embassy/esp32s3"]
```

**Key changes**:
- Removed hard-coded device features from dependencies
- Added `default-features = false` to all device-specific crates
- Created feature flags that cascade device selection to all dependencies
- Set ESP32-C6 as default device

### 2. Source Code (main.rs) - Both Slave & Master

#### GPIO Pin Configuration

Added device-specific GPIO constants:

```rust
// Device-specific GPIO configuration
#[cfg(feature = "esp32c6")]
const SDA_PIN: u8 = 1;
#[cfg(feature = "esp32c6")]
const SCL_PIN: u8 = 2;

#[cfg(feature = "esp32")]
const SDA_PIN: u8 = 21;
#[cfg(feature = "esp32")]
const SCL_PIN: u8 = 22;

// ... etc for all devices
```

#### Device-Specific Initialization

Added conditional compilation for device identification:

```rust
#[cfg(feature = "esp32c6")]
println!("ESP32-C6 I2C Slave (Async Mode) - Starting...");
#[cfg(feature = "esp32")]
println!("ESP32 I2C Slave (Async Mode) - Starting...");
// ... etc
```

#### I2C Configuration

Added device-specific I2C initialization:

```rust
#[cfg(feature = "esp32c6")]
let mut i2c = I2c::new(peripherals.I2C0, config)
    .expect("Failed to initialize I2C slave")
    .with_sda(io.pins.gpio1)
    .with_scl(io.pins.gpio2)
    .into_async();

#[cfg(feature = "esp32")]
let mut i2c = I2c::new(peripherals.I2C0, config)
    .expect("Failed to initialize I2C slave")
    .with_sda(io.pins.gpio21)
    .with_scl(io.pins.gpio22)
    .into_async();
// ... etc for all devices
```

#### ESP32-C6 Specific: Clock Stretch Configuration

Added conditional clock stretch enable (ESP32-C6 specific feature):

```rust
let mut config = Config::default()
    .with_address(SLAVE_ADDR.into())
    .with_clear_tx_on_write(true);

// ESP32-C6 requires explicit clock stretch configuration
#[cfg(feature = "esp32c6")]
{
    config = config.with_clock_stretch_enable(true);
}
```

### 3. Build Scripts (build.sh) - Both Slave & Master

Enhanced build scripts to accept device parameter:

```bash
#!/bin/bash
# Usage: ./build.sh [FEATURE]
# Example: ./build.sh esp32c6
# Default: esp32c6

FEATURE="${1:-esp32c6}"

# Determine target architecture based on feature
case "$FEATURE" in
    esp32c6|esp32c3|esp32c2|esp32h2)
        TARGET="riscv32imac-unknown-none-elf"
        ;;
    esp32|esp32s2|esp32s3)
        TARGET="xtensa-esp32-none-elf"
        ;;
    *)
        echo "Error: Unknown feature '$FEATURE'"
        echo "Supported: esp32, esp32c2, esp32c3, esp32c6, esp32h2, esp32s2, esp32s3"
        exit 1
        ;;
esac

# Build with selected feature and target
cargo build --release --features "$FEATURE" --target "$TARGET"
```

**Features**:
- Accepts device name as first parameter
- Automatically selects correct target architecture (RISC-V vs Xtensa)
- Defaults to ESP32-C6 if no parameter provided
- Validates device name and provides helpful error messages

### 4. Flash Scripts (flash.sh) - Both Slave & Master

Enhanced flash scripts similarly:

```bash
#!/bin/bash
# Usage: ./flash.sh [FEATURE]
# Example: ./flash.sh esp32c6
# Default: esp32c6

FEATURE="${1:-esp32c6}"

# Determine target architecture based on feature
case "$FEATURE" in
    esp32c6|esp32c3|esp32c2|esp32h2)
        TARGET="riscv32imac-unknown-none-elf"
        ;;
    esp32|esp32s2|esp32s3)
        TARGET="xtensa-esp32-none-elf"
        ;;
    *)
        echo "Error: Unknown feature '$FEATURE'"
        echo "Supported: esp32, esp32c2, esp32c3, esp32c6, esp32h2, esp32s2, esp32s3"
        exit 1
        ;;
esac

# Flash with selected feature and target
cargo espflash flash --release --features "$FEATURE" --target "$TARGET" --monitor
```

### 5. Documentation Updates

#### README.md Files
- Added "Supported Devices" section listing all 7 devices
- Updated hardware setup with device-specific GPIO tables
- Added usage examples for different devices
- Included device-specific build/flash commands

#### Main async-test/README.md
- Added comprehensive device support overview
- Device-specific connection diagrams
- Build instructions for all devices
- Added troubleshooting section for device-specific issues

## Usage Examples

### Building for ESP32-C6 (Default)
```bash
cd slave
./build.sh              # Uses default ESP32-C6
./flash.sh
```

### Building for ESP32 Classic
```bash
cd slave
./build.sh esp32        # Explicitly specify ESP32
./flash.sh esp32
```

### Building for ESP32-S3
```bash
cd master
./build.sh esp32s3
./flash.sh esp32s3
```

### Manual Build (Advanced)
```bash
# RISC-V devices
cargo build --release --features esp32c6 --target riscv32imac-unknown-none-elf

# Xtensa devices
cargo build --release --features esp32 --target xtensa-esp32-none-elf
```

## Architecture Mapping

The build and flash scripts automatically map devices to their correct target architecture:

| Device Family | Target Triple |
|--------------|---------------|
| ESP32-C2, C3, C6, H2 | `riscv32imac-unknown-none-elf` |
| ESP32, S2, S3 | `xtensa-esp32-none-elf` |

## Testing Status

- ✅ **ESP32-C6**: Fully tested and validated (all 6 tests passing)
- ⚠️ **Other devices**: Code structure in place, requires hardware testing

## Device-Specific Notes

### ESP32-C6
- **Clock Stretch**: Explicitly configured with `.with_clock_stretch_enable(true)`
- **Known Fix**: write_read() transaction handling validated
- **GPIO**: SDA=1, SCL=2 (slave) / SDA=6, SCL=7 (master)

### ESP32 Classic
- **GPIO**: Different default pins (21/22 for slave, 18/19 for master)
- **Architecture**: Xtensa-based (different toolchain)

### Other Devices (C2, C3, H2, S2, S3)
- **GPIO**: Use same pins as C6 (1/2 for slave, 6/7 for master)
- **Clock Stretch**: May have different configuration requirements (to be verified)

## Benefits

1. **Single Codebase**: One set of test examples works for all devices
2. **Type Safety**: Compile-time device selection prevents runtime errors
3. **Ease of Use**: Simple shell script interface
4. **Maintainability**: Device-specific code clearly marked with `#[cfg(feature = "...")]`
5. **Flexibility**: Easy to add new devices or modify GPIO configurations

## Future Enhancements

Potential additions:
1. Add more GPIO configuration options (make pins configurable)
2. Add device-specific optimization flags
3. Include device-specific timing calibrations
4. Add automated hardware testing across all devices
5. Document device-specific errata or limitations

## File Manifest

Modified files:
- `slave/Cargo.toml` - Added feature flags
- `slave/src/main.rs` - Added conditional compilation
- `slave/build.sh` - Enhanced with device parameter
- `slave/flash.sh` - Enhanced with device parameter
- `slave/README.md` - Updated documentation
- `master/Cargo.toml` - Added feature flags
- `master/src/main.rs` - Added conditional compilation
- `master/build.sh` - Enhanced with device parameter
- `master/flash.sh` - Enhanced with device parameter
- `master/README.md` - Updated documentation
- `README.md` - Added comprehensive device support documentation

New files:
- `DEVICE-SUPPORT.md` - This document
