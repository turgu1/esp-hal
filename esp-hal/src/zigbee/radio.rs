//! Radio integration layer for Zigbee driver
//!
//! Connects the Zigbee stack to the IEEE 802.15.4 radio (esp-radio)

use esp_radio::ieee802154::{self, Ieee802154, Config as RadioConfig, Frame, ReceivedFrame};
use esp_hal::peripherals::IEEE802154;
use heapless::Vec;

use crate::zigbee::{NetworkError, Result};

/// Radio frame wrapper
#[derive(Debug, Clone)]
pub struct RadioFrame {
    /// Frame type
    pub frame_type: FrameType,
    
    /// Source PAN ID
    pub src_pan_id: Option<u16>,
    
    /// Destination PAN ID
    pub dst_pan_id: Option<u16>,
    
    /// Source address
    pub src_addr: Option<Address>,
    
    /// Destination address
    pub dst_addr: Option<Address>,
    
    /// Sequence number
    pub sequence: u8,
    
    /// Payload
    pub payload: Vec<u8, 127>,
    
    /// Link quality indicator (for received frames)
    pub lqi: u8,
    
    /// Received signal strength (for received frames)
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

/// Address type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    /// Short 16-bit address
    Short(u16),
    
    /// Extended 64-bit IEEE address
    Extended(u64),
}

/// Radio driver wrapper
pub struct Radio<'a> {
    ieee802154: Ieee802154<'a>,
    tx_buffer: Vec<u8, 127>,
    rx_callback_set: bool,
    tx_callback_set: bool,
}

impl<'a> Radio<'a> {
    /// Create a new radio driver
    pub fn new(radio: IEEE802154<'a>, channel: u8, pan_id: Option<u16>, short_addr: Option<u16>) -> Self {
        let mut ieee802154 = Ieee802154::new(radio);
        
        // Configure radio
        let config = RadioConfig {
            auto_ack_tx: true,
            auto_ack_rx: true,
            enhance_ack_tx: false,
            promiscuous: false,
            coordinator: false,
            rx_when_idle: true,
            txpower: 10,
            channel,
            cca_threshold: ieee802154::pib::CONFIG_IEEE802154_CCA_THRESHOLD,
            cca_mode: ieee802154::CcaMode::Ed,
            pan_id,
            short_addr,
            ext_addr: None,
            rx_queue_size: 10,
        };
        
        ieee802154.set_config(config);
        ieee802154.start_receive();
        
        Self {
            ieee802154,
            tx_buffer: Vec::new(),
            rx_callback_set: false,
            tx_callback_set: false,
        }
    }
    
    /// Set the radio as coordinator
    pub fn set_coordinator(&mut self, is_coordinator: bool) {
        let mut config = RadioConfig::default();
        config.coordinator = is_coordinator;
        self.ieee802154.set_config(config);
    }
    
    /// Set the channel
    pub fn set_channel(&mut self, channel: u8) {
        let mut config = RadioConfig::default();
        config.channel = channel;
        self.ieee802154.set_config(config);
    }
    
    /// Set the PAN ID
    pub fn set_pan_id(&mut self, pan_id: u16) {
        let mut config = RadioConfig::default();
        config.pan_id = Some(pan_id);
        self.ieee802154.set_config(config);
    }
    
    /// Set the short address
    pub fn set_short_address(&mut self, addr: u16) {
        let mut config = RadioConfig::default();
        config.short_addr = Some(addr);
        self.ieee802154.set_config(config);
    }
    
    /// Set the extended address
    pub fn set_extended_address(&mut self, addr: u64) {
        let mut config = RadioConfig::default();
        config.ext_addr = Some(addr);
        self.ieee802154.set_config(config);
    }
    
    /// Set TX power
    pub fn set_tx_power(&mut self, power_dbm: i8) {
        let mut config = RadioConfig::default();
        config.txpower = power_dbm;
        self.ieee802154.set_config(config);
    }
    
