//! Tracking device implementation for the Hardware Access API.
//!
//! This module provides concrete implementations of tracking devices for the VR headset,
//! including IMUs, cameras, and controllers.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};
use super::tracking::{
    CameraConfig, CameraDevice, CameraFrame, CameraFrameFormat, CameraResolution,
    ControllerButtonState, ControllerDevice, ControllerInputEvent, ControllerInputType,
    ControllerState, HapticFeedback, IMUConfig, IMUDevice, IMUData, IMUSampleRate,
    TrackingCapability, TrackingDevice, TrackingMode, TrackingState,
};

/// VR IMU device implementation.
#[derive(Debug)]
pub struct VRIMUDevice {
    /// Device information
    info: DeviceInfo,
    
    /// IMU configuration
    config: IMUConfig,
    
    /// Available sample rates
    available_sample_rates: Vec<IMUSampleRate>,
    
    /// Current tracking state
    tracking_state: TrackingState,
    
    /// Last IMU data
    last_data: Option<IMUData>,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Temperature in Celsius
    temperature: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRIMUDevice {
    /// Create a new VRIMUDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Tracking,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::I2C,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::ThermalManagement,
                DeviceCapability::Tracking,
            ],
            state: DeviceState::Connected,
            description: Some("VR IMU Sensor".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add tracking-specific properties
        info.properties.insert("tracking_type".to_string(), "imu".to_string());
        
        // Create available sample rates
        let available_sample_rates = vec![
            IMUSampleRate::new(100),
            IMUSampleRate::new(200),
            IMUSampleRate::new(500),
            IMUSampleRate::new(1000),
        ];
        
        // Create IMU configuration
        let config = IMUConfig::vr_optimized(available_sample_rates[3]); // 1000 Hz
        
        Self {
            info,
            config,
            available_sample_rates,
            tracking_state: TrackingState::Idle,
            last_data: None,
            power_consumption: 0.1,
            temperature: 30.0,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate temperature changes based on activity and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let activity_factor = if self.tracking_state == TrackingState::Tracking { 1.0 } else { 0.1 };
        
        // Temperature increases with activity and time
        self.temperature += elapsed * activity_factor * 0.05;
        
        // Temperature decreases over time (cooling)
        self.temperature -= elapsed * 0.01;
        
        // Clamp temperature to reasonable range
        self.temperature = self.temperature.clamp(25.0, 50.0);
        
        // Power consumption varies with activity and sample rate
        let sample_rate_factor = self.config.sample_rate.rate as f32 / 1000.0;
        self.power_consumption = 0.05 + (activity_factor * sample_rate_factor * 0.1);
        
        self.last_update = Instant::now();
    }
    
    /// Simulate new IMU data.
    fn simulate_data(&mut self) {
        if self.tracking_state == TrackingState::Tracking {
            let now = chrono::Utc::now();
            let timestamp = now.timestamp_nanos_opt().unwrap_or(0) as u64;
            
            // Generate random data
            let data = IMUData {
                timestamp,
                acceleration: (
                    rand::random::<f32>() * 2.0 - 1.0, // -1 to 1
                    rand::random::<f32>() * 2.0 - 1.0,
                    rand::random::<f32>() * 2.0 - 1.0,
                ),
                gyroscope: (
                    rand::random::<f32>() * 10.0 - 5.0, // -5 to 5
                    rand::random::<f32>() * 10.0 - 5.0,
                    rand::random::<f32>() * 10.0 - 5.0,
                ),
                magnetometer: Some((
                    rand::random::<f32>() * 100.0 - 50.0, // -50 to 50
                    rand::random::<f32>() * 100.0 - 50.0,
                    rand::random::<f32>() * 100.0 - 50.0,
                )),
                orientation: Some((
                    rand::random::<f32>(), // 0 to 1
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                )),
                temperature: Some(self.temperature),
            };
            
            self.last_data = Some(data.clone());
            
            // Dispatch data event
            self.dispatch_event(DeviceEventType::Custom {
                name: "IMUData".to_string(),
                data: {
                    let mut map = HashMap::new();
                    map.insert("timestamp".to_string(), timestamp.to_string());
                    // Add other data fields if needed
                    map
                },
            });
        }
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRIMUDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR IMU: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR IMU: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR IMU: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = IMUConfig::vr_optimized(self.available_sample_rates[3]); // 1000 Hz
        self.tracking_state = TrackingState::Idle;
        self.last_data = None;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
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
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            tracking_state: self.tracking_state,
            last_data: self.last_data.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl TrackingDevice for VRIMUDevice {
    fn get_tracking_state(&self) -> DeviceResult<TrackingState> {
        Ok(self.tracking_state)
    }
    
    fn start_tracking(&mut self) -> DeviceResult<()> {
        info!("Starting tracking on VR IMU: {}", self.info.id);
        
        // Check if device is ready
        if self.info.state != DeviceState::Ready {
            return Err(DeviceError::InvalidState(format!(
                "Device is not ready: {:?}",
                self.info.state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Tracking;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStarted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn stop_tracking(&mut self) -> DeviceResult<()> {
        info!("Stopping tracking on VR IMU: {}", self.info.id);
        
        // Check if device is tracking
        if self.tracking_state != TrackingState::Tracking {
            return Err(DeviceError::InvalidState(format!(
                "Device is not tracking: {:?}",
                self.tracking_state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Idle;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStopped".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn get_tracking_mode(&self) -> DeviceResult<TrackingMode> {
        Ok(TrackingMode::SixDOF) // IMU provides 6DOF
    }
    
    fn set_tracking_mode(&mut self, _mode: TrackingMode) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Tracking mode cannot be set for IMU".to_string()))
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        Ok(self.temperature)
    }
    
    fn has_tracking_capability(&self, capability: TrackingCapability) -> DeviceResult<bool> {
        match capability {
            TrackingCapability::IMU => Ok(true),
            TrackingCapability::Camera => Ok(false),
            TrackingCapability::Controller => Ok(false),
            TrackingCapability::HapticFeedback => Ok(false),
            TrackingCapability::EyeTracking => Ok(false),
            TrackingCapability::HandTracking => Ok(false),
            TrackingCapability::FaceTracking => Ok(false),
        }
    }
    
    fn clone_tracking_box(&self) -> Box<dyn TrackingDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            tracking_state: self.tracking_state,
            last_data: self.last_data.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl IMUDevice for VRIMUDevice {
    fn get_imu_config(&self) -> DeviceResult<IMUConfig> {
        Ok(self.config.clone())
    }
    
    fn set_imu_config(&mut self, config: &IMUConfig) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&config.sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                config.sample_rate.rate
            )));
        }
        
        // Apply configuration
        self.config = config.clone();
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", config.sample_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "IMUConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("sample_rate".to_string(), format!("{} Hz", config.sample_rate.rate));
                data.insert("enable_accelerometer".to_string(), config.enable_accelerometer.to_string());
                data.insert("enable_gyroscope".to_string(), config.enable_gyroscope.to_string());
                data.insert("enable_magnetometer".to_string(), config.enable_magnetometer.to_string());
                data.insert("enable_orientation".to_string(), config.enable_orientation.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_sample_rates(&self) -> DeviceResult<Vec<IMUSampleRate>> {
        Ok(self.available_sample_rates.clone())
    }
    
    fn set_sample_rate(&mut self, sample_rate: IMUSampleRate) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                sample_rate.rate
            )));
        }
        
        // Update configuration
        self.config.sample_rate = sample_rate;
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", sample_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "IMUSampleRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("rate".to_string(), sample_rate.rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_latest_data(&mut self) -> DeviceResult<IMUData> {
        // Simulate new data if tracking
        self.simulate_data();
        
        match &self.last_data {
            Some(data) => Ok(data.clone()),
            None => Err(DeviceError::NotAvailable("No IMU data available".to_string())),
        }
    }
    
    fn calibrate(&mut self) -> DeviceResult<bool> {
        info!("Calibrating VR IMU: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Calibrating;
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CalibrationStarted".to_string(),
            data: HashMap::new(),
        });
        
        // Simulate calibration delay
        std::thread::sleep(Duration::from_millis(1000));
        
        // Update state
        self.info.state = previous_state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CalibrationCompleted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("success".to_string(), "true".to_string());
                data
            },
        });
        
        Ok(true)
    }
    
    fn clone_imu_box(&self) -> Box<dyn IMUDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            tracking_state: self.tracking_state,
            last_data: self.last_data.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

/// VR Camera device implementation.
#[derive(Debug)]
pub struct VRCameraDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Camera configuration
    config: CameraConfig,
    
    /// Available resolutions
    available_resolutions: Vec<CameraResolution>,
    
    /// Available frame formats
    available_formats: Vec<CameraFrameFormat>,
    
    /// Current tracking state
    tracking_state: TrackingState,
    
    /// Last camera frame
    last_frame: Option<CameraFrame>,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Temperature in Celsius
    temperature: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRCameraDevice {
    /// Create a new VRCameraDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Tracking,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::USB,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::ThermalManagement,
                DeviceCapability::Tracking,
                DeviceCapability::Camera,
            ],
            state: DeviceState::Connected,
            description: Some("VR Tracking Camera".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add tracking-specific properties
        info.properties.insert("tracking_type".to_string(), "camera".to_string());
        
        // Create available resolutions
        let available_resolutions = vec![
            CameraResolution::new(640, 480),
            CameraResolution::new(1280, 720),
            CameraResolution::new(1920, 1080),
        ];
        
        // Create available frame formats
        let available_formats = vec![
            CameraFrameFormat::MJPEG,
            CameraFrameFormat::YUYV,
            CameraFrameFormat::NV12,
            CameraFrameFormat::RGB24,
        ];
        
        // Create camera configuration
        let config = CameraConfig::vr_optimized(
            available_resolutions[1], // 1280x720
            30.0, // 30 FPS
            available_formats[0], // MJPEG
        );
        
        Self {
            info,
            config,
            available_resolutions,
            available_formats,
            tracking_state: TrackingState::Idle,
            last_frame: None,
            power_consumption: 1.5,
            temperature: 35.0,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate temperature changes based on activity and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let activity_factor = if self.tracking_state == TrackingState::Tracking { 1.0 } else { 0.1 };
        
        // Temperature increases with activity and time
        self.temperature += elapsed * activity_factor * 0.1;
        
        // Temperature decreases over time (cooling)
        self.temperature -= elapsed * 0.02;
        
        // Clamp temperature to reasonable range
        self.temperature = self.temperature.clamp(25.0, 60.0);
        
        // Power consumption varies with activity, resolution, and frame rate
        let resolution_factor = (self.config.resolution.width * self.config.resolution.height) as f32 / (1280.0 * 720.0);
        let frame_rate_factor = self.config.frame_rate / 30.0;
        self.power_consumption = 0.5 + (activity_factor * resolution_factor * frame_rate_factor * 1.5);
        
        self.last_update = Instant::now();
    }
    
    /// Simulate a new camera frame.
    fn simulate_frame(&mut self) {
        if self.tracking_state == TrackingState::Tracking {
            let now = chrono::Utc::now();
            let timestamp = now.timestamp_nanos_opt().unwrap_or(0) as u64;
            
            // Calculate frame size based on resolution and format
            let frame_size = match self.config.format {
                CameraFrameFormat::MJPEG => (self.config.resolution.width * self.config.resolution.height / 8) as usize, // Estimate
                CameraFrameFormat::YUYV => (self.config.resolution.width * self.config.resolution.height * 2) as usize,
                CameraFrameFormat::NV12 => (self.config.resolution.width * self.config.resolution.height * 3 / 2) as usize,
                CameraFrameFormat::RGB24 => (self.config.resolution.width * self.config.resolution.height * 3) as usize,
                CameraFrameFormat::Unknown => 0,
            };
            
            // Generate random frame data
            let frame_data = vec![0u8; frame_size];
            
            let frame = CameraFrame {
                timestamp,
                resolution: self.config.resolution,
                format: self.config.format,
                data: frame_data,
                metadata: HashMap::new(),
            };
            
            self.last_frame = Some(frame.clone());
            
            // Dispatch frame event
            self.dispatch_event(DeviceEventType::Custom {
                name: "CameraFrame".to_string(),
                data: {
                    let mut map = HashMap::new();
                    map.insert("timestamp".to_string(), timestamp.to_string());
                    map.insert("width".to_string(), frame.resolution.width.to_string());
                    map.insert("height".to_string(), frame.resolution.height.to_string());
                    map.insert("format".to_string(), format!("{:?}", frame.format));
                    map
                },
            });
        }
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRCameraDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Camera: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Camera: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Camera: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = CameraConfig::vr_optimized(
            self.available_resolutions[1], // 1280x720
            30.0, // 30 FPS
            self.available_formats[0], // MJPEG
        );
        self.tracking_state = TrackingState::Idle;
        self.last_frame = None;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
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
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_resolutions: self.available_resolutions.clone(),
            available_formats: self.available_formats.clone(),
            tracking_state: self.tracking_state,
            last_frame: self.last_frame.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl TrackingDevice for VRCameraDevice {
    fn get_tracking_state(&self) -> DeviceResult<TrackingState> {
        Ok(self.tracking_state)
    }
    
    fn start_tracking(&mut self) -> DeviceResult<()> {
        info!("Starting tracking on VR Camera: {}", self.info.id);
        
        // Check if device is ready
        if self.info.state != DeviceState::Ready {
            return Err(DeviceError::InvalidState(format!(
                "Device is not ready: {:?}",
                self.info.state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Tracking;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStarted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn stop_tracking(&mut self) -> DeviceResult<()> {
        info!("Stopping tracking on VR Camera: {}", self.info.id);
        
        // Check if device is tracking
        if self.tracking_state != TrackingState::Tracking {
            return Err(DeviceError::InvalidState(format!(
                "Device is not tracking: {:?}",
                self.tracking_state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Idle;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStopped".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn get_tracking_mode(&self) -> DeviceResult<TrackingMode> {
        Ok(TrackingMode::SixDOF) // Assuming camera provides 6DOF
    }
    
    fn set_tracking_mode(&mut self, _mode: TrackingMode) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Tracking mode cannot be set for camera".to_string()))
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        Ok(self.temperature)
    }
    
    fn has_tracking_capability(&self, capability: TrackingCapability) -> DeviceResult<bool> {
        match capability {
            TrackingCapability::IMU => Ok(false),
            TrackingCapability::Camera => Ok(true),
            TrackingCapability::Controller => Ok(false),
            TrackingCapability::HapticFeedback => Ok(false),
            TrackingCapability::EyeTracking => Ok(false),
            TrackingCapability::HandTracking => Ok(true), // Assume camera supports hand tracking
            TrackingCapability::FaceTracking => Ok(false),
        }
    }
    
    fn clone_tracking_box(&self) -> Box<dyn TrackingDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_resolutions: self.available_resolutions.clone(),
            available_formats: self.available_formats.clone(),
            tracking_state: self.tracking_state,
            last_frame: self.last_frame.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl CameraDevice for VRCameraDevice {
    fn get_camera_config(&self) -> DeviceResult<CameraConfig> {
        Ok(self.config.clone())
    }
    
    fn set_camera_config(&mut self, config: &CameraConfig) -> DeviceResult<()> {
        // Validate resolution
        if !self.available_resolutions.contains(&config.resolution) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported resolution: {}x{}",
                config.resolution.width, config.resolution.height
            )));
        }
        
        // Validate frame format
        if !self.available_formats.contains(&config.format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported frame format: {:?}",
                config.format
            )));
        }
        
        // Validate frame rate
        if config.frame_rate <= 0.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Invalid frame rate: {}",
                config.frame_rate
            )));
        }
        
        // Apply configuration
        self.config = config.clone();
        
        // Update properties
        self.info.properties.insert(
            "resolution".to_string(),
            format!("{}x{}", config.resolution.width, config.resolution.height),
        );
        self.info.properties.insert(
            "frame_rate".to_string(),
            format!("{} FPS", config.frame_rate),
        );
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", config.format),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CameraConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("resolution".to_string(), format!("{}x{}", config.resolution.width, config.resolution.height));
                data.insert("frame_rate".to_string(), format!("{} FPS", config.frame_rate));
                data.insert("format".to_string(), format!("{:?}", config.format));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_resolutions(&self) -> DeviceResult<Vec<CameraResolution>> {
        Ok(self.available_resolutions.clone())
    }
    
    fn get_available_formats(&self) -> DeviceResult<Vec<CameraFrameFormat>> {
        Ok(self.available_formats.clone())
    }
    
    fn set_resolution(&mut self, resolution: CameraResolution) -> DeviceResult<()> {
        // Validate resolution
        if !self.available_resolutions.contains(&resolution) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported resolution: {}x{}",
                resolution.width, resolution.height
            )));
        }
        
        // Update configuration
        self.config.resolution = resolution;
        
        // Update properties
        self.info.properties.insert(
            "resolution".to_string(),
            format!("{}x{}", resolution.width, resolution.height),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CameraResolutionChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("width".to_string(), resolution.width.to_string());
                data.insert("height".to_string(), resolution.height.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_frame_rate(&mut self, frame_rate: f32) -> DeviceResult<()> {
        // Validate frame rate
        if frame_rate <= 0.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Invalid frame rate: {}",
                frame_rate
            )));
        }
        
