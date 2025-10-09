//! Master support code for I2C slave testing
//!
//! This module provides I2C master implementations to support HIL testing
//! of the I2C slave driver. Each submodule corresponds to a test category.

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
