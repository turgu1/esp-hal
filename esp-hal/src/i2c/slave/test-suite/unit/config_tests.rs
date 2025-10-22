//! Configuration tests
//!
//! Tests for the Config struct and builder pattern.
//!
//! Corresponds to TESTING.md: Configuration Tests

#[cfg(test)]
mod tests {
    use crate::i2c::slave::{Config, ConfigError, I2cAddress};

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.address, I2cAddress::SevenBit(0x55));
        assert_eq!(config.clock_stretch_enable, true);
        assert_eq!(config.sda_filter_enable, true);
        assert_eq!(config.sda_filter_threshold, 7);
        assert_eq!(config.scl_filter_enable, true);
        assert_eq!(config.scl_filter_threshold, 7);
    }

    #[test]
    fn test_with_address() {
        let config = Config::default().with_address(I2cAddress::SevenBit(0x42));

        assert_eq!(config.address, I2cAddress::SevenBit(0x42));
    }

    #[test]
    fn test_with_address_from_u8() {
        let config = Config::default().with_address(0x42.into());

        assert_eq!(config.address, I2cAddress::SevenBit(0x42));
    }

    #[test]
    fn test_valid_7bit_addresses() {
        // Test boundary values
        let addresses = vec![0x00, 0x01, 0x55, 0x7E, 0x7F];

        for addr in addresses {
            let address = I2cAddress::SevenBit(addr);
            assert!(
                address.validate().is_ok(),
                "Address 0x{:02X} should be valid",
                addr
            );
        }
    }

    #[test]
    fn test_invalid_7bit_addresses() {
        // Test out-of-range values
        let addresses = vec![0x80, 0xFF, 0xA5];

        for addr in addresses {
            let address = I2cAddress::SevenBit(addr);
            assert!(
                address.validate().is_err(),
                "Address 0x{:02X} should be invalid",
                addr
            );
        }
    }

    #[test]
    fn test_valid_10bit_addresses() {
        // Test boundary and typical values for 10-bit addresses
        let addresses = vec![0x000, 0x001, 0x080, 0x1FF, 0x2F3, 0x3FE, 0x3FF];

        for addr in addresses {
            let address = I2cAddress::TenBit(addr);
            assert!(
                address.validate().is_ok(),
                "Address 0x{:03X} should be valid",
                addr
            );
        }
    }

    #[test]
    fn test_invalid_10bit_addresses() {
        // Test out-of-range values for 10-bit addresses (> 0x3FF)
        let addresses = vec![0x400, 0x500, 0x7FF, 0xFFF, 0xFFFF];

        for addr in addresses {
            let address = I2cAddress::TenBit(addr);
            assert!(
                address.validate().is_err(),
                "Address 0x{:03X} should be invalid",
                addr
            );
        }
    }

    #[test]
    fn test_with_10bit_address() {
        let config = Config::default().with_address(I2cAddress::TenBit(0x2F3));

        assert_eq!(config.address, I2cAddress::TenBit(0x2F3));
    }

    #[test]
    fn test_with_10bit_address_from_u16() {
        // Test conversion from u16 to 10-bit address
        let config = Config::default().with_address(0x2F3u16.into());

        assert_eq!(config.address, I2cAddress::TenBit(0x2F3));
    }

    #[test]
    fn test_10bit_address_boundary_values() {
        // Test boundary values
        let addr_min = I2cAddress::TenBit(0x000);
        assert!(addr_min.validate().is_ok());

        let addr_max = I2cAddress::TenBit(0x3FF);
        assert!(addr_max.validate().is_ok());

        // Test just beyond boundary
        let addr_beyond = I2cAddress::TenBit(0x400);
        assert!(addr_beyond.validate().is_err());
    }

    #[test]
    fn test_10bit_address_from_u16_conversion() {
        // Values in 10-bit range should create TenBit addresses
        let addr_80: I2cAddress = 0x80u16.into();
        assert_eq!(addr_80, I2cAddress::TenBit(0x80));

        let addr_3ff: I2cAddress = 0x3FFu16.into();
        assert_eq!(addr_3ff, I2cAddress::TenBit(0x3FF));
    }

    #[test]
    fn test_10bit_address_from_u16_wrapping() {
        // Values beyond 10-bit range should wrap to 10-bit
        let addr: I2cAddress = 0x400u16.into();
        assert_eq!(addr, I2cAddress::TenBit(0x000));

        let addr: I2cAddress = 0x7FFu16.into();
        assert_eq!(addr, I2cAddress::TenBit(0x3FF));

        let addr: I2cAddress = 0xFFFFu16.into();
        assert_eq!(addr, I2cAddress::TenBit(0x3FF));
    }

    #[test]
    fn test_address_from_u16_seven_bit_range() {
        // Values in 7-bit range (0x00-0x7F) should create SevenBit addresses
        let addr: I2cAddress = 0x42u16.into();
        assert_eq!(addr, I2cAddress::SevenBit(0x42));

        let addr: I2cAddress = 0x7Fu16.into();
        assert_eq!(addr, I2cAddress::SevenBit(0x7F));
    }

    #[test]
    fn test_10bit_address_debug() {
        let addr = I2cAddress::TenBit(0x2F3);
        let debug_str = format!("{:?}", addr);

        assert!(debug_str.contains("TenBit"));
        assert!(
            debug_str.contains("2f3") || debug_str.contains("2F3") || debug_str.contains("755")
        );
    }

    #[test]
    fn test_10bit_address_clone() {
        let addr1 = I2cAddress::TenBit(0x2F3);
        let addr2 = addr1.clone();

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_10bit_address_copy() {
        let addr1 = I2cAddress::TenBit(0x2F3);
        let addr2 = addr1; // Uses Copy trait

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_10bit_vs_7bit_address_inequality() {
        let addr_7bit = I2cAddress::SevenBit(0x55);
        let addr_10bit = I2cAddress::TenBit(0x55);

        assert_ne!(addr_7bit, addr_10bit);
    }

    #[test]
    fn test_with_clock_stretch_enable() {
        let config = Config::default().with_clock_stretch_enable(false);

        assert_eq!(config.clock_stretch_enable, false);
    }

    #[test]
    fn test_with_sda_filter_enable() {
        let config = Config::default().with_sda_filter_enable(false);

        assert_eq!(config.sda_filter_enable, false);
    }

    #[test]
    fn test_with_sda_filter_threshold() {
        let config = Config::default().with_sda_filter_threshold(15);

        assert_eq!(config.sda_filter_threshold, 15);
    }

    #[test]
    fn test_with_scl_filter_enable() {
        let config = Config::default().with_scl_filter_enable(false);

        assert_eq!(config.scl_filter_enable, false);
    }

    #[test]
    fn test_with_scl_filter_threshold() {
        let config = Config::default().with_scl_filter_threshold(10);

        assert_eq!(config.scl_filter_threshold, 10);
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let config = Config::default()
            .with_address(0x42.into())
            .with_clock_stretch_enable(false)
            .with_sda_filter_enable(true)
            .with_sda_filter_threshold(5)
            .with_scl_filter_enable(true)
            .with_scl_filter_threshold(3);

        assert_eq!(config.address, I2cAddress::SevenBit(0x42));
        assert_eq!(config.clock_stretch_enable, false);
        assert_eq!(config.sda_filter_enable, true);
        assert_eq!(config.sda_filter_threshold, 5);
        assert_eq!(config.scl_filter_enable, true);
        assert_eq!(config.scl_filter_threshold, 3);
    }

    #[test]
    fn test_config_clone() {
        let config1 = Config::default().with_address(0x42.into());
        let config2 = config1.clone();

        assert_eq!(config1.address, config2.address);
        assert_eq!(config1.clock_stretch_enable, config2.clock_stretch_enable);
    }

    #[test]
    fn test_config_copy() {
        let config1 = Config::default();
        let config2 = config1; // Uses Copy trait

        // Both should have same values
        assert_eq!(config1.address, config2.address);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_config_equality() {
        let config1 = Config::default();
        let config2 = Config::default();

        assert_eq!(config1, config2);
    }

    #[test]
    fn test_config_inequality() {
        let config1 = Config::default();
        let config2 = Config::default().with_address(0x42.into());

        assert_ne!(config1, config2);
    }

    #[test]
    fn test_address_from_trait() {
        let addr: I2cAddress = 0x55u8.into();

        assert_eq!(addr, I2cAddress::SevenBit(0x55));
    }

    #[test]
    fn test_address_debug() {
        let addr = I2cAddress::SevenBit(0x55);
        let debug_str = format!("{:?}", addr);

        assert!(debug_str.contains("SevenBit"));
        assert!(debug_str.contains("55") || debug_str.contains("0x55") || debug_str.contains("85"));
    }

    #[test]
    fn test_address_clone() {
        let addr1 = I2cAddress::SevenBit(0x55);
        let addr2 = addr1.clone();

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_address_copy() {
        let addr1 = I2cAddress::SevenBit(0x55);
        let addr2 = addr1; // Uses Copy trait

        assert_eq!(addr1, addr2);
    }
}
