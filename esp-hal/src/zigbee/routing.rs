//! Routing Manager - AODV-based Zigbee Routing
//!
//! Implements Ad-hoc On-Demand Distance Vector (AODV) routing
//! for Zigbee networks according to Zigbee Specification R22.

use super::nwk::*;
use heapless::Vec;

/// Routing manager
pub struct RoutingManager {
    /// Routing table
    routing_table: RoutingTable,
    
    /// Route discovery table
    discovery_table: RouteDiscoveryTable,
    
    /// Route request ID counter
    rreq_id: u8,
    
    /// Network address of this device
    network_address: u16,
    
    /// IEEE address of this device
    ieee_address: u64,
    
    /// Route discovery in progress
    discovery_in_progress: Vec<(u16, u8), 8>, // (destination, rreq_id)
    
    /// Enable many-to-one routing
    many_to_one_enabled: bool,
    
    /// Concentrator discovery time (seconds)
    concentrator_discovery_time: u32,
}

impl RoutingManager {
    /// Create a new routing manager
    pub fn new(network_address: u16, ieee_address: u64) -> Self {
        Self {
            routing_table: RoutingTable::new(),
            discovery_table: RouteDiscoveryTable::new(),
            rreq_id: 0,
            network_address,
            ieee_address,
            discovery_in_progress: Vec::new(),
            many_to_one_enabled: false,
            concentrator_discovery_time: 0,
        }
    }
    
    /// Get next route request ID
    fn next_rreq_id(&mut self) -> u8 {
        let id = self.rreq_id;
        self.rreq_id = self.rreq_id.wrapping_add(1);
        id
    }
    
    /// Find route to destination
    pub fn find_route(&self, destination: u16) -> Option<&RoutingTableEntry> {
        self.routing_table.find_route(destination)
    }
    
    /// Add route
    pub fn add_route(&mut self, destination: u16, next_hop: u16, cost: u8) -> Result<(), NwkError> {
        self.routing_table.add_route(destination, next_hop, cost)
    }
    
    /// Remove route
    pub fn remove_route(&mut self, destination: u16) {
        self.routing_table.remove_route(destination);
    }
    
    /// Mark route as failed
    pub fn mark_route_failed(&mut self, destination: u16) {
        self.routing_table.mark_route_failed(destination);
    }
    
    /// Initiate route discovery
    pub fn discover_route(&mut self, destination: u16) -> Result<RouteRequest, NwkError> {
        // Check if discovery already in progress
        if self.discovery_in_progress.iter().any(|(dst, _)| *dst == destination) {
            return Err(NwkError::Timeout); // Discovery already in progress
        }
        
        let rreq_id = self.next_rreq_id();
        
        // Add to discovery in progress
        self.discovery_in_progress
            .push((destination, rreq_id))
            .map_err(|_| NwkError::TableFull)?;
        
        // Create route request
        let mut rreq = RouteRequest::new(rreq_id, destination);
        rreq.options.many_to_one = if self.many_to_one_enabled { 2 } else { 0 };
        
        Ok(rreq)
    }
    
    /// Process incoming route request
    pub fn process_route_request(
        &mut self,
        rreq: &RouteRequest,
        sender_address: u16,
        link_cost: u8,
    ) -> RouteRequestAction {
        // Check if this is a duplicate request
        if self.discovery_table
            .find_entry(rreq.identifier, sender_address)
            .is_some()
        {
            return RouteRequestAction::Drop; // Duplicate
        }
        
        // Calculate path cost
        let path_cost = rreq.path_cost.saturating_add(link_cost);
        
        // Add to discovery table
        let entry = RouteDiscoveryEntry {
            request_id: rreq.identifier,
            source_address: sender_address,
            sender_address,
            forward_cost: link_cost,
            residual_cost: path_cost,
            timestamp: 0,
        };
        self.discovery_table.add_entry(entry).ok();
        
        // Add reverse route to originator
        self.add_route(sender_address, sender_address, link_cost).ok();
        
        // Check if we are the destination
        if rreq.destination_address == self.network_address {
            // We are the destination, send route reply
            let rrep = RouteReply::new(
                rreq.identifier,
                sender_address, // originator
                self.network_address, // responder
                path_cost,
            );
            
            return RouteRequestAction::SendReply(rrep);
        }
        
        // Check if we have a route to destination
        if let Some(route) = self.find_route(rreq.destination_address) {
            // We have a route, send route reply
            let total_cost = path_cost.saturating_add(route.cost);
            let rrep = RouteReply::new(
                rreq.identifier,
                sender_address, // originator
                self.network_address, // responder (intermediate)
                total_cost,
            );
            
            return RouteRequestAction::SendReply(rrep);
        }
        
        // Forward route request
        let mut forward_rreq = rreq.clone();
        forward_rreq.path_cost = path_cost;
        RouteRequestAction::Forward(forward_rreq)
    }
    