    /// Transmit a data frame
    pub fn transmit_data(
        &mut self,
        dst_pan_id: u16,
        dst_addr: Address,
        src_addr: Address,
        payload: &[u8],
        sequence: u8,
    ) -> Result<()> {
        use ieee802154::mac::{
            Address as MacAddress, AddressingMode, FrameControl, FrameType as MacFrameType,
            FrameVersion, Header, PanId,
        };
        
        // Build MAC frame
        let frame_control = FrameControl {
            frame_type: MacFrameType::Data,
            security_enabled: false,
            frame_pending: false,
            ack_request: true,
            pan_id_compression: false,
            sequence_number_suppression: false,
            ie_present: false,
            destination_addressing_mode: match dst_addr {
                Address::Short(_) => AddressingMode::Short,
                Address::Extended(_) => AddressingMode::Extended,
            },
            frame_version: FrameVersion::Ieee802154_2003,
            source_addressing_mode: match src_addr {
                Address::Short(_) => AddressingMode::Short,
                Address::Extended(_) => AddressingMode::Extended,
            },
        };
        
        let destination = match dst_addr {
            Address::Short(addr) => MacAddress::Short(PanId(dst_pan_id), ieee802154::mac::ShortAddress(addr)),
            Address::Extended(addr) => {
                let bytes = addr.to_le_bytes();
                MacAddress::Extended(PanId(dst_pan_id), ieee802154::mac::ExtendedAddress(bytes))
            }
        };
        
        let source = match src_addr {
            Address::Short(addr) => Some(MacAddress::Short(PanId(dst_pan_id), ieee802154::mac::ShortAddress(addr))),
            Address::Extended(addr) => {
                let bytes = addr.to_le_bytes();
                Some(MacAddress::Extended(PanId(dst_pan_id), ieee802154::mac::ExtendedAddress(bytes)))
            }
        };
        
        let header = Header {
            frame_control,
            seq: sequence,
            destination: Some(destination),
            source,
        };
        
        let frame = Frame {
            header,
            content: ieee802154::mac::FrameContent::Data,
            payload: payload.to_vec(),
            footer: [0, 0], // Will be calculated by radio
        };
        
        self.ieee802154.transmit(&frame).map_err(|_| NetworkError::TransmissionFailed)
    }
    
    /// Transmit a beacon frame
    pub fn transmit_beacon(
        &mut self,
        pan_id: u16,
        src_addr: u16,
        payload: &[u8],
        sequence: u8,
    ) -> Result<()> {
        use ieee802154::mac::{
            Address as MacAddress, AddressingMode, FrameControl, FrameType as MacFrameType,
            FrameVersion, Header, PanId, ShortAddress,
        };
        
        let frame_control = FrameControl {
            frame_type: MacFrameType::Beacon,
            security_enabled: false,
            frame_pending: false,
            ack_request: false,
            pan_id_compression: false,
            sequence_number_suppression: false,
            ie_present: false,
            destination_addressing_mode: AddressingMode::None,
            frame_version: FrameVersion::Ieee802154_2003,
            source_addressing_mode: AddressingMode::Short,
        };
        
        let source = MacAddress::Short(PanId(pan_id), ShortAddress(src_addr));
        
        let header = Header {
            frame_control,
            seq: sequence,
            destination: None,
            source: Some(source),
        };
        
        let frame = Frame {
            header,
            content: ieee802154::mac::FrameContent::Beacon,
            payload: payload.to_vec(),
            footer: [0, 0],
        };
        
        self.ieee802154.transmit(&frame).map_err(|_| NetworkError::TransmissionFailed)
    }
    
    /// Transmit a MAC command frame
    pub fn transmit_mac_command(
        &mut self,
        dst_pan_id: u16,
        dst_addr: Address,
        src_addr: Address,
        command: u8,
        payload: &[u8],
        sequence: u8,
    ) -> Result<()> {
        use ieee802154::mac::{
            Address as MacAddress, AddressingMode, FrameControl, FrameType as MacFrameType,
            FrameVersion, Header, PanId,
        };
        
        let frame_control = FrameControl {
            frame_type: MacFrameType::MacCommand,
            security_enabled: false,
            frame_pending: false,
            ack_request: true,
            pan_id_compression: false,
            sequence_number_suppression: false,
            ie_present: false,
            destination_addressing_mode: match dst_addr {
                Address::Short(_) => AddressingMode::Short,
                Address::Extended(_) => AddressingMode::Extended,
            },
            frame_version: FrameVersion::Ieee802154_2003,
            source_addressing_mode: match src_addr {
                Address::Short(_) => AddressingMode::Short,
                Address::Extended(_) => AddressingMode::Extended,
            },
        };
        
        let destination = match dst_addr {
            Address::Short(addr) => MacAddress::Short(PanId(dst_pan_id), ieee802154::mac::ShortAddress(addr)),
            Address::Extended(addr) => {
                let bytes = addr.to_le_bytes();
                MacAddress::Extended(PanId(dst_pan_id), ieee802154::mac::ExtendedAddress(bytes))
            }
        };
        
        let source = match src_addr {
            Address::Short(addr) => Some(MacAddress::Short(PanId(dst_pan_id), ieee802154::mac::ShortAddress(addr))),
            Address::Extended(addr) => {
                let bytes = addr.to_le_bytes();
                Some(MacAddress::Extended(PanId(dst_pan_id), ieee802154::mac::ExtendedAddress(bytes)))
            }
        };
        
        let header = Header {
            frame_control,
            seq: sequence,
            destination: Some(destination),
            source,
        };
        
        // Prepend command byte to payload
        let mut full_payload = Vec::new();
        full_payload.push(command).ok();
        full_payload.extend_from_slice(payload).ok();
        
        let frame = Frame {
            header,
            content: ieee802154::mac::FrameContent::Command,
            payload: full_payload.to_vec(),
            footer: [0, 0],
        };
        
        self.ieee802154.transmit(&frame).map_err(|_| NetworkError::TransmissionFailed)
    }
    
