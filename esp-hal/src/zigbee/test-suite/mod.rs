//! Zigbee Driver Test Suite
//!
//! Comprehensive testing infrastructure for the Zigbee driver including:
//! - Unit tests for individual modules
//! - Integration tests for cross-module functionality
//! - Mock utilities for hardware abstraction
//! - Test helpers and utilities

#[cfg(test)]
pub mod unit_tests;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod mocks;

#[cfg(test)]
pub mod helpers;

// Re-export test utilities
#[cfg(test)]
pub use mocks::*;
#[cfg(test)]
pub use helpers::*;
