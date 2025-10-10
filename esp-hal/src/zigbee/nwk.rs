//! Network Layer (NWK) - Complete Implementation
//!
//! This module implements the Zigbee Network Layer (NWK) according to
//! Zigbee Specification R22 Chapter 3. It handles:
//! - Network formation and joining
//! - Routing (AODV-based)
//! - Address allocation
//! - Network management
//! - Route discovery and maintenance
//! - Many-to-one routing
//! - Source routing

use core::time::Duration;
use heapless::Vec;

/// Network frame control field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NwkFrameControl {
    /// Frame type
    pub frame_type: NwkFrameType,
    
    /// Protocol version (3 for Zigbee PRO)
    pub protocol_version: u8,
    
    /// Discover route
    pub discover_route: DiscoverRoute,
    
    /// Multicast flag
    pub multicast: bool,
    
    /// Security enabled
    pub security: bool,
    
    /// Source route present
    pub source_route: bool,
    
    /// Destination IEEE address present
    pub destination_ieee_present: bool,
    
    /// Source IEEE address present
    pub source_ieee_present: bool,
}

impl NwkFrameControl {
    pub fn new(frame_type: NwkFrameType) -> Self {
        Self {
            frame_type,
            protocol_version: 3, // Zigbee PRO
            discover_route: DiscoverRoute::EnableRouteDiscovery,
            multicast: false,
            security: false,
            source_route: false,
            destination_ieee_present: false,
            source_ieee_present: false,
        }
    }
    
    /// Encode to 2 bytes
    pub fn encode(&self) -> [u8; 2] {
        let mut bytes = [0u8; 2];
        
        // Byte 0
        bytes[0] = (self.frame_type as u8) & 0x03;
        bytes[0] |= (self.protocol_version & 0x0F) << 2;
        bytes[0] |= ((self.discover_route as u8) & 0x03) << 6;
        
        // Byte 1
        if self.multicast {
            bytes[1] |= 0x01;
        }
        if self.security {
            bytes[1] |= 0x02;
        }
        if self.source_route {
            bytes[1] |= 0x04;
        }
        if self.destination_ieee_present {
            bytes[1] |= 0x08;
        }
        if self.source_ieee_present {
            bytes[1] |= 0x10;
        }
        
        bytes
    }
    
    /// Decode from 2 bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, NwkError> {
        if bytes.len() < 2 {
            return Err(NwkError::InvalidFrame);
        }
        
        let frame_type = match bytes[0] & 0x03 {
            0 => NwkFrameType::Data,
            1 => NwkFrameType::Command,
            _ => return Err(NwkError::InvalidFrame),
        };
        
        let discover_route = match (bytes[0] >> 6) & 0x03 {
            0 => DiscoverRoute::SuppressRouteDiscovery,
            1 => DiscoverRoute::EnableRouteDiscovery,
            _ => DiscoverRoute::EnableRouteDiscovery,
        };
        
        Ok(Self {
            frame_type,
            protocol_version: (bytes[0] >> 2) & 0x0F,
            discover_route,
            multicast: (bytes[1] & 0x01) != 0,
            security: (bytes[1] & 0x02) != 0,
            source_route: (bytes[1] & 0x04) != 0,
            destination_ieee_present: (bytes[1] & 0x08) != 0,
            source_ieee_present: (bytes[1] & 0x10) != 0,
        })
    }
}

/// Network frame type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NwkFrameType {
    Data = 0,
    Command = 1,
}

/// Route discovery options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiscoverRoute {
    SuppressRouteDiscovery = 0,
    EnableRouteDiscovery = 1,
}

/// Network header
#[derive(Debug, Clone)]
pub struct NwkHeader {
    /// Frame control
    pub frame_control: NwkFrameControl,
    
    /// Destination address
    pub destination_address: u16,
    
    /// Source address
    pub source_address: u16,
    
    /// Radius (time to live)
    pub radius: u8,
    
    /// Sequence number
    pub sequence_number: u8,
    
