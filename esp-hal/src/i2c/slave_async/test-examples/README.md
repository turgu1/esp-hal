# SlaveAsync Test Examples

This directory contains test examples for the **interrupt-driven async I2C slave driver** (`SlaveAsync`).

## Overview

These examples demonstrate the key advantage of the async driver: **true concurrency** on a single-core system. While the I2C slave waits for master transactions, other async tasks continue running without any blocking or delays.

## Available Examples

### 1. Basic Tests (`basic-tests/slave/`)

**Purpose**: Comprehensive async slave implementation that works with the blocking master test while running concurrent LED and counter tasks.

**Key Features**:
- ✅ Responds to master test protocol (commands 0x01-0x40)
- ✅ LED blinks smoothly at 500ms (visual proof of non-blocking)
- ✅ Counter increments every second (concurrent execution)
- ✅ Supports all ESP32 variants

**Hardware Required**:
- 2x ESP32 boards (master + slave)
- 2x 4.7kΩ resistors (I2C pull-ups)
- 1x LED + 330Ω resistor (optional, for demonstration)

**Documentation**: [basic-tests/slave/README.md](basic-tests/slave/README.md)

## Quick Start

### Run the Basic Test

```bash
# Terminal 1: Flash async slave (this example)
cd esp-hal/src/i2c/slave_async/test-examples/basic-tests/slave
cargo run --release --features esp32c6

# Terminal 2: Flash blocking master test
cd esp-hal/src/i2c/slave/test-examples/blocking-test/master
cargo run --release --features esp32c6
```

**Expected Result**: 
- Master: `🎉 ALL TESTS PASSED! 🎉`
- Slave: LED blinks smoothly, counter increments, I2C transactions logged

## Why These Tests Matter

### The Problem (Standard Blocking Driver)

```rust
// Blocking I2C slave
loop {
    i2c.read(&mut buffer)?;  // ← BLOCKS CPU for up to 2 seconds!
    process(&buffer);
    i2c.write(&response)?;
}

// Meanwhile...
led_task() {
    loop {
        blink_led();  // ← Can't run! Waiting for I2C!
        delay(500ms);
    }
}
```

**Result**: LED freezes, counter stops, system appears hung during I2C waits.

### The Solution (Async Driver - These Examples)

```rust
// Async I2C slave
loop {
    i2c.read_async(&mut buffer).await?;  // ← Yields CPU to other tasks!
    process(&buffer).await;
    i2c.write_async(&response).await?;
}

// Meanwhile...
led_task() {
    loop {
        blink_led();  // ← Runs perfectly! ✓
        Timer::after(500ms).await;
    }
}
```

**Result**: LED blinks smoothly, counter increments precisely, I2C works concurrently!

## Test Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                  Master Board (Blocking)                       │
│  • Sends test commands (0x01, 0x10, 0x20, 0x30, 0x40)        │
│  • Validates responses                                         │
│  • Reports PASS/FAIL                                          │
└───────────────────────┬────────────────────────────────────────┘
                        │ I2C Bus
                        │ (SDA/SCL + pull-ups)
                        │
