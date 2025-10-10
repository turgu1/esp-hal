//! Persistent Storage for Zigbee Network Configuration
//!
//! This module provides NVS-like persistent storage for Zigbee network
//! parameters, keys, bindings, and other critical data that needs to
//! survive device reboots.

use core::mem::size_of;
use heapless::Vec;

/// Storage error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    /// Key not found
    NotFound,
    
    /// Storage full
    StorageFull,
    
    /// Invalid data
    InvalidData,
    
    /// Write failed
    WriteFailed,
    
    /// Read failed
    ReadFailed,
    
    /// Erase failed
    EraseFailed,
    
    /// Invalid parameters
    InvalidParameter,
    
    /// CRC mismatch
    CrcMismatch,
}

impl core::fmt::Display for StorageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Key not found"),
            Self::StorageFull => write!(f, "Storage full"),
            Self::InvalidData => write!(f, "Invalid data"),
            Self::WriteFailed => write!(f, "Write failed"),
            Self::ReadFailed => write!(f, "Read failed"),
            Self::EraseFailed => write!(f, "Erase failed"),
            Self::InvalidParameter => write!(f, "Invalid parameter"),
            Self::CrcMismatch => write!(f, "CRC mismatch"),
        }
    }
}

impl core::error::Error for StorageError {}

pub type Result<T> = core::result::Result<T, StorageError>;

/// Storage keys for different data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageKey {
    /// Network PAN ID
    PanId = 0x01,
    
    /// Extended PAN ID
    ExtendedPanId = 0x02,
    
    /// Network channel
    Channel = 0x03,
    
    /// Device short address
    ShortAddress = 0x04,
    
    /// Device IEEE address
    IeeeAddress = 0x05,
    
    /// Network key
    NetworkKey = 0x10,
    
    /// Link key
    LinkKey = 0x11,
    
    /// Trust center address
    TrustCenterAddress = 0x12,
    
    /// Security level
    SecurityLevel = 0x13,
    
    /// Frame counter
    FrameCounter = 0x14,
    
    /// Binding table
    BindingTable = 0x20,
    
    /// Group table
    GroupTable = 0x21,
    
    /// Neighbor table
    NeighborTable = 0x22,
    
    /// Routing table
    RoutingTable = 0x23,
    
    /// Device configuration
    DeviceConfig = 0x30,
    
    /// Coordinator configuration
    CoordinatorConfig = 0x31,
    
    /// Network state
    NetworkState = 0x32,
    
    /// Install code
    InstallCode = 0x40,
    
    /// Custom data start (0x80-0xFF reserved for user)
    CustomDataStart = 0x80,
}

impl StorageKey {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(StorageKey::PanId),
            0x02 => Some(StorageKey::ExtendedPanId),
            0x03 => Some(StorageKey::Channel),
            0x04 => Some(StorageKey::ShortAddress),
            0x05 => Some(StorageKey::IeeeAddress),
            0x10 => Some(StorageKey::NetworkKey),
            0x11 => Some(StorageKey::LinkKey),
            0x12 => Some(StorageKey::TrustCenterAddress),
            0x13 => Some(StorageKey::SecurityLevel),
            0x14 => Some(StorageKey::FrameCounter),
            0x20 => Some(StorageKey::BindingTable),
            0x21 => Some(StorageKey::GroupTable),
            0x22 => Some(StorageKey::NeighborTable),
            0x23 => Some(StorageKey::RoutingTable),
            0x30 => Some(StorageKey::DeviceConfig),
            0x31 => Some(StorageKey::CoordinatorConfig),
            0x32 => Some(StorageKey::NetworkState),
            0x40 => Some(StorageKey::InstallCode),
            0x80 => Some(StorageKey::CustomDataStart),
            _ => None,
        }
    }
}

/// Network configuration that can be persisted
#[derive(Debug, Clone, Copy)]
pub struct PersistedNetworkConfig {
    /// PAN ID
    pub pan_id: u16,
    
    /// Extended PAN ID
    pub extended_pan_id: u64,
    
    /// Channel (11-26)
    pub channel: u8,
    
    /// Device short address
    pub short_address: u16,
    
    /// Device IEEE address
    pub ieee_address: u64,
    
    /// Security enabled
    pub security_enabled: bool,
    
    /// Network key
    pub network_key: [u8; 16],
    
    /// Frame counter
    pub frame_counter: u32,
}

