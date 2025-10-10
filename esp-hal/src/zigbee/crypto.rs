//! Cryptographic primitives for Zigbee
//!
//! Implements AES-128 CCM* (Counter with CBC-MAC) mode for Zigbee frame security
//! as specified in the Zigbee specification.
//!
//! CCM* is a variation of CCM (Counter with CBC-MAC) that allows for
//! authentication-only mode (no encryption).

use crate::aes::{Aes, Key as AesKey};

/// Maximum additional authenticated data (AAD) length
const MAX_AAD_LEN: usize = 128;

/// AES block size in bytes
const BLOCK_SIZE: usize = 16;

/// CCM* nonce size for Zigbee (13 bytes)
const NONCE_SIZE: usize = 13;

/// CCM* encryption/authentication context
pub struct Ccm<'a> {
    aes: &'a mut Aes<'static>,
}

impl<'a> Ccm<'a> {
    /// Create a new CCM* context with the given AES instance
    pub fn new(aes: &'a mut Aes<'static>) -> Self {
        Self { aes }
    }

    /// Encrypt and authenticate data using CCM* mode
    ///
    /// # Arguments
    /// * `key` - 128-bit encryption key
    /// * `nonce` - 13-byte nonce (source addr + frame counter + security level)
    /// * `aad` - Additional authenticated data (header)
    /// * `plaintext` - Data to encrypt (modified in-place)
    /// * `mic` - Output buffer for message integrity code
    ///
    /// # Returns
    /// Ok(()) on success, Err(CryptoError) on failure
    pub fn encrypt_and_auth(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        plaintext: &mut [u8],
        mic: &mut [u8],
    ) -> Result<(), CryptoError> {
        if aad.len() > MAX_AAD_LEN {
            return Err(CryptoError::InvalidLength);
        }

        if mic.len() != 4 && mic.len() != 8 && mic.len() != 16 {
            return Err(CryptoError::InvalidMicLength);
        }

        // Step 1: Calculate authentication tag using CBC-MAC
        let mut auth_tag = [0u8; BLOCK_SIZE];
        self.calculate_cbc_mac(key, nonce, aad, plaintext, mic.len(), &mut auth_tag)?;

        // Step 2: Encrypt plaintext using CTR mode
        if !plaintext.is_empty() {
            self.ctr_mode(key, nonce, 1, plaintext)?;
        }

        // Step 3: Encrypt authentication tag for MIC
        let mut ctr0 = [0u8; BLOCK_SIZE];
        self.format_counter_block(nonce, 0, &mut ctr0);
        self.aes.encrypt(&mut ctr0, (*key).into());

        // XOR to get encrypted MIC
        for i in 0..mic.len() {
            mic[i] = auth_tag[i] ^ ctr0[i];
        }

        Ok(())
    }

    /// Decrypt and verify data using CCM* mode
    ///
    /// # Arguments
    /// * `key` - 128-bit encryption key
    /// * `nonce` - 13-byte nonce
    /// * `aad` - Additional authenticated data (header)
    /// * `ciphertext` - Encrypted data (modified in-place to plaintext)
    /// * `mic` - Received message integrity code
    ///
    /// # Returns
    /// Ok(()) if decryption and authentication succeed
    pub fn decrypt_and_verify(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        ciphertext: &mut [u8],
        mic: &[u8],
    ) -> Result<(), CryptoError> {
        if aad.len() > MAX_AAD_LEN {
            return Err(CryptoError::InvalidLength);
        }

        if mic.len() != 4 && mic.len() != 8 && mic.len() != 16 {
            return Err(CryptoError::InvalidMicLength);
        }

        // Step 1: Decrypt authentication tag
        let mut ctr0 = [0u8; BLOCK_SIZE];
        self.format_counter_block(nonce, 0, &mut ctr0);
        self.aes.encrypt(&mut ctr0, (*key).into());

        let mut expected_auth_tag = [0u8; BLOCK_SIZE];
        for i in 0..mic.len() {
            expected_auth_tag[i] = mic[i] ^ ctr0[i];
        }

        // Step 2: Decrypt ciphertext using CTR mode
        if !ciphertext.is_empty() {
            self.ctr_mode(key, nonce, 1, ciphertext)?;
        }

        // Step 3: Calculate authentication tag from plaintext
        let mut calculated_auth_tag = [0u8; BLOCK_SIZE];
        self.calculate_cbc_mac(key, nonce, aad, ciphertext, mic.len(), &mut calculated_auth_tag)?;

        // Step 4: Constant-time comparison
        let mut match_result = 0u8;
        for i in 0..mic.len() {
            match_result |= expected_auth_tag[i] ^ calculated_auth_tag[i];
        }

        if match_result != 0 {
            // Zero out plaintext on authentication failure
            ciphertext.fill(0);
            return Err(CryptoError::AuthenticationFailed);
        }

        Ok(())
    }