    /// Process incoming route reply
    pub fn process_route_reply(
        &mut self,
        rrep: &RouteReply,
        sender_address: u16,
        link_cost: u8,
    ) -> RouteReplyAction {
        // Add route to responder through sender
        self.add_route(rrep.responder_address, sender_address, link_cost).ok();
        
        // Calculate total path cost
        let path_cost = rrep.path_cost.saturating_add(link_cost);
        
        // Check if we are the originator
        if rrep.originator_address == self.network_address {
            // We are the originator, route discovery complete
            self.discovery_in_progress
                .retain(|(_, id)| *id != rrep.identifier);
            
            return RouteReplyAction::Complete;
        }
        
        // Find route to originator
        if let Some(route) = self.find_route(rrep.originator_address) {
            // Forward route reply to originator
            let mut forward_rrep = rrep.clone();
            forward_rrep.path_cost = path_cost;
            
            return RouteReplyAction::Forward(forward_rrep, route.next_hop);
        }
        
        // No route to originator, drop
        RouteReplyAction::Drop
    }
    
    /// Process network status
    pub fn process_network_status(&mut self, status: &NetworkStatus) {
        match status.status_code {
            NetworkStatusCode::NoRouteAvailable
            | NetworkStatusCode::TreeLinkFailure
            | NetworkStatusCode::NonTreeLinkFailure => {
                // Mark route as failed
                self.mark_route_failed(status.destination_address);
            }
            NetworkStatusCode::SourceRoutingFailure => {
                // Remove source route
                self.remove_route(status.destination_address);
            }
            _ => {}
        }
    }
    
    /// Age routing tables
    pub fn age_tables(&mut self, seconds: u32) {
        self.routing_table.age_routes(seconds);
        self.discovery_table.age_entries(seconds);
    }
    
    /// Enable many-to-one routing
    pub fn enable_many_to_one(&mut self, enable: bool) {
        self.many_to_one_enabled = enable;
    }
    
    /// Get routing table
    pub fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }
    
    /// Get route count
    pub fn route_count(&self) -> usize {
        self.routing_table.count()
    }
    
    /// Clear all routes
    pub fn clear_routes(&mut self) {
        self.routing_table.clear();
        self.discovery_table.clear();
        self.discovery_in_progress.clear();
    }
}

/// Action to take after processing route request
#[derive(Debug)]
pub enum RouteRequestAction {
    /// Drop the request (duplicate or invalid)
    Drop,
    
    /// Send route reply
    SendReply(RouteReply),
    
    /// Forward route request
    Forward(RouteRequest),
}

/// Action to take after processing route reply
#[derive(Debug)]
pub enum RouteReplyAction {
    /// Drop the reply
    Drop,
    
    /// Route discovery complete
    Complete,
    
    /// Forward route reply to next hop
    Forward(RouteReply, u16), // (reply, next_hop)
}

/// Address allocation manager
pub struct AddressManager {
    /// Network address of this device
    network_address: u16,
    
    /// Address pool for allocation
    address_pool_start: u16,
    address_pool_end: u16,
    
    /// Next address to allocate
    next_address: u16,
    
    /// Allocated addresses
    allocated: Vec<(u16, u64), 64>, // (short_address, ieee_address)
    
    /// Maximum depth
    max_depth: u8,
    
    /// Maximum children
    max_children: u8,
    
    /// Maximum routers
    max_routers: u8,
    
    /// Current depth
    current_depth: u8,
}