        // Update configuration
        self.config.frame_rate = frame_rate;
        
        // Update properties
        self.info.properties.insert(
            "frame_rate".to_string(),
            format!("{} FPS", frame_rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CameraFrameRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("frame_rate".to_string(), frame_rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_format(&mut self, format: CameraFrameFormat) -> DeviceResult<()> {
        // Validate frame format
        if !self.available_formats.contains(&format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported frame format: {:?}",
                format
            )));
        }
        
        // Update configuration
        self.config.format = format;
        
        // Update properties
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", format),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CameraFormatChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("format".to_string(), format!("{:?}", format));
                data
            },
        });
        
        Ok(())
    }
    
    fn capture_frame(&mut self) -> DeviceResult<CameraFrame> {
        // Simulate new frame if tracking
        self.simulate_frame();
        
        match &self.last_frame {
            Some(frame) => Ok(frame.clone()),
            None => Err(DeviceError::NotAvailable("No camera frame available".to_string())),
        }
    }
    
    fn start_streaming(&mut self) -> DeviceResult<()> {
        self.start_tracking()
    }
    
    fn stop_streaming(&mut self) -> DeviceResult<()> {
        self.stop_tracking()
    }
    
    fn is_streaming(&self) -> DeviceResult<bool> {
        Ok(self.tracking_state == TrackingState::Tracking)
    }
    
    fn clone_camera_box(&self) -> Box<dyn CameraDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_resolutions: self.available_resolutions.clone(),
            available_formats: self.available_formats.clone(),
            tracking_state: self.tracking_state,
            last_frame: self.last_frame.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