    /// Authentication-only mode (no encryption)
    ///
    /// Used for frames that require authentication but not confidentiality
    pub fn auth_only(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        data: &[u8],
        mic: &mut [u8],
    ) -> Result<(), CryptoError> {
        if aad.len() > MAX_AAD_LEN {
            return Err(CryptoError::InvalidLength);
        }

        if mic.len() != 4 && mic.len() != 8 && mic.len() != 16 {
            return Err(CryptoError::InvalidMicLength);
        }

        // Calculate authentication tag
        let mut auth_tag = [0u8; BLOCK_SIZE];
        self.calculate_cbc_mac(key, nonce, aad, data, mic.len(), &mut auth_tag)?;

        // Encrypt authentication tag
        let mut ctr0 = [0u8; BLOCK_SIZE];
        self.format_counter_block(nonce, 0, &mut ctr0);
        self.aes.encrypt(&mut ctr0, (*key).into());

        // XOR to get encrypted MIC
        for i in 0..mic.len() {
            mic[i] = auth_tag[i] ^ ctr0[i];
        }

        Ok(())
    }

    /// Verify authentication-only mode
    pub fn verify_auth(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        data: &[u8],
        mic: &[u8],
    ) -> Result<(), CryptoError> {
        if aad.len() > MAX_AAD_LEN {
            return Err(CryptoError::InvalidLength);
        }

        if mic.len() != 4 && mic.len() != 8 && mic.len() != 16 {
            return Err(CryptoError::InvalidMicLength);
        }

        // Decrypt authentication tag
        let mut ctr0 = [0u8; BLOCK_SIZE];
        self.format_counter_block(nonce, 0, &mut ctr0);
        self.aes.encrypt(&mut ctr0, (*key).into());

        let mut expected_auth_tag = [0u8; BLOCK_SIZE];
        for i in 0..mic.len() {
            expected_auth_tag[i] = mic[i] ^ ctr0[i];
        }

        // Calculate authentication tag
        let mut calculated_auth_tag = [0u8; BLOCK_SIZE];
        self.calculate_cbc_mac(key, nonce, aad, data, mic.len(), &mut calculated_auth_tag)?;

        // Constant-time comparison
        let mut match_result = 0u8;
        for i in 0..mic.len() {
            match_result |= expected_auth_tag[i] ^ calculated_auth_tag[i];
        }

        if match_result != 0 {
            return Err(CryptoError::AuthenticationFailed);
        }

        Ok(())
    }

    /// Calculate CBC-MAC for authentication
    fn calculate_cbc_mac(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        aad: &[u8],
        data: &[u8],
        mic_len: usize,
        output: &mut [u8; BLOCK_SIZE],
    ) -> Result<(), CryptoError> {
        // Build first block (B_0)
        let mut b = [0u8; BLOCK_SIZE];
        
        // Flags byte
        let mut flags = 0u8;
        if !aad.is_empty() {
            flags |= 0x40; // Adata bit
        }
        let m = ((mic_len - 2) / 2) as u8; // M value (0-3)
        flags |= (m & 0x07) << 3;
        
        // L value (length of message length field)
        // For Zigbee, L = 2 (15 - nonce_len)
        let l = (15 - NONCE_SIZE) as u8;
        flags |= (l - 1) & 0x07;
        
        b[0] = flags;
        b[1..14].copy_from_slice(nonce);
        
        // Message length (2 bytes for L=2)
        let msg_len = data.len() as u16;
        b[14] = (msg_len >> 8) as u8;
        b[15] = msg_len as u8;

        // Encrypt B_0
        self.aes.encrypt(&mut b, (*key).into());

        // Process AAD if present
        if !aad.is_empty() {
            self.process_aad(&mut b, key, aad)?;
        }

        // Process message data
        if !data.is_empty() {
            self.process_data(&mut b, key, data)?;
        }

        // Copy result
        output.copy_from_slice(&b);
        Ok(())
    }

    /// Process additional authenticated data in CBC-MAC
    fn process_aad(
        &mut self,
        state: &mut [u8; BLOCK_SIZE],
        key: &[u8; 16],
        aad: &[u8],
    ) -> Result<(), CryptoError> {
        let aad_len = aad.len();
        
        // Encode AAD length
        let mut block = [0u8; BLOCK_SIZE];
        let mut offset = 0;

        if aad_len < 0xFF00 {
            // Short form: 2 bytes
            block[0] = (aad_len >> 8) as u8;
            block[1] = aad_len as u8;
            offset = 2;
        } else {
            // Long form: 6 bytes (not expected in Zigbee)
            return Err(CryptoError::InvalidLength);
        }

        // Process AAD in blocks
        let mut aad_processed = 0;
        let remaining_in_first_block = BLOCK_SIZE - offset;
        let to_copy = core::cmp::min(remaining_in_first_block, aad_len);
        
        block[offset..offset + to_copy].copy_from_slice(&aad[..to_copy]);
        aad_processed += to_copy;
        
        // XOR with state and encrypt
        for i in 0..BLOCK_SIZE {
            block[i] ^= state[i];
        }
        self.aes.encrypt(&mut block, (*key).into());
        state.copy_from_slice(&block);

        // Process remaining AAD blocks
        while aad_processed < aad_len {
            let remaining = aad_len - aad_processed;
            let to_process = core::cmp::min(BLOCK_SIZE, remaining);
            
            block.fill(0);
            block[..to_process].copy_from_slice(&aad[aad_processed..aad_processed + to_process]);
            
            for i in 0..BLOCK_SIZE {
                block[i] ^= state[i];
            }
            self.aes.encrypt(&mut block, (*key).into());
            state.copy_from_slice(&block);
            
            aad_processed += to_process;
        }

        Ok(())
    }

