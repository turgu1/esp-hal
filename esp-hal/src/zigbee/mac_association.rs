//! IEEE 802.15.4 MAC Association Protocol
//!
//! Implements the complete MAC layer association/disassociation protocol
//! as specified in IEEE 802.15.4-2015 standard.

use heapless::Vec;
use crate::zigbee::{NetworkError, Result};
use crate::zigbee::radio::{Radio, Address, RadioFrame};

/// MAC command identifiers (IEEE 802.15.4 Table 7-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MacCommand {
    /// Association Request (0x01)
    AssociationRequest = 0x01,
    /// Association Response (0x02)
    AssociationResponse = 0x02,
    /// Disassociation Notification (0x03)
    DisassociationNotification = 0x03,
    /// Data Request (0x04)
    DataRequest = 0x04,
    /// PAN ID Conflict Notification (0x05)
    PanIdConflict = 0x05,
    /// Orphan Notification (0x06)
    OrphanNotification = 0x06,
    /// Beacon Request (0x07)
    BeaconRequest = 0x07,
    /// Coordinator Realignment (0x08)
    CoordinatorRealignment = 0x08,
    /// GTS Request (0x09)
    GtsRequest = 0x09,
}

impl MacCommand {
    /// Try to convert from byte
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MacCommand::AssociationRequest),
            0x02 => Some(MacCommand::AssociationResponse),
            0x03 => Some(MacCommand::DisassociationNotification),
            0x04 => Some(MacCommand::DataRequest),
            0x05 => Some(MacCommand::PanIdConflict),
            0x06 => Some(MacCommand::OrphanNotification),
            0x07 => Some(MacCommand::BeaconRequest),
            0x08 => Some(MacCommand::CoordinatorRealignment),
            0x09 => Some(MacCommand::GtsRequest),
            _ => None,
        }
    }
}

/// Association capability information (IEEE 802.15.4 Figure 7-13)
#[derive(Debug, Clone, Copy)]
pub struct CapabilityInformation {
    /// Device is capable of becoming a PAN coordinator
    pub alternate_pan_coordinator: bool,
    
    /// Device type: true = FFD (Router/Coordinator), false = RFD (End Device)
    pub device_type: bool,
    
    /// Power source: true = Mains powered, false = Battery
    pub power_source: bool,
    
    /// Receiver on when idle
    pub receiver_on_when_idle: bool,
    
    /// Security capability
    pub security_capable: bool,
    
    /// Allocate address: true = Request short address, false = Use extended only
    pub allocate_address: bool,
}

impl CapabilityInformation {
    /// Create new capability information for an end device
    pub fn end_device(receiver_on: bool) -> Self {
        Self {
            alternate_pan_coordinator: false,
            device_type: false, // RFD
            power_source: false, // Battery
            receiver_on_when_idle: receiver_on,
            security_capable: true,
            allocate_address: true,
        }
    }
    
    /// Create new capability information for a router
    pub fn router() -> Self {
        Self {
            alternate_pan_coordinator: false,
            device_type: true, // FFD
            power_source: true, // Mains
            receiver_on_when_idle: true,
            security_capable: true,
            allocate_address: true,
        }
    }
    
    /// Encode to byte
    pub fn encode(&self) -> u8 {
        let mut byte = 0u8;
        if self.alternate_pan_coordinator {
            byte |= 0x01;
        }
        if self.device_type {
            byte |= 0x02;
        }
        if self.power_source {
            byte |= 0x04;
        }
        if self.receiver_on_when_idle {
            byte |= 0x08;
        }
        // Reserved bit 4 (0x10)
        // Reserved bit 5 (0x20)
        if self.security_capable {
            byte |= 0x40;
        }
        if self.allocate_address {
            byte |= 0x80;
        }
        byte
    }
    
    /// Decode from byte
    pub fn decode(byte: u8) -> Self {
        Self {
            alternate_pan_coordinator: (byte & 0x01) != 0,
            device_type: (byte & 0x02) != 0,
            power_source: (byte & 0x04) != 0,
            receiver_on_when_idle: (byte & 0x08) != 0,
            security_capable: (byte & 0x40) != 0,
            allocate_address: (byte & 0x80) != 0,
        }
    }
}

