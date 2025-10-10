//! APS (Application Support Sublayer) Layer
//!
//! The APS layer provides services to the application and network layers:
//! - Data transfer services (unicast, broadcast, group)
//! - Binding management
//! - Group management
//! - Fragmentation and reassembly
//! - Acknowledgment and retry
//! - Duplicate rejection

use crate::zigbee::{
    network::{NetworkAddress, NetworkInfo},
    NetworkError, Result,
};

/// Maximum APS payload size (before fragmentation)
pub const APS_MAX_PAYLOAD: usize = 82;

/// Maximum number of fragments per message
pub const APS_MAX_FRAGMENTS: usize = 16;

/// APS frame counter for security
pub type ApsCounter = u32;

/// APS Frame Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApsFrameType {
    /// Data frame
    Data = 0,
    /// Command frame
    Command = 1,
    /// Acknowledgment frame
    Acknowledgment = 2,
}

/// APS Delivery Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApsDeliveryMode {
    /// Normal unicast delivery
    Unicast = 0,
    /// Broadcast to all devices
    Broadcast = 2,
    /// Group addressing
    Group = 3,
}

/// APS Command Identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApsCommandId {
    /// SKKE-1 (Symmetric Key Key Establishment)
    SkkeCommand1 = 0x01,
    /// SKKE-2
    SkkeCommand2 = 0x02,
    /// SKKE-3
    SkkeCommand3 = 0x03,
    /// SKKE-4
    SkkeCommand4 = 0x04,
    /// Transport Key
    TransportKey = 0x05,
    /// Update Device
    UpdateDevice = 0x06,
    /// Remove Device
    RemoveDevice = 0x07,
    /// Request Key
    RequestKey = 0x08,
    /// Switch Key
    SwitchKey = 0x09,
    /// Tunnel
    Tunnel = 0x0E,
    /// Verify Key
    VerifyKey = 0x0F,
    /// Confirm Key
    ConfirmKey = 0x10,
}

/// APS Frame Control Field
#[derive(Debug, Clone, Copy)]
pub struct ApsFrameControl {
    /// Frame type
    pub frame_type: ApsFrameType,
    /// Delivery mode
    pub delivery_mode: ApsDeliveryMode,
    /// Acknowledgment format (0 = data, 1 = command)
    pub ack_format: bool,
    /// Security enabled
    pub security: bool,
    /// Acknowledgment request
    pub ack_request: bool,
    /// Extended header present
    pub extended_header: bool,
}

impl ApsFrameControl {
    /// Create a new frame control field
    pub fn new(frame_type: ApsFrameType, delivery_mode: ApsDeliveryMode) -> Self {
        Self {
            frame_type,
            delivery_mode,
            ack_format: false,
            security: false,
            ack_request: false,
            extended_header: false,
        }
    }

    /// Encode to byte
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        byte |= (self.frame_type as u8) & 0x03;
        byte |= ((self.delivery_mode as u8) & 0x03) << 2;
        if self.ack_format {
            byte |= 1 << 4;
        }
        if self.security {
            byte |= 1 << 5;
        }
        if self.ack_request {
            byte |= 1 << 6;
        }
        if self.extended_header {
            byte |= 1 << 7;
        }
        byte
    }

    /// Decode from byte
    pub fn from_byte(byte: u8) -> Self {
        let frame_type = match byte & 0x03 {
            0 => ApsFrameType::Data,
            1 => ApsFrameType::Command,
            2 => ApsFrameType::Acknowledgment,
            _ => ApsFrameType::Data,
        };

        let delivery_mode = match (byte >> 2) & 0x03 {
            0 => ApsDeliveryMode::Unicast,
            2 => ApsDeliveryMode::Broadcast,
            3 => ApsDeliveryMode::Group,
            _ => ApsDeliveryMode::Unicast,
        };

        Self {
            frame_type,
            delivery_mode,
            ack_format: (byte & (1 << 4)) != 0,
            security: (byte & (1 << 5)) != 0,
            ack_request: (byte & (1 << 6)) != 0,
            extended_header: (byte & (1 << 7)) != 0,
        }
    }
}

