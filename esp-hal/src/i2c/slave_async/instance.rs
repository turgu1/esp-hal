//! Peripheral instance definitions for I2C slave

use crate::{
    gpio::{InputSignal, OutputSignal},
    interrupt::{InterruptHandler, Priority},
    pac::i2c0::RegisterBlock,
    peripheral::Peripheral,
    system,
};

use super::{Event, driver::async_handler, state::State};

/// Peripheral instance information
#[doc(hidden)]
#[derive(Debug)]
pub struct Info {
    /// Pointer to the register block
    pub register_block: *const RegisterBlock,

    /// System peripheral marker
    pub peripheral: system::Peripheral,

    /// Async interrupt handler
    pub async_handler: InterruptHandler,

    /// SCL output signal
    pub scl_output: OutputSignal,

    /// SCL input signal
    pub scl_input: InputSignal,

    /// SDA output signal
    pub sda_output: OutputSignal,

    /// SDA input signal
    pub sda_input: InputSignal,
}

impl Info {
    /// Get register block
    pub fn regs(&self) -> &RegisterBlock {
        unsafe { &*self.register_block }
    }

    /// Enable or disable interrupt listening for specific events
    pub fn enable_listen(&self, interrupts: enumset::EnumSet<Event>, enable: bool) {
        let regs = self.regs();

        regs.int_ena().modify(|_, w| {
            for interrupt in interrupts {
                match interrupt {
                    #[cfg(esp32)]
                    Event::RxFifoThreshold => w.rxfifo_full().bit(enable),
                    #[cfg(not(esp32))]
                    Event::RxFifoThreshold => w.rxfifo_wm().bit(enable),

                    #[cfg(esp32)]
                    Event::TxFifoThreshold => w.txfifo_empty().bit(enable),
                    #[cfg(not(esp32))]
                    Event::TxFifoThreshold => w.txfifo_wm().bit(enable),

                    #[cfg(esp32)]
                    Event::ByteReceived => w.rx_rec_full().bit(enable),
                    #[cfg(not(esp32))]
                    Event::ByteReceived => w.rxfifo_wm().bit(enable),

                    #[cfg(esp32)]
                    Event::ByteTransmitted => w.tx_send_empty().bit(enable),
                    #[cfg(not(esp32))]
                    Event::ByteTransmitted => w.txfifo_wm().bit(enable),

                    Event::TransComplete => w.trans_complete().bit(enable),
                    Event::AddressMatch | Event::StartDetected => w.trans_start().bit(enable),
                    Event::StopDetected => w.end_detect().bit(enable),
                    Event::ArbitrationLost => w.arbitration_lost().bit(enable),
                    Event::Timeout => w.time_out().bit(enable),

                    #[cfg(not(esp32))]
                    Event::RxFifoOverflow => w.rxfifo_ovf().bit(enable),
                    #[cfg(esp32)]
                    Event::RxFifoOverflow => w,

                    #[cfg(not(esp32))]
                    Event::TxFifoUnderflow => w.txfifo_udf().bit(enable),
                    #[cfg(esp32)]
                    Event::TxFifoUnderflow => w,
                };
            }
            w
        });
    }

