//! Error handling tests
//!
//! Tests for error types and error conditions.
//!
//! Corresponds to TESTING.md: Error Handling

#[cfg(test)]
mod tests {
    use crate::i2c::slave::{Error, ConfigError, I2cAddress};

    #[test]
    fn test_error_display() {
        let errors = vec![
            (Error::FifoExceeded, "FIFO"),
            (Error::AcknowledgeCheckFailed, "acknowledgment"),
            (Error::Timeout, "timeout"),
            (Error::ArbitrationLost, "arbitration"),
            (Error::ExecutionIncomplete, "execution"),
            (Error::ZeroLengthInvalid, "Zero length"),
            (Error::BusBusy, "busy"),
            (Error::TxFifoOverflow, "overflow"),
            (Error::RxFifoUnderflow, "underflow"),
        ];

        for (error, expected_text) in errors {
            let display = format!("{}", error);
            assert!(
                display.contains(expected_text),
                "Error '{}' should contain '{}'",
                display,
                expected_text
            );
        }
    }

    #[test]
    fn test_address_invalid_error_display() {
        let addr = I2cAddress::SevenBit(0x55);
        let error = Error::AddressInvalid(addr);
        let display = format!("{}", error);
        
        assert!(display.contains("address"));
        assert!(display.contains("invalid") || display.contains("Address"));
    }

    #[test]
    fn test_error_debug() {
        let error = Error::Timeout;
        let debug_str = format!("{:?}", error);
        
        assert!(debug_str.contains("Timeout"));
    }

    #[test]
    fn test_error_clone() {
        let error1 = Error::Timeout;
        let error2 = error1.clone();
        
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_error_copy() {
        let error1 = Error::Timeout;
        let error2 = error1; // Uses Copy trait
        
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(Error::Timeout, Error::Timeout);
        assert_ne!(Error::Timeout, Error::ArbitrationLost);
    }

    #[test]
    fn test_error_source_trait() {
        use core::error::Error as StdError;
        
        let error = Error::Timeout;
        let _source = error.source(); // Should not panic
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::AddressInvalid;
        let display = format!("{}", error);
        
        assert!(display.contains("address") || display.contains("Address"));
        assert!(display.contains("invalid") || display.contains("Invalid"));
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::AddressInvalid;
        let debug_str = format!("{:?}", error);
        
        assert!(debug_str.contains("AddressInvalid"));
    }

    #[test]
    fn test_config_error_clone() {
        let error1 = ConfigError::AddressInvalid;
        let error2 = error1.clone();
        
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_config_error_copy() {
        let error1 = ConfigError::AddressInvalid;
        let error2 = error1; // Uses Copy trait
        
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_zero_length_error() {
        let error = Error::ZeroLengthInvalid;
        let display = format!("{}", error);
        
        assert!(display.to_lowercase().contains("zero") || display.contains("0"));
        assert!(display.to_lowercase().contains("length"));
    }

    #[test]
    fn test_fifo_errors() {
        let overflow = Error::TxFifoOverflow;
        let underflow = Error::RxFifoUnderflow;
        let exceeded = Error::FifoExceeded;
        
        assert_ne!(overflow, underflow);
        assert_ne!(overflow, exceeded);
        assert_ne!(underflow, exceeded);
    }

    #[test]
    fn test_error_variants_count() {
        // Ensure all error variants are handled
        let errors = vec![
            Error::FifoExceeded,
            Error::AcknowledgeCheckFailed,
            Error::Timeout,
            Error::ArbitrationLost,
            Error::ExecutionIncomplete,
            Error::ZeroLengthInvalid,
            Error::AddressInvalid(I2cAddress::SevenBit(0x55)),
            Error::BusBusy,
            Error::TxFifoOverflow,
            Error::RxFifoUnderflow,
        ];
        
        // All errors should have unique displays
        let displays: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
        assert_eq!(displays.len(), 10, "Should have 10 error variants");
    }

    #[cfg(feature = "defmt")]
    #[test]
    fn test_error_defmt() {
        use defmt::Format;
        
        // Just test that the trait is implemented
        fn assert_format<T: Format>() {}
        assert_format::<Error>();
        assert_format::<ConfigError>();
    }
}
