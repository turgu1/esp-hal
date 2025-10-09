# I2C Slave Driver Implementation Notes

## Design Philosophy

The I2C slave driver has been designed to mirror the architecture and patterns used in the I2C master driver (`../master/mod.rs`) for several important reasons:

1. **Consistency**: Developers familiar with the master driver can quickly understand the slave driver
2. **Maintainability**: Shared patterns make it easier to maintain both drivers
3. **Code Reuse**: Many utility functions and patterns are similar
4. **Documentation**: Similar structure makes documentation easier to understand

## Key Architectural Decisions

### 1. Driver Mode System

Both master and slave drivers use the same `DriverMode` pattern with `Blocking` and `Async` modes:

```rust
pub struct I2c<'d, Dm: DriverMode> { ... }
```

This allows:
- Type-safe mode switching
- Zero-cost abstractions
- Clear API separation

### 2. Configuration Pattern

Using the builder pattern with `BuilderLite` macro:

```rust
Config::default()
    .with_address(0x55.into())
    .with_clock_stretch_enable(true)
```

Benefits:
- Discoverable API
- Type-safe configuration
- Easy to extend with new options

### 3. Pin Connection Methods

The `with_sda()` and `with_scl()` methods provide a fluent interface:

```rust
I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2)
```

This matches the master driver API exactly.

### 4. Error Handling

Comprehensive error types with `std::error::Error` implementations:

- Configuration errors separate from runtime errors
- Detailed error messages
- defmt support for embedded debugging

### 5. Interrupt System

The `Event` enumeration and interrupt management API:

```rust
i2c.listen(Event::RxFifoFull | Event::TxFifoEmpty);
let events = i2c.interrupts();
i2c.clear_interrupts(events);
```

Provides fine-grained control over interrupt handling.

## Implementation Differences from Master

While the architecture is similar, the slave driver has important differences due to the nature of slave operation:

### 1. No Transaction Queue

**Master Driver:**
```rust
// Uses command queue for complex transactions
setup_write(..., cmd_iterator)?;
setup_read(..., cmd_iterator)?;
```

**Slave Driver:**
```rust
// Direct FIFO access, no command queue
read_fifo(buffer);
write_fifo(buffer)?;
```

**Reason**: Slaves respond to master requests byte-by-byte, they don't initiate multi-step transactions.

### 2. Address Configuration

**Master Driver:**
```rust
// Addresses target slaves
fn write<A: Into<I2cAddress>>(&mut self, address: A, buffer: &[u8])
```

**Slave Driver:**
```rust
// Has its own address
Config::default().with_address(0x55.into())
```

**Reason**: Slaves have a fixed address they respond to, masters address different targets.

### 3. No Frequency Configuration

**Master Driver:**
```rust
Config::default().with_frequency(Rate::from_khz(100))
```

**Slave Driver:**
```rust
// No frequency configuration
// Clock is controlled by master
```

**Reason**: The master controls the clock, slave must follow.

### 4. Clock Stretching

**Slave Driver Only:**
```rust
Config::default().with_clock_stretch_enable(true)
```

Allows the slave to hold SCL low to slow down the master when needed.

### 5. Different Events

**Master Events:**
- EndDetect
- TxComplete
- TxFifoWatermark

**Slave Events:**
- RxFifoFull
- TxFifoEmpty
- ByteReceived/ByteTransmitted
- SlaveAddressed
- StartDetected/StopDetected

**Reason**: Different roles require monitoring different conditions.

## Hardware Register Usage

### Common Registers

Both drivers use:
- `CTR`: Control register (but configure differently)
- `FIFO_CONF`: FIFO configuration
- `DATA`: Data register
- `INT_ENA/INT_CLR/INT_RAW`: Interrupt registers
- `FILTER_CFG`: Signal filtering

### Master-Specific Registers

- `COMD[n]`: Command registers for transaction queue
- `SCL_LOW_PERIOD/SCL_HIGH_PERIOD`: Clock timing
- `CLK_CONF`: Clock divider configuration

