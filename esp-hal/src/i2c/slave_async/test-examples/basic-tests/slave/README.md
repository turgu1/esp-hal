# SlaveAsync Basic Test Example

This example demonstrates the **interrupt-driven async I2C slave driver** (`SlaveAsync`) working with the blocking master test while running **concurrent tasks** on the same core.

## Purpose

This test **proves** that the async I2C slave driver enables true concurrency:
- ✅ LED blinks smoothly at 500ms intervals **even while waiting for I2C**
- ✅ Counter increments every second **regardless of I2C activity**
- ✅ I2C transactions complete successfully **without blocking other tasks**

## Hardware Setup

### Required Components
- **2x ESP32 boards** (master + slave)
- **2x 4.7kΩ resistors** (I2C pull-ups)
- **1x LED** (optional, for visual demonstration)
- **Breadboard and jumper wires**

### Wiring (ESP32-C6 Default)

```
Master (GPIO)          Slave (GPIO)
─────────────          ────────────
GPIO 6 (SDA) ──────────── GPIO 1 (SDA)
                   │
                  4.7kΩ to 3.3V
                   
GPIO 7 (SCL) ──────────── GPIO 2 (SCL)
                   │
                  4.7kΩ to 3.3V

GND ───────────────────── GND

                      GPIO 8 ──── LED ──── 330Ω ──── GND
```

### GPIO Pin Mappings (by device)

| Device | SDA (Slave) | SCL (Slave) | LED |
|--------|-------------|-------------|-----|
| ESP32-C6 | GPIO 1 | GPIO 2 | GPIO 8 |
| ESP32 | GPIO 21 | GPIO 22 | GPIO 2 |
| ESP32-C2 | GPIO 1 | GPIO 2 | GPIO 8 |
| ESP32-C3 | GPIO 1 | GPIO 2 | GPIO 8 |
| ESP32-H2 | GPIO 1 | GPIO 2 | GPIO 8 |
| ESP32-S2 | GPIO 1 | GPIO 2 | GPIO 18 |
| ESP32-S3 | GPIO 1 | GPIO 2 | GPIO 48 |

**Master** uses different pins - see [master test](../../../../slave/test-examples/blocking-test/master/README.md).

## Test Protocol

The slave responds to commands from the blocking master test:

| Command | Test Name | Slave Response | Description |
|---------|-----------|----------------|-------------|
| `0x01` | Echo test | Echoes received data | Stores data, sends on next read |
| `0x10` | Simple read | `0x42` | Single byte response |
| `0x20` | write_read single | `0x43` | Write+read with repeated START |
| `0x30` | write_read multi | `[0..15]` | 16 sequential bytes |
| `0x40` | write_read max FIFO | `[0..30]` | 31 sequential bytes (max) |

## Running the Tests

### 1. Flash the Slave (this example)

```bash
cd esp-hal/src/i2c/slave_async/test-examples/basic-tests/slave

# ESP32-C6 (default)
cargo run --release

# Other devices
cargo run --release --no-default-features --features esp32
cargo run --release --no-default-features --features esp32c3
# ... etc
```

### 2. Flash the Master

In a **separate terminal**:

```bash
cd esp-hal/src/i2c/slave/test-examples/blocking-test/master

# ESP32-C6 (default)
cargo run --release

# Other devices
cargo run --release --no-default-features --features esp32
# ... etc
```

### 3. Monitor Both Terminals

