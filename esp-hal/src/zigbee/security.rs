//! Zigbee security implementation
//!
//! Implements AES-128 based security for Zigbee networks

use crate::aes::Aes;
use super::crypto::{Ccm, CryptoError, NonceBuilder};

/// Link key type
pub type LinkKey = [u8; 16];

/// Network key type  
pub type NetworkKey = [u8; 16];

/// Security manager
pub struct SecurityManager {
    /// Network key
    network_key: Option<NetworkKey>,
    
    /// Network key sequence number
    network_key_sequence: u8,
    
    /// Default trust center link key (ZigBeeAlliance09)
    default_tc_link_key: LinkKey,
    
    /// Install codes
    install_codes: heapless::Vec<InstallCodeEntry, 16>,
    
    /// Outgoing frame counter for network key
    outgoing_frame_counter: u32,
}

/// Install code entry
#[derive(Debug, Clone, Copy)]
pub struct InstallCodeEntry {
    /// IEEE address
    pub ieee_address: u64,
    
    /// Install code (with CRC)
    pub install_code: [u8; 18],
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        // Default trust center link key: "ZigBeeAlliance09"
        const DEFAULT_TC_KEY: LinkKey = [
            0x5A, 0x69, 0x67, 0x42, 0x65, 0x65, 0x41, 0x6C,
            0x6C, 0x69, 0x61, 0x6E, 0x63, 0x65, 0x30, 0x39,
        ];
        
