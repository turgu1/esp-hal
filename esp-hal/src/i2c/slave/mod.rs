#![cfg_attr(docsrs, procmacros::doc_replace)]
//! # Inter-Integrated Circuit (I2C) - Slave mode
//!
//! ## Overview
//!
//! This driver implements the I2C Slave mode. In this mode, the MCU acts as
//! an I2C slave device that responds to requests from an I2C master. The slave
//! device is identified by its configured I2C address.
//!
//! ## Configuration
//!
//! The driver can be configured using the [`Config`] struct. To create a
//! configuration, you can use the [`Config::default()`] method, and then modify
//! the individual settings as needed, by calling `with_*` methods on the
//! [`Config`] struct.
//!
//! The driver supports both 7-bit and 10-bit I2C slave addresses.
//! Addresses can be specified using `u8`, `u16`, or `i32` and are automatically
//! converted to the appropriate mode:
//!
//! ```rust, no_run
//! # {before_snippet}
//! use esp_hal::i2c::slave::Config;
//!
//! // 7-bit address from u8
//! let config_7bit = Config::default()
//!     .with_address(0x55.into())
//!     .with_timeout_ms(2000);
//!
//! // 10-bit address from u16
//! let config_10bit = Config::default()
//!     .with_address(0x1A5.into())  // 10-bit address (0x000 - 0x3FF)
//!     .with_timeout_ms(2000);
//!
//! // Also supports i32 (negative values converted to absolute value)
//! let config_i32 = Config::default()
//!     .with_address((-85).into())  // Converts to 0x55 (7-bit)
//!     .with_timeout_ms(2000);
//! # {after_snippet}
//! ```
//!
//! ### Register-Based Mode (ESP32-C6)
//!
//! On ESP32-C6, you can enable register-based mode to emulate I2C devices with
//! register addressing (like sensors). When enabled, the first byte after the
//! slave address is treated as a register address:
//!
//! ```rust, no_run
//! # {before_snippet}
//! use esp_hal::i2c::slave::Config;
//!
//! #[cfg(esp32c6)]
//! let config = Config::default()
//!     .with_address(0x55)
//!     .with_register_based_mode(true);
//! # {after_snippet}
//! ```
//!
//! You will then need to pass the configuration to [`I2c::new`], and you can
//! also change the configuration later by calling [`I2c::apply_config`].
//!
//! You will also need to specify the SDA and SCL pins when you create the
//! driver instance.
//! ```rust, no_run
//! # {before_snippet}
//! use esp_hal::i2c::slave::I2c;
//! # use esp_hal::i2c::slave::Config;
//! #
//! # let config = Config::default();
//! #
//! // You need to configure the driver during initialization:
//! let mut i2c = I2c::new(peripherals.I2C0, config)?
//!     .with_sda(peripherals.GPIO2)
//!     .with_scl(peripherals.GPIO3);
//! # {after_snippet}
//! ```
//!
//! ## Usage
//!
//! The slave responds to requests from the I2C master. The driver provides
//! methods for reading data sent by the master and sending data back:
//! ```rust, no_run
//! # {before_snippet}
//! # use esp_hal::i2c::slave::{I2c, Config};
//! # let config = Config::default();
//! # let mut i2c = I2c::new(peripherals.I2C0, config)?;
//! #
//! let mut read_buffer = [0u8; 128];
//! let mut write_buffer = [0xAA; 128];
//!
//! // Read data from master
//! let bytes_read = i2c.read(&mut read_buffer)?;
//!
//! // Write data to master
//! i2c.write(&write_buffer)?;
//! # {after_snippet}
//! ```
//!
//! ### Important: FIFO Size Limitation & Clock Stretching Compatibility
//!
//! The hardware FIFO is limited to 32 bytes. This means:
//! - **Reading**: A single `read()` call can only retrieve up to 32 bytes from the FIFO
//! - **For packets ≥ 32 bytes**: You may need interrupt-driven reception with `RxFifoFull` event
//! - **Writing**: A single `write()` call can only load up to 32 bytes into the TX FIFO
//!
//! **Clock Stretching Compatibility**: Clock stretching on ESP32-C6 slave is now properly
//! implemented with automatic SCL release when TX FIFO has data ready. Previous versions
//! had a bug where the slave could hold SCL low indefinitely during read operations.
//!
//! **ESP32 Master Compatibility**: While clock stretching now works correctly, ESP32
//! (original) masters still have limited clock stretching support and may timeout when
//! the slave holds SCL low for extended periods.
//!
//! **Recommendation for ESP32 master**: For maximum compatibility, disable clock stretching:
//! ```rust, no_run
//! # {before_snippet}
//! # use esp_hal::i2c::slave::Config;
//! let config = Config::default()
//!     .with_clock_stretch_enable(false)
//!     .with_address(0x55.into());
//! # {after_snippet}
//! ```
//!
//! **For packets < 30 bytes**: Blocking `read()` works fine without clock stretching.
//!
//! **For packets ≥ 30 bytes without clock stretching**: The FIFO can overflow causing NACK.
//! Use interrupt-driven approach to read data as it arrives (FIFO watermark triggers at 30 bytes).
//!
//! For packets of 32 bytes or more, use interrupt-driven approach:
//!
//! ```rust, no_run
//! # {before_snippet}
//! # use esp_hal::i2c::slave::{I2c, Config, Event};
//! # let config = Config::default();
//! # let mut i2c = I2c::new(peripherals.I2C0, config)?;
//! #
//! // Enable RX FIFO full interrupt (triggers at 30 bytes, leaving room for more)
//! i2c.listen(Event::RxFifoFull);
//!
//! let mut large_buffer = [0u8; 128];
//! let mut offset = 0;
//!
//! loop {
//!     // Check if RX FIFO has data
//!     let events = i2c.interrupts();
//!     if events.contains(Event::RxFifoFull) {
//!         // Read chunk from FIFO (up to 32 bytes)
//!         let chunk_read = i2c.read(&mut large_buffer[offset..])?;
//!         offset += chunk_read;
//!         
//!         i2c.clear_interrupts(Event::RxFifoFull);
//!     }
//!     
//!     // Check for transaction complete
//!     if events.contains(Event::TransComplete) {
//!         // Process complete packet in large_buffer[..offset]
//!         offset = 0; // Reset for next packet
//!         i2c.clear_interrupts(Event::TransComplete);
//!     }
//! }
//! # {after_snippet}
//! ```
//!
//! ### Register-Based Mode Example (ESP32-C6)
//!
//! When emulating a register-based I2C device (like a sensor), use register-based
//! mode to handle register addressing:
//!
//! ```rust, no_run
//! # {before_snippet}
//! # use esp_hal::i2c::slave::{I2c, Config};
//! #[cfg(esp32c6)]
//! {
//!     let config = Config::default()
//!         .with_address(0x48)  // Sensor address
//!         .with_register_based_mode(true);
//!     
//!     let mut i2c = I2c::new(peripherals.I2C0, config)?
//!         .with_sda(peripherals.GPIO1)
//!         .with_scl(peripherals.GPIO2);
//!     
//!     // Simulate sensor registers
//!     let mut registers = [0u8; 256];
//!     registers[0x00] = 0x48; // Device ID
//!     registers[0x01] = 0x25; // Temperature high byte
//!     registers[0x02] = 0x60; // Temperature low byte
//!     
//!     loop {
//!         // Wait for master write (register address + optional data)
//!         let mut rx_buffer = [0u8; 32];
//!         if let Ok(bytes_read) = i2c.read(&mut rx_buffer) {
//!             let register_addr = i2c.read_register_address();
//!             
//!             if bytes_read > 0 {
//!                 // Master wrote data to register
//!                 for (i, &byte) in rx_buffer[..bytes_read].iter().enumerate() {
//!                     registers[(register_addr as usize + i) & 0xFF] = byte;
//!                 }
//!             }
//!             
//!             // Prepare response for potential master read
//!             let response = &registers[register_addr as usize..][..4];
//!             i2c.write(response)?;
//!         }
//!     }
//! }
//! # {after_snippet}
//! ```
//! # {after_snippet}
//! ```

use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use enumset::{EnumSet, EnumSetType};

use crate::{
    Async, Blocking, DriverMode, any_peripheral,
    asynch::AtomicWaker,
    gpio::{
        DriveMode, InputSignal, OutputConfig, OutputSignal, PinGuard, Pull,
        interconnect::{self, PeripheralOutput},
    },
    handler,
    interrupt::{self, InterruptHandler},
    pac::i2c0::RegisterBlock,
    private, ram,
    system::PeripheralGuard,
    time::Instant,
};

const I2C_FIFO_SIZE: usize = property!("i2c_master.fifo_size");

/// Representation of I2C slave address.
///
/// Addresses can be created from `u8`, `u16`, or `i32` types using `.into()`:
/// - `u8` values create 7-bit addresses
/// - `u16` values create 7-bit if ≤ 0x7F, otherwise 10-bit
/// - `i32` values are converted to their absolute value, then use same logic as `u16`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum I2cAddress {
    /// 7-bit address mode type.
    ///
    /// Note that 7-bit addresses are specified in **right-aligned** form, e.g.
    /// in the range `0x00..=0x7F`.
    ///
    /// For example, a device that has the seven bit address of `0b011_0010`,
    /// is addressed as `0x32`, NOT `0x64` or `0x65`.
    SevenBit(u8),

    /// 10-bit address mode type.
    ///
    /// Note that 10-bit addresses are specified in **right-aligned** form, e.g.
    /// in the range `0x00..=0x3FF`.
    ///
    /// 10-bit addressing uses a special addressing scheme where the first byte
    /// starts with `11110` followed by the two MSBs of the address.
    TenBit(u16),
}

