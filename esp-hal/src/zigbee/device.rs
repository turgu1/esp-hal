//! Zigbee End Device and Router implementation

use super::{NetworkError, Result, network::*};

/// End Device functionality
pub struct EndDevice {
    /// Parent information
    parent: Option<ParentInfo>,
    
    /// Poll rate in milliseconds
    poll_rate_ms: u32,
    
    /// Sleepy mode enabled
    sleepy: bool,
    
    /// Last poll timestamp
    last_poll: u32,
}

/// Parent information
#[derive(Debug, Clone, Copy)]
pub struct ParentInfo {
    /// Parent network address
    pub network_address: NetworkAddress,
    
    /// Parent IEEE address
    pub ieee_address: IeeeAddress,
    
    /// Link quality to parent
    pub lqi: u8,
    
    /// RSSI to parent
    pub rssi: i8,
    
    /// Last communication timestamp
    pub last_communication: u32,
}

impl EndDevice {
    /// Create a new end device
    pub(crate) fn new(sleepy: bool, poll_rate_ms: u32) -> Self {
        Self {
            parent: None,
            poll_rate_ms,
            sleepy,
            last_poll: 0,
        }
    }
    
    /// Set parent information
    pub fn set_parent(&mut self, parent: ParentInfo) {
        self.parent = Some(parent);
    }
    
    /// Get parent information
    pub fn parent(&self) -> Option<&ParentInfo> {
        self.parent.as_ref()
    }
    
    /// Check if parent is set
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    
    /// Clear parent (orphaned)
    pub fn clear_parent(&mut self) {
        self.parent = None;
    }
    
    /// Get poll rate
    pub fn poll_rate_ms(&self) -> u32 {
        self.poll_rate_ms
    }
    
    /// Set poll rate
    pub fn set_poll_rate_ms(&mut self, rate_ms: u32) {
        self.poll_rate_ms = rate_ms;
    }
    
    /// Check if sleepy mode is enabled
    pub fn is_sleepy(&self) -> bool {
        self.sleepy
    }
    
    /// Check if time to poll parent
    pub fn should_poll(&self, current_time: u32) -> bool {
        if !self.sleepy || self.parent.is_none() {
            return false;
        }
        
        current_time.saturating_sub(self.last_poll) >= self.poll_rate_ms
    }
    
    /// Update last poll timestamp
    pub fn update_last_poll(&mut self, timestamp: u32) {
        self.last_poll = timestamp;
    }
    
    /// Update parent link quality
    pub fn update_parent_link_quality(&mut self, lqi: u8, rssi: i8, timestamp: u32) {
        if let Some(parent) = &mut self.parent {
            parent.lqi = lqi;
            parent.rssi = rssi;
            parent.last_communication = timestamp;
        }
    }
    
    /// Check if parent link is stale
    pub fn is_parent_link_stale(&self, current_time: u32, timeout: u32) -> bool {
        if let Some(parent) = &self.parent {
            current_time.saturating_sub(parent.last_communication) > timeout
        } else {
            false
        }
    }
}

/// Router functionality
pub struct Router {
    /// Children information
    children: heapless::Vec<ChildInfo, 32>,
    
    /// Maximum number of children allowed
    max_children: u8,
    
    /// Routing table
    routing_table: heapless::Vec<RoutingEntry, 16>,
}

/// Child device information
#[derive(Debug, Clone, Copy)]
pub struct ChildInfo {
    /// Network address
    pub network_address: NetworkAddress,
    
    /// IEEE address
    pub ieee_address: IeeeAddress,
    
    /// Device capability
    pub capability: ChildCapability,
    
    /// Link quality
    pub lqi: u8,
    
    /// RSSI
    pub rssi: i8,
    
    /// Last seen timestamp
    pub last_seen: u32,
    
    /// Timeout period (for sleepy devices)
    pub timeout: u32,
}

/// Child capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildCapability {
    /// Sleepy end device
    SleepyEndDevice,
    
    /// Non-sleepy end device
    EndDevice,
    
    /// Router
    Router,
}

/// Routing table entry
#[derive(Debug, Clone, Copy)]
pub struct RoutingEntry {
    /// Destination network address
    pub destination: NetworkAddress,
    
    /// Next hop network address
    pub next_hop: NetworkAddress,
    
    /// Status
    pub status: RoutingStatus,
    
    /// Cost (hop count)
    pub cost: u8,
}

/// Routing status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStatus {
    Active,
    DiscoveryUnderway,
    DiscoveryFailed,
    Inactive,
}

impl Router {
    /// Create a new router
    pub(crate) fn new(max_children: u8) -> Self {
        Self {
            children: heapless::Vec::new(),
            max_children,
            routing_table: heapless::Vec::new(),
        }
    }
    
