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
//! ```rust, no_run
//! # {before_snippet}
//! use esp_hal::i2c::slave::Config;
//!
//! let config = Config::default().with_address(0x55);
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

use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use enumset::{EnumSet, EnumSetType};

use crate::{
    Async,
    Blocking,
    DriverMode,
    asynch::AtomicWaker,
    clock::Clocks,
    gpio::{
        DriveMode,
        InputSignal,
        OutputConfig,
        OutputSignal,
        PinGuard,
        Pull,
        interconnect::{self, PeripheralOutput},
    },
    handler,
    interrupt::{self, InterruptHandler},
    pac::i2c0::{RegisterBlock, COMD},
    private,
    ram,
    system::PeripheralGuard,
    time::{Duration, Instant},
};

const I2C_FIFO_SIZE: usize = property!("i2c_master.fifo_size");

/// Representation of I2C slave address.
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
}

impl I2cAddress {
    fn validate(&self) -> Result<(), Error> {
        match self {
            I2cAddress::SevenBit(addr) => {
                if *addr > 0x7F {
                    return Err(Error::AddressInvalid(*self));
                }
            }
        }

        Ok(())
    }
}

impl From<u8> for I2cAddress {
    fn from(value: u8) -> Self {
        I2cAddress::SevenBit(value)
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
    /// Default value: 0x55.
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
            for event in events {
                match event {
                    Event::RxFifoFull => w.rxfifo_full().set_bit(),
                    Event::TxFifoEmpty => w.txfifo_empty().set_bit(),
                    Event::ByteReceived => w.rx_rec_full().set_bit(),
                    Event::ByteTransmitted => w.tx_send_empty().set_bit(),
                    Event::TransComplete => w.trans_complete().set_bit(),
                    Event::SlaveAddressed => w.slave_addressed().set_bit(),
                    Event::StopDetected => w.trans_complete().set_bit(), // Use trans_complete for STOP
                    Event::StartDetected => w.slave_addressed().set_bit(), // Use slave_addressed for START
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
        driver.wait_for_rx_data();
        Ok(driver.read_fifo(buffer))
    }

    #[procmacros::doc_replace]
    /// Writes data to be sent to the master
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
    /// i2c.write(&[0xAA, 0xBB])?;
    /// # {after_snippet}
    /// ```
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Err(Error::ZeroLengthInvalid);
        }

        self.driver().write_fifo(buffer)?;
        Ok(())
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
        config.address.validate().map_err(|_| ConfigError::AddressInvalid)?;
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
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_full().bit(enable),
                    Event::TxFifoEmpty => w.txfifo_empty().bit(enable),
                    Event::ByteReceived => w.rx_rec_full().bit(enable),
                    Event::ByteTransmitted => w.tx_send_empty().bit(enable),
                    Event::TransComplete => w.trans_complete().bit(enable),
                    Event::SlaveAddressed => w.slave_addressed().bit(enable),
                    Event::StopDetected => w.trans_complete().bit(enable),
                    Event::StartDetected => w.slave_addressed().bit(enable),
                };
            }
            w
        });
    }

    fn interrupts(&self) -> EnumSet<Event> {
        let mut res = EnumSet::new();
        let reg_block = self.regs();

        let ints = reg_block.int_raw().read();

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
        if ints.trans_complete().bit_is_set() {
            res.insert(Event::TransComplete);
            res.insert(Event::StopDetected);
        }
        if ints.slave_addressed().bit_is_set() {
            res.insert(Event::SlaveAddressed);
            res.insert(Event::StartDetected);
        }

        res
    }

    fn clear_interrupts(&self, interrupts: EnumSet<Event>) {
        let reg_block = self.regs();

        reg_block.int_clr().write(|w| {
            for interrupt in interrupts {
                match interrupt {
                    Event::RxFifoFull => w.rxfifo_full().clear_bit_by_one(),
                    Event::TxFifoEmpty => w.txfifo_empty().clear_bit_by_one(),
                    Event::ByteReceived => w.rx_rec_full().clear_bit_by_one(),
                    Event::ByteTransmitted => w.tx_send_empty().clear_bit_by_one(),
                    Event::TransComplete => w.trans_complete().clear_bit_by_one(),
                    Event::SlaveAddressed => w.slave_addressed().clear_bit_by_one(),
                    Event::StopDetected => w.trans_complete().clear_bit_by_one(),
                    Event::StartDetected => w.slave_addressed().clear_bit_by_one(),
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
        // Configure pin for open-drain with pull-up
        pin.set_output_high(true);

        pin.apply_output_config(
            &OutputConfig::default()
                .with_drive_mode(DriveMode::OpenDrain)
                .with_pull(Pull::Up),
        );
        pin.set_output_enable(true);
        pin.set_input_enable(true);

        input.connect_to(&pin);

        *guard = interconnect::OutputSignal::connect_with_guard(pin, output);
    }

    fn init_slave(&self) {
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
    }

    /// Configures the I2C peripheral in slave mode
    fn setup(&self, config: &Config) -> Result<(), ConfigError> {
        self.init_slave();

        // Set slave address
        match config.address {
            I2cAddress::SevenBit(addr) => {
                self.regs().slave_addr().write(|w| unsafe {
                    w.slave_addr().bits(addr as u16);
                    w.addr_10bit_en().clear_bit()
                });
            }
        }

        // Configure clock stretching
        #[cfg(not(esp32))]
        self.regs().scl_stretch_conf().modify(|_, w| {
            w.slave_scl_stretch_en().bit(config.clock_stretch_enable)
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

        #[cfg(not(esp32))]
        self.regs().ctr2().modify(|_, w| w.conf_upgate().set_bit());
    }

    fn wait_for_rx_data(&self) {
        loop {
            let status = self.regs().sr().read();
            
            #[cfg(not(esp32))]
            if status.rxfifo_cnt().bits() > 0 {
                break;
            }
            
            #[cfg(esp32)]
            if !status.rx_fifo_empty().bit_is_set() {
                break;
            }
        }
    }

    fn read_fifo(&self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        
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
                if status.rx_fifo_empty().bit_is_set() {
                    break;
                }
            }
            
            *byte = read_fifo(self.regs());
            count += 1;
        }
        
        count
    }

    fn write_fifo(&self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.len() > I2C_FIFO_SIZE {
            return Err(Error::FifoExceeded);
        }

        for &byte in buffer {
            write_fifo(self.regs(), byte);
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

for_each_i2c_master!(
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

crate::any_peripheral! {
    /// Any I2C peripheral.
    pub peripheral AnyI2c<'d> {
        #[cfg(i2c_master_i2c0)]
        I2c0(crate::peripherals::I2C0<'d>),
        #[cfg(i2c_master_i2c1)]
        I2c1(crate::peripherals::I2C1<'d>),
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

mod any {
    use super::*;

    pub trait Degrade {
        fn degrade(self) -> AnyI2c<'static>;
    }

    impl Degrade for AnyI2c<'_> {
        fn degrade(self) -> AnyI2c<'static> {
            AnyI2c::I2c0(unsafe { core::mem::transmute(self) })
        }
    }

    macro_rules! impl_degrade {
        ($inst:ident) => {
            impl Degrade for crate::peripherals::$inst<'_> {
                fn degrade(self) -> AnyI2c<'static> {
                    AnyI2c::$inst(unsafe { core::mem::transmute(self) })
                }
            }
        };
    }

    #[cfg(i2c_master_i2c0)]
    impl_degrade!(I2c0);
    #[cfg(i2c_master_i2c1)]
    impl_degrade!(I2c1);
}