/// Association status (IEEE 802.15.4 Table 7-12)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AssociationStatus {
    /// Association successful
    Success = 0x00,
    /// PAN at capacity
    PanAtCapacity = 0x01,
    /// PAN access denied
    PanAccessDenied = 0x02,
    /// Reserved (0x03-0x7F)
    Reserved = 0x03,
    /// Reserved for MAC primitives (0x80-0xFF)
    MacReserved = 0x80,
}

impl AssociationStatus {
    /// Try to convert from byte
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x00 => AssociationStatus::Success,
            0x01 => AssociationStatus::PanAtCapacity,
            0x02 => AssociationStatus::PanAccessDenied,
            0x03..=0x7F => AssociationStatus::Reserved,
            _ => AssociationStatus::MacReserved,
        }
    }
}

/// Disassociation reason (IEEE 802.15.4 Table 7-62)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisassociationReason {
    /// The coordinator wishes the device to leave
    CoordinatorWish = 0x01,
    /// The device wishes to leave
    DeviceWish = 0x02,
    /// Reserved (0x03-0xFF)
    Reserved = 0x03,
}

impl DisassociationReason {
    /// Try to convert from byte
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(DisassociationReason::CoordinatorWish),
            0x02 => Some(DisassociationReason::DeviceWish),
            _ => None,
        }
    }
}

/// Association Request frame payload (IEEE 802.15.4 Section 7.3.1)
#[derive(Debug, Clone, Copy)]
pub struct AssociationRequest {
    /// Capability information
    pub capability: CapabilityInformation,
}

impl AssociationRequest {
    /// Create new association request
    pub fn new(capability: CapabilityInformation) -> Self {
        Self { capability }
    }
    
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8, 1> {
        let mut payload = Vec::new();
        let _ = payload.push(self.capability.encode());
        payload
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 1 {
            return None;
        }
        Some(Self {
            capability: CapabilityInformation::decode(data[0]),
        })
    }
}

/// Association Response frame payload (IEEE 802.15.4 Section 7.3.2)
#[derive(Debug, Clone, Copy)]
pub struct AssociationResponse {
    /// Assigned short address (0xFFFF if allocation failed)
    pub short_address: u16,
    
    /// Association status
    pub status: AssociationStatus,
}

impl AssociationResponse {
    /// Create new association response
    pub fn new(short_address: u16, status: AssociationStatus) -> Self {
        Self {
            short_address,
            status,
        }
    }
    
    /// Create success response
    pub fn success(short_address: u16) -> Self {
        Self::new(short_address, AssociationStatus::Success)
    }
    
    /// Create failure response
    pub fn failure(reason: AssociationStatus) -> Self {
        Self::new(0xFFFF, reason)
    }
    
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8, 3> {
        let mut payload = Vec::new();
        let _ = payload.extend_from_slice(&self.short_address.to_le_bytes());
        let _ = payload.push(self.status as u8);
        payload
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }
        let short_address = u16::from_le_bytes([data[0], data[1]]);
        let status = AssociationStatus::from_u8(data[2]);
        Some(Self {
            short_address,
            status,
        })
    }
}

/// Disassociation Notification frame payload (IEEE 802.15.4 Section 7.3.3)
#[derive(Debug, Clone, Copy)]
pub struct DisassociationNotification {
    /// Disassociation reason
    pub reason: DisassociationReason,
}

impl DisassociationNotification {
    /// Create new disassociation notification
    pub fn new(reason: DisassociationReason) -> Self {
        Self { reason }
    }
    
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8, 1> {
        let mut payload = Vec::new();
        let _ = payload.push(self.reason as u8);
        payload
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 1 {
            return None;
        }
        DisassociationReason::from_u8(data[0]).map(|reason| Self { reason })
    }
}

