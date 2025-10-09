//! Test suite for I2C slave driver
//!
//! Comprehensive tests organized by category.

#[cfg(test)]
pub mod unit;

#[cfg(test)]
pub mod functional;

#[cfg(test)]
pub mod async_tests;

#[cfg(test)]
pub mod performance;

#[cfg(test)]
pub mod reliability;

#[cfg(test)]
pub mod integration;

#[cfg(test)]
pub mod helpers;
