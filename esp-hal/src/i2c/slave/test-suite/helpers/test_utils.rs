//! Test utility functions
//!
//! Common utilities used across multiple test modules.

use core::time::Duration;

/// Test timeout for operations
pub const TEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Short delay between operations
pub const SHORT_DELAY_MS: u32 = 10;

/// Standard test buffer size
pub const TEST_BUFFER_SIZE: usize = 64;

/// Compare two byte slices with detailed error message
#[cfg(test)]
pub fn assert_bytes_equal(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{}: Length mismatch. Expected {} bytes, got {}",
        context,
        expected.len(),
        actual.len()
    );

    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            a, e,
            "{}: Byte mismatch at index {}. Expected 0x{:02X}, got 0x{:02X}",
            context, i, e, a
        );
    }
}

/// Generate test pattern data
#[cfg(test)]
pub fn generate_test_pattern(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i & 0xFF) as u8).collect()
}

/// Generate sequential test data
#[cfg(test)]
pub fn generate_sequential(start: u8, count: usize) -> Vec<u8> {
    (0..count).map(|i| start.wrapping_add(i as u8)).collect()
}

/// Generate random-looking test data (deterministic)
#[cfg(test)]
pub fn generate_pseudo_random(seed: u8, count: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(count);
    let mut value = seed;
    for _ in 0..count {
        value = value.wrapping_mul(13).wrapping_add(7);
        data.push(value);
    }
    data
}

/// Verify buffer contains expected pattern
#[cfg(test)]
pub fn verify_pattern(buffer: &[u8], start: u8, context: &str) -> bool {
    for (i, &byte) in buffer.iter().enumerate() {
        let expected = start.wrapping_add(i as u8);
        if byte != expected {
            panic!(
                "{}: Pattern mismatch at index {}. Expected 0x{:02X}, got 0x{:02X}",
                context, i, expected, byte
            );
        }
    }
    true
}

/// Timing helper for performance tests
#[cfg(test)]
pub struct Timer {
    start: Option<core::time::Duration>,
}

#[cfg(test)]
impl Timer {
    pub fn new() -> Self {
        Self { start: None }
    }

    pub fn start(&mut self) {
        // In actual implementation, use Instant::now()
        self.start = Some(core::time::Duration::from_millis(0));
    }

    pub fn elapsed(&self) -> core::time::Duration {
        // In actual implementation, calculate from Instant::now()
        core::time::Duration::from_millis(0)
    }

    pub fn reset(&mut self) {
        self.start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_pattern() {
        let pattern = generate_test_pattern(10);
        assert_eq!(pattern.len(), 10);
        assert_eq!(pattern[0], 0);
        assert_eq!(pattern[9], 9);
    }

    #[test]
    fn test_generate_sequential() {
        let seq = generate_sequential(0x50, 5);
        assert_eq!(seq, vec![0x50, 0x51, 0x52, 0x53, 0x54]);
    }

    #[test]
    fn test_generate_sequential_wrapping() {
        let seq = generate_sequential(0xFE, 3);
        assert_eq!(seq, vec![0xFE, 0xFF, 0x00]);
    }

    #[test]
    fn test_generate_pseudo_random() {
        let data1 = generate_pseudo_random(42, 10);
        let data2 = generate_pseudo_random(42, 10);
        
        // Should be deterministic
        assert_eq!(data1, data2);
        
        // Should not be all same value
        let first = data1[0];
        assert!(data1.iter().any(|&x| x != first));
    }

    #[test]
    fn test_assert_bytes_equal_success() {
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        assert_bytes_equal(&a, &b, "test");
    }

    #[test]
    #[should_panic(expected = "Length mismatch")]
    fn test_assert_bytes_equal_length_fail() {
        let a = [1, 2, 3];
        let b = [1, 2];
        assert_bytes_equal(&a, &b, "test");
    }

    #[test]
    #[should_panic(expected = "Byte mismatch")]
    fn test_assert_bytes_equal_content_fail() {
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        assert_bytes_equal(&a, &b, "test");
    }

    #[test]
    fn test_verify_pattern_success() {
        let buffer = [5, 6, 7, 8];
        assert!(verify_pattern(&buffer, 5, "test"));
    }

    #[test]
    #[should_panic(expected = "Pattern mismatch")]
    fn test_verify_pattern_fail() {
        let buffer = [5, 6, 99, 8];
        verify_pattern(&buffer, 5, "test");
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new();
        timer.start();
        let _ = timer.elapsed();
        timer.reset();
    }
}
