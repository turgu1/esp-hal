//! Zigbee Device Object (ZDO) implementation
//!
//! Device and service discovery, binding, and network management

use super::network::{NetworkAddress, IeeeAddress};

/// Device announce message
#[derive(Debug, Clone, Copy)]
pub struct DeviceAnnounce {
    /// Network address
    pub network_address: NetworkAddress,
    
    /// IEEE address
    pub ieee_address: IeeeAddress,
    
    /// Capability flags
    pub capability: DeviceCapability,
}

/// Device capability flags
#[derive(Debug, Clone, Copy)]
pub struct DeviceCapability {
    /// Alternate PAN coordinator
    pub alternate_pan_coordinator: bool,
    
    /// Device type (true = FFD, false = RFD)
    pub full_function_device: bool,
    
    /// AC power (true = mains powered, false = battery)
    pub mains_power: bool,
    
    /// Receiver on when idle
    pub rx_on_when_idle: bool,
    
    /// Security capability
    pub security_capable: bool,
    
    /// Allocate address
    pub allocate_address: bool,
}

impl DeviceCapability {
    /// Create capability flags for coordinator
    pub fn coordinator() -> Self {
        Self {
            alternate_pan_coordinator: true,
            full_function_device: true,
            mains_power: true,
            rx_on_when_idle: true,
            security_capable: true,
            allocate_address: true,
        }
    }
    
    /// Create capability flags for router
    pub fn router() -> Self {
        Self {
            alternate_pan_coordinator: false,
            full_function_device: true,
            mains_power: true,
            rx_on_when_idle: true,
            security_capable: true,
            allocate_address: true,
        }
    }
    
    /// Create capability flags for end device
    pub fn end_device(sleepy: bool) -> Self {
        Self {
            alternate_pan_coordinator: false,
            full_function_device: false,
            mains_power: !sleepy,
            rx_on_when_idle: !sleepy,
            security_capable: true,
            allocate_address: true,
        }
    }
    
    /// Encode capability to byte
    pub fn encode(&self) -> u8 {
        let mut byte = 0u8;
        if self.alternate_pan_coordinator {
            byte |= 0x01;
        }
        if self.full_function_device {
            byte |= 0x02;
        }
        if self.mains_power {
            byte |= 0x04;
        }
        if self.rx_on_when_idle {
            byte |= 0x08;
        }
        if self.security_capable {
            byte |= 0x40;
        }
        if self.allocate_address {
            byte |= 0x80;
        }
        byte
    }
    
    /// Decode capability from byte
    pub fn decode(byte: u8) -> Self {
        Self {
            alternate_pan_coordinator: (byte & 0x01) != 0,
            full_function_device: (byte & 0x02) != 0,
            mains_power: (byte & 0x04) != 0,
            rx_on_when_idle: (byte & 0x08) != 0,
            security_capable: (byte & 0x40) != 0,
            allocate_address: (byte & 0x80) != 0,
        }
    }
}

/// Node descriptor
#[derive(Debug, Clone, Copy)]
pub struct NodeDescriptor {
    /// Logical type
    pub logical_type: LogicalType,
    
    /// Complex descriptor available
    pub complex_descriptor_available: bool,
    
    /// User descriptor available
    pub user_descriptor_available: bool,
    
    /// Frequency band
    pub frequency_band: FrequencyBand,
    
    /// MAC capability
    pub mac_capability: DeviceCapability,
    
    /// Manufacturer code
    pub manufacturer_code: u16,
    
    /// Maximum buffer size
    pub max_buffer_size: u8,
    
    /// Maximum incoming transfer size
    pub max_incoming_transfer_size: u16,
    
    /// Maximum outgoing transfer size
    pub max_outgoing_transfer_size: u16,
    
    /// Descriptor capability
    pub descriptor_capability: u8,
}

/// Logical device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalType {
    Coordinator = 0,
    Router = 1,
    EndDevice = 2,
}

/// Frequency band
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyBand {
    /// 2.4 GHz (2400-2483.5 MHz)
    TwoPointFourGHz = 0x08,
}

impl NodeDescriptor {
    /// Create coordinator node descriptor
    pub fn coordinator() -> Self {
        Self {
            logical_type: LogicalType::Coordinator,
            complex_descriptor_available: false,
            user_descriptor_available: false,
            frequency_band: FrequencyBand::TwoPointFourGHz,
            mac_capability: DeviceCapability::coordinator(),
            manufacturer_code: 0x0000,
            max_buffer_size: 127,
            max_incoming_transfer_size: 127,
            max_outgoing_transfer_size: 127,
            descriptor_capability: 0x00,
        }
    }
    
    /// Create router node descriptor
    pub fn router() -> Self {
        Self {
            logical_type: LogicalType::Router,
            complex_descriptor_available: false,
            user_descriptor_available: false,
            frequency_band: FrequencyBand::TwoPointFourGHz,
            mac_capability: DeviceCapability::router(),
            manufacturer_code: 0x0000,
            max_buffer_size: 127,
            max_incoming_transfer_size: 127,
            max_outgoing_transfer_size: 127,
            descriptor_capability: 0x00,
        }
    }
    