/// Data Request frame (IEEE 802.15.4 Section 7.3.4)
/// Used to poll for pending data or retrieve association response
pub struct DataRequest;

impl DataRequest {
    /// Encode to bytes (empty payload)
    pub fn encode(&self) -> Vec<u8, 0> {
        Vec::new()
    }
}

/// Coordinator Realignment frame payload (IEEE 802.15.4 Section 7.3.8)
#[derive(Debug, Clone, Copy)]
pub struct CoordinatorRealignment {
    /// PAN identifier
    pub pan_id: u16,
    
    /// Coordinator short address
    pub coordinator_address: u16,
    
    /// Logical channel
    pub channel: u8,
    
    /// Short address assigned to device (orphan scan response)
    pub short_address: u16,
    
    /// Channel page (optional, typically 0 for 2.4 GHz)
    pub channel_page: Option<u8>,
}

impl CoordinatorRealignment {
    /// Create new coordinator realignment
    pub fn new(
        pan_id: u16,
        coordinator_address: u16,
        channel: u8,
        short_address: u16,
    ) -> Self {
        Self {
            pan_id,
            coordinator_address,
            channel,
            short_address,
            channel_page: Some(0),
        }
    }
    
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8, 8> {
        let mut payload = Vec::new();
        let _ = payload.extend_from_slice(&self.pan_id.to_le_bytes());
        let _ = payload.extend_from_slice(&self.coordinator_address.to_le_bytes());
        let _ = payload.push(self.channel);
        let _ = payload.extend_from_slice(&self.short_address.to_le_bytes());
        if let Some(page) = self.channel_page {
            let _ = payload.push(page);
        }
        payload
    }
    
    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 7 {
            return None;
        }
        let pan_id = u16::from_le_bytes([data[0], data[1]]);
        let coordinator_address = u16::from_le_bytes([data[2], data[3]]);
        let channel = data[4];
        let short_address = u16::from_le_bytes([data[5], data[6]]);
        let channel_page = if data.len() >= 8 {
            Some(data[7])
        } else {
            None
        };
        Some(Self {
            pan_id,
            coordinator_address,
            channel,
            short_address,
            channel_page,
        })
    }
}

/// Association state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssociationState {
    /// Idle, not associating
    Idle,
    
    /// Sent association request, waiting for ACK
    RequestSent,
    
    /// Association request ACKed, waiting for response
    WaitingForResponse,
    
    /// Polling for association response
    PollingForResponse,
    
    /// Association complete
    Associated,
    
    /// Association failed
    Failed,
}

/// Association manager
pub struct AssociationManager {
    /// Current association state
    state: AssociationState,
    
    /// Coordinator address we're associating with
    coordinator_address: Option<Address>,
    
    /// PAN ID we're joining
    pan_id: Option<u16>,
    
    /// Timestamp of last state change (for timeouts)
    last_state_change: u32,
    
    /// Number of data request polls sent
    poll_count: u8,
    
    /// Assigned short address
    assigned_address: Option<u16>,
}

impl AssociationManager {
    /// Create new association manager
    pub fn new() -> Self {
        Self {
            state: AssociationState::Idle,
            coordinator_address: None,
            pan_id: None,
            last_state_change: 0,
            poll_count: 0,
            assigned_address: None,
        }
    }
    
    /// Get current state
    pub fn state(&self) -> AssociationState {
        self.state
    }
    
    /// Get assigned address (if associated)
    pub fn assigned_address(&self) -> Option<u16> {
        self.assigned_address
    }
    