**Slave output** (this example):
```
=== ESP32-C6 I2C Slave Async (Basic Test) ===
I2C Slave initialized (Async/Interrupt-Driven)
GPIO: SDA=1, SCL=2
Slave address: 0x55
LED pin: GPIO 8

✓ All tasks spawned successfully
✓ System is now fully concurrent - I2C + LED + Counter

[LED] Blinker task started - 500ms interval
[COUNTER] Task started - 1 second interval

[LED] 💡 ON  (blink #1)
[LED] 🌑 OFF (blink #1)
[COUNTER] 📊 Count = 1 seconds elapsed
[I2C #001] Received 4 bytes: [01, AA, BB, CC]
[LED] 💡 ON  (blink #2)        ← LED STILL BLINKS DURING I2C! ✓
[I2C #001] Echo test: storing 4 bytes for next read
[I2C #001] Master reading echo: 4 bytes [01, AA, BB, CC]
[LED] 🌑 OFF (blink #2)
[COUNTER] 📊 Count = 2 seconds elapsed
[I2C #002] Received 1 bytes: [10]
[I2C #002] Simple read command: preloading 0x42
...
```

**Master output**:
```
=== ESP32-C6 I2C Master (Blocking Mode) ===
Testing I2C Slave Blocking Functionality

Starting I2C Slave Test Suite

Test 1: Simple Write with Echo
  Writing 4 bytes: [01, AA, BB, CC]
  Write successful
  Reading echo response...
  Echo received: [01, AA, BB, CC]
  Echo matches sent data
  ✓ PASS

Test 2: Simple Read
  Writing command: [10]
  Reading 1 bytes...
  Received: [42]
  Response matches expected value (0x42)
  ✓ PASS
...

🎉 ALL TESTS PASSED! 🎉
```

## Expected Behavior

### ✅ Success Indicators

1. **Master**: All tests show `✓ PASS`
2. **Slave**: 
   - LED blinks **continuously** and **smoothly** (500ms on/off)
   - Counter increments **every second** without interruption
   - I2C transactions logged with command details
   - **No gaps** in LED/counter timing during I2C activity

### ❌ Failure Indicators

| Symptom | Cause | Solution |
|---------|-------|----------|
| LED freezes during I2C | Clock stretch timeout or FIFO overflow | Check pull-up resistors, verify wiring |
| Master timeout errors | Slave not responding | Verify GPIO pins match, check power |
| Echo test fails | Protocol mismatch | Ensure master/slave use same command codes |
| Bus hangs (SCL low) | Clock stretch issue with ESP32 master | Disable clock stretch in config |

## Performance Observations

With the **async interrupt-driven** driver, you should observe:

| Metric | Standard Slave | SlaveAsync (this) |
|--------|---------------|-------------------|
| LED blink regularity | ❌ Pauses during I2C | ✅ Smooth, no pauses |
| Counter accuracy | ❌ Delayed by I2C | ✅ Precise 1 second |
| CPU usage idle | ~100% polling | ~0% (sleeping) |
| Response latency | ~1µs | <1µs (interrupt) |

### Example Timeline (Async Driver)

```
Time      Event
────────  ──────────────────────────────────────────
0ms       Slave boots, tasks start
500ms     LED blinks #1 ✓
1000ms    Counter = 1 ✓
1234ms    Master sends command 0x01
1234ms    Interrupt wakes I2C task (I2C handles it)
1500ms    LED blinks #2 ✓  ← No delay from I2C!
2000ms    Counter = 2 ✓
2100ms    Master sends command 0x10
2100ms    Interrupt wakes, responds instantly
2500ms    LED blinks #3 ✓
3000ms    Counter = 3 ✓
```

The LED and counter maintain **perfect timing** regardless of I2C activity!

## Concurrent Tasks Architecture

```
┌─────────────────────────────────────────────────────────────┐
│           Embassy Async Executor (Single Core)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Task 1: i2c_slave_task()                                  │
│    • read_async() → waits for interrupt → processes cmd    │
│    • write_async() → preloads response → yields            │
│    • DOES NOT BLOCK - yields while waiting                 │
│                                                             │
│  Task 2: led_blinker_task()                                │
│    • Timer::after(500ms) → yields                          │
│    • Toggle LED → yields                                   │
│    • Runs DURING I2C waits ✓                               │
│                                                             │
│  Task 3: counter_task()                                    │
│    • Timer::after(1000ms) → yields                         │
│    • Increment count → yields                              │
│    • Runs DURING I2C waits ✓                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                           ↑
                           │ Hardware Interrupt
                           │
┌─────────────────────────────────────────────────────────────┐
│              I2C Peripheral + Interrupt Handler             │
│  • Master transaction → interrupt (200-500ns)              │
│  • Handler reads/writes FIFO (2-5µs)                       │
│  • Waker wakes i2c_slave_task                              │
└─────────────────────────────────────────────────────────────┘
```

