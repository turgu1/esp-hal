# I2C Slave Driver - Testing and Validation Checklist

This document provides a comprehensive checklist for testing and validating the I2C slave driver implementation.

## Pre-Testing Setup

### Hardware Requirements

- [ ] ESP32 development board (slave)
- [ ] I2C master device (another ESP32, Arduino, Raspberry Pi, or USB-to-I2C adapter)
- [ ] Pull-up resistors (typically 4.7kΩ) for SDA and SCL
- [ ] Breadboard and jumper wires
- [ ] Logic analyzer (optional but highly recommended)
- [ ] Oscilloscope (optional)

### Software Requirements

- [ ] Rust toolchain installed
- [ ] esp-hal repository cloned
- [ ] Probe-rs or other flashing tool
- [ ] Serial monitor for debugging output

### Wiring Checklist

- [ ] SDA: Master → Slave (with pull-up resistor to VCC)
- [ ] SCL: Master → Slave (with pull-up resistor to VCC)
- [ ] GND: Master → Slave (common ground)
- [ ] VCC: Proper power supply for both devices

## Compilation Tests

### Basic Compilation

- [ ] `cargo build` succeeds without errors
- [ ] `cargo build --release` succeeds
- [ ] No clippy warnings: `cargo clippy`
- [ ] Formatting is correct: `cargo fmt --check`

### Feature Flag Testing

- [ ] Compile with `defmt` feature
- [ ] Compile without optional features
- [ ] Test on different target chips:
  - [ ] ESP32
  - [ ] ESP32-S2
  - [ ] ESP32-S3
  - [ ] ESP32-C3
  - [ ] ESP32-C6
  - [ ] ESP32-H2

## Unit Tests (Code Level)

### Configuration Tests

- [ ] Default configuration values are correct
- [ ] Valid 7-bit addresses (0x00-0x7F) are accepted
- [ ] Invalid addresses (>0x7F) return error
- [ ] Builder pattern works correctly:
  - [ ] `with_address()`
  - [ ] `with_clock_stretch_enable()`
  - [ ] `with_sda_filter_enable()`
  - [ ] `with_scl_filter_enable()`

### Driver Instantiation

- [ ] `I2c::new()` succeeds with valid config
- [ ] `I2c::new()` fails with invalid config
- [ ] Pin connections work:
  - [ ] `with_sda()` accepts valid GPIO
  - [ ] `with_scl()` accepts valid GPIO
- [ ] Mode switching works:
  - [ ] `into_async()` returns Async driver
  - [ ] `into_blocking()` returns Blocking driver

### Error Handling

- [ ] Zero-length read returns `ZeroLengthInvalid`
- [ ] Zero-length write returns `ZeroLengthInvalid`
- [ ] FIFO overflow detected
- [ ] FIFO underflow detected
- [ ] Timeout errors handled correctly

## Functional Tests (Hardware Required)

### Basic Communication

#### Test 1: Simple Write from Master
**Setup:** Master writes 1 byte to slave
- [ ] Slave receives byte correctly
- [ ] Slave address matching works
- [ ] No errors reported
- [ ] Interrupts fire as expected (if enabled)

#### Test 2: Simple Read by Master
**Setup:** Master reads 1 byte from slave
- [ ] Slave sends prepared byte
- [ ] Master receives correct byte
- [ ] No errors reported
- [ ] ACK/NACK handled correctly

#### Test 3: Multi-byte Write
**Setup:** Master writes multiple bytes (2-31 bytes)
- [ ] All bytes received correctly
- [ ] Correct byte order
- [ ] FIFO management works
- [ ] No data loss

#### Test 4: Multi-byte Read
**Setup:** Master reads multiple bytes (2-31 bytes)
- [ ] All bytes transmitted correctly
- [ ] Correct byte order
- [ ] FIFO management works
- [ ] Final byte NACK handled

#### Test 5: Maximum FIFO Usage
**Setup:** Test at FIFO read capacity (31 bytes for reads, 32 for writes)
- [ ] Write 32 bytes successfully
- [ ] Read 31 bytes successfully (hardware limit)
- [ ] No overflow/underflow

