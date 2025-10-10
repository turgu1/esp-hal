//! Timer Service for Zigbee Driver
//!
//! Provides timing services for the Zigbee protocol stack including:
//! - Monotonic timestamp generation
//! - Timeout tracking for associations, route discovery, etc.
//! - Periodic event scheduling
//! - Route aging and maintenance
//! - Fragment timeout management
//!
//! The timer service uses the esp-hal TIMG (Timer Group) peripheral
//! for high-resolution timing with microsecond precision.

use crate::time::{Duration, Instant};
use heapless::Vec;

/// Timer service for Zigbee operations
pub struct TimerService {
    /// Monotonic timestamp in milliseconds
    timestamp_ms: u32,
    
    /// Last instant when timestamp was updated
    last_update: Option<Instant>,
    
    /// Scheduled timers
    timers: Vec<ScheduledTimer, 16>,
    
    /// Next timer ID
    next_timer_id: u16,
}

/// A scheduled timer callback
#[derive(Debug, Clone)]
pub struct ScheduledTimer {
    /// Timer ID
    pub id: u16,
    
    /// Timer type
    pub timer_type: TimerType,
    
    /// Expiration time (milliseconds since start)
    pub expiry_ms: u32,
    
    /// Interval for periodic timers (milliseconds)
    pub interval_ms: Option<u32>,
}

/// Timer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    /// Association timeout
    AssociationTimeout,
    
    /// Route discovery timeout
    RouteDiscoveryTimeout,
    
    /// Route aging (periodic)
    RouteAging,
    
    /// Fragment reassembly timeout
    FragmentTimeout,
    
    /// Permit joining timeout
    PermitJoiningTimeout,
    
    /// Poll rate timer (for sleepy end devices)
    PollRate,
    
    /// Link status update (periodic)
    LinkStatusUpdate,
    
    /// Network formation timeout
    NetworkFormationTimeout,
    
    /// Generic one-shot timer
    OneShot,
    
    /// Generic periodic timer
    Periodic,
}

impl TimerService {
    /// Create a new timer service
    pub fn new() -> Self {
        Self {
            timestamp_ms: 0,
            last_update: None,
            timers: Vec::new(),
            next_timer_id: 1,
        }
    }
    
    /// Initialize the timer service with a reference instant
    pub fn init(&mut self, now: Instant) {
        self.last_update = Some(now);
        self.timestamp_ms = 0;
    }
    
    /// Update the timer service
    /// 
    /// Should be called periodically (e.g., in the main event loop)
    /// Returns a list of expired timer IDs
    pub fn update(&mut self, now: Instant) -> Vec<(u16, TimerType), 16> {
        // Update timestamp
        if let Some(last) = self.last_update {
            let elapsed = now.duration_since(last);
            self.timestamp_ms = self.timestamp_ms.wrapping_add(elapsed.as_millis() as u32);
        }
        self.last_update = Some(now);
        
        // Check for expired timers
        let mut expired = Vec::new();
        let mut i = 0;
        
        while i < self.timers.len() {
            let timer = &self.timers[i];
            
            // Check if timer has expired
            if self.timestamp_ms.wrapping_sub(timer.expiry_ms) < 0x80000000 {
                // Timer expired
                let id = timer.id;
                let timer_type = timer.timer_type;
                let interval = timer.interval_ms;
                
                // For periodic timers, reschedule
                if let Some(interval_ms) = interval {
                    self.timers[i].expiry_ms = self.timestamp_ms.wrapping_add(interval_ms);
                    expired.push((id, timer_type)).ok();
                    i += 1;
                } else {
                    // One-shot timer, remove it
                    self.timers.swap_remove(i);
                    expired.push((id, timer_type)).ok();
                    // Don't increment i since we removed an element
                }
            } else {
                i += 1;
            }
        }
        
        expired
    }
    
    /// Get current timestamp in milliseconds
    pub fn now_ms(&self) -> u32 {
        self.timestamp_ms
    }
    
    /// Get current timestamp in seconds
    pub fn now_secs(&self) -> u32 {
        self.timestamp_ms / 1000
    }
    