impl PersistedNetworkConfig {
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8, 64> {
        let mut bytes = Vec::new();
        let _ = bytes.extend_from_slice(&self.pan_id.to_le_bytes());
        let _ = bytes.extend_from_slice(&self.extended_pan_id.to_le_bytes());
        let _ = bytes.push(self.channel);
        let _ = bytes.extend_from_slice(&self.short_address.to_le_bytes());
        let _ = bytes.extend_from_slice(&self.ieee_address.to_le_bytes());
        let _ = bytes.push(if self.security_enabled { 1 } else { 0 });
        let _ = bytes.extend_from_slice(&self.network_key);
        let _ = bytes.extend_from_slice(&self.frame_counter.to_le_bytes());
        bytes
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 45 {
            return None;
        }
        
        let mut offset = 0;
        let pan_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        
        let extended_pan_id = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]);
        offset += 8;
        
        let channel = data[offset];
        offset += 1;
        
        let short_address = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        
        let ieee_address = u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]);
        offset += 8;
        
        let security_enabled = data[offset] != 0;
        offset += 1;
        
        let mut network_key = [0u8; 16];
        network_key.copy_from_slice(&data[offset..offset + 16]);
        offset += 16;
        
        let frame_counter = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        
        Some(Self {
            pan_id,
            extended_pan_id,
            channel,
            short_address,
            ieee_address,
            security_enabled,
            network_key,
            frame_counter,
        })
    }
}

/// Binding table entry for persistence
#[derive(Debug, Clone, Copy)]
pub struct PersistedBinding {
    /// Source endpoint
    pub src_endpoint: u8,
    
    /// Cluster ID
    pub cluster_id: u16,
    
    /// Destination address
    pub dst_address: u64,
    
    /// Destination endpoint
    pub dst_endpoint: u8,
}

impl PersistedBinding {
    /// Size in bytes
    pub const SIZE: usize = 12;
    
    /// Encode to bytes
    pub fn encode(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.src_endpoint;
        bytes[1..3].copy_from_slice(&self.cluster_id.to_le_bytes());
        bytes[3..11].copy_from_slice(&self.dst_address.to_le_bytes());
        bytes[11] = self.dst_endpoint;
        bytes
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        Some(Self {
            src_endpoint: data[0],
            cluster_id: u16::from_le_bytes([data[1], data[2]]),
            dst_address: u64::from_le_bytes([
                data[3], data[4], data[5], data[6],
                data[7], data[8], data[9], data[10],
            ]),
            dst_endpoint: data[11],
        })
    }
}

/// Group membership entry for persistence
#[derive(Debug, Clone, Copy)]
pub struct PersistedGroup {
    /// Group address
    pub group_address: u16,
    
    /// Endpoint
    pub endpoint: u8,
}

impl PersistedGroup {
    /// Size in bytes
    pub const SIZE: usize = 3;
    
    /// Encode to bytes
    pub fn encode(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..2].copy_from_slice(&self.group_address.to_le_bytes());
        bytes[2] = self.endpoint;
        bytes
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        Some(Self {
            group_address: u16::from_le_bytes([data[0], data[1]]),
            endpoint: data[2],
        })
    }
}

/// Storage entry header
#[derive(Debug, Clone, Copy)]
struct StorageEntry {
    /// Key identifier
    key: u8,
    
    /// Data length
    length: u16,
    
    /// CRC16 of data
    crc: u16,
    
    /// Entry valid flag
    valid: bool,
}

impl StorageEntry {
    const SIZE: usize = 6;
    
    fn encode(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = self.key;
        bytes[1..3].copy_from_slice(&self.length.to_le_bytes());
        bytes[3..5].copy_from_slice(&self.crc.to_le_bytes());
        bytes[5] = if self.valid { 0xFF } else { 0x00 };
        bytes
    }
    
    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        
        Some(Self {
            key: data[0],
            length: u16::from_le_bytes([data[1], data[2]]),
            crc: u16::from_le_bytes([data[3], data[4]]),
            valid: data[5] == 0xFF,
        })
    }
}

/// Persistent storage manager
pub struct PersistentStorage {
    /// Base address in flash
    base_address: u32,
    
    /// Storage size
    size: u32,
    
    /// Current write offset
    write_offset: u32,
    
    /// Initialized flag
    initialized: bool,
}

impl PersistentStorage {
    /// Flash sector size (4KB)
    pub const SECTOR_SIZE: u32 = 4096;
    
    /// Default storage size (2 sectors = 8KB)
    pub const DEFAULT_SIZE: u32 = Self::SECTOR_SIZE * 2;
    
    /// Magic number for identification
    const MAGIC: u32 = 0x5A494742; // "ZIGB"
    
    /// Version
    const VERSION: u8 = 0x01;
    
    /// Create a new persistent storage instance
    ///
    /// # Arguments
    ///
    /// * `base_address` - Flash address (must be sector-aligned)
    /// * `size` - Storage size in bytes (must be multiple of sector size)
    pub fn new(base_address: u32, size: u32) -> Self {
        Self {
            base_address,
            size,
            write_offset: 0,
            initialized: false,
        }
    }
    
