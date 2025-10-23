# Interrupt-Based I2C Slave Implementation: Analysis and Design

## The Question

**Could an interrupt-based I2C slave driver resolve the async limitations by allowing other tasks to run on the same CPU core through interrupt handlers?**

**Short Answer**: Yes, theoretically. An interrupt-driven design could solve the blocking problem, but it requires significant driver changes and comes with its own challenges.

## Current Implementation Analysis

### How the Current Driver Works

The existing I2C slave driver uses a **polling + blocking** approach:

```rust
// Current blocking implementation (simplified)
pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    loop {
        // Poll status register
        if self.has_data() {
            // Read from FIFO
            for byte in buffer {
                *byte = self.read_fifo();
            }
            return Ok(());
        }
        
        // If no data, keep polling (BLOCKS!)
        if timeout_expired() {
            return Err(Error::Timeout);
        }
    }
}
```

**Problem**: The CPU is stuck in this loop, preventing other code from running.

### Why It Blocks Async Tasks

Even in an async context:

```rust
// Async wrapper around blocking implementation
pub async fn read_async(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    // Still blocking! Just wrapped in async function
    self.read(buffer)  
    // No .await points inside read() where executor could switch tasks
}
```

The executor cannot switch tasks because there are no yield points during the actual I2C waiting period.

## Interrupt-Based Design: The Solution

### Core Concept

Instead of polling, use **hardware interrupts** to notify when I2C events occur:

```rust
// Interrupt-driven design (conceptual)
pub struct I2cSlaveInterrupt<'d> {
    rx_buffer: &'static mut [u8],
    tx_buffer: &'static [u8],
    rx_waker: Option<Waker>,
    tx_waker: Option<Waker>,
    state: InterruptState,
}

// 1. User initiates read (doesn't block!)
pub async fn read_async(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    // Set up state
    self.rx_buffer = buffer;
    self.rx_waker = Some(cx.waker().clone());
    
    // Enable RX interrupt
    self.enable_rx_interrupt();
    
    // Yield to executor - OTHER TASKS CAN RUN!
    poll_fn(|cx| {
        if self.rx_complete {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending  // Executor switches to other tasks
        }
    }).await
}

// 2. Hardware interrupt fires when data arrives
#[interrupt]
fn I2C0_INTERRUPT() {
    let driver = get_i2c_driver();
    
    // Read data from FIFO into buffer (FAST!)
    while driver.has_data() {
        driver.rx_buffer[driver.rx_index] = driver.read_fifo();
        driver.rx_index += 1;
    }
    
    if driver.rx_complete() {
        // Wake the async task
        if let Some(waker) = driver.rx_waker.take() {
            waker.wake();  // Task resumes on next executor poll
        }
    }
}
```

### Key Advantages

1. **True Async**: CPU free to run other tasks while waiting for I2C
2. **Fast Response**: Interrupt fires immediately when master communicates
3. **Efficient**: No polling loops wasting CPU cycles
4. **Embassy Compatible**: Works naturally with async/await

## Detailed Design

### State Machine

```rust
enum I2cSlaveState {
    Idle,
    ReceivingAddress,
    ReceivingData { bytes_received: usize },
    TransmittingData { bytes_sent: usize },
    ClockStretching,
}

struct I2cSlaveDriver {
    state: I2cSlaveState,
    rx_buffer: Option<&'static mut [u8]>,
    tx_buffer: Option<&'static [u8]>,
    rx_waker: Option<Waker>,
    tx_waker: Option<Waker>,
    config: I2cSlaveConfig,
}
```

### Interrupt Handler Flow