impl I2cAddress {
    fn validate(&self) -> Result<(), Error> {
        match self {
            I2cAddress::SevenBit(addr) => {
                if *addr > 0x7F {
                    return Err(Error::AddressInvalid(*self));
                }
            }
            I2cAddress::TenBit(addr) => {
                if *addr > 0x3FF {
                    return Err(Error::AddressInvalid(*self));
                }
            }
        }

        Ok(())
    }

    /// Returns true if this is a 10-bit address
    fn is_ten_bit(&self) -> bool {
        matches!(self, I2cAddress::TenBit(_))
    }

    /// Returns the address value as u16
    fn as_u16(&self) -> u16 {
        match self {
            I2cAddress::SevenBit(addr) => *addr as u16,
            I2cAddress::TenBit(addr) => *addr,
        }
    }
}

impl From<u8> for I2cAddress {
    fn from(value: u8) -> Self {
        I2cAddress::SevenBit(value)
    }
}

impl From<u16> for I2cAddress {
    fn from(value: u16) -> Self {
        if value <= 0x7F {
            I2cAddress::SevenBit(value as u8)
        } else if value <= 0x3FF {
            I2cAddress::TenBit(value)
        } else {
            // For values beyond 10-bit range, wrap to 10-bit
            I2cAddress::TenBit(value & 0x3FF)
        }
    }
}

impl From<i32> for I2cAddress {
    fn from(value: i32) -> Self {
        // Convert signed to unsigned, taking absolute value for negative numbers
        let unsigned = value.unsigned_abs();

        if unsigned <= 0x7F {
            I2cAddress::SevenBit(unsigned as u8)
        } else if unsigned <= 0x3FF {
            I2cAddress::TenBit(unsigned as u16)
        } else {
            // For values beyond 10-bit range, wrap to 10-bit
            I2cAddress::TenBit((unsigned & 0x3FF) as u16)
        }
    }
}

/// I2C-specific transmission errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The transmission exceeded the FIFO size.
    FifoExceeded,
    /// The acknowledgment check failed.
    AcknowledgeCheckFailed,
    /// A timeout occurred during transmission.
    Timeout,
    /// The arbitration for the bus was lost.
    ArbitrationLost,
    /// The execution of the I2C command was incomplete.
    ExecutionIncomplete,
    /// Zero length read or write operation.
    ZeroLengthInvalid,
    /// The given address is invalid.
    AddressInvalid(I2cAddress),
    /// Bus is busy.
    BusBusy,
    /// TX FIFO overflow.
    TxFifoOverflow,
    /// RX FIFO underflow.
    RxFifoUnderflow,
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::FifoExceeded => write!(f, "The transmission exceeded the FIFO size"),
            Error::AcknowledgeCheckFailed => write!(f, "The acknowledgment check failed"),
            Error::Timeout => write!(f, "A timeout occurred during transmission"),
            Error::ArbitrationLost => write!(f, "The arbitration for the bus was lost"),
            Error::ExecutionIncomplete => {
                write!(f, "The execution of the I2C command was incomplete")
            }
            Error::ZeroLengthInvalid => write!(f, "Zero length read or write operation"),
            Error::AddressInvalid(address) => {
                write!(f, "The given address ({address:?}) is invalid")
            }
            Error::BusBusy => write!(f, "Bus is busy"),
            Error::TxFifoOverflow => write!(f, "TX FIFO overflow"),
            Error::RxFifoUnderflow => write!(f, "RX FIFO underflow"),
        }
    }
}

/// I2C-specific configuration errors
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Provided address is not valid.
    AddressInvalid,
}

impl core::error::Error for ConfigError {}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::AddressInvalid => write!(f, "Provided address is invalid"),
        }
    }
}

/// I2C slave driver configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, procmacros::BuilderLite)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// The I2C slave address.
    ///
    /// Supports both 7-bit (0x00..=0x7F) and 10-bit (0x000..=0x3FF) addresses.
    /// Use `I2cAddress::SevenBit(addr)` or `I2cAddress::TenBit(addr)`, or simply
    /// convert from `u8` for 7-bit or `u16` for automatic detection.
    ///
    /// Default value: 7-bit address 0x55.
    address: I2cAddress,

    /// Enable clock stretching.
    ///
    /// Default value: true.
    clock_stretch_enable: bool,

    /// Enable SDA filtering.
    ///
    /// Default value: true.
    sda_filter_enable: bool,

    /// SDA filter threshold.
    ///
    /// Default value: 7.
    sda_filter_threshold: u8,

    /// Enable SCL filtering.
    ///
    /// Default value: true.
    scl_filter_enable: bool,

    /// SCL filter threshold.
    ///
    /// Default value: 7.
    scl_filter_threshold: u8,

    /// Timeout duration for blocking read operations in milliseconds.
    ///
    /// This timeout prevents infinite waiting when no master transmission occurs.
    ///
    /// Default value: 1000 (1 second).
    timeout_ms: u32,

    /// Enable register-based mode (FIFO address configuration).
    ///
    /// When enabled, the first byte received after the slave address is treated
    /// as a "register address" and stored in a separate internal register (not
    /// the main RX FIFO). This is useful for emulating register-based I2C devices
    /// like sensors that use register addressing.
    ///
    /// When disabled (default), all received bytes are stored sequentially in
    /// the RX FIFO (raw data stream mode).
    ///
    /// **Supported devices**: Currently only ESP32-C6 is confirmed to support
    /// this feature. The setting is only available when compiling for ESP32-C6.
    ///
    /// To use this feature:
    /// - Set `register_based_mode` to `true` in the config
    /// - After a transaction, call `read()` to get the data bytes
    /// - Call `read_register_address()` to get the register address byte
    /// - Handle the request based on the register address and data
    ///
    /// Default value: false (raw data stream mode).
    #[cfg(esp32c6)]
    register_based_mode: bool,

    /// Automatically clear TX FIFO when receiving data from master (write operation).
    ///
    /// When enabled, the TX FIFO is automatically cleared whenever the slave
    /// receives data from the master (`read()` is called). This ensures no stale
    /// response data from previous transactions remains in the TX FIFO.
    ///
    /// **Use cases:**
    /// - **Enable (true)**: Request/response protocols where each master write
    ///   requires a fresh response. The slave reads a command, processes it,
    ///   then writes a response. This prevents stale responses from previous
    ///   commands.
    ///
    /// - **Disable (false)**: Protocols where responses are pre-loaded before
    ///   master writes, or when fine-grained control over TX FIFO is needed.
    ///   You must manually call `clear_tx_fifo()` when appropriate.
    ///
    /// **Example with auto-clear enabled:**
    /// ```rust, no_run
    /// # use esp_hal::i2c::slave::{Config, I2c};
    /// # let peripherals = ();
    /// let config = Config::default()
    ///     .with_clear_tx_on_write(true);  // Auto-clear TX FIFO
    /// # let mut i2c: Result<I2c<'_, esp_hal::Blocking>, _> = Err(esp_hal::i2c::slave::ConfigError::AddressInvalid);
    /// // let mut i2c = I2c::new(peripherals.I2C0, config)?;
    ///
    /// loop {
    ///     let mut cmd = [0u8; 1];
    ///     i2c.read(&mut cmd)?; // TX FIFO automatically cleared here!
    ///     
    ///     let response = process_command(cmd[0]);
    ///     i2c.write(&response)?;
    /// }
    /// # fn process_command(_: u8) -> &'static [u8] { &[0] }
    /// ```
    ///
    /// Default value: false (manual control).
    clear_tx_on_write: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: I2cAddress::SevenBit(0x55),
            clock_stretch_enable: true,
            sda_filter_enable: true,
            sda_filter_threshold: 7,
            scl_filter_enable: true,
            scl_filter_threshold: 7,
            timeout_ms: 1000,
            #[cfg(esp32c6)]
            register_based_mode: false,
            clear_tx_on_write: false, // Manual control by default for backward compatibility
        }
    }
}

#[procmacros::doc_replace]
/// I2C slave driver
///
/// ## Example
///
/// ```rust, no_run
/// # {before_snippet}
/// use esp_hal::i2c::slave::{Config, I2c};
/// let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
///     .with_sda(peripherals.GPIO1)
///     .with_scl(peripherals.GPIO2);
///
/// let mut data = [0u8; 128];
/// let bytes_read = i2c.read(&mut data)?;
/// # {after_snippet}
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct I2c<'d, Dm: DriverMode> {
    i2c: AnyI2c<'d>,
    phantom: PhantomData<Dm>,
    guard: PeripheralGuard,
    config: DriverConfig,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct DriverConfig {
    config: Config,
    sda_pin: PinGuard,
    scl_pin: PinGuard,
}

#[instability::unstable]
impl<Dm: DriverMode> embassy_embedded_hal::SetConfig for I2c<'_, Dm> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.apply_config(config)
    }
}

impl<'d> I2c<'d, Blocking> {
    #[procmacros::doc_replace]
    /// Create a new I2C slave instance.
    ///
    /// ## Errors
    ///
    /// A [`ConfigError`] variant will be returned if the slave address
    /// passed in config is invalid.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let i2c = I2c::new(peripherals.I2C0, Config::default())?
    ///     .with_sda(peripherals.GPIO1)
    ///     .with_scl(peripherals.GPIO2);
    /// # {after_snippet}
    /// ```
    pub fn new(i2c: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
        let guard = PeripheralGuard::new(i2c.info().peripheral);

        let sda_pin = PinGuard::new_unconnected(i2c.info().sda_output);
        let scl_pin = PinGuard::new_unconnected(i2c.info().scl_output);

        let mut i2c = I2c {
            i2c: i2c.degrade(),
            phantom: PhantomData,
            guard,
            config: DriverConfig {
                config,
                sda_pin,
                scl_pin,
            },
        };

        i2c.apply_config(&config)?;

        Ok(i2c)
    }

