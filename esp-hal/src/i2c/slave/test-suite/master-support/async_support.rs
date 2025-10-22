//! Master support for async tests
//!
//! Async I2C master implementations to test slave async operations.
//!
//! ## Async Master Types
//!
//! - `AsyncTestMaster` - Low-level async master wrapper
//! - `AsyncOperationsMaster` - Basic async read/write testing
//! - `ConcurrentTestMaster` - Concurrent operation testing
//! - `AsyncWriteReadTestMaster` - Async write_read() with repeated START
//!
//! ## Async write_read() Support
//!
//! The `AsyncWriteReadTestMaster` provides async testing for I2C repeated START
//! transactions using embassy-executor. All operations are non-blocking and can be
//! composed with timeouts, retries, and progress monitoring.
//!
//! See: `I2C_SLAVE_WRITE_READ_SUPPORT.md` for implementation details

use super::common::{TestMasterConfig, patterns, timing};
use esp_hal::{
    gpio::{InputPin, OutputPin},
    i2c::master::{Config as MasterConfig, I2c as MasterI2c},
    peripheral::Peripheral,
};

/// Async test master wrapper
pub struct AsyncTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: MasterI2c<'d, T, esp_hal::Async>,
    slave_address: u8,
}

impl<'d, T> AsyncTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    /// Create a new async test master
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        config: TestMasterConfig,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let master_config = MasterConfig::default().with_frequency(config.frequency.Hz());

        let master = MasterI2c::new_async(peripheral, master_config)?
            .with_sda(sda)
            .with_scl(scl);

        Ok(Self {
            master,
            slave_address: config.slave_address,
        })
    }

    /// Async write to slave
    pub async fn write(&mut self, data: &[u8]) -> Result<(), esp_hal::i2c::Error> {
        self.master.write(self.slave_address, data).await
    }

    /// Async read from slave
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), esp_hal::i2c::Error> {
        self.master.read(self.slave_address, buffer).await
    }

    /// Async write then read
    pub async fn write_read(
        &mut self,
        write_data: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), esp_hal::i2c::Error> {
        self.master
            .write_read(self.slave_address, write_data, read_buffer)
            .await
    }

    /// Change slave address
    pub fn set_slave_address(&mut self, addr: u8) {
        self.slave_address = addr;
    }
}

/// Master for testing async operations
pub struct AsyncOperationsMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: AsyncTestMaster<'d, T>,
}

impl<'d, T> AsyncOperationsMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = AsyncTestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test async write operation
    pub async fn test_async_write(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03, 0x04];
        self.master.write(&data).await
    }

    /// Test async read operation
    pub async fn test_async_read(&mut self, expected: &[u8]) -> Result<(), esp_hal::i2c::Error> {
        let mut buffer = vec![0u8; expected.len()];
        self.master.read(&mut buffer).await?;

        assert_eq!(&buffer, expected, "Async read mismatch");
        Ok(())
    }

    /// Test sequential async operations
    pub async fn test_sequential_operations(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Write
        let write_data = [0xAA, 0xBB, 0xCC, 0xDD];
        self.master.write(&write_data).await?;

        // Delay
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;

        // Read
        let mut read_buffer = [0u8; 4];
        self.master.read(&mut read_buffer).await?;

        Ok(())
    }

    /// Test async operation with timeout
    pub async fn test_with_timeout(
        &mut self,
        timeout_ms: u64,
    ) -> Result<(), embassy_time::TimeoutError> {
        use embassy_time::{Duration, with_timeout};

        let data = [0x01, 0x02];
        with_timeout(Duration::from_millis(timeout_ms), self.master.write(&data))
            .await
            .map(|_| ())
    }

    /// Test rapid async operations
    pub async fn test_rapid_operations(&mut self, count: usize) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..count {
            let data = [i as u8; 4];
            self.master.write(&data).await?;
        }
        Ok(())
    }
}

/// Master for testing concurrent operations
pub struct ConcurrentTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: AsyncTestMaster<'d, T>,
}