/// APS Extended Header (for fragmentation)
#[derive(Debug, Clone, Copy)]
pub struct ApsExtendedHeader {
    /// Fragment number (0-15)
    pub fragment_number: u8,
    /// Total fragments (0-15)
    pub fragment_count: u8,
    /// Block number for large messages
    pub block_number: u8,
}

impl ApsExtendedHeader {
    /// Encode to bytes
    pub fn to_bytes(&self) -> [u8; 2] {
        let byte0 = ((self.fragment_count & 0x0F) << 4) | (self.fragment_number & 0x0F);
        let byte1 = self.block_number;
        [byte0, byte1]
    }

    /// Decode from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 2 {
            return None;
        }
        Some(Self {
            fragment_number: bytes[0] & 0x0F,
            fragment_count: (bytes[0] >> 4) & 0x0F,
            block_number: bytes[1],
        })
    }

    /// Check if this is the first fragment
    pub fn is_first(&self) -> bool {
        self.fragment_number == 0
    }

    /// Check if this is the last fragment
    pub fn is_last(&self) -> bool {
        self.fragment_number == self.fragment_count - 1
    }
}

/// APS Data Frame
#[derive(Debug, Clone)]
pub struct ApsDataFrame {
    /// Frame control
    pub frame_control: ApsFrameControl,
    /// Destination endpoint (0-240)
    pub dst_endpoint: u8,
    /// Group address (if group delivery)
    pub group_address: Option<u16>,
    /// Cluster identifier
    pub cluster_id: u16,
    /// Profile identifier
    pub profile_id: u16,
    /// Source endpoint (0-240)
    pub src_endpoint: u8,
    /// APS counter
    pub aps_counter: u8,
    /// Extended header (for fragmentation)
    pub extended_header: Option<ApsExtendedHeader>,
    /// Payload data
    pub payload: heapless::Vec<u8, 128>,
}

impl ApsDataFrame {
    /// Create a new APS data frame
    pub fn new(
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        payload: &[u8],
    ) -> Result<Self> {
        if payload.len() > 128 {
            return Err(NetworkError::InvalidParameter);
        }

        let mut payload_vec = heapless::Vec::new();
        payload_vec.extend_from_slice(payload).map_err(|_| NetworkError::InvalidParameter)?;

        Ok(Self {
            frame_control: ApsFrameControl::new(ApsFrameType::Data, ApsDeliveryMode::Unicast),
            dst_endpoint,
            group_address: None,
            cluster_id,
            profile_id,
            src_endpoint,
            aps_counter: 0,
            extended_header: None,
            payload: payload_vec,
        })
    }

    /// Create a broadcast frame
    pub fn new_broadcast(
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        payload: &[u8],
    ) -> Result<Self> {
        let mut frame = Self::new(dst_endpoint, src_endpoint, cluster_id, profile_id, payload)?;
        frame.frame_control.delivery_mode = ApsDeliveryMode::Broadcast;
        Ok(frame)
    }

    /// Create a group frame
    pub fn new_group(
        group_address: u16,
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        payload: &[u8],
    ) -> Result<Self> {
        let mut frame = Self::new(dst_endpoint, src_endpoint, cluster_id, profile_id, payload)?;
        frame.frame_control.delivery_mode = ApsDeliveryMode::Group;
        frame.group_address = Some(group_address);
        Ok(frame)
    }

    /// Enable acknowledgment request
    pub fn with_ack_request(mut self) -> Self {
        self.frame_control.ack_request = true;
        self
    }

    /// Enable security
    pub fn with_security(mut self) -> Self {
        self.frame_control.security = true;
        self
    }