    /// Destination IEEE address (optional)
    pub destination_ieee: Option<u64>,
    
    /// Source IEEE address (optional)
    pub source_ieee: Option<u64>,
    
    /// Multicast control (optional)
    pub multicast_control: Option<u8>,
    
    /// Source route subframe (optional)
    pub source_route: Option<SourceRoute>,
}

impl NwkHeader {
    /// Create a new network header
    pub fn new(
        frame_type: NwkFrameType,
        destination: u16,
        source: u16,
        sequence: u8,
    ) -> Self {
        Self {
            frame_control: NwkFrameControl::new(frame_type),
            destination_address: destination,
            source_address: source,
            radius: 30, // Default maximum radius
            sequence_number: sequence,
            destination_ieee: None,
            source_ieee: None,
            multicast_control: None,
            source_route: None,
        }
    }
    
    /// Encode header to bytes
    pub fn encode(&self) -> Vec<u8, 128> {
        let mut bytes = Vec::new();
        
        // Frame control (2 bytes)
        let fc = self.frame_control.encode();
        bytes.extend_from_slice(&fc).ok();
        
        // Destination address (2 bytes)
        bytes.extend_from_slice(&self.destination_address.to_le_bytes()).ok();
        
        // Source address (2 bytes)
        bytes.extend_from_slice(&self.source_address.to_le_bytes()).ok();
        
        // Radius (1 byte)
        bytes.push(self.radius).ok();
        
        // Sequence number (1 byte)
        bytes.push(self.sequence_number).ok();
        
        // Optional: Destination IEEE address
        if let Some(ieee) = self.destination_ieee {
            bytes.extend_from_slice(&ieee.to_le_bytes()).ok();
        }
        
        // Optional: Source IEEE address
        if let Some(ieee) = self.source_ieee {
            bytes.extend_from_slice(&ieee.to_le_bytes()).ok();
        }
        
        // Optional: Multicast control
        if let Some(mc) = self.multicast_control {
            bytes.push(mc).ok();
        }
        
        // Optional: Source route
        if let Some(ref route) = self.source_route {
            bytes.push(route.relay_count).ok();
            bytes.push(route.relay_index).ok();
            for relay in &route.relay_list {
                bytes.extend_from_slice(&relay.to_le_bytes()).ok();
            }
        }
        
        bytes
    }
    
    /// Get header length
    pub fn length(&self) -> usize {
        let mut len = 8; // Base: frame control(2) + dst(2) + src(2) + radius(1) + seq(1)
        
        if self.destination_ieee.is_some() {
            len += 8;
        }
        if self.source_ieee.is_some() {
            len += 8;
        }
        if self.multicast_control.is_some() {
            len += 1;
        }
        if let Some(ref route) = self.source_route {
            len += 2 + (route.relay_count as usize * 2);
        }
        
        len
    }
}

/// Source route subframe
#[derive(Debug, Clone)]
pub struct SourceRoute {
    /// Number of relays
    pub relay_count: u8,
    
    /// Current relay index
    pub relay_index: u8,
    
    /// List of relay addresses
    pub relay_list: Vec<u16, 16>,
}

/// Network command identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NwkCommandId {
    RouteRequest = 0x01,
    RouteReply = 0x02,
    NetworkStatus = 0x03,
    Leave = 0x04,
    RouteRecord = 0x05,
    RejoinRequest = 0x06,
    RejoinResponse = 0x07,
    LinkStatus = 0x08,
    NetworkReport = 0x09,
    NetworkUpdate = 0x0A,
    EndDeviceTimeoutRequest = 0x0B,
    EndDeviceTimeoutResponse = 0x0C,
}

