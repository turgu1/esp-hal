//! Master support for performance tests
//!
//! I2C master implementations for measuring slave performance.

use super::common::{TestMaster, TestMasterConfig, assertions, patterns, timing};
use esp_hal::{
    gpio::{InputPin, OutputPin},
    peripheral::Peripheral,
};

/// Master for speed testing at different frequencies
pub struct SpeedTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> SpeedTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    /// Create master at standard mode (100 kHz)
    pub fn new_standard_mode(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default().with_frequency(100_000);
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Create master at fast mode (400 kHz)
    pub fn new_fast_mode(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default().with_frequency(400_000);
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Create master at fast mode plus (1 MHz)
    pub fn new_fast_mode_plus(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default().with_frequency(1_000_000);
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test communication reliability at current speed
    pub fn test_reliability(
        &mut self,
        iterations: usize,
    ) -> Result<SpeedTestResults, esp_hal::i2c::Error> {
        let mut results = SpeedTestResults::new();
        let data_size = 32;
        let mut data = vec![0u8; data_size];

        for i in 0..iterations {
            patterns::sequential(&mut data, i as u8);

            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => {
                    results.record_success(timer.elapsed_us());
                }
                Err(e) => {
                    results.record_failure();
                    if results.errors > 10 {
                        return Err(e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Measure transaction time
    pub fn measure_transaction_time(
        &mut self,
        data_size: usize,
    ) -> Result<u64, esp_hal::i2c::Error> {
        let mut data = vec![0u8; data_size];
        patterns::sequential(&mut data, 0);

        let timer = timing::Timer::new();
        self.master.write(&data)?;
        Ok(timer.elapsed_us())
    }

    /// Test maximum sustainable rate
    pub fn test_maximum_rate(
        &mut self,
        duration_ms: u64,
    ) -> Result<RateTestResults, esp_hal::i2c::Error> {
        let mut results = RateTestResults::new();
        let data = [0xAAu8; 32];

        let start_timer = timing::Timer::new();

        while start_timer.elapsed_ms() < duration_ms {
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => {
                    results.successful_transactions += 1;
                    results.total_bytes += data.len();
                    results.total_time_us += timer.elapsed_us();
                }
                Err(_) => {
                    results.failed_transactions += 1;
                }
            }
        }

        results.calculate_rates();
        Ok(results)
    }

    /// Test with clock stretching impact
    pub fn test_with_clock_stretch_delay(&mut self) -> Result<u64, esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        let timer = timing::Timer::new();
        self.master.write(&data)?;
        Ok(timer.elapsed_us())
    }
}

/// Results from speed testing
#[derive(Debug, Clone)]
pub struct SpeedTestResults {
    pub successes: usize,
    pub errors: usize,
    pub total_time_us: u64,
    pub min_time_us: u64,
    pub max_time_us: u64,
}

impl SpeedTestResults {
    pub fn new() -> Self {
        Self {
            successes: 0,
            errors: 0,
            total_time_us: 0,
            min_time_us: u64::MAX,
            max_time_us: 0,
        }
    }

    pub fn record_success(&mut self, time_us: u64) {
        self.successes += 1;
        self.total_time_us += time_us;
        self.min_time_us = self.min_time_us.min(time_us);
        self.max_time_us = self.max_time_us.max(time_us);
    }

    pub fn record_failure(&mut self) {
        self.errors += 1;
    }

    pub fn average_time_us(&self) -> u64 {
        if self.successes > 0 {
            self.total_time_us / self.successes as u64
        } else {
            0
        }
    }

    pub fn success_rate(&self) -> f32 {
        let total = self.successes + self.errors;
        if total > 0 {
            (self.successes as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Master for throughput testing
pub struct ThroughputTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> ThroughputTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        frequency: u32,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default().with_frequency(frequency);
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test single-byte throughput
    pub fn test_single_byte_throughput(
        &mut self,
        iterations: usize,
    ) -> Result<ThroughputResults, esp_hal::i2c::Error> {
        let mut results = ThroughputResults::new();
        let data = [0xAAu8];

        let timer = timing::Timer::new();

        for _ in 0..iterations {
            self.master.write(&data)?;
            results.bytes_transferred += 1;
        }

        results.duration_us = timer.elapsed_us();
        results.calculate();
        Ok(results)
    }

    /// Test bulk transfer throughput
    pub fn test_bulk_throughput(
        &mut self,
        chunk_size: usize,
        iterations: usize,
    ) -> Result<ThroughputResults, esp_hal::i2c::Error> {
        let mut results = ThroughputResults::new();
        let mut data = vec![0u8; chunk_size];
        patterns::sequential(&mut data, 0);

        let timer = timing::Timer::new();

        for i in 0..iterations {
            patterns::sequential(&mut data, i as u8);
            self.master.write(&data)?;
            results.bytes_transferred += chunk_size;
        }

        results.duration_us = timer.elapsed_us();
        results.calculate();
        Ok(results)
    }

    /// Test optimal FIFO size throughput
    pub fn test_fifo_optimal(
        &mut self,
        iterations: usize,
    ) -> Result<ThroughputResults, esp_hal::i2c::Error> {
        let mut results = ThroughputResults::new();
        let mut data = [0u8; 32]; // FIFO size

        let timer = timing::Timer::new();

        for i in 0..iterations {
            patterns::sequential(&mut data, i as u8);
            self.master.write(&data)?;
            results.bytes_transferred += 32;
        }

        results.duration_us = timer.elapsed_us();
        results.calculate();
        Ok(results)
    }

    /// Test sustained transfer rate
    pub fn test_sustained_rate(
        &mut self,
        duration_ms: u64,
    ) -> Result<ThroughputResults, esp_hal::i2c::Error> {
        let mut results = ThroughputResults::new();
        let data = [0xFFu8; 32];

        let start_timer = timing::Timer::new();

        while start_timer.elapsed_ms() < duration_ms {
            self.master.write(&data)?;
            results.bytes_transferred += 32;
        }

        results.duration_us = start_timer.elapsed_us();
        results.calculate();
        Ok(results)
    }

    /// Test various chunk sizes for efficiency
    pub fn test_chunk_efficiency(
        &mut self,
    ) -> Result<Vec<(usize, ThroughputResults)>, esp_hal::i2c::Error> {
        let chunk_sizes = [1, 2, 4, 8, 16, 32, 64, 128];
        let mut results = Vec::new();

        for &size in &chunk_sizes {
            let effective_size = size.min(32); // Limit to FIFO
            let result = self.test_bulk_throughput(effective_size, 100)?;
            results.push((size, result));
        }

        Ok(results)
    }

    /// Compare read vs write throughput
    pub fn test_read_write_comparison(
        &mut self,
        iterations: usize,
    ) -> Result<(ThroughputResults, ThroughputResults), esp_hal::i2c::Error> {
        let data = [0xAAu8; 32];

        // Test writes
        let mut write_results = ThroughputResults::new();
        let write_timer = timing::Timer::new();
        for _ in 0..iterations {
            self.master.write(&data)?;
            write_results.bytes_transferred += 32;
        }
        write_results.duration_us = write_timer.elapsed_us();
        write_results.calculate();

        // Test reads
        let mut read_results = ThroughputResults::new();
        let read_timer = timing::Timer::new();
        for _ in 0..iterations {
            let mut buffer = [0u8; 32];
            self.master.read(&mut buffer)?;
            read_results.bytes_transferred += 32;
        }
        read_results.duration_us = read_timer.elapsed_us();
        read_results.calculate();

        Ok((write_results, read_results))
    }
}

/// Results from throughput testing
#[derive(Debug, Clone)]
pub struct ThroughputResults {
    pub bytes_transferred: usize,
    pub duration_us: u64,
    pub bytes_per_second: u32,
    pub bits_per_second: u32,
    pub transactions_per_second: u32,
}

impl ThroughputResults {
    pub fn new() -> Self {
        Self {
            bytes_transferred: 0,
            duration_us: 0,
            bytes_per_second: 0,
            bits_per_second: 0,
            transactions_per_second: 0,
        }
    }

    pub fn calculate(&mut self) {
        if self.duration_us > 0 {
            let duration_sec = self.duration_us as f64 / 1_000_000.0;
            self.bytes_per_second = (self.bytes_transferred as f64 / duration_sec) as u32;
            self.bits_per_second = self.bytes_per_second * 8;
        }
    }

    pub fn efficiency_percent(&self, theoretical_bps: u32) -> f32 {
        if theoretical_bps > 0 {
            (self.bits_per_second as f32 / theoretical_bps as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Results from rate testing
#[derive(Debug, Clone)]
pub struct RateTestResults {
    pub successful_transactions: usize,
    pub failed_transactions: usize,
    pub total_bytes: usize,
    pub total_time_us: u64,
    pub average_time_per_transaction_us: u64,
    pub bytes_per_second: u32,
}

impl RateTestResults {
    pub fn new() -> Self {
        Self {
            successful_transactions: 0,
            failed_transactions: 0,
            total_bytes: 0,
            total_time_us: 0,
            average_time_per_transaction_us: 0,
            bytes_per_second: 0,
        }
    }

    pub fn calculate_rates(&mut self) {
        if self.successful_transactions > 0 {
            self.average_time_per_transaction_us =
                self.total_time_us / self.successful_transactions as u64;
        }

        if self.total_time_us > 0 {
            let duration_sec = self.total_time_us as f64 / 1_000_000.0;
            self.bytes_per_second = (self.total_bytes as f64 / duration_sec) as u32;
        }
    }

    pub fn success_rate(&self) -> f32 {
        let total = self.successful_transactions + self.failed_transactions;
        if total > 0 {
            (self.successful_transactions as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_results() {
        let mut results = SpeedTestResults::new();
        results.record_success(1000);
        results.record_success(2000);
        results.record_success(1500);

        assert_eq!(results.successes, 3);
        assert_eq!(results.average_time_us(), 1500);
        assert_eq!(results.min_time_us, 1000);
        assert_eq!(results.max_time_us, 2000);
    }

    #[test]
    fn test_throughput_results() {
        let mut results = ThroughputResults::new();
        results.bytes_transferred = 1000;
        results.duration_us = 1_000_000; // 1 second
        results.calculate();

        assert_eq!(results.bytes_per_second, 1000);
        assert_eq!(results.bits_per_second, 8000);
    }

    #[test]
    fn test_efficiency_calculation() {
        let mut results = ThroughputResults::new();
        results.bits_per_second = 80_000;

        let efficiency = results.efficiency_percent(100_000);
        assert!((efficiency - 80.0).abs() < 0.1);
    }
}