    /// Create end device node descriptor
    pub fn end_device(sleepy: bool) -> Self {
        Self {
            logical_type: LogicalType::EndDevice,
            complex_descriptor_available: false,
            user_descriptor_available: false,
            frequency_band: FrequencyBand::TwoPointFourGHz,
            mac_capability: DeviceCapability::end_device(sleepy),
            manufacturer_code: 0x0000,
            max_buffer_size: 127,
            max_incoming_transfer_size: 127,
            max_outgoing_transfer_size: 127,
            descriptor_capability: 0x00,
        }
    }
}

/// Power descriptor
#[derive(Debug, Clone, Copy)]
pub struct PowerDescriptor {
    /// Current power mode
    pub current_power_mode: PowerMode,
    
    /// Available power sources
    pub available_power_sources: PowerSource,
    
    /// Current power source
    pub current_power_source: PowerSource,
    
    /// Current power source level
    pub current_power_source_level: PowerLevel,
}

/// Power mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    /// Receiver on when idle
    ReceiverOnWhenIdle = 0,
    
    /// Receiver periodically on
    ReceiverPeriodicallyOn = 1,
    
    /// Receiver on on request
    ReceiverOnWhenRequested = 2,
}

/// Power source flags
#[derive(Debug, Clone, Copy)]
pub struct PowerSource {
    /// Mains power
    pub mains: bool,
    
    /// Rechargeable battery
    pub rechargeable_battery: bool,
    
    /// Disposable battery
    pub disposable_battery: bool,
}

impl PowerSource {
    /// Mains powered
    pub fn mains() -> Self {
        Self {
            mains: true,
            rechargeable_battery: false,
            disposable_battery: false,
        }
    }
    
    /// Battery powered
    pub fn battery() -> Self {
        Self {
            mains: false,
            rechargeable_battery: false,
            disposable_battery: true,
        }
    }
}

/// Power level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerLevel {
    Critical = 0,
    Low = 4,
    Medium = 8,
    High = 12,
}

/// Simple descriptor (endpoint descriptor)
#[derive(Debug, Clone)]
pub struct SimpleDescriptor {
    /// Endpoint number
    pub endpoint: u8,
    
    /// Application profile ID
    pub profile_id: u16,
    
    /// Application device ID
    pub device_id: u16,
    
    /// Application device version
    pub device_version: u8,
    
    /// Input clusters (server side)
    pub input_clusters: heapless::Vec<u16, 32>,
    
    /// Output clusters (client side)
    pub output_clusters: heapless::Vec<u16, 32>,
}

impl SimpleDescriptor {
    /// Create a new simple descriptor
    pub fn new(endpoint: u8, device_id: u16) -> Self {
        Self {
            endpoint,
            profile_id: 0x0104, // ZigBee Home Automation
            device_id,
            device_version: 0,
            input_clusters: heapless::Vec::new(),
            output_clusters: heapless::Vec::new(),
        }
    }
    
    /// Add input cluster
    pub fn with_input_cluster(mut self, cluster: u16) -> Self {
        let _ = self.input_clusters.push(cluster);
        self
    }
    
    /// Add output cluster
    pub fn with_output_cluster(mut self, cluster: u16) -> Self {
        let _ = self.output_clusters.push(cluster);
        self
    }
}

/// ZDO cluster IDs
pub mod cluster_id {
    pub const NWK_ADDR_REQ: u16 = 0x0000;
    pub const IEEE_ADDR_REQ: u16 = 0x0001;
    pub const NODE_DESC_REQ: u16 = 0x0002;
    pub const POWER_DESC_REQ: u16 = 0x0003;
    pub const SIMPLE_DESC_REQ: u16 = 0x0004;
    pub const ACTIVE_EP_REQ: u16 = 0x0005;
    pub const MATCH_DESC_REQ: u16 = 0x0006;
    pub const DEVICE_ANNCE: u16 = 0x0013;
    pub const BIND_REQ: u16 = 0x0021;
    pub const UNBIND_REQ: u16 = 0x0022;
    pub const MGMT_LEAVE_REQ: u16 = 0x0034;
    pub const MGMT_PERMIT_JOIN_REQ: u16 = 0x0036;
}

/// ZDO status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZdoStatus {
    Success = 0x00,
    InvalidRequestType = 0x80,
    DeviceNotFound = 0x81,
    InvalidEndpoint = 0x82,
    NotActive = 0x83,
    NotSupported = 0x84,
    Timeout = 0x85,
    NoMatch = 0x86,
    NoEntry = 0x88,
    NoDescriptor = 0x89,
    InsufficientSpace = 0x8A,
    NotPermitted = 0x8B,
    TableFull = 0x8C,
    NotAuthorized = 0x8D,
}

impl core::fmt::Display for ZdoStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::InvalidRequestType => write!(f, "Invalid request type"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::InvalidEndpoint => write!(f, "Invalid endpoint"),
            Self::NotActive => write!(f, "Not active"),
            Self::NotSupported => write!(f, "Not supported"),
            Self::Timeout => write!(f, "Timeout"),
            Self::NoMatch => write!(f, "No match"),
            Self::NoEntry => write!(f, "No entry"),
            Self::NoDescriptor => write!(f, "No descriptor"),
            Self::InsufficientSpace => write!(f, "Insufficient space"),
            Self::NotPermitted => write!(f, "Not permitted"),
            Self::TableFull => write!(f, "Table full"),
            Self::NotAuthorized => write!(f, "Not authorized"),
        }
    }
}

impl core::error::Error for ZdoStatus {}