    /// Reconfigures the driver to operate in [`Async`] mode.
    pub fn into_async(mut self) -> I2c<'d, Async> {
        self.set_interrupt_handler(self.driver().info.async_handler);

        I2c {
            i2c: self.i2c,
            phantom: PhantomData,
            guard: self.guard,
            config: self.config,
        }
    }

    #[cfg_attr(
        not(multi_core),
        doc = "Registers an interrupt handler for the peripheral."
    )]
    #[cfg_attr(
        multi_core,
        doc = "Registers an interrupt handler for the peripheral on the current core."
    )]
    #[doc = ""]
    /// Note that this will replace any previously registered interrupt
    /// handlers.
    ///
    /// You can restore the default/unhandled interrupt handler by passing
    /// [DEFAULT_INTERRUPT_HANDLER][crate::interrupt::DEFAULT_INTERRUPT_HANDLER].
    ///
    /// # Panics
    ///
    /// Panics if passed interrupt handler is invalid (e.g. has priority
    /// `None`)
    #[instability::unstable]
    pub fn set_interrupt_handler(&mut self, handler: InterruptHandler) {
        self.i2c.set_interrupt_handler(handler);
    }

    /// Listen for the given interrupts
    #[instability::unstable]
    pub fn listen(&mut self, interrupts: impl Into<EnumSet<Event>>) {
        self.i2c.info().enable_listen(interrupts.into(), true)
    }

    /// Unlisten the given interrupts
    #[instability::unstable]
    pub fn unlisten(&mut self, interrupts: impl Into<EnumSet<Event>>) {
        self.i2c.info().enable_listen(interrupts.into(), false)
    }

    /// Gets asserted interrupts
    #[instability::unstable]
    pub fn interrupts(&mut self) -> EnumSet<Event> {
        self.i2c.info().interrupts()
    }

    /// Resets asserted interrupts
    #[instability::unstable]
    pub fn clear_interrupts(&mut self, interrupts: EnumSet<Event>) {
        self.i2c.info().clear_interrupts(interrupts)
    }
}

impl private::Sealed for I2c<'_, Blocking> {}

#[instability::unstable]
impl crate::interrupt::InterruptConfigurable for I2c<'_, Blocking> {
    fn set_interrupt_handler(&mut self, handler: InterruptHandler) {
        self.i2c.set_interrupt_handler(handler);
    }
}

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
#[instability::unstable]
pub enum Event {
    /// Triggered when the slave receives data from master.
    RxFifoFull,

    /// Triggered when the slave needs to send data to master.
    TxFifoEmpty,

    /// Triggered when a byte has been received.
    ByteReceived,

    /// Triggered when a byte has been transmitted.
    ByteTransmitted,

    /// Triggered when transaction is complete.
    TransComplete,

    /// Triggered when slave is addressed.
    SlaveAddressed,

    /// Triggered when a STOP condition is detected.
    StopDetected,

    /// Triggered when a START condition is detected.
    StartDetected,
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct I2cFuture<'a> {
    events: EnumSet<Event>,
    driver: Driver<'a>,
    deadline: Option<Instant>,
    /// True if the Future has been polled to completion.
    finished: bool,
}

impl<'a> I2cFuture<'a> {
    pub fn new(events: EnumSet<Event>, driver: Driver<'a>, deadline: Option<Instant>) -> Self {
        driver.regs().int_ena().modify(|_, w| {
            #[cfg(esp32)]
            for event in events {
                match event {
                    Event::RxFifoFull => w.rxfifo_full().set_bit(),
                    Event::TxFifoEmpty => w.txfifo_empty().set_bit(),
                    Event::ByteReceived => w.rx_rec_full().set_bit(),
                    Event::ByteTransmitted => w.tx_send_empty().set_bit(),
                    Event::TransComplete => w.trans_complete().set_bit(),
                    Event::SlaveAddressed => w.trans_complete().set_bit(),
                    Event::StopDetected => w.end_detect().set_bit(), // Use end_detect for STOP
                    Event::StartDetected => w.trans_start().set_bit(), // Use trans_start for START
                };
            }

            #[cfg(not(esp32))]
            for event in events {
                match event {
                    Event::RxFifoFull => w.rxfifo_wm().set_bit(),
                    Event::TxFifoEmpty => w.txfifo_wm().set_bit(),
                    Event::ByteReceived => w.rxfifo_wm().set_bit(),
                    Event::ByteTransmitted => w.txfifo_wm().set_bit(),
                    Event::TransComplete => w.trans_complete().set_bit(),
                    Event::SlaveAddressed => w.trans_complete().set_bit(),
                    Event::StopDetected => w.end_detect().set_bit(), // Use end_detect for STOP
                    Event::StartDetected => w.trans_start().set_bit(), // Use trans_start for START
                };
            }

            w.arbitration_lost().set_bit();
            w.time_out().set_bit();

            w
        });

        Self::new_blocking(events, driver, deadline)
    }

    pub fn new_blocking(
        events: EnumSet<Event>,
        driver: Driver<'a>,
        deadline: Option<Instant>,
    ) -> Self {
        Self {
            events,
            driver,
            deadline,
            finished: false,
        }
    }

    fn is_done(&self) -> bool {
        !self.driver.info.interrupts().is_disjoint(self.events)
    }

    fn poll_completion(&mut self) -> Poll<Result<(), Error>> {
        let now = if self.deadline.is_some() {
            Instant::now()
        } else {
            Instant::EPOCH
        };
        let error = self.driver.check_errors();

        let result = if self.is_done() {
            let result = if error == Err(Error::Timeout) {
                Ok(())
            } else {
                error
            };
            Poll::Ready(result)
        } else if error.is_err() {
            Poll::Ready(error)
        } else if let Some(deadline) = self.deadline
            && now > deadline
        {
            Poll::Ready(Err(Error::Timeout))
        } else {
            Poll::Pending
        };

        if result.is_ready() {
            self.finished = true;
        }

        result
    }
}

impl core::future::Future for I2cFuture<'_> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.driver.state.waker.register(ctx.waker());

        let result = self.poll_completion();

        if result.is_pending() && self.deadline.is_some() {
            ctx.waker().wake_by_ref();
        }

        result
    }
}

impl Drop for I2cFuture<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let result = self.poll_completion();
            if result.is_pending() || result == Poll::Ready(Err(Error::Timeout)) {
                self.driver.reset_fifo();
            }
        }
    }
}

impl<'d> I2c<'d, Async> {
    /// Reconfigures the driver to operate in [`Blocking`] mode.
    pub fn into_blocking(self) -> I2c<'d, Blocking> {
        self.i2c.disable_peri_interrupt();

        I2c {
            i2c: self.i2c,
            phantom: PhantomData,
            guard: self.guard,
            config: self.config,
        }
    }

    #[procmacros::doc_replace]
    /// Reads data sent by the master asynchronously.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
    ///     .with_sda(peripherals.GPIO1)
    ///     .with_scl(peripherals.GPIO2)
    ///     .into_async();
    ///
    /// let mut data = [0u8; 128];
    /// let bytes_read = i2c.read_async(&mut data).await?;
    /// # {after_snippet}
    /// ```
    pub async fn read_async(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let driver = self.driver();
        driver.reset_fifo();

        I2cFuture::new(Event::RxFifoFull.into(), driver, None).await?;

        Ok(self.driver().read_fifo(buffer))
    }

    #[procmacros::doc_replace]
    /// Writes data to be sent to the master asynchronously.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let mut i2c = I2c::new(peripherals.I2C0, Config::default())?
    ///     .with_sda(peripherals.GPIO1)
    ///     .with_scl(peripherals.GPIO2)
    ///     .into_async();
    ///
    /// i2c.write_async(&[0xAA, 0xBB]).await?;
    /// # {after_snippet}
    /// ```
    pub async fn write_async(&mut self, buffer: &[u8]) -> Result<(), Error> {
        // ESP32-C6 specific: Prepare for async write
        #[cfg(esp32c6)]
        {
            let driver = self.driver();
            // Reset any pending errors
            driver.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });

            // Clear TX FIFO before writing new data
            driver
                .regs()
                .fifo_conf()
                .modify(|_, w| w.tx_fifo_rst().set_bit());
            driver
                .regs()
                .fifo_conf()
                .modify(|_, w| w.tx_fifo_rst().clear_bit());
        }

        let driver = self.driver();
        driver.write_fifo(buffer)?;

        I2cFuture::new(Event::TxFifoEmpty.into(), driver, None).await?;

        Ok(())
    }
}