#### Test 6: Beyond FIFO Capacity
**Setup:** Attempt >31 byte read operations (>32 byte write)
- [ ] Proper error handling
- [ ] No data corruption
- [ ] Recovery possible

### Write-Read (Repeated START) Tests

> **Note:** write_read() functionality is FULLY SUPPORTED on ESP32-C6 and other modern chips.
> See I2C_SLAVE_WRITE_READ_SUPPORT.md for detailed implementation information.

#### Test 6a: write_read() - Single Byte
**Setup:** Master performs write_read([0x10], 1 byte read) with repeated START
- [ ] Slave receives register address (0x10) in write phase
- [ ] Slave responds with data byte in read phase
- [ ] No STOP condition between write and read phases
- [ ] Transaction completes successfully

#### Test 6b: write_read() - Multi-byte Read
**Setup:** Master performs write_read([0x20], 4 byte read)
- [ ] Slave receives register address in write phase
- [ ] Slave responds with 4 bytes in read phase
- [ ] All bytes transmitted correctly
- [ ] Repeated START handled properly

#### Test 6c: write_read() - Register-Based Mode (ESP32-C6)
**Setup:** Enable register_based_mode config, master performs write_read()
- [ ] Hardware automatically separates register address
- [ ] `read_register_address()` returns correct register
- [ ] Data buffer contains only data bytes (not register address)
- [ ] Response can be customized based on register value

#### Test 6d: write_read() - Maximum FIFO
**Setup:** Master performs write_read() with 31-byte read
- [ ] All 31 bytes transmitted correctly
- [ ] FIFO management works properly
- [ ] No overflow/underflow

#### Test 6e: write_read() - Normal Mode (No Register-Based Mode)
**Setup:** Use default config (register_based_mode = false), master performs write_read()
- [ ] Transaction works correctly in normal mode
- [ ] Slave manually handles register address extraction
- [ ] Repeated START handled by hardware automatically
- [ ] Confirms register-based mode is optional, not required

#### Test 6f: write_read() vs Separate Transactions
**Setup:** Compare atomic write_read() vs separate write/read transactions
- [ ] write_read() is atomic (no other master can intervene)
- [ ] Separate transactions have STOP between them
- [ ] Both methods produce correct results
- [ ] write_read() timing is tighter

#### Test 6g: write_read() - ESP32 Master Compatibility
**Setup:** Test with ESP32 (original) as master
- [ ] Disable clock stretching (compatibility requirement)
- [ ] Ensure quick slave response (<10us)
- [ ] Monitor for bus hangs (ESP32 has poor clock stretch support)
- [ ] See ESP32_MASTER_COMPATIBILITY.md for known issues

### Address Testing

#### Test 7: Correct Address Match
**Setup:** Master addresses slave with configured address
- [ ] Slave responds
- [ ] Communication successful

#### Test 8: Wrong Address
**Setup:** Master addresses different address
- [ ] Slave ignores communication
- [ ] No false responses
- [ ] Bus not affected

#### Test 9: Address Configuration Change
**Setup:** Change slave address via `apply_config()`
- [ ] New address takes effect
- [ ] Old address ignored
- [ ] No communication disruption

### Clock Stretching Tests

#### Test 10: Clock Stretching Enabled
**Setup:** Enable clock stretching, slow response
- [ ] Slave can stretch clock
- [ ] Master waits correctly
- [ ] No timeout errors
- [ ] Communication completes

#### Test 11: Clock Stretching Disabled
**Setup:** Disable clock stretching
- [ ] Slave doesn't stretch clock
- [ ] Fast response required
- [ ] Works with fast master

### Filtering Tests

#### Test 12: Noise Rejection
**Setup:** Inject noise on SDA/SCL lines
- [ ] Filter removes noise
- [ ] Communication remains stable
- [ ] No false START/STOP

#### Test 13: Filter Threshold Adjustment
**Setup:** Vary filter threshold (0-7)
- [ ] Different thresholds work
- [ ] Appropriate noise rejection
- [ ] No performance degradation

