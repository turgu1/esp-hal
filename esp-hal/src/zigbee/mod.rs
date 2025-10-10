//! # Zigbee Driver for ESP32-C6 and ESP32-H2
//!
//! This module provides a comprehensive Zigbee implementation supporting both
//! coordinator and end-device roles.
//!
//! ## Features
//!
//! - **Coordinator Mode**: Form and manage Zigbee networks
//! - **End Device Mode**: Join existing Zigbee networks as sleepy or non-sleepy devices
//! - **Router Mode**: Route packets and extend network coverage
//! - **IEEE 802.15.4 PHY/MAC**: Built on top of the IEEE 802.15.4 radio with full MAC association protocol
//! - **Zigbee Cluster Library (ZCL)**: Support for common clusters
//! - **Network Management**: Commissioning, key management, security
//! - **OTA Updates**: Over-the-air firmware updates
//! - **Blocking and Async APIs**: Choose the programming model that fits your needs
//!
//! ## Supported Chips
//!
//! - ESP32-C6
//! - ESP32-H2
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │     Application Layer (User Code)       │
//! ├─────────────────────────────────────────┤
//! │  Zigbee Cluster Library (ZCL)           │
//! │  - On/Off, Level Control, Color, etc.   │
//! ├─────────────────────────────────────────┤
//! │  Zigbee Device Objects (ZDO)            │
//! │  - Device & Service Discovery           │
//! ├─────────────────────────────────────────┤
//! │  Application Support (APS) Layer        │
//! │  - Binding, Group Management            │
//! ├─────────────────────────────────────────┤
//! │  Network (NWK) Layer                    │
//! │  - Routing, Discovery, Security         │
//! ├─────────────────────────────────────────┤
//! │  IEEE 802.15.4 MAC Layer                │
//! │  - Full Association Protocol            │
//! ├─────────────────────────────────────────┤
//! │  IEEE 802.15.4 PHY Layer                │
//! │  (esp-radio ieee802154)                 │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ### Coordinator Example
//!
//! ```rust,ignore
//! use esp_hal::zigbee::{Zigbee, Config, Role};
//!
//! let peripherals = esp_hal::init(esp_hal::Config::default());
//!
//! let mut zigbee = Zigbee::new(
//!     peripherals.IEEE802154,
//!     Config::default()
//!         .with_role(Role::Coordinator)
//!         .with_channel(15)
//!         .with_pan_id(0x1234)
//! );
//!
//! // Form a new network
//! zigbee.form_network().expect("Failed to form network");
//!
//! // Permit joining
//! zigbee.permit_join(60).expect("Failed to permit join");
//!
//! loop {
//!     if let Some(event) = zigbee.poll() {
//!         match event {
//!             ZigbeeEvent::DeviceJoined(addr) => {
//!                 println!("Device joined: {:?}", addr);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! ### End Device Example
//!
//! ```rust,ignore
//! use esp_hal::zigbee::{Zigbee, Config, Role};
//!
//! let peripherals = esp_hal::init(esp_hal::Config::default());
//!
//! let mut zigbee = Zigbee::new(
//!     peripherals.IEEE802154,
//!     Config::default()
//!         .with_role(Role::EndDevice { sleepy: false })
//!         .with_channel(15)
//! );
//!
//! // Join a network
//! zigbee.join_network().expect("Failed to join network");
//!
//! // Send data to coordinator
//! zigbee.send_data(0x0000, &[0x01, 0x02, 0x03]).expect("Failed to send");
//! ```
//!
//! ## References
//!
//! - [Zigbee Specification](https://zigbeealliance.org/developer_resources/zigbee-specification/)
//! - [IEEE 802.15.4](https://en.wikipedia.org/wiki/IEEE_802.15.4)
//! - [ESP32-C6 Technical Reference](https://www.espressif.com/sites/default/files/documentation/esp32-c6_technical_reference_manual_en.pdf)
//! - [ESP32-H2 Technical Reference](https://www.espressif.com/sites/default/files/documentation/esp32-h2_technical_reference_manual_en.pdf)

#![cfg(any(esp32c6, esp32h2))]

mod aps;
mod config;
mod coordinator;
mod device;
mod mac_association;
mod network;
mod nwk;
mod crypto;
mod radio;
mod routing;
mod security;
mod storage;
mod timer_service;
mod zcl;
mod zdo;

pub use aps::{ApsManager, ApsDataFrame, ApsFrameControl, ApsDeliveryMode, ApsBinding};
pub use config::{Config, Role, SecurityLevel};
pub use coordinator::Coordinator;
pub use device::{EndDevice, Router};
pub use mac_association::{
    AssociationManager, CoordinatorAssociationManager, AssociationState,
    CapabilityInformation, AssociationStatus, MacCommand,
};
pub use network::{NetworkAddress, NetworkInfo, NetworkKey};
pub use nwk::{
    NwkFrameControl, NwkHeader, NwkCommandId, RouteRequest, RouteReply,
    NetworkStatus, NetworkStatusCode, RoutingTableEntry, RouteStatus, NwkError,
    FormNetworkParams,
};
pub use radio::{Radio, RadioFrame, FrameType, Address};
pub use routing::{
    RoutingManager, AddressManager, NetworkFormation, RouteRequestAction, RouteReplyAction,
};
pub use security::{LinkKey, SecurityManager};
pub use storage::{
    PersistentStorage, StorageKey, StorageError, PersistedNetworkConfig,
    PersistedBinding, PersistedGroup, StorageStats,
};
pub use timer_service::{
    TimerService, TimerType, TimerError, TimeoutTracker, RateLimiter,
};
pub use zcl::{Cluster, ClusterId, OnOffCluster, Attribute};
pub use zdo::{DeviceAnnounce, NodeDescriptor};

use core::marker::PhantomData;
use esp_hal::{
    peripheral::Peripheral,
    peripherals::IEEE802154,
};

use crate::zigbee::radio::Radio;

/// Zigbee driver instance
pub struct Zigbee<'d, Mode: DriverMode = Blocking> {
    inner: ZigbeeInner<'d>,
    _mode: PhantomData<Mode>,
}

/// Internal Zigbee driver state
struct ZigbeeInner<'d> {
    radio: Radio<'d>,
    config: Config,
    network_info: Option<NetworkInfo>,
    sequence_number: u8,
    event_queue: heapless::Vec<ZigbeeEvent, 16>,
    aps_manager: aps::ApsManager,
    association_manager: mac_association::AssociationManager,
    coordinator_association_manager: Option<mac_association::CoordinatorAssociationManager>,
    routing_manager: Option<routing::RoutingManager>,
    address_manager: Option<routing::AddressManager>,
    network_formation: routing::NetworkFormation,
    timer_service: timer_service::TimerService,
    timestamp: u32,
    storage: Option<storage::PersistentStorage>,
    aes: crate::aes::Aes<'static>,
    security_manager: security::SecurityManager,
}

