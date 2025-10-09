//! Address matching tests
//!
//! Tests for slave address configuration and matching.
//!
//! Corresponds to TESTING.md: Test 7-9 (Address Testing)

#[cfg(all(test, feature = "hil-test"))]
mod hil_tests {
    use crate::i2c::slave::{Config, I2c, I2cAddress};

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_correct_address_match() {
        // Test 7: Correct Address Match
        // Setup: Master addresses slave with configured address
        // Expected: Slave responds, communication successful
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        let config = Config::default().with_address(0x55.into());
        let mut slave = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Master addresses 0x55
        let mut buffer = [0u8; 4];
        let result = slave.read(&mut buffer);
        
        assert!(result.is_ok());
        */
    }

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_wrong_address_ignored() {
        // Test 8: Wrong Address
        // Setup: Master addresses different address
        // Expected: Slave ignores communication
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        let config = Config::default().with_address(0x55.into());
        let mut slave = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Master addresses 0x66 (different from 0x55)
        // Slave should not respond
        // This would timeout or return no data
        */
    }

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_address_reconfiguration() {
        // Test 9: Address Configuration Change
        // Setup: Change slave address via apply_config()
        // Expected: New address takes effect, old ignored
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        let mut slave = I2c::new(peripherals.I2C0, Config::default())
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Initially at 0x55
        // Master can communicate at 0x55
        
        // Change to 0x66
        let new_config = Config::default().with_address(0x66.into());
        slave.apply_config(&new_config).unwrap();
        
        // Now master must use 0x66
        // Attempts at 0x55 should be ignored
        */
    }

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_multiple_address_changes() {
        // Test changing address multiple times
        // Expected: Each change takes effect immediately
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        let mut slave = I2c::new(peripherals.I2C0, Config::default())
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        let addresses = [0x42, 0x43, 0x44, 0x45];
        
        for addr in addresses {
            let config = Config::default().with_address(I2cAddress::SevenBit(addr));
            slave.apply_config(&config).unwrap();
            
            // Verify communication works at new address
        }
        */
    }

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_general_call_address() {
        // Test response to general call address (0x00)
        // Expected: Configurable behavior
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        let config = Config::default().with_address(0x00.into());
        let mut slave = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO1)
            .with_scl(peripherals.GPIO2);
        
        // Master sends general call
        // Slave should respond (or not, depending on config)
        */
    }

    #[test]
    #[ignore = "Requires HIL setup"]
    fn test_reserved_addresses() {
        // Test using reserved I2C addresses
        // Expected: Hardware may or may not support
        
        /*
        let peripherals = unsafe { Peripherals::steal() };
        
        // Try reserved addresses
        let reserved = [0x01, 0x02, 0x03, 0x78, 0x79, 0x7F];
        
        for addr in reserved {
            let config = Config::default().with_address(I2cAddress::SevenBit(addr));
            let slave = I2c::new(peripherals.I2C0, config);
            
            // May succeed or fail depending on hardware
        }
        */
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::i2c::slave::{Config, I2cAddress};

    #[test]
    fn test_address_format() {
        let addr = I2cAddress::SevenBit(0x55);
        
        // Verify it's stored correctly
        match addr {
            I2cAddress::SevenBit(a) => assert_eq!(a, 0x55),
        }
    }

    #[test]
    fn test_address_equality() {
        let addr1 = I2cAddress::SevenBit(0x55);
        let addr2 = I2cAddress::SevenBit(0x55);
        let addr3 = I2cAddress::SevenBit(0x56);
        
        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_config_address_comparison() {
        let config1 = Config::default().with_address(0x55.into());
        let config2 = Config::default().with_address(0x55.into());
        let config3 = Config::default().with_address(0x56.into());
        
        assert_eq!(config1.address, config2.address);
        assert_ne!(config1.address, config3.address);
    }

    #[test]
    fn test_all_valid_addresses() {
        // Test that all valid 7-bit addresses work
        for addr in 0u8..=0x7F {
            let address = I2cAddress::SevenBit(addr);
            assert!(address.validate().is_ok());
        }
    }

    #[test]
    fn test_all_invalid_addresses() {
        // Test that invalid addresses are rejected
        for addr in 0x80u8..=0xFF {
            let address = I2cAddress::SevenBit(addr);
            assert!(address.validate().is_err());
        }
    }
}
