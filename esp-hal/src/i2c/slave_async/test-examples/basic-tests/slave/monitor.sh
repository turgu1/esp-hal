#!/bin/bash

# Flash script for I2C slave blocking test
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

echo "=== Monitoring ESP32 I2C Slave (Blocking) ==="
echo "Device: $FEATURE"
echo ""

# Check if cargo-espflash is available
if ! command -v cargo-espflash &> /dev/null; then
    echo "Error: cargo-espflash not found."
    echo "Install with: cargo install cargo-espflash"
    exit 1
fi

# Flash and monitor
cargo espflash monitor

if [ $? -ne 0 ]; then
    echo ""
    echo "✗ Monitor failed!"
    exit 1
fi