impl AddressManager {
    /// Create new address manager
    pub fn new(
        network_address: u16,
        depth: u8,
        max_depth: u8,
        max_children: u8,
        max_routers: u8,
    ) -> Self {
        // Calculate address pool based on Cskip algorithm
        let (pool_start, pool_end) = Self::calculate_address_pool(
            network_address,
            depth,
            max_depth,
            max_children,
            max_routers,
        );
        
        Self {
            network_address,
            address_pool_start: pool_start,
            address_pool_end: pool_end,
            next_address: pool_start,
            allocated: Vec::new(),
            max_depth,
            max_children,
            max_routers,
            current_depth: depth,
        }
    }
    
    /// Calculate address pool using Cskip algorithm
    fn calculate_address_pool(
        parent_address: u16,
        depth: u8,
        max_depth: u8,
        max_children: u8,
        max_routers: u8,
    ) -> (u16, u16) {
        if depth >= max_depth {
            return (parent_address + 1, parent_address + 1);
        }
        
        // Simplified Cskip calculation
        let cskip = if depth < max_depth - 1 {
            1 + max_children as u16 * (max_depth - depth - 1) as u16
        } else {
            1
        };
        
        let pool_start = parent_address + 1;
        let pool_size = max_children as u16 * cskip;
        let pool_end = pool_start + pool_size;
        
        (pool_start, pool_end.min(0xFFF7)) // Max address is 0xFFF7
    }
    
    /// Allocate address for a device
    pub fn allocate_address(&mut self, ieee_address: u64, is_router: bool) -> Result<u16, NwkError> {
        // Check if already allocated
        if let Some((addr, _)) = self.allocated.iter().find(|(_, ieee)| *ieee == ieee_address) {
            return Ok(*addr);
        }
        
        // Check capacity
        if self.allocated.len() >= self.max_children as usize {
            return Err(NwkError::TableFull);
        }
        
        // Check router capacity
        let router_count = self.allocated.iter().filter(|(addr, _)| {
            // In a real implementation, we would track device types
            // For now, assume addresses in certain range are routers
            *addr < self.network_address + (self.max_routers as u16 * 100)
        }).count();
        
        if is_router && router_count >= self.max_routers as usize {
            return Err(NwkError::TableFull);
        }
        
        // Allocate next available address
        if self.next_address > self.address_pool_end {
            return Err(NwkError::TableFull);
        }
        
        let address = self.next_address;
        self.next_address += 1;
        
        // Record allocation
        self.allocated
            .push((address, ieee_address))
            .map_err(|_| NwkError::TableFull)?;
        
        Ok(address)
    }
    
    /// Free an address
    pub fn free_address(&mut self, address: u16) {
        self.allocated.retain(|(addr, _)| *addr != address);
    }
    
    /// Find IEEE address by short address
    pub fn find_ieee_address(&self, short_address: u16) -> Option<u64> {
        self.allocated
            .iter()
            .find(|(addr, _)| *addr == short_address)
            .map(|(_, ieee)| *ieee)
    }
    
    /// Find short address by IEEE address
    pub fn find_short_address(&self, ieee_address: u64) -> Option<u16> {
        self.allocated
            .iter()
            .find(|(_, ieee)| *ieee == ieee_address)
            .map(|(addr, _)| *addr)
    }
    
    /// Get allocated address count
    pub fn allocated_count(&self) -> usize {
        self.allocated.len()
    }
    
    /// Get available address count
    pub fn available_count(&self) -> usize {
        let total = (self.address_pool_end - self.address_pool_start + 1) as usize;
        total.saturating_sub(self.allocated.len())
    }
    
    /// Check if address is in our pool
    pub fn is_in_pool(&self, address: u16) -> bool {
        address >= self.address_pool_start && address <= self.address_pool_end
    }
}

/// Network formation manager
pub struct NetworkFormation {
    /// Network configuration
    pan_id: u16,
    extended_pan_id: u64,
    channel: u8,
    
    /// Permit joining
    permit_joining: bool,
    permit_duration: u32, // seconds, 0 = off, 0xFF = always on
    
    /// Network depth
    depth: u8,
    max_depth: u8,
    
    /// Stack profile
    stack_profile: u8,
    
    /// Network key
    network_key: Option<[u8; 16]>,
}

impl NetworkFormation {
    /// Create new network formation manager
    pub fn new() -> Self {
        Self {
            pan_id: 0,
            extended_pan_id: 0,
            channel: 15,
            permit_joining: false,
            permit_duration: 0,
            depth: 0,
            max_depth: 15,
            stack_profile: 2, // Zigbee PRO
            network_key: None,
        }
    }
    