### Interrupt Tests

#### Test 14: RxFifoFull Interrupt
**Setup:** Enable RxFifoFull interrupt
- [ ] Interrupt fires when FIFO fills
- [ ] Correct event reported
- [ ] Can be cleared

#### Test 15: TxFifoEmpty Interrupt
**Setup:** Enable TxFifoEmpty interrupt
- [ ] Interrupt fires when FIFO empties
- [ ] Correct event reported
- [ ] Can be cleared

#### Test 16: TransComplete Interrupt
**Setup:** Enable TransComplete interrupt
- [ ] Interrupt fires at transaction end
- [ ] Correct timing
- [ ] Can be cleared

#### Test 17: Multiple Interrupts
**Setup:** Enable multiple interrupt sources
- [ ] All enabled interrupts fire correctly
- [ ] Can distinguish between events
- [ ] Proper clearing mechanism

### Error Condition Tests

#### Test 18: Bus Arbitration Lost
**Setup:** Multiple masters on same bus
- [ ] Arbitration detected
- [ ] Error reported correctly
- [ ] Recovery possible

#### Test 19: Timeout Handling
**Setup:** Force timeout condition
- [ ] Timeout detected
- [ ] Error reported
- [ ] Driver recovers

#### Test 20: Bus Busy Condition
**Setup:** Attempt operation while bus busy
- [ ] Busy detected
- [ ] Error reported
- [ ] Waits or fails gracefully

### Async Mode Tests

#### Test 21: Async Read
**Setup:** Use `read_async()` with executor
- [ ] Returns future
- [ ] Future completes on data
- [ ] Correct data received
- [ ] Executor not blocked

#### Test 22: Async Write
**Setup:** Use `write_async()` with executor
- [ ] Returns future
- [ ] Future completes on send
- [ ] Correct data sent
- [ ] Executor not blocked

#### Test 23: Concurrent Operations
**Setup:** Multiple async operations
- [ ] Operations don't interfere
- [ ] Correct ordering
- [ ] No data corruption

#### Test 24: Future Cancellation
**Setup:** Drop future before completion
- [ ] Proper cleanup
- [ ] No resource leaks
- [ ] Can start new operation

## Performance Tests

### Speed Tests

#### Test 25: 100 kHz Operation
**Setup:** Master at 100 kHz
- [ ] Slave keeps up
- [ ] No data loss
- [ ] Stable communication

#### Test 26: 400 kHz Operation (Fast Mode)
**Setup:** Master at 400 kHz
- [ ] Slave keeps up
- [ ] Clock stretching may be needed
- [ ] Stable communication

#### Test 27: 1 MHz Operation (Fast Mode Plus)
**Setup:** Master at 1 MHz (if supported)
- [ ] Slave behavior documented
- [ ] Clock stretching requirement noted
- [ ] Limitations documented

### Throughput Tests

#### Test 28: Continuous Write
**Setup:** Master writes continuously
- [ ] Sustained throughput measured
- [ ] No buffer overruns
- [ ] Consistent performance

#### Test 29: Continuous Read
**Setup:** Master reads continuously
- [ ] Sustained throughput measured
- [ ] No buffer underruns
- [ ] Consistent performance

#### Test 30: Alternating Read/Write
**Setup:** Master alternates operations
- [ ] Both directions work
- [ ] No mode switching issues
- [ ] Performance acceptable

## Reliability Tests

### Stress Tests

#### Test 31: Extended Duration
**Setup:** Run for extended period (hours)
- [ ] No degradation over time
- [ ] No memory leaks
- [ ] Stable operation

#### Test 32: Repeated Start/Stop
**Setup:** Repeatedly initialize and deinitialize
- [ ] Clean initialization each time
- [ ] No resource leaks
- [ ] Consistent behavior

#### Test 33: Error Recovery
**Setup:** Force errors and recover
- [ ] Driver recovers from errors
- [ ] Can resume normal operation
- [ ] No permanent damage

### Environmental Tests

