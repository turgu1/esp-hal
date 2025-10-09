//! Common master test utilities
//!
//! Shared utilities for all master test support code.

use esp_hal::{
    i2c::master::{Config as MasterConfig, I2c as MasterI2c},
    peripheral::Peripheral,
    gpio::{GpioPin, OutputPin, InputPin},
};

/// Standard I2C master configuration for testing
pub struct TestMasterConfig {
    pub slave_address: u8,
    pub frequency: u32,
    pub timeout_ms: u32,
}

impl Default for TestMasterConfig {
    fn default() -> Self {
        Self {
            slave_address: 0x55,
            frequency: 100_000, // 100 kHz
            timeout_ms: 1000,
        }
    }
}

impl TestMasterConfig {
    pub fn with_slave_address(mut self, addr: u8) -> Self {
        self.slave_address = addr;
        self
    }

    pub fn with_frequency(mut self, freq: u32) -> Self {
        self.frequency = freq;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn fast_mode() -> Self {
        Self {
            slave_address: 0x55,
            frequency: 400_000, // 400 kHz
            timeout_ms: 1000,
        }
    }

    pub fn fast_mode_plus() -> Self {
        Self {
            slave_address: 0x55,
            frequency: 1_000_000, // 1 MHz
            timeout_ms: 500,
        }
    }
}

/// Test master wrapper for I2C operations
pub struct TestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: MasterI2c<'d, T, esp_hal::Blocking>,
    slave_address: u8,
}

impl<'d, T> TestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    /// Create a new test master
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        config: TestMasterConfig,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let master_config = MasterConfig::default()
            .with_frequency(config.frequency.Hz());

        let master = MasterI2c::new(peripheral, master_config)?
            .with_sda(sda)
            .with_scl(scl);

        Ok(Self {
            master,
            slave_address: config.slave_address,
        })
    }

    /// Write data to slave
    pub fn write(&mut self, data: &[u8]) -> Result<(), esp_hal::i2c::Error> {
        self.master.write(self.slave_address, data)
    }

    /// Read data from slave
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), esp_hal::i2c::Error> {
        self.master.read(self.slave_address, buffer)
    }

    /// Write then read (repeated start)
    pub fn write_read(
        &mut self,
        write_data: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), esp_hal::i2c::Error> {
        self.master.write_read(self.slave_address, write_data, read_buffer)
    }

    /// Change slave address for next operation
    pub fn set_slave_address(&mut self, addr: u8) {
        self.slave_address = addr;
    }

    /// Get current slave address
    pub fn slave_address(&self) -> u8 {
        self.slave_address
    }
}

/// Data pattern generators for testing
pub mod patterns {
    /// Generate sequential pattern: 0, 1, 2, 3, ...
    pub fn sequential(buffer: &mut [u8], start: u8) {
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = start.wrapping_add(i as u8);
        }
    }

    /// Generate constant pattern: all same value
    pub fn constant(buffer: &mut [u8], value: u8) {
        buffer.fill(value);
    }

    /// Generate alternating pattern: 0xAA, 0x55, 0xAA, 0x55, ...
    pub fn alternating(buffer: &mut [u8]) {
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = if i % 2 == 0 { 0xAA } else { 0x55 };
        }
    }

    /// Generate pseudo-random pattern (deterministic)
    pub fn pseudo_random(buffer: &mut [u8], seed: u8) {
        let mut state = seed;
        for byte in buffer.iter_mut() {
            // Simple PRNG
            state = state.wrapping_mul(37).wrapping_add(17);
            *byte = state;
        }
    }

    /// Verify buffer matches expected pattern
    pub fn verify_sequential(buffer: &[u8], start: u8) -> bool {
        buffer.iter().enumerate().all(|(i, &byte)| {
            byte == start.wrapping_add(i as u8)
        })
    }

    /// Verify buffer matches constant value
    pub fn verify_constant(buffer: &[u8], value: u8) -> bool {
        buffer.iter().all(|&byte| byte == value)
    }
}

