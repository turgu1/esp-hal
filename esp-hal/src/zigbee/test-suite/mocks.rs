//! Mock implementations for testing
//!
//! Provides mock IEEE 802.15.4 radio and other hardware abstractions

use crate::zigbee::*;
use heapless::Vec;

/// Mock IEEE 802.15.4 radio
pub struct MockRadio {
    /// Transmission queue
    tx_queue: Vec<MockFrame, 32>,
    
    /// Reception queue
    rx_queue: Vec<MockFrame, 32>,
    
    /// Current channel
    channel: u8,
    
    /// Current PAN ID
    pan_id: u16,
    
    /// Current short address
    short_addr: u16,
    
    /// IEEE address
    ieee_addr: u64,
    
    /// TX power
    tx_power: i8,
    
    /// Promiscuous mode
    promiscuous: bool,
    
    /// Coordinator mode
    is_coordinator: bool,
    
    /// Error injection
    inject_tx_error: bool,
    inject_rx_error: bool,
    
    /// Statistics
    tx_count: u32,
    rx_count: u32,
    tx_errors: u32,
    rx_errors: u32,
}

/// Mock frame
#[derive(Debug, Clone)]
pub struct MockFrame {
    /// Source address
    pub src_addr: u16,
    
    /// Destination address
    pub dst_addr: u16,
    
    /// PAN ID
    pub pan_id: u16,
    
    /// Sequence number
    pub sequence: u8,
    
    /// Frame type
    pub frame_type: FrameType,
    
    /// Payload
    pub payload: Vec<u8, 128>,
    
    /// LQI (Link Quality Indicator)
    pub lqi: u8,
    
    /// RSSI (Received Signal Strength Indicator)
    pub rssi: i8,
}

/// Frame type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Beacon frame
    Beacon,
    
    /// Data frame
    Data,
    
    /// Acknowledgment frame
    Ack,
    
    /// MAC command frame
    MacCommand,
}

impl MockRadio {
    /// Create a new mock radio
    pub fn new(ieee_addr: u64) -> Self {
        Self {
            tx_queue: Vec::new(),
            rx_queue: Vec::new(),
            channel: 15,
            pan_id: 0xFFFF,
            short_addr: 0xFFFF,
            ieee_addr,
            tx_power: 10,
            promiscuous: false,
            is_coordinator: false,
            inject_tx_error: false,
            inject_rx_error: false,
            tx_count: 0,
            rx_count: 0,
            tx_errors: 0,
            rx_errors: 0,
        }
    }
    
    /// Set channel
    pub fn set_channel(&mut self, channel: u8) {
        self.channel = channel;
    }
    
    /// Get channel
    pub fn channel(&self) -> u8 {
        self.channel
    }
    
    /// Set PAN ID
    pub fn set_pan_id(&mut self, pan_id: u16) {
        self.pan_id = pan_id;
    }
    
    /// Get PAN ID
    pub fn pan_id(&self) -> u16 {
        self.pan_id
    }
    
    /// Set short address
    pub fn set_short_address(&mut self, addr: u16) {
        self.short_addr = addr;
    }
    
    /// Get short address
    pub fn short_address(&self) -> u16 {
        self.short_addr
    }
    
    /// Get IEEE address
    pub fn ieee_address(&self) -> u64 {
        self.ieee_addr
    }
    
    /// Set coordinator mode
    pub fn set_coordinator(&mut self, is_coord: bool) {
        self.is_coordinator = is_coord;
    }
    
    /// Transmit a frame
    pub fn transmit(&mut self, frame: MockFrame) -> Result<(), MockRadioError> {
        if self.inject_tx_error {
            self.tx_errors += 1;
            return Err(MockRadioError::TransmitFailed);
        }
        
        self.tx_queue.push(frame).map_err(|_| MockRadioError::TxQueueFull)?;
        self.tx_count += 1;
        Ok(())
    }
    
    /// Receive a frame
    pub fn receive(&mut self) -> Option<MockFrame> {
        if self.inject_rx_error {
            self.rx_errors += 1;
            return None;
        }
        
        if let Some(frame) = self.rx_queue.pop() {
            self.rx_count += 1;
            Some(frame)
        } else {
            None
        }
    }
    
    /// Inject a frame for reception (simulate incoming frame)
    pub fn inject_frame(&mut self, frame: MockFrame) -> Result<(), MockRadioError> {
        self.rx_queue.push(frame).map_err(|_| MockRadioError::RxQueueFull)
    }
    
    /// Get transmitted frame (for verification)
    pub fn get_transmitted(&mut self) -> Option<MockFrame> {
        self.tx_queue.pop()
    }
    