#### Test 34: Temperature Range
**Setup:** Test at different temperatures
- [ ] Works at room temperature
- [ ] Works at temperature extremes
- [ ] Performance consistent

#### Test 35: Power Supply Variation
**Setup:** Vary supply voltage (within spec)
- [ ] Works at nominal voltage
- [ ] Works at voltage extremes
- [ ] No communication errors

#### Test 36: Cable Length
**Setup:** Test with different cable lengths
- [ ] Short cables work
- [ ] Longer cables tested
- [ ] Limits documented

## Real-World Scenario Tests

### Application Tests

#### Test 37: Sensor Emulation
**Setup:** Slave emulates sensor behavior
- [ ] Register reads work
- [ ] Register writes work
- [ ] Sequential access works

#### Test 38: EEPROM Emulation
**Setup:** Slave emulates EEPROM
- [ ] Address-based access works
- [ ] Write-then-read works
- [ ] Page boundaries handled

#### Test 39: Protocol Implementation
**Setup:** Implement specific protocol (SMBus, PMBus, etc.)
- [ ] Protocol commands work
- [ ] CRC/checksum if required
- [ ] Timing requirements met

## Chip-Specific Tests

### ESP32 Tests
- [ ] Legacy FIFO access works
- [ ] No clock stretching verified
- [ ] Register layout differences handled

### ESP32-S2 Tests
- [ ] Clock stretching works
- [ ] Improved interrupt system works
- [ ] AHB FIFO access works

### ESP32-S3/C3/C6/H2 Tests
- [ ] Modern features work
- [ ] Direct FIFO access works
- [ ] All interrupts available

## Documentation Tests

### Code Documentation

- [ ] All public APIs documented
- [ ] Examples compile and work
- [ ] Doc comments accurate
- [ ] Links work

### External Documentation

- [ ] README.md accurate
- [ ] DESIGN.md reflects implementation
- [ ] EXAMPLE.md works
- [ ] Testing checklist complete

## Integration Tests

### With Other Peripherals

#### Test 40: I2C + GPIO
**Setup:** Use I2C slave with GPIO interrupts
- [ ] No conflicts
- [ ] Both work simultaneously
- [ ] Timing not affected

#### Test 41: I2C + UART
**Setup:** Use I2C slave with UART
- [ ] No conflicts
- [ ] Both work simultaneously
- [ ] No data corruption

#### Test 42: I2C + Timer
**Setup:** Use I2C slave with timers
- [ ] No conflicts
- [ ] Precise timing maintained
- [ ] Interrupts coexist

### With Operating Systems

#### Test 43: Bare Metal
**Setup:** No OS, just hardware
- [ ] Works correctly
- [ ] Direct hardware access
- [ ] Optimal performance

#### Test 44: With Embassy
**Setup:** Embassy async runtime
- [ ] Async operations work
- [ ] Executor scheduling correct
- [ ] No deadlocks

#### Test 45: With RTIC
**Setup:** RTIC framework (if applicable)
- [ ] Task scheduling works
- [ ] Priority handling correct
- [ ] Resource sharing safe

## Regression Tests

After any code changes:

- [ ] All previous tests still pass
- [ ] No new warnings
- [ ] Performance not degraded
- [ ] Documentation still accurate

## Sign-Off

### Testing Summary

**Date:** _______________

**Tester:** _______________

**Chips Tested:**
- [ ] ESP32
- [ ] ESP32-S2
- [ ] ESP32-S3
- [ ] ESP32-C3
- [ ] ESP32-C6
- [ ] ESP32-H2

**Pass Rate:** _____ / _____ tests passed

**Known Issues:**
1. _______________________________________
2. _______________________________________
3. _______________________________________

**Blockers:**
1. _______________________________________
2. _______________________________________

**Ready for Release:** [ ] Yes [ ] No

**Notes:**
_____________________________________________
_____________________________________________
_____________________________________________

### Reviewer Sign-Off

**Reviewer:** _______________

**Date:** _______________

**Approval:** [ ] Approved [ ] Changes Required

**Comments:**
_____________________________________________
_____________________________________________
_____________________________________________