```rust
#[interrupt]
fn I2C0_INTERRUPT() {
    let driver = unsafe { &mut I2C_SLAVE_DRIVER };
    let status = driver.read_interrupt_status();
    
    // Address match interrupt
    if status.address_match() {
        driver.state = I2cSlaveState::ReceivingAddress;
        driver.clear_interrupt(InterruptType::AddressMatch);
    }
    
    // RX FIFO threshold interrupt
    if status.rx_fifo_threshold() {
        match driver.state {
            I2cSlaveState::ReceivingData { ref mut bytes_received } => {
                // Read all available bytes from FIFO
                while driver.fifo_has_data() {
                    if let Some(buffer) = &mut driver.rx_buffer {
                        if *bytes_received < buffer.len() {
                            buffer[*bytes_received] = driver.read_fifo();
                            *bytes_received += 1;
                        }
                    }
                }
                
                // Check if complete
                if driver.transaction_complete() {
                    driver.state = I2cSlaveState::Idle;
                    if let Some(waker) = driver.rx_waker.take() {
                        waker.wake();  // Wake async task!
                    }
                }
            }
            _ => {}
        }
        driver.clear_interrupt(InterruptType::RxFifoThreshold);
    }
    
    // TX FIFO threshold interrupt (need to send more data)
    if status.tx_fifo_threshold() {
        match driver.state {
            I2cSlaveState::TransmittingData { ref mut bytes_sent } => {
                // Fill TX FIFO
                while driver.fifo_has_space() {
                    if let Some(buffer) = &driver.tx_buffer {
                        if *bytes_sent < buffer.len() {
                            driver.write_fifo(buffer[*bytes_sent]);
                            *bytes_sent += 1;
                        } else {
                            // All data sent
                            driver.release_scl_stretch();
                            driver.state = I2cSlaveState::Idle;
                            if let Some(waker) = driver.tx_waker.take() {
                                waker.wake();
                            }
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        driver.clear_interrupt(InterruptType::TxFifoThreshold);
    }
    
    // STOP condition interrupt
    if status.stop_detected() {
        driver.state = I2cSlaveState::Idle;
        driver.clear_interrupt(InterruptType::StopDetected);
    }
}
```

### Async API Usage

```rust
#[embassy_executor::task]
async fn i2c_handler_task(mut i2c: I2cSlaveInterrupt<'static>) {
    let mut rx_buffer = [0u8; 32];
    let tx_buffer = [0x42u8; 1];
    
    loop {
        // Read command - DOESN'T BLOCK! 
        // Other tasks run while waiting for master
        i2c.read_async(&mut rx_buffer).await.unwrap();
        
        // Process (can yield here too)
        let response = process_command(&rx_buffer).await;
        
        // Send response - DOESN'T BLOCK!
        i2c.write_async(&response).await.unwrap();
    }
}

#[embassy_executor::task]
async fn led_task() {
    loop {
        Timer::after(Duration::from_millis(500)).await;
        toggle_led();  // NOW THIS WORKS SMOOTHLY! ✓
    }
}
```

### Benefits Realized

| Feature | Blocking Driver | Current Async | Interrupt-Based Async |
|---------|----------------|---------------|----------------------|
| CPU usage while idle | 100% (polling) | 100% (polling) | ~0% (sleeping) |
| Other tasks run | ❌ No | ❌ Only during timeout | ✅ Yes, always |
| Response latency | ~µs (polling) | ~µs (polling) | <1µs (interrupt) |
| Code complexity | Low | Medium | High |
| Memory overhead | Low | Medium | Medium-High |

## Implementation Challenges

### Challenge 1: Static Lifetime Management

Interrupt handlers cannot capture context, requiring static/global state:

```rust
// Problem: Need static mut access
static mut I2C_SLAVE_DRIVER: Option<I2cSlaveDriver> = None;

// Better: Use critical section
use critical_section::Mutex;
use core::cell::RefCell;

static I2C_SLAVE: Mutex<RefCell<Option<I2cSlaveDriver>>> = 
    Mutex::new(RefCell::new(None));

#[interrupt]
fn I2C0_INTERRUPT() {
    critical_section::with(|cs| {
        if let Some(driver) = I2C_SLAVE.borrow_ref_mut(cs).as_mut() {
            driver.handle_interrupt();
        }
    });
}
```

### Challenge 2: Buffer Management

Buffers must live long enough for interrupt handler:

```rust
// Won't work - buffer dropped too soon
pub async fn read_async(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    self.set_rx_buffer(buffer);  // Danger! buffer might be on stack
    // ...
}

// Solution: Require static buffers
pub async fn read_async(
    &mut self, 
    buffer: &'static mut [u8]  // Must be static!
) -> Result<(), Error> {
    // Safe now
}

// Or use internal static buffers
static RX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
```

### Challenge 3: Clock Stretching Coordination

Must coordinate between interrupt handler and state machine:

```rust
// When to release clock stretch?
#[interrupt]
fn I2C0_INTERRUPT() {
    if tx_data_ready() {
        // Fill FIFO
        write_tx_fifo();
        
        // Release stretch - but only if data is valid!
        if buffer_ready {
            release_scl_stretch();  // Timing critical!
        }
    }
}
```

### Challenge 4: ESP32-C6 Specific Issues