impl<'d, Dm> I2c<'d, Dm>
where
    Dm: DriverMode,
{
    fn driver(&self) -> Driver<'_> {
        Driver {
            info: self.i2c.info(),
            state: self.i2c.state(),
            config: &self.config,
        }
    }

    /// ESP32-C6 specific: Manually release clock stretching after write()
    ///
    /// This function must be called after `write()` in write_read scenarios.
    /// The hardware requires the clock stretch to be manually cleared after
    /// loading TX FIFO data.
    ///
    /// See [`Driver::release_scl_stretch`] for implementation details.
    #[cfg(esp32c6)]
    pub fn release_scl_stretch(&self) {
        self.driver().release_scl_stretch();
    }

    /// Connect a pin to the I2C SDA signal.
    ///
    /// This will replace previous pin assignments for this signal.
    pub fn with_sda(mut self, sda: impl PeripheralOutput<'d>) -> Self {
        let info = self.driver().info;
        let input = info.sda_input;
        let output = info.sda_output;
        Driver::connect_pin(sda.into(), input, output, &mut self.config.sda_pin);

        self
    }

    #[procmacros::doc_replace]
    /// Connect a pin to the I2C SCL signal.
    ///
    /// This will replace previous pin assignments for this signal.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let i2c = I2c::new(peripherals.I2C0, Config::default())?.with_scl(peripherals.GPIO2);
    /// # {after_snippet}
    /// ```
    pub fn with_scl(mut self, scl: impl PeripheralOutput<'d>) -> Self {
        let info = self.driver().info;
        let input = info.scl_input;
        let output = info.scl_output;
        Driver::connect_pin(scl.into(), input, output, &mut self.config.scl_pin);

        self
    }

    #[procmacros::doc_replace]
    /// Reads data sent by the master
    ///
    /// If the configuration has `clear_tx_on_write` enabled, this method will
    /// automatically clear the TX FIFO before reading. This prevents stale response
    /// data from previous transactions.
    ///
    /// **For write_read() transactions:** The TX FIFO is still cleared, but clock
    /// stretching (if enabled) will automatically hold SCL low during the read phase
    /// until you call `write()` to load the response data. This allows the slave to
    /// process the command and prepare a response even though the master has already
    /// moved to the read phase.
    ///
    /// See [`Config::with_clear_tx_on_write`] for more details.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// # let mut i2c = I2c::new(
    /// #   peripherals.I2C0,
    /// #   Config::default(),
    /// # )?;
    /// let mut data = [0u8; 128];
    /// let bytes_read = i2c.read(&mut data)?;
    /// # {after_snippet}
    /// ```
    ///
    /// ## Errors
    ///
    /// The corresponding error variant from [`Error`] will be returned if the passed buffer has
    /// zero length.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Err(Error::ZeroLengthInvalid);
        }

        let driver = self.driver();

        // Wait for RX data and detect transaction type
        let _is_write_read = driver.wait_for_rx_data()?;

        // Read data from RX FIFO first
        // The RX FIFO is automatically emptied by the read_fifo() operation
        // DO NOT manually clear RX FIFO - this can interfere with hardware state
        let count = driver.read_fifo(buffer);

        // Clear TX FIFO AFTER reading if auto-clear is enabled
        // This ensures stale response data is removed before the application
        // calls write() with new data
        if self.config.config.clear_tx_on_write {
            // Step 1: Set reset bit
            driver
                .regs()
                .fifo_conf()
                .modify(|_, w| w.tx_fifo_rst().set_bit());
            // Sufficient delay for hardware to process reset
            for _ in 0..100 {
                unsafe { core::arch::asm!("nop") };
            }

            // Step 2: Clear reset bit
            driver
                .regs()
                .fifo_conf()
                .modify(|_, w| w.tx_fifo_rst().clear_bit());
            // Wait for FIFO to stabilize
            for _ in 0..100 {
                unsafe { core::arch::asm!("nop") };
            }

            // Step 3: Update configuration to ensure FIFO reset takes effect
            driver.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());
            for _ in 0..50 {
                unsafe { core::arch::asm!("nop") };
            }

            // Step 4: Verify TX FIFO is actually empty (ESP32-C6 specific)
            #[cfg(esp32c6)]
            {
                // Retry up to 3 times if FIFO isn't empty
                for retry in 0..3 {
                    let status = driver.regs().sr().read();
                    if status.txfifo_cnt().bits() == 0 {
                        break; // Success!
                    }

                    if retry < 2 {
                        // One more attempt with full reset cycle
                        driver
                            .regs()
                            .fifo_conf()
                            .modify(|_, w| w.tx_fifo_rst().set_bit());
                        for _ in 0..150 {
                            unsafe { core::arch::asm!("nop") };
                        }
                        driver
                            .regs()
                            .fifo_conf()
                            .modify(|_, w| w.tx_fifo_rst().clear_bit());
                        for _ in 0..150 {
                            unsafe { core::arch::asm!("nop") };
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Reads the register address byte when in register-based mode
    ///
    /// When register-based mode is enabled in the configuration, the first byte
    /// received after the slave address is treated as a "register address" and
    /// stored separately from the main RX FIFO data bytes.
    ///
    /// This method retrieves that register address byte. It should be called
    /// after receiving data from the master to determine which register was
    /// addressed.
    ///
    /// **Important**: In register-based mode:
    /// - The first byte sent by master is the register address (retrieved by this method)
    /// - Subsequent bytes are the data (retrieved by `read()`)
    /// - This allows emulating register-based I2C devices like sensors
    ///
    /// **Note**: This method is only available on ESP32-C6. The register address
    /// is stored in the hardware's RAM_DATA register at offset 0.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let config = Config::default().with_register_based_mode(true);
    /// let mut i2c = I2c::new(peripherals.I2C0, config)?
    ///     .with_sda(peripherals.GPIO1)
    ///     .with_scl(peripherals.GPIO2);
    ///
    /// // Wait for master transaction
    /// let mut data = [0u8; 128];
    /// let bytes_read = i2c.read(&mut data)?;
    ///
    /// // Get the register address that the master specified
    /// let register_addr = i2c.read_register_address();
    ///
    /// // Handle the request based on register_addr and data
    /// match register_addr {
    ///     0x00 => { /* handle register 0x00 */ }
    ///     0x01 => { /* handle register 0x01 */ }
    ///     _ => { /* unknown register */ }
    /// }
    /// # {after_snippet}
    /// ```
    #[cfg(esp32c6)]
    pub fn read_register_address(&self) -> u8 {
        // In register-based mode, the register address is stored in RAM_DATA[0]
        // We can read it from the fifo_st register's rxfifo_raddr field
        self.driver().regs().fifo_st().read().rxfifo_raddr().bits()
    }

    #[procmacros::doc_replace]
    #[procmacros::doc_replace]
    /// Writes data to be sent to the master
    ///
    /// **IMPORTANT for ESP32-C6**: For slave write (master read) operations, you must call
    /// this function to load data into the TX FIFO **BEFORE** the master initiates a read
    /// request. If the TX FIFO is empty when the master requests data, the slave will
    /// clock-stretch (hold SCL low) until data is available.
    ///
    /// **Clock Stretching Fix**: This implementation now correctly releases clock stretching
    /// after loading data into the TX FIFO, preventing the previous issue where SCL could
    /// remain low indefinitely during read operations.
    ///
    /// **Recommended usage pattern**:
    /// 1. Call `write()` to preload response data into the TX FIFO
    /// 2. Wait for the master to address the slave for reading
    /// 3. The hardware will automatically transmit the preloaded data
    /// 4. Clock stretching will be properly released when TX FIFO has data
    /// 5. Reload the TX FIFO with `write()` for subsequent read requests
    ///
    /// ## Errors
    ///
    /// The corresponding error variant from [`Error`] will be returned if the passed buffer has
    /// zero length.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// # let mut i2c = I2c::new(
    /// #   peripherals.I2C0,
    /// #   Config::default(),
    /// # )?;
    /// // Preload data BEFORE master reads
    /// i2c.write(&[0xAA, 0xBB])?;
    ///
    /// // Now master can read this data
    /// # {after_snippet}
    /// ```
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::ZeroLengthInvalid);
        }

        // ESP32-C6 specific: For slave write, we need to ensure we're ready to transmit
        #[cfg(esp32c6)]
        {
            // Clear all interrupts from previous operations
            self.driver()
                .regs()
                .int_clr()
                .write(|w| unsafe { w.bits(0x1FFF) });

            // Ensure slave mode is set (don't touch trans_start - that's for master mode)
            self.driver().regs().ctr().modify(|_, w| {
                w.ms_mode().clear_bit(); // Slave mode
                w.sda_force_out().set_bit();
                w.scl_force_out().set_bit();
                w.slv_tx_auto_start_en().set_bit(); // Ensure auto TX is enabled
                w
            });
        }

        // Load data into TX FIFO
        let result = self.driver().write_fifo(buffer);

        // ESP32-C6 specific: Additional safety check - if write_fifo failed but we have
        // clock stretching enabled, make sure we don't leave SCL stuck low
        #[cfg(esp32c6)]
        if result.is_err() && self.driver().config.config.clock_stretch_enable {
            // Emergency SCL release to prevent bus hang
            self.driver().force_release_scl_stretch();
        }

        result
    }

    #[procmacros::doc_replace]
    /// Clears the TX FIFO buffer.
    ///
    /// This method should be called after the master has finished reading data
    /// to ensure that no stale data remains in the TX FIFO for the next transaction.
    ///
    /// **Recommended usage pattern** for repeated read operations:
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// # let mut i2c = I2c::new(
    /// #   peripherals.I2C0,
    /// #   Config::default(),
    /// # )?;
    /// loop {
    ///     // 1. Receive command from master
    ///     let mut cmd = [0u8; 1];
    ///     let bytes_read = i2c.read(&mut cmd)?;
    ///     
    ///     // 2. Prepare response
    ///     let response = process_command(cmd[0]);
    ///     
    ///     // 3. Write response to TX FIFO
    ///     i2c.write(&response)?;
    ///     
    ///     // 4. Wait for master to read (optional, depends on your protocol)
    ///     // The master will read the data...
    ///     
    ///     // 5. IMPORTANT: Clear TX FIFO after master reads
    ///     //    This prevents stale data in next transaction
    ///     i2c.clear_tx_fifo();
    /// }
    /// # {after_snippet}
    /// ```
    pub fn clear_tx_fifo(&mut self) {
        let driver = self.driver();
        driver
            .regs()
            .fifo_conf()
            .modify(|_, w| w.tx_fifo_rst().set_bit());
        // Small delay for reset to take effect
        for _ in 0..5 {
            unsafe { core::arch::asm!("nop") };
        }
        driver
            .regs()
            .fifo_conf()
            .modify(|_, w| w.tx_fifo_rst().clear_bit());
    }

    #[procmacros::doc_replace]
    /// Applies a new configuration.
    ///
    /// ## Errors
    ///
    /// A [`ConfigError`] variant will be returned if the slave address
    /// passed in config is invalid.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # {before_snippet}
    /// use esp_hal::i2c::slave::{Config, I2c};
    /// let mut i2c = I2c::new(peripherals.I2C0, Config::default())?;
    ///
    /// i2c.apply_config(&Config::default().with_address(0x66.into()))?;
    /// # {after_snippet}
    /// ```
    pub fn apply_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        config
            .address
            .validate()
            .map_err(|_| ConfigError::AddressInvalid)?;
        self.config.config = *config;
        self.driver().setup(config)?;
        Ok(())
    }
}

