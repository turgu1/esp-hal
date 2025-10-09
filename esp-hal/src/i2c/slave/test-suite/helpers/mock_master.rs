//! Mock I2C master for testing
//!
//! Provides a software-based I2C master implementation for testing
//! the slave driver without requiring a second physical device.

/// Mock I2C master for testing slave implementation
#[cfg(test)]
pub struct MockMaster {
    address: u8,
    tx_buffer: Vec<u8>,
    rx_buffer: Vec<u8>,
}

#[cfg(test)]
impl MockMaster {
    /// Create a new mock master
    pub fn new(slave_address: u8) -> Self {
        Self {
            address: slave_address,
            tx_buffer: Vec::new(),
            rx_buffer: Vec::new(),
        }
    }

    /// Write data to slave
    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            return Err("Empty write");
        }
        self.tx_buffer.extend_from_slice(data);
        Ok(())
    }

    /// Read data from slave
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if buffer.is_empty() {
            return Err("Empty buffer");
        }
        
        let len = core::cmp::min(buffer.len(), self.rx_buffer.len());
        buffer[..len].copy_from_slice(&self.rx_buffer[..len]);
        self.rx_buffer.drain(..len);
        
        Ok(len)
    }

    /// Write then read in a single transaction
    pub fn write_read(&mut self, write_data: &[u8], read_buffer: &mut [u8]) -> Result<usize, &'static str> {
        self.write(write_data)?;
        self.read(read_buffer)
    }

    /// Get transmitted data (for verification)
    pub fn get_tx_buffer(&self) -> &[u8] {
        &self.tx_buffer
    }

    /// Set receive buffer (simulate slave response)
    pub fn set_rx_buffer(&mut self, data: &[u8]) {
        self.rx_buffer.clear();
        self.rx_buffer.extend_from_slice(data);
    }

    /// Clear buffers
    pub fn clear(&mut self) {
        self.tx_buffer.clear();
        self.rx_buffer.clear();
    }

    /// Get configured slave address
    pub fn slave_address(&self) -> u8 {
        self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_master_creation() {
        let master = MockMaster::new(0x55);
        assert_eq!(master.slave_address(), 0x55);
    }

    #[test]
    fn test_mock_master_write() {
        let mut master = MockMaster::new(0x55);
        assert!(master.write(&[0xAA, 0xBB]).is_ok());
        assert_eq!(master.get_tx_buffer(), &[0xAA, 0xBB]);
    }

    #[test]
    fn test_mock_master_read() {
        let mut master = MockMaster::new(0x55);
        master.set_rx_buffer(&[0xCC, 0xDD]);
        
        let mut buffer = [0u8; 2];
        let len = master.read(&mut buffer).unwrap();
        
        assert_eq!(len, 2);
        assert_eq!(buffer, [0xCC, 0xDD]);
    }

    #[test]
    fn test_mock_master_clear() {
        let mut master = MockMaster::new(0x55);
        master.write(&[0xAA]).unwrap();
        master.set_rx_buffer(&[0xBB]);
        
        master.clear();
        
        assert_eq!(master.get_tx_buffer().len(), 0);
    }
}