/// Timing utilities
pub mod timing {
    use esp_hal::delay::Delay;

    /// Delay in milliseconds
    pub fn delay_ms(ms: u32) {
        let mut delay = Delay::new();
        delay.delay_millis(ms);
    }

    /// Delay in microseconds
    pub fn delay_us(us: u32) {
        let mut delay = Delay::new();
        delay.delay_micros(us);
    }

    /// Measure operation duration
    pub struct Timer {
        start: u64,
    }

    impl Timer {
        pub fn new() -> Self {
            Self {
                start: esp_hal::time::current_time().duration_since_epoch().to_micros(),
            }
        }

        pub fn elapsed_us(&self) -> u64 {
            let now = esp_hal::time::current_time().duration_since_epoch().to_micros();
            now - self.start
        }

        pub fn elapsed_ms(&self) -> u64 {
            self.elapsed_us() / 1000
        }
    }
}

/// Assertions for test validation
pub mod assertions {
    /// Assert buffers are equal
    pub fn assert_buffers_equal(expected: &[u8], actual: &[u8], msg: &str) {
        assert_eq!(
            expected.len(),
            actual.len(),
            "{}: Length mismatch - expected {}, got {}",
            msg,
            expected.len(),
            actual.len()
        );

        for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_eq!(
                exp, act,
                "{}: Mismatch at index {} - expected 0x{:02X}, got 0x{:02X}",
                msg, i, exp, act
            );
        }
    }

    /// Assert operation completed within timeout
    pub fn assert_within_timeout(elapsed_us: u64, timeout_us: u64, operation: &str) {
        assert!(
            elapsed_us <= timeout_us,
            "{} took too long: {} µs (max: {} µs)",
            operation,
            elapsed_us,
            timeout_us
        );
    }

    /// Assert data rate is within expected range
    pub fn assert_data_rate(bytes: usize, elapsed_us: u64, min_bps: u32, max_bps: u32) {
        let bytes_per_sec = (bytes as u64 * 1_000_000) / elapsed_us;
        assert!(
            bytes_per_sec >= min_bps as u64,
            "Data rate too slow: {} bytes/s (min: {} bytes/s)",
            bytes_per_sec,
            min_bps
        );
        assert!(
            bytes_per_sec <= max_bps as u64,
            "Data rate too fast: {} bytes/s (max: {} bytes/s)",
            bytes_per_sec,
            max_bps
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = TestMasterConfig::default();
        assert_eq!(config.slave_address, 0x55);
        assert_eq!(config.frequency, 100_000);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_config_builder() {
        let config = TestMasterConfig::default()
            .with_slave_address(0x66)
            .with_frequency(400_000)
            .with_timeout(500);
        
        assert_eq!(config.slave_address, 0x66);
        assert_eq!(config.frequency, 400_000);
        assert_eq!(config.timeout_ms, 500);
    }

    #[test]
    fn test_pattern_sequential() {
        let mut buffer = [0u8; 10];
        patterns::sequential(&mut buffer, 5);
        
        for (i, &byte) in buffer.iter().enumerate() {
            assert_eq!(byte, 5 + i as u8);
        }
    }

    #[test]
    fn test_pattern_constant() {
        let mut buffer = [0u8; 10];
        patterns::constant(&mut buffer, 0x42);
        
        assert!(buffer.iter().all(|&b| b == 0x42));
    }

    #[test]
    fn test_pattern_alternating() {
        let mut buffer = [0u8; 10];
        patterns::alternating(&mut buffer);
        
        for (i, &byte) in buffer.iter().enumerate() {
            let expected = if i % 2 == 0 { 0xAA } else { 0x55 };
            assert_eq!(byte, expected);
        }
    }

    #[test]
    fn test_pattern_verify() {
        let mut buffer = [0u8; 10];
        patterns::sequential(&mut buffer, 10);
        
        assert!(patterns::verify_sequential(&buffer, 10));
        assert!(!patterns::verify_sequential(&buffer, 11));
    }
}
