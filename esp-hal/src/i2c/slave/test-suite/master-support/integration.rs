//! Master support for integration tests
//!
//! I2C master implementations for testing slave with other peripherals and frameworks.

use super::common::{TestMaster, TestMasterConfig, patterns, timing};
use esp_hal::{
    peripheral::Peripheral,
    gpio::{InputPin, OutputPin},
};

/// Master for peripheral integration testing
pub struct PeripheralIntegrationMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> PeripheralIntegrationMaster<'d, T>
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

    /// Test I2C while other peripheral is active (generic)
    pub fn test_with_active_peripheral(&mut self, iterations: usize) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        let mut result = IntegrationTestResult::new();
        let data = [0xAAu8; 16];
        
        for _ in 0..iterations {
            // Perform I2C operation
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => {
                    result.record_success(timer.elapsed_us());
                }
                Err(_) => {
                    result.record_error();
                }
            }
            
            // Small delay for other peripheral activity
            timing::delay_ms(1);
        }
        
        Ok(result)
    }

    /// Test SPI + I2C coexistence
    pub fn test_with_spi_active(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate SPI transactions between I2C operations
        self.test_interleaved_operations(50, 10)
    }

    /// Test UART + I2C coexistence
    pub fn test_with_uart_active(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate UART activity
        self.test_interleaved_operations(100, 5)
    }

    /// Test with GPIO interrupts active
    pub fn test_with_gpio_interrupts(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate GPIO interrupt load
        self.test_with_active_peripheral(100)
    }

    /// Test with timer interrupts active
    pub fn test_with_timer_interrupts(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate periodic timer interrupts
        self.test_with_active_peripheral(100)
    }

    /// Test with ADC sampling
    pub fn test_with_adc_sampling(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate continuous ADC sampling
        self.test_with_active_peripheral(50)
    }

    /// Test with PWM output
    pub fn test_with_pwm_active(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Simulate PWM generation
        self.test_with_active_peripheral(50)
    }

    /// Test with WiFi active (if available)
    pub fn test_with_wifi_active(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // WiFi can cause timing jitter
        // Use longer delays to allow for WiFi activity
        self.test_interleaved_operations(30, 50)
    }

    /// Test with Bluetooth active (if available)
    pub fn test_with_bluetooth_active(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Bluetooth shares radio with WiFi
        self.test_interleaved_operations(30, 50)
    }

    /// Helper for interleaved operations
    fn test_interleaved_operations(
        &mut self,
        iterations: usize,
        delay_ms: u64,
    ) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        let mut result = IntegrationTestResult::new();
        let mut data = [0u8; 16];
        
        for i in 0..iterations {
            patterns::sequential(&mut data, i as u8);
            
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => {
                    result.record_success(timer.elapsed_us());
                }
                Err(_) => {
                    result.record_error();
                }
            }
            
            // Simulate other peripheral activity
            timing::delay_ms(delay_ms);
        }
        
        Ok(result)
    }

    /// Test interrupt priority handling
    pub fn test_interrupt_priorities(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Rapid transactions to test interrupt handling
        let mut result = IntegrationTestResult::new();
        let data = [0x01, 0x02, 0x03];
        
        for _ in 0..100 {
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => result.record_success(timer.elapsed_us()),
                Err(_) => result.record_error(),
            }
        }
        
        Ok(result)
    }

    /// Test shared resource contention
    pub fn test_resource_contention(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        // Stress test with minimal delays
        let mut result = IntegrationTestResult::new();
        let data = [0xFFu8; 32];
        
        for _ in 0..50 {
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => result.record_success(timer.elapsed_us()),
                Err(_) => result.record_error(),
            }
            timing::delay_us(100); // Minimal delay
        }
        
        Ok(result)
    }
}

/// Result from integration testing
#[derive(Debug, Clone)]
pub struct IntegrationTestResult {
    pub attempts: usize,
    pub successes: usize,
    pub errors: usize,
    pub total_time_us: u64,
    pub min_time_us: u64,
    pub max_time_us: u64,
}

impl IntegrationTestResult {
    pub fn new() -> Self {
        Self {
            attempts: 0,
            successes: 0,
            errors: 0,
            total_time_us: 0,
            min_time_us: u64::MAX,
            max_time_us: 0,
        }
    }

    pub fn record_success(&mut self, time_us: u64) {
        self.attempts += 1;
        self.successes += 1;
        self.total_time_us += time_us;
        self.min_time_us = self.min_time_us.min(time_us);
        self.max_time_us = self.max_time_us.max(time_us);
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

    pub fn average_time_us(&self) -> u64 {
        if self.successes > 0 {
            self.total_time_us / self.successes as u64
        } else {
            0
        }
    }

    pub fn timing_variance(&self) -> u64 {
        if self.max_time_us > self.min_time_us {
            self.max_time_us - self.min_time_us
        } else {
            0
        }
    }
}

/// Master for OS/framework integration testing
pub struct OsIntegrationMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> OsIntegrationMaster<'d, T>
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