    /// Form network
    pub fn form_network(&mut self, params: FormNetworkParams) -> Result<(), NwkError> {
        // Set channel
        self.channel = params.channel.unwrap_or(15);
        
        // Set PAN ID
        self.pan_id = params.pan_id.unwrap_or_else(|| {
            // Generate random PAN ID
            (self.channel as u16) * 0x1234
        });
        
        // Set extended PAN ID
        self.extended_pan_id = params.extended_pan_id.unwrap_or_else(|| {
            // Generate random extended PAN ID
            0x0011223344556677u64.wrapping_add(self.pan_id as u64)
        });
        
        self.stack_profile = params.stack_profile;
        self.permit_joining = params.permit_joining;
        
        if self.permit_joining {
            self.permit_duration = 0xFF; // Always on
        }
        
        // Generate network key if not set
        if self.network_key.is_none() {
            // In production, use secure random key generation
            let mut key = [0u8; 16];
            for (i, byte) in key.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(17);
            }
            self.network_key = Some(key);
        }
        
        Ok(())
    }
    
    /// Set permit joining
    pub fn set_permit_joining(&mut self, permit: bool, duration: u8) {
        self.permit_joining = permit;
        self.permit_duration = duration as u32;
    }
    
    /// Update permit joining timer
    pub fn update_permit_timer(&mut self, seconds: u32) {
        if self.permit_duration > 0 && self.permit_duration != 0xFF {
            self.permit_duration = self.permit_duration.saturating_sub(seconds);
            if self.permit_duration == 0 {
                self.permit_joining = false;
            }
        }
    }
    
    /// Check if joining is permitted
    pub fn is_permit_joining(&self) -> bool {
        self.permit_joining
    }
    
    /// Get network key
    pub fn network_key(&self) -> Option<&[u8; 16]> {
        self.network_key.as_ref()
    }
    
    /// Get PAN ID
    pub fn pan_id(&self) -> u16 {
        self.pan_id
    }
    
    /// Get extended PAN ID
    pub fn extended_pan_id(&self) -> u64 {
        self.extended_pan_id
    }
    
    /// Get channel
    pub fn channel(&self) -> u8 {
        self.channel
    }
}

use super::nwk::FormNetworkParams;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_routing_manager() {
        let mut manager = RoutingManager::new(0x0000, 0x1122334455667788);
        
        // Add route
        manager.add_route(0x1234, 0x5678, 5).unwrap();
        
        // Find route
        let route = manager.find_route(0x1234).unwrap();
        assert_eq!(route.next_hop, 0x5678);
        assert_eq!(route.cost, 5);
    }
    
    #[test]
    fn test_address_allocation() {
        let mut manager = AddressManager::new(0x0000, 0, 15, 20, 6);
        
        // Allocate address
        let addr1 = manager.allocate_address(0x1111111111111111, false).unwrap();
        let addr2 = manager.allocate_address(0x2222222222222222, false).unwrap();
        
        assert_ne!(addr1, addr2);
        assert_eq!(manager.allocated_count(), 2);
        
        // Find addresses
        assert_eq!(manager.find_short_address(0x1111111111111111), Some(addr1));
        assert_eq!(manager.find_ieee_address(addr1), Some(0x1111111111111111));
    }
    
    #[test]
    fn test_network_formation() {
        let mut formation = NetworkFormation::new();
        
        let params = FormNetworkParams {
            pan_id: Some(0x1234),
            extended_pan_id: None,
            channel: 15,
            permit_joining: true,
            stack_profile: 2,
        };
        
        formation.form_network(params).unwrap();
        
        assert_eq!(formation.pan_id(), 0x1234);
        assert_eq!(formation.channel(), 15);
        assert!(formation.is_permit_joining());
    }
    
    #[test]
    fn test_route_discovery() {
        let mut manager = RoutingManager::new(0x0000, 0x1122334455667788);
        
        // Initiate discovery
        let rreq = manager.discover_route(0x1234).unwrap();
        assert_eq!(rreq.destination_address, 0x1234);
        
        // Should not allow duplicate discovery
        assert!(manager.discover_route(0x1234).is_err());
    }
}