    /// Encode frame to bytes
    pub fn encode(&self) -> Result<heapless::Vec<u8, 256>> {
        let mut buffer = heapless::Vec::new();

        // Frame control
        buffer.push(self.frame_control.to_byte()).map_err(|_| NetworkError::InvalidParameter)?;

        // Destination endpoint
        buffer.push(self.dst_endpoint).map_err(|_| NetworkError::InvalidParameter)?;

        // Group address (if group delivery)
        if let Some(group_addr) = self.group_address {
            buffer.extend_from_slice(&group_addr.to_le_bytes())
                .map_err(|_| NetworkError::InvalidParameter)?;
        }

        // Cluster ID
        buffer.extend_from_slice(&self.cluster_id.to_le_bytes())
            .map_err(|_| NetworkError::InvalidParameter)?;

        // Profile ID
        buffer.extend_from_slice(&self.profile_id.to_le_bytes())
            .map_err(|_| NetworkError::InvalidParameter)?;

        // Source endpoint
        buffer.push(self.src_endpoint).map_err(|_| NetworkError::InvalidParameter)?;

        // APS counter
        buffer.push(self.aps_counter).map_err(|_| NetworkError::InvalidParameter)?;

        // Extended header (if present)
        if let Some(ext_hdr) = &self.extended_header {
            let ext_bytes = ext_hdr.to_bytes();
            buffer.extend_from_slice(&ext_bytes).map_err(|_| NetworkError::InvalidParameter)?;
        }

        // Payload
        buffer.extend_from_slice(&self.payload).map_err(|_| NetworkError::InvalidParameter)?;

        Ok(buffer)
    }

    /// Decode frame from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidParameter);
        }

        let mut offset = 0;

        // Frame control
        let frame_control = ApsFrameControl::from_byte(data[offset]);
        offset += 1;

        // Destination endpoint
        let dst_endpoint = data[offset];
        offset += 1;

        // Group address (if group delivery)
        let group_address = if frame_control.delivery_mode == ApsDeliveryMode::Group {
            if data.len() < offset + 2 {
                return Err(NetworkError::InvalidParameter);
            }
            let addr = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            Some(addr)
        } else {
            None
        };

        // Cluster ID
        if data.len() < offset + 2 {
            return Err(NetworkError::InvalidParameter);
        }
        let cluster_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Profile ID
        if data.len() < offset + 2 {
            return Err(NetworkError::InvalidParameter);
        }
        let profile_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Source endpoint
        let src_endpoint = data[offset];
        offset += 1;

        // APS counter
        let aps_counter = data[offset];
        offset += 1;

        // Extended header (if present)
        let extended_header = if frame_control.extended_header {
            if data.len() < offset + 2 {
                return Err(NetworkError::InvalidParameter);
            }
            let ext_hdr = ApsExtendedHeader::from_bytes(&data[offset..offset + 2])
                .ok_or(NetworkError::InvalidParameter)?;
            offset += 2;
            Some(ext_hdr)
        } else {
            None
        };

        // Payload
        let mut payload = heapless::Vec::new();
        if offset < data.len() {
            payload.extend_from_slice(&data[offset..])
                .map_err(|_| NetworkError::InvalidParameter)?;
        }

        Ok(Self {
            frame_control,
            dst_endpoint,
            group_address,
            cluster_id,
            profile_id,
            src_endpoint,
            aps_counter,
            extended_header,
            payload,
        })
    }
}

/// APS Acknowledgment Frame
#[derive(Debug, Clone)]
pub struct ApsAckFrame {
    /// Frame control
    pub frame_control: ApsFrameControl,
    /// Destination endpoint
    pub dst_endpoint: u8,
    /// Cluster identifier
    pub cluster_id: u16,
    /// Profile identifier
    pub profile_id: u16,
    /// Source endpoint
    pub src_endpoint: u8,
    /// APS counter being acknowledged
    pub aps_counter: u8,
}

impl ApsAckFrame {
    /// Create a new APS acknowledgment frame
    pub fn new(
        dst_endpoint: u8,
        src_endpoint: u8,
        cluster_id: u16,
        profile_id: u16,
        aps_counter: u8,
    ) -> Self {
        Self {
            frame_control: ApsFrameControl::new(
                ApsFrameType::Acknowledgment,
                ApsDeliveryMode::Unicast,
            ),
            dst_endpoint,
            cluster_id,
            profile_id,
            src_endpoint,
            aps_counter,
        }
    }

