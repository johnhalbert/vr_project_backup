//! Core device interfaces and traits for the Hardware Access API.
//!
//! This module defines the fundamental traits and structures that all hardware
//! devices in the VR headset must implement. It provides a common interface for
//! device discovery, initialization, control, and monitoring.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Error type for device operations.
#[derive(Error, Debug, Clone)]
pub enum DeviceError {
    /// Device not found
    #[error("Device not found: {0}")]
    NotFound(String),
    
    /// Device initialization failed
    #[error("Device initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Device communication error
    #[error("Device communication error: {0}")]
    CommunicationError(String),
    
    /// Device timeout
    #[error("Device operation timed out: {0}")]
    Timeout(String),
    
    /// Device busy
    #[error("Device is busy: {0}")]
    Busy(String),
    
    /// Device in invalid state
    #[error("Device in invalid state: {0}")]
    InvalidState(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Hardware failure
    #[error("Hardware failure: {0}")]
    HardwareFailure(String),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for device operations.
pub type DeviceResult<T> = Result<T, DeviceError>;

/// Device capability flags.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// Device can be powered on/off
    PowerControl,
    
    /// Device supports standby mode
    Standby,
    
    /// Device supports firmware updates
    FirmwareUpdate,
    
    /// Device supports self-test
    SelfTest,
    
    /// Device supports calibration
    Calibration,
    
    /// Device supports diagnostics
    Diagnostics,
    
    /// Device supports event generation
    Events,
    
    /// Device supports configuration
    Configuration,
    
    /// Device supports streaming
    Streaming,
    
    /// Device supports DMA
    DMA,
    
    /// Device supports interrupts
    Interrupts,
    
    /// Device supports hot-plugging
    HotPlug,
    
    /// Device supports power management
    PowerManagement,
    
    /// Device supports thermal management
    ThermalManagement,
    
    /// Device supports error recovery
    ErrorRecovery,
    
    /// Device supports statistics
    Statistics,
    
    /// Device supports logging
    Logging,
    
    /// Device supports remote control
    RemoteControl,
    
    /// Device supports security features
    Security,
    
    /// Device supports multi-user access
    MultiUser,
    
    /// Device supports synchronization
    Synchronization,
    
    /// Device supports low-latency operation
    LowLatency,
    
    /// Device supports high-precision timing
    HighPrecisionTiming,
    
    /// Device supports real-time operation
    RealTime,
    
    /// Device supports custom capabilities (see device documentation)
    Custom(u32),
}

/// Device state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceState {
    /// Device is disconnected
    Disconnected,
    
    /// Device is connected but not initialized
    Connected,
    
    /// Device is initializing
    Initializing,
    
    /// Device is ready
    Ready,
    
    /// Device is in use
    Active,
    
    /// Device is in standby mode
    Standby,
    
    /// Device is in error state
    Error,
    
    /// Device is in recovery mode
    Recovery,
    
    /// Device is updating firmware
    Updating,
    
    /// Device is shutting down
    ShuttingDown,
    
    /// Device state is unknown
    Unknown,
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Disconnected => write!(f, "Disconnected"),
            DeviceState::Connected => write!(f, "Connected"),
            DeviceState::Initializing => write!(f, "Initializing"),
            DeviceState::Ready => write!(f, "Ready"),
            DeviceState::Active => write!(f, "Active"),
            DeviceState::Standby => write!(f, "Standby"),
            DeviceState::Error => write!(f, "Error"),
            DeviceState::Recovery => write!(f, "Recovery"),
            DeviceState::Updating => write!(f, "Updating"),
            DeviceState::ShuttingDown => write!(f, "Shutting Down"),
            DeviceState::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Device type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    /// Display device
    Display,
    
    /// Audio input device
    AudioInput,
    
    /// Audio output device
    AudioOutput,
    
    /// Camera device
    Camera,
    
    /// IMU device
    IMU,
    
    /// Power management device
    Power,
    
    /// Storage device
    Storage,
    
    /// Network device
    Network,
    
    /// Input device
    Input,
    
    /// Output device
    Output,
    
    /// Sensor device
    Sensor,
    
    /// Actuator device
    Actuator,
    
    /// Processor device
    Processor,
    
    /// Accelerator device
    Accelerator,
    