#[ram]
fn async_handler(info: &Info, state: &State) {
    // Disable all interrupts. The I2C Future will check events based on the
    // interrupt status bits.
    info.regs().int_ena().write(|w| unsafe { w.bits(0) });

    state.waker.wake();
}

/// Sets the filter with a supplied threshold in clock cycles for which a
/// pulse must be present to pass the filter
fn set_filter(
    register_block: &RegisterBlock,
    sda_threshold: Option<u8>,
    scl_threshold: Option<u8>,
) {
    cfg_if::cfg_if! {
        if #[cfg(i2c_master_separate_filter_config_registers)] {
            register_block.sda_filter_cfg().modify(|_, w| {
                if let Some(threshold) = sda_threshold {
                    unsafe { w.sda_filter_thres().bits(threshold) };
                }
                w.sda_filter_en().bit(sda_threshold.is_some())
            });
            register_block.scl_filter_cfg().modify(|_, w| {
                if let Some(threshold) = scl_threshold {
                    unsafe { w.scl_filter_thres().bits(threshold) };
                }
                w.scl_filter_en().bit(scl_threshold.is_some())
            });
        } else {
            register_block.filter_cfg().modify(|_, w| {
                if let Some(threshold) = sda_threshold {
                    unsafe { w.sda_filter_thres().bits(threshold) };
                }
                if let Some(threshold) = scl_threshold {
                    unsafe { w.scl_filter_thres().bits(threshold) };
                }
                w.sda_filter_en().bit(sda_threshold.is_some());
                w.scl_filter_en().bit(scl_threshold.is_some())
            });
        }
    }
}

/// Peripheral data describing a particular I2C instance.
#[doc(hidden)]
#[derive(Debug)]
#[non_exhaustive]
pub struct Info {
    /// Pointer to the register block for this I2C instance.
    ///
    /// Use [Self::register_block] to access the register block.
    pub register_block: *const RegisterBlock,

    /// System peripheral marker.
    pub peripheral: crate::system::Peripheral,

    /// Interrupt handler for the asynchronous operations of this I2C instance.
    pub async_handler: InterruptHandler,

    /// SCL output signal.
    pub scl_output: OutputSignal,

    /// SCL input signal.
    pub scl_input: InputSignal,

    /// SDA output signal.
    pub sda_output: OutputSignal,

    /// SDA input signal.
    pub sda_input: InputSignal,
}

impl Info {
    /// Returns the register block for this I2C instance.
    pub fn regs(&self) -> &RegisterBlock {
        unsafe { &*self.register_block }
    }

    /// Listen for the given interrupts
    fn enable_listen(&self, interrupts: EnumSet<Event>, enable: bool) {
        let reg_block = self.regs();

        reg_block.int_ena().modify(|_, w| {
            #[cfg(esp32)]
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_full().bit(enable),
                    Event::TxFifoEmpty => w.txfifo_empty().bit(enable),
                    Event::ByteReceived => w.rx_rec_full().bit(enable),
                    Event::ByteTransmitted => w.tx_send_empty().bit(enable),
                    Event::TransComplete => w.trans_complete().bit(enable),
                    Event::SlaveAddressed => w.trans_complete().bit(enable),
                    Event::StopDetected => w.end_detect().bit(enable),
                    Event::StartDetected => w.trans_start().bit(enable),
                };
            }
            #[cfg(not(esp32))]
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_wm().bit(enable),
                    Event::TxFifoEmpty => w.txfifo_wm().bit(enable),
                    Event::ByteReceived => w.rxfifo_wm().bit(enable),
                    Event::ByteTransmitted => w.txfifo_wm().bit(enable),
                    Event::TransComplete => w.trans_complete().bit(enable),
                    Event::SlaveAddressed => w.trans_complete().bit(enable),
                    Event::StopDetected => w.end_detect().bit(enable),
                    Event::StartDetected => w.trans_start().bit(enable),
                };
            }

            w
        });
    }

    fn interrupts(&self) -> EnumSet<Event> {
        let mut res = EnumSet::new();
        let reg_block = self.regs();

        let ints = reg_block.int_raw().read();

        #[cfg(esp32)]
        {
            if ints.rxfifo_full().bit_is_set() {
                res.insert(Event::RxFifoFull);
            }
            if ints.txfifo_empty().bit_is_set() {
                res.insert(Event::TxFifoEmpty);
            }
            if ints.rx_rec_full().bit_is_set() {
                res.insert(Event::ByteReceived);
            }
            if ints.tx_send_empty().bit_is_set() {
                res.insert(Event::ByteTransmitted);
            }
        }

        #[cfg(esp32c6)]
        {
            // ESP32-C6 uses watermark-based FIFO interrupts
            if ints.rxfifo_wm().bit_is_set() {
                res.insert(Event::RxFifoFull);
            }
            if ints.txfifo_wm().bit_is_set() {
                res.insert(Event::TxFifoEmpty);
            }
            // ESP32-C6 doesn't have separate byte-level interrupts
            // Map watermark interrupts to byte events as well
            if ints.rxfifo_wm().bit_is_set() {
                res.insert(Event::ByteReceived);
            }
            if ints.txfifo_wm().bit_is_set() {
                res.insert(Event::ByteTransmitted);
            }
        }

        #[cfg(not(any(esp32, esp32c6)))]
        {
            if ints.rxfifo_full().bit_is_set() {
                res.insert(Event::RxFifoFull);
            }
            if ints.txfifo_empty().bit_is_set() {
                res.insert(Event::TxFifoEmpty);
            }
            if ints.rx_rec_full().bit_is_set() {
                res.insert(Event::ByteReceived);
            }
            if ints.tx_send_empty().bit_is_set() {
                res.insert(Event::ByteTransmitted);
            }
        }

        if ints.trans_complete().bit_is_set() {
            res.insert(Event::TransComplete);
            res.insert(Event::StopDetected);
        }
        if ints.trans_complete().bit_is_set() {
            res.insert(Event::SlaveAddressed);
            res.insert(Event::StartDetected);
        }

        res
    }

    fn clear_interrupts(&self, interrupts: EnumSet<Event>) {
        let reg_block = self.regs();

        reg_block.int_clr().write(|w| {
            #[cfg(esp32)]
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_full().clear_bit_by_one(),
                    Event::TxFifoEmpty => w.txfifo_empty().clear_bit_by_one(),
                    Event::ByteReceived => w.rx_rec_full().clear_bit_by_one(),
                    Event::ByteTransmitted => w.tx_send_empty().clear_bit_by_one(),
                    Event::TransComplete => w.trans_complete().clear_bit_by_one(),
                    Event::SlaveAddressed => w.trans_complete().clear_bit_by_one(),
                    Event::StopDetected => w.end_detect().clear_bit_by_one(),
                    Event::StartDetected => w.trans_start().clear_bit_by_one(),
                };
            }

            #[cfg(not(esp32))]
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_wm().clear_bit_by_one(),
                    Event::TxFifoEmpty => w.txfifo_wm().clear_bit_by_one(),
                    Event::ByteReceived => w.rxfifo_wm().clear_bit_by_one(),
                    Event::ByteTransmitted => w.txfifo_wm().clear_bit_by_one(),
                    Event::TransComplete => w.trans_complete().clear_bit_by_one(),
                    Event::SlaveAddressed => w.trans_complete().clear_bit_by_one(),
                    Event::StopDetected => w.end_detect().clear_bit_by_one(),
                    Event::StartDetected => w.trans_start().clear_bit_by_one(),
                };
            }

            w
        });
    }
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.register_block, other.register_block)
    }
}

unsafe impl Sync for Info {}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Driver<'a> {
    info: &'a Info,
    state: &'a State,
    config: &'a DriverConfig,
}