    /// Process message data in CBC-MAC
    fn process_data(
        &mut self,
        state: &mut [u8; BLOCK_SIZE],
        key: &[u8; 16],
        data: &[u8],
    ) -> Result<(), CryptoError> {
        let mut block = [0u8; BLOCK_SIZE];
        let mut offset = 0;

        while offset < data.len() {
            let remaining = data.len() - offset;
            let to_process = core::cmp::min(BLOCK_SIZE, remaining);
            
            block.fill(0);
            block[..to_process].copy_from_slice(&data[offset..offset + to_process]);
            
            // XOR with state and encrypt
            for i in 0..BLOCK_SIZE {
                block[i] ^= state[i];
            }
            self.aes.encrypt(&mut block, (*key).into());
            state.copy_from_slice(&block);
            
            offset += to_process;
        }

        Ok(())
    }

    /// Apply CTR mode encryption/decryption
    fn ctr_mode(
        &mut self,
        key: &[u8; 16],
        nonce: &[u8; NONCE_SIZE],
        start_counter: u16,
        data: &mut [u8],
    ) -> Result<(), CryptoError> {
        let mut counter = start_counter;
        let mut offset = 0;

        while offset < data.len() {
            // Format counter block
            let mut ctr_block = [0u8; BLOCK_SIZE];
            self.format_counter_block(nonce, counter, &mut ctr_block);

            // Encrypt counter
            self.aes.encrypt(&mut ctr_block, (*key).into());

            // XOR with data
            let remaining = data.len() - offset;
            let to_process = core::cmp::min(BLOCK_SIZE, remaining);
            
            for i in 0..to_process {
                data[offset + i] ^= ctr_block[i];
            }

            offset += to_process;
            counter += 1;
        }

        Ok(())
    }

    /// Format a counter block for CTR mode
    fn format_counter_block(&self, nonce: &[u8; NONCE_SIZE], counter: u16, output: &mut [u8; BLOCK_SIZE]) {
        // Flags byte: L=2 (counter length)
        output[0] = 1; // L-1 = 1
        
        // Nonce (13 bytes)
        output[1..14].copy_from_slice(nonce);
        
        // Counter (2 bytes, big-endian)
        output[14] = (counter >> 8) as u8;
        output[15] = counter as u8;
    }
}

/// Nonce builder for Zigbee CCM*
pub struct NonceBuilder {
    nonce: [u8; NONCE_SIZE],
}

impl NonceBuilder {
    /// Create a new nonce for CCM*
    ///
    /// Zigbee nonce format (13 bytes):
    /// - Source address (8 bytes, little-endian)
    /// - Frame counter (4 bytes, little-endian)
    /// - Security control (1 byte)
    pub fn new(source_addr: u64, frame_counter: u32, security_control: u8) -> Self {
        let mut nonce = [0u8; NONCE_SIZE];
        
        // Source address (8 bytes)
        nonce[0..8].copy_from_slice(&source_addr.to_le_bytes());
        
        // Frame counter (4 bytes)
        nonce[8..12].copy_from_slice(&frame_counter.to_le_bytes());
        
        // Security control
        nonce[12] = security_control;
        
        Self { nonce }
    }

    /// Get the nonce as a byte array
    pub fn as_bytes(&self) -> &[u8; NONCE_SIZE] {
        &self.nonce
    }

    /// Consume and return the nonce
    pub fn build(self) -> [u8; NONCE_SIZE] {
        self.nonce
    }
}

/// Crypto errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    /// Invalid data length
    InvalidLength,
    
    /// Invalid MIC length (must be 4, 8, or 16)
    InvalidMicLength,
    
    /// Authentication failed
    AuthenticationFailed,
    
    /// Encryption failed
    EncryptionFailed,
}

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "Invalid data length"),
            Self::InvalidMicLength => write!(f, "Invalid MIC length"),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::EncryptionFailed => write!(f, "Encryption failed"),
        }
    }
}

impl core::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vector from Zigbee specification Annex C.6.1
    #[test]
    fn test_ccm_star_encrypt() {
        // Test vectors would go here
        // Note: Requires actual AES hardware, so these are integration tests
    }

    #[test]
    fn test_nonce_builder() {
        let source_addr = 0x1122334455667788u64;
        let frame_counter = 0xAABBCCDDu32;
        let security_control = 0x05u8;

        let nonce = NonceBuilder::new(source_addr, frame_counter, security_control);
        let bytes = nonce.as_bytes();

        // Verify source address (little-endian)
        assert_eq!(&bytes[0..8], &[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]);
        
        // Verify frame counter (little-endian)
        assert_eq!(&bytes[8..12], &[0xDD, 0xCC, 0xBB, 0xAA]);
        
        // Verify security control
        assert_eq!(bytes[12], 0x05);
    }
}