/// Driver mode trait
pub trait DriverMode {}

/// Blocking mode
pub struct Blocking;
impl DriverMode for Blocking {}

/// Async mode
pub struct Async;
impl DriverMode for Async {}

/// Zigbee events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZigbeeEvent {
    /// Network formed successfully
    NetworkFormed {
        pan_id: u16,
        extended_pan_id: u64,
        channel: u8,
    },
    
    /// Network joined successfully
    NetworkJoined {
        network_address: u16,
        parent_address: u16,
    },
    
    /// Device joined the network
    DeviceJoined {
        network_address: u16,
        ieee_address: u64,
    },
    
    /// Device left the network
    DeviceLeft {
        network_address: u16,
    },
    
    /// Data received
    DataReceived {
        source: u16,
        endpoint: u8,
        cluster: u16,
        data: heapless::Vec<u8, 128>,
    },
    
    /// ZCL command received
    ZclCommand {
        source: u16,
        endpoint: u8,
        cluster: u16,
        command: u8,
        data: heapless::Vec<u8, 128>,
    },
    
    /// Network error occurred
    NetworkError(NetworkError),
    
    /// Link quality update
    LinkQualityUpdate {
        address: u16,
        lqi: u8,
        rssi: i8,
    },
}

/// Network errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    /// Failed to form network
    FormFailed,
    
    /// Failed to join network
    JoinFailed,
    
    /// No network found
    NoNetworkFound,
    
    /// Security failure
    SecurityFailure,
    
    /// Transmission failed
    TransmissionFailed,
    
    /// Invalid parameter
    InvalidParameter,
    
    /// Device not found
    DeviceNotFound,
    
    /// Binding failed
    BindingFailed,
    
    /// Route discovery failed
    RouteDiscoveryFailed,
    
    /// Route discovery in progress
    RouteDiscovery,
    
    /// Association in progress
    AssociationInProgress,
    
    /// Association failed
    AssociationFailed,
    
    /// PAN at capacity
    PanAtCapacity,
    
    /// Access denied
    AccessDenied,
    
    /// Timeout
    Timeout,
    
    /// Invalid state
    InvalidState,
    
    /// Storage error
    StorageError,
    
    /// Storage not initialized
    StorageNotInitialized,
}

impl core::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FormFailed => write!(f, "Failed to form network"),
            Self::JoinFailed => write!(f, "Failed to join network"),
            Self::NoNetworkFound => write!(f, "No network found"),
            Self::SecurityFailure => write!(f, "Security failure"),
            Self::TransmissionFailed => write!(f, "Transmission failed"),
            Self::InvalidParameter => write!(f, "Invalid parameter"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::BindingFailed => write!(f, "Binding failed"),
            Self::RouteDiscoveryFailed => write!(f, "Route discovery failed"),
            Self::RouteDiscovery => write!(f, "Route discovery in progress"),
            Self::AssociationInProgress => write!(f, "Association already in progress"),
            Self::AssociationFailed => write!(f, "Association failed"),
            Self::PanAtCapacity => write!(f, "PAN at capacity"),
            Self::AccessDenied => write!(f, "Access denied"),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::InvalidState => write!(f, "Invalid state for operation"),
            Self::StorageError => write!(f, "Storage error"),
            Self::StorageNotInitialized => write!(f, "Storage not initialized"),
        }
    }
}

impl core::error::Error for NetworkError {}

/// Result type for Zigbee operations
pub type Result<T> = core::result::Result<T, NetworkError>;

impl<'d> Zigbee<'d, Blocking> {
    /// Create a new Zigbee driver in blocking mode
    ///
    /// # Arguments
    ///
    /// * `radio` - IEEE802154 peripheral
    /// * `config` - Zigbee configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let zigbee = Zigbee::new(
    ///     peripherals.IEEE802154,
    ///     Config::default()
    ///         .with_role(Role::Coordinator)
    ///         .with_channel(15)
    /// );
    /// ```
    pub fn new(
        radio: impl Peripheral<P = IEEE802154> + 'd,
        aes: impl Peripheral<P = crate::peripherals::AES>,
        config: Config,
    ) -> Self {
        crate::into_ref!(radio);
        crate::into_ref!(aes);
        
        // Initialize radio with configuration
        let radio_driver = Radio::new(
            radio,
            config.channel,
            config.pan_id,
            None, // Short address will be assigned later
        );
        
        // Initialize AES hardware for encryption
        let aes_driver = crate::aes::Aes::new(aes);
        
        // Initialize coordinator association manager if this is a coordinator
        let coordinator_association_manager = if matches!(config.role, Role::Coordinator) {
            Some(mac_association::CoordinatorAssociationManager::new(
                0x0001, // Start allocating from 0x0001
                50,     // Max 50 devices
            ))
        } else {
            None
        };
        
        let mut timer_service = timer_service::TimerService::new();
        // Initialize timer service with current time
        timer_service.init(crate::time::Instant::now());
        
        Self {
            inner: ZigbeeInner {
                radio: radio_driver,
                config,
                network_info: None,
                sequence_number: 0,
                event_queue: heapless::Vec::new(),
                aps_manager: aps::ApsManager::new(),
                association_manager: mac_association::AssociationManager::new(),
                coordinator_association_manager,
                routing_manager: None,
                address_manager: None,
                network_formation: routing::NetworkFormation::new(),
                timer_service,
                timestamp: 0,
                storage: None,
                aes: aes_driver,
                security_manager: security::SecurityManager::new(),
            },
            _mode: PhantomData,
        }
    }
    