    /// Clear specific interrupts
    pub fn clear_interrupts(&self, interrupts: enumset::EnumSet<Event>) {
        let regs = self.regs();

        regs.int_clr().write(|w| {
            for interrupt in interrupts {
                match interrupt {
                    #[cfg(esp32)]
                    Event::RxFifoThreshold => w.rxfifo_full().clear_bit_by_one(),
                    #[cfg(not(esp32))]
                    Event::RxFifoThreshold => w.rxfifo_wm().clear_bit_by_one(),

                    #[cfg(esp32)]
                    Event::TxFifoThreshold => w.txfifo_empty().clear_bit_by_one(),
                    #[cfg(not(esp32))]
                    Event::TxFifoThreshold => w.txfifo_wm().clear_bit_by_one(),

                    #[cfg(esp32)]
                    Event::ByteReceived => w.rx_rec_full().clear_bit_by_one(),
                    #[cfg(not(esp32))]
                    Event::ByteReceived => w.rxfifo_wm().clear_bit_by_one(),

                    #[cfg(esp32)]
                    Event::ByteTransmitted => w.tx_send_empty().clear_bit_by_one(),
                    #[cfg(not(esp32))]
                    Event::ByteTransmitted => w.txfifo_wm().clear_bit_by_one(),

                    Event::TransComplete => w.trans_complete().clear_bit_by_one(),
                    Event::AddressMatch | Event::StartDetected => {
                        w.trans_start().clear_bit_by_one()
                    }
                    Event::StopDetected => w.end_detect().clear_bit_by_one(),
                    Event::ArbitrationLost => w.arbitration_lost().clear_bit_by_one(),
                    Event::Timeout => w.time_out().clear_bit_by_one(),

                    #[cfg(not(esp32))]
                    Event::RxFifoOverflow => w.rxfifo_ovf().clear_bit_by_one(),
                    #[cfg(esp32)]
                    Event::RxFifoOverflow => w,

                    #[cfg(not(esp32))]
                    Event::TxFifoUnderflow => w.txfifo_udf().clear_bit_by_one(),
                    #[cfg(esp32)]
                    Event::TxFifoUnderflow => w,
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

/// Trait implemented by I2C peripheral instances
pub trait Instance: Peripheral<P = Self> + 'static {
    /// Get peripheral info and state
    fn parts(&self) -> (&Info, &State);

    /// Degrade to any I2C instance
    fn degrade(self) -> AnyI2cSlave<'static>
    where
        Self: Sized,
    {
        AnyI2cSlave {
            info: self.parts().0,
            state: self.parts().1,
        }
    }
}

/// Macro to implement peripheral instances
macro_rules! impl_instance {
    ($inst:ident, $peri:ident, $irq:ident, $scl:ident, $sda:ident) => {
        impl Instance for crate::peripherals::$inst {
            fn parts(&self) -> (&Info, &State) {
                #[crate::macros::handler]
                fn irq_handler() {
                    let (info, state) = Self::parts_static();
                    async_handler(info, state);
                }

                static STATE: State = State::new();

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

        impl crate::peripherals::$inst {
            fn parts_static() -> (&'static Info, &'static State) {
                static STATE: State = State::new();

                static PERIPHERAL: Info = Info {
                    register_block: crate::peripherals::$inst::ptr(),
                    peripheral: crate::system::Peripheral::$peri,
                    async_handler: Self::irq_handler_static,
                    scl_output: OutputSignal::$scl,
                    scl_input: InputSignal::$scl,
                    sda_output: OutputSignal::$sda,
                    sda_input: InputSignal::$sda,
                };

                (&PERIPHERAL, &STATE)
            }

            #[crate::macros::handler]
            fn irq_handler_static() {
                let (info, state) = Self::parts_static();
                async_handler(info, state);
            }

            fn bind_peri_interrupt(&self, handler: crate::interrupt::IsrCallback) {
                unsafe {
                    crate::interrupt::bind_interrupt(crate::peripherals::Interrupt::$irq, handler);
                }
            }

            fn disable_peri_interrupt(&self) {
                crate::interrupt::disable(
                    crate::interrupt::CpuInterrupt::Interrupt0LevelPriority1,
                    crate::peripherals::Interrupt::$irq,
                );
            }

            fn enable_peri_interrupt(&self, priority: Priority) {
                crate::interrupt::enable(crate::peripherals::Interrupt::$irq, priority).unwrap();
            }

            fn set_interrupt_handler(&self, handler: InterruptHandler) {
                self.disable_peri_interrupt();

                let (info, _) = self.parts();
                info.enable_listen(enumset::EnumSet::all(), false);
                info.clear_interrupts(enumset::EnumSet::all());

                self.bind_peri_interrupt(handler.handler());
                self.enable_peri_interrupt(handler.priority());
            }

            fn set_interrupt_priority(&self, priority: Priority) {
                self.enable_peri_interrupt(priority);
            }
        }
    };
}

// Implement for I2C0
#[cfg(i2c_slave_i2c0)]
impl_instance!(I2C0, I2cExt0, I2C_EXT0, I2CEXT0_SCL, I2CEXT0_SDA);

// Implement for I2C1
#[cfg(i2c_slave_i2c1)]
impl_instance!(I2C1, I2cExt1, I2C_EXT1, I2CEXT1_SCL, I2CEXT1_SDA);

/// Type-erased I2C slave instance
pub struct AnyI2cSlave<'d> {
    pub(crate) info: &'d Info,
    pub(crate) state: &'d State,
}

impl<'d> AnyI2cSlave<'d> {
    pub(crate) fn info(&self) -> &'d Info {
        self.info
    }

    pub(crate) fn state(&self) -> &'d State {
        self.state
    }

    pub(crate) fn set_interrupt_handler(&self, handler: InterruptHandler) {
        // Implementations should be added per-chip
        // For now, this is a placeholder
        _ = handler;
    }

    pub(crate) fn set_interrupt_priority(&self, priority: Priority) {
        // Implementations should be added per-chip
        _ = priority;
    }
}

impl core::fmt::Debug for AnyI2cSlave<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnyI2cSlave").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AnyI2cSlave<'_> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "AnyI2cSlave")
    }
}
