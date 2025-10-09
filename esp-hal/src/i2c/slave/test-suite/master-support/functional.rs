//! Master support for functional tests
//!
//! I2C master implementations to test slave functional operations.

use super::common::{TestMaster, TestMasterConfig, patterns, timing, assertions};
use esp_hal::{
    peripheral::Peripheral,
    gpio::{InputPin, OutputPin},
};

/// Master for basic communication tests
pub struct BasicCommMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> BasicCommMaster<'d, T>
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

    /// Test 1: Simple write to slave
    pub fn test_simple_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        self.master.write(&data)
    }

    /// Test 2: Simple read from slave
    pub fn test_simple_read(&mut self, expected_data: &[u8]) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer = vec![0u8; expected_data.len()];
        self.master.read(&mut buffer)?;
        
        assertions::assert_buffers_equal(expected_data, &buffer, "Simple read");
        Ok(())
    }

    /// Test 3: Multi-byte write
    pub fn test_multi_byte_write(&mut self, size: usize) -> Result<(), esp_hal::i2c::Error> {
        let mut data = vec![0u8; size];
        patterns::sequential(&mut data, 0);
        self.master.write(&data)
    }

    /// Test 4: Multi-byte read
    pub fn test_multi_byte_read(&mut self, size: usize) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer = vec![0u8; size];
        self.master.read(&mut buffer)?;
        
        // Verify sequential pattern
        assert!(
            patterns::verify_sequential(&buffer, 0),
            "Multi-byte read pattern mismatch"
        );
        Ok(())
    }

    /// Test 5: Maximum FIFO write (32 bytes)
    pub fn test_maximum_fifo_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let mut data = [0u8; 32];
        patterns::sequential(&mut data, 0);
        self.master.write(&data)
    }

    /// Test 6: Maximum FIFO read (32 bytes)
    pub fn test_maximum_fifo_read(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer = [0u8; 32];
        self.master.read(&mut buffer)?;
        
        assert!(
            patterns::verify_sequential(&buffer, 0),
            "Maximum FIFO read pattern mismatch"
        );
        Ok(())
    }

    /// Test: Beyond FIFO capacity (should use multiple transactions internally)
    pub fn test_beyond_fifo_capacity(&mut self, size: usize) -> Result<(), esp_hal::i2c::Error> {
        assert!(size > 32, "Size must exceed FIFO capacity");
        
        let mut data = vec![0u8; size];
        patterns::sequential(&mut data, 0);
        
        // Write in chunks
        for chunk in data.chunks(32) {
            self.master.write(chunk)?;
            timing::delay_ms(10); // Give slave time to process
        }
        Ok(())
    }

    /// Test: Zero-length write
    pub fn test_zero_length_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data: [u8; 0] = [];
        self.master.write(&data)
    }

    /// Test: Zero-length read
    pub fn test_zero_length_read(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer: [u8; 0] = [];
        self.master.read(&mut buffer)
    }
}

/// Master for address testing
pub struct AddressTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> AddressTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        slave_address: u8,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default().with_slave_address(slave_address);
        let master = TestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test 7: Write to correct address
    pub fn test_correct_address(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0xAA, 0xBB, 0xCC];
        self.master.write(&data)
    }

    /// Test 8: Write to wrong address (should NACK)
    pub fn test_wrong_address(&mut self, wrong_address: u8) -> Result<bool, esp_hal::i2c::Error> {
        self.master.set_slave_address(wrong_address);
        let data = [0x11, 0x22];
        
        match self.master.write(&data) {
            Ok(_) => Ok(false), // Unexpected success
            Err(_) => Ok(true), // Expected NACK
        }
    }

    /// Change slave address and test
    pub fn change_address(&mut self, new_address: u8) {
        self.master.set_slave_address(new_address);
    }

    /// Test general call address (0x00)
    pub fn test_general_call(&mut self) -> Result<(), esp_hal::i2c::Error> {
        self.master.set_slave_address(0x00);
        let data = [0xFF];
        self.master.write(&data)
    }
}