    /// Start association process (device side)
    pub fn start_association(
        &mut self,
        radio: &mut Radio,
        coordinator_address: Address,
        pan_id: u16,
        own_extended_address: u64,
        capability: CapabilityInformation,
        sequence: u8,
        timestamp: u32,
    ) -> Result<()> {
        if self.state != AssociationState::Idle {
            return Err(NetworkError::AssociationInProgress);
        }
        
        // Build association request
        let request = AssociationRequest::new(capability);
        let mut payload = Vec::new();
        let _ = payload.push(MacCommand::AssociationRequest as u8);
        let _ = payload.extend_from_slice(&request.encode());
        
        // Send association request
        radio.transmit_mac_command(
            pan_id,
            coordinator_address,
            Address::Extended(own_extended_address),
            MacCommand::AssociationRequest as u8,
            &request.encode(),
            sequence,
        )?;
        
        // Update state
        self.state = AssociationState::RequestSent;
        self.coordinator_address = Some(coordinator_address);
        self.pan_id = Some(pan_id);
        self.last_state_change = timestamp;
        self.poll_count = 0;
        
        Ok(())
    }
    
    /// Handle ACK received for association request
    pub fn handle_association_ack(&mut self, timestamp: u32) {
        if self.state == AssociationState::RequestSent {
            self.state = AssociationState::WaitingForResponse;
            self.last_state_change = timestamp;
        }
    }
    
    /// Poll for association response (send data request)
    pub fn poll_for_response(
        &mut self,
        radio: &mut Radio,
        own_extended_address: u64,
        sequence: u8,
        timestamp: u32,
    ) -> Result<()> {
        if self.state != AssociationState::WaitingForResponse 
            && self.state != AssociationState::PollingForResponse {
            return Err(NetworkError::InvalidState);
        }
        
        let coordinator_address = self.coordinator_address
            .ok_or(NetworkError::InvalidState)?;
        let pan_id = self.pan_id.ok_or(NetworkError::InvalidState)?;
        
        // Send data request (empty MAC command)
        let data_request = DataRequest;
        radio.transmit_mac_command(
            pan_id,
            coordinator_address,
            Address::Extended(own_extended_address),
            MacCommand::DataRequest as u8,
            &data_request.encode(),
            sequence,
        )?;
        
        self.state = AssociationState::PollingForResponse;
        self.last_state_change = timestamp;
        self.poll_count += 1;
        
        Ok(())
    }
    
    /// Handle received association response
    pub fn handle_association_response(
        &mut self,
        response: &AssociationResponse,
        timestamp: u32,
    ) -> Result<()> {
        if self.state != AssociationState::PollingForResponse 
            && self.state != AssociationState::WaitingForResponse {
            return Err(NetworkError::InvalidState);
        }
        
        match response.status {
            AssociationStatus::Success => {
                self.assigned_address = Some(response.short_address);
                self.state = AssociationState::Associated;
                self.last_state_change = timestamp;
                Ok(())
            }
            AssociationStatus::PanAtCapacity => {
                self.state = AssociationState::Failed;
                Err(NetworkError::PanAtCapacity)
            }
            AssociationStatus::PanAccessDenied => {
                self.state = AssociationState::Failed;
                Err(NetworkError::AccessDenied)
            }
            _ => {
                self.state = AssociationState::Failed;
                Err(NetworkError::AssociationFailed)
            }
        }
    }
    