/// VR Controller device implementation.
#[derive(Debug)]
pub struct VRControllerDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Current tracking state
    tracking_state: TrackingState,
    
    /// Current controller state
    controller_state: ControllerState,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Battery level (0.0 - 1.0)
    battery_level: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRControllerDevice {
    /// Create a new VRControllerDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Tracking,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::Bluetooth,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::Tracking,
                DeviceCapability::Controller,
                DeviceCapability::HapticFeedback,
            ],
            state: DeviceState::Connected,
            description: Some("VR Controller".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add tracking-specific properties
        info.properties.insert("tracking_type".to_string(), "controller".to_string());
        
        // Create initial controller state
        let controller_state = ControllerState {
            timestamp: now.timestamp_nanos_opt().unwrap_or(0) as u64,
            position: (0.0, 0.0, 0.0),
            orientation: (1.0, 0.0, 0.0, 0.0), // Identity quaternion
            velocity: (0.0, 0.0, 0.0),
            angular_velocity: (0.0, 0.0, 0.0),
            buttons: HashMap::new(),
            axes: HashMap::new(),
            touch: HashMap::new(),
            battery_level: 1.0,
            is_charging: false,
            tracking_quality: 1.0,
        };
        
        Self {
            info,
            tracking_state: TrackingState::Idle,
            controller_state,
            power_consumption: 0.2,
            battery_level: 1.0,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate battery drain and power consumption
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let activity_factor = if self.tracking_state == TrackingState::Tracking { 1.0 } else { 0.1 };
        
        // Battery drains faster when tracking
        self.battery_level -= elapsed * activity_factor * 0.0001;
        self.battery_level = self.battery_level.clamp(0.0, 1.0);
        
        // Power consumption varies with activity
        self.power_consumption = 0.05 + (activity_factor * 0.2);
        
        self.last_update = Instant::now();
    }
    
    /// Simulate controller state changes.
    fn simulate_state(&mut self) {
        if self.tracking_state == TrackingState::Tracking {
            let now = chrono::Utc::now();
            let timestamp = now.timestamp_nanos_opt().unwrap_or(0) as u64;
            
            // Simulate movement
            self.controller_state.position.0 += (rand::random::<f32>() - 0.5) * 0.01;
            self.controller_state.position.1 += (rand::random::<f32>() - 0.5) * 0.01;
            self.controller_state.position.2 += (rand::random::<f32>() - 0.5) * 0.01;
            
            // Simulate button press
            if rand::random::<f32>() < 0.01 { // 1% chance per update
                let button_id = "trigger".to_string();
                let pressed = rand::random::<bool>();
                
                self.controller_state.buttons.insert(button_id.clone(), ControllerButtonState {
                    pressed,
                    touched: pressed, // Assume touched if pressed
                    value: if pressed { 1.0 } else { 0.0 },
                });
                
                // Dispatch input event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "ControllerInput".to_string(),
                    data: {
                        let mut map = HashMap::new();
                        map.insert("input_type".to_string(), "Button".to_string());
                        map.insert("input_id".to_string(), button_id);
                        map.insert("pressed".to_string(), pressed.to_string());
                        map
                    },
                });
            }
            
            // Update battery level in state
            self.controller_state.battery_level = self.battery_level;
            self.controller_state.timestamp = timestamp;
            
            // Dispatch state update event
            self.dispatch_event(DeviceEventType::Custom {
                name: "ControllerStateUpdate".to_string(),
                data: {
                    let mut map = HashMap::new();
                    map.insert("timestamp".to_string(), timestamp.to_string());
                    // Add other state fields if needed
                    map
                },
            });
        }
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRControllerDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Controller: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Controller: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Controller: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset state
        self.tracking_state = TrackingState::Idle;
        self.controller_state = ControllerState {
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            position: (0.0, 0.0, 0.0),
            orientation: (1.0, 0.0, 0.0, 0.0),
            velocity: (0.0, 0.0, 0.0),
            angular_velocity: (0.0, 0.0, 0.0),
            buttons: HashMap::new(),
            axes: HashMap::new(),
            touch: HashMap::new(),
            battery_level: 1.0,
            is_charging: false,
            tracking_quality: 1.0,
        };
        self.battery_level = 1.0;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
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
        Box::new(Self {
            info: self.info.clone(),
            tracking_state: self.tracking_state,
            controller_state: self.controller_state.clone(),
            power_consumption: self.power_consumption,
            battery_level: self.battery_level,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl TrackingDevice for VRControllerDevice {
    fn get_tracking_state(&self) -> DeviceResult<TrackingState> {
        Ok(self.tracking_state)
    }
    
    fn start_tracking(&mut self) -> DeviceResult<()> {
        info!("Starting tracking on VR Controller: {}", self.info.id);
        
        // Check if device is ready
        if self.info.state != DeviceState::Ready {
            return Err(DeviceError::InvalidState(format!(
                "Device is not ready: {:?}",
                self.info.state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Tracking;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStarted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn stop_tracking(&mut self) -> DeviceResult<()> {
        info!("Stopping tracking on VR Controller: {}", self.info.id);
        
        // Check if device is tracking
        if self.tracking_state != TrackingState::Tracking {
            return Err(DeviceError::InvalidState(format!(
                "Device is not tracking: {:?}",
                self.tracking_state
            )));
        }
        
        // Update tracking state
        self.tracking_state = TrackingState::Idle;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "TrackingStopped".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn get_tracking_mode(&self) -> DeviceResult<TrackingMode> {
        Ok(TrackingMode::SixDOF) // Controller provides 6DOF
    }
    
    fn set_tracking_mode(&mut self, _mode: TrackingMode) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Tracking mode cannot be set for controller".to_string()))
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        Err(DeviceError::UnsupportedOperation("Temperature sensing not available for controller".to_string()))
    }
    
    fn has_tracking_capability(&self, capability: TrackingCapability) -> DeviceResult<bool> {
        match capability {
            TrackingCapability::IMU => Ok(true), // Assume controller has IMU
            TrackingCapability::Camera => Ok(false),
            TrackingCapability::Controller => Ok(true),
            TrackingCapability::HapticFeedback => Ok(true),
            TrackingCapability::EyeTracking => Ok(false),
            TrackingCapability::HandTracking => Ok(false),
            TrackingCapability::FaceTracking => Ok(false),
        }
    }
    
    fn clone_tracking_box(&self) -> Box<dyn TrackingDevice> {
        Box::new(Self {
            info: self.info.clone(),
            tracking_state: self.tracking_state,
            controller_state: self.controller_state.clone(),
            power_consumption: self.power_consumption,
            battery_level: self.battery_level,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl ControllerDevice for VRControllerDevice {
    fn get_controller_state(&mut self) -> DeviceResult<ControllerState> {
        // Simulate state changes
        self.simulate_state();
        
        Ok(self.controller_state.clone())
    }
    
    fn trigger_haptic_feedback(&mut self, feedback: HapticFeedback) -> DeviceResult<()> {
        info!("Triggering haptic feedback on VR Controller: {} - Duration: {}ms, Intensity: {}", 
              self.info.id, feedback.duration_ms, feedback.intensity);
        
        // Check if device is ready
        if self.info.state != DeviceState::Ready {
            return Err(DeviceError::InvalidState(format!(
                "Device is not ready: {:?}",
                self.info.state
            )));
        }
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "HapticFeedbackTriggered".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("duration_ms".to_string(), feedback.duration_ms.to_string());
                data.insert("intensity".to_string(), feedback.intensity.to_string());
                data
            },
        });
        
        // Simulate haptic feedback delay
        std::thread::sleep(Duration::from_millis(feedback.duration_ms as u64));
        
        Ok(())
    }
    
    fn get_battery_level(&self) -> DeviceResult<f32> {
        Ok(self.battery_level)
    }
    
    fn is_charging(&self) -> DeviceResult<bool> {
        Ok(self.controller_state.is_charging)
    }
    
    fn clone_controller_box(&self) -> Box<dyn ControllerDevice> {
        Box::new(Self {
            info: self.info.clone(),
            tracking_state: self.tracking_state,
            controller_state: self.controller_state.clone(),
            power_consumption: self.power_consumption,
            battery_level: self.battery_level,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_imu_creation() {
        let imu = VRIMUDevice::new(
            "imu1".to_string(),
            "VR IMU".to_string(),
            "Test Manufacturer".to_string(),
            "IMU-VR-1000".to_string(),
        );
        
        let info = imu.info().unwrap();
        assert_eq!(info.id, "imu1");
        assert_eq!(info.name, "VR IMU");
        assert_eq!(info.device_type, DeviceType::Tracking);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "IMU-VR-1000");
        assert_eq!(info.bus_type, DeviceBus::I2C);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = imu.get_imu_config().unwrap();
        assert_eq!(config.sample_rate.rate, 1000);
    }
    
    #[test]
    fn test_camera_creation() {
        let camera = VRCameraDevice::new(
            "cam1".to_string(),
            "VR Camera".to_string(),
            "Test Manufacturer".to_string(),
            "CAM-VR-2000".to_string(),
        );
        
        let info = camera.info().unwrap();
        assert_eq!(info.id, "cam1");
        assert_eq!(info.name, "VR Camera");
        assert_eq!(info.device_type, DeviceType::Tracking);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "CAM-VR-2000");
        assert_eq!(info.bus_type, DeviceBus::USB);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = camera.get_camera_config().unwrap();
        assert_eq!(config.resolution.width, 1280);
        assert_eq!(config.resolution.height, 720);
        assert_eq!(config.frame_rate, 30.0);
        assert_eq!(config.format, CameraFrameFormat::MJPEG);
    }
    
    #[test]
    fn test_controller_creation() {
        let controller = VRControllerDevice::new(
            "ctrl1".to_string(),
            "VR Controller".to_string(),
            "Test Manufacturer".to_string(),
            "CTRL-VR-3000".to_string(),
        );
        
        let info = controller.info().unwrap();
        assert_eq!(info.id, "ctrl1");
        assert_eq!(info.name, "VR Controller");
        assert_eq!(info.device_type, DeviceType::Tracking);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "CTRL-VR-3000");
        assert_eq!(info.bus_type, DeviceBus::Bluetooth);
        assert_eq!(info.state, DeviceState::Connected);
        
        assert!(controller.has_tracking_capability(TrackingCapability::HapticFeedback).unwrap());
    }
    
    #[test]
    fn test_imu_tracking() {
        let mut imu = VRIMUDevice::new(
            "imu1".to_string(),
            "VR IMU".to_string(),
            "Test Manufacturer".to_string(),
            "IMU-VR-1000".to_string(),
        );
        
        imu.initialize().unwrap();
        
        assert_eq!(imu.get_tracking_state().unwrap(), TrackingState::Idle);
        
        imu.start_tracking().unwrap();
        assert_eq!(imu.get_tracking_state().unwrap(), TrackingState::Tracking);
        
        let data = imu.get_latest_data().unwrap();
        assert!(data.timestamp > 0);
        
        imu.stop_tracking().unwrap();
        assert_eq!(imu.get_tracking_state().unwrap(), TrackingState::Idle);
    }
    
    #[test]
    fn test_camera_streaming() {
        let mut camera = VRCameraDevice::new(
            "cam1".to_string(),
            "VR Camera".to_string(),
            "Test Manufacturer".to_string(),
            "CAM-VR-2000".to_string(),
        );
        
        camera.initialize().unwrap();
        
        assert_eq!(camera.is_streaming().unwrap(), false);
        
        camera.start_streaming().unwrap();
        assert_eq!(camera.is_streaming().unwrap(), true);
        
        let frame = camera.capture_frame().unwrap();
        assert!(frame.timestamp > 0);
        assert_eq!(frame.resolution.width, 1280);
        assert_eq!(frame.resolution.height, 720);
        assert_eq!(frame.format, CameraFrameFormat::MJPEG);
        
        camera.stop_streaming().unwrap();
        assert_eq!(camera.is_streaming().unwrap(), false);
    }
    
    #[test]
    fn test_controller_state_and_haptics() {
        let mut controller = VRControllerDevice::new(
            "ctrl1".to_string(),
            "VR Controller".to_string(),
            "Test Manufacturer".to_string(),
            "CTRL-VR-3000".to_string(),
        );
        
        controller.initialize().unwrap();
        controller.start_tracking().unwrap();
        
        let state = controller.get_controller_state().unwrap();
        assert!(state.timestamp > 0);
        assert_eq!(state.battery_level, 1.0);
        
        // Test haptics
        let feedback = HapticFeedback {
            duration_ms: 100,
            intensity: 0.8,
        };
        controller.trigger_haptic_feedback(feedback).unwrap();
    }
}