    /// Schedule a one-shot timer
    /// 
    /// Returns the timer ID if successful
    pub fn schedule_oneshot(
        &mut self,
        timeout: Duration,
        timer_type: TimerType,
    ) -> Result<u16, TimerError> {
        let timeout_ms = timeout.as_millis() as u32;
        let expiry_ms = self.timestamp_ms.wrapping_add(timeout_ms);
        
        let id = self.next_timer_id;
        self.next_timer_id = self.next_timer_id.wrapping_add(1);
        
        let timer = ScheduledTimer {
            id,
            timer_type,
            expiry_ms,
            interval_ms: None,
        };
        
        self.timers.push(timer).map_err(|_| TimerError::TooManyTimers)?;
        
        Ok(id)
    }
    
    /// Schedule a periodic timer
    /// 
    /// Returns the timer ID if successful
    pub fn schedule_periodic(
        &mut self,
        interval: Duration,
        timer_type: TimerType,
    ) -> Result<u16, TimerError> {
        let interval_ms = interval.as_millis() as u32;
        let expiry_ms = self.timestamp_ms.wrapping_add(interval_ms);
        
        let id = self.next_timer_id;
        self.next_timer_id = self.next_timer_id.wrapping_add(1);
        
        let timer = ScheduledTimer {
            id,
            timer_type,
            expiry_ms,
            interval_ms: Some(interval_ms),
        };
        
        self.timers.push(timer).map_err(|_| TimerError::TooManyTimers)?;
        
        Ok(id)
    }
    
    /// Cancel a timer
    pub fn cancel_timer(&mut self, timer_id: u16) -> bool {
        if let Some(index) = self.timers.iter().position(|t| t.id == timer_id) {
            self.timers.swap_remove(index);
            true
        } else {
            false
        }
    }
    
    /// Cancel all timers of a specific type
    pub fn cancel_timers_by_type(&mut self, timer_type: TimerType) -> usize {
        let before = self.timers.len();
        self.timers.retain(|t| t.timer_type != timer_type);
        before - self.timers.len()
    }
    
    /// Check if a timer is active
    pub fn is_timer_active(&self, timer_id: u16) -> bool {
        self.timers.iter().any(|t| t.id == timer_id)
    }
    
    /// Get time until timer expires (in milliseconds)
    pub fn time_until_expiry(&self, timer_id: u16) -> Option<u32> {
        self.timers.iter()
            .find(|t| t.id == timer_id)
            .map(|t| t.expiry_ms.wrapping_sub(self.timestamp_ms))
    }
    
    /// Get number of active timers
    pub fn active_timer_count(&self) -> usize {
        self.timers.len()
    }
    
    /// Clear all timers
    pub fn clear_all(&mut self) {
        self.timers.clear();
    }
}

impl Default for TimerService {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// Too many timers scheduled
    TooManyTimers,
    
    /// Timer not found
    TimerNotFound,
    
    /// Invalid duration
    InvalidDuration,
}

impl core::fmt::Display for TimerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooManyTimers => write!(f, "Too many timers scheduled"),
            Self::TimerNotFound => write!(f, "Timer not found"),
            Self::InvalidDuration => write!(f, "Invalid timer duration"),
        }
    }
}

impl core::error::Error for TimerError {}

/// Timeout tracker for protocol operations
pub struct TimeoutTracker {
    /// Start time in milliseconds
    start_ms: u32,
    
    /// Timeout duration in milliseconds
    timeout_ms: u32,
}

impl TimeoutTracker {
    /// Create a new timeout tracker
    pub fn new(start_ms: u32, timeout: Duration) -> Self {
        Self {
            start_ms,
            timeout_ms: timeout.as_millis() as u32,
        }
    }
    
    /// Check if the timeout has expired
    pub fn is_expired(&self, current_ms: u32) -> bool {
        current_ms.wrapping_sub(self.start_ms) >= self.timeout_ms
    }
    
    /// Get remaining time in milliseconds
    pub fn remaining_ms(&self, current_ms: u32) -> u32 {
        let elapsed = current_ms.wrapping_sub(self.start_ms);
        if elapsed >= self.timeout_ms {
            0
        } else {
            self.timeout_ms - elapsed
        }
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self, current_ms: u32) -> u32 {
        current_ms.wrapping_sub(self.start_ms)
    }
}

/// Rate limiter for periodic operations
pub struct RateLimiter {
    /// Last execution time in milliseconds
    last_ms: u32,
    
