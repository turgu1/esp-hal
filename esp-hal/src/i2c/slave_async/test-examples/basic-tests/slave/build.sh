#!/bin/bash

# Build script for I2C slave blocking test
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

echo "=== ESP32 I2C Slave Blocking Test ==="
echo "Device: $FEATURE"
echo "Target: $TARGET"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust toolchain."
    exit 1
fi

# Build the project
echo "Building..."
cargo build --release --features "$FEATURE" --target "$TARGET"

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo ""
    echo "To flash to device, run:"
    echo "  ./flash.sh $FEATURE"
else
    echo ""
    echo "✗ Build failed!"
    exit 1
fi