impl Driver<'_> {
    fn regs(&self) -> &RegisterBlock {
        self.info.regs()
    }

    fn connect_pin(
        pin: crate::gpio::interconnect::OutputSignal<'_>,
        input: InputSignal,
        output: OutputSignal,
        guard: &mut PinGuard,
    ) {
        // Configure pin for open-drain without internal pull-ups (external pull-ups required)
        pin.set_output_high(true);

        pin.apply_output_config(
            &OutputConfig::default()
                .with_drive_mode(DriveMode::OpenDrain)
                .with_pull(Pull::None),
        );
        pin.set_output_enable(true);
        pin.set_input_enable(true);

        input.connect_to(&pin);

        *guard = interconnect::OutputSignal::connect_with_guard(pin, output);
    }

    fn init_slave(&self) {
        // ESP32-C6 specific: Ensure peripheral is properly reset before configuration
        #[cfg(esp32c6)]
        {
            // Reset the I2C peripheral to ensure clean state
            self.regs().ctr().write(|w| unsafe { w.bits(0) });

            // Small delay to ensure reset takes effect
            for _ in 0..100 {
                unsafe { core::arch::asm!("nop") };
            }
        }

        self.regs().ctr().write(|w| {
            // Set I2C controller to slave mode
            w.ms_mode().clear_bit();
            // Use open drain output for SDA and SCL
            w.sda_force_out().set_bit();
            w.scl_force_out().set_bit();
            // Use Most Significant Bit first for sending and receiving data
            w.tx_lsb_first().clear_bit();
            w.rx_lsb_first().clear_bit();

            #[cfg(esp32s2)]
            w.ref_always_on().set_bit();

            // Ensure that clock is enabled
            w.clk_en().set_bit()
        });

        // Note: Address mode (7-bit vs 10-bit) will be configured in setup()
        // after we know the actual address type from the config

        #[cfg(not(any(esp32, esp32c6)))]
        self.regs().ctr().modify(|_, w| {
            w.slave_addr_en().set_bit() // Enable slave address matching
        });

        // ESP32-C6 specific configuration for slave mode
        #[cfg(esp32c6)]
        {
            // For ESP32-C6, we need to be very explicit about slave mode setup
            // First, ensure we're definitely in slave mode
            self.regs().ctr().modify(|_, w| {
                w.ms_mode().clear_bit() // 0 = slave mode, 1 = master mode
            });

            // Configure slave-specific settings
            self.regs().ctr().modify(|_, w| {
                w.addr_broadcasting_en().clear_bit(); // Disable broadcasting - respond only to our address
                w.rx_lsb_first().clear_bit(); // MSB first for data
                w.tx_lsb_first().clear_bit(); // MSB first for data
                w.slv_tx_auto_start_en().clear_bit() // Disable auto TX start initially
            });
        }

        // Configure FIFO thresholds for proper interrupt generation
        #[cfg(esp32c6)]
        {
            // ESP32-C6 specific FIFO configuration for slave mode
            self.regs().fifo_conf().modify(|_, w| {
                unsafe {
                    // Set RX threshold to trigger interrupt before FIFO completely fills
                    // This allows software to read data before FIFO overflows and causes NACK
                    // At 30 bytes, there's still room for a few more bytes during interrupt latency
                    w.rxfifo_wm_thrhd().bits(30);
                    w.txfifo_wm_thrhd().bits(1); // Interrupt when TX FIFO has space
                }
                // Configure register-based mode (FIFO address configuration)
                // When enabled: first byte after slave address is treated as register address
                // When disabled: all bytes stored in RX FIFO (raw data stream mode)
                if self.config.config.register_based_mode {
                    w.fifo_addr_cfg_en().set_bit()
                } else {
                    w.fifo_addr_cfg_en().clear_bit()
                };
                w
            });
        }

        #[cfg(not(any(esp32, esp32c6)))]
        self.regs().fifo_conf().modify(|_, w| {
            unsafe {
                // Set RX threshold to trigger before FIFO fills to prevent overflow
                w.rxfifo_wm_thrhd().bits(30);
                w.txfifo_wm_thrhd().bits(I2C_FIFO_SIZE as u8 - 1); // Generate interrupt when FIFO nearly empty
            }
            w
        });

        #[cfg(esp32)]
        self.regs().fifo_conf().modify(|_, w| {
            unsafe {
                // Set RX threshold to trigger before FIFO fills to prevent overflow
                w.rxfifo_full_thrhd().bits(30);
                w.txfifo_empty_thrhd().bits(I2C_FIFO_SIZE as u8 - 1); // Generate interrupt when FIFO nearly empty
            }
            w
        });
    }

    /// Configures the I2C peripheral in slave mode
    fn setup(&self, config: &Config) -> Result<(), ConfigError> {
        self.init_slave();

        // Set slave address
        let is_10bit = config.address.is_ten_bit();
        let addr_value = config.address.as_u16();

        self.regs().slave_addr().write(|w| unsafe {
            w.slave_addr().bits(addr_value);
            w.addr_10bit_en().bit(is_10bit)
        });

        // ESP32-C6 specific: Critical slave acknowledgment setup
        #[cfg(esp32c6)]
        {
            // ESP32-C6 requires specific sequence for slave address acknowledgment

            // 1. Ensure we're in slave mode
            self.regs().ctr().modify(|_, w| {
                w.ms_mode().clear_bit() // 0 = slave mode
            });

            // 2. Configure the slave address register properly
            self.regs().slave_addr().write(|w| unsafe {
                w.slave_addr().bits(addr_value);
                w.addr_10bit_en().bit(is_10bit)
            });

            // 3. Enable slave address matching by configuring control register
            self.regs().ctr().modify(|_, w| {
                w.addr_10bit_rw_check_en().bit(is_10bit); // Enable 10-bit check if needed
                w.slv_tx_auto_start_en().set_bit() // Enable auto TX start for slave
            });
        }

        // Ensure slave address matching is enabled after setting address
        #[cfg(not(any(esp32, esp32c6)))]
        self.regs().ctr().modify(|_, w| {
            w.addr_10bit_en().bit(is_10bit);
            w.slave_addr_en().set_bit()
        });

        // Configure clock stretching
        #[cfg(esp32c6)]
        self.regs().scl_stretch_conf().modify(|_, w| {
            // Clock stretching with automatic SCL release when TX FIFO is ready.
            // The release logic is implemented in release_scl_stretch() method.
            // This prevents the previous bug where SCL could remain low indefinitely.
            //
            // ESP32 (original) master compatibility note: ESP32 masters have poor
            // clock stretching support and may timeout. For maximum compatibility
            // with ESP32 masters, disable clock stretching:
            // Config::default().with_clock_stretch_enable(false)
            //
            // For large packets (≥30 bytes) without clock stretching, use interrupt-driven
            // or async reception to prevent FIFO overflow.
            w.slave_scl_stretch_en().bit(config.clock_stretch_enable);
            unsafe {
                w.stretch_protect_num().bits(1023); // Max stretch timeout (10-bit field, i2c_sclk cycles)
            }
            // Disable byte ACK control features that can cause clock holding
            w.slave_byte_ack_ctl_en().clear_bit();
            w.slave_byte_ack_lvl().clear_bit()
        });

        #[cfg(not(any(esp32, esp32c6)))]
        self.regs().scl_stretch_conf().modify(|_, w| {
            w.slave_scl_stretch_en().bit(config.clock_stretch_enable);
            // Max stretch timeout to prevent hanging (10-bit field, i2c_sclk cycles)
            unsafe { w.stretch_protect_num().bits(1023) }
        });

        // Configure timeout settings to ensure proper operation
        #[cfg(any(esp32c3, esp32c6, esp32h2, esp32s2, esp32s3))]
        self.regs().to().modify(|_, w| {
            unsafe {
                w.time_out_en().set_bit();
                w.time_out_value().bits(0x10); // Set a reasonable timeout value
            }
            w
        });

        // Configure filters
        let sda_filter = if config.sda_filter_enable {
            Some(config.sda_filter_threshold)
        } else {
            None
        };
        let scl_filter = if config.scl_filter_enable {
            Some(config.scl_filter_threshold)
        } else {
            None
        };
        set_filter(self.regs(), sda_filter, scl_filter);

        // Reset FIFO
        self.reset_fifo();

        // ESP32-C6 specific: Final slave mode activation
        #[cfg(esp32c6)]
        {
            // Clear any pending interrupts that might interfere
            self.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });

            // The key for ESP32-C6 acknowledgment: enable the slave to actively monitor the bus
            // ESP32-C6 has a different hardware implementation that requires explicit activation
            self.regs().ctr().modify(|_, w| {
                // Ensure slave mode is active
                w.ms_mode().clear_bit();
                // Force open-drain outputs
                w.sda_force_out().set_bit();
                w.scl_force_out().set_bit();
                // Enable clock
                w.clk_en().set_bit()
            });
        }

        self.update_registers();

        Ok(())
    }

    /// Resets the transmit and receive FIFO buffers
    fn reset_fifo(&self) {
        #[cfg(not(esp32))]
        {
            self.regs().fifo_conf().modify(|_, w| {
                w.tx_fifo_rst().set_bit();
                w.rx_fifo_rst().set_bit()
            });

            self.regs().fifo_conf().modify(|_, w| {
                w.tx_fifo_rst().clear_bit();
                w.rx_fifo_rst().clear_bit()
            });
        }

        #[cfg(esp32)]
        {
            self.regs().fifo_conf().modify(|_, w| {
                w.tx_fifo_rst().set_bit();
                w.rx_fifo_rst().set_bit()
            });

            self.regs().fifo_conf().modify(|_, w| {
                w.tx_fifo_rst().clear_bit();
                w.rx_fifo_rst().clear_bit()
            });
        }

        // Clear all interrupts
        self.clear_all_interrupts();

        self.update_registers();
    }

    /// Clears all pending interrupts for the I2C peripheral.
    fn clear_all_interrupts(&self) {
        self.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });
    }

    fn update_registers(&self) {
        #[cfg(esp32)]
        self.regs().ctr().modify(|_, w| w.trans_start().set_bit());

        #[cfg(esp32c6)]
        {
            // ESP32-C6 critical sequence for slave acknowledgment:

            // 1. Update configuration
            self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());

            // 2. Start slave operation - this is what actually enables acknowledgment
            // The hardware needs this bit set to begin monitoring the I2C bus for its address
            self.regs().ctr().modify(|_, w| {
                w.trans_start().set_bit();
                w.ms_mode().clear_bit() // Double-ensure slave mode
            });

            // 3. Additional ESP32-C6 specific: enable slave reception
            self.regs().ctr().modify(|_, w| {
                w.slv_tx_auto_start_en().set_bit() // This might be key for acknowledgment
            });
        }

        #[cfg(not(any(esp32, esp32c6)))]
        // Other ESP32 variants (S2, S3, etc.) use ctr2
        self.regs().ctr2().modify(|_, w| w.conf_upgate().set_bit());
    }

    fn wait_for_rx_data(&self) -> Result<bool, Error> {
        // ESP-IDF approach: Wait for interrupts, not poll registers
        // The hardware will signal when:
        // 1. RX FIFO reaches watermark (rxfifo_wm_int)
        // 2. Transaction complete / STOP detected (trans_complete_int)
        //
        // Returns: true if write_read detected (timeout), false if normal write (STOP)

        let start = Instant::now();
        let timeout = crate::time::Duration::from_millis(self.config.config.timeout_ms as u64);

        // Clear any pending interrupts from previous transactions
        self.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });

        // Wait for trans_complete interrupt (STOP condition for normal writes)
        // OR rxfifo_wm interrupt with timeout (for write_read)
        loop {
            let int_raw = self.regs().int_raw().read();
            let status = self.regs().sr().read();

            // Check for transaction complete (STOP detected)
            if int_raw.trans_complete().bit_is_set() {
                // Normal write transaction - STOP received
                // Clear the interrupt
                self.regs()
                    .int_clr()
                    .write(|w| w.trans_complete().clear_bit_by_one());

                // Brief delay to ensure all bytes are in FIFO
                for _ in 0..50 {
                    unsafe { core::arch::asm!("nop") };
                }
                return Ok(false); // Normal write with STOP
            }

            // Check for data in FIFO (could be write_read or normal write in progress)
            #[cfg(esp32c6)]
            let has_data = status.rxfifo_cnt().bits() > 0;

            #[cfg(not(esp32c6))]
            let has_data = !status.rx_fifo_empty().bit_is_set();

            if has_data {
                // Data present - wait for STOP to arrive
                // For write_read: no STOP will arrive (use longer timeout to detect)
                // For normal write: STOP should arrive after all bytes transmitted
                //
                // Timing calculation at 100kHz I2C:
                // - Each byte: ~90µs (including ACK)
                // - 4 bytes: ~360µs
                // - STOP: ~10µs
                // - Total: ~370µs minimum
                // - Use 1000µs (1ms) timeout to be safe
                let wait_start = Instant::now();
                let stop_wait = crate::time::Duration::from_micros(1000);

                loop {
                    let int_now = self.regs().int_raw().read();

                    if int_now.trans_complete().bit_is_set() {
                        // STOP arrived - normal write complete
                        self.regs()
                            .int_clr()
                            .write(|w| w.trans_complete().clear_bit_by_one());
                        for _ in 0..50 {
                            unsafe { core::arch::asm!("nop") };
                        }
                        return Ok(false); // Normal write with STOP
                    }

                    if Instant::now() > wait_start + stop_wait {
                        // No STOP after 1ms timeout - assume write_read
                        return Ok(true); // write_read detected
                    }

                    // Check overall timeout
                    if Instant::now() > start + timeout {
                        return Err(Error::Timeout);
                    }
                }
            }

            // Check for errors
            if int_raw.arbitration_lost().bit_is_set() {
                self.regs()
                    .int_clr()
                    .write(|w| w.arbitration_lost().clear_bit_by_one());
                return Err(Error::ArbitrationLost);
            }
            if int_raw.time_out().bit_is_set() {
                self.regs()
                    .int_clr()
                    .write(|w| w.time_out().clear_bit_by_one());
                return Err(Error::Timeout);
            }

            // Check overall timeout
            if Instant::now() > start + timeout {
                return Err(Error::Timeout);
            }

            // Small delay to avoid busy-spinning
            for _ in 0..10 {
                unsafe { core::arch::asm!("nop") };
            }
        }
    }

    fn read_fifo(&self, buffer: &mut [u8]) -> usize {
        let mut count = 0;

        // ESP32-C6 specific: Check if there's actually data in the FIFO
        #[cfg(esp32c6)]
        {
            let status = self.regs().sr().read();
            let fifo_count = status.rxfifo_cnt().bits();

            // Debug: On ESP32-C6, the first byte might be consumed by address matching
            // We need to check if the FIFO count matches what we expect
            if fifo_count == 0 {
                return 0; // No data available
            }
        }

        for byte in buffer.iter_mut() {
            let status = self.regs().sr().read();

            #[cfg(not(esp32))]
            {
                if status.rxfifo_cnt().bits() == 0 {
                    break;
                }
            }

            #[cfg(esp32)]
            {
                if status.rxfifo_cnt().bits() == 0 {
                    break;
                }
            }

            *byte = read_fifo(self.regs());
            count += 1;
        }

        // ESP32-C6 specific: After reading, reset transaction state for next operation
        // BUT: Do NOT release clock stretch - let write() handle that
        // For write_read transactions, we need to keep stretching until response is loaded
        #[cfg(esp32c6)]
        {
            // Clear all transaction-related interrupts
            self.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });

            // DO NOT clear stretch here - commented out:
            // self.regs().scl_stretch_conf().modify(|_, w| {
            //     w.slave_scl_stretch_clr().set_bit()
            // });

            // Reset the controller state for fresh transaction
            self.regs().ctr().modify(|_, w| {
                w.ms_mode().clear_bit(); // Ensure slave mode
                w.slv_tx_auto_start_en().set_bit(); // Enable auto TX start
                w.sda_force_out().set_bit(); // Force SDA output
                w.scl_force_out().set_bit() // Force SCL output
            });

            // Update configuration
            self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());
        }

        count
    }

    fn write_fifo(&self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.len() > I2C_FIFO_SIZE {
            return Err(Error::FifoExceeded);
        }

        // ESP32-C6 specific: Prepare for transmission
        #[cfg(esp32c6)]
        {
            // Only call prepare_slave_tx() if auto-clear is NOT enabled
            // If auto-clear is enabled, read() already prepared TX FIFO
            // Calling prepare_slave_tx() again during write_read() causes race conditions
            if !self.config.config.clear_tx_on_write {
                self.prepare_slave_tx();
            }

            // Check if we can write to TX FIFO
            let status = self.regs().sr().read();
            if status.txfifo_cnt().bits() >= I2C_FIFO_SIZE as u8 {
                return Err(Error::TxFifoOverflow);
            }
        }

        for &byte in buffer {
            // ESP32-C6: Check FIFO space before each write
            #[cfg(esp32c6)]
            {
                let status = self.regs().sr().read();
                if status.txfifo_cnt().bits() >= I2C_FIFO_SIZE as u8 {
                    return Err(Error::TxFifoOverflow);
                }
            }

            write_fifo(self.regs(), byte);
        }

        // ESP32-C6 specific: Finalize and release the bus
        #[cfg(esp32c6)]
        {
            // After loading TX FIFO, minimal intervention - let hardware handle transmission
            // The slv_tx_auto_start_en was already set during init_slave/setup
            // We just need to ensure configuration is synchronized

            // Update configuration to ensure FIFO writes are visible to hardware
            self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());

            // Brief delay for hardware synchronization
            for _ in 0..50 {
                unsafe { core::arch::asm!("nop") };
            }

            // Release clock stretch so master can read the data
            // This is the critical moment - we've loaded fresh data and can release
            self.release_scl_stretch();
        }

        Ok(())
    }

    fn check_errors(&self) -> Result<(), Error> {
        let interrupts = self.regs().int_raw().read();

        if interrupts.arbitration_lost().bit_is_set() {
            return Err(Error::ArbitrationLost);
        }

        if interrupts.time_out().bit_is_set() {
            return Err(Error::Timeout);
        }

        Ok(())
    }

    /// ESP32-C6 specific: Prepare slave for transmission
    #[cfg(esp32c6)]
    fn prepare_slave_tx(&self) {
        // CRITICAL: Only clear TX FIFO, NOT RX FIFO
        // Clearing RX FIFO here would interfere with write_read() transactions
        // where the master has already written data that we haven't read yet

        // Step 1: Update configuration to flush any pending operations
        self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());
        for _ in 0..30 {
            unsafe { core::arch::asm!("nop") };
        }

        // Step 2: Reset ONLY TX FIFO to clear stale response data
        self.regs()
            .fifo_conf()
            .modify(|_, w| w.tx_fifo_rst().set_bit());

        // Critical delay for hardware to process reset
        for _ in 0..100 {
            unsafe { core::arch::asm!("nop") };
        }

        // Step 3: Clear reset bit
        self.regs()
            .fifo_conf()
            .modify(|_, w| w.tx_fifo_rst().clear_bit());

        // Wait for FIFO to stabilize
        for _ in 0..100 {
            unsafe { core::arch::asm!("nop") };
        }

        // Step 4: Verify TX FIFO is empty, retry up to 3 times if needed
        for attempt in 0..3 {
            let status = self.regs().sr().read();
            let fifo_count = status.txfifo_cnt().bits();

            if fifo_count == 0 {
                break; // Success!
            }

            if attempt < 2 {
                // Try again with longer delays - ONLY clear TX FIFO
                self.regs()
                    .fifo_conf()
                    .modify(|_, w| w.tx_fifo_rst().set_bit());
                for _ in 0..150 {
                    unsafe { core::arch::asm!("nop") };
                }
                self.regs()
                    .fifo_conf()
                    .modify(|_, w| w.tx_fifo_rst().clear_bit());
                for _ in 0..150 {
                    unsafe { core::arch::asm!("nop") };
                }
            }
        }

        // Clear any previous transmission state
        self.regs().int_clr().write(|w| unsafe { w.bits(0x1FFF) });

        // Configure clock stretching per config setting
        // NOTE: See setup() for important compatibility warnings about clock stretching
        // with ESP32 master peripherals
        self.regs().scl_stretch_conf().modify(|_, w| {
            w.slave_scl_stretch_en()
                .bit(self.config.config.clock_stretch_enable);
            unsafe {
                w.stretch_protect_num()
                    .bits(if self.config.config.clock_stretch_enable {
                        1023
                    } else {
                        0
                    });
            }
            w.slave_byte_ack_ctl_en().clear_bit();
            w.slave_byte_ack_lvl().clear_bit();
            w
        });

        // Ensure slave TX is properly configured
        self.regs().ctr().modify(|_, w| {
            w.ms_mode().clear_bit(); // Ensure slave mode
            w.slv_tx_auto_start_en().set_bit(); // Enable auto TX start
            w.sda_force_out().set_bit(); // Force SDA output
            w.scl_force_out().set_bit() // Force SCL output
        });

        // Set TX FIFO watermark to trigger interrupt when ready
        self.regs().fifo_conf().modify(|_, w| {
            unsafe {
                w.txfifo_wm_thrhd().bits(0); // Trigger immediately when any space available
            }
            // Configure register-based mode consistently with init_slave()
            // In TX operations, this should match the RX configuration
            if self.config.config.register_based_mode {
                w.fifo_addr_cfg_en().set_bit()
            } else {
                w.fifo_addr_cfg_en().clear_bit()
            };
            w
        });

        // Update configuration
        self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());
    }

    /// ESP32-C6 specific: Manually release clock stretching
    ///
    /// This function must be called after `write()` in write_read scenarios when
    /// `clear_tx_on_write` is enabled. The hardware requires significant time
    /// (~7-10ms) after loading TX FIFO before data is stable for transmission.
    ///
    /// ## Example for write_read handling:
    ///
    /// ```rust,no_run
    /// # use esp_hal::i2c::slave::I2c;
    /// # let mut slave: I2c<'_, esp_hal::peripherals::I2C0, esp_hal::Async> = unsafe { core::mem::zeroed() };
    /// # let mut rx_buffer = [0u8; 32];
    /// # let response = [0x43u8];
    /// // In write_read transaction handler:
    /// let count = slave.read(&mut rx_buffer)?;
    /// // Process command and prepare response
    /// slave.write(&response)?;  // Loads TX FIFO but doesn't release stretch
    ///
    /// // CRITICAL: Wait for hardware to stabilize (empirically ~7-10ms needed)
    /// for _ in 0..700000 { unsafe { core::arch::asm!("nop") }; }
    ///
    /// // Now release stretch to let master read the response
    /// slave.release_scl_stretch();
    /// # Ok::<(), esp_hal::i2c::Error>(())
    /// ```
    #[cfg(esp32c6)]
    pub fn release_scl_stretch(&self) {
        if !self.config.config.clock_stretch_enable {
            return; // No stretch to release if disabled
        }

        // CRITICAL: Use the proper ESP-IDF mechanism to clear clock stretching
        // The slave_scl_stretch_clr bit tells hardware to release the SCL line
        self.regs()
            .scl_stretch_conf()
            .modify(|_, w| w.slave_scl_stretch_clr().set_bit());

        // Significant delay to let hardware process the clear
        // This is critical - the hardware needs time to:
        // 1. Process the stretch clear command
        // 2. Release the SCL line
        // 3. Update internal state machines
        for _ in 0..50 {
            unsafe { core::arch::asm!("nop") };
        }

        // Update configuration to ensure state machine is synchronized
        self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());

        // Final delay to ensure all hardware changes are applied
        for _ in 0..30 {
            unsafe { core::arch::asm!("nop") };
        }
    }

    /// ESP32-C6 specific: Force release of clock stretching (emergency release)
    #[cfg(esp32c6)]
    fn force_release_scl_stretch(&self) {
        if !self.config.config.clock_stretch_enable {
            return;
        }

        // Multi-step aggressive release for stuck transactions
        // Step 1: Force clear using the clear bit multiple times if needed
        for _ in 0..3 {
            self.regs()
                .scl_stretch_conf()
                .modify(|_, w| w.slave_scl_stretch_clr().set_bit());

            // Delay between attempts
            for _ in 0..10 {
                unsafe { core::arch::asm!("nop") };
            }
        }

        // Step 2: Temporarily disable and re-enable stretching to reset state
        self.regs()
            .scl_stretch_conf()
            .modify(|_, w| w.slave_scl_stretch_en().clear_bit());

        // Longer delay for state reset
        for _ in 0..20 {
            unsafe { core::arch::asm!("nop") };
        }

        // Re-enable if originally enabled
        self.regs()
            .scl_stretch_conf()
            .modify(|_, w| w.slave_scl_stretch_en().set_bit());

        // Step 3: Update configuration to ensure changes take effect
        self.regs().ctr().modify(|_, w| w.conf_upgate().set_bit());

        // Final delay
        for _ in 0..10 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}

