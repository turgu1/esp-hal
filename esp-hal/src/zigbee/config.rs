//! Zigbee configuration structures

/// Zigbee driver configuration
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Device role in the network
    pub role: Role,
    
    /// IEEE 802.15.4 channel (11-26)
    pub channel: u8,
    
    /// PAN ID (0x0000-0xFFFE, None = auto-select)
    pub pan_id: Option<u16>,
    
    /// Extended PAN ID (None = auto-generate)
    pub extended_pan_id: Option<u64>,
    
    /// IEEE address (None = use factory programmed address)
    pub ieee_address: Option<u64>,
    
    /// TX power in dBm
    pub tx_power: i8,
    
    /// Enable security
    pub security_enabled: bool,
    
    /// Security level
    pub security_level: SecurityLevel,
    
    /// Network key (None = generate random key)
    pub network_key: Option<[u8; 16]>,
    
    /// Link key (None = use default trust center link key)
    pub link_key: Option<[u8; 16]>,
    
    /// Maximum number of child devices (Coordinator/Router only)
    pub max_children: u8,
    
    /// Maximum depth in network tree
    pub max_depth: u8,
    
    /// Stack profile (ZigBee PRO = 2)
    pub stack_profile: u8,
    
    /// Enable automatic network discovery
    pub auto_discovery: bool,
    
    /// Network scan duration (0-14, higher = longer scan)
    pub scan_duration: u8,
    
    /// Join timeout in seconds
    pub join_timeout: u16,
    
    /// Poll rate for end devices in milliseconds
    pub poll_rate_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            role: Role::EndDevice { sleepy: false },
            channel: 15,
            pan_id: None,
            extended_pan_id: None,
            ieee_address: None,
            tx_power: 10,
            security_enabled: true,
            security_level: SecurityLevel::Standard,
            network_key: None,
            link_key: None,
            max_children: 20,
            max_depth: 5,
            stack_profile: 2, // ZigBee PRO
            auto_discovery: true,
            scan_duration: 3,
            join_timeout: 30,
            poll_rate_ms: 1000,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the device role
    pub fn with_role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }
    
    /// Set the channel
    pub fn with_channel(mut self, channel: u8) -> Self {
        self.channel = channel;
        self
    }
    
    /// Set the PAN ID
    pub fn with_pan_id(mut self, pan_id: u16) -> Self {
        self.pan_id = Some(pan_id);
        self
    }
    
    /// Set the extended PAN ID
    pub fn with_extended_pan_id(mut self, extended_pan_id: u64) -> Self {
        self.extended_pan_id = Some(extended_pan_id);
        self
    }
    
    /// Set the IEEE address
    pub fn with_ieee_address(mut self, ieee_address: u64) -> Self {
        self.ieee_address = Some(ieee_address);
        self
    }
    
    /// Set the TX power
    pub fn with_tx_power(mut self, tx_power: i8) -> Self {
        self.tx_power = tx_power;
        self
    }
    
    /// Enable or disable security
    pub fn with_security(mut self, enabled: bool) -> Self {
        self.security_enabled = enabled;
        self
    }
    
    /// Set the security level
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Set the network key
    pub fn with_network_key(mut self, key: [u8; 16]) -> Self {
        self.network_key = Some(key);
        self
    }
    
    /// Set the link key
    pub fn with_link_key(mut self, key: [u8; 16]) -> Self {
        self.link_key = Some(key);
        self
    }
    
    /// Set maximum number of children
    pub fn with_max_children(mut self, max_children: u8) -> Self {
        self.max_children = max_children;
        self
    }
    
    /// Set the poll rate for end devices
    pub fn with_poll_rate(mut self, poll_rate_ms: u32) -> Self {
        self.poll_rate_ms = poll_rate_ms;
        self
    }
    
    /// Create a coordinator configuration
    pub fn coordinator() -> Self {
        Self::default().with_role(Role::Coordinator)
    }
    
    /// Create an end device configuration
    pub fn end_device(sleepy: bool) -> Self {
        Self::default().with_role(Role::EndDevice { sleepy })
    }
    
    /// Create a router configuration
    pub fn router() -> Self {
        Self::default().with_role(Role::Router)
    }
    
    /// Check if this is a sleepy end device
    pub fn sleepy_end_device(&self) -> bool {
        matches!(self.role, Role::EndDevice { sleepy: true })
    }
}

