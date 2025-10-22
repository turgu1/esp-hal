//! Master support code for I2C slave testing
//!
//! This module provides I2C master implementations to support HIL testing
//! of the I2C slave driver. Each submodule corresponds to a test category.
//!
//! ## Testing write_read() Support
//!
//! The functional and async_support modules now include dedicated master structs
//! for testing write_read() operations (repeated START transactions):
//!
//! - `functional::WriteReadTestMaster` - Blocking write_read tests
//! - `async_support::AsyncWriteReadTestMaster` - Async write_read tests
//!
//! These support the comprehensive write_read() testing added in Tests 6a-6g.
//!
//! See: `I2C_SLAVE_WRITE_READ_SUPPORT.md` for implementation details

#[cfg(test)]
pub mod functional;

#[cfg(test)]
pub mod async_support;

#[cfg(test)]
pub mod performance;

#[cfg(test)]
pub mod reliability;

#[cfg(test)]
pub mod integration;

#[cfg(test)]
pub mod common;
