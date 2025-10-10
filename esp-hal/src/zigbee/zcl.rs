//! Zigbee Cluster Library (ZCL) implementation
//!
//! Common cluster definitions and command handling

/// Cluster ID type
pub type ClusterId = u16;

/// Attribute ID type
pub type AttributeId = u16;

/// Command ID type
pub type CommandId = u8;

/// ZCL cluster trait
pub trait Cluster {
    /// Get cluster ID
    fn cluster_id(&self) -> ClusterId;
    
    /// Handle received command
    fn handle_command(&mut self, command: CommandId, data: &[u8]) -> Result<(), ZclError>;
    
    /// Read attribute
    fn read_attribute(&self, attribute_id: AttributeId) -> Result<AttributeValue, ZclError>;
    
    /// Write attribute
    fn write_attribute(&mut self, attribute_id: AttributeId, value: AttributeValue) -> Result<(), ZclError>;
}

/// Standard cluster IDs
pub mod cluster_id {
    use super::ClusterId;
    
    // General clusters
    pub const BASIC: ClusterId = 0x0000;
    pub const POWER_CONFIG: ClusterId = 0x0001;
    pub const DEVICE_TEMP_CONFIG: ClusterId = 0x0002;
    pub const IDENTIFY: ClusterId = 0x0003;
    pub const GROUPS: ClusterId = 0x0004;
    pub const SCENES: ClusterId = 0x0005;
    pub const ON_OFF: ClusterId = 0x0006;
    pub const ON_OFF_SWITCH_CONFIG: ClusterId = 0x0007;
    pub const LEVEL_CONTROL: ClusterId = 0x0008;
    pub const ALARMS: ClusterId = 0x0009;
    pub const TIME: ClusterId = 0x000A;
    
    // Lighting clusters
    pub const COLOR_CONTROL: ClusterId = 0x0300;
    pub const BALLAST_CONFIG: ClusterId = 0x0301;
    
    // HVAC clusters
    pub const PUMP_CONFIG_CONTROL: ClusterId = 0x0200;
    pub const THERMOSTAT: ClusterId = 0x0201;
    pub const FAN_CONTROL: ClusterId = 0x0202;
    
    // Closures clusters
    pub const SHADE_CONFIG: ClusterId = 0x0100;
    pub const DOOR_LOCK: ClusterId = 0x0101;
    pub const WINDOW_COVERING: ClusterId = 0x0102;
    
    // Security clusters
    pub const IAS_ZONE: ClusterId = 0x0500;
    pub const IAS_ACE: ClusterId = 0x0501;
    pub const IAS_WD: ClusterId = 0x0502;
    
    // Measurement clusters
    pub const ILLUMINANCE_MEASUREMENT: ClusterId = 0x0400;
    pub const ILLUMINANCE_LEVEL_SENSING: ClusterId = 0x0401;
    pub const TEMPERATURE_MEASUREMENT: ClusterId = 0x0402;
    pub const PRESSURE_MEASUREMENT: ClusterId = 0x0403;
    pub const FLOW_MEASUREMENT: ClusterId = 0x0404;
    pub const RELATIVE_HUMIDITY: ClusterId = 0x0405;
    pub const OCCUPANCY_SENSING: ClusterId = 0x0406;
}

/// Attribute value types
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// No data
    NoData,
    
    /// 8-bit data
    Data8(u8),
    
    /// 16-bit data
    Data16(u16),
    
    /// 24-bit data
    Data24(u32),
    
    /// 32-bit data
    Data32(u32),
    
    /// Boolean
    Boolean(bool),
    
    /// 8-bit bitmap
    Bitmap8(u8),
    
    /// 16-bit bitmap
    Bitmap16(u16),
    
    /// Unsigned 8-bit integer
    Uint8(u8),
    
    /// Unsigned 16-bit integer
    Uint16(u16),
    
    /// Unsigned 32-bit integer
    Uint32(u32),
    
    /// Signed 8-bit integer
    Int8(i8),
    
    /// Signed 16-bit integer
    Int16(i16),
    
    /// Signed 32-bit integer
    Int32(i32),
    
    /// Character string
    String(heapless::String<64>),
    
    /// Octet string
    OctetString(heapless::Vec<u8, 64>),
}

/// Attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Attribute ID
    pub id: AttributeId,
    
    /// Attribute value
    pub value: AttributeValue,
    
    /// Read only
    pub read_only: bool,
}

/// ZCL errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZclError {
    /// Invalid command
    InvalidCommand,
    
    /// Invalid attribute
    InvalidAttribute,
    
    /// Invalid value
    InvalidValue,
    
    /// Read only attribute
    ReadOnly,
    
    /// Unsupported cluster
    UnsupportedCluster,
    
    /// Hardware failure
    HardwareFailure,
}

impl core::fmt::Display for ZclError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "Invalid command"),
            Self::InvalidAttribute => write!(f, "Invalid attribute"),
            Self::InvalidValue => write!(f, "Invalid value"),
            Self::ReadOnly => write!(f, "Read only attribute"),
            Self::UnsupportedCluster => write!(f, "Unsupported cluster"),
            Self::HardwareFailure => write!(f, "Hardware failure"),
        }
    }
}

impl core::error::Error for ZclError {}

/// On/Off cluster (0x0006)
pub struct OnOffCluster {
    /// On/off state
    on_off: bool,
}

impl OnOffCluster {
    /// Create a new On/Off cluster
    pub fn new() -> Self {
        Self { on_off: false }
    }
    
    /// Get on/off state
    pub fn is_on(&self) -> bool {
        self.on_off
    }
    
    /// Set on/off state
    pub fn set_on_off(&mut self, on: bool) {
        self.on_off = on;
    }
    
    /// Turn on
    pub fn turn_on(&mut self) {
        self.on_off = true;
    }
    
    /// Turn off
    pub fn turn_off(&mut self) {
        self.on_off = false;
    }
    
    /// Toggle
    pub fn toggle(&mut self) {
        self.on_off = !self.on_off;
    }
}

impl Cluster for OnOffCluster {
    fn cluster_id(&self) -> ClusterId {
        cluster_id::ON_OFF
    }
    
    fn handle_command(&mut self, command: CommandId, _data: &[u8]) -> Result<(), ZclError> {
        match command {
            0x00 => {
                // Off command
                self.turn_off();
                Ok(())
            }
            0x01 => {
                // On command
                self.turn_on();
                Ok(())
            }
            0x02 => {
                // Toggle command
                self.toggle();
                Ok(())
            }
            _ => Err(ZclError::InvalidCommand),
        }
    }
    
    fn read_attribute(&self, attribute_id: AttributeId) -> Result<AttributeValue, ZclError> {
        match attribute_id {
            0x0000 => Ok(AttributeValue::Boolean(self.on_off)),
            _ => Err(ZclError::InvalidAttribute),
        }
    }
    
    fn write_attribute(&mut self, attribute_id: AttributeId, value: AttributeValue) -> Result<(), ZclError> {
        match attribute_id {
            0x0000 => {
                if let AttributeValue::Boolean(on) = value {
                    self.on_off = on;
                    Ok(())
                } else {
                    Err(ZclError::InvalidValue)
                }
            }
            _ => Err(ZclError::InvalidAttribute),
        }
    }
}

impl Default for OnOffCluster {
    fn default() -> Self {
        Self::new()
    }
}

/// Level Control cluster (0x0008)
pub struct LevelControlCluster {
    /// Current level (0-254)
    current_level: u8,
    
    /// On/off state (linked to On/Off cluster)
    on_off: bool,
}

impl LevelControlCluster {
    /// Create a new Level Control cluster
    pub fn new() -> Self {
        Self {
            current_level: 254,
            on_off: false,
        }
    }
    
    /// Get current level
    pub fn current_level(&self) -> u8 {
        self.current_level
    }
    
    /// Set level
    pub fn set_level(&mut self, level: u8) {
        self.current_level = level.min(254);
        if self.current_level > 0 {
            self.on_off = true;
        }
    }
    
    /// Is on
    pub fn is_on(&self) -> bool {
        self.on_off
    }
}

impl Cluster for LevelControlCluster {
    fn cluster_id(&self) -> ClusterId {
        cluster_id::LEVEL_CONTROL
    }
    