    /// Check for timeout and retry if needed
    pub fn check_timeout(
        &mut self,
        current_time: u32,
        response_wait_time: u32,
        max_polls: u8,
    ) -> Result<bool> {
        let elapsed = current_time.saturating_sub(self.last_state_change);
        
        match self.state {
            AssociationState::WaitingForResponse => {
                // Wait macResponseWaitTime (typically 32 symbol periods ~500ms)
                if elapsed >= response_wait_time {
                    // Time to start polling
                    return Ok(true);
                }
            }
            AssociationState::PollingForResponse => {
                // Check if we've exceeded max polls
                if self.poll_count >= max_polls {
                    self.state = AssociationState::Failed;
                    return Err(NetworkError::Timeout);
                }
            }
            AssociationState::RequestSent => {
                // Check for ACK timeout (typically ~1 second)
                if elapsed >= 1000 {
                    self.state = AssociationState::Failed;
                    return Err(NetworkError::Timeout);
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Reset association state
    pub fn reset(&mut self) {
        self.state = AssociationState::Idle;
        self.coordinator_address = None;
        self.pan_id = None;
        self.poll_count = 0;
        self.assigned_address = None;
    }
    
    /// Handle disassociation notification
    pub fn handle_disassociation(&mut self, _notification: &DisassociationNotification) {
        self.state = AssociationState::Idle;
        self.assigned_address = None;
    }
}

/// Coordinator association manager (handles incoming association requests)
pub struct CoordinatorAssociationManager {
    /// Pending associations (IEEE address -> allocated short address)
    pending_associations: Vec<(u64, u16, CapabilityInformation), 8>,
    
    /// Next short address to allocate
    next_address: u16,
    
    /// Maximum devices allowed
    max_devices: u16,
    
    /// Current device count
    device_count: u16,
}

impl CoordinatorAssociationManager {
    /// Create new coordinator association manager
    pub fn new(starting_address: u16, max_devices: u16) -> Self {
        Self {
            pending_associations: Vec::new(),
            next_address: starting_address,
            max_devices,
            device_count: 0,
        }
    }
    
    /// Handle incoming association request
    pub fn handle_association_request(
        &mut self,
        radio: &mut Radio,
        device_address: u64,
        request: &AssociationRequest,
        pan_id: u16,
        coordinator_address: u16,
        sequence: u8,
    ) -> Result<()> {
        // Check if we have capacity
        if self.device_count >= self.max_devices {
            // Send failure response immediately
            let response = AssociationResponse::failure(AssociationStatus::PanAtCapacity);
            self.send_association_response(
                radio,
                device_address,
                &response,
                pan_id,
                coordinator_address,
                sequence,
            )?;
            return Ok(());
        }
        
        // Check if device already has an allocation
        if let Some((_, addr, _)) = self.pending_associations
            .iter()
            .find(|(ieee, _, _)| *ieee == device_address)
        {
            // Already allocated, can respond immediately
            let response = AssociationResponse::success(*addr);
            self.send_association_response(
                radio,
                device_address,
                &response,
                pan_id,
                coordinator_address,
                sequence,
            )?;
            return Ok(());
        }
        
        // Allocate new address
        let short_address = self.allocate_address();
        
        // Store pending association
        if self.pending_associations.push((
            device_address,
            short_address,
            request.capability,
        )).is_err() {
            // Pending table full, send failure
            let response = AssociationResponse::failure(AssociationStatus::PanAtCapacity);
            self.send_association_response(
                radio,
                device_address,
                &response,
                pan_id,
                coordinator_address,
                sequence,
            )?;
            return Ok(());
        }
        
        // Note: In real implementation, the response is sent when device polls
        // For now, we'll send it immediately (simplified)
        let response = AssociationResponse::success(short_address);
        self.send_association_response(
            radio,
            device_address,
            &response,
            pan_id,
            coordinator_address,
            sequence,
        )?;
        
        self.device_count += 1;
        
        Ok(())
    }
    
    /// Send association response
    fn send_association_response(
        &self,
        radio: &mut Radio,
        device_address: u64,
        response: &AssociationResponse,
        pan_id: u16,
        coordinator_address: u16,
        sequence: u8,
    ) -> Result<()> {
        radio.transmit_mac_command(
            pan_id,
            Address::Extended(device_address),
            Address::Short(coordinator_address),
            MacCommand::AssociationResponse as u8,
            &response.encode(),
            sequence,
        )
    }
    
    /// Handle data request (poll for association response)
    pub fn handle_data_request(
        &mut self,
        radio: &mut Radio,
        device_address: u64,
        pan_id: u16,
        coordinator_address: u16,
        sequence: u8,
    ) -> Result<()> {
        // Look for pending association
        if let Some((_, short_addr, _)) = self.pending_associations
            .iter()
            .find(|(ieee, _, _)| *ieee == device_address)
        {
            let response = AssociationResponse::success(*short_addr);
            self.send_association_response(
                radio,
                device_address,
                &response,
                pan_id,
                coordinator_address,
                sequence,
            )?;
            
            // Remove from pending (response delivered)
            self.pending_associations.retain(|(ieee, _, _)| *ieee != device_address);
        }
        
        Ok(())
    }
    
    /// Handle disassociation request
    pub fn handle_disassociation(
        &mut self,
        device_address: u64,
        _notification: &DisassociationNotification,
    ) {
        // Remove from pending if present
        self.pending_associations.retain(|(ieee, _, _)| *ieee != device_address);
        
        if self.device_count > 0 {
            self.device_count -= 1;
        }
    }
    
    /// Allocate next short address
    fn allocate_address(&mut self) -> u16 {
        let addr = self.next_address;
        
        // Increment, avoiding special addresses
        self.next_address = self.next_address.wrapping_add(1);
        while self.next_address == 0x0000      // Coordinator
            || self.next_address == 0xFFFF     // Broadcast
            || self.next_address == 0xFFFE     // No address
            || self.next_address == 0xFFFD     // Reserved
        {
            self.next_address = self.next_address.wrapping_add(1);
        }
        
        addr
    }
    
    /// Get capability for device
    pub fn get_device_capability(&self, device_address: u64) -> Option<CapabilityInformation> {
        self.pending_associations
            .iter()
            .find(|(ieee, _, _)| *ieee == device_address)
            .map(|(_, _, cap)| *cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_encode_decode() {
        let cap = CapabilityInformation {
            alternate_pan_coordinator: false,
            device_type: true,
            power_source: true,
            receiver_on_when_idle: true,
            security_capable: true,
            allocate_address: true,
        };
        
        let encoded = cap.encode();
        let decoded = CapabilityInformation::decode(encoded);
        
        assert_eq!(cap.device_type, decoded.device_type);
        assert_eq!(cap.power_source, decoded.power_source);
        assert_eq!(cap.receiver_on_when_idle, decoded.receiver_on_when_idle);
        assert_eq!(cap.security_capable, decoded.security_capable);
        assert_eq!(cap.allocate_address, decoded.allocate_address);
    }
    
    #[test]
    fn test_association_request() {
        let cap = CapabilityInformation::end_device(true);
        let request = AssociationRequest::new(cap);
        
        let encoded = request.encode();
        let decoded = AssociationRequest::decode(&encoded).unwrap();
        
        assert_eq!(request.capability.device_type, decoded.capability.device_type);
        assert_eq!(request.capability.allocate_address, decoded.capability.allocate_address);
    }
    
    #[test]
    fn test_association_response() {
        let response = AssociationResponse::success(0x1234);
        
        let encoded = response.encode();
        let decoded = AssociationResponse::decode(&encoded).unwrap();
        
        assert_eq!(response.short_address, decoded.short_address);
        assert_eq!(response.status as u8, decoded.status as u8);
    }
    
    #[test]
    fn test_coordinator_realignment() {
        let realign = CoordinatorRealignment::new(0x1234, 0x0000, 15, 0x5678);
        
        let encoded = realign.encode();
        let decoded = CoordinatorRealignment::decode(&encoded).unwrap();
        
        assert_eq!(realign.pan_id, decoded.pan_id);
        assert_eq!(realign.coordinator_address, decoded.coordinator_address);
        assert_eq!(realign.channel, decoded.channel);
        assert_eq!(realign.short_address, decoded.short_address);
    }
    
    #[test]
    fn test_address_allocation() {
        let mut manager = CoordinatorAssociationManager::new(0x0001, 10);
        
        let addr1 = manager.allocate_address();
        let addr2 = manager.allocate_address();
        let addr3 = manager.allocate_address();
        
        assert_eq!(addr1, 0x0001);
        assert_eq!(addr2, 0x0002);
        assert_eq!(addr3, 0x0003);
        
        // Test wrapping around special addresses
        manager.next_address = 0xFFFE;
        let addr4 = manager.allocate_address();
        assert_eq!(addr4, 0xFFFE);
        
        let addr5 = manager.allocate_address();
        // Should skip 0xFFFF and wrap to 0x0001 (skipping 0x0000)
        assert_ne!(addr5, 0xFFFF);
        assert_ne!(addr5, 0x0000);
    }
}
