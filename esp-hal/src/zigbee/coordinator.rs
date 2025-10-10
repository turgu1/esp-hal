//! Zigbee Coordinator implementation
//!
//! The coordinator is responsible for forming and managing a Zigbee network.

use super::{NetworkError, Result, network::*};

/// Coordinator-specific functionality
pub struct Coordinator {
    /// Devices currently in the network
    devices: heapless::Vec<DeviceInfo, 64>,
    
    /// Trust center link keys
    trust_center_keys: heapless::Vec<TrustCenterKey, 32>,
    
    /// Permit join status
    permit_join_remaining: u8,
}

/// Device information stored by coordinator
#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    /// Network address assigned to device
    pub network_address: NetworkAddress,
    
    /// IEEE address
    pub ieee_address: IeeeAddress,
    
    /// Device type
    pub device_type: DeviceCapability,
    
    /// Parent address (for routers and end devices)
    pub parent_address: Option<NetworkAddress>,
    
    /// Depth in network
    pub depth: u8,
    
    /// Link quality
    pub lqi: u8,
    
    /// RSSI
    pub rssi: i8,
    
    /// Last seen timestamp
    pub last_seen: u32,
    
    /// Endpoints
    pub endpoints: heapless::Vec<u8, 16>,
}

/// Device capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCapability {
    /// End device
    EndDevice { sleepy: bool },
    
    /// Router
    Router,
}

/// Trust center link key entry
#[derive(Debug, Clone, Copy)]
pub struct TrustCenterKey {
    /// IEEE address of device
    pub ieee_address: IeeeAddress,
    
    /// Link key
    pub key: [u8; 16],
    
    /// Key sequence number
    pub sequence: u32,
}

impl Coordinator {
    /// Create a new coordinator
    pub(crate) fn new() -> Self {
        Self {
            devices: heapless::Vec::new(),
            trust_center_keys: heapless::Vec::new(),
            permit_join_remaining: 0,
        }
    }
    
    /// Set permit join duration
    ///
    /// # Arguments
    ///
    /// * `duration` - Duration in seconds (0 = close, 255 = always open)
    pub fn set_permit_join(&mut self, duration: u8) {
        self.permit_join_remaining = duration;
    }
    
    /// Get permit join status
    pub fn is_permit_join_enabled(&self) -> bool {
        self.permit_join_remaining > 0
    }
    
    /// Decrement permit join timer (call every second)
    pub fn tick_permit_join(&mut self) {
        if self.permit_join_remaining > 0 && self.permit_join_remaining < 255 {
            self.permit_join_remaining -= 1;
        }
    }
    
    /// Add a device to the network
    pub fn add_device(&mut self, device: DeviceInfo) -> Result<()> {
        // Check if device already exists
        if let Some(existing) = self.find_device_mut(device.ieee_address) {
            // Update existing device
            *existing = device;
            Ok(())
        } else {
            // Add new device
            self.devices
                .push(device)
                .map_err(|_| NetworkError::DeviceNotFound)
        }
    }
    
    /// Remove a device from the network
    pub fn remove_device(&mut self, ieee_address: IeeeAddress) {
        self.devices.retain(|d| d.ieee_address != ieee_address);
        self.trust_center_keys.retain(|k| k.ieee_address != ieee_address);
    }
    
    /// Find device by IEEE address
    pub fn find_device(&self, ieee_address: IeeeAddress) -> Option<&DeviceInfo> {
        self.devices.iter().find(|d| d.ieee_address == ieee_address)
    }
    
    /// Find device by network address
    pub fn find_device_by_network_address(&self, network_address: NetworkAddress) -> Option<&DeviceInfo> {
        self.devices.iter().find(|d| d.network_address == network_address)
    }
    
    /// Find device (mutable)
    fn find_device_mut(&mut self, ieee_address: IeeeAddress) -> Option<&mut DeviceInfo> {
        self.devices.iter_mut().find(|d| d.ieee_address == ieee_address)
    }
    
    /// Get all devices
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }
    
    /// Get device count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    
    /// Allocate a network address for a new device
    pub fn allocate_network_address(&self) -> Option<NetworkAddress> {
        // Simple allocation: find first available address starting from 0x0001
        for addr in 0x0001..=0xFFF7 {
            if !self.devices.iter().any(|d| d.network_address == addr) {
                return Some(addr);
            }
        }
        None
    }
    
    /// Store trust center link key for a device
    pub fn store_trust_center_key(&mut self, ieee_address: IeeeAddress, key: [u8; 16]) -> Result<()> {
        // Check if key already exists
        if let Some(existing) = self.find_trust_center_key_mut(ieee_address) {
            existing.key = key;
            existing.sequence += 1;
            Ok(())
        } else {
            // Add new key
            self.trust_center_keys
                .push(TrustCenterKey {
                    ieee_address,
                    key,
                    sequence: 0,
                })
                .map_err(|_| NetworkError::SecurityFailure)
        }
    }
    
    /// Get trust center link key for a device
    pub fn get_trust_center_key(&self, ieee_address: IeeeAddress) -> Option<&[u8; 16]> {
        self.trust_center_keys
            .iter()
            .find(|k| k.ieee_address == ieee_address)
            .map(|k| &k.key)
    }
    
    /// Find trust center key (mutable)
    fn find_trust_center_key_mut(&mut self, ieee_address: IeeeAddress) -> Option<&mut TrustCenterKey> {
        self.trust_center_keys
            .iter_mut()
            .find(|k| k.ieee_address == ieee_address)
    }
    
    /// Update device last seen timestamp
    pub fn update_device_last_seen(&mut self, ieee_address: IeeeAddress, timestamp: u32) {
        if let Some(device) = self.find_device_mut(ieee_address) {
            device.last_seen = timestamp;
        }
    }
    
    /// Update device link quality
    pub fn update_device_link_quality(&mut self, ieee_address: IeeeAddress, lqi: u8, rssi: i8) {
        if let Some(device) = self.find_device_mut(ieee_address) {
            device.lqi = lqi;
            device.rssi = rssi;
        }
    }
    
    /// Get devices that haven't been seen recently
    pub fn get_stale_devices(&self, current_time: u32, timeout: u32) -> heapless::Vec<IeeeAddress, 64> {
        let mut stale = heapless::Vec::new();
        for device in &self.devices {
            if current_time.saturating_sub(device.last_seen) > timeout {
                let _ = stale.push(device.ieee_address);
            }
        }
        stale
    }
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinator network formation parameters
#[derive(Debug, Clone, Copy)]
pub struct FormationParams {
    /// Scan duration for channel energy scan
    pub scan_duration: u8,
    
    /// Scan all channels or use specified channel
    pub scan_all_channels: bool,
    
    /// Preferred channel (if not scanning all)
    pub channel: u8,
    
    /// Maximum number of association retries
    pub max_retries: u8,
}

impl Default for FormationParams {
    fn default() -> Self {
        Self {
            scan_duration: 3,
            scan_all_channels: true,
            channel: 15,
            max_retries: 3,
        }
    }
}