    fn handle_command(&mut self, command: CommandId, data: &[u8]) -> Result<(), ZclError> {
        match command {
            0x00 => {
                // Move to level
                if data.len() >= 2 {
                    self.set_level(data[0]);
                    Ok(())
                } else {
                    Err(ZclError::InvalidValue)
                }
            }
            0x01 => {
                // Move (start moving)
                Ok(())
            }
            0x02 => {
                // Step (increment/decrement by amount)
                if data.len() >= 2 {
                    let step_mode = data[0];
                    let step_size = data[1];
                    if step_mode == 0 {
                        // Up
                        self.set_level(self.current_level.saturating_add(step_size));
                    } else {
                        // Down
                        self.set_level(self.current_level.saturating_sub(step_size));
                    }
                    Ok(())
                } else {
                    Err(ZclError::InvalidValue)
                }
            }
            0x03 => {
                // Stop
                Ok(())
            }
            _ => Err(ZclError::InvalidCommand),
        }
    }
    
    fn read_attribute(&self, attribute_id: AttributeId) -> Result<AttributeValue, ZclError> {
        match attribute_id {
            0x0000 => Ok(AttributeValue::Uint8(self.current_level)),
            _ => Err(ZclError::InvalidAttribute),
        }
    }
    
    fn write_attribute(&mut self, attribute_id: AttributeId, value: AttributeValue) -> Result<(), ZclError> {
        match attribute_id {
            0x0000 => {
                if let AttributeValue::Uint8(level) = value {
                    self.set_level(level);
                    Ok(())
                } else {
                    Err(ZclError::InvalidValue)
                }
            }
            _ => Err(ZclError::InvalidAttribute),
        }
    }
}

impl Default for LevelControlCluster {
    fn default() -> Self {
        Self::new()
    }
}

/// Temperature Measurement cluster (0x0402)
pub struct TemperatureMeasurementCluster {
    /// Measured value in 0.01°C
    measured_value: i16,
    
    /// Minimum measured value
    min_measured_value: i16,
    
    /// Maximum measured value
    max_measured_value: i16,
}

impl TemperatureMeasurementCluster {
    /// Create a new Temperature Measurement cluster
    pub fn new() -> Self {
        Self {
            measured_value: 0,
            min_measured_value: -27315, // -273.15°C (absolute zero)
            max_measured_value: 32767,
        }
    }
    
    /// Get measured value in 0.01°C
    pub fn measured_value(&self) -> i16 {
        self.measured_value
    }
    
    /// Set measured value in 0.01°C
    pub fn set_measured_value(&mut self, value: i16) {
        self.measured_value = value.clamp(self.min_measured_value, self.max_measured_value);
    }
}

impl Cluster for TemperatureMeasurementCluster {
    fn cluster_id(&self) -> ClusterId {
        cluster_id::TEMPERATURE_MEASUREMENT
    }
    
    fn handle_command(&mut self, _command: CommandId, _data: &[u8]) -> Result<(), ZclError> {
        // Temperature measurement cluster has no commands
        Err(ZclError::InvalidCommand)
    }
    
    fn read_attribute(&self, attribute_id: AttributeId) -> Result<AttributeValue, ZclError> {
        match attribute_id {
            0x0000 => Ok(AttributeValue::Int16(self.measured_value)),
            0x0001 => Ok(AttributeValue::Int16(self.min_measured_value)),
            0x0002 => Ok(AttributeValue::Int16(self.max_measured_value)),
            _ => Err(ZclError::InvalidAttribute),
        }
    }
    
    fn write_attribute(&mut self, attribute_id: AttributeId, value: AttributeValue) -> Result<(), ZclError> {
        match attribute_id {
            0x0000 => {
                if let AttributeValue::Int16(temp) = value {
                    self.set_measured_value(temp);
                    Ok(())
                } else {
                    Err(ZclError::InvalidValue)
                }
            }
            _ => Err(ZclError::ReadOnly),
        }
    }
}

impl Default for TemperatureMeasurementCluster {
    fn default() -> Self {
        Self::new()
    }
}