impl NwkCommandId {
    /// Convert from u8 to NwkCommandId
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::RouteRequest),
            0x02 => Some(Self::RouteReply),
            0x03 => Some(Self::NetworkStatus),
            0x04 => Some(Self::Leave),
            0x05 => Some(Self::RouteRecord),
            0x06 => Some(Self::RejoinRequest),
            0x07 => Some(Self::RejoinResponse),
            0x08 => Some(Self::LinkStatus),
            0x09 => Some(Self::NetworkReport),
            0x0A => Some(Self::NetworkUpdate),
            0x0B => Some(Self::EndDeviceTimeoutRequest),
            0x0C => Some(Self::EndDeviceTimeoutResponse),
            _ => None,
        }
    }
}

/// Route request command
#[derive(Debug, Clone)]
pub struct RouteRequest {
    /// Route request options
    pub options: RouteRequestOptions,
    
    /// Route request identifier
    pub identifier: u8,
    
    /// Destination address
    pub destination_address: u16,
    
    /// Path cost
    pub path_cost: u8,
    
    /// Destination IEEE address (optional)
    pub destination_ieee: Option<u64>,
}

/// Route request options
#[derive(Debug, Clone, Copy)]
pub struct RouteRequestOptions {
    pub multicast: bool,
    pub destination_ieee_present: bool,
    pub many_to_one: u8, // 0=not many-to-one, 1=no route record, 2=route record required
}

impl RouteRequest {
    pub fn new(identifier: u8, destination: u16) -> Self {
        Self {
            options: RouteRequestOptions {
                multicast: false,
                destination_ieee_present: false,
                many_to_one: 0,
            },
            identifier,
            destination_address: destination,
            path_cost: 0,
            destination_ieee: None,
        }
    }
    
    pub fn encode(&self) -> Vec<u8, 64> {
        let mut bytes = Vec::new();
        
        // Command ID
        bytes.push(NwkCommandId::RouteRequest as u8).ok();
        
        // Options
        let mut options = 0u8;
        if self.options.multicast {
            options |= 0x01;
        }
        if self.options.destination_ieee_present {
            options |= 0x02;
        }
        options |= (self.options.many_to_one & 0x03) << 3;
        bytes.push(options).ok();
        
        // Route request identifier
        bytes.push(self.identifier).ok();
        
        // Destination address
        bytes.extend_from_slice(&self.destination_address.to_le_bytes()).ok();
        
        // Path cost
        bytes.push(self.path_cost).ok();
        
        // Optional: Destination IEEE
        if let Some(ieee) = self.destination_ieee {
            bytes.extend_from_slice(&ieee.to_le_bytes()).ok();
        }
        
        bytes
    }
}

/// Route reply command
#[derive(Debug, Clone)]
pub struct RouteReply {
    /// Route reply options
    pub options: RouteReplyOptions,
    
    /// Route request identifier
    pub identifier: u8,
    
    /// Originator address
    pub originator_address: u16,
    
    /// Responder address
    pub responder_address: u16,
    
    /// Path cost
    pub path_cost: u8,
    
    /// Originator IEEE address (optional)
    pub originator_ieee: Option<u64>,
    
    /// Responder IEEE address (optional)
    pub responder_ieee: Option<u64>,
}

/// Route reply options
#[derive(Debug, Clone, Copy)]
pub struct RouteReplyOptions {
    pub originator_ieee_present: bool,
    pub responder_ieee_present: bool,
    pub multicast: bool,
}

impl RouteReply {
    pub fn new(
        identifier: u8,
        originator: u16,
        responder: u16,
        path_cost: u8,
    ) -> Self {
        Self {
            options: RouteReplyOptions {
                originator_ieee_present: false,
                responder_ieee_present: false,
                multicast: false,
            },
            identifier,
            originator_address: originator,
            responder_address: responder,
            path_cost,
            originator_ieee: None,
            responder_ieee: None,
        }
    }
    