    /// Initialize storage (format if needed)
    pub fn init(&mut self) -> Result<()> {
        // Check if storage is formatted
        if !self.is_formatted()? {
            self.format()?;
        }
        
        // Scan to find write offset
        self.scan_entries()?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if storage is formatted
    fn is_formatted(&self) -> Result<bool> {
        // Read magic and version from start of storage
        let mut header = [0u8; 8];
        self.read_flash(self.base_address, &mut header)?;
        
        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let version = header[4];
        
        Ok(magic == Self::MAGIC && version == Self::VERSION)
    }
    
    /// Format storage
    pub fn format(&mut self) -> Result<()> {
        // Erase all sectors
        for sector in 0..(self.size / Self::SECTOR_SIZE) {
            let addr = self.base_address + (sector * Self::SECTOR_SIZE);
            self.erase_sector(addr)?;
        }
        
        // Write header
        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&Self::MAGIC.to_le_bytes());
        header[4] = Self::VERSION;
        header[5] = 0; // Reserved
        header[6..8].copy_from_slice(&self.size.to_le_bytes()[0..2]);
        
        self.write_flash(self.base_address, &header)?;
        
        self.write_offset = 8;
        Ok(())
    }
    
    /// Scan entries to find write offset
    fn scan_entries(&mut self) -> Result<()> {
        let mut offset = 8u32; // Skip header
        
        while offset < self.size {
            let mut entry_data = [0u8; StorageEntry::SIZE];
            self.read_flash(self.base_address + offset, &mut entry_data)?;
            
            // Check if entry is erased (all 0xFF)
            if entry_data.iter().all(|&b| b == 0xFF) {
                break;
            }
            
            if let Some(entry) = StorageEntry::decode(&entry_data) {
                if entry.valid {
                    offset += StorageEntry::SIZE as u32 + entry.length as u32;
                } else {
                    offset += StorageEntry::SIZE as u32;
                }
            } else {
                break;
            }
        }
        
        self.write_offset = offset;
        Ok(())
    }
    
    /// Write data to storage
    pub fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<()> {
        if !self.initialized {
            return Err(StorageError::InvalidParameter);
        }
        
        if data.len() > u16::MAX as usize {
            return Err(StorageError::InvalidParameter);
        }
        
        // Check if we need to compact
        let required_space = (StorageEntry::SIZE + data.len()) as u32;
        if self.write_offset + required_space > self.size {
            self.compact()?;
            
            // Check again after compaction
            if self.write_offset + required_space > self.size {
                return Err(StorageError::StorageFull);
            }
        }
        
        // Calculate CRC
        let crc = self.calculate_crc(data);
        
        // Create entry
        let entry = StorageEntry {
            key: key as u8,
            length: data.len() as u16,
            crc,
            valid: true,
        };
        
        // Write entry header
        let entry_bytes = entry.encode();
        self.write_flash(self.base_address + self.write_offset, &entry_bytes)?;
        self.write_offset += StorageEntry::SIZE as u32;
        
        // Write data
        self.write_flash(self.base_address + self.write_offset, data)?;
        self.write_offset += data.len() as u32;
        
        Ok(())
    }
    
    /// Read data from storage
    pub fn read(&self, key: StorageKey, buffer: &mut [u8]) -> Result<usize> {
        if !self.initialized {
            return Err(StorageError::InvalidParameter);
        }
        
        let mut offset = 8u32; // Skip header
        let target_key = key as u8;
        
        // Scan from end to beginning to get latest value
        let mut found_offset: Option<(u32, u16, u16)> = None;
        
        while offset < self.write_offset {
            let mut entry_data = [0u8; StorageEntry::SIZE];
            self.read_flash(self.base_address + offset, &mut entry_data)?;
            
            if let Some(entry) = StorageEntry::decode(&entry_data) {
                if entry.valid && entry.key == target_key {
                    found_offset = Some((offset + StorageEntry::SIZE as u32, entry.length, entry.crc));
                }
                offset += StorageEntry::SIZE as u32 + entry.length as u32;
            } else {
                break;
            }
        }
        
        if let Some((data_offset, length, expected_crc)) = found_offset {
            if buffer.len() < length as usize {
                return Err(StorageError::InvalidParameter);
            }
            
            // Read data
            self.read_flash(self.base_address + data_offset, &mut buffer[..length as usize])?;
            
            // Verify CRC
            let actual_crc = self.calculate_crc(&buffer[..length as usize]);
            if actual_crc != expected_crc {
                return Err(StorageError::CrcMismatch);
            }
            
            Ok(length as usize)
        } else {
            Err(StorageError::NotFound)
        }
    }
    
