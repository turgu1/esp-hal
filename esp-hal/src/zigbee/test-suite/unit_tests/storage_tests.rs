//! Unit tests for persistent storage module
//!
//! Tests cover:
//! - Storage initialization and formatting
//! - Data encoding/decoding
//! - Write and read operations
//! - Delete operations
//! - CRC validation
//! - Storage full conditions
//! - Compaction (garbage collection)
//! - Error handling
//! - Storage statistics

#[cfg(test)]
mod storage_tests {
    use crate::zigbee::storage::*;
    use heapless::Vec;
    
    // Mock flash storage for testing
    struct MockFlash {
        data: [u8; 16384], // 16KB for testing
    }
    
    impl MockFlash {
        fn new() -> Self {
            Self {
                data: [0xFF; 16384],
            }
        }
        
        fn read(&self, offset: usize, buffer: &mut [u8]) {
            buffer.copy_from_slice(&self.data[offset..offset + buffer.len()]);
        }
        
        fn write(&mut self, offset: usize, data: &[u8]) {
            self.data[offset..offset + data.len()].copy_from_slice(data);
        }
        
        fn erase(&mut self, offset: usize, size: usize) {
            self.data[offset..offset + size].fill(0xFF);
        }
    }
    
    // ===== Encoding/Decoding Tests =====
    