fn read_fifo(register_block: &RegisterBlock) -> u8 {
    cfg_if::cfg_if! {
        if #[cfg(esp32s2)] {
            let peri_offset = register_block as *const _ as usize - crate::peripherals::I2C0::ptr() as usize;
            let fifo_ptr = (property!("i2c_master.i2c0_data_register_ahb_address") + peri_offset) as *mut u32;
            unsafe { (fifo_ptr.read_volatile() & 0xff) as u8 }
        } else {
            register_block.data().read().fifo_rdata().bits()
        }
    }
}

fn write_fifo(register_block: &RegisterBlock, data: u8) {
    cfg_if::cfg_if! {
        if #[cfg(any(esp32, esp32s2))] {
            let peri_offset = register_block as *const _ as usize - crate::peripherals::I2C0::ptr() as usize;
            let fifo_ptr = (property!("i2c_master.i2c0_data_register_ahb_address") + peri_offset) as *mut u32;
            unsafe {
                fifo_ptr.write_volatile(data as u32);
            }
        } else {
            register_block
                .data()
                .write(|w| unsafe { w.fifo_rdata().bits(data) });
        }
    }
}

/// Peripheral state for an I2C instance.
#[doc(hidden)]
#[non_exhaustive]
pub struct State {
    /// Waker for the asynchronous operations.
    pub waker: AtomicWaker,
}

