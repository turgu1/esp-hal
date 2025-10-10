# Timer Service Documentation

## Overview

The Timer Service provides comprehensive timing capabilities for the Zigbee protocol stack, including:

- **Monotonic timestamps**: Millisecond-precision time tracking
- **Scheduled timers**: One-shot and periodic timers for protocol operations
- **Timeout tracking**: Automatic timeout detection for associations, route discovery, etc.
- **Rate limiting**: Control frequency of periodic operations

## Architecture

```
┌─────────────────────────────────────────────┐
│          Application Layer                   │
│  (Uses timer service for custom timing)     │
└─────────────────────────────────────────────┘
                    ↓ ↑
┌─────────────────────────────────────────────┐
│          Timer Service                       │
│  • Timestamp generation (ms precision)      │
│  • Scheduled timers (16 max)                │
│  • Expiry detection                          │
│  • Timer cancellation                        │
└─────────────────────────────────────────────┘
                    ↓ ↑
┌─────────────────────────────────────────────┐
│          Zigbee Protocol Stack               │
│  • Association timeouts                      │
│  • Route discovery timeouts                  │
│  • Route aging (periodic)                    │
│  • Fragment reassembly timeouts              │
│  • Permit joining timeouts                   │
│  • Link status updates (periodic)            │
└─────────────────────────────────────────────┘
                    ↓ ↑
┌─────────────────────────────────────────────┐
│          ESP-HAL Time Infrastructure         │
│  (Instant::now() for monotonic time)        │
└─────────────────────────────────────────────┘
```

## Components

### 1. TimerService

The main timer service manages scheduled timers and monotonic timestamps.

**Key Features:**
- Monotonic timestamp in milliseconds
- Up to 16 concurrent scheduled timers
- Automatic expiry detection
- One-shot and periodic timers
- Timer cancellation

**API:**

```rust
use esp_hal::zigbee::{TimerService, TimerType};
use esp_hal::time::Duration;

// Get timer service from Zigbee driver
let timer_service = zigbee.timer_service_mut();

// Schedule a one-shot timer (e.g., association timeout)
let timer_id = timer_service.schedule_oneshot(
    Duration::from_secs(5),
    TimerType::AssociationTimeout,
)?;

// Schedule a periodic timer (e.g., route aging)
let timer_id = timer_service.schedule_periodic(
    Duration::from_secs(60),
    TimerType::RouteAging,
)?;

// Cancel a timer
timer_service.cancel_timer(timer_id);

// Get current timestamp
let now_ms = timer_service.now_ms();
```

### 2. Timer Types

The timer service supports multiple timer types for different protocol operations:

| Timer Type | Description | Typical Duration | Usage |
|------------|-------------|------------------|-------|
| `AssociationTimeout` | MAC association timeout | 5 seconds | Detect failed associations |
| `RouteDiscoveryTimeout` | Route discovery timeout | 10 seconds | RREQ/RREP timeout |
| `RouteAging` | Route table aging | 60 seconds | Expire old routes |
| `FragmentTimeout` | Fragment reassembly timeout | 10 seconds | Clean up partial messages |
| `PermitJoiningTimeout` | Permit joining duration | 1-254 seconds | Auto-disable joining |
| `PollRate` | End device poll rate | Variable | Sleepy device polling |
| `LinkStatusUpdate` | Link status broadcast | 30 seconds | Neighbor management |
| `NetworkFormationTimeout` | Network formation timeout | 10 seconds | Formation failure |
| `OneShot` | Generic one-shot timer | Variable | Application use |
| `Periodic` | Generic periodic timer | Variable | Application use |

### 3. TimeoutTracker

Utility for tracking individual operation timeouts.

**Usage:**

```rust
use esp_hal::zigbee::TimeoutTracker;
use esp_hal::time::Duration;

// Create timeout tracker
let tracker = TimeoutTracker::new(
    timer_service.now_ms(),
    Duration::from_secs(5),
);

// Check if expired
loop {
    let current_ms = timer_service.now_ms();
    if tracker.is_expired(current_ms) {
        // Timeout occurred
        break;
    }
    
    // Check remaining time
    let remaining = tracker.remaining_ms(current_ms);
    if remaining < 1000 {
        // Less than 1 second remaining
    }
}
```

### 4. RateLimiter

Utility for rate-limiting periodic operations.

**Usage:**

```rust
use esp_hal::zigbee::RateLimiter;
use esp_hal::time::Duration;

// Create rate limiter (max once per second)
let mut limiter = RateLimiter::new(Duration::from_secs(1));

loop {
    let current_ms = timer_service.now_ms();
    
    if limiter.can_proceed(current_ms) {
        // Execute rate-limited operation
        send_link_status();
    }
}
```

## Integration with Zigbee Driver

The timer service is automatically integrated into the Zigbee driver and handles protocol timing requirements:

### Automatic Timer Management

The driver automatically manages timers for:

1. **Association Timeout** (5 seconds)
   - Triggered when device association takes too long
   - Generates `NetworkError::AssociationFailed` event

2. **Route Discovery Timeout** (10 seconds)
   - Triggered when route discovery doesn't complete
   - Generates `NetworkError::RouteDiscoveryFailed` event

3. **Route Aging** (every 60 seconds)
   - Periodically ages routing table entries
   - Removes stale routes (>300 seconds old)

4. **Fragment Timeout** (10 seconds)
   - Cleans up incomplete fragmented messages
   - Prevents memory leaks from partial messages

5. **Permit Joining** (1-254 seconds)
   - Automatically disables permit joining after duration
   - Enforces security by limiting join window

### Timer Service in Poll Loop

The timer service is automatically updated in the `poll()` method:

```rust
pub fn poll(&mut self) -> Option<ZigbeeEvent> {
    // Update timer service and check for expired timers
    let now = Instant::now();
    let expired_timers = self.inner.timer_service.update(now);
    
    // Process expired timers
    for (timer_id, timer_type) in expired_timers {
        self.handle_timer_expiry(timer_id, timer_type);
    }
    
    // ... rest of poll logic
}
```

## Usage Examples

### Example 1: Schedule Association Timeout

```rust
use esp_hal::zigbee::{Zigbee, TimerType};
use esp_hal::time::Duration;

let mut zigbee = Zigbee::new(radio, config);

// Start association
zigbee.join_network()?;

// Schedule association timeout
let timer_service = zigbee.timer_service_mut();
let timer_id = timer_service.schedule_oneshot(
    Duration::from_secs(5),
    TimerType::AssociationTimeout,
)?;

// Poll for events
loop {
    if let Some(event) = zigbee.poll() {
        match event {
            ZigbeeEvent::NetworkJoined { .. } => {
                // Success! Cancel timeout
                zigbee.timer_service_mut().cancel_timer(timer_id);
                break;
            }
            ZigbeeEvent::NetworkError { error } => {
                // Timeout or other error
                break;
            }
            _ => {}
        }
    }
}
```

### Example 2: Periodic Link Status

```rust
use esp_hal::zigbee::{Zigbee, TimerType};
use esp_hal::time::Duration;

let mut zigbee = Zigbee::new(radio, config);
zigbee.form_network()?;

// Schedule periodic link status updates (every 30 seconds)
let timer_service = zigbee.timer_service_mut();
timer_service.schedule_periodic(
    Duration::from_secs(30),
    TimerType::LinkStatusUpdate,
)?;

// Poll loop
loop {
    if let Some(event) = zigbee.poll() {
        // Link status updates happen automatically
    }
}
```

### Example 3: Custom Application Timer

```rust
use esp_hal::zigbee::{Zigbee, TimerType};
use esp_hal::time::Duration;

let mut zigbee = Zigbee::new(radio, config);

// Schedule custom periodic timer for sensor reading
let timer_id = zigbee.timer_service_mut().schedule_periodic(
    Duration::from_secs(60), // Every 60 seconds
    TimerType::Periodic,
)?;

loop {
    if let Some(event) = zigbee.poll() {
        // Handle events
    }
    
    // Check timer service for expired timers
    // (Automatically done in poll, but can also check manually)
}
```

### Example 4: Timeout with Fallback

```rust
use esp_hal::zigbee::{TimeoutTracker, Zigbee};
use esp_hal::time::Duration;

let mut zigbee = Zigbee::new(radio, config);

// Create timeout tracker for route discovery
let timeout = TimeoutTracker::new(
    zigbee.timer_service().now_ms(),
    Duration::from_secs(10),
);

// Attempt route discovery with timeout
loop {
    let current_ms = zigbee.timer_service().now_ms();
    
    if timeout.is_expired(current_ms) {
        // Timeout - use default route or fail
        break;
    }
    
    if let Some(event) = zigbee.poll() {
        match event {
            ZigbeeEvent::RouteDiscovered { .. } => {
                // Success!
                break;
            }
            _ => {}
        }
    }
}
```

### Example 5: Rate-Limited Broadcasts

```rust
use esp_hal::zigbee::{RateLimiter, Zigbee};
use esp_hal::time::Duration;

let mut zigbee = Zigbee::new(radio, config);

// Rate limit broadcasts to once per second
let mut rate_limiter = RateLimiter::new(Duration::from_secs(1));

loop {
    let current_ms = zigbee.timer_service().now_ms();
    
    if rate_limiter.can_proceed(current_ms) {
        // Send broadcast (rate-limited)
        zigbee.send_data(0xFFFF, b"Broadcast")?;
    }
    
    // Handle events
    zigbee.poll();
}
```

## Performance Characteristics

### Memory Usage

```
Component              Size
────────────────────────────
TimerService           ~120 bytes
  - ScheduledTimer     ~16 bytes each
  - Max 16 timers      ~256 bytes total
TimeoutTracker         ~8 bytes
RateLimiter            ~8 bytes
────────────────────────────
Total (typical)        ~400 bytes
```