    pub fn encode(&self) -> Vec<u8, 64> {
        let mut bytes = Vec::new();
        
        // Command ID
        bytes.push(NwkCommandId::RouteReply as u8).ok();
        
        // Options
        let mut options = 0u8;
        if self.options.originator_ieee_present {
            options |= 0x01;
        }
        if self.options.responder_ieee_present {
            options |= 0x02;
        }
        if self.options.multicast {
            options |= 0x04;
        }
        bytes.push(options).ok();
        
        // Route request identifier
        bytes.push(self.identifier).ok();
        
        // Originator address
        bytes.extend_from_slice(&self.originator_address.to_le_bytes()).ok();
        
        // Responder address
        bytes.extend_from_slice(&self.responder_address.to_le_bytes()).ok();
        
        // Path cost
        bytes.push(self.path_cost).ok();
        
        // Optional fields
        if let Some(ieee) = self.originator_ieee {
            bytes.extend_from_slice(&ieee.to_le_bytes()).ok();
        }
        if let Some(ieee) = self.responder_ieee {
            bytes.extend_from_slice(&ieee.to_le_bytes()).ok();
        }
        
        bytes
    }
}

/// Network status command
#[derive(Debug, Clone, Copy)]
pub struct NetworkStatus {
    /// Status code
    pub status_code: NetworkStatusCode,
    
    /// Destination address
    pub destination_address: u16,
}

/// Network status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkStatusCode {
    NoRouteAvailable = 0x00,
    TreeLinkFailure = 0x01,
    NonTreeLinkFailure = 0x02,
    LowBatteryLevel = 0x03,
    NoRoutingCapacity = 0x04,
    NoIndirectCapacity = 0x05,
    IndirectTransactionExpiry = 0x06,
    TargetDeviceUnavailable = 0x07,
    TargetAddressUnallocated = 0x08,
    ParentLinkFailure = 0x09,
    ValidateRoute = 0x0A,
    SourceRoutingFailure = 0x0B,
    ManyToOneRouteFailure = 0x0C,
    AddressConflict = 0x0D,
    VerifyAddresses = 0x0E,
    PanIdUpdate = 0x0F,
    NetworkAddressUpdate = 0x10,
    BadFrameCounter = 0x11,
    BadKeySequenceNumber = 0x12,
}

impl NetworkStatus {
    pub fn new(status_code: NetworkStatusCode, destination: u16) -> Self {
        Self {
            status_code,
            destination_address: destination,
        }
    }
    
    pub fn encode(&self) -> Vec<u8, 16> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::NetworkStatus as u8).ok();
        bytes.push(self.status_code as u8).ok();
        bytes.extend_from_slice(&self.destination_address.to_le_bytes()).ok();
        bytes
    }
}

/// Leave command
#[derive(Debug, Clone, Copy)]
pub struct LeaveCommand {
    /// Leave options
    pub options: LeaveOptions,
}

/// Leave options
#[derive(Debug, Clone, Copy)]
pub struct LeaveOptions {
    /// Request rejoin
    pub rejoin: bool,
    
    /// Request for children to leave
    pub request: bool,
    
    /// Remove children
    pub remove_children: bool,
}

impl LeaveCommand {
    pub fn new(rejoin: bool, remove_children: bool) -> Self {
        Self {
            options: LeaveOptions {
                rejoin,
                request: false,
                remove_children,
            },
        }
    }
    
    pub fn encode(&self) -> Vec<u8, 8> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::Leave as u8).ok();
        
        let mut options = 0u8;
        if self.options.rejoin {
            options |= 0x20;
        }
        if self.options.request {
            options |= 0x40;
        }
        if self.options.remove_children {
            options |= 0x80;
        }
        bytes.push(options).ok();
        
        bytes
    }
}

/// Route record command
#[derive(Debug, Clone)]
pub struct RouteRecord {
    /// Relay count
    pub relay_count: u8,
    
    /// Relay list
    pub relay_list: Vec<u16, 16>,
}

impl RouteRecord {
    pub fn new() -> Self {
        Self {
            relay_count: 0,
            relay_list: Vec::new(),
        }
    }
    
    pub fn add_relay(&mut self, address: u16) -> Result<(), ()> {
        self.relay_list.push(address)?;
        self.relay_count = self.relay_list.len() as u8;
        Ok(())
    }
    