impl<'d, T> ConcurrentTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = AsyncTestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Continuous write operations
    pub async fn continuous_write(
        &mut self,
        duration_ms: u64,
    ) -> Result<usize, esp_hal::i2c::Error> {
        use embassy_time::{Duration, Instant};

        let start = Instant::now();
        let mut count = 0;

        while start.elapsed() < Duration::from_millis(duration_ms) {
            let data = [count as u8; 4];
            self.master.write(&data).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Continuous read operations
    pub async fn continuous_read(
        &mut self,
        duration_ms: u64,
    ) -> Result<usize, esp_hal::i2c::Error> {
        use embassy_time::{Duration, Instant};

        let start = Instant::now();
        let mut count = 0;

        while start.elapsed() < Duration::from_millis(duration_ms) {
            let mut buffer = [0u8; 4];
            self.master.read(&mut buffer).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Interleaved operations
    pub async fn interleaved_operations(
        &mut self,
        iterations: usize,
    ) -> Result<(), esp_hal::i2c::Error> {
        for i in 0..iterations {
            // Write
            let write_data = [i as u8; 4];
            self.master.write(&write_data).await?;

            // Small delay
            embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;

            // Read
            let mut read_buffer = [0u8; 4];
            self.master.read(&mut read_buffer).await?;
        }
        Ok(())
    }
}

/// Master for testing Future behavior
pub struct FutureTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: AsyncTestMaster<'d, T>,
}

impl<'d, T> FutureTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = AsyncTestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test Future cancellation by dropping
    pub async fn test_cancellable_operation(&mut self) -> Result<(), esp_hal::i2c::Error> {
        let data = [0x01, 0x02, 0x03];
        self.master.write(&data).await
    }

    /// Test select between two operations
    pub async fn test_select_operations(&mut self) -> Result<bool, esp_hal::i2c::Error> {
        use embassy_futures::select::{Either, select};

        let write_data = [0xAA, 0xBB];
        let mut read_buffer = [0u8; 2];

        match select(
            self.master.write(&write_data),
            self.master.read(&mut read_buffer),
        )
        .await
        {
            Either::First(_) => Ok(true),   // Write completed first
            Either::Second(_) => Ok(false), // Read completed first
        }
    }

    /// Test with timeout that will expire
    pub async fn test_timeout_expiry(&mut self) -> Result<bool, esp_hal::i2c::Error> {
        use embassy_time::{Duration, with_timeout};

        let data = [0x01, 0x02];

        match with_timeout(Duration::from_micros(1), self.master.write(&data)).await {
            Ok(_) => Ok(false), // Unexpectedly completed
            Err(_) => Ok(true), // Expected timeout
        }
    }

    /// Multiple sequential operations
    pub async fn test_sequential_futures(&mut self) -> Result<(), esp_hal::i2c::Error> {
        // Create multiple operations sequentially
        for i in 0..5 {
            let data = [i; 2];
            self.master.write(&data).await?;
            embassy_time::Timer::after(embassy_time::Duration::from_millis(5)).await;
        }
        Ok(())
    }
}

/// Helper functions for async master testing
pub mod async_helpers {
    use embassy_time::{Duration, Timer};

    /// Async delay in milliseconds
    pub async fn delay_ms(ms: u64) {
        Timer::after(Duration::from_millis(ms)).await;
    }

    /// Async delay in microseconds
    pub async fn delay_us(us: u64) {
        Timer::after(Duration::from_micros(us)).await;
    }

    /// Measure async operation duration
    pub async fn measure_operation<F, R>(operation: F) -> (R, u64)
    where
        F: core::future::Future<Output = R>,
    {
        use embassy_time::Instant;

        let start = Instant::now();
        let result = operation.await;
        let elapsed = start.elapsed().as_micros();

        (result, elapsed)
    }

    /// Retry async operation with exponential backoff
    pub async fn retry_with_backoff<F, R, E>(mut operation: F, max_retries: usize) -> Result<R, E>
    where
        F: FnMut() -> core::future::Future<Output = Result<R, E>>,
    {
        let mut delay_ms = 10;

        for attempt in 0..max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e);
                    }
                    delay_ms(delay_ms).await;
                    delay_ms *= 2; // Exponential backoff
                }
            }
        }

        unreachable!()
    }

    /// Run operation with progress callback
    pub async fn with_progress<F, R>(
        operation: F,
        check_interval_ms: u64,
        mut callback: impl FnMut(u64),
    ) -> R
    where
        F: core::future::Future<Output = R>,
    {
        use embassy_futures::select::{Either, select};
        use embassy_time::Instant;

        let start = Instant::now();
        let mut result = None;

        loop {
            match select(
                async {
                    if result.is_none() {
                        result = Some(operation.await);
                    }
                },
                delay_ms(check_interval_ms),
            )
            .await
            {
                Either::First(_) => {
                    if let Some(r) = result {
                        return r;
                    }
                }
                Either::Second(_) => {
                    callback(start.elapsed().as_millis());
                }
            }
        }
    }
}

/// Master for async write_read() testing (repeated START)
///
/// Provides async versions of write_read() tests with embassy-executor support.
/// Tests I2C slave's ability to handle async repeated START transactions.
///
/// See: I2C_SLAVE_WRITE_READ_SUPPORT.md for implementation details
pub struct AsyncWriteReadTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    master: AsyncTestMaster<'d, T>,
}