All three tasks share one CPU core, switching when they `await` or yield!

## Troubleshooting

### Issue: LED stops blinking during I2C

**Diagnosis**:
```bash
# Monitor slave output for errors
tail -f /dev/ttyUSB0  # or ttyACM0
```

Look for:
- `[I2C] Read error: Timeout` → Increase timeout in config
- `[I2C] Read error: RxFifoOverflow` → Lower RX FIFO threshold
- LED freezes completely → Not using async driver

**Fix**:
```rust
let config = Config::default()
    .with_rx_fifo_threshold(8)  // Lower threshold
    .with_timeout_ms(5000);     // Increase timeout
```

### Issue: Master reports timeout errors

**Cause**: Slave not responding fast enough

**Solutions**:
1. Enable clock stretching:
   ```rust
   .with_clock_stretch_enable(true)
   ```

2. Increase master timeout:
   ```rust
   .with_timeout(BusTimeout::BusCycles(5000))
   ```

3. Check hardware: verify pull-ups (4.7kΩ) and clean wiring

### Issue: Echo test fails

**Cause**: State management issue with echo buffer

**Debug**:
```rust
println!("Echo buffer before: {:?}", echo_buffer.borrow());
println!("Echo len: {}", *echo_len.borrow());
```

Verify echo data is saved correctly before the master reads it.

## Code Modifications

### Change LED Blink Rate

```rust
#[embassy_executor::task]
async fn led_blinker_task(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(250)).await;  // 250ms → faster
    }
}
```

### Add More Concurrent Tasks

```rust
#[embassy_executor::task]
async fn sensor_reader_task() {
    loop {
        let temp = read_sensor().await;
        println!("[SENSOR] Temperature: {}°C", temp);
        Timer::after(Duration::from_secs(5)).await;
    }
}

// In main():
spawner.spawn(sensor_reader_task()).ok();
```

### Disable Clock Stretching (ESP32 Master)

If using ESP32 as master (poor clock stretch support):

```rust
let config = Config::default()
    .with_clock_stretch_enable(false)  // Disable for ESP32
    .with_rx_fifo_threshold(4);        // Interrupt more frequently
```

## Benchmarking

To measure interrupt latency:

```rust
use esp_hal::time::Instant;

let start = Instant::now();
match i2c.read_async(&mut buffer).await {
    Ok(len) => {
        let elapsed = start.elapsed();
        println!("I2C transaction took: {}µs", elapsed.as_micros());
    }
    Err(e) => { /* ... */ }
}
```

Expected measurements:
- Interrupt latency: 200-500ns
- FIFO read (32 bytes): 2-5µs
- Total transaction: 10-50µs (depends on size)

## Implementation Status

- [x] Basic I2C slave async handler
- [x] Protocol implementation (commands 0x01-0x40)
- [x] LED blinker concurrent task
- [x] Counter concurrent task
- [x] Multi-device GPIO support
- [x] Echo buffer state management
- [ ] Error recovery stress testing
- [ ] Performance benchmarks vs blocking driver
- [ ] DMA integration (future)

## See Also

- [SlaveAsync README](../../README.md) - Driver documentation
- [Blocking Master Test](../../../../slave/test-examples/blocking-test/master/) - Master test code
- [INTERRUPT_BASED_DESIGN.md](../../../../slave/test-examples/INTERRUPT_BASED_DESIGN.md) - Design rationale
- [Embassy Documentation](https://embassy.dev/) - Async runtime

## License

This example is part of esp-hal and follows the same license terms.

---

**Status**: Experimental - Requires `unstable` feature  
**Last updated**: October 2025