    /// Add a child device
    pub fn add_child(&mut self, child: ChildInfo) -> Result<()> {
        if self.children.len() >= self.max_children as usize {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Check if child already exists
        if let Some(existing) = self.find_child_mut(child.ieee_address) {
            *existing = child;
            Ok(())
        } else {
            self.children
                .push(child)
                .map_err(|_| NetworkError::InvalidParameter)
        }
    }
    
    /// Remove a child device
    pub fn remove_child(&mut self, ieee_address: IeeeAddress) {
        self.children.retain(|c| c.ieee_address != ieee_address);
    }
    
    /// Find child by IEEE address
    pub fn find_child(&self, ieee_address: IeeeAddress) -> Option<&ChildInfo> {
        self.children.iter().find(|c| c.ieee_address == ieee_address)
    }
    
    /// Find child by network address
    pub fn find_child_by_network_address(&self, network_address: NetworkAddress) -> Option<&ChildInfo> {
        self.children.iter().find(|c| c.network_address == network_address)
    }
    
    /// Find child (mutable)
    fn find_child_mut(&mut self, ieee_address: IeeeAddress) -> Option<&mut ChildInfo> {
        self.children.iter_mut().find(|c| c.ieee_address == ieee_address)
    }
    
    /// Get all children
    pub fn children(&self) -> &[ChildInfo] {
        &self.children
    }
    
    /// Get child count
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
    
    /// Check if can accept more children
    pub fn can_accept_children(&self) -> bool {
        self.children.len() < self.max_children as usize
    }
    
    /// Update child last seen timestamp
    pub fn update_child_last_seen(&mut self, ieee_address: IeeeAddress, timestamp: u32) {
        if let Some(child) = self.find_child_mut(ieee_address) {
            child.last_seen = timestamp;
        }
    }
    
    /// Update child link quality
    pub fn update_child_link_quality(&mut self, ieee_address: IeeeAddress, lqi: u8, rssi: i8) {
        if let Some(child) = self.find_child_mut(ieee_address) {
            child.lqi = lqi;
            child.rssi = rssi;
        }
    }
    
    /// Get children that haven't been seen recently
    pub fn get_stale_children(&self, current_time: u32) -> heapless::Vec<IeeeAddress, 32> {
        let mut stale = heapless::Vec::new();
        for child in &self.children {
            if current_time.saturating_sub(child.last_seen) > child.timeout {
                let _ = stale.push(child.ieee_address);
            }
        }
        stale
    }
    
    /// Add routing entry
    pub fn add_route(&mut self, entry: RoutingEntry) -> Result<()> {
        // Check if route exists
        if let Some(existing) = self.find_route_mut(entry.destination) {
            *existing = entry;
            Ok(())
        } else {
            self.routing_table
                .push(entry)
                .map_err(|_| NetworkError::RouteDiscoveryFailed)
        }
    }
    
    /// Remove routing entry
    pub fn remove_route(&mut self, destination: NetworkAddress) {
        self.routing_table.retain(|r| r.destination != destination);
    }
    
    /// Find route to destination
    pub fn find_route(&self, destination: NetworkAddress) -> Option<&RoutingEntry> {
        self.routing_table.iter().find(|r| r.destination == destination)
    }
    
    /// Find route (mutable)
    fn find_route_mut(&mut self, destination: NetworkAddress) -> Option<&mut RoutingEntry> {
        self.routing_table.iter_mut().find(|r| r.destination == destination)
    }
    
    /// Get routing table
    pub fn routing_table(&self) -> &[RoutingEntry] {
        &self.routing_table
    }
    
    /// Get next hop for destination
    pub fn get_next_hop(&self, destination: NetworkAddress) -> Option<NetworkAddress> {
        self.find_route(destination)
            .and_then(|r| {
                if r.status == RoutingStatus::Active {
                    Some(r.next_hop)
                } else {
                    None
                }
            })
    }
}

/// Join network parameters for end devices and routers
#[derive(Debug, Clone, Copy)]
pub struct JoinParams {
    /// Scan duration for active scan
    pub scan_duration: u8,
    
    /// Scan all channels or use specified channel
    pub scan_all_channels: bool,
    
    /// Preferred channel (if not scanning all)
    pub channel: u8,
    
    /// Join timeout in seconds
    pub timeout: u16,
    
    /// Maximum join attempts
    pub max_attempts: u8,
    
    /// Rejoin instead of fresh join
    pub rejoin: bool,
}

impl Default for JoinParams {
    fn default() -> Self {
        Self {
            scan_duration: 3,
            scan_all_channels: true,
            channel: 15,
            timeout: 30,
            max_attempts: 3,
            rejoin: false,
        }
    }
}