The manual stretch release requirement might complicate interrupt handling:

```rust
// Current workaround (blocking context)
#[cfg(esp32c6)]
{
    wait_stabilization();  // 700K nop loop
    i2c.release_scl_stretch();
}

// In interrupt context - CAN'T DO THIS!
// Interrupt handlers must be FAST (<1µs)
// Can't spin for 10ms in interrupt!

// Possible solution: Two-stage release
#[interrupt]
fn I2C0_INTERRUPT() {
    if tx_complete() {
        // Signal that release is needed
        schedule_scl_release();  // Defer to task
    }
}

// In async task
async fn delayed_scl_release() {
    SCL_RELEASE_SIGNAL.wait().await;
    wait_stabilization();  // OK here
    i2c.release_scl_stretch();
}
```

### Challenge 5: Interrupt Priority and Nesting

ESP32 has multiple priority levels:

```rust
// Must configure interrupt priority
#[interrupt(priority = Level1)]  // Higher than embassy timer
fn I2C0_INTERRUPT() {
    // ...
}

// Risk: If processing takes too long, might miss next interrupt
// Need to keep interrupt handler MINIMAL
```

## Complete Example Design

### Driver Structure

```rust
use embassy_sync::waitqueue::AtomicWaker;

pub struct I2cSlaveInterrupt<'d> {
    _peripheral: PeripheralRef<'d, I2C0>,
    config: I2cSlaveConfig,
}

// Shared state accessible from interrupt
struct I2cSlaveState {
    rx_buffer: Option<*mut [u8]>,  // Raw pointer (unsafe but necessary)
    tx_buffer: Option<*const [u8]>,
    rx_waker: AtomicWaker,
    tx_waker: AtomicWaker,
    rx_len: AtomicUsize,
    tx_len: AtomicUsize,
    state: AtomicU8,  // State machine state
}

static I2C_STATE: I2cSlaveState = I2cSlaveState::new();

impl<'d> I2cSlaveInterrupt<'d> {
    pub fn new(
        peripheral: impl Peripheral<P = I2C0> + 'd,
        config: I2cSlaveConfig,
    ) -> Self {
        // Configure interrupts
        peripheral.enable_interrupt(Interrupt::RxFifoThreshold);
        peripheral.enable_interrupt(Interrupt::TxFifoThreshold);
        peripheral.enable_interrupt(Interrupt::AddressMatch);
        peripheral.enable_interrupt(Interrupt::StopDetected);
        
        // Bind interrupt handler
        unsafe {
            interrupt::enable(Interrupt::I2C0, Priority::Priority1).unwrap();
        }
        
        Self {
            _peripheral: peripheral.into_ref(),
            config,
        }
    }
    
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        poll_fn(|cx| {
            // Set up waker
            I2C_STATE.rx_waker.register(cx.waker());
            
            // Check if data already received
            let len = I2C_STATE.rx_len.load(Ordering::Acquire);
            if len > 0 {
                // Copy from internal buffer
                unsafe {
                    if let Some(rx_buf) = I2C_STATE.rx_buffer {
                        buffer[..len].copy_from_slice(&(*rx_buf)[..len]);
                    }
                }
                I2C_STATE.rx_len.store(0, Ordering::Release);
                Poll::Ready(Ok(len))
            } else {
                // Set buffer pointer for interrupt handler
                I2C_STATE.rx_buffer = Some(buffer as *mut [u8]);
                Poll::Pending  // Yield - other tasks can run!
            }
        }).await
    }
    
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        poll_fn(|cx| {
            I2C_STATE.tx_waker.register(cx.waker());
            
            let len = I2C_STATE.tx_len.load(Ordering::Acquire);
            if len == 0 {
                Poll::Ready(Ok(()))
            } else {
                I2C_STATE.tx_buffer = Some(buffer as *const [u8]);
                I2C_STATE.tx_len.store(buffer.len(), Ordering::Release);
                Poll::Pending
            }
        }).await
    }
}

#[interrupt]
fn I2C0_INTERRUPT() {
    // Fast interrupt handler
    let status = read_interrupt_status();
    
    if status.rx_fifo_threshold() {
        // Read data from FIFO
        let mut count = 0;
        unsafe {
            if let Some(buffer_ptr) = I2C_STATE.rx_buffer {
                let buffer = &mut *buffer_ptr;
                while fifo_has_data() && count < buffer.len() {
                    buffer[count] = read_fifo();
                    count += 1;
                }
            }
        }
        
        if transaction_complete() {
            I2C_STATE.rx_len.store(count, Ordering::Release);
            I2C_STATE.rx_buffer = None;
            I2C_STATE.rx_waker.wake();  // Wake async task!
        }
        
        clear_interrupt(Interrupt::RxFifoThreshold);
    }
    
    if status.tx_fifo_threshold() {
        // Similar for TX
        // ...
        clear_interrupt(Interrupt::TxFifoThreshold);
    }
}
```