    /// Other device type
    Other(String),
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Display => write!(f, "Display"),
            DeviceType::AudioInput => write!(f, "Audio Input"),
            DeviceType::AudioOutput => write!(f, "Audio Output"),
            DeviceType::Camera => write!(f, "Camera"),
            DeviceType::IMU => write!(f, "IMU"),
            DeviceType::Power => write!(f, "Power"),
            DeviceType::Storage => write!(f, "Storage"),
            DeviceType::Network => write!(f, "Network"),
            DeviceType::Input => write!(f, "Input"),
            DeviceType::Output => write!(f, "Output"),
            DeviceType::Sensor => write!(f, "Sensor"),
            DeviceType::Actuator => write!(f, "Actuator"),
            DeviceType::Processor => write!(f, "Processor"),
            DeviceType::Accelerator => write!(f, "Accelerator"),
            DeviceType::Other(name) => write!(f, "Other({})", name),
        }
    }
}

/// Device bus type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceBus {
    /// USB bus
    USB,
    
    /// PCI Express bus
    PCIe,
    
    /// I2C bus
    I2C,
    
    /// SPI bus
    SPI,
    
    /// UART bus
    UART,
    
    /// GPIO pins
    GPIO,
    
    /// MIPI CSI bus
    MIPI_CSI,
    
    /// MIPI DSI bus
    MIPI_DSI,
    
    /// Internal bus
    Internal,
    
    /// Virtual bus
    Virtual,
    
    /// Other bus type
    Other(String),
}

/// Device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier
    pub id: String,
    
    /// Device name
    pub name: String,
    
    /// Device type
    pub device_type: DeviceType,
    
    /// Device manufacturer
    pub manufacturer: String,
    
    /// Device model
    pub model: String,
    
    /// Device serial number
    pub serial_number: Option<String>,
    
    /// Device firmware version
    pub firmware_version: Option<String>,
    
    /// Device driver version
    pub driver_version: Option<String>,
    
    /// Device bus type
    pub bus_type: DeviceBus,
    
    /// Device bus address
    pub bus_address: Option<String>,
    
    /// Device capabilities
    pub capabilities: Vec<DeviceCapability>,
    
    /// Device state
    pub state: DeviceState,
    
    /// Device description
    pub description: Option<String>,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Device creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Device last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl DeviceInfo {
    /// Create a new DeviceInfo with minimal required fields.
    pub fn new(
        id: String,
        name: String,
        device_type: DeviceType,
        manufacturer: String,
        model: String,
        bus_type: DeviceBus,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            device_type,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type,
            bus_address: None,
            capabilities: Vec::new(),
            state: DeviceState::Connected,
            description: None,
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if the device has a specific capability.
    pub fn has_capability(&self, capability: DeviceCapability) -> bool {
        self.capabilities.contains(&capability)
    }
    
    /// Add a capability to the device.
    pub fn add_capability(&mut self, capability: DeviceCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
            self.updated_at = chrono::Utc::now();
        }
    }
    
    /// Remove a capability from the device.
    pub fn remove_capability(&mut self, capability: DeviceCapability) {
        if let Some(index) = self.capabilities.iter().position(|c| *c == capability) {
            self.capabilities.remove(index);
            self.updated_at = chrono::Utc::now();
        }
    }
    
    /// Update the device state.
    pub fn update_state(&mut self, state: DeviceState) {
        if self.state != state {
            self.state = state;
            self.updated_at = chrono::Utc::now();
        }
    }
    
    /// Set a device property.
    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }
    
    /// Get a device property.
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
    
    /// Remove a device property.
    pub fn remove_property(&mut self, key: &str) -> Option<String> {
        let result = self.properties.remove(key);
        if result.is_some() {
            self.updated_at = chrono::Utc::now();
        }
        result
    }
}

/// Device event type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceEventType {
    /// Device state changed
    StateChanged {
        /// Previous state
        previous: DeviceState,
        /// New state
        current: DeviceState,
    },
    
    /// Device property changed
    PropertyChanged {
        /// Property key
        key: String,
        /// Previous value
        previous: Option<String>,
        /// New value
        current: Option<String>,
    },
    
    /// Device error occurred
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
        /// Error severity
        severity: DeviceErrorSeverity,
    },
    
    /// Device connected
    Connected,
    
    /// Device disconnected
    Disconnected,
    
    /// Device initialized
    Initialized,
    
    /// Device shutdown
    Shutdown,
    
    /// Device reset
    Reset,
    
    /// Device firmware update started
    FirmwareUpdateStarted,
    
    /// Device firmware update progress
    FirmwareUpdateProgress {
        /// Progress percentage (0-100)
        progress: u8,
        /// Status message
        status: String,
    },
    
    /// Device firmware update completed
    FirmwareUpdateCompleted {
        /// Success flag
        success: bool,
        /// Status message
        status: String,
    },
    
    /// Device calibration started
    CalibrationStarted,
    
    /// Device calibration progress
    CalibrationProgress {
        /// Progress percentage (0-100)
        progress: u8,
        /// Status message
        status: String,
    },
    
    /// Device calibration completed
    CalibrationCompleted {
        /// Success flag
        success: bool,
        /// Status message
        status: String,
    },
    
    /// Device self-test started
    SelfTestStarted,
    
    /// Device self-test completed
    SelfTestCompleted {
        /// Success flag
        success: bool,
        /// Status message
        status: String,
    },
    
    /// Device custom event
    Custom {
        /// Event name
        name: String,
        /// Event data
        data: HashMap<String, String>,
    },
}