    /// Encode frame to bytes
    pub fn encode(&self) -> Result<heapless::Vec<u8, 64>> {
        let mut buffer = heapless::Vec::new();

        buffer.push(self.frame_control.to_byte()).map_err(|_| NetworkError::InvalidParameter)?;
        buffer.push(self.dst_endpoint).map_err(|_| NetworkError::InvalidParameter)?;
        buffer.extend_from_slice(&self.cluster_id.to_le_bytes())
            .map_err(|_| NetworkError::InvalidParameter)?;
        buffer.extend_from_slice(&self.profile_id.to_le_bytes())
            .map_err(|_| NetworkError::InvalidParameter)?;
        buffer.push(self.src_endpoint).map_err(|_| NetworkError::InvalidParameter)?;
        buffer.push(self.aps_counter).map_err(|_| NetworkError::InvalidParameter)?;

        Ok(buffer)
    }
}

/// Binding entry for APS
#[derive(Debug, Clone)]
pub struct ApsBinding {
    /// Source endpoint
    pub src_endpoint: u8,
    /// Cluster ID
    pub cluster_id: u16,
    /// Destination address type
    pub dst_addr_mode: u8,
    /// Destination address (short or extended)
    pub dst_address: u64,
    /// Destination endpoint
    pub dst_endpoint: u8,
}

impl ApsBinding {
    /// Create a new binding entry
    pub fn new(
        src_endpoint: u8,
        cluster_id: u16,
        dst_address: u64,
        dst_endpoint: u8,
    ) -> Self {
        Self {
            src_endpoint,
            cluster_id,
            dst_addr_mode: 3, // 64-bit extended address
            dst_address,
            dst_endpoint,
        }
    }
}

/// Group membership entry
#[derive(Debug, Clone, Copy)]
pub struct ApsGroupMembership {
    /// Group address
    pub group_address: u16,
    /// Endpoint
    pub endpoint: u8,
}

/// Fragment reassembly state
#[derive(Debug, Clone)]
struct FragmentState {
    /// Source address
    src_address: u16,
    /// APS counter
    aps_counter: u8,
    /// Total fragments expected
    fragment_count: u8,
    /// Received fragments bitmap
    received_mask: u16,
    /// Reassembled payload
    payload: heapless::Vec<u8, 256>,
    /// Timestamp for timeout
    timestamp: u32,
}

/// APS Layer Manager
pub struct ApsManager {
    /// APS counter for outgoing frames
    aps_counter: u8,
    /// Binding table
    bindings: heapless::Vec<ApsBinding, 16>,
    /// Group memberships
    groups: heapless::Vec<ApsGroupMembership, 16>,
    /// Fragment reassembly table
    fragments: heapless::Vec<FragmentState, 4>,
    /// Pending acknowledgments
    pending_acks: heapless::Vec<(u16, u8), 8>, // (dst_addr, aps_counter)
}

impl ApsManager {
    /// Create a new APS manager
    pub fn new() -> Self {
        Self {
            aps_counter: 0,
            bindings: heapless::Vec::new(),
            groups: heapless::Vec::new(),
            fragments: heapless::Vec::new(),
            pending_acks: heapless::Vec::new(),
        }
    }

    /// Get next APS counter
    pub fn next_counter(&mut self) -> u8 {
        let counter = self.aps_counter;
        self.aps_counter = self.aps_counter.wrapping_add(1);
        counter
    }

    /// Add a binding entry
    pub fn add_binding(&mut self, binding: ApsBinding) -> Result<()> {
        // Check if binding already exists
        for b in &self.bindings {
            if b.src_endpoint == binding.src_endpoint
                && b.cluster_id == binding.cluster_id
                && b.dst_address == binding.dst_address
                && b.dst_endpoint == binding.dst_endpoint
            {
                return Ok(()); // Already bound
            }
        }

        self.bindings
            .push(binding)
            .map_err(|_| NetworkError::BindingFailed)
    }

    /// Remove a binding entry
    pub fn remove_binding(
        &mut self,
        src_endpoint: u8,
        cluster_id: u16,
        dst_address: u64,
        dst_endpoint: u8,
    ) -> Result<()> {
        self.bindings.retain(|b| {
            !(b.src_endpoint == src_endpoint
                && b.cluster_id == cluster_id
                && b.dst_address == dst_address
                && b.dst_endpoint == dst_endpoint)
        });
        Ok(())
    }