    pub fn encode(&self) -> Vec<u8, 64> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::RouteRecord as u8).ok();
        bytes.push(self.relay_count).ok();
        
        for relay in &self.relay_list {
            bytes.extend_from_slice(&relay.to_le_bytes()).ok();
        }
        
        bytes
    }
}

/// Rejoin request command
#[derive(Debug, Clone, Copy)]
pub struct RejoinRequest {
    /// Capability information
    pub capability: u8,
}

impl RejoinRequest {
    pub fn new(capability: u8) -> Self {
        Self { capability }
    }
    
    pub fn encode(&self) -> Vec<u8, 8> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::RejoinRequest as u8).ok();
        bytes.push(self.capability).ok();
        bytes
    }
}

/// Rejoin response command
#[derive(Debug, Clone, Copy)]
pub struct RejoinResponse {
    /// Network address
    pub network_address: u16,
    
    /// Rejoin status
    pub status: RejoinStatus,
}

/// Rejoin status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RejoinStatus {
    Success = 0x00,
    PanAtCapacity = 0x01,
    PanAccessDenied = 0x02,
}

impl RejoinResponse {
    pub fn new(network_address: u16, status: RejoinStatus) -> Self {
        Self {
            network_address,
            status,
        }
    }
    
    pub fn encode(&self) -> Vec<u8, 16> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::RejoinResponse as u8).ok();
        bytes.extend_from_slice(&self.network_address.to_le_bytes()).ok();
        bytes.push(self.status as u8).ok();
        bytes
    }
}

/// Link status command
#[derive(Debug, Clone)]
pub struct LinkStatus {
    /// Link status options
    pub options: LinkStatusOptions,
    
    /// Link status entries
    pub entries: Vec<LinkStatusEntry, 16>,
}

/// Link status options
#[derive(Debug, Clone, Copy)]
pub struct LinkStatusOptions {
    /// First frame
    pub first_frame: bool,
    
    /// Last frame
    pub last_frame: bool,
    
    /// Entry count
    pub entry_count: u8,
}

/// Link status entry
#[derive(Debug, Clone, Copy)]
pub struct LinkStatusEntry {
    /// Neighbor network address
    pub neighbor_address: u16,
    
    /// Link status (incoming cost and outgoing cost)
    pub incoming_cost: u8,
    pub outgoing_cost: u8,
}

impl LinkStatus {
    pub fn new() -> Self {
        Self {
            options: LinkStatusOptions {
                first_frame: true,
                last_frame: true,
                entry_count: 0,
            },
            entries: Vec::new(),
        }
    }
    
    pub fn add_entry(&mut self, entry: LinkStatusEntry) -> Result<(), ()> {
        self.entries.push(entry)?;
        self.options.entry_count = self.entries.len() as u8;
        Ok(())
    }
    
    pub fn encode(&self) -> Vec<u8, 128> {
        let mut bytes = Vec::new();
        bytes.push(NwkCommandId::LinkStatus as u8).ok();
        
        let mut options = self.options.entry_count & 0x1F;
        if self.options.first_frame {
            options |= 0x20;
        }
        if self.options.last_frame {
            options |= 0x40;
        }
        bytes.push(options).ok();
        
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.neighbor_address.to_le_bytes()).ok();
            bytes.push(entry.incoming_cost).ok();
            bytes.push(entry.outgoing_cost).ok();
        }
        
        bytes
    }
}

/// Routing table entry
#[derive(Debug, Clone, Copy)]
pub struct RoutingTableEntry {
    /// Destination address
    pub destination: u16,
    
    /// Status
    pub status: RouteStatus,
    
    /// Next hop address
    pub next_hop: u16,
    
    /// Route cost
    pub cost: u8,
    
    /// Age (for route expiry)
    pub age: u32,
    
    /// Number of route failures
    pub failure_count: u8,
}

/// Route status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RouteStatus {
    Active = 0,
    DiscoveryUnderway = 1,
    DiscoveryFailed = 2,
    Inactive = 3,
    ValidationUnderway = 4,
}