/// Device error severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceErrorSeverity {
    /// Informational message
    Info,
    
    /// Warning message
    Warning,
    
    /// Error message
    Error,
    
    /// Critical error message
    Critical,
    
    /// Fatal error message
    Fatal,
}

/// Device event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    /// Event ID
    pub id: String,
    
    /// Device ID
    pub device_id: String,
    
    /// Event type
    pub event_type: DeviceEventType,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

impl DeviceEvent {
    /// Create a new DeviceEvent.
    pub fn new(device_id: String, event_type: DeviceEventType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            event_type,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the event.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Device event handler function type.
pub type DeviceEventHandler = Box<dyn Fn(&DeviceEvent) + Send + Sync>;

/// Device trait.
///
/// This trait defines the common interface that all hardware devices must implement.
pub trait Device: Send + Sync + Debug {
    /// Get the device information.
    fn info(&self) -> DeviceResult<DeviceInfo>;
    
    /// Initialize the device.
    fn initialize(&mut self) -> DeviceResult<()>;
    
    /// Shutdown the device.
    fn shutdown(&mut self) -> DeviceResult<()>;
    
    /// Reset the device.
    fn reset(&mut self) -> DeviceResult<()>;
    
    /// Check if the device is connected.
    fn is_connected(&self) -> DeviceResult<bool>;
    
    /// Get the device state.
    fn state(&self) -> DeviceResult<DeviceState>;
    
    /// Set the device state.
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()>;
    
    /// Check if the device has a specific capability.
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool>;
    
    /// Get the device properties.
    fn properties(&self) -> DeviceResult<HashMap<String, String>>;
    
    /// Get a specific device property.
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>>;
    
    /// Set a device property.
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()>;
    
    /// Register an event handler.
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()>;
    
    /// Unregister all event handlers.
    fn unregister_event_handlers(&mut self) -> DeviceResult<()>;
    
    /// Perform a self-test.
    fn self_test(&mut self) -> DeviceResult<bool> {
        Err(DeviceError::NotSupported("Self-test not supported".to_string()))
    }
    
    /// Perform a calibration.
    fn calibrate(&mut self) -> DeviceResult<bool> {
        Err(DeviceError::NotSupported("Calibration not supported".to_string()))
    }
    
    /// Update the device firmware.
    fn update_firmware(&mut self, _firmware_path: &str) -> DeviceResult<bool> {
        Err(DeviceError::NotSupported("Firmware update not supported".to_string()))
    }
    
    /// Get the device diagnostics.
    fn get_diagnostics(&self) -> DeviceResult<HashMap<String, String>> {
        Err(DeviceError::NotSupported("Diagnostics not supported".to_string()))
    }
    
    /// Get the device statistics.
    fn get_statistics(&self) -> DeviceResult<HashMap<String, String>> {
        Err(DeviceError::NotSupported("Statistics not supported".to_string()))
    }
    
    /// Get the device logs.
    fn get_logs(&self, _max_entries: Option<usize>) -> DeviceResult<Vec<String>> {
        Err(DeviceError::NotSupported("Logs not supported".to_string()))
    }
    
    /// Clear the device logs.
    fn clear_logs(&mut self) -> DeviceResult<()> {
        Err(DeviceError::NotSupported("Logs not supported".to_string()))
    }
    
    /// Get the device power state.
    fn get_power_state(&self) -> DeviceResult<DevicePowerState> {
        Err(DeviceError::NotSupported("Power state not supported".to_string()))
    }
    
    /// Set the device power state.
    fn set_power_state(&mut self, _state: DevicePowerState) -> DeviceResult<()> {
        Err(DeviceError::NotSupported("Power state not supported".to_string()))
    }
    
    /// Get the device temperature.
    fn get_temperature(&self) -> DeviceResult<f32> {
        Err(DeviceError::NotSupported("Temperature not supported".to_string()))
    }
    
    /// Get the device power consumption.
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Err(DeviceError::NotSupported("Power consumption not supported".to_string()))
    }
    
    /// Clone the device.
    fn clone_box(&self) -> Box<dyn Device>;
    
    /// Convert to Any for downcasting.
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to Any for downcasting (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Device power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevicePowerState {
    /// Device is powered off
    Off,
    
    /// Device is in low-power mode
    LowPower,
    
    /// Device is in standby mode
    Standby,
    
    /// Device is powered on
    On,
    
    /// Device is in high-performance mode
    HighPerformance,
}

/// Device discovery trait.
pub trait DeviceDiscovery: Send + Sync + Debug {
    /// Discover devices.
    fn discover(&self) -> DeviceResult<Vec<Box<dyn Device>>>;
    
    /// Discover devices by type.
    fn discover_by_type(&self, device_type: DeviceType) -> DeviceResult<Vec<Box<dyn Device>>>;
    
    /// Discover devices by bus.
    fn discover_by_bus(&self, bus_type: DeviceBus) -> DeviceResult<Vec<Box<dyn Device>>>;
    
    /// Discover devices by capability.
    fn discover_by_capability(
        &self,
        capability: DeviceCapability,
    ) -> DeviceResult<Vec<Box<dyn Device>>>;
    
    /// Discover a specific device by ID.
    fn discover_by_id(&self, id: &str) -> DeviceResult<Option<Box<dyn Device>>>;
    
    /// Discover a specific device by serial number.
    fn discover_by_serial_number(
        &self,
        serial_number: &str,
    ) -> DeviceResult<Option<Box<dyn Device>>>;
    
    /// Register a device discovery callback.
    fn register_discovery_callback(
        &mut self,
        callback: Box<dyn Fn(Box<dyn Device>) + Send + Sync>,
    ) -> DeviceResult<()>;
    
    /// Unregister all device discovery callbacks.
    fn unregister_discovery_callbacks(&mut self) -> DeviceResult<()>;
    
    /// Start device discovery.
    fn start_discovery(&mut self) -> DeviceResult<()>;
    
    /// Stop device discovery.
    fn stop_discovery(&mut self) -> DeviceResult<()>;
    
    /// Check if device discovery is running.
    fn is_discovery_running(&self) -> DeviceResult<bool>;
}

/// Device factory trait.
pub trait DeviceFactory: Send + Sync + Debug {
    /// Create a device.
    fn create_device(&self, device_type: DeviceType, config: &HashMap<String, String>) -> DeviceResult<Box<dyn Device>>;
    
    /// Get the supported device types.
    fn get_supported_device_types(&self) -> Vec<DeviceType>;
    
    /// Get the required configuration keys for a device type.
    fn get_required_config_keys(&self, device_type: DeviceType) -> Vec<String>;
    
    /// Get the optional configuration keys for a device type.
    fn get_optional_config_keys(&self, device_type: DeviceType) -> Vec<String>;
    
    /// Validate the configuration for a device type.
    fn validate_config(&self, device_type: DeviceType, config: &HashMap<String, String>) -> DeviceResult<()>;
}

/// Mock device for testing.
#[derive(Debug, Clone)]
pub struct MockDevice {
    /// Device info
    pub info: DeviceInfo,
    
    /// Device state
    pub state: DeviceState,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Event handlers
    pub event_handlers: Vec<DeviceEventHandler>,
}

impl MockDevice {
    /// Create a new MockDevice.
    pub fn new(
        id: String,
        name: String,
        device_type: DeviceType,
        manufacturer: String,
        model: String,
    ) -> Self {
        let mut info = DeviceInfo::new(
            id,
            name,
            device_type,
            manufacturer,
            model,
            DeviceBus::Virtual,
        );
        
        info.state = DeviceState::Connected;
        
        Self {
            info,
            state: DeviceState::Connected,
            properties: HashMap::new(),
            event_handlers: Vec::new(),
        }
    }
    
    /// Fire an event.
    pub fn fire_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for MockDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        self.fire_event(DeviceEventType::Initialized);
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::ShuttingDown;
        self.info.state = DeviceState::ShuttingDown;
        self.fire_event(DeviceEventType::Shutdown);
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Initializing;
        self.info.state = DeviceState::Initializing;
        self.fire_event(DeviceEventType::Reset);
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous = self.state;
        self.state = state;
        self.info.state = state;
        self.fire_event(DeviceEventType::StateChanged {
            previous,
            current: state,
        });
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.has_capability(capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.properties.get(key).cloned();
        self.properties.insert(key.to_string(), value.to_string());
        self.fire_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