/// Master for FIFO testing
pub struct FifoTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> FifoTestMaster<'d, T>
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

    /// Fill slave FIFO completely
    pub fn fill_fifo(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let mut data = [0u8; 32];
        patterns::sequential(&mut data, 0);
        self.master.write(&data)
    }

    /// Write multiple FIFO-sized chunks
    pub fn test_sequential_fifo_operations(&mut self, count: usize) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..count {
            let mut data = [0u8; 32];
            patterns::sequential(&mut data, i as u8);
            self.master.write(&data)?;
            timing::delay_ms(50); // Allow slave to process
        }
        Ok(())
    }

    /// Test alternating read/write
    pub fn test_alternating_operations(&mut self) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..5 {
            // Write
            let mut write_data = [0u8; 16];
            patterns::sequential(&mut write_data, i * 16);
            self.master.write(&write_data)?;
            timing::delay_ms(10);
            
            // Read
            let mut read_buffer = [0u8; 16];
            self.master.read(&mut read_buffer)?;
            timing::delay_ms(10);
        }
        Ok(())
    }

    /// Trigger FIFO overflow by writing without slave reading
    pub fn trigger_overflow(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Write more than FIFO can hold without delays
        for _ in 0..3 {
            let data = [0xFFu8; 32];
            self.master.write(&data)?;
            // No delay - slave may not keep up
        }
        Ok(())
    }
}

/// Master for clock stretching tests
pub struct ClockStretchMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> ClockStretchMaster<'d, T>
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

    /// Test 10: Write with clock stretching (measure time)
    pub fn test_with_clock_stretch(&mut self) -> Result<u64, esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        let timer = timing::Timer::new();
        self.master.write(&data)?;
        Ok(timer.elapsed_us())
    }

    /// Test 11: Write without clock stretching (measure time)
    pub fn test_without_clock_stretch(&mut self) -> Result<u64, esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        let timer = timing::Timer::new();
        self.master.write(&data)?;
        Ok(timer.elapsed_us())
    }

    /// Rapid write to test clock stretch limits
    pub fn test_rapid_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..10 {
            let data = [i as u8; 8];
            self.master.write(&data)?;
            // No delay - let slave stretch if needed
        }
        Ok(())
    }
}

/// Master for filter testing
pub struct FilterTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> FilterTestMaster<'d, T>
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

    /// Test 12: Normal write (should work with filtering)
    pub fn test_normal_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0xA5, 0x5A, 0xFF, 0x00];
        self.master.write(&data)
    }

    /// Test 13: Multiple writes at different speeds
    pub fn test_speed_variations(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x12, 0x34, 0x56, 0x78];
        
        for _ in 0..5 {
            self.master.write(&data)?;
            timing::delay_ms(20);
        }
        Ok(())
    }
}

/// Master for interrupt testing
pub struct InterruptTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> InterruptTestMaster<'d, T>
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

    /// Test 14: Trigger transaction complete interrupt
    pub fn trigger_transaction_complete(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02];
        self.master.write(&data)
    }

    /// Test 15: Trigger RxFifoFull interrupt
    pub fn trigger_rx_fifo_full(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0u8; 32]; // Fill FIFO
        self.master.write(&data)
    }

    /// Test 16: Trigger TxFifoEmpty interrupt
    pub fn trigger_tx_fifo_empty(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer = [0u8; 32]; // Empty FIFO
        self.master.read(&mut buffer)
    }

    /// Test 17: Multiple rapid transactions for interrupts
    pub fn trigger_multiple_interrupts(&mut self) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..10 {
            let data = [i as u8; 4];
            self.master.write(&data)?;
            timing::delay_ms(5);
        }
        Ok(())
    }
}

/// Master for error condition testing
pub struct ErrorTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: TestMaster<'d, T>,
}

impl<'d, T> ErrorTestMaster<'d, T>
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

    /// Test 18: Arbitration (requires multi-master setup)
    pub fn trigger_arbitration_scenario(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // This requires another master competing for the bus
        let data = [0xAA, 0xBB];
        self.master.write(&data)
    }

    /// Test 19: Timeout (stop mid-transaction)
    pub fn trigger_timeout(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Start transaction but don't complete
        // This is hardware-dependent
        let data = [0x01];
        self.master.write(&data)
    }

    /// Test 20: Bus busy
    pub fn check_bus_busy(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Multiple quick operations
        for _ in 0..5 {
            let data = [0xFF];
            self.master.write(&data)?;
        }
        Ok(())
    }

    /// Trigger NACK by wrong address
    pub fn trigger_nack(&mut self) -> Result<bool, esp_hal::i2c::Error> {
        self.master.set_slave_address(0x7F); // Non-existent address
        let data = [0x01];
        
        match self.master.write(&data) {
            Ok(_) => Ok(false),
            Err(_) => Ok(true), // Expected NACK
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_generation() {
        let mut buffer = [0u8; 32];
        patterns::sequential(&mut buffer, 0);
        
        for (i, &byte) in buffer.iter().enumerate() {
            assert_eq!(byte, i as u8);
        }
    }
}