impl<'d, T> AsyncWriteReadTestMaster<'d, T>
where
    T: esp_hal::i2c::master::Instance,
{
    pub fn new(
        peripheral: impl Peripheral<P = T> + 'd,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, esp_hal::i2c::Error> {
        let config = TestMasterConfig::default();
        let master = AsyncTestMaster::new(peripheral, sda, scl, config)?;
        Ok(Self { master })
    }

    /// Test async write_read() with single byte
    pub async fn test_single_byte_write_read(
        &mut self,
        register: u8,
    ) -> Result<u8, esp_hal::i2c::Error> {
        let write_data = [register];
        let mut read_buffer = [0u8; 1];

        self.master
            .write_read(&write_data, &mut read_buffer)
            .await?;
        Ok(read_buffer[0])
    }

    /// Test async write_read() with multi-byte read
    pub async fn test_multi_byte_write_read(
        &mut self,
        register: u8,
        read_count: usize,
    ) -> Result<Vec<u8>, esp_hal::i2c::Error> {
        let write_data = [register];
        let mut read_buffer = vec![0u8; read_count];

        self.master
            .write_read(&write_data, &mut read_buffer)
            .await?;
        Ok(read_buffer)
    }

    /// Test async write_read() with register-based mode compatibility
    pub async fn test_register_mode_compatibility(
        &mut self,
        register: u8,
    ) -> Result<Vec<u8>, esp_hal::i2c::Error> {
        let write_data = [register];
        let mut read_buffer = vec![0u8; 4];

        self.master
            .write_read(&write_data, &mut read_buffer)
            .await?;
        Ok(read_buffer)
    }

    /// Test async write_read() with maximum FIFO
    pub async fn test_maximum_fifo_write_read(
        &mut self,
        register: u8,
    ) -> Result<[u8; 32], esp_hal::i2c::Error> {
        let write_data = [register];
        let mut read_buffer = [0u8; 32];

        self.master
            .write_read(&write_data, &mut read_buffer)
            .await?;
        Ok(read_buffer)
    }

    /// Test async write_read() with timeout
    pub async fn test_write_read_with_timeout(
        &mut self,
        register: u8,
        timeout_ms: u64,
    ) -> Result<Vec<u8>, embassy_time::TimeoutError> {
        use embassy_time::{Duration, with_timeout};

        let write_data = [register];
        let mut read_buffer = vec![0u8; 4];

        with_timeout(
            Duration::from_millis(timeout_ms),
            self.master.write_read(&write_data, &mut read_buffer),
        )
        .await?;

        Ok(read_buffer)
    }

    /// Test multiple concurrent async write_read operations
    pub async fn test_concurrent_write_read(
        &mut self,
        count: usize,
    ) -> Result<Vec<Vec<u8>>, esp_hal::i2c::Error> {
        let mut results = Vec::new();

        for i in 0..count {
            let register = i as u8;
            let write_data = [register];
            let mut read_buffer = vec![0u8; 2];

            self.master
                .write_read(&write_data, &mut read_buffer)
                .await?;
            results.push(read_buffer);

            // Brief delay between operations
            timing::delay_ms(1).await;
        }

        Ok(results)
    }

    /// Test async write_read() with progress monitoring
    pub async fn test_write_read_with_progress(
        &mut self,
        register: u8,
        mut progress_callback: impl FnMut(u64),
    ) -> Result<Vec<u8>, esp_hal::i2c::Error> {
        use embassy_time::Instant;

        let start = Instant::now();

        let write_data = [register];
        let mut read_buffer = vec![0u8; 4];

        self.master
            .write_read(&write_data, &mut read_buffer)
            .await?;

        let elapsed = start.elapsed().as_micros();
        progress_callback(elapsed);

        Ok(read_buffer)
    }

    /// Test async write_read() vs separate transactions
    pub async fn test_atomic_vs_separate(
        &mut self,
        register: u8,
        read_count: usize,
    ) -> Result<(Vec<u8>, Vec<u8>), esp_hal::i2c::Error> {
        // Method 1: Atomic write_read()
        let write_data = [register];
        let mut read_buffer1 = vec![0u8; read_count];
        self.master
            .write_read(&write_data, &mut read_buffer1)
            .await?;

        // Delay
        timing::delay_ms(10).await;

        // Method 2: Separate operations
        self.master.write(&[register]).await?;
        timing::delay_ms(5).await;
        let mut read_buffer2 = vec![0u8; read_count];
        self.master.read(&mut read_buffer2).await?;

        Ok((read_buffer1, read_buffer2))
    }

    /// Test async write_read() with rapid sequential access
    pub async fn test_rapid_sequential_reads(
        &mut self,
        start_register: u8,
        count: usize,
    ) -> Result<Vec<Vec<u8>>, esp_hal::i2c::Error> {
        let mut results = Vec::new();

        for offset in 0..count {
            let register = start_register.wrapping_add(offset as u8);
            let write_data = [register];
            let mut read_buffer = vec![0u8; 2];

            self.master
                .write_read(&write_data, &mut read_buffer)
                .await?;
            results.push(read_buffer);

            // No delay - test rapid access
        }

        Ok(results)
    }

    /// Test async write_read() with error recovery
    pub async fn test_write_read_with_retry(
        &mut self,
        register: u8,
        max_retries: usize,
    ) -> Result<Vec<u8>, esp_hal::i2c::Error> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            let write_data = [register];
            let mut read_buffer = vec![0u8; 4];

            match self.master.write_read(&write_data, &mut read_buffer).await {
                Ok(_) => return Ok(read_buffer),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        timing::delay_ms(10 * (attempt as u64 + 1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Set slave address
    pub fn set_slave_address(&mut self, address: u8) {
        self.master.set_slave_address(address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_config() {
        let config = TestMasterConfig::default();
        assert_eq!(config.slave_address, 0x55);
        assert_eq!(config.frequency, 100_000);
    }
}