## Performance Comparison

### Scenario: LED Blinking While Handling I2C

**Blocking Driver**:
```
Time: 0ms     - I2C read() starts, CPU blocked
Time: 2000ms  - Timeout, I2C read() returns error
Time: 2000ms  - LED task runs (missed 3 blinks!)
Time: 2001ms  - I2C read() starts again, CPU blocked
```

**Interrupt-Based Async Driver**:
```
Time: 0ms     - I2C read() starts, yield to executor
Time: 0ms     - LED task runs
Time: 500ms   - LED blinks ✓
Time: 1000ms  - LED blinks ✓
Time: 1234ms  - I2C interrupt fires, data received
Time: 1234ms  - I2C task wakes, processes data
Time: 1500ms  - LED blinks ✓
Time: 2000ms  - LED blinks ✓
```

## Implementation Effort

### Required Changes

1. **Driver Core** (~500-800 lines):
   - Interrupt handler implementation
   - State machine for tracking I2C transaction state
   - Waker management for async integration
   - Safe static state management

2. **Async API** (~200-300 lines):
   - `poll_fn` based async read/write
   - Buffer lifetime management
   - Error handling

3. **Platform Specific** (~100-200 lines per chip):
   - ESP32-C6 stretch release workaround
   - Interrupt priority configuration
   - FIFO threshold tuning

4. **Testing** (~1000+ lines):
   - Interrupt timing tests
   - Concurrent task tests
   - Stress tests (fast master)
   - Edge cases (FIFO overflow, etc.)

**Total Estimate**: 2-3 weeks for experienced embedded developer

## Recommendations

### When to Use Interrupt-Based Design

✅ **Use if**:
- Need true concurrent tasks on same core
- I2C communication is sporadic (not continuous)
- Response time is critical
- Battery-powered (sleep between transactions)

❌ **Don't use if**:
- Simple request/response pattern
- I2C is the only function
- Development time is limited
- Code simplicity is priority

### Alternative: Multi-Core Architecture

For ESP32 (dual-core), consider dedicating cores:

```rust
// Core 0: I2C slave (blocking, simple)
#[entry]
fn main() -> ! {
    let i2c = I2c::new_blocking(/* ... */);
    loop {
        i2c.read(&mut buffer).unwrap();
        i2c.write(&response).unwrap();
    }
}

// Core 1: Async tasks (LED, sensors, etc.)
#[embassy_executor::task]
async fn core1_main(spawner: Spawner) {
    spawner.spawn(led_task()).unwrap();
    spawner.spawn(sensor_task()).unwrap();
    // ...
}
```

**Note**: ESP32-C6 is single-core, so this won't work there.

## Conclusion

**Yes, an interrupt-based I2C slave driver WOULD solve the async limitations**, allowing true concurrent task execution on a single core. The CPU would be free to run other tasks while waiting for I2C communication, and interrupt handlers would provide immediate response when the master communicates.

**However**, the implementation is significantly more complex than the current polling-based driver:

| Aspect | Blocking Driver | Interrupt Driver |
|--------|----------------|------------------|
| Code complexity | ⭐ Simple | ⭐⭐⭐⭐ Complex |
| Memory overhead | ⭐ Low | ⭐⭐⭐ Medium |
| Response time | ⭐⭐⭐ Good (~µs) | ⭐⭐⭐⭐⭐ Excellent (<1µs) |
| Power efficiency | ⭐⭐ Poor (polling) | ⭐⭐⭐⭐⭐ Excellent (sleep) |
| Async benefits | ⭐ None | ⭐⭐⭐⭐⭐ Full |
| Debug difficulty | ⭐ Easy | ⭐⭐⭐⭐ Hard |

**For most applications**, the blocking driver or channel-based async pattern is sufficient. **For battery-powered or highly concurrent applications**, an interrupt-based driver would be worth the implementation effort.

---

*Last updated: October 2025*  
*Analysis based on: ESP32-C6, esp-hal 0.22.0*
