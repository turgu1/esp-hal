# Quick Start Guide

## Hardware Setup

1. **Connect two ESP32-C6 boards:**
   ```
   Master          Slave
   GPIO 6 (SDA) -- GPIO 1 (SDA)
   GPIO 7 (SCL) -- GPIO 2 (SCL)
   GND --------- GND
   ```

2. **Add pull-up resistors:**
   - 4.7kΩ resistor from SDA to 3.3V
   - 4.7kΩ resistor from SCL to 3.3V

## Building and Running

### Option 1: Using provided scripts

**For Slave:**
```bash
cd test-examples
chmod +x build_slave.sh
./build_slave.sh
cargo espflash flash --release --monitor
```

**For Master:**
```bash
cd test-examples
chmod +x build_master.sh
./build_master.sh
cargo espflash flash --release --monitor
```

### Option 2: Manual build

**For Slave:**
```bash
cd test-examples
cargo build --manifest-path Cargo_slave.toml --release --target riscv32imac-unknown-none-elf
cargo espflash flash --manifest-path Cargo_slave.toml --release --monitor
```

**For Master:**
```bash
cd test-examples
cargo build --manifest-path Cargo_master.toml --release --target riscv32imac-unknown-none-elf
cargo espflash flash --manifest-path Cargo_master.toml --release --monitor
```

## Expected Results

### Slave Terminal Output:
```
ESP32-C6 I2C Slave (Async Mode) - Starting...
I2C Slave initialized at address 0x55
Waiting for master transactions...

[Test 1] Echo: 4 bytes received
  ✓ Echoed 4 bytes
[Test 2] Simple command received
  ✓ Sent response: 0x42
[Test 3] write_read command: 1 bytes
  ✓ Sent response: 0x43 (clock stretch: ~10-12ms)
[Test 4] Multi-byte command received
  ✓ Sent 4-byte response
[Test 5] Status query received
  ✓ Sent status: 0xFF
[Test 6] write_read command: 31 bytes
  ✓ Sent response: 0x43 (clock stretch: ~10-12ms)

=== All tests completed! ===
Tests passed: 6/6
```

### Master Terminal Output:
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

## Troubleshooting

### Problem: Cannot build
- Ensure you're in the correct directory
- Check that Cargo.toml paths are correct
- Verify esp-hal dependency path points to the correct location

### Problem: Flash fails
- Check USB cable connection
- Ensure correct serial port selected
- Try: `cargo espflash board-info` to verify connection

### Problem: Tests fail
- Verify hardware connections (especially pull-ups)
- Check both boards are powered
- Ensure slave is started first
- Use logic analyzer to inspect I2C signals

### Problem: Test 3 or 6 fails with wrong data (0x42 instead of 0x43)
- This indicates clock stretch is being released too early
- Verify the clock stretch fix is applied correctly in the driver
- Check that `read_fifo()` does NOT release clock stretch

## Next Steps

After successful testing:
1. Analyze clock stretch timing with logic analyzer
2. Verify FIFO behavior at threshold (30 bytes)
3. Test with different I2C speeds (50kHz, 400kHz)
4. Test with longer command processing times
5. Test error conditions (bus errors, timeouts)

## Notes

- These examples use embassy-executor for async/await
- Clock stretching allows slave to process commands without data loss
- Test 3 and 6 are critical - they validate the write_read() fix
- All tests should show consistent clock stretch timing