┌───────────────────────┴────────────────────────────────────────┐
│                  Slave Board (Async)                           │
│                                                                │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  Embassy Async Executor (Single Core)                   │  │
│  ├─────────────────────────────────────────────────────────┤  │
│  │  Task 1: I2C Slave Handler                             │  │
│  │    • read_async() → yields while waiting               │  │
│  │    • Process command                                   │  │
│  │    • write_async() → yields                            │  │
│  │                                                         │  │
│  │  Task 2: LED Blinker                                   │  │
│  │    • Runs during I2C waits ✓                           │  │
│  │    • 500ms on/off                                      │  │
│  │                                                         │  │
│  │  Task 3: Counter                                       │  │
│  │    • Runs during I2C waits ✓                           │  │
│  │    • 1 second increment                                │  │
│  └─────────────────────────────────────────────────────────┘  │
│                         ↑                                      │
│                         │ Hardware Interrupt                   │
│                         │                                      │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  I2C Peripheral + Interrupt Handler (#[ram])           │  │
│  │    • <1µs response to master                           │  │
│  │    • Wakes appropriate async task                      │  │
│  └─────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

## Hardware Setup (ESP32-C6 Example)

```
                Master                    Slave
              ┌────────┐                ┌────────┐
              │ ESP32  │                │ ESP32  │
              │        │                │        │
    GPIO 6 ───┤ SDA    │                │ SDA    ├─── GPIO 1
              │        │    ┌──────┐    │        │
              │        ├────┤4.7kΩ ├────┤        │
              │        │    └───┬──┘    │        │
    GPIO 7 ───┤ SCL    │        │       │ SCL    ├─── GPIO 2
              │        │    ┌───┴──┐    │        │
              │        ├────┤4.7kΩ ├────┤        │
              │        │    └───┬──┘    │        │
              │        │        │       │        │
              │    GND ├────────┼───────┤ GND    │
              └────────┘        │       └────────┘
                                │
                              3.3V
                              
                            LED ┄┄┄┄┄ GPIO 8 (Slave)
                             │         with 330Ω resistor
                            GND
```

### Pull-up Resistors

**Critical**: I2C requires pull-up resistors on SDA and SCL!
- **Value**: 4.7kΩ (1kΩ-10kΩ works, 4.7kΩ optimal for 100kHz)
- **Location**: Between SDA/SCL and 3.3V
- **Quantity**: One per line (2 total)

Without pull-ups, the bus won't work!

## Supported Devices

All ESP32 variants are supported with device-specific GPIO:

| Device | Master SDA/SCL | Slave SDA/SCL | Slave LED |
|--------|----------------|---------------|-----------|
| ESP32-C6 | GPIO 6/7 | GPIO 1/2 | GPIO 8 |
| ESP32 | GPIO 18/19 | GPIO 21/22 | GPIO 2 |
| ESP32-C2 | GPIO 6/7 | GPIO 1/2 | GPIO 8 |
| ESP32-C3 | GPIO 6/7 | GPIO 1/2 | GPIO 8 |
| ESP32-H2 | GPIO 6/7 | GPIO 1/2 | GPIO 8 |
| ESP32-S2 | GPIO 6/7 | GPIO 1/2 | GPIO 18 |
| ESP32-S3 | GPIO 6/7 | GPIO 1/2 | GPIO 48 |

## Common Issues

### Master Timeout

**Symptom**: Master reports timeout errors

**Causes**:
1. Missing pull-up resistors → Add 4.7kΩ resistors
2. Wrong GPIO pins → Verify wiring matches pin table
3. Clock stretching disabled → Enable in slave config
4. Insufficient master timeout → Increase BusTimeout

**Fix**:
```rust
// Slave: Enable clock stretch
let config = Config::default()
    .with_clock_stretch_enable(true);

// Master: Increase timeout
let config = Config::default()
    .with_timeout(BusTimeout::BusCycles(5000));
```

### LED Not Blinking Smoothly

**Symptom**: LED pauses or freezes during I2C

**Diagnosis**: Either not using async driver or FIFO overflow

**Fix**:
```rust
// Verify using SlaveAsync (not standard Slave)
let i2c = SlaveAsync::new(peripherals.I2C0, config)?;  // ✓ Correct

// Adjust FIFO thresholds
let config = Config::default()
    .with_rx_fifo_threshold(8)   // Lower = more frequent interrupts
    .with_tx_fifo_threshold(24); // Higher = less frequent interrupts
```

### Echo Test Fails

**Symptom**: Master receives wrong data in echo test

**Cause**: Timing issue with echo buffer state

**Debug**:
```rust
println!("Echo buffer: {:02X?}", echo_buffer.borrow());
println!("Echo length: {}", *echo_len.borrow());
```

Verify data is saved correctly and cleared after sending.

### Bus Hangs (SCL Stuck Low)

**Symptom**: I2C bus locks up, SCL line held low

**Cause**: Clock stretching incompatible with ESP32 master

**Fix for ESP32 Master**:
```rust
// Slave: Disable clock stretching
let config = Config::default()
    .with_clock_stretch_enable(false)
    .with_rx_fifo_threshold(4);  // Interrupt more frequently instead
```

## Performance Metrics

Expected measurements with async driver:

| Metric | Value | Comparison |
|--------|-------|------------|
| Interrupt latency | 200-500ns | Very fast |
| FIFO read (32 bytes) | 2-5µs | Hardware limit |
| Task wakeup | <10µs | Embassy overhead |
| LED jitter | <5ms | Nearly perfect |
| Counter accuracy | ±1ms | Excellent |
| CPU idle usage | ~0% | vs 100% blocking |

## Future Examples

Planned additions:
- [ ] Register emulation (device firmware emulation)
- [ ] DMA transfer example (large data)
- [ ] Error recovery stress test
- [ ] Power consumption measurement
- [ ] Multi-peripheral example (I2C0 + I2C1)

## Development

### Add a New Test Example

```bash
cd esp-hal/src/i2c/slave_async/test-examples
mkdir -p my-test/slave/src
cd my-test/slave

# Create Cargo.toml (copy from basic-tests)
# Create src/main.rs
# Create README.md
# Add to this index
```

### Test Against Master

Always verify compatibility with the blocking master test:

```bash
# Build both first
cd basic-tests/slave && cargo build --release
cd ../../slave/test-examples/blocking-test/master && cargo build --release

# Flash in sequence
espflash flash target/release/... --monitor
```

## Documentation

- [SlaveAsync Driver README](../README.md) - Full driver documentation
- [Basic Test README](basic-tests/slave/README.md) - This test example
- [Master Test README](../../slave/test-examples/blocking-test/master/README.md) - Master test
- [Design Document](../../slave/test-examples/INTERRUPT_BASED_DESIGN.md) - Architecture rationale

## Contributing

When adding examples:
1. Test on **at least 2 ESP32 variants**
2. Document **hardware setup** clearly
3. Include **expected output** samples
4. Add **troubleshooting** section
5. Update this README index

## License

These examples are part of esp-hal and follow the same license terms.

---

**Status**: Experimental - Requires `unstable` feature  
**Last updated**: October 2025