        Self {
            network_key: None,
            network_key_sequence: 0,
            default_tc_link_key: DEFAULT_TC_KEY,
            install_codes: heapless::Vec::new(),
            outgoing_frame_counter: 0,
        }
    }
    
    /// Set network key
    pub fn set_network_key(&mut self, key: NetworkKey) {
        self.network_key = Some(key);
        self.network_key_sequence = 0;
    }
    
    /// Get network key
    pub fn network_key(&self) -> Option<&NetworkKey> {
        self.network_key.as_ref()
    }
    
    /// Generate random network key
    pub fn generate_network_key(&mut self) -> NetworkKey {
        // In a real implementation, this would use a TRNG
        let mut key = [0u8; 16];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = (i * 17) as u8; // Placeholder
        }
        self.network_key = Some(key);
        key
    }
    
    /// Rotate network key
    pub fn rotate_network_key(&mut self) -> NetworkKey {
        let new_key = self.generate_network_key();
        self.network_key_sequence = self.network_key_sequence.wrapping_add(1);
        new_key
    }
    
    /// Get network key sequence number
    pub fn network_key_sequence(&self) -> u8 {
        self.network_key_sequence
    }
    
    /// Get default trust center link key
    pub fn default_tc_link_key(&self) -> &LinkKey {
        &self.default_tc_link_key
    }
    
    /// Derive link key from install code
    pub fn derive_link_key_from_install_code(&self, install_code: &[u8; 18]) -> LinkKey {
        // In a real implementation, this would use AES-MMO hash
        // For now, return a placeholder
        let mut key = [0u8; 16];
        key.copy_from_slice(&install_code[..16]);
        key
    }
    
    /// Add install code
    pub fn add_install_code(&mut self, ieee_address: u64, install_code: [u8; 18]) -> Result<(), ()> {
        self.install_codes
            .push(InstallCodeEntry {
                ieee_address,
                install_code,
            })
            .map_err(|_| ())
    }
    
    /// Get install code for device
    pub fn get_install_code(&self, ieee_address: u64) -> Option<&[u8; 18]> {
        self.install_codes
            .iter()
            .find(|e| e.ieee_address == ieee_address)
            .map(|e| &e.install_code)
    }
    
    /// Get and increment the outgoing frame counter
    pub fn next_frame_counter(&mut self) -> u32 {
        let counter = self.outgoing_frame_counter;
        self.outgoing_frame_counter = self.outgoing_frame_counter.wrapping_add(1);
        counter
    }
    
    /// Get current outgoing frame counter value
    pub fn frame_counter(&self) -> u32 {
        self.outgoing_frame_counter
    }
    
    /// Encrypt frame with security header
    ///
    /// # Arguments
    /// * `aes` - AES hardware instance
    /// * `source_addr` - Source IEEE address
    /// * `security_header` - Security header to use
    /// * `header` - Frame header (authenticated but not encrypted)
    /// * `payload` - Frame payload (encrypted and authenticated)
    /// * `mic` - Output buffer for MIC
    ///
    /// # Returns
    /// Frame counter used for this encryption
    pub fn encrypt_frame(
        &mut self,
        aes: &mut Aes<'static>,
        source_addr: u64,
        security_header: &SecurityHeader,
        header: &[u8],
        payload: &mut [u8],
        mic: &mut [u8],
    ) -> Result<u32, SecurityError> {
        // Get the key to use
        let key = match security_header.key_id {
            0 => {
                // Network key
                self.network_key.as_ref().ok_or(SecurityError::NoKey)?
            }
            _ => {
                // Link key - use default TC link key for now
                &self.default_tc_link_key
            }
        };

        // Build nonce
        let security_control = (security_header.security_level as u8) 
            | ((security_header.key_id & 0x03) << 3);
        
        let nonce = NonceBuilder::new(
            source_addr,
            security_header.frame_counter,
            security_control,
        );

        // Create CCM context
        let mut ccm = Ccm::new(aes);

        // Encrypt or authenticate based on security level
        if security_header.security_level.is_encrypted() {
            // Encrypt and authenticate
            ccm.encrypt_and_auth(key, nonce.as_bytes(), header, payload, mic)
                .map_err(|e| match e {
                    CryptoError::AuthenticationFailed => SecurityError::AuthenticationFailed,
                    _ => SecurityError::InvalidKey,
                })?;
        } else {
            // Authentication only
            ccm.auth_only(key, nonce.as_bytes(), header, payload, mic)
                .map_err(|e| match e {
                    CryptoError::AuthenticationFailed => SecurityError::AuthenticationFailed,
                    _ => SecurityError::InvalidKey,
                })?;
        }

        Ok(security_header.frame_counter)
    }

    /// Decrypt frame with security header
    ///
    /// # Arguments
    /// * `aes` - AES hardware instance
    /// * `source_addr` - Source IEEE address
    /// * `security_header` - Security header from frame
    /// * `header` - Frame header (authenticated but not encrypted)
    /// * `payload` - Frame payload (decrypted and authenticated)
    /// * `mic` - Received MIC
    pub fn decrypt_frame(
        &mut self,
        aes: &mut Aes<'static>,
        source_addr: u64,
        security_header: &SecurityHeader,
        header: &[u8],
        payload: &mut [u8],
        mic: &[u8],
    ) -> Result<(), SecurityError> {
        // Get the key to use
        let key = match security_header.key_id {
            0 => {
                // Network key
                self.network_key.as_ref().ok_or(SecurityError::NoKey)?
            }
            _ => {
                // Link key - use default TC link key for now
                &self.default_tc_link_key
            }
        };

        // Build nonce
        let security_control = (security_header.security_level as u8) 
            | ((security_header.key_id & 0x03) << 3);
        
        let nonce = NonceBuilder::new(
            source_addr,
            security_header.frame_counter,
            security_control,
        );

        // Create CCM context
        let mut ccm = Ccm::new(aes);

        // Decrypt or verify based on security level
        if security_header.security_level.is_encrypted() {
            // Decrypt and verify
            ccm.decrypt_and_verify(key, nonce.as_bytes(), header, payload, mic)
                .map_err(|e| match e {
                    CryptoError::AuthenticationFailed => SecurityError::AuthenticationFailed,
                    _ => SecurityError::DecryptionFailed,
                })?;
        } else {
            // Verify authentication only
            ccm.verify_auth(key, nonce.as_bytes(), header, payload, mic)
                .map_err(|e| match e {
                    CryptoError::AuthenticationFailed => SecurityError::AuthenticationFailed,
                    _ => SecurityError::DecryptionFailed,
                })?;
        }

        Ok(())
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Security errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// No key available
    NoKey,
    
    /// Invalid key
    InvalidKey,
    
    /// Authentication failed
    AuthenticationFailed,
    
    /// Decryption failed
    DecryptionFailed,
    
    /// Invalid install code
    InvalidInstallCode,
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoKey => write!(f, "No key available"),
            Self::InvalidKey => write!(f, "Invalid key"),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed"),
            Self::InvalidInstallCode => write!(f, "Invalid install code"),
        }
    }
}