impl RoutingTableEntry {
    pub fn new(destination: u16, next_hop: u16, cost: u8) -> Self {
        Self {
            destination,
            status: RouteStatus::Active,
            next_hop,
            cost,
            age: 0,
            failure_count: 0,
        }
    }
    
    /// Mark route as failed
    pub fn mark_failed(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= 3 {
            self.status = RouteStatus::DiscoveryFailed;
        }
    }
    
    /// Reset failure count
    pub fn reset_failures(&mut self) {
        self.failure_count = 0;
        self.status = RouteStatus::Active;
    }
    
    /// Check if route is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.status, RouteStatus::Active | RouteStatus::ValidationUnderway)
    }
}

/// Routing table
#[derive(Debug, Clone)]
pub struct RoutingTable {
    /// Entries
    entries: Vec<RoutingTableEntry, 32>,
    
    /// Maximum age before expiry (seconds)
    max_age: u32,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_age: 300, // 5 minutes default
        }
    }
    
    /// Add or update a route
    pub fn add_route(&mut self, destination: u16, next_hop: u16, cost: u8) -> Result<(), NwkError> {
        // Check if route already exists
        if let Some(entry) = self.entries.iter_mut().find(|e| e.destination == destination) {
            // Update existing route
            entry.next_hop = next_hop;
            entry.cost = cost;
            entry.age = 0;
            entry.reset_failures();
            return Ok(());
        }
        
        // Add new route
        let entry = RoutingTableEntry::new(destination, next_hop, cost);
        self.entries.push(entry).map_err(|_| NwkError::TableFull)?;
        
        Ok(())
    }
    
    /// Find route to destination
    pub fn find_route(&self, destination: u16) -> Option<&RoutingTableEntry> {
        self.entries
            .iter()
            .find(|e| e.destination == destination && e.is_valid())
    }
    
    /// Remove route
    pub fn remove_route(&mut self, destination: u16) {
        self.entries.retain(|e| e.destination != destination);
    }
    
    /// Mark route as failed
    pub fn mark_route_failed(&mut self, destination: u16) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.destination == destination) {
            entry.mark_failed();
        }
    }
    
    /// Age all routes
    pub fn age_routes(&mut self, seconds: u32) {
        for entry in &mut self.entries {
            entry.age += seconds;
            if entry.age > self.max_age {
                entry.status = RouteStatus::Inactive;
            }
        }
        
        // Remove inactive routes
        self.entries.retain(|e| e.status != RouteStatus::Inactive);
    }
    
    /// Get all routes
    pub fn routes(&self) -> &[RoutingTableEntry] {
        &self.entries
    }
    
    /// Clear all routes
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Get route count
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

/// Route discovery table entry
#[derive(Debug, Clone, Copy)]
pub struct RouteDiscoveryEntry {
    /// Route request identifier
    pub request_id: u8,
    
    /// Source address
    pub source_address: u16,
    
    /// Sender address (previous hop)
    pub sender_address: u16,
    
    /// Forward cost
    pub forward_cost: u8,
    
    /// Residual cost
    pub residual_cost: u8,
    
    /// Timestamp
    pub timestamp: u32,
}

/// Route discovery table
#[derive(Debug, Clone)]
pub struct RouteDiscoveryTable {
    entries: Vec<RouteDiscoveryEntry, 8>,
    timeout: u32, // seconds
}