    /// Minimum interval in milliseconds
    interval_ms: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(interval: Duration) -> Self {
        Self {
            last_ms: 0,
            interval_ms: interval.as_millis() as u32,
        }
    }
    
    /// Check if operation can proceed
    pub fn can_proceed(&mut self, current_ms: u32) -> bool {
        let elapsed = current_ms.wrapping_sub(self.last_ms);
        if elapsed >= self.interval_ms {
            self.last_ms = current_ms;
            true
        } else {
            false
        }
    }
    
    /// Reset the rate limiter
    pub fn reset(&mut self, current_ms: u32) {
        self.last_ms = current_ms;
    }
    
    /// Get time until next allowed operation (milliseconds)
    pub fn time_until_ready(&self, current_ms: u32) -> u32 {
        let elapsed = current_ms.wrapping_sub(self.last_ms);
        if elapsed >= self.interval_ms {
            0
        } else {
            self.interval_ms - elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer_service_basic() {
        let mut service = TimerService::new();
        let now = Instant::from_millis(0);
        service.init(now);
        
        assert_eq!(service.now_ms(), 0);
        assert_eq!(service.active_timer_count(), 0);
    }
    
    #[test]
    fn test_schedule_oneshot() {
        let mut service = TimerService::new();
        let now = Instant::from_millis(0);
        service.init(now);
        
        let timer_id = service.schedule_oneshot(
            Duration::from_millis(100),
            TimerType::AssociationTimeout,
        ).unwrap();
        
        assert_eq!(service.active_timer_count(), 1);
        assert!(service.is_timer_active(timer_id));
    }
    
    #[test]
    fn test_timer_expiry() {
        let mut service = TimerService::new();
        let now = Instant::from_millis(0);
        service.init(now);
        
        service.schedule_oneshot(
            Duration::from_millis(100),
            TimerType::AssociationTimeout,
        ).unwrap();
        
        // Update to 50ms - should not expire
        let now = Instant::from_millis(50);
        let expired = service.update(now);
        assert_eq!(expired.len(), 0);
        assert_eq!(service.active_timer_count(), 1);
        
        // Update to 150ms - should expire
        let now = Instant::from_millis(150);
        let expired = service.update(now);
        assert_eq!(expired.len(), 1);
        assert_eq!(service.active_timer_count(), 0);
    }
    
    #[test]
    fn test_periodic_timer() {
        let mut service = TimerService::new();
        let now = Instant::from_millis(0);
        service.init(now);
        
        service.schedule_periodic(
            Duration::from_millis(100),
            TimerType::RouteAging,
        ).unwrap();
        
        // First expiry at 100ms
        let now = Instant::from_millis(100);
        let expired = service.update(now);
        assert_eq!(expired.len(), 1);
        assert_eq!(service.active_timer_count(), 1); // Still active
        
        // Second expiry at 200ms
        let now = Instant::from_millis(200);
        let expired = service.update(now);
        assert_eq!(expired.len(), 1);
        assert_eq!(service.active_timer_count(), 1); // Still active
    }
    
    #[test]
    fn test_cancel_timer() {
        let mut service = TimerService::new();
        let now = Instant::from_millis(0);
        service.init(now);
        
        let timer_id = service.schedule_oneshot(
            Duration::from_millis(100),
            TimerType::AssociationTimeout,
        ).unwrap();
        
        assert_eq!(service.active_timer_count(), 1);
        assert!(service.cancel_timer(timer_id));
        assert_eq!(service.active_timer_count(), 0);
    }
    
    #[test]
    fn test_timeout_tracker() {
        let tracker = TimeoutTracker::new(0, Duration::from_millis(100));
        
        assert!(!tracker.is_expired(50));
        assert!(tracker.is_expired(100));
        assert!(tracker.is_expired(150));
        
        assert_eq!(tracker.remaining_ms(50), 50);
        assert_eq!(tracker.remaining_ms(100), 0);
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(Duration::from_millis(100));
        
        // First call should succeed
        assert!(limiter.can_proceed(0));
        
        // Immediate second call should fail
        assert!(!limiter.can_proceed(0));
        
        // Call after interval should succeed
        assert!(limiter.can_proceed(100));
        
        // Check time until ready
        assert_eq!(limiter.time_until_ready(150), 50);
    }
}
