//! Master support for reliability tests
//!
//! I2C master implementations for stress and recovery testing.

use super::common::{TestMaster, TestMasterConfig, patterns, timing};
use esp_hal::{
    peripheral::Peripheral,
    gpio::{InputPin, OutputPin},
};

/// Master for stress testing
pub struct StressTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
    stats: StressTestStats,
}

impl<'d, T> StressTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self {
            master,
            stats: StressTestStats::new(),
        })
    }

    /// Continuous operation stress test
    pub fn run_continuous_stress(
        &mut self,
        duration_ms: u64,
        chunk_size: usize,
    ) -> Result<StressTestStats, esp_hal::i2c::Error> {
        let mut data = vec![0u8; chunk_size];
        let start_timer = timing::Timer::new();
        
        while start_timer.elapsed_ms() < duration_ms {
            patterns::sequential(&mut data, self.stats.iterations as u8);
            
            match self.master.write(&data) {
                Ok(_) => self.stats.record_success(),
                Err(e) => {
                    self.stats.record_error();
                    // Continue despite errors for stress testing
                    if self.stats.consecutive_errors > 100 {
                        return Err(e);
                    }
                }
            }
            
            self.stats.iterations += 1;
        }
        
        self.stats.duration_ms = start_timer.elapsed_ms();
        Ok(self.stats.clone())
    }

    /// High-frequency burst stress test
    pub fn run_burst_stress(
        &mut self,
        bursts: usize,
        transactions_per_burst: usize,
    ) -> Result<StressTestStats, esp_hal::i2c::Error> {
        let data = [0xAAu8; 4];
        
        for burst_num in 0..bursts {
            // Rapid burst of transactions
            for _ in 0..transactions_per_burst {
                match self.master.write(&data) {
                    Ok(_) => self.stats.record_success(),
                    Err(_) => self.stats.record_error(),
                }
                self.stats.iterations += 1;
            }
            
            // Brief pause between bursts
            if burst_num < bursts - 1 {
                timing::delay_ms(10);
            }
        }
        
        Ok(self.stats.clone())
    }

    /// Variable transaction size stress test
    pub fn run_variable_size_stress(
        &mut self,
        iterations: usize,
    ) -> Result<StressTestStats, esp_hal::i2c::Error> {
        for i in 0..iterations {
            // Vary size from 1 to 32 bytes
            let size = ((i % 32) + 1).min(32);
            let mut data = vec![0u8; size];
            patterns::pseudo_random(&mut data, i as u8);
            
            match self.master.write(&data) {
                Ok(_) => self.stats.record_success(),
                Err(_) => self.stats.record_error(),
            }
            
            self.stats.iterations += 1;
        }
        
        Ok(self.stats.clone())
    }

    /// Random pattern stress test
    pub fn run_random_pattern_stress(
        &mut self,
        iterations: usize,
    ) -> Result<StressTestStats, esp_hal::i2c::Error> {
        for i in 0..iterations {
            let mut data = [0u8; 32];
            patterns::pseudo_random(&mut data, (i * 7 + 13) as u8);
            
            match self.master.write(&data) {
                Ok(_) => self.stats.record_success(),
                Err(_) => self.stats.record_error(),
            }
            
            self.stats.iterations += 1;
            
            // Occasional delay
            if i % 10 == 0 {
                timing::delay_us(100);
            }
        }
        
        Ok(self.stats.clone())
    }

    /// Maximum throughput stress test
    pub fn run_maximum_throughput_stress(
        &mut self,
        duration_ms: u64,
    ) -> Result<StressTestStats, esp_hal::i2c::Error> {
        let data = [0xFFu8; 32];
        let start_timer = timing::Timer::new();
        
        // No delays - maximum stress
        while start_timer.elapsed_ms() < duration_ms {
            match self.master.write(&data) {
                Ok(_) => self.stats.record_success(),
                Err(_) => self.stats.record_error(),
            }
            self.stats.iterations += 1;
        }
        
        self.stats.duration_ms = start_timer.elapsed_ms();
        Ok(self.stats.clone())
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &StressTestStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = StressTestStats::new();
    }
}