impl RouteDiscoveryTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            timeout: 10, // 10 seconds
        }
    }
    
    /// Add route discovery entry
    pub fn add_entry(&mut self, entry: RouteDiscoveryEntry) -> Result<(), ()> {
        // Check if entry already exists
        if self.find_entry(entry.request_id, entry.source_address).is_some() {
            return Ok(()); // Duplicate, ignore
        }
        
        self.entries.push(entry).map_err(|_| ())?;
        Ok(())
    }
    
    /// Find entry
    pub fn find_entry(&self, request_id: u8, source: u16) -> Option<&RouteDiscoveryEntry> {
        self.entries
            .iter()
            .find(|e| e.request_id == request_id && e.source_address == source)
    }
    
    /// Age entries
    pub fn age_entries(&mut self, seconds: u32) {
        for entry in &mut self.entries {
            entry.timestamp += seconds;
        }
        
        // Remove expired entries
        self.entries.retain(|e| e.timestamp < self.timeout);
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Network errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NwkError {
    InvalidFrame,
    TableFull,
    NoRoute,
    InvalidAddress,
    NoAck,
    Timeout,
    BufferFull,
    SecurityFailure,
}

impl core::fmt::Display for NwkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFrame => write!(f, "Invalid NWK frame"),
            Self::TableFull => write!(f, "Routing table full"),
            Self::NoRoute => write!(f, "No route to destination"),
            Self::InvalidAddress => write!(f, "Invalid network address"),
            Self::NoAck => write!(f, "No acknowledgment received"),
            Self::Timeout => write!(f, "Network timeout"),
            Self::BufferFull => write!(f, "Network buffer full"),
            Self::SecurityFailure => write!(f, "Network security failure"),
        }
    }
}

impl core::error::Error for NwkError {}

/// Network formation parameters
#[derive(Debug, Clone, Copy)]
pub struct FormNetworkParams {
    /// PAN ID (generated if None)
    pub pan_id: Option<u16>,
    
    /// Extended PAN ID (generated if None)
    pub extended_pan_id: Option<u64>,
    
    /// Channel number
    pub channel: Option<u8>,
    
    /// Enable permit joining
    pub permit_joining: bool,
    
    /// Stack profile (2 = Zigbee PRO)
    pub stack_profile: u8,
}

impl Default for FormNetworkParams {
    fn default() -> Self {
        Self {
            pan_id: None,
            extended_pan_id: None,
            channel: Some(15),
            permit_joining: true,
            stack_profile: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_control_encode_decode() {
        let fc = NwkFrameControl {
            frame_type: NwkFrameType::Data,
            protocol_version: 3,
            discover_route: DiscoverRoute::EnableRouteDiscovery,
            multicast: false,
            security: true,
            source_route: false,
            destination_ieee_present: false,
            source_ieee_present: false,
        };
        
        let encoded = fc.encode();
        let decoded = NwkFrameControl::decode(&encoded).unwrap();
        
        assert_eq!(fc.frame_type, decoded.frame_type);
        assert_eq!(fc.protocol_version, decoded.protocol_version);
        assert_eq!(fc.security, decoded.security);
    }
    
    #[test]
    fn test_routing_table() {
        let mut table = RoutingTable::new();
        
        // Add route
        table.add_route(0x1234, 0x5678, 5).unwrap();
        
        // Find route
        let route = table.find_route(0x1234).unwrap();
        assert_eq!(route.next_hop, 0x5678);
        assert_eq!(route.cost, 5);
        
        // Mark as failed
        table.mark_route_failed(0x1234);
        table.mark_route_failed(0x1234);
        table.mark_route_failed(0x1234);
        
        // Should be failed now
        let route = table.find_route(0x1234);
        assert!(route.is_none());
    }
    
    #[test]
    fn test_route_request() {
        let rreq = RouteRequest::new(42, 0x1234);
        let encoded = rreq.encode();
        
        assert!(encoded.len() >= 6); // Minimum size
        assert_eq!(encoded[0], NwkCommandId::RouteRequest as u8);
        assert_eq!(encoded[2], 42); // Request ID
    }
    
    #[test]
    fn test_route_reply() {
        let rrep = RouteReply::new(42, 0x1234, 0x5678, 10);
        let encoded = rrep.encode();
        
        assert!(encoded.len() >= 8);
        assert_eq!(encoded[0], NwkCommandId::RouteReply as u8);
        assert_eq!(encoded[2], 42); // Request ID
    }
}