    /// Get bindings for an endpoint and cluster
    pub fn get_bindings(&self, src_endpoint: u8, cluster_id: u16) -> heapless::Vec<&ApsBinding, 16> {
        let mut result = heapless::Vec::new();
        for binding in &self.bindings {
            if binding.src_endpoint == src_endpoint && binding.cluster_id == cluster_id {
                result.push(binding).ok();
            }
        }
        result
    }
    
    /// Get all bindings
    pub fn get_all_bindings(&self) -> &heapless::Vec<ApsBinding, 16> {
        &self.bindings
    }
    
    /// Get all groups
    pub fn get_all_groups(&self) -> &heapless::Vec<ApsGroupMembership, 16> {
        &self.groups
    }

    /// Add group membership
    pub fn add_group(&mut self, group_address: u16, endpoint: u8) -> Result<()> {
        // Check if already a member
        for g in &self.groups {
            if g.group_address == group_address && g.endpoint == endpoint {
                return Ok(()); // Already member
            }
        }

        self.groups
            .push(ApsGroupMembership {
                group_address,
                endpoint,
            })
            .map_err(|_| NetworkError::InvalidParameter)
    }

    /// Remove group membership
    pub fn remove_group(&mut self, group_address: u16, endpoint: u8) -> Result<()> {
        self.groups.retain(|g| {
            !(g.group_address == group_address && g.endpoint == endpoint)
        });
        Ok(())
    }

    /// Check if endpoint is member of group
    pub fn is_group_member(&self, group_address: u16, endpoint: u8) -> bool {
        self.groups.iter().any(|g| {
            g.group_address == group_address && g.endpoint == endpoint
        })
    }

    /// Fragment a large payload
    pub fn fragment_payload(
        &self,
        payload: &[u8],
        max_fragment_size: usize,
    ) -> Result<heapless::Vec<heapless::Vec<u8, 128>, 16>> {
        if payload.len() <= max_fragment_size {
            // No fragmentation needed
            let mut result = heapless::Vec::new();
            let mut fragment = heapless::Vec::new();
            fragment.extend_from_slice(payload).map_err(|_| NetworkError::InvalidParameter)?;
            result.push(fragment).map_err(|_| NetworkError::InvalidParameter)?;
            return Ok(result);
        }

        let fragment_count = (payload.len() + max_fragment_size - 1) / max_fragment_size;
        if fragment_count > APS_MAX_FRAGMENTS {
            return Err(NetworkError::InvalidParameter);
        }

        let mut fragments = heapless::Vec::new();
        for i in 0..fragment_count {
            let start = i * max_fragment_size;
            let end = core::cmp::min(start + max_fragment_size, payload.len());
            let mut fragment = heapless::Vec::new();
            fragment.extend_from_slice(&payload[start..end])
                .map_err(|_| NetworkError::InvalidParameter)?;
            fragments.push(fragment).map_err(|_| NetworkError::InvalidParameter)?;
        }

        Ok(fragments)
    }