impl core::error::Error for SecurityError {}

/// Security level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No security
    None = 0,
    
    /// MIC-32 (4-byte authentication tag)
    Mic32 = 1,
    
    /// MIC-64 (8-byte authentication tag)
    Mic64 = 2,
    
    /// MIC-128 (16-byte authentication tag)
    Mic128 = 3,
    
    /// Encryption + MIC-32
    EncMic32 = 5,
    
    /// Encryption + MIC-64
    EncMic64 = 6,
    
    /// Encryption + MIC-128
    EncMic128 = 7,
}

impl SecurityLevel {
    /// Check if encryption is enabled
    pub fn is_encrypted(&self) -> bool {
        matches!(self, 
            Self::EncMic32 | Self::EncMic64 | Self::EncMic128
        )
    }
    
    /// Get MIC length in bytes
    pub fn mic_length(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Mic32 | Self::EncMic32 => 4,
            Self::Mic64 | Self::EncMic64 => 8,
            Self::Mic128 | Self::EncMic128 => 16,
        }
    }
}

/// Key type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Network key
    Network = 1,
    
    /// Trust center link key
    TrustCenterLink = 2,
    
    /// Application link key
    ApplicationLink = 3,
}

/// Security header
#[derive(Debug, Clone, Copy)]
pub struct SecurityHeader {
    /// Security level
    pub security_level: SecurityLevel,
    
    /// Key identifier
    pub key_id: u8,
    
    /// Frame counter
    pub frame_counter: u32,
    
    /// Source address (for key_id mode 3)
    pub source_address: Option<u64>,
    
    /// Key sequence number (for network key)
    pub key_sequence: Option<u8>,
}

impl SecurityHeader {
    /// Create a new security header
    pub fn new(security_level: SecurityLevel, key_id: u8, frame_counter: u32) -> Self {
        Self {
            security_level,
            key_id,
            frame_counter,
            source_address: None,
            key_sequence: None,
        }
    }
    
    /// Encode security header to bytes
    pub fn encode(&self, buffer: &mut [u8]) -> Result<usize, ()> {
        if buffer.len() < 5 {
            return Err(());
        }
        
        // Security control field
        let mut control = (self.security_level as u8) & 0x07;
        control |= (self.key_id & 0x03) << 3;
        buffer[0] = control;
        
        // Frame counter (little-endian)
        buffer[1..5].copy_from_slice(&self.frame_counter.to_le_bytes());
        
        let mut len = 5;
        
        // Optional fields based on key identifier mode
        if self.key_id == 0 {
            // Network key with sequence number
            if let Some(seq) = self.key_sequence {
                buffer[len] = seq;
                len += 1;
            }
        } else if self.key_id == 3 {
            // Application link key with source address
            if let Some(addr) = self.source_address {
                if buffer.len() < len + 8 {
                    return Err(());
                }
                buffer[len..len + 8].copy_from_slice(&addr.to_le_bytes());
                len += 8;
            }
        }
        
        Ok(len)
    }
    
    /// Decode security header from bytes
    pub fn decode(buffer: &[u8]) -> Result<(Self, usize), ()> {
        if buffer.len() < 5 {
            return Err(());
        }
        
        let control = buffer[0];
        let security_level = match control & 0x07 {
            0 => SecurityLevel::None,
            1 => SecurityLevel::Mic32,
            2 => SecurityLevel::Mic64,
            3 => SecurityLevel::Mic128,
            5 => SecurityLevel::EncMic32,
            6 => SecurityLevel::EncMic64,
            7 => SecurityLevel::EncMic128,
            _ => return Err(()),
        };
        
        let key_id = (control >> 3) & 0x03;
        let frame_counter = u32::from_le_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        
        let mut header = Self::new(security_level, key_id, frame_counter);
        let mut len = 5;
        
        // Optional fields
        if key_id == 0 && buffer.len() > len {
            header.key_sequence = Some(buffer[len]);
            len += 1;
        } else if key_id == 3 && buffer.len() >= len + 8 {
            let mut addr_bytes = [0u8; 8];
            addr_bytes.copy_from_slice(&buffer[len..len + 8]);
            header.source_address = Some(u64::from_le_bytes(addr_bytes));
            len += 8;
        }
        
        Ok((header, len))
    }
}
