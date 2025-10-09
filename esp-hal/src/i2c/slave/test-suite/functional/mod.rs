//! Functional tests (Hardware-in-Loop required)
//!
//! These tests require actual hardware setup with I2C master and slave.

pub(crate) mod basic_comm;
pub(crate) mod address_tests;
pub(crate) mod fifo_tests;
pub(crate) mod clock_stretch_tests;
pub(crate) mod filter_tests;
pub(crate) mod interrupt_tests;
pub(crate) mod error_condition_tests;