    /// Receive a frame if available
    pub fn receive(&mut self) -> Option<Result<RadioFrame>> {
        match self.ieee802154.received() {
            Some(Ok(received)) => Some(Self::convert_received_frame(received)),
            Some(Err(_)) => Some(Err(NetworkError::TransmissionFailed)),
            None => None,
        }
    }
    
    /// Convert IEEE 802.15.4 frame to RadioFrame
    fn convert_received_frame(received: ReceivedFrame) -> Result<RadioFrame> {
        use ieee802154::mac::{Address as MacAddress, FrameType as MacFrameType};
        
        let frame = received.frame;
        let header = frame.header;
        
        let frame_type = match header.frame_control.frame_type {
            MacFrameType::Beacon => FrameType::Beacon,
            MacFrameType::Data => FrameType::Data,
            MacFrameType::Ack => FrameType::Ack,
            MacFrameType::MacCommand => FrameType::MacCommand,
            _ => return Err(NetworkError::InvalidParameter),
        };
        
        let (dst_pan_id, dst_addr) = match header.destination {
            Some(MacAddress::Short(pan_id, addr)) => (Some(pan_id.0), Some(Address::Short(addr.0))),
            Some(MacAddress::Extended(pan_id, addr)) => {
                let ieee_addr = u64::from_le_bytes(addr.0);
                (Some(pan_id.0), Some(Address::Extended(ieee_addr)))
            }
            None => (None, None),
        };
        
        let (src_pan_id, src_addr) = match header.source {
            Some(MacAddress::Short(pan_id, addr)) => (Some(pan_id.0), Some(Address::Short(addr.0))),
            Some(MacAddress::Extended(pan_id, addr)) => {
                let ieee_addr = u64::from_le_bytes(addr.0);
                (Some(pan_id.0), Some(Address::Extended(ieee_addr)))
            }
            None => (None, None),
        };
        
        let mut payload = Vec::new();
        payload.extend_from_slice(&frame.payload).ok();
        
        Ok(RadioFrame {
            frame_type,
            src_pan_id,
            dst_pan_id,
            src_addr,
            dst_addr,
            sequence: header.seq,
            payload,
            lqi: received.lqi,
            rssi: received.rssi,
        })
    }
    
    /// Start receiving frames
    pub fn start_receive(&mut self) {
        self.ieee802154.start_receive();
    }
    
    /// Get raw received frame (for debugging)
    pub fn raw_received(&mut self) -> Option<ieee802154::RawReceived> {
        self.ieee802154.raw_received()
    }
}

/// Energy detection for channel scanning
pub fn perform_energy_detection(radio: &mut Radio, channel: u8) -> Result<i8> {
    // Switch to channel
    radio.set_channel(channel);
    
    // In a real implementation, this would:
    // 1. Perform energy detection scan
    // 2. Return the energy level in dBm
    
    // Placeholder: return -80 dBm
    Ok(-80)
}

/// Scan for beacons on a channel
pub fn scan_beacons(
    radio: &mut Radio,
    channel: u8,
    duration_ms: u32,
) -> Result<Vec<RadioFrame, 16>> {
    // Switch to channel
    radio.set_channel(channel);
    
    let mut beacons = Vec::new();
    let start = 0; // Would use actual timer
    
    // Scan for beacons
    loop {
        if let Some(Ok(frame)) = radio.receive() {
            if frame.frame_type == FrameType::Beacon {
                beacons.push(frame).ok();
            }
        }
        
        // Check timeout (placeholder)
        // In real implementation, check elapsed time
        if beacons.len() >= 16 {
            break;
        }
    }
    
    Ok(beacons)
}
