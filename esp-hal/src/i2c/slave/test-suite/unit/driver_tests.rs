//! Driver instantiation tests
//!
//! Tests for creating and configuring the I2C slave driver.
//!
//! Corresponds to TESTING.md: Driver Instantiation

#[cfg(test)]
mod tests {
    use crate::i2c::slave::{Config, I2cAddress, ConfigError};

    // Note: Most driver instantiation tests require actual hardware
    // These tests focus on configuration validation

    #[test]
    fn test_valid_config_accepted() {
        let config = Config::default().with_address(0x55.into());
        
        // Validate address
        assert!(config.address.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_rejected() {
        let config = Config::default().with_address(I2cAddress::SevenBit(0xFF));
        
        // Should fail validation
        assert!(config.address.validate().is_err());
    }

    #[test]
    fn test_config_validation_boundaries() {
        // Test valid boundary
        let valid_config = Config::default().with_address(I2cAddress::SevenBit(0x7F));
        assert!(valid_config.address.validate().is_ok());
        
        // Test invalid boundary
        let invalid_config = Config::default().with_address(I2cAddress::SevenBit(0x80));
        assert!(invalid_config.address.validate().is_err());
    }

    #[test]
    fn test_multiple_configs() {
        let config1 = Config::default().with_address(0x42.into());
        let config2 = Config::default().with_address(0x43.into());
        
        assert_ne!(config1.address, config2.address);
    }

    #[test]
    fn test_config_immutability() {
        let config1 = Config::default();
        let config2 = config1.with_address(0x42.into());
        
        // Original should be unchanged (moved, but value comparison)
        assert_eq!(config1.address, I2cAddress::SevenBit(0x55));
        assert_eq!(config2.address, I2cAddress::SevenBit(0x42));
    }

    #[test]
    fn test_filter_threshold_range() {
        // Filter thresholds should accept any u8 value
        let configs = vec![0, 1, 7, 15, 31, 63, 127, 255];
        
        for threshold in configs {
            let config = Config::default()
                .with_sda_filter_threshold(threshold)
                .with_scl_filter_threshold(threshold);
            
            assert_eq!(config.sda_filter_threshold, threshold);
            assert_eq!(config.scl_filter_threshold, threshold);
        }
    }

    #[test]
    fn test_clock_stretch_toggle() {
        let enabled = Config::default().with_clock_stretch_enable(true);
        let disabled = Config::default().with_clock_stretch_enable(false);
        
        assert_eq!(enabled.clock_stretch_enable, true);
        assert_eq!(disabled.clock_stretch_enable, false);
    }

    #[test]
    fn test_filter_enable_combinations() {
        let both = Config::default()
            .with_sda_filter_enable(true)
            .with_scl_filter_enable(true);
        
        let sda_only = Config::default()
            .with_sda_filter_enable(true)
            .with_scl_filter_enable(false);
        
        let scl_only = Config::default()
            .with_sda_filter_enable(false)
            .with_scl_filter_enable(true);
        
        let neither = Config::default()
            .with_sda_filter_enable(false)
            .with_scl_filter_enable(false);
        
        assert_eq!(both.sda_filter_enable, true);
        assert_eq!(both.scl_filter_enable, true);
        
        assert_eq!(sda_only.sda_filter_enable, true);
        assert_eq!(sda_only.scl_filter_enable, false);
        
        assert_eq!(scl_only.sda_filter_enable, false);
        assert_eq!(scl_only.scl_filter_enable, true);
        
        assert_eq!(neither.sda_filter_enable, false);
        assert_eq!(neither.scl_filter_enable, false);
    }

    #[test]
    fn test_reserved_addresses() {
        // Some I2C addresses are reserved
        // 0x00: General call
        // 0x01-0x07: Reserved
        // 0x78-0x7F: Reserved for 10-bit addressing and other uses
        
        // Test that we can still configure these (driver accepts them)
        let general_call = Config::default().with_address(I2cAddress::SevenBit(0x00));
        assert_eq!(general_call.address, I2cAddress::SevenBit(0x00));
        
        let reserved = Config::default().with_address(I2cAddress::SevenBit(0x78));
        assert_eq!(reserved.address, I2cAddress::SevenBit(0x78));
    }

    #[test]
    fn test_common_addresses() {
        // Test common I2C addresses used in practice
        let common = vec![
            0x50, // EEPROM
            0x68, // RTC, MPU-6050
            0x76, // BMP280
            0x77, // BME280, BMP180
        ];
        
        for addr in common {
            let config = Config::default().with_address(I2cAddress::SevenBit(addr));
            assert_eq!(config.address, I2cAddress::SevenBit(addr));
        }
    }
}