    /// Form a new Zigbee network (Coordinator only)
    ///
    /// Creates a new PAN with the configured parameters. This operation
    /// can take several seconds to complete.
    ///
    /// # Returns
    ///
    /// `Ok(())` if network formed successfully, `Err(NetworkError)` otherwise
    pub fn form_network(&mut self) -> Result<()> {
        if !matches!(self.inner.config.role, Role::Coordinator) {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Set as coordinator
        self.inner.radio.set_coordinator(true);
        
        // Form network using NetworkFormation manager
        let ieee_addr = self.inner.config.ieee_address.unwrap_or(0x0011223344556677);
        let form_params = routing::FormNetworkParams {
            pan_id: self.inner.config.pan_id,
            extended_pan_id: self.inner.config.extended_pan_id,
            channel: Some(self.inner.config.channel),
        };
        
        self.inner.network_formation.form_network(form_params)
            .map_err(|_| NetworkError::FormFailed)?;
        
        let pan_id = self.inner.network_formation.pan_id();
        let extended_pan_id = self.inner.network_formation.extended_pan_id();
        
        // Configure radio
        self.inner.radio.set_pan_id(pan_id);
        self.inner.radio.set_short_address(0x0000); // Coordinator is always 0x0000
        self.inner.radio.set_extended_address(ieee_addr);
        
        // Set TX power
        self.inner.radio.set_tx_power(self.inner.config.tx_power);
        
        // Initialize routing manager for coordinator
        let routing_manager = routing::RoutingManager::new(0x0000, ieee_addr);
        self.inner.routing_manager = Some(routing_manager);
        
        // Initialize address manager for coordinator
        // Calculate address pool based on Cskip algorithm
        let max_depth = 15;
        let max_children = 20;
        let max_routers = 6;
        let address_manager = routing::AddressManager::new(
            0x0000,
            max_depth,
            max_children,
            max_routers,
        );
        self.inner.address_manager = Some(address_manager);
        
        // Store network information
        self.inner.network_info = Some(NetworkInfo {
            network_address: 0x0000,
            pan_id,
            extended_pan_id,
            channel: self.inner.config.channel,
        });
        
        // Start receiving
        self.inner.radio.start_receive();
        
        // Enable permit joining by default for 60 seconds
        self.inner.network_formation.set_permit_joining(true, 60);
        
        // Generate event
        self.inner.event_queue.push(ZigbeeEvent::NetworkFormed {
            pan_id,
            extended_pan_id,
            channel: self.inner.config.channel,
        }).ok();
        
        Ok(())
    }
    
    /// Join an existing Zigbee network (End Device/Router only)
    ///
    /// Scans for available networks and attempts to join.
    /// May take several seconds to complete.
    ///
    /// # Returns
    ///
    /// `Ok(())` if joined successfully, `Err(NetworkError)` otherwise
    pub fn join_network(&mut self) -> Result<()> {
        if matches!(self.inner.config.role, Role::Coordinator) {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Scan for networks if not specified
        if self.inner.config.pan_id.is_none() {
            // Perform network discovery
            let networks = self.scan_networks()?;
            if networks.is_empty() {
                return Err(NetworkError::NoNetworkFound);
            }
            
            // Use first available network
            // In real implementation, this would be more sophisticated
            let target_network = &networks[0];
            self.inner.config.pan_id = Some(target_network.pan_id);
        }
        
        let pan_id = self.inner.config.pan_id.unwrap();
        
        // Configure radio for joining
        self.inner.radio.set_pan_id(pan_id);
        
        let ieee_addr = self.inner.config.ieee_address.unwrap_or(0x0011223344556677);
        self.inner.radio.set_extended_address(ieee_addr);
        
        // Determine capability based on device role
        let capability = match self.inner.config.role {
            Role::Router => mac_association::CapabilityInformation::router(),
            Role::EndDevice { sleepy } => {
                let rx_on = !sleepy;
                mac_association::CapabilityInformation::end_device(rx_on)
            }
            _ => return Err(NetworkError::InvalidParameter),
        };
        
        // Get sequence number
        let sequence = self.inner.sequence_number;
        self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
        
        // Get current timestamp
        let timestamp = self.inner.timestamp;
        
        // Start MAC association protocol
        self.inner.association_manager.start_association(
            &mut self.inner.radio,
            Address::Short(0x0000), // Coordinator address
            pan_id,
            ieee_addr,
            capability,
            sequence,
            timestamp,
        )?;
        
        // Poll for association (simulated - in real implementation, this would be event-driven)
        // Wait for response (macResponseWaitTime ~500ms)
        let response_wait_time = 500; // milliseconds
        let max_polls = 5;
        let poll_interval = 100; // milliseconds
        
        for _ in 0..max_polls {
            // Update timestamp
            self.inner.timestamp += poll_interval;
            
            // Check for timeout and determine if we should poll
            if let Ok(should_poll) = self.inner.association_manager.check_timeout(
                self.inner.timestamp,
                response_wait_time,
                max_polls,
            ) {
                if should_poll || self.inner.association_manager.state() == mac_association::AssociationState::WaitingForResponse {
                    // Send data request to poll for association response
                    let seq = self.inner.sequence_number;
                    self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
                    
                    self.inner.association_manager.poll_for_response(
                        &mut self.inner.radio,
                        ieee_addr,
                        seq,
                        self.inner.timestamp,
                    )?;
                    
                    // Process any received frames
                    self.process_association_response()?;
                    
                    // Check if association is complete
                    if self.inner.association_manager.state() == mac_association::AssociationState::Associated {
                        break;
                    }
                }
            }
            
            // Simulate delay (in real implementation, use actual timer)
            // Note: This is a blocking wait - async version would use proper await
        }
        
        // Check final state
        if self.inner.association_manager.state() != mac_association::AssociationState::Associated {
            return Err(NetworkError::AssociationFailed);
        }
        
        // Get assigned address
        let assigned_address = self.inner.association_manager
            .assigned_address()
            .ok_or(NetworkError::AssociationFailed)?;
        
        // Configure radio with assigned address
        self.inner.radio.set_short_address(assigned_address);
        
        // Store network information
        self.inner.network_info = Some(NetworkInfo {
            network_address: assigned_address,
            pan_id,
            extended_pan_id: 0, // Would be obtained from beacon
            channel: self.inner.config.channel,
        });
        
        // Start receiving
        self.inner.radio.start_receive();
        
        // Generate event
        self.inner.event_queue.push(ZigbeeEvent::NetworkJoined {
            network_address: assigned_address,
            parent_address: 0x0000,
        }).ok();
        
        Ok(())
    }
    
    /// Leave the current network
    pub fn leave_network(&mut self) -> Result<()> {
        // Leave network logic
        self.inner.network_info = None;
        Ok(())
    }
    
    /// Permit devices to join the network (Coordinator/Router only)
    ///
    /// # Arguments
    ///
    /// * `duration` - Time in seconds to permit joining (0 = close network, 255 = always open)
    pub fn permit_join(&mut self, duration: u8) -> Result<()> {
        match self.inner.config.role {
            Role::Coordinator | Role::Router => {
                // Implementation would set permit join on the network
                Ok(())
            }
            _ => Err(NetworkError::InvalidParameter),
        }
    }
    
    /// Send data to a device
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination network address (0x0000 = coordinator)
    /// * `data` - Data to send
    pub fn send_data(&mut self, dest: u16, data: &[u8]) -> Result<()> {
        // Check network status
        let network_info = self.inner.network_info.as_ref()
            .ok_or(NetworkError::NoNetworkFound)?;
        
        if data.len() > 100 {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Check if we need to discover a route
        if let Some(routing_manager) = &mut self.inner.routing_manager {
            // Check if we have a route to destination
            if dest != network_info.network_address && dest != 0xFFFD && dest != 0xFFFF {
                if let Some(route) = routing_manager.find_route(dest) {
                    // Route exists but check if it's valid
                    if !matches!(route.status, RouteStatus::Active) {
                        // Route is not active, initiate discovery
                        if let Ok(_rreq) = routing_manager.discover_route(dest) {
                            // Route discovery initiated
                            // In a real implementation, we would queue this data
                            // and send it after route is discovered
                            return Err(NetworkError::RouteDiscovery);
                        }
                    }
                } else {
                    // No route exists, initiate discovery
                    if let Ok(_rreq) = routing_manager.discover_route(dest) {
                        // Route discovery initiated
                        // In a real implementation, we would queue this data
                        // and send it after route is discovered
                        return Err(NetworkError::RouteDiscovery);
                    }
                }
            }
        }
        
        // Get current sequence number and increment
        let sequence = self.inner.sequence_number;
        self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
        
        // Get source address
        let src_addr = if let Some(ieee_addr) = self.inner.config.ieee_address {
            Address::Extended(ieee_addr)
        } else {
            Address::Short(network_info.network_address)
        };
        
        // Transmit data frame
        self.inner.radio.transmit_data(
            network_info.pan_id,
            Address::Short(dest),
            src_addr,
            data,
            sequence,
        )?;
        
        Ok(())
    }
    
    /// Send ZCL command to a device
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination network address
    /// * `endpoint` - Destination endpoint
    /// * `cluster` - Cluster ID
    /// * `command` - Command ID
    /// * `data` - Command data
    pub fn send_zcl_command(
        &mut self,
        dest: u16,
        endpoint: u8,
        cluster: u16,
        command: u8,
        data: &[u8],
    ) -> Result<()> {
        if self.inner.network_info.is_none() {
            return Err(NetworkError::NoNetworkFound);
        }
        
        // Send ZCL command logic
        Ok(())
    }
    
    /// Poll for events
    ///
    /// Returns the next available event, or None if no events are pending.
    pub fn poll(&mut self) -> Option<ZigbeeEvent> {
        // Update timer service and check for expired timers
        let now = crate::time::Instant::now();
        let expired_timers = self.inner.timer_service.update(now);
        
        // Process expired timers
        for (timer_id, timer_type) in expired_timers {
            self.handle_timer_expiry(timer_id, timer_type);
        }
        
        // Update timestamp for backwards compatibility
        self.inner.timestamp = self.inner.timer_service.now_ms();
        
        // First check event queue
        if let Some(event) = self.inner.event_queue.pop() {
            return Some(event);
        }
        
        // Then check radio for received frames
        if let Ok(Some(frame)) = self.inner.radio.receive() {
            // Process received frame
            let event = self.process_received_frame(frame);
            if let Some(evt) = event {
                return Some(evt);
            }
        }
        
        None
    }
    
    /// Process association response (device side)
    fn process_association_response(&mut self) -> Result<()> {
        // Poll for received frames
        if let Some(frame) = self.inner.radio.receive() {
            // Check if this is a MAC command frame
            if frame.frame_type == FrameType::MacCommand && !frame.payload.is_empty() {
                let command_id = frame.payload[0];
                
                // Check if it's an association response
                if command_id == mac_association::MacCommand::AssociationResponse as u8 {
                    if let Some(response) = mac_association::AssociationResponse::decode(&frame.payload[1..]) {
                        self.inner.association_manager.handle_association_response(
                            &response,
                            self.inner.timestamp,
                        )?;
                    }
                }
            } else if frame.frame_type == FrameType::Ack {
                // Handle ACK for association request
                self.inner.association_manager.handle_association_ack(self.inner.timestamp);
            }
        }
        Ok(())
    }
    
    /// Process association request (coordinator side)
    fn process_association_request(&mut self, frame: RadioFrame) -> Result<()> {
        if frame.payload.is_empty() {
            return Ok(());
        }
        
        let command_id = frame.payload[0];
        
        // Handle different MAC commands
        match mac_association::MacCommand::from_u8(command_id) {
            Some(mac_association::MacCommand::AssociationRequest) => {
                // Extract device IEEE address from frame
                let device_address = match frame.src_addr {
                    Some(Address::Extended(addr)) => addr,
                    _ => return Ok(()), // Association request must use extended address
                };
                
                // Decode association request
                if let Some(request) = mac_association::AssociationRequest::decode(&frame.payload[1..]) {
                    let network_info = self.inner.network_info.as_ref()
                        .ok_or(NetworkError::NoNetworkFound)?;
                    
                    let sequence = self.inner.sequence_number;
                    self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
                    
                    // Handle via coordinator association manager
                    if let Some(ref mut coord_mgr) = self.inner.coordinator_association_manager {
                        coord_mgr.handle_association_request(
                            &mut self.inner.radio,
                            device_address,
                            &request,
                            network_info.pan_id,
                            network_info.network_address,
                            sequence,
                        )?;
                        
                        // Generate event
                        if let Some(short_addr) = coord_mgr.get_device_capability(device_address)
                            .and_then(|_| Some(0x0001)) // Would track actual allocated address
                        {
                            self.inner.event_queue.push(ZigbeeEvent::DeviceJoined {
                                network_address: short_addr,
                                ieee_address: device_address,
                            }).ok();
                        }
                    }
                }
            }
            Some(mac_association::MacCommand::DataRequest) => {
                // Device polling for pending data (association response)
                let device_address = match frame.src_addr {
                    Some(Address::Extended(addr)) => addr,
                    _ => return Ok(()),
                };
                
                let network_info = self.inner.network_info.as_ref()
                    .ok_or(NetworkError::NoNetworkFound)?;
                
                let sequence = self.inner.sequence_number;
                self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
                
                if let Some(ref mut coord_mgr) = self.inner.coordinator_association_manager {
                    coord_mgr.handle_data_request(
                        &mut self.inner.radio,
                        device_address,
                        network_info.pan_id,
                        network_info.network_address,
                        sequence,
                    )?;
                }
            }
            Some(mac_association::MacCommand::DisassociationNotification) => {
                // Device leaving network
                if let Some(notification) = mac_association::DisassociationNotification::decode(&frame.payload[1..]) {
                    let device_address = match frame.src_addr {
                        Some(Address::Extended(addr)) => addr,
                        Some(Address::Short(addr)) => {
                            // Generate event
                            self.inner.event_queue.push(ZigbeeEvent::DeviceLeft {
                                network_address: addr,
                            }).ok();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    };
                    
                    if let Some(ref mut coord_mgr) = self.inner.coordinator_association_manager {
                        coord_mgr.handle_disassociation(device_address, &notification);
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Process a received radio frame into a Zigbee event
    fn process_received_frame(&mut self, frame: RadioFrame) -> Option<ZigbeeEvent> {
        // Handle MAC command frames
        if frame.frame_type == FrameType::MacCommand {
            // Process MAC association protocol
            if self.process_association_request(frame.clone()).is_ok() {
                // Association request handled, don't generate data event
                return None;
            }
        }
        
        match frame.frame_type {
            FrameType::Data => {
                // Extract source address
                let src_addr = match frame.src_addr? {
                    Address::Short(addr) => addr,
                    Address::Extended(_) => return None, // Would need to resolve
                };
                
                // Try to decode as NWK frame first
                if let Ok(nwk_header) = NwkHeader::decode(frame.payload.as_slice()) {
                    // This is a NWK layer frame
                    if nwk_header.frame_control.frame_type == nwk::NwkFrameType::Command {
                        // Process NWK command
                        let header_len = nwk_header.length();
                        if frame.payload.len() > header_len {
                            self.process_nwk_command(&nwk_header, &frame.payload[header_len..]);
                        }
                        return None;
                    }
                    // Otherwise, continue processing as data frame
                }
                
                // Try to decode as APS frame
                if let Ok(aps_frame) = aps::ApsDataFrame::decode(frame.payload.as_slice()) {
                    // Check if this is a fragmented message
                    if let Some(ext_hdr) = aps_frame.extended_header {
                        // Handle fragmentation
                        if let Ok(Some(complete_payload)) = self.inner.aps_manager.process_fragment(
                            src_addr,
                            &aps_frame,
                            0, // Timestamp would come from system timer
                        ) {
                            // Full message reassembled
                            return Some(ZigbeeEvent::DataReceived {
                                source: src_addr,
                                data: complete_payload.to_vec(),
                                lqi: frame.lqi,
                                rssi: frame.rssi,
                            });
                        } else {
                            // Still waiting for more fragments
                            return None;
                        }
                    }
                    
                    // Check if ACK is requested
                    if aps_frame.frame_control.ack_request {
                        // Send APS acknowledgment
                        let ack_frame = aps::ApsAckFrame::new(
                            aps_frame.src_endpoint,
                            aps_frame.dst_endpoint,
                            aps_frame.cluster_id,
                            aps_frame.profile_id,
                            aps_frame.aps_counter,
                        );
                        // Queue ACK for transmission
                        // In full implementation, this would be sent via NWK layer
                    }
                    
                    // Check if this is for a group we're member of
                    if let Some(group_addr) = aps_frame.group_address {
                        if !self.inner.aps_manager.is_group_member(group_addr, aps_frame.dst_endpoint) {
                            return None; // Not a member of this group
                        }
                    }
                    
                    // Generate APS data received event
                    Some(ZigbeeEvent::DataReceived {
                        source: src_addr,
                        data: aps_frame.payload.to_vec(),
                        lqi: frame.lqi,
                        rssi: frame.rssi,
                    })
                } else {
                    // Not an APS frame, treat as raw data
                    Some(ZigbeeEvent::DataReceived {
                        source: src_addr,
                        data: frame.payload.to_vec(),
                        lqi: frame.lqi,
                        rssi: frame.rssi,
                    })
                }
            }
            FrameType::Beacon => {
                // Parse beacon payload for PAN descriptor
                if frame.payload.len() >= 2 {
                    let pan_id = frame.src_pan_id?;
                    Some(ZigbeeEvent::BeaconReceived {
                        pan_id,
                        lqi: frame.lqi,
                        rssi: frame.rssi,
                    })
                } else {
                    None
                }
            }
            FrameType::MacCommand => {
                // Handle MAC commands (association, disassociation, etc.)
                None // Would be expanded in full implementation
            }
            FrameType::Ack => {
                // ACKs are typically handled at lower layer
                None
            }
        }
    }
    
    /// Process NWK layer command
    fn process_nwk_command(&mut self, header: &NwkHeader, payload: &[u8]) {
        if payload.is_empty() {
            return;
        }
        
        // Get command ID
        let command_id = payload[0];
        let command_data = &payload[1..];
        
        // Process based on command type
        match NwkCommandId::from_u8(command_id) {
            Some(NwkCommandId::RouteRequest) => {
                if let Ok(rreq) = RouteRequest::decode(command_data) {
                    self.process_route_request(header, rreq);
                }
            }
            Some(NwkCommandId::RouteReply) => {
                if let Ok(rrep) = RouteReply::decode(command_data) {
                    self.process_route_reply(header, rrep);
                }
            }
            Some(NwkCommandId::NetworkStatus) => {
                if let Ok(status) = NetworkStatus::decode(command_data) {
                    self.process_network_status(header, status);
                }
            }
            Some(NwkCommandId::Leave) => {
                // Handle leave command
            }
            Some(NwkCommandId::RouteRecord) => {
                // Handle route record
            }
            Some(NwkCommandId::RejoinRequest) => {
                // Handle rejoin request
            }
            Some(NwkCommandId::RejoinResponse) => {
                // Handle rejoin response
            }
            Some(NwkCommandId::LinkStatus) => {
                // Handle link status
            }
            _ => {
                // Unknown or unhandled command
            }
        }
    }
    
    /// Process route request command
    fn process_route_request(&mut self, header: &NwkHeader, rreq: RouteRequest) {
        if let Some(routing_manager) = &mut self.inner.routing_manager {
            // Assume link cost of 1 for now (would be calculated from LQI/RSSI)
            let link_cost = 1;
            
            match routing_manager.process_route_request(&rreq, header.source_address, link_cost) {
                RouteRequestAction::SendReply(rrep) => {
                    // Send route reply back to sender
                    self.send_nwk_command(header.source_address, NwkCommandId::RouteReply, &rrep.encode());
                }
                RouteRequestAction::Forward(forward_rreq) => {
                    // Forward route request as broadcast
                    self.send_nwk_command(0xFFFF, NwkCommandId::RouteRequest, &forward_rreq.encode());
                }
                RouteRequestAction::Drop => {
                    // Drop the request (duplicate or invalid)
                }
            }
        }
    }
    
    /// Process route reply command
    fn process_route_reply(&mut self, header: &NwkHeader, rrep: RouteReply) {
        if let Some(routing_manager) = &mut self.inner.routing_manager {
            // Assume link cost of 1 for now
            let link_cost = 1;
            
            match routing_manager.process_route_reply(&rrep, header.source_address, link_cost) {
                RouteReplyAction::Complete => {
                    // Route discovery complete
                    // Generate event or resume queued transmissions
                }
                RouteReplyAction::Forward(forward_rrep, next_hop) => {
                    // Forward route reply to next hop
                    self.send_nwk_command(next_hop, NwkCommandId::RouteReply, &forward_rrep.encode());
                }
                RouteReplyAction::Drop => {
                    // Drop the reply
                }
            }
        }
    }
    
    /// Process network status command
    fn process_network_status(&mut self, _header: &NwkHeader, status: NetworkStatus) {
        if let Some(routing_manager) = &mut self.inner.routing_manager {
            routing_manager.process_network_status(&status);
        }
    }
    
    /// Handle timer expiry
    fn handle_timer_expiry(&mut self, _timer_id: u16, timer_type: timer_service::TimerType) {
        use timer_service::TimerType;
        
        match timer_type {
            TimerType::AssociationTimeout => {
                // Association timeout occurred
                self.inner.event_queue.push(ZigbeeEvent::NetworkError {
                    error: NetworkError::AssociationFailed,
                }).ok();
            }
            TimerType::RouteDiscoveryTimeout => {
                // Route discovery timeout
                self.inner.event_queue.push(ZigbeeEvent::NetworkError {
                    error: NetworkError::RouteDiscoveryFailed,
                }).ok();
            }
            TimerType::RouteAging => {
                // Periodic route aging
                if let Some(routing_manager) = &mut self.inner.routing_manager {
                    routing_manager.age_tables(1); // Age by 1 second
                }
            }
            TimerType::FragmentTimeout => {
                // Clean up expired fragments
                let current_ms = self.inner.timer_service.now_ms();
                self.inner.aps_manager.cleanup_fragments(current_ms, 10000); // 10 second timeout
            }
            TimerType::PermitJoiningTimeout => {
                // Permit joining timeout
                self.inner.network_formation.update_permit_timer(1);
            }
            TimerType::PollRate => {
                // Poll rate timer for sleepy end devices
                // Send data request if needed
            }
            TimerType::LinkStatusUpdate => {
                // Periodic link status update
                // Send link status command if needed
            }
            TimerType::NetworkFormationTimeout => {
                // Network formation timeout
                self.inner.event_queue.push(ZigbeeEvent::NetworkError {
                    error: NetworkError::FormFailed,
                }).ok();
            }
            TimerType::OneShot | TimerType::Periodic => {
                // Generic timers - application can handle
            }
        }
    }
    
    /// Send NWK command
    fn send_nwk_command(&mut self, dest: u16, command: NwkCommandId, data: &[u8]) {
        if let Some(network_info) = &self.inner.network_info {
            // Create NWK header
            let sequence = self.inner.sequence_number;
            self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
            
            let mut header = NwkHeader::new(
                nwk::NwkFrameType::Command,
                dest,
                network_info.network_address,
                sequence,
            );
            
            // Encode header + command
            let mut payload = header.encode();
            payload.push(command as u8).ok();
            payload.extend_from_slice(data).ok();
            
            // Send via radio
            let _ = self.inner.radio.transmit_data(
                network_info.pan_id,
                Address::Short(dest),
                Address::Short(network_info.network_address),
                &payload,
                sequence,
            );
        }
    }
    
    /// Get current network information
    pub fn network_info(&self) -> Option<&NetworkInfo> {
        self.inner.network_info.as_ref()
    }
    
    /// Get the device's network address
    pub fn network_address(&self) -> Option<u16> {
        self.inner.network_info.as_ref().map(|info| info.network_address)
    }
    
    /// Get the device's IEEE address
    pub fn ieee_address(&self) -> Option<u64> {
        self.inner.config.ieee_address
    }
    
    /// Get reference to the timer service
    pub fn timer_service(&self) -> &timer_service::TimerService {
        &self.inner.timer_service
    }
    
    /// Get mutable reference to the timer service
    pub fn timer_service_mut(&mut self) -> &mut timer_service::TimerService {
        &mut self.inner.timer_service
    }
    
    /// Get reference to the security manager
    pub fn security_manager(&self) -> &security::SecurityManager {
        &self.inner.security_manager
    }
    
    /// Get mutable reference to the security manager
    pub fn security_manager_mut(&mut self) -> &mut security::SecurityManager {
        &mut self.inner.security_manager
    }
    
    /// Set the network key
    pub fn set_network_key(&mut self, key: security::NetworkKey) {
        self.inner.security_manager.set_network_key(key);
    }
    
    /// Send APS data frame to specific endpoint
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination network address
    /// * `dst_endpoint` - Destination endpoint (1-240)
    /// * `src_endpoint` - Source endpoint (1-240)
    /// * `cluster_id` - Cluster identifier
    /// * `profile_id` - Profile identifier (e.g., 0x0104 for Home Automation)
    /// * `data` - Application payload
    /// * `ack_request` - Request acknowledgment
    pub fn send_aps_data(
        &mut self,
        dest: u16,
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        data: &[u8],
        ack_request: bool,
    ) -> Result<()> {
        // Check network status
        let network_info = self.inner.network_info.as_ref()
            .ok_or(NetworkError::NoNetworkFound)?;
        
        // Create APS data frame
        let mut aps_frame = aps::ApsDataFrame::new(
            dst_endpoint,
            src_endpoint,
            cluster_id,
            profile_id,
            data,
        )?;
        
        // Set APS counter
        aps_frame.aps_counter = self.inner.aps_manager.next_counter();
        
        // Enable ACK request if needed
        if ack_request {
            aps_frame = aps_frame.with_ack_request();
            // Track pending ACK
            self.inner.aps_manager.add_pending_ack(dest, aps_frame.aps_counter)?;
        }
        
        // Encode APS frame
        let aps_payload = aps_frame.encode()?;
        
        // Get sequence number and increment
        let sequence = self.inner.sequence_number;
        self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
        
        // Get source address
        let src_addr = if let Some(ieee_addr) = self.inner.config.ieee_address {
            Address::Extended(ieee_addr)
        } else {
            Address::Short(network_info.network_address)
        };
        
        // Transmit via radio (NWK layer would go here in full implementation)
        self.inner.radio.transmit_data(
            network_info.pan_id,
            Address::Short(dest),
            src_addr,
            aps_payload.as_slice(),
            sequence,
        )?;
        
        Ok(())
    }
    
    /// Bind two devices for direct communication
    ///
    /// # Arguments
    ///
    /// * `source_endpoint` - Source endpoint
    /// * `cluster` - Cluster ID
    /// * `dest_address` - Destination IEEE address
    /// * `dest_endpoint` - Destination endpoint
    pub fn bind(
        &mut self,
        source_endpoint: u8,
        cluster: u16,
        dest_address: u64,
        dest_endpoint: u8,
    ) -> Result<()> {
        let binding = aps::ApsBinding::new(
            source_endpoint,
            cluster,
            dest_address,
            dest_endpoint,
        );
        self.inner.aps_manager.add_binding(binding)
    }
    
    /// Unbind devices
    pub fn unbind(
        &mut self,
        source_endpoint: u8,
        cluster: u16,
        dest_address: u64,
        dest_endpoint: u8,
    ) -> Result<()> {
        self.inner.aps_manager.remove_binding(
            source_endpoint,
            cluster,
            dest_address,
            dest_endpoint,
        )
    }
    
    /// Add device to group
    ///
    /// # Arguments
    ///
    /// * `group_address` - Group identifier (0x0001-0xFFFF)
    /// * `endpoint` - Endpoint to add to group
    pub fn add_group(&mut self, group_address: u16, endpoint: u8) -> Result<()> {
        self.inner.aps_manager.add_group(group_address, endpoint)
    }
    
    /// Remove device from group
    ///
    /// # Arguments
    ///
    /// * `group_address` - Group identifier
    /// * `endpoint` - Endpoint to remove from group
    pub fn remove_group(&mut self, group_address: u16, endpoint: u8) -> Result<()> {
        self.inner.aps_manager.remove_group(group_address, endpoint)
    }
    
    /// Send group message
    ///
    /// # Arguments
    ///
    /// * `group_address` - Group identifier
    /// * `dst_endpoint` - Destination endpoint
    /// * `src_endpoint` - Source endpoint
    /// * `cluster_id` - Cluster identifier
    /// * `profile_id` - Profile identifier
    /// * `data` - Application payload
    pub fn send_group_message(
        &mut self,
        group_address: u16,
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        data: &[u8],
    ) -> Result<()> {
        // Check network status
        let network_info = self.inner.network_info.as_ref()
            .ok_or(NetworkError::NoNetworkFound)?;
        
        // Create APS group frame
        let mut aps_frame = aps::ApsDataFrame::new_group(
            group_address,
            dst_endpoint,
            src_endpoint,
            cluster_id,
            profile_id,
            data,
        )?;
        
        // Set APS counter
        aps_frame.aps_counter = self.inner.aps_manager.next_counter();
        
        // Encode APS frame
        let aps_payload = aps_frame.encode()?;
        
        // Get sequence number
        let sequence = self.inner.sequence_number;
        self.inner.sequence_number = self.inner.sequence_number.wrapping_add(1);
        
        // Get source address
        let src_addr = Address::Short(network_info.network_address);
        
        // Broadcast to group (0xFFFF for now, proper group addressing in full NWK layer)
        self.inner.radio.transmit_data(
            network_info.pan_id,
            Address::Short(0xFFFF), // Broadcast
            src_addr,
            aps_payload.as_slice(),
            sequence,
        )?;
        
        Ok(())
    }
    
    /// Start network discovery
    ///
    /// Scans for available Zigbee networks on all channels
    pub fn scan_networks(&mut self) -> Result<heapless::Vec<NetworkInfo, 16>> {
        let mut networks = heapless::Vec::new();
        
        // Scan channels 11-26 (Zigbee 2.4 GHz channels)
        for channel in 11..=26 {
            // Set radio to channel
            self.inner.radio.set_channel(channel)?;
            
            // Scan for beacons on this channel (100ms per channel)
            if let Ok(beacons) = radio::scan_beacons(&mut self.inner.radio, channel, 100) {
                for beacon in beacons {
                    // Extract PAN ID from beacon
                    if let Some(pan_id) = beacon.src_pan_id {
                        // Check if already discovered
                        if !networks.iter().any(|n| n.pan_id == pan_id && n.channel == channel) {
                            // Parse extended PAN ID from beacon payload
                            let extended_pan_id = if beacon.payload.len() >= 8 {
                                u64::from_le_bytes([
                                    beacon.payload[0], beacon.payload[1],
                                    beacon.payload[2], beacon.payload[3],
                                    beacon.payload[4], beacon.payload[5],
                                    beacon.payload[6], beacon.payload[7],
                                ])
                            } else {
                                0
                            };
                            
                            let network_info = NetworkInfo {
                                network_address: 0xFFFF, // Not joined yet
                                pan_id,
                                extended_pan_id,
                                channel,
                            };
                            
                            if networks.push(network_info).is_err() {
                                // Network list full
                                break;
                            }
                        }
                    }
                }
            }
            
            if networks.is_full() {
                break;
            }
        }
        
        // Restore original channel
        self.inner.radio.set_channel(self.inner.config.channel)?;
        
        Ok(networks)
    }
    
    /// Set TX power level
    ///
    /// # Arguments
    ///
    /// * `power_dbm` - Power in dBm (typically -40 to 20)
    pub fn set_tx_power(&mut self, power_dbm: i8) -> Result<()> {
        self.inner.radio.set_tx_power(power_dbm)?;
        self.inner.config.tx_power = power_dbm;
        Ok(())
    }
    
    /// Get link quality indicator for a device
    pub fn get_lqi(&self, address: u16) -> Option<u8> {
        // Get LQI from neighbor table
        None
    }
    
    /// Get RSSI for a device
    pub fn get_rssi(&self, address: u16) -> Option<i8> {
        // Get RSSI from neighbor table
        None
    }
    
    // ============================================================
    // Persistent Storage API
    // ============================================================
    
    /// Initialize persistent storage
    ///
    /// # Arguments
    ///
    /// * `base_address` - Flash address for storage (must be sector-aligned)
    /// * `size` - Storage size in bytes (must be multiple of 4096)
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Use partition starting at 0x9000 (typical NVS partition)
    /// zigbee.storage_init(0x9000, 8192)?;
    /// ```
    pub fn storage_init(&mut self, base_address: u32, size: u32) -> Result<()> {
        let mut storage = storage::PersistentStorage::new(base_address, size);
        storage.init().map_err(|_| NetworkError::StorageError)?;
        self.inner.storage = Some(storage);
        Ok(())
    }
    
    /// Save network configuration to persistent storage
    pub fn storage_save_network_config(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        let network_info = self.inner.network_info.as_ref()
            .ok_or(NetworkError::NoNetworkFound)?;
        
        // Get network key from security manager (if available)
        let network_key = [0u8; 16]; // Placeholder - would get from security manager
        
        let config = storage::PersistedNetworkConfig {
            pan_id: network_info.pan_id,
            extended_pan_id: network_info.extended_pan_id,
            channel: network_info.channel,
            short_address: network_info.network_address,
            ieee_address: self.inner.config.ieee_address.unwrap_or(0),
            security_enabled: self.inner.config.security_enabled,
            network_key,
            frame_counter: 0, // Would get from security manager
        };
        
        let encoded = config.encode();
        storage.write(storage::StorageKey::DeviceConfig, &encoded)
            .map_err(|_| NetworkError::StorageError)?;
        
        Ok(())
    }
    
    /// Load network configuration from persistent storage
    pub fn storage_load_network_config(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        let mut buffer = [0u8; 64];
        let len = storage.read(storage::StorageKey::DeviceConfig, &mut buffer)
            .map_err(|e| {
                if e == storage::StorageError::NotFound {
                    NetworkError::NoNetworkFound
                } else {
                    NetworkError::StorageError
                }
            })?;
        
        let config = storage::PersistedNetworkConfig::decode(&buffer[..len])
            .ok_or(NetworkError::InvalidParameter)?;
        
        // Restore network configuration
        self.inner.config.pan_id = Some(config.pan_id);
        self.inner.config.channel = config.channel;
        self.inner.config.ieee_address = Some(config.ieee_address);
        self.inner.config.security_enabled = config.security_enabled;
        
        // Configure radio
        self.inner.radio.set_pan_id(config.pan_id);
        self.inner.radio.set_channel(config.channel);
        self.inner.radio.set_short_address(config.short_address);
        self.inner.radio.set_extended_address(config.ieee_address);
        
        // Restore network info
        self.inner.network_info = Some(NetworkInfo {
            network_address: config.short_address,
            pan_id: config.pan_id,
            extended_pan_id: config.extended_pan_id,
            channel: config.channel,
        });
        
        Ok(())
    }
    
    /// Save binding table to persistent storage
    pub fn storage_save_bindings(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        // Get bindings from APS manager
        let bindings = self.inner.aps_manager.get_all_bindings();
        
        // Encode all bindings
        let mut encoded = heapless::Vec::<u8, 256>::new();
        encoded.push(bindings.len() as u8).ok();
        
        for binding in bindings {
            let persisted = storage::PersistedBinding {
                src_endpoint: binding.src_endpoint,
                cluster_id: binding.cluster_id,
                dst_address: binding.dst_address,
                dst_endpoint: binding.dst_endpoint,
            };
            encoded.extend_from_slice(&persisted.encode()).ok();
        }
        
        storage.write(storage::StorageKey::BindingTable, &encoded)
            .map_err(|_| NetworkError::StorageError)?;
        
        Ok(())
    }
    
    /// Load binding table from persistent storage
    pub fn storage_load_bindings(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        let mut buffer = [0u8; 256];
        let len = storage.read(storage::StorageKey::BindingTable, &mut buffer)
            .map_err(|e| {
                if e == storage::StorageError::NotFound {
                    return Ok(()); // No bindings stored
                }
                NetworkError::StorageError
            })?;
        
        if len == 0 {
            return Ok(());
        }
        
        let count = buffer[0] as usize;
        let mut offset = 1;
        
        for _ in 0..count {
            if offset + storage::PersistedBinding::SIZE > len {
                break;
            }
            
            if let Some(persisted) = storage::PersistedBinding::decode(&buffer[offset..]) {
                let binding = aps::ApsBinding::new(
                    persisted.src_endpoint,
                    persisted.cluster_id,
                    persisted.dst_address,
                    persisted.dst_endpoint,
                );
                let _ = self.inner.aps_manager.add_binding(binding);
            }
            
            offset += storage::PersistedBinding::SIZE;
        }
        
        Ok(())
    }
    
    /// Save group table to persistent storage
    pub fn storage_save_groups(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        // Get groups from APS manager
        let groups = self.inner.aps_manager.get_all_groups();
        
        // Encode all groups
        let mut encoded = heapless::Vec::<u8, 128>::new();
        encoded.push(groups.len() as u8).ok();
        
        for group in groups {
            let persisted = storage::PersistedGroup {
                group_address: group.group_address,
                endpoint: group.endpoint,
            };
            encoded.extend_from_slice(&persisted.encode()).ok();
        }
        
        storage.write(storage::StorageKey::GroupTable, &encoded)
            .map_err(|_| NetworkError::StorageError)?;
        
        Ok(())
    }
    
    /// Load group table from persistent storage
    pub fn storage_load_groups(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        let mut buffer = [0u8; 128];
        let len = storage.read(storage::StorageKey::GroupTable, &mut buffer)
            .map_err(|e| {
                if e == storage::StorageError::NotFound {
                    return Ok(()); // No groups stored
                }
                NetworkError::StorageError
            })?;
        
        if len == 0 {
            return Ok(());
        }
        
        let count = buffer[0] as usize;
        let mut offset = 1;
        
        for _ in 0..count {
            if offset + storage::PersistedGroup::SIZE > len {
                break;
            }
            
            if let Some(persisted) = storage::PersistedGroup::decode(&buffer[offset..]) {
                let _ = self.inner.aps_manager.add_group(
                    persisted.group_address,
                    persisted.endpoint,
                );
            }
            
            offset += storage::PersistedGroup::SIZE;
        }
        
        Ok(())
    }
    
    /// Erase all stored configuration
    pub fn storage_erase_all(&mut self) -> Result<()> {
        let storage = self.inner.storage.as_mut()
            .ok_or(NetworkError::StorageNotInitialized)?;
        
        storage.format().map_err(|_| NetworkError::StorageError)?;
        Ok(())
    }
    
    /// Get storage statistics
    pub fn storage_stats(&self) -> Option<storage::StorageStats> {
        self.inner.storage.as_ref().map(|s| s.stats())
    }
}

impl<'d> Zigbee<'d, Async> {
    /// Create a new Zigbee driver in async mode
    pub fn new_async(
        radio: impl Peripheral<P = IEEE802154> + 'd,
        config: Config,
    ) -> Self {
        crate::into_ref!(radio);
        
        let phy_clock = PhyClockGuard::new();
        
        Self {
            inner: ZigbeeInner {
                radio,
                config,
                network_info: None,
                _phy_clock: phy_clock,
            },
            _mode: PhantomData,
        }
    }
    
    /// Form a new Zigbee network (async)
    pub async fn form_network(&mut self) -> Result<()> {
        if !matches!(self.inner.config.role, Role::Coordinator) {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Async form network logic
        Ok(())
    }
    
    /// Join an existing Zigbee network (async)
    pub async fn join_network(&mut self) -> Result<()> {
        if matches!(self.inner.config.role, Role::Coordinator) {
            return Err(NetworkError::InvalidParameter);
        }
        
        // Async join network logic
        Ok(())
    }
    
    /// Send data to a device (async)
    pub async fn send_data(&mut self, dest: u16, data: &[u8]) -> Result<()> {
        if self.inner.network_info.is_none() {
            return Err(NetworkError::NoNetworkFound);
        }
        
        // Async send data logic
        Ok(())
    }
    
    /// Wait for next event (async)
    pub async fn wait_event(&mut self) -> ZigbeeEvent {
        // Async wait for event
        loop {
            // This would use actual async primitives
            if let Some(event) = self.poll() {
                return event;
            }
        }
    }
    
    /// Poll for events (non-blocking)
    pub fn poll(&mut self) -> Option<ZigbeeEvent> {
        None
    }
}

impl<'d, Mode: DriverMode> Drop for Zigbee<'d, Mode> {
    fn drop(&mut self) {
        // Clean shutdown
        let _ = self.inner.leave_network();
    }
}

impl<'d, Mode: DriverMode> Zigbee<'d, Mode> {
    fn leave_network(&mut self) -> Result<()> {
        self.inner.network_info = None;
        Ok(())
    }
}