### Slave-Specific Registers

- `SLAVE_ADDR`: Slave address configuration
- `SCL_STRETCH_CONF`: Clock stretching (newer chips)

## Chip-Specific Considerations

### ESP32 (Original)

**Limitations:**
- No hardware clock stretching
- Different FIFO access (AHB address)
- Limited interrupt capabilities
- Different register bit layouts

**Workarounds:**
```rust
#[cfg(esp32)]
{
    // Use legacy FIFO access
    let fifo_ptr = (i2c0_data_register_ahb_address + offset) as *mut u32;
}
```

### ESP32-S2

**Improvements:**
- Better interrupt system
- Hardware clock stretching
- More consistent register layout

**Similarities to ESP32:**
- Still uses AHB FIFO access
- Some register layout legacy

### ESP32-S3, C3, C6, H2

**Modern Features:**
- Full hardware clock stretching support
- Direct FIFO register access
- Enhanced interrupt system
- Better FSM timeout handling

## FIFO Management

### Master Driver FIFO

- Used for buffering transaction data
- Managed through command queue
- Reset between transactions

### Slave Driver FIFO

- Used for receive/transmit buffers
- Polled or interrupt-driven
- Must be managed carefully to prevent overflow/underflow

### Best Practices

1. **Check FIFO Status Before Operations:**
```rust
// Master
if buffer.len() > I2C_CHUNK_SIZE {
    return Err(Error::FifoExceeded);
}

// Slave
let status = self.regs().sr().read();
if status.rxfifo_cnt().bits() == 0 { ... }
```

2. **Reset FIFO on Errors:**
```rust
fn reset_fifo(&self) {
    self.regs().fifo_conf().modify(|_, w| {
        w.tx_fifo_rst().set_bit();
        w.rx_fifo_rst().set_bit()
    });
    // Clear and restore
}
```

3. **Handle Overflow/Underflow:**
```rust
// Check for overflow before writing
// Check for underflow before reading
```

## Async Implementation

Both drivers use the same async pattern:

```rust
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct I2cFuture<'a> {
    events: EnumSet<Event>,
    driver: Driver<'a>,
    deadline: Option<Instant>,
    finished: bool,
}
```

Key points:
- Polls interrupt status
- Registers waker
- Handles deadlines
- Proper cleanup in `Drop`

## Testing Strategy

### Unit Tests (Future Work)

1. **Configuration Tests:**
   - Valid/invalid addresses
   - Filter settings
   - Clock stretching options

2. **FIFO Tests:**
   - Read/write at capacity
   - Overflow/underflow conditions
   - Reset behavior

3. **Mode Switching:**
   - Blocking to Async
   - Async to Blocking
   - State preservation

### Integration Tests (Future Work)

1. **Master-Slave Communication:**
   - Basic read/write
   - Multi-byte transfers
   - Error conditions

2. **Clock Stretching:**
   - Verify timing
   - Master compatibility

3. **Address Matching:**
   - Correct responses
   - Ignore non-matching

### Hardware Tests

Required setup:
- Two ESP32 devices (one as master, one as slave)
- Or one ESP32 as slave with external I2C master
- Logic analyzer for debugging

Test cases:
1. Basic communication
2. Large data transfers
3. Clock stretching scenarios
4. Error recovery
5. Interrupt handling
6. Async operations

## Migration from ESP-IDF

If migrating from ESP-IDF's I2C slave driver:

### Key Differences

1. **Initialization:**

**ESP-IDF:**
```c
i2c_config_t conf = {
    .mode = I2C_MODE_SLAVE,
    .sda_io_num = GPIO_NUM_1,
    .scl_io_num = GPIO_NUM_2,
    .slave.addr = 0x55,
    .slave.addr_10bit_en = 0,
};
i2c_param_config(I2C_NUM_0, &conf);
i2c_driver_install(I2C_NUM_0, conf.mode, ...);
```