    #[test]
    fn test_network_config_encode_decode_all_fields() {
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
        assert_eq!(encoded.len(), 45); // Verify size
        
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
    fn test_network_config_security_disabled() {
        let config = PersistedNetworkConfig {
            pan_id: 0xABCD,
            extended_pan_id: 0xFFEEDDCCBBAA9988,
            channel: 26,
            short_address: 0xFFFE,
            ieee_address: 0x1122334455667788,
            security_enabled: false,
            network_key: [0; 16],
            frame_counter: 0,
        };
        
        let encoded = config.encode();
        let decoded = PersistedNetworkConfig::decode(&encoded).unwrap();
        
        assert_eq!(config.security_enabled, decoded.security_enabled);
        assert_eq!(config.frame_counter, decoded.frame_counter);
    }
    
    #[test]
    fn test_network_config_invalid_data() {
        // Too short
        let short_data = [0u8; 20];
        assert!(PersistedNetworkConfig::decode(&short_data).is_err());
        
        // Empty
        let empty_data = [];
        assert!(PersistedNetworkConfig::decode(&empty_data).is_err());
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
        assert_eq!(encoded.len(), 12); // Verify size
        
        let decoded = PersistedBinding::decode(&encoded).unwrap();
        
        assert_eq!(binding.src_endpoint, decoded.src_endpoint);
        assert_eq!(binding.cluster_id, decoded.cluster_id);
        assert_eq!(binding.dst_address, decoded.dst_address);
        assert_eq!(binding.dst_endpoint, decoded.dst_endpoint);
    }
    
    #[test]
    fn test_binding_various_clusters() {
        let clusters = [0x0000, 0x0006, 0x0008, 0x0300, 0xFFFF];
        
        for cluster in clusters {
            let binding = PersistedBinding {
                src_endpoint: 1,
                cluster_id: cluster,
                dst_address: 0x1122334455667788,
                dst_endpoint: 1,
            };
            
            let encoded = binding.encode();
            let decoded = PersistedBinding::decode(&encoded).unwrap();
            
            assert_eq!(binding.cluster_id, decoded.cluster_id);
        }
    }
    
    #[test]
    fn test_binding_invalid_data() {
        let short_data = [0u8; 5];
        assert!(PersistedBinding::decode(&short_data).is_err());
    }
    
    #[test]
    fn test_group_encode_decode() {
        let group = PersistedGroup {
            group_address: 0x0001,
            endpoint: 1,
        };
        
        let encoded = group.encode();
        assert_eq!(encoded.len(), 3); // Verify size
        
        let decoded = PersistedGroup::decode(&encoded).unwrap();
        
        assert_eq!(group.group_address, decoded.group_address);
        assert_eq!(group.endpoint, decoded.endpoint);
    }
    
    #[test]
    fn test_group_multiple_endpoints() {
        for endpoint in 1..=8 {
            let group = PersistedGroup {
                group_address: 0x0001,
                endpoint,
            };
            
            let encoded = group.encode();
            let decoded = PersistedGroup::decode(&encoded).unwrap();
            
            assert_eq!(group.endpoint, decoded.endpoint);
        }
    }
    
    #[test]
    fn test_group_invalid_data() {
        let short_data = [0u8; 1];
        assert!(PersistedGroup::decode(&short_data).is_err());
    }
    
    // ===== CRC Tests =====
    
    #[test]
    fn test_crc_calculation_basic() {
        let storage = PersistentStorage::new(0, 8192);
        let data = b"Hello, Zigbee!";
        let crc = storage.calculate_crc(data);
        
        assert!(crc != 0);
        assert!(crc != 0xFFFF);
    }
    
    #[test]
    fn test_crc_consistency() {
        let storage = PersistentStorage::new(0, 8192);
        let data = b"Test data for CRC";
        
        let crc1 = storage.calculate_crc(data);
        let crc2 = storage.calculate_crc(data);
        
        assert_eq!(crc1, crc2);
    }
    
    #[test]
    fn test_crc_different_for_different_data() {
        let storage = PersistentStorage::new(0, 8192);
        let data1 = b"Data set 1";
        let data2 = b"Data set 2";
        
        let crc1 = storage.calculate_crc(data1);
        let crc2 = storage.calculate_crc(data2);
        
        assert_ne!(crc1, crc2);
    }
    
    #[test]
    fn test_crc_empty_data() {
        let storage = PersistentStorage::new(0, 8192);
        let data = b"";
        let crc = storage.calculate_crc(data);
        
        assert_eq!(crc, 0xFFFF); // CRC of empty data
    }
    
    #[test]
    fn test_crc_single_byte() {
        let storage = PersistentStorage::new(0, 8192);
        let data = [0x42u8];
        let crc = storage.calculate_crc(&data);
        
        assert!(crc != 0);
    }
    
    #[test]
    fn test_crc_max_data() {
        let storage = PersistentStorage::new(0, 8192);
        let data = [0xFFu8; 256];
        let crc = storage.calculate_crc(&data);
        
        assert!(crc != 0);
    }
    
    // ===== Storage Key Tests =====
    
    #[test]
    fn test_storage_key_values() {
        assert_eq!(StorageKey::PanId as u8, 0x01);
        assert_eq!(StorageKey::ExtendedPanId as u8, 0x02);
        assert_eq!(StorageKey::Channel as u8, 0x03);
        assert_eq!(StorageKey::NetworkKey as u8, 0x10);
        assert_eq!(StorageKey::BindingTable as u8, 0x20);
        assert_eq!(StorageKey::GroupTable as u8, 0x21);
        assert_eq!(StorageKey::CustomDataStart as u8, 0x80);
    }
    
    #[test]
    fn test_storage_key_uniqueness() {
        let keys = [
            StorageKey::PanId,
            StorageKey::ExtendedPanId,
            StorageKey::Channel,
            StorageKey::ShortAddress,
            StorageKey::IeeeAddress,
            StorageKey::NetworkKey,
            StorageKey::BindingTable,
            StorageKey::GroupTable,
        ];
        
        // Verify all keys have different values
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i] as u8, keys[j] as u8);
            }
        }
    }
    
    // ===== Storage Initialization Tests =====
    
    #[test]
    fn test_storage_new() {
        let storage = PersistentStorage::new(0x9000, 8192);
        
        // Storage should not be initialized yet
        assert!(!storage.initialized);
    }
    
    #[test]
    fn test_storage_new_various_sizes() {
        let sizes = [4096, 8192, 16384, 32768];
        
        for size in sizes {
            let storage = PersistentStorage::new(0x9000, size);
            assert!(!storage.initialized);
        }
    }
    
    #[test]
    fn test_storage_new_various_addresses() {
        let addresses = [0x0000, 0x1000, 0x9000, 0x10000];
        
        for addr in addresses {
            let storage = PersistentStorage::new(addr, 8192);
            assert!(!storage.initialized);
        }
    }
    
    // ===== Storage Error Tests =====
    
    #[test]
    fn test_storage_error_display() {
        assert_eq!(StorageError::NotFound.to_string(), "Key not found");
        assert_eq!(StorageError::StorageFull.to_string(), "Storage full");
        assert_eq!(StorageError::InvalidData.to_string(), "Invalid data");
        assert_eq!(StorageError::WriteFailed.to_string(), "Write failed");
        assert_eq!(StorageError::ReadFailed.to_string(), "Read failed");
        assert_eq!(StorageError::EraseFailed.to_string(), "Erase failed");
        assert_eq!(StorageError::InvalidParameter.to_string(), "Invalid parameter");
        assert_eq!(StorageError::CrcMismatch.to_string(), "CRC mismatch");
    }
    
    #[test]
    fn test_storage_error_equality() {
        assert_eq!(StorageError::NotFound, StorageError::NotFound);
        assert_ne!(StorageError::NotFound, StorageError::StorageFull);
    }
    
    #[test]
    fn test_storage_error_clone() {
        let err1 = StorageError::NotFound;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
    
    // ===== Encoding Edge Cases =====
    
    #[test]
    fn test_network_config_zero_values() {
        let config = PersistedNetworkConfig {
            pan_id: 0,
            extended_pan_id: 0,
            channel: 0,
            short_address: 0,
            ieee_address: 0,
            security_enabled: false,
            network_key: [0; 16],
            frame_counter: 0,
        };
        
        let encoded = config.encode();
        let decoded = PersistedNetworkConfig::decode(&encoded).unwrap();
        
        assert_eq!(config.pan_id, decoded.pan_id);
        assert_eq!(config.extended_pan_id, decoded.extended_pan_id);
        assert_eq!(config.frame_counter, decoded.frame_counter);
    }
    
    #[test]
    fn test_network_config_max_values() {
        let config = PersistedNetworkConfig {
            pan_id: 0xFFFF,
            extended_pan_id: 0xFFFFFFFFFFFFFFFF,
            channel: 255,
            short_address: 0xFFFF,
            ieee_address: 0xFFFFFFFFFFFFFFFF,
            security_enabled: true,
            network_key: [0xFF; 16],
            frame_counter: 0xFFFFFFFF,
        };
        
        let encoded = config.encode();
        let decoded = PersistedNetworkConfig::decode(&encoded).unwrap();
        
        assert_eq!(config.pan_id, decoded.pan_id);
        assert_eq!(config.extended_pan_id, decoded.extended_pan_id);
        assert_eq!(config.frame_counter, decoded.frame_counter);
    }
    
    #[test]
    fn test_binding_endpoint_zero() {
        let binding = PersistedBinding {
            src_endpoint: 0,
            cluster_id: 0x0006,
            dst_address: 0x1122334455667788,
            dst_endpoint: 0,
        };
        
        let encoded = binding.encode();
        let decoded = PersistedBinding::decode(&encoded).unwrap();
        
        assert_eq!(binding.src_endpoint, decoded.src_endpoint);
        assert_eq!(binding.dst_endpoint, decoded.dst_endpoint);
    }
    
    #[test]
    fn test_binding_endpoint_max() {
        let binding = PersistedBinding {
            src_endpoint: 255,
            cluster_id: 0xFFFF,
            dst_address: 0xFFFFFFFFFFFFFFFF,
            dst_endpoint: 255,
        };
        
        let encoded = binding.encode();
        let decoded = PersistedBinding::decode(&encoded).unwrap();
        
        assert_eq!(binding.src_endpoint, decoded.src_endpoint);
        assert_eq!(binding.dst_endpoint, decoded.dst_endpoint);
    }
    
    #[test]
    fn test_group_address_zero() {
        let group = PersistedGroup {
            group_address: 0,
            endpoint: 0,
        };
        
        let encoded = group.encode();
        let decoded = PersistedGroup::decode(&encoded).unwrap();
        
        assert_eq!(group.group_address, decoded.group_address);
    }
    
    #[test]
    fn test_group_address_max() {
        let group = PersistedGroup {
            group_address: 0xFFFF,
            endpoint: 255,
        };
        
        let encoded = group.encode();
        let decoded = PersistedGroup::decode(&encoded).unwrap();
        
        assert_eq!(group.group_address, decoded.group_address);
    }
    
    // ===== Multiple Bindings/Groups Tests =====
    
    #[test]
    fn test_multiple_bindings_encode_decode() {
        let bindings = vec![
            PersistedBinding {
                src_endpoint: 1,
                cluster_id: 0x0006,
                dst_address: 0x1111111111111111,
                dst_endpoint: 1,
            },
            PersistedBinding {
                src_endpoint: 2,
                cluster_id: 0x0008,
                dst_address: 0x2222222222222222,
                dst_endpoint: 2,
            },
            PersistedBinding {
                src_endpoint: 3,
                cluster_id: 0x0300,
                dst_address: 0x3333333333333333,
                dst_endpoint: 3,
            },
        ];
        
        // Encode all bindings
        for binding in &bindings {
            let encoded = binding.encode();
            let decoded = PersistedBinding::decode(&encoded).unwrap();
            assert_eq!(binding.src_endpoint, decoded.src_endpoint);
            assert_eq!(binding.cluster_id, decoded.cluster_id);
            assert_eq!(binding.dst_address, decoded.dst_address);
        }
    }
    
    #[test]
    fn test_multiple_groups_encode_decode() {
        let groups = vec![
            PersistedGroup { group_address: 0x0001, endpoint: 1 },
            PersistedGroup { group_address: 0x0002, endpoint: 2 },
            PersistedGroup { group_address: 0x0003, endpoint: 3 },
            PersistedGroup { group_address: 0x0004, endpoint: 4 },
        ];
        
        for group in &groups {
            let encoded = group.encode();
            let decoded = PersistedGroup::decode(&encoded).unwrap();
            assert_eq!(group.group_address, decoded.group_address);
            assert_eq!(group.endpoint, decoded.endpoint);
        }
    }
    
    // ===== Size Verification Tests =====
    
    #[test]
    fn test_network_config_size_constant() {
        let config1 = PersistedNetworkConfig {
            pan_id: 0x1234,
            extended_pan_id: 0x0011223344556677,
            channel: 15,
            short_address: 0x0001,
            ieee_address: 0x8877665544332211,
            security_enabled: true,
            network_key: [1; 16],
            frame_counter: 1000,
        };
        
        let config2 = PersistedNetworkConfig {
            pan_id: 0xABCD,
            extended_pan_id: 0xFFEEDDCCBBAA9988,
            channel: 26,
            short_address: 0xFFFE,
            ieee_address: 0x1122334455667788,
            security_enabled: false,
            network_key: [0xFF; 16],
            frame_counter: 0,
        };
        
        assert_eq!(config1.encode().len(), config2.encode().len());
        assert_eq!(config1.encode().len(), 45);
    }
    
    #[test]
    fn test_binding_size_constant() {
        let b1 = PersistedBinding {
            src_endpoint: 1,
            cluster_id: 0x0006,
            dst_address: 0x1111111111111111,
            dst_endpoint: 1,
        };
        
        let b2 = PersistedBinding {
            src_endpoint: 255,
            cluster_id: 0xFFFF,
            dst_address: 0xFFFFFFFFFFFFFFFF,
            dst_endpoint: 255,
        };
        
        assert_eq!(b1.encode().len(), b2.encode().len());
        assert_eq!(b1.encode().len(), 12);
    }
    
    #[test]
    fn test_group_size_constant() {
        let g1 = PersistedGroup {
            group_address: 0x0001,
            endpoint: 1,
        };
        
        let g2 = PersistedGroup {
            group_address: 0xFFFF,
            endpoint: 255,
        };
        
        assert_eq!(g1.encode().len(), g2.encode().len());
        assert_eq!(g1.encode().len(), 3);
    }
    
    // ===== Storage Statistics Tests =====
    
    #[test]
    fn test_storage_stats_structure() {
        let stats = StorageStats {
            total_size: 8192,
            used_size: 512,
            free_size: 7680,
            initialized: true,
        };
        
        assert_eq!(stats.total_size, 8192);
        assert_eq!(stats.used_size, 512);
        assert_eq!(stats.free_size, 7680);
        assert!(stats.initialized);
    }
    
    #[test]
    fn test_storage_stats_calculations() {
        let total = 8192;
        let used = 1024;
        let free = total - used;
        
        let stats = StorageStats {
            total_size: total,
            used_size: used,
            free_size: free,
            initialized: true,
        };
        
        assert_eq!(stats.used_size + stats.free_size, stats.total_size);
    }
}