### Timing Precision

- **Resolution**: 1 millisecond
- **Accuracy**: Depends on poll loop frequency
  - Poll every 10ms → ~10ms timer accuracy
  - Poll every 100ms → ~100ms timer accuracy
- **Recommended poll rate**: 10-50ms for good timer accuracy

### Timer Limits

- **Maximum concurrent timers**: 16
- **Timer ID wrap**: 65,535 (2^16-1)
- **Timestamp wrap**: ~49 days (2^32 milliseconds)
  - Wrapping is handled correctly by comparison logic

## Best Practices

### 1. Poll Frequently

Timer accuracy depends on poll frequency:

```rust
loop {
    zigbee.poll(); // Poll frequently for accurate timers
    delay_ms(10);  // 10ms polling → ±10ms timer accuracy
}
```

### 2. Clean Up Timers

Always cancel timers when no longer needed:

```rust
let timer_id = timer_service.schedule_oneshot(timeout, timer_type)?;

// Later, when operation completes:
timer_service.cancel_timer(timer_id);
```

### 3. Use Appropriate Timer Types

Use specific timer types for protocol operations:

```rust
// Good: Specific timer type
timer_service.schedule_oneshot(
    Duration::from_secs(5),
    TimerType::AssociationTimeout,
)?;

// Avoid: Generic timer for protocol operation
timer_service.schedule_oneshot(
    Duration::from_secs(5),
    TimerType::OneShot, // Less clear what this is for
)?;
```

### 4. Handle Timer Expiry

Process timer expiry events appropriately:

```rust
loop {
    if let Some(event) = zigbee.poll() {
        match event {
            ZigbeeEvent::NetworkError { error } => {
                match error {
                    NetworkError::AssociationFailed => {
                        // Handle association timeout
                    }
                    NetworkError::RouteDiscoveryFailed => {
                        // Handle route discovery timeout
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
```

### 5. Rate Limit Broadcasts

Use RateLimiter to prevent flooding:

```rust
let mut limiter = RateLimiter::new(Duration::from_secs(1));

// In poll loop:
if limiter.can_proceed(timer_service.now_ms()) {
    send_broadcast(); // Rate-limited to once per second
}
```

## Troubleshooting

### Timers Not Expiring

**Problem**: Scheduled timers never expire

**Causes:**
- Not calling `poll()` frequently enough
- Timer service not initialized
- Incorrect timeout duration

**Solutions:**
```rust
// Ensure poll is called frequently
loop {
    zigbee.poll();
    delay_ms(10); // Don't delay too long
}

// Check timer service is initialized
let timer_service = zigbee.timer_service();
assert!(timer_service.now_ms() > 0);

// Verify timer is scheduled
let timer_id = timer_service.schedule_oneshot(timeout, timer_type)?;
assert!(timer_service.is_timer_active(timer_id));
```

### Timer Accuracy Issues

**Problem**: Timers expire at wrong time

**Causes:**
- Infrequent polling
- Long operations in poll loop
- System time issues

**Solutions:**
```rust
// Increase poll frequency
loop {
    zigbee.poll();
    delay_ms(10); // Shorter delay = better accuracy
}

// Avoid long operations in poll loop
loop {
    let event = zigbee.poll();
    
    // Process quickly
    handle_event(event);
    
    // Don't block for long periods
}
```

### Too Many Timers

**Problem**: `TimerError::TooManyTimers` when scheduling

**Causes:**
- Not canceling completed timers
- Too many concurrent operations
- Memory constraints

**Solutions:**
```rust
// Cancel timers when done
timer_service.cancel_timer(timer_id);

// Use timer types to manage groups
timer_service.cancel_timers_by_type(TimerType::AssociationTimeout);

// Check active count
if timer_service.active_timer_count() < 16 {
    timer_service.schedule_oneshot(timeout, timer_type)?;
}
```

## Future Enhancements

Planned improvements to the timer service:

1. **Hardware Timer Integration**
   - Use ESP32-C6/H2 hardware timers for precise timing
   - Reduce CPU overhead
   - Improve accuracy

2. **Timer Priorities**
   - Critical timers execute first
   - Application timers have lower priority

3. **Timer Statistics**
   - Track timer usage
   - Measure accuracy
   - Detect missed deadlines

4. **Async/Await Integration**
   - Timer futures for async operations
   - Timeout combinators

## References

- **esp-hal timer module**: Hardware timer integration
- **Zigbee Specification R22**: Protocol timing requirements
- **IEEE 802.15.4**: MAC layer timing
- **AODV RFC 3561**: Routing protocol timeouts

## Conclusion

The Timer Service provides a robust, efficient timing infrastructure for the Zigbee protocol stack. With millisecond precision, automatic timer management, and flexible scheduling options, it handles all timing requirements from short protocol timeouts to long-term route aging.