    /// Test basic operation (blocking mode for RTOS/FreeRTOS)
    pub fn test_blocking_operation(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        self.master.write(&data)?;
        
        timing::delay_ms(10);
        
        let mut buffer = [0u8; 4];
        self.master.read(&mut buffer)?;
        
        Ok(())
    }

    /// Test with message passing pattern
    pub fn test_message_passing(&mut self, iterations: usize) -> Result<Vec<[u8; 4]>, esp_hal::i2c::Error> {
        let mut messages = Vec::new();
        
        for i in 0..iterations {
            let mut data = [0u8; 4];
            patterns::sequential(&mut data, i as u8);
            
            self.master.write(&data)?;
            timing::delay_ms(5);
            
            let mut response = [0u8; 4];
            self.master.read(&mut response)?;
            
            messages.push(response);
        }
        
        Ok(messages)
    }

    /// Test synchronization scenario
    pub fn test_synchronization(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Simulate synchronized access pattern
        for _ in 0..10 {
            let data = [0xAAu8; 4];
            self.master.write(&data)?;
            timing::delay_ms(20); // Simulate task switching
        }
        Ok(())
    }

    /// Test task priority impact
    pub fn test_task_priority_impact(&mut self) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        let mut result = IntegrationTestResult::new();
        let data = [0x12, 0x34];
        
        for _ in 0..50 {
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => result.record_success(timer.elapsed_us()),
                Err(_) => result.record_error(),
            }
            timing::delay_ms(10); // Allow task scheduling
        }
        
        Ok(result)
    }

    /// Test with shared resource (mutex) pattern
    pub fn test_shared_resource(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Simulate mutex-protected access
        for _ in 0..20 {
            // "Acquire mutex"
            timing::delay_us(10);
            
            let data = [0x55u8; 4];
            self.master.write(&data)?;
            
            // "Release mutex"
            timing::delay_us(10);
        }
        Ok(())
    }

    /// Test event notification pattern
    pub fn test_event_notification(&mut self) -> Result<Vec<u8>, esp_hal::i2c::Error> {
        let mut events = Vec::new();
        
        for i in 0..10 {
            let data = [i as u8];
            self.master.write(&data)?;
            events.push(i as u8);
            timing::delay_ms(15);
        }
        
        Ok(events)
    }
}

/// Master for async framework testing (Embassy, etc.)
pub struct AsyncFrameworkMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> AsyncFrameworkMaster<'d, T>
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

    /// Test basic communication for async framework validation
    pub fn test_basic_async_support(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03];
        self.master.write(&data)?;
        timing::delay_ms(10);
        
        let mut buffer = [0u8; 3];
        self.master.read(&mut buffer)?;
        
        Ok(())
    }

    /// Test rapid operations for async executor stress
    pub fn test_async_executor_stress(&mut self, iterations: usize) -> Result<IntegrationTestResult, esp_hal::i2c::Error> {
        let mut result = IntegrationTestResult::new();
        let data = [0xAAu8; 4];
        
        for _ in 0..iterations {
            let timer = timing::Timer::new();
            match self.master.write(&data) {
                Ok(_) => result.record_success(timer.elapsed_us()),
                Err(_) => result.record_error(),
            }
            timing::delay_us(500); // Minimal delay
        }
        
        Ok(result)
    }

    /// Test for channel-based communication pattern
    pub fn test_channel_pattern(&mut self, messages: usize) -> Result<Vec<[u8; 4]>, esp_hal::i2c::Error> {
        let mut responses = Vec::new();
        
        for i in 0..messages {
            let mut data = [0u8; 4];
            patterns::sequential(&mut data, i as u8);
            self.master.write(&data)?;
            
            timing::delay_ms(5);
            
            let mut response = [0u8; 4];
            self.master.read(&mut response)?;
            responses.push(response);
        }
        
        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_result() {
        let mut result = IntegrationTestResult::new();
        
        result.record_success(1000);
        result.record_success(2000);
        result.record_error();
        result.record_success(1500);
        
        assert_eq!(result.attempts, 4);
        assert_eq!(result.successes, 3);
        assert_eq!(result.errors, 1);
        assert_eq!(result.success_rate(), 75.0);
        assert_eq!(result.average_time_us(), 1500);
    }

    #[test]
    fn test_timing_variance() {
        let mut result = IntegrationTestResult::new();
        
        result.record_success(1000);
        result.record_success(3000);
        
        assert_eq!(result.timing_variance(), 2000);
    }
}