**esp-hal:**
```rust
let config = Config::default().with_address(0x55.into());
let i2c = I2c::new(peripherals.I2C0, config)?
    .with_sda(peripherals.GPIO1)
    .with_scl(peripherals.GPIO2);
```

2. **Reading Data:**

**ESP-IDF:**
```c
uint8_t buffer[128];
int len = i2c_slave_read_buffer(I2C_NUM_0, buffer, sizeof(buffer), 1000 / portTICK_PERIOD_MS);
```

**esp-hal:**
```rust
let mut buffer = [0u8; 128];
let len = i2c.read(&mut buffer)?;
```

3. **Writing Data:**

**ESP-IDF:**
```c
uint8_t data[] = {0xAA, 0xBB};
i2c_slave_write_buffer(I2C_NUM_0, data, sizeof(data), 1000 / portTICK_PERIOD_MS);
```

**esp-hal:**
```rust
i2c.write(&[0xAA, 0xBB])?;
```

## Performance Considerations

### Blocking Mode

- Suitable for simple applications
- Lower overhead
- Easier to debug
- May miss master requests if not polling frequently

### Async Mode

- Better for complex applications
- More efficient CPU usage
- Requires async runtime (Embassy)
- Slightly higher overhead

### Recommendations

1. **Use Blocking Mode When:**
   - Simple request-response patterns
   - Low frequency communication
   - Predictable timing

2. **Use Async Mode When:**
   - Multiple concurrent I2C slaves
   - Complex application logic
   - Need to handle other peripherals concurrently

## Troubleshooting Guide

### Common Issues

1. **Slave Not Responding:**
   - Check address configuration
   - Verify pin connections
   - Check pull-up resistors
   - Enable clock stretching if timing is tight

2. **Data Corruption:**
   - Check signal integrity
   - Adjust filter thresholds
   - Verify FIFO management
   - Check for buffer overruns

3. **Timeout Errors:**
   - Increase timeout values
   - Check bus capacitance
   - Verify clock stretching
   - Check master timing

4. **Async Not Working:**
   - Verify interrupt handler registration
   - Check executor configuration
   - Ensure waker is properly set up

### Debug Tools

1. **Logic Analyzer:**
   - Monitor SDA/SCL signals
   - Verify START/STOP conditions
   - Check ACK/NACK timing

2. **Print Debugging:**
```rust
esp_println::println!("Bytes read: {}", bytes_read);
```

3. **Interrupt Monitoring:**
```rust
let events = i2c.interrupts();
esp_println::println!("Events: {:?}", events);
```

## Future Improvements

### Short Term
1. Add more comprehensive examples
2. Add HIL (Hardware-in-Loop) tests
3. Document chip-specific behavior
4. Add benchmarks

### Medium Term
1. 10-bit addressing support
2. DMA integration
3. General call address support
4. SMBus protocol helpers

### Long Term
1. Multi-slave support on same bus
2. Hot-plug detection
3. Advanced error recovery
4. Protocol analyzer integration

## Contributing

When contributing to the slave driver:

1. **Follow the Master Driver Patterns:**
   - Similar code structure
   - Consistent naming
   - Same documentation style

2. **Test on Multiple Chips:**
   - ESP32 (if possible)
   - ESP32-S2/S3
   - ESP32-C3/C6
   - ESP32-H2

3. **Document Chip-Specific Behavior:**
   - Use conditional compilation
   - Add comments explaining differences
   - Update README.md

4. **Add Tests:**
   - Unit tests where possible
   - Integration tests with master
   - Document test setup

## Conclusion

The I2C slave driver provides a robust, well-architected solution for I2C slave operations on ESP32 chips. By following the same patterns as the master driver, it ensures consistency and maintainability while providing the unique features required for slave operation.

The driver is production-ready for basic use cases but can be extended with additional features as needed. The modular design makes it easy to add chip-specific optimizations or protocol-specific helpers without affecting the core functionality.