    /// Enable error injection
    pub fn inject_errors(&mut self, tx: bool, rx: bool) {
        self.inject_tx_error = tx;
        self.inject_rx_error = rx;
    }
    
    /// Get statistics
    pub fn statistics(&self) -> RadioStatistics {
        RadioStatistics {
            tx_count: self.tx_count,
            rx_count: self.rx_count,
            tx_errors: self.tx_errors,
            rx_errors: self.rx_errors,
        }
    }
    
    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.tx_count = 0;
        self.rx_count = 0;
        self.tx_errors = 0;
        self.rx_errors = 0;
    }
    
    /// Clear all queues
    pub fn clear_queues(&mut self) {
        self.tx_queue.clear();
        self.rx_queue.clear();
    }
}

/// Radio statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioStatistics {
    pub tx_count: u32,
    pub rx_count: u32,
    pub tx_errors: u32,
    pub rx_errors: u32,
}

/// Mock radio errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockRadioError {
    TransmitFailed,
    TxQueueFull,
    RxQueueFull,
    InvalidChannel,
    InvalidAddress,
}

impl MockFrame {
    /// Create a new data frame
    pub fn data(src: u16, dst: u16, pan_id: u16, payload: &[u8]) -> Self {
        let mut frame_payload = Vec::new();
        frame_payload.extend_from_slice(payload).ok();
        
        Self {
            src_addr: src,
            dst_addr: dst,
            pan_id,
            sequence: 0,
            frame_type: FrameType::Data,
            payload: frame_payload,
            lqi: 255,
            rssi: -40,
        }
    }
    
    /// Create a new beacon frame
    pub fn beacon(src: u16, pan_id: u16) -> Self {
        Self {
            src_addr: src,
            dst_addr: 0xFFFF,
            pan_id,
            sequence: 0,
            frame_type: FrameType::Beacon,
            payload: Vec::new(),
            lqi: 255,
            rssi: -40,
        }
    }
    
    /// Create an ACK frame
    pub fn ack(sequence: u8) -> Self {
        Self {
            src_addr: 0x0000,
            dst_addr: 0x0000,
            pan_id: 0x0000,
            sequence,
            frame_type: FrameType::Ack,
            payload: Vec::new(),
            lqi: 255,
            rssi: -40,
        }
    }
    
    /// Set LQI
    pub fn with_lqi(mut self, lqi: u8) -> Self {
        self.lqi = lqi;
        self
    }
    
    /// Set RSSI
    pub fn with_rssi(mut self, rssi: i8) -> Self {
        self.rssi = rssi;
        self
    }
}

/// Mock timer for testing time-dependent functionality
pub struct MockTimer {
    current_time_ms: u64,
    auto_advance: bool,
}

impl MockTimer {
    /// Create a new mock timer
    pub fn new() -> Self {
        Self {
            current_time_ms: 0,
            auto_advance: false,
        }
    }
    
    /// Get current time in milliseconds
    pub fn now_ms(&self) -> u64 {
        self.current_time_ms
    }
    
    /// Advance time manually
    pub fn advance(&mut self, ms: u64) {
        self.current_time_ms += ms;
    }
    
    /// Set auto-advance mode
    pub fn set_auto_advance(&mut self, enabled: bool) {
        self.auto_advance = enabled;
    }
    
    /// Reset timer
    pub fn reset(&mut self) {
        self.current_time_ms = 0;
    }
}

impl Default for MockTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock storage for persistent data
pub struct MockStorage {
    data: Vec<(u32, Vec<u8, 256>), 16>,
}

impl MockStorage {
    /// Create new mock storage
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }
    
    /// Write data
    pub fn write(&mut self, key: u32, data: &[u8]) -> Result<(), MockStorageError> {
        // Remove existing entry with same key
        self.data.retain(|entry| entry.0 != key);
        
        let mut value = Vec::new();
        value.extend_from_slice(data).map_err(|_| MockStorageError::DataTooLarge)?;
        
        self.data.push((key, value)).map_err(|_| MockStorageError::StorageFull)
    }
    
    /// Read data
    pub fn read(&self, key: u32) -> Result<&[u8], MockStorageError> {
        self.data
            .iter()
            .find(|entry| entry.0 == key)
            .map(|entry| entry.1.as_slice())
            .ok_or(MockStorageError::KeyNotFound)
    }
    
    /// Delete data
    pub fn delete(&mut self, key: u32) -> Result<(), MockStorageError> {
        let len_before = self.data.len();
        self.data.retain(|entry| entry.0 != key);
        
        if self.data.len() == len_before {
            Err(MockStorageError::KeyNotFound)
        } else {
            Ok(())
        }
    }
    
    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock storage errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockStorageError {
    KeyNotFound,
    StorageFull,
    DataTooLarge,
}
