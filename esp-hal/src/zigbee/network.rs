//! Zigbee network management

/// Network address type (short address)
pub type NetworkAddress = u16;

/// IEEE address type (extended address)
pub type IeeeAddress = u64;

/// Network information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkInfo {
    /// Network address (short address)
    pub network_address: NetworkAddress,
    
    /// PAN ID
    pub pan_id: u16,
    
    /// Extended PAN ID
    pub extended_pan_id: u64,
    
    /// Channel (11-26)
    pub channel: u8,
    
    /// Parent address (for end devices and routers)
    pub parent_address: Option<NetworkAddress>,
    
    /// Depth in network tree
    pub depth: u8,
    
    /// Link quality indicator
    pub lqi: u8,
    
    /// RSSI
    pub rssi: i8,
    
    /// Security enabled
    pub security_enabled: bool,
}

/// Network key
pub type NetworkKey = [u8; 16];

/// Network state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkState {
    /// Not joined to any network
    Idle,
    
    /// Scanning for networks
    Scanning,
    
    /// Joining a network
    Joining,
    
    /// Connected to network
    Connected,
    
    /// Leaving network
    Leaving,
    
    /// Error state
    Error,
}

/// Neighbor information
##[derive(Debug, Clone, Copy)]
pub struct Neighbor {
    /// Network address
    pub network_address: NetworkAddress,
    
    /// IEEE address
    pub ieee_address: IeeeAddress,
    
    /// Device type
    pub device_type: NeighborDeviceType,
    
    /// Link quality indicator (0-255)
    pub lqi: u8,
    
    /// RSSI in dBm
    pub rssi: i8,
    
    /// Depth in network
    pub depth: u8,
    
    /// Receiving on when idle
    pub rx_on_when_idle: bool,
    
    /// Relationship to this device
    pub relationship: Relationship,
}

/// Neighbor device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborDeviceType {
    Coordinator,
    Router,
    EndDevice,
}

/// Relationship to neighbor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    /// This device is the parent
    Parent,
    
    /// This device is a child
    Child,
    
    /// Sibling (same parent)
    Sibling,
    
    /// No relationship
    None,
    
    /// Previous child (left network)
    PreviousChild,
}

/// Route information
#[derive(Debug, Clone, Copy)]
pub struct Route {
    /// Destination address
    pub destination: NetworkAddress,
    
    /// Next hop address
    pub next_hop: NetworkAddress,
    
    /// Route status
    pub status: RouteStatus,
    
    /// Number of hops to destination
    pub hop_count: u8,
}

/// Route status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStatus {
    Active,
    DiscoveryUnderway,
    DiscoveryFailed,
    Inactive,
    ValidationUnderway,
}

/// Binding entry
#[derive(Debug, Clone, Copy)]
pub struct Binding {
    /// Source IEEE address
    pub source_address: IeeeAddress,
    
    /// Source endpoint
    pub source_endpoint: u8,
    
    /// Cluster ID
    pub cluster_id: u16,
    
    /// Destination address
    pub destination: BindingDestination,
}

/// Binding destination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingDestination {
    /// Direct binding to a device
    Device {
        address: IeeeAddress,
        endpoint: u8,
    },
    
    /// Group binding
    Group { group_id: u16 },
}

/// Network discovery result
#[derive(Debug, Clone, Copy)]
pub struct DiscoveredNetwork {
    /// PAN ID
    pub pan_id: u16,
    
    /// Extended PAN ID
    pub extended_pan_id: u64,
    
    /// Channel
    pub channel: u8,
    
    /// Stack profile
    pub stack_profile: u8,
    
    /// Depth
    pub depth: u8,
    
    /// Permit joining
    pub permit_joining: bool,
    
    /// Link quality
    pub lqi: u8,
    
    /// RSSI
    pub rssi: i8,
}

/// Network parameters for forming a network
#[derive(Debug, Clone, Copy)]
pub struct FormNetworkParams {
    /// PAN ID (None = choose randomly)
    pub pan_id: Option<u16>,
    
    /// Extended PAN ID (None = generate)
    pub extended_pan_id: Option<u64>,
    
    /// Channel (11-26)
    pub channel: u8,
    
    /// Permit join initially
    pub permit_joining: bool,
    
    /// Stack profile (2 = ZigBee PRO)
    pub stack_profile: u8,
}

impl Default for FormNetworkParams {
    fn default() -> Self {
        Self {
            pan_id: None,
            extended_pan_id: None,
            channel: 15,
            permit_joining: true,
            stack_profile: 2,
        }
    }
}

/// Network join parameters
#[derive(Debug, Clone, Copy)]
pub struct JoinNetworkParams {
    /// PAN ID to join (None = any)
    pub pan_id: Option<u16>,
    
    /// Extended PAN ID to join (None = any)
    pub extended_pan_id: Option<u64>,
    
    /// Channel to scan (None = scan all)
    pub channel: Option<u8>,
    
    /// Join as router (vs end device)
    pub join_as_router: bool,
    
    /// Rejoin (vs initial join)
    pub rejoin: bool,
}

impl Default for JoinNetworkParams {
    fn default() -> Self {
        Self {
            pan_id: None,
            extended_pan_id: None,
            channel: None,
            join_as_router: false,
            rejoin: false,
        }
    }
}

/// Network manager - handles network operations
pub struct NetworkManager {
    state: NetworkState,
    network_info: Option<NetworkInfo>,
    neighbors: heapless::Vec<Neighbor, 32>,
    routes: heapless::Vec<Route, 16>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new() -> Self {
        Self {
            state: NetworkState::Idle,
            network_info: None,
            neighbors: heapless::Vec::new(),
            routes: heapless::Vec::new(),
        }
    }
    
    /// Get current network state
    pub fn state(&self) -> NetworkState {
        self.state
    }
    
    /// Get network information
    pub fn network_info(&self) -> Option<&NetworkInfo> {
        self.network_info.as_ref()
    }
    
    /// Get neighbors
    pub fn neighbors(&self) -> &[Neighbor] {
        &self.neighbors
    }
    
    /// Get routes
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }
    
    /// Add a neighbor
    pub fn add_neighbor(&mut self, neighbor: Neighbor) -> Result<(), ()> {
        self.neighbors.push(neighbor).map_err(|_| ())
    }
    
    /// Remove a neighbor
    pub fn remove_neighbor(&mut self, address: NetworkAddress) {
        self.neighbors.retain(|n| n.network_address != address);
    }
    
    /// Find neighbor by address
    pub fn find_neighbor(&self, address: NetworkAddress) -> Option<&Neighbor> {
        self.neighbors.iter().find(|n| n.network_address == address)
    }
    
    /// Add a route
    pub fn add_route(&mut self, route: Route) -> Result<(), ()> {
        self.routes.push(route).map_err(|_| ())
    }
    
    /// Find route to destination
    pub fn find_route(&self, destination: NetworkAddress) -> Option<&Route> {
        self.routes.iter().find(|r| r.destination == destination)
    }
    
    /// Update network state
    pub fn set_state(&mut self, state: NetworkState) {
        self.state = state;
    }
    
    /// Update network info
    pub fn set_network_info(&mut self, info: NetworkInfo) {
        self.network_info = Some(info);
        self.state = NetworkState::Connected;
    }
    
    /// Clear network info (leave network)
    pub fn clear_network_info(&mut self) {
        self.network_info = None;
        self.neighbors.clear();
        self.routes.clear();
        self.state = NetworkState::Idle;
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}