/// Device role in the Zigbee network
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Coordinator - Forms and manages the network
    ///
    /// The coordinator is responsible for:
    /// - Forming the network
    /// - Managing security keys
    /// - Permitting devices to join
    /// - Acting as trust center
    Coordinator,
    
    /// Router - Routes packets and extends network coverage
    ///
    /// Routers can:
    /// - Join existing networks
    /// - Route packets for other devices
    /// - Have child devices
    /// - Stay powered on continuously
    Router,
    
    /// End Device - Leaf node in the network
    ///
    /// End devices:
    /// - Join existing networks
    /// - Cannot have children
    /// - Cannot route packets
    /// - Can be sleepy (battery powered) or non-sleepy
    EndDevice {
        /// Whether the device enters sleep mode to save power
        sleepy: bool,
    },
}

/// Zigbee security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No security (not recommended)
    None,
    
    /// Standard security (ZigBee 3.0)
    ///
    /// Uses:
    /// - AES-128 encryption
    /// - Network key for data encryption
    /// - Link keys for join/rejoin
    Standard,
    
    /// High security (additional protections)
    ///
    /// Includes all Standard security plus:
    /// - Install codes
    /// - Enhanced security policies
    /// - Frequent key rotation
    High,
}

/// Channel mask for network scanning
#[derive(Debug, Clone, Copy)]
pub struct ChannelMask(pub u32);

impl ChannelMask {
    /// All 2.4 GHz channels (11-26)
    pub const ALL_2_4_GHZ: Self = Self(0x07FFF800);
    
    /// Single channel
    pub const fn single(channel: u8) -> Self {
        Self(1 << channel)
    }
    
    /// Custom mask
    pub const fn from_mask(mask: u32) -> Self {
        Self(mask)
    }
    
    /// Check if channel is in mask
    pub fn contains(&self, channel: u8) -> bool {
        (self.0 & (1 << channel)) != 0
    }
    
    /// Add channel to mask
    pub fn add(&mut self, channel: u8) {
        self.0 |= 1 << channel;
    }
    
    /// Remove channel from mask
    pub fn remove(&mut self, channel: u8) {
        self.0 &= !(1 << channel);
    }
}

impl Default for ChannelMask {
    fn default() -> Self {
        Self::ALL_2_4_GHZ
    }
}

/// Device type identifiers (ZigBee Device Profile)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// On/Off Light (0x0100)
    OnOffLight,
    
    /// Dimmable Light (0x0101)
    DimmableLight,
    
    /// Color Light (0x0102)
    ColorLight,
    
    /// On/Off Switch (0x0103)
    OnOffSwitch,
    
    /// Dimmer Switch (0x0104)
    DimmerSwitch,
    
    /// Temperature Sensor (0x0302)
    TemperatureSensor,
    
    /// Occupancy Sensor (0x0107)
    OccupancySensor,
    
    /// Door Lock (0x000A)
    DoorLock,
    
    /// Smart Plug (0x0051)
    SmartPlug,
    
    /// Custom device type
    Custom(u16),
}

impl DeviceType {
    /// Get the device type ID
    pub fn id(&self) -> u16 {
        match self {
            DeviceType::OnOffLight => 0x0100,
            DeviceType::DimmableLight => 0x0101,
            DeviceType::ColorLight => 0x0102,
            DeviceType::OnOffSwitch => 0x0103,
            DeviceType::DimmerSwitch => 0x0104,
            DeviceType::TemperatureSensor => 0x0302,
            DeviceType::OccupancySensor => 0x0107,
            DeviceType::DoorLock => 0x000A,
            DeviceType::SmartPlug => 0x0051,
            DeviceType::Custom(id) => *id,
        }
    }
}

/// Endpoint configuration
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// Endpoint number (1-240)
    pub endpoint: u8,
    
    /// Device type
    pub device_type: DeviceType,
    
    /// Input clusters (server side)
    pub input_clusters: heapless::Vec<u16, 32>,
    
    /// Output clusters (client side)
    pub output_clusters: heapless::Vec<u16, 32>,
}

impl EndpointConfig {
    /// Create a new endpoint configuration
    pub fn new(endpoint: u8, device_type: DeviceType) -> Self {
        Self {
            endpoint,
            device_type,
            input_clusters: heapless::Vec::new(),
            output_clusters: heapless::Vec::new(),
        }
    }
    
    /// Add an input cluster
    pub fn with_input_cluster(mut self, cluster: u16) -> Self {
        let _ = self.input_clusters.push(cluster);
        self
    }
    
    /// Add an output cluster
    pub fn with_output_cluster(mut self, cluster: u16) -> Self {
        let _ = self.output_clusters.push(cluster);
        self
    }
}