    /// Process received fragment and reassemble if complete
    pub fn process_fragment(
        &mut self,
        src_address: u16,
        frame: &ApsDataFrame,
        timestamp: u32,
    ) -> Result<Option<heapless::Vec<u8, 256>>> {
        let ext_hdr = frame.extended_header.ok_or(NetworkError::InvalidParameter)?;

        // Find or create fragment state
        let state_index = self.fragments.iter().position(|s| {
            s.src_address == src_address && s.aps_counter == frame.aps_counter
        });

        let state = if let Some(idx) = state_index {
            &mut self.fragments[idx]
        } else {
            // Create new fragment state
            let new_state = FragmentState {
                src_address,
                aps_counter: frame.aps_counter,
                fragment_count: ext_hdr.fragment_count,
                received_mask: 0,
                payload: heapless::Vec::new(),
                timestamp,
            };
            self.fragments.push(new_state).map_err(|_| NetworkError::InvalidParameter)?;
            self.fragments.last_mut().unwrap()
        };

        // Mark fragment as received
        if ext_hdr.fragment_number < 16 {
            state.received_mask |= 1 << ext_hdr.fragment_number;
        }

        // Append payload
        state.payload.extend_from_slice(&frame.payload)
            .map_err(|_| NetworkError::InvalidParameter)?;

        // Check if all fragments received
        let expected_mask = (1u16 << state.fragment_count) - 1;
        if state.received_mask == expected_mask {
            // Reassembly complete
            let payload = state.payload.clone();
            // Remove state
            if let Some(idx) = state_index {
                self.fragments.swap_remove(idx);
            }
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }

    /// Cleanup expired fragment states
    pub fn cleanup_fragments(&mut self, current_timestamp: u32, timeout_ms: u32) {
        self.fragments.retain(|s| {
            current_timestamp.wrapping_sub(s.timestamp) < timeout_ms
        });
    }

    /// Add pending acknowledgment
    pub fn add_pending_ack(&mut self, dst_addr: u16, aps_counter: u8) -> Result<()> {
        self.pending_acks
            .push((dst_addr, aps_counter))
            .map_err(|_| NetworkError::InvalidParameter)
    }

    /// Check if acknowledgment is pending
    pub fn is_ack_pending(&self, dst_addr: u16, aps_counter: u8) -> bool {
        self.pending_acks.iter().any(|(addr, counter)| {
            *addr == dst_addr && *counter == aps_counter
        })
    }

    /// Remove pending acknowledgment
    pub fn remove_pending_ack(&mut self, dst_addr: u16, aps_counter: u8) {
        self.pending_acks.retain(|(addr, counter)| {
            !(*addr == dst_addr && *counter == aps_counter)
        });
    }
}

impl Default for ApsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_control_encode_decode() {
        let fc = ApsFrameControl::new(ApsFrameType::Data, ApsDeliveryMode::Unicast);
        let byte = fc.to_byte();
        let decoded = ApsFrameControl::from_byte(byte);
        
        assert_eq!(fc.frame_type, decoded.frame_type);
        assert_eq!(fc.delivery_mode, decoded.delivery_mode);
    }

    #[test]
    fn test_data_frame_encode_decode() {
        let payload = b"Hello, Zigbee!";
        let frame = ApsDataFrame::new(1, 1, 0x0006, 0x0104, payload).unwrap();
        
        let encoded = frame.encode().unwrap();
        let decoded = ApsDataFrame::decode(&encoded).unwrap();
        
        assert_eq!(frame.dst_endpoint, decoded.dst_endpoint);
        assert_eq!(frame.src_endpoint, decoded.src_endpoint);
        assert_eq!(frame.cluster_id, decoded.cluster_id);
        assert_eq!(frame.profile_id, decoded.profile_id);
        assert_eq!(frame.payload.as_slice(), decoded.payload.as_slice());
    }

    #[test]
    fn test_fragmentation() {
        let manager = ApsManager::new();
        let payload = [0u8; 200];
        let fragments = manager.fragment_payload(&payload, 82).unwrap();
        
        assert_eq!(fragments.len(), 3);
        assert_eq!(fragments[0].len(), 82);
        assert_eq!(fragments[1].len(), 82);
        assert_eq!(fragments[2].len(), 36);
    }

    #[test]
    fn test_binding_management() {
        let mut manager = ApsManager::new();
        
        let binding = ApsBinding::new(1, 0x0006, 0x1122334455667788, 1);
        manager.add_binding(binding.clone()).unwrap();
        
        let bindings = manager.get_bindings(1, 0x0006);
        assert_eq!(bindings.len(), 1);
        
        manager.remove_binding(1, 0x0006, 0x1122334455667788, 1).unwrap();
        let bindings = manager.get_bindings(1, 0x0006);
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_group_management() {
        let mut manager = ApsManager::new();
        
        manager.add_group(0x0001, 1).unwrap();
        assert!(manager.is_group_member(0x0001, 1));
        assert!(!manager.is_group_member(0x0002, 1));
        
        manager.remove_group(0x0001, 1).unwrap();
        assert!(!manager.is_group_member(0x0001, 1));
    }
}