/// Statistics from stress testing
#[derive(Debug, Clone)]
pub struct StressTestStats {
    pub iterations: usize,
    pub successes: usize,
    pub errors: usize,
    pub consecutive_errors: usize,
    pub max_consecutive_errors: usize,
    pub duration_ms: u64,
}

impl StressTestStats {
    pub fn new() -> Self {
        Self {
            iterations: 0,
            successes: 0,
            errors: 0,
            consecutive_errors: 0,
            max_consecutive_errors: 0,
            duration_ms: 0,
        }
    }

    pub fn record_success(&mut self) {
        self.successes += 1;
        self.consecutive_errors = 0;
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
        self.consecutive_errors += 1;
        self.max_consecutive_errors = self.max_consecutive_errors.max(self.consecutive_errors);
    }

    pub fn success_rate(&self) -> f32 {
        if self.iterations > 0 {
            (self.successes as f32 / self.iterations as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn error_rate(&self) -> f32 {
        if self.iterations > 0 {
            (self.errors as f32 / self.iterations as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn transactions_per_second(&self) -> f32 {
        if self.duration_ms > 0 {
            (self.iterations as f32 / self.duration_ms as f32) * 1000.0
        } else {
            0.0
        }
    }
}

/// Master for recovery testing
pub struct RecoveryTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> RecoveryTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test recovery from bus error
    pub fn test_bus_error_recovery(&mut self) -> Result<RecoveryTestResult, esp_hal::i2c::Error> {
        let mut result = RecoveryTestResult::new();
        let data = [0x01, 0x02, 0x03];
        
        // Trigger error (implementation-specific)
        match self.master.write(&data) {
            Ok(_) => result.initial_state = TestState::Success,
            Err(_) => result.initial_state = TestState::Error,
        }
        
        // Try recovery
        timing::delay_ms(10);
        
        match self.master.write(&data) {
            Ok(_) => result.recovery_state = TestState::Success,
            Err(_) => result.recovery_state = TestState::Error,
        }
        
        Ok(result)
    }

    /// Test recovery from timeout
    pub fn test_timeout_recovery(&mut self) -> Result<RecoveryTestResult, esp_hal::i2c::Error> {
        let mut result = RecoveryTestResult::new();
        let data = [0xAA];
        
        // Attempt operation that might timeout
        match self.master.write(&data) {
            Ok(_) => result.initial_state = TestState::Success,
            Err(_) => result.initial_state = TestState::Error,
        }
        
        // Recovery attempt
        timing::delay_ms(100);
        
        match self.master.write(&data) {
            Ok(_) => result.recovery_state = TestState::Success,
            Err(_) => result.recovery_state = TestState::Error,
        }
        
        Ok(result)
    }

    /// Test recovery from FIFO overflow
    pub fn test_fifo_overflow_recovery(&mut self) -> Result<RecoveryTestResult, esp_hal::i2c::Error> {
        let mut result = RecoveryTestResult::new();
        
        // Try to overflow by rapid writes
        for _ in 0..5 {
            let data = [0xFFu8; 32];
            match self.master.write(&data) {
                Ok(_) => {}
                Err(_) => {
                    result.initial_state = TestState::Error;
                    break;
                }
            }
        }
        
        if result.initial_state == TestState::Success {
            result.initial_state = TestState::Success;
        }
        
        // Recovery attempt
        timing::delay_ms(50);
        
        let data = [0xAAu8; 4];
        match self.master.write(&data) {
            Ok(_) => result.recovery_state = TestState::Success,
            Err(_) => result.recovery_state = TestState::Error,
        }
        
        Ok(result)
    }

    /// Test repeated error recovery
    pub fn test_repeated_recovery(&mut self, attempts: usize) -> Result<Vec<RecoveryTestResult>, esp_hal::i2c::Error> {
        let mut results = Vec::new();
        
        for _ in 0..attempts {
            let mut result = RecoveryTestResult::new();
            let data = [0x12, 0x34];
            
            // Trigger error
            match self.master.write(&data) {
                Ok(_) => result.initial_state = TestState::Success,
                Err(_) => result.initial_state = TestState::Error,
            }
            
            // Immediate recovery
            match self.master.write(&data) {
                Ok(_) => result.recovery_state = TestState::Success,
                Err(_) => result.recovery_state = TestState::Error,
            }
            
            results.push(result);
            timing::delay_ms(5);
        }
        
        Ok(results)
    }

    /// Test recovery with address change
    pub fn test_address_change_recovery(&mut self, wrong_addr: u8, correct_addr: u8) -> Result<RecoveryTestResult, esp_hal::i2c::Error> {
        let mut result = RecoveryTestResult::new();
        let data = [0x01, 0x02];
        
        // Try wrong address
        self.master.set_slave_address(wrong_addr);
        match self.master.write(&data) {
            Ok(_) => result.initial_state = TestState::Success,
            Err(_) => result.initial_state = TestState::Error,
        }
        
        // Switch to correct address
        self.master.set_slave_address(correct_addr);
        match self.master.write(&data) {
            Ok(_) => result.recovery_state = TestState::Success,
            Err(_) => result.recovery_state = TestState::Error,
        }
        
        Ok(result)
    }

    /// Test graceful degradation under errors
    pub fn test_graceful_degradation(&mut self, iterations: usize) -> Result<DegradationTestResult, esp_hal::i2c::Error> {
        let mut result = DegradationTestResult::new();
        let data = [0xAAu8; 4];
        
        for _ in 0..iterations {
            match self.master.write(&data) {
                Ok(_) => result.record_success(),
                Err(_) => result.record_error(),
            }
        }
        
        Ok(result)
    }
}

/// Result from recovery testing
#[derive(Debug, Clone, PartialEq)]
pub enum TestState {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct RecoveryTestResult {
    pub initial_state: TestState,
    pub recovery_state: TestState,
}

impl RecoveryTestResult {
    pub fn new() -> Self {
        Self {
            initial_state: TestState::Success,
            recovery_state: TestState::Success,
        }
    }

    pub fn recovered(&self) -> bool {
        matches!(self.recovery_state, TestState::Success)
    }

    pub fn fully_successful(&self) -> bool {
        matches!(self.initial_state, TestState::Success) &&
        matches!(self.recovery_state, TestState::Success)
    }
}

/// Result from degradation testing
#[derive(Debug, Clone)]
pub struct DegradationTestResult {
    pub attempts: usize,
    pub successes: usize,
    pub errors: usize,
}

impl DegradationTestResult {
    pub fn new() -> Self {
        Self {
            attempts: 0,
            successes: 0,
            errors: 0,
        }
    }

    pub fn record_success(&mut self) {
        self.attempts += 1;
        self.successes += 1;
    }

    pub fn record_error(&mut self) {
        self.attempts += 1;
        self.errors += 1;
    }

    pub fn success_rate(&self) -> f32 {
        if self.attempts > 0 {
            (self.successes as f32 / self.attempts as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn is_acceptable(&self, min_success_rate: f32) -> bool {
        self.success_rate() >= min_success_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_stats() {
        let mut stats = StressTestStats::new();
        
        stats.record_success();
        stats.record_success();
        stats.record_error();
        stats.record_success();
        
        assert_eq!(stats.successes, 3);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.consecutive_errors, 0);
        assert_eq!(stats.max_consecutive_errors, 1);
    }

    #[test]
    fn test_recovery_result() {
        let result = RecoveryTestResult {
            initial_state: TestState::Error,
            recovery_state: TestState::Success,
        };
        
        assert!(result.recovered());
        assert!(!result.fully_successful());
    }

    #[test]
    fn test_degradation_result() {
        let mut result = DegradationTestResult::new();
        
        result.record_success();
        result.record_success();
        result.record_error();
        result.record_success();
        
        assert_eq!(result.success_rate(), 75.0);
        assert!(result.is_acceptable(70.0));
        assert!(!result.is_acceptable(80.0));
    }
}