/// A peripheral singleton compatible with the I2C slave driver.
pub trait Instance: crate::private::Sealed + any::Degrade {
    #[doc(hidden)]
    /// Returns the peripheral data and state describing this instance.
    fn parts(&self) -> (&Info, &State);

    /// Returns the peripheral data describing this instance.
    #[doc(hidden)]
    #[inline(always)]
    fn info(&self) -> &Info {
        self.parts().0
    }

    /// Returns the peripheral state for this instance.
    #[doc(hidden)]
    #[inline(always)]
    fn state(&self) -> &State {
        self.parts().1
    }
}

for_each_i2c_slave!(
    ($inst:ident, $peri:ident, $scl:ident, $sda:ident) => {
        impl Instance for crate::peripherals::$inst<'_> {
            fn parts(&self) -> (&Info, &State) {
                #[handler]
                #[ram]
                pub(super) fn irq_handler() {
                    async_handler(&PERIPHERAL, &STATE);
                }

                static STATE: State = State {
                    waker: AtomicWaker::new(),
                };

                static PERIPHERAL: Info = Info {
                    register_block: crate::peripherals::$inst::ptr(),
                    peripheral: crate::system::Peripheral::$peri,
                    async_handler: irq_handler,
                    scl_output: OutputSignal::$scl,
                    scl_input: InputSignal::$scl,
                    sda_output: OutputSignal::$sda,
                    sda_input: InputSignal::$sda,
                };
                (&PERIPHERAL, &STATE)
            }
        }
    };
);

any_peripheral! {
    /// Any I2C peripheral.
    pub peripheral AnyI2c<'d> {
        #[cfg(i2c_slave_i2c0)]
        I2C0(crate::peripherals::I2C0<'d>),
        #[cfg(i2c_slave_i2c1)]
        I2C1(crate::peripherals::I2C1<'d>),
    }
}

impl Instance for AnyI2c<'_> {
    fn parts(&self) -> (&Info, &State) {
        any::delegate!(self, i2c => { i2c.parts() })
    }
}

impl AnyI2c<'_> {
    fn bind_peri_interrupt(&self, handler: interrupt::IsrCallback) {
        any::delegate!(self, i2c => { i2c.bind_peri_interrupt(handler) })
    }

    fn disable_peri_interrupt(&self) {
        any::delegate!(self, i2c => { i2c.disable_peri_interrupt() })
    }

    fn enable_peri_interrupt(&self, priority: crate::interrupt::Priority) {
        any::delegate!(self, i2c => { i2c.enable_peri_interrupt(priority) })
    }

    fn set_interrupt_handler(&self, handler: InterruptHandler) {
        self.disable_peri_interrupt();

        self.info().enable_listen(EnumSet::all(), false);
        self.info().clear_interrupts(EnumSet::all());

        self.bind_peri_interrupt(handler.handler());
        self.enable_peri_interrupt(handler.priority());
    }
}