    /// Delete a key
    pub fn delete(&mut self, key: StorageKey) -> Result<()> {
        // Mark entry as invalid (will be removed during compaction)
        // For now, just write a new empty entry
        self.write(key, &[])?;
        Ok(())
    }
    
    /// Compact storage (remove deleted entries)
    fn compact(&mut self) -> Result<()> {
        // This is a simplified compaction - in production, would use
        // a second sector for atomic operations
        
        // For now, just reset if >75% full
        if self.write_offset > (self.size * 3) / 4 {
            self.format()?;
        }
        
        Ok(())
    }
    
    /// Calculate CRC16 for data
    fn calculate_crc(&self, data: &[u8]) -> u16 {
        let mut crc = 0xFFFFu16;
        
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        crc
    }
    
    /// Read from flash (platform-specific)
    fn read_flash(&self, address: u32, buffer: &mut [u8]) -> Result<()> {
        #[cfg(feature = "esp-storage")]
        {
            use esp_storage::FlashStorage;
            
            FlashStorage::read(address, buffer)
                .map_err(|_| StorageError::ReadFailed)
        }
        
        #[cfg(not(feature = "esp-storage"))]
        {
            // Stub for testing
            Ok(())
        }
    }
    
    /// Write to flash (platform-specific)
    fn write_flash(&self, address: u32, data: &[u8]) -> Result<()> {
        #[cfg(feature = "esp-storage")]
        {
            use esp_storage::FlashStorage;
            
            FlashStorage::write(address, data)
                .map_err(|_| StorageError::WriteFailed)
        }
        
        #[cfg(not(feature = "esp-storage"))]
        {
            // Stub for testing
            Ok(())
        }
    }
    
    /// Erase sector (platform-specific)
    fn erase_sector(&self, address: u32) -> Result<()> {
        #[cfg(feature = "esp-storage")]
        {
            use esp_storage::FlashStorage;
            
            FlashStorage::erase(address, Self::SECTOR_SIZE as usize)
                .map_err(|_| StorageError::EraseFailed)
        }
        
        #[cfg(not(feature = "esp-storage"))]
        {
            // Stub for testing
            Ok(())
        }
    }
    
    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        StorageStats {
            total_size: self.size,
            used_size: self.write_offset,
            free_size: self.size.saturating_sub(self.write_offset),
            initialized: self.initialized,
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Copy)]
pub struct StorageStats {
    /// Total storage size
    pub total_size: u32,
    
    /// Used space
    pub used_size: u32,
    
    /// Free space
    pub free_size: u32,
    
    /// Initialized flag
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_config_encode_decode() {
        let config = PersistedNetworkConfig {
            pan_id: 0x1234,
            extended_pan_id: 0x0011223344556677,
            channel: 15,
            short_address: 0x0001,
            ieee_address: 0x8877665544332211,
            security_enabled: true,
            network_key: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            frame_counter: 1000,
        };
        
        let encoded = config.encode();
        let decoded = PersistedNetworkConfig::decode(&encoded).unwrap();
        
        assert_eq!(config.pan_id, decoded.pan_id);
        assert_eq!(config.extended_pan_id, decoded.extended_pan_id);
        assert_eq!(config.channel, decoded.channel);
        assert_eq!(config.short_address, decoded.short_address);
        assert_eq!(config.ieee_address, decoded.ieee_address);
        assert_eq!(config.security_enabled, decoded.security_enabled);
        assert_eq!(config.network_key, decoded.network_key);
        assert_eq!(config.frame_counter, decoded.frame_counter);
    }
    
    #[test]
    fn test_binding_encode_decode() {
        let binding = PersistedBinding {
            src_endpoint: 1,
            cluster_id: 0x0006,
            dst_address: 0x1122334455667788,
            dst_endpoint: 1,
        };
        
        let encoded = binding.encode();
        let decoded = PersistedBinding::decode(&encoded).unwrap();
        
        assert_eq!(binding.src_endpoint, decoded.src_endpoint);
        assert_eq!(binding.cluster_id, decoded.cluster_id);
        assert_eq!(binding.dst_address, decoded.dst_address);
        assert_eq!(binding.dst_endpoint, decoded.dst_endpoint);
    }
    
    #[test]
    fn test_group_encode_decode() {
        let group = PersistedGroup {
            group_address: 0x0001,
            endpoint: 1,
        };
        
        let encoded = group.encode();
        let decoded = PersistedGroup::decode(&encoded).unwrap();
        
        assert_eq!(group.group_address, decoded.group_address);
        assert_eq!(group.endpoint, decoded.endpoint);
    }
    
    #[test]
    fn test_crc_calculation() {
        let storage = PersistentStorage::new(0, 8192);
        let data = b"Hello, Zigbee!";
        let crc = storage.calculate_crc(data);
        assert!(crc != 0);
        assert!(crc != 0xFFFF);
    }
}
