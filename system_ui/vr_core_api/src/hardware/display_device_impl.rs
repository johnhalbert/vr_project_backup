//! Display device implementation for the Hardware Access API.
//!
//! This module provides concrete implementations of display devices for the VR headset,
//! including LCD and OLED displays with VR-specific optimizations.

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
use super::display::{
    DisplayCapability, DisplayColorMode, DisplayColorTemperature, DisplayConfig,
    DisplayDevice, DisplayRefreshRate, DisplayResolution,
};

/// VR LCD display device implementation.
#[derive(Debug)]
pub struct VRLCDDisplay {
    /// Device information
    info: DeviceInfo,
    
    /// Display configuration
    config: DisplayConfig,
    
    /// Available resolutions
    available_resolutions: Vec<DisplayResolution>,
    
    /// Available refresh rates
    available_refresh_rates: Vec<DisplayRefreshRate>,
    
    /// Physical dimensions in millimeters (width, height)
    physical_dimensions: (f32, f32),
    
    /// Panel type
    panel_type: String,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Temperature in Celsius
    temperature: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRLCDDisplay {
    /// Create a new VRLCDDisplay.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
        resolution: DisplayResolution,
        refresh_rate: DisplayRefreshRate,
        physical_dimensions: (f32, f32),
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Display,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::MIPI_DSI,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::ThermalManagement,
            ],
            state: DeviceState::Connected,
            description: Some("VR LCD Display".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add display-specific properties
        info.properties.insert("panel_type".to_string(), "LCD".to_string());
        info.properties.insert("resolution".to_string(), format!("{}x{}", resolution.width, resolution.height));
        info.properties.insert("refresh_rate".to_string(), format!("{} Hz", refresh_rate.rate));
        
        // Create available resolutions
        let available_resolutions = vec![
            resolution,
            DisplayResolution::new(resolution.width / 2, resolution.height / 2),
        ];
        
        // Create available refresh rates
        let available_refresh_rates = vec![
            refresh_rate,
            DisplayRefreshRate::fixed(60.0),
        ];
        
        // Create display configuration
        let config = DisplayConfig::vr_optimized(resolution, refresh_rate);
        
        Self {
            info,
            config,
            available_resolutions,
            available_refresh_rates,
            physical_dimensions,
            panel_type: "LCD".to_string(),
            power_consumption: 2.5,
            temperature: 35.0,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the display status.
    fn update_status(&mut self) {
        // Simulate temperature changes based on brightness and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let brightness_factor = self.config.brightness;
        
        // Temperature increases with brightness and time
        self.temperature += elapsed * brightness_factor * 0.01;
        
        // Temperature decreases over time (cooling)
        self.temperature -= elapsed * 0.005;
        
        // Clamp temperature to reasonable range
        self.temperature = self.temperature.clamp(25.0, 60.0);
        
        // Power consumption varies with brightness and refresh rate
        let refresh_rate_factor = self.config.refresh_rate.rate / 90.0;
        self.power_consumption = 1.5 + (brightness_factor * refresh_rate_factor);
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRLCDDisplay {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR LCD Display: {}", self.info.id);
        
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
        info!("Shutting down VR LCD Display: {}", self.info.id);
        
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
        info!("Resetting VR LCD Display: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = DisplayConfig::vr_optimized(
            self.available_resolutions[0],
            self.available_refresh_rates[0],
        );
        
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
            available_refresh_rates: self.available_refresh_rates.clone(),
            physical_dimensions: self.physical_dimensions,
            panel_type: self.panel_type.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl DisplayDevice for VRLCDDisplay {
    fn get_config(&self) -> DeviceResult<DisplayConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &DisplayConfig) -> DeviceResult<()> {
        // Validate resolution
        if !self.available_resolutions.contains(&config.resolution) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported resolution: {}x{}",
                config.resolution.width, config.resolution.height
            )));
        }
        
        // Validate refresh rate
        let mut valid_refresh_rate = false;
        for rate in &self.available_refresh_rates {
            if rate.rate == config.refresh_rate.rate {
                valid_refresh_rate = true;
                break;
            }
        }
        
        if !valid_refresh_rate {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported refresh rate: {} Hz",
                config.refresh_rate.rate
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
            "refresh_rate".to_string(),
            format!("{} Hz", config.refresh_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("resolution".to_string(), format!("{}x{}", config.resolution.width, config.resolution.height));
                data.insert("refresh_rate".to_string(), format!("{} Hz", config.refresh_rate.rate));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_resolutions(&self) -> DeviceResult<Vec<DisplayResolution>> {
        Ok(self.available_resolutions.clone())
    }
    
    fn get_available_refresh_rates(&self) -> DeviceResult<Vec<DisplayRefreshRate>> {
        Ok(self.available_refresh_rates.clone())
    }
    
    fn set_resolution(&mut self, resolution: DisplayResolution) -> DeviceResult<()> {
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
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ResolutionChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("width".to_string(), resolution.width.to_string());
                data.insert("height".to_string(), resolution.height.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_refresh_rate(&mut self, refresh_rate: DisplayRefreshRate) -> DeviceResult<()> {
        // Validate refresh rate
        let mut valid_refresh_rate = false;
        for rate in &self.available_refresh_rates {
            if rate.rate == refresh_rate.rate {
                valid_refresh_rate = true;
                break;
            }
        }
        
        if !valid_refresh_rate {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported refresh rate: {} Hz",
                refresh_rate.rate
            )));
        }
        
        // Update configuration
        self.config.refresh_rate = refresh_rate;
        
        // Update properties
        self.info.properties.insert(
            "refresh_rate".to_string(),
            format!("{} Hz", refresh_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "RefreshRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("rate".to_string(), refresh_rate.rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_brightness(&mut self, brightness: f32) -> DeviceResult<()> {
        // Validate brightness
        if !(0.0..=1.0).contains(&brightness) {
            return Err(DeviceError::InvalidParameter(format!(
                "Brightness must be between 0.0 and 1.0: {}",
                brightness
            )));
        }
        
        // Update configuration
        self.config.brightness = brightness;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "BrightnessChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("brightness".to_string(), brightness.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_contrast(&mut self, contrast: f32) -> DeviceResult<()> {
        // Validate contrast
        if !(0.0..=1.0).contains(&contrast) {
            return Err(DeviceError::InvalidParameter(format!(
                "Contrast must be between 0.0 and 1.0: {}",
                contrast
            )));
        }
        
        // Update configuration
        self.config.contrast = contrast;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ContrastChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("contrast".to_string(), contrast.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_gamma(&mut self, gamma: f32) -> DeviceResult<()> {
        // Validate gamma
        if !(1.0..=3.0).contains(&gamma) {
            return Err(DeviceError::InvalidParameter(format!(
                "Gamma must be between 1.0 and 3.0: {}",
                gamma
            )));
        }
        
        // Update configuration
        self.config.gamma = gamma;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "GammaChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("gamma".to_string(), gamma.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_color_mode(&mut self, color_mode: DisplayColorMode) -> DeviceResult<()> {
        // Update configuration
        self.config.color_mode = color_mode;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ColorModeChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("color_mode".to_string(), format!("{:?}", color_mode));
                data
            },
        });
        
        Ok(())
    }
    
    fn set_color_temperature(&mut self, color_temperature: DisplayColorTemperature) -> DeviceResult<()> {
        // Update configuration
        self.config.color_temperature = color_temperature;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ColorTemperatureChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("temperature".to_string(), color_temperature.temperature.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_saturation(&mut self, saturation: f32) -> DeviceResult<()> {
        // Validate saturation
        if !(0.0..=1.0).contains(&saturation) {
            return Err(DeviceError::InvalidParameter(format!(
                "Saturation must be between 0.0 and 1.0: {}",
                saturation
            )));
        }
        
        // Update configuration
        self.config.saturation = saturation;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SaturationChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("saturation".to_string(), saturation.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_sharpness(&mut self, sharpness: f32) -> DeviceResult<()> {
        // Validate sharpness
        if !(0.0..=1.0).contains(&sharpness) {
            return Err(DeviceError::InvalidParameter(format!(
                "Sharpness must be between 0.0 and 1.0: {}",
                sharpness
            )));
        }
        
        // Update configuration
        self.config.sharpness = sharpness;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SharpnessChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("sharpness".to_string(), sharpness.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_blue_light_filter(&mut self, level: f32) -> DeviceResult<()> {
        // Validate level
        if !(0.0..=1.0).contains(&level) {
            return Err(DeviceError::InvalidParameter(format!(
                "Blue light filter level must be between 0.0 and 1.0: {}",
                level
            )));
        }
        
        // Update configuration
        self.config.blue_light_filter = level;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "BlueLightFilterChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("level".to_string(), level.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_low_persistence(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update configuration
        self.config.low_persistence = enabled;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "LowPersistenceChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_local_dimming(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update configuration
        self.config.local_dimming = enabled;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "LocalDimmingChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_adaptive_brightness(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update configuration
        self.config.adaptive_brightness = enabled;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "AdaptiveBrightnessChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_physical_dimensions(&self) -> DeviceResult<(f32, f32)> {
        Ok(self.physical_dimensions)
    }
    
    fn get_pixel_density(&self) -> DeviceResult<f32> {
        let (width_mm, height_mm) = self.physical_dimensions;
        let width_inches = width_mm / 25.4;
        let height_inches = height_mm / 25.4;
        
        let diagonal_pixels = ((self.config.resolution.width.pow(2) + self.config.resolution.height.pow(2)) as f32).sqrt();
        let diagonal_inches = ((width_inches.powi(2) + height_inches.powi(2)) as f32).sqrt();
        
        Ok(diagonal_pixels / diagonal_inches)
    }
    
    fn get_panel_type(&self) -> DeviceResult<String> {
        Ok(self.panel_type.clone())
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        self.update_status();
        Ok(self.power_consumption)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        self.update_status();
        Ok(self.temperature)
    }
    
    fn calibrate(&mut self) -> DeviceResult<bool> {
        info!("Calibrating VR LCD Display: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Calibrating;
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CalibrationStarted".to_string(),
            data: HashMap::new(),
        });
        
        // Simulate calibration delay
        std::thread::sleep(Duration::from_millis(500));
        
        // Update configuration with calibrated values
        self.config.gamma = 2.2;
        self.config.contrast = 0.6;
        self.config.brightness = 0.8;
        self.config.saturation = 0.5;
        self.config.sharpness = 0.7;
        
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
    
    fn run_test_pattern(&mut self, pattern_type: &str) -> DeviceResult<()> {
        info!("Running test pattern '{}' on VR LCD Display: {}", pattern_type, self.info.id);
        
        // Validate pattern type
        match pattern_type {
            "color_bars" | "grid" | "solid_white" | "solid_black" | "solid_red" | "solid_green" | "solid_blue" => {
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestPatternStarted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("pattern".to_string(), pattern_type.to_string());
                        data
                    },
                });
                
                // Simulate test pattern delay
                std::thread::sleep(Duration::from_millis(200));
                
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestPatternCompleted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("pattern".to_string(), pattern_type.to_string());
                        data
                    },
                });
                
                Ok(())
            },
            _ => Err(DeviceError::InvalidParameter(format!(
                "Unsupported test pattern: {}",
                pattern_type
            ))),
        }
    }
    
    fn clone_display_box(&self) -> Box<dyn DisplayDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_resolutions: self.available_resolutions.clone(),
            available_refresh_rates: self.available_refresh_rates.clone(),
            physical_dimensions: self.physical_dimensions,
            panel_type: self.panel_type.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

/// VR OLED display device implementation.
#[derive(Debug)]
pub struct VROLEDDisplay {
    /// Device information
    info: DeviceInfo,
    
    /// Display configuration
    config: DisplayConfig,
    
    /// Available resolutions
    available_resolutions: Vec<DisplayResolution>,
    
    /// Available refresh rates
    available_refresh_rates: Vec<DisplayRefreshRate>,
    
    /// Physical dimensions in millimeters (width, height)
    physical_dimensions: (f32, f32),
    
    /// Panel type
    panel_type: String,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Temperature in Celsius
    temperature: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VROLEDDisplay {
    /// Create a new VROLEDDisplay.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
        resolution: DisplayResolution,
        refresh_rate: DisplayRefreshRate,
        physical_dimensions: (f32, f32),
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Display,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::MIPI_DSI,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::ThermalManagement,
            ],
            state: DeviceState::Connected,
            description: Some("VR OLED Display".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add display-specific properties
        info.properties.insert("panel_type".to_string(), "OLED".to_string());
        info.properties.insert("resolution".to_string(), format!("{}x{}", resolution.width, resolution.height));
        info.properties.insert("refresh_rate".to_string(), format!("{} Hz", refresh_rate.rate));
        
        // Create available resolutions
        let available_resolutions = vec![
            resolution,
            DisplayResolution::new(resolution.width / 2, resolution.height / 2),
        ];
        
        // Create available refresh rates
        let available_refresh_rates = vec![
            refresh_rate,
            DisplayRefreshRate::fixed(60.0),
            DisplayRefreshRate::fixed(90.0),
            DisplayRefreshRate::fixed(120.0),
        ];
        
        // Create display configuration
        let config = DisplayConfig::vr_optimized(resolution, refresh_rate);
        
        Self {
            info,
            config,
            available_resolutions,
            available_refresh_rates,
            physical_dimensions,
            panel_type: "OLED".to_string(),
            power_consumption: 2.0,
            temperature: 30.0,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the display status.
    fn update_status(&mut self) {
        // Simulate temperature changes based on brightness and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let brightness_factor = self.config.brightness;
        
        // Temperature increases with brightness and time
        self.temperature += elapsed * brightness_factor * 0.008;
        
        // Temperature decreases over time (cooling)
        self.temperature -= elapsed * 0.004;
        
        // Clamp temperature to reasonable range
        self.temperature = self.temperature.clamp(25.0, 55.0);
        
        // Power consumption varies with brightness and refresh rate
        let refresh_rate_factor = self.config.refresh_rate.rate / 90.0;
        self.power_consumption = 1.2 + (brightness_factor * refresh_rate_factor);
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

// Implement Device trait for VROLEDDisplay
// (Implementation similar to VRLCDDisplay, with OLED-specific optimizations)
impl Device for VROLEDDisplay {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR OLED Display: {}", self.info.id);
        
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
        info!("Shutting down VR OLED Display: {}", self.info.id);
        
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
        info!("Resetting VR OLED Display: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = DisplayConfig::vr_optimized(
            self.available_resolutions[0],
            self.available_refresh_rates[0],
        );
        
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
            available_refresh_rates: self.available_refresh_rates.clone(),
            physical_dimensions: self.physical_dimensions,
            panel_type: self.panel_type.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

// Implement DisplayDevice trait for VROLEDDisplay
// (Implementation similar to VRLCDDisplay, with OLED-specific optimizations)
impl DisplayDevice for VROLEDDisplay {
    fn get_config(&self) -> DeviceResult<DisplayConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &DisplayConfig) -> DeviceResult<()> {
        // Validate resolution
        if !self.available_resolutions.contains(&config.resolution) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported resolution: {}x{}",
                config.resolution.width, config.resolution.height
            )));
        }
        
        // Validate refresh rate
        let mut valid_refresh_rate = false;
        for rate in &self.available_refresh_rates {
            if rate.rate == config.refresh_rate.rate {
                valid_refresh_rate = true;
                break;
            }
        }
        
        if !valid_refresh_rate {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported refresh rate: {} Hz",
                config.refresh_rate.rate
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
            "refresh_rate".to_string(),
            format!("{} Hz", config.refresh_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("resolution".to_string(), format!("{}x{}", config.resolution.width, config.resolution.height));
                data.insert("refresh_rate".to_string(), format!("{} Hz", config.refresh_rate.rate));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_resolutions(&self) -> DeviceResult<Vec<DisplayResolution>> {
        Ok(self.available_resolutions.clone())
    }
    
    fn get_available_refresh_rates(&self) -> DeviceResult<Vec<DisplayRefreshRate>> {
        Ok(self.available_refresh_rates.clone())
    }
    
    fn set_resolution(&mut self, resolution: DisplayResolution) -> DeviceResult<()> {
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
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ResolutionChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("width".to_string(), resolution.width.to_string());
                data.insert("height".to_string(), resolution.height.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_refresh_rate(&mut self, refresh_rate: DisplayRefreshRate) -> DeviceResult<()> {
        // Validate refresh rate
        let mut valid_refresh_rate = false;
        for rate in &self.available_refresh_rates {
            if rate.rate == refresh_rate.rate {
                valid_refresh_rate = true;
                break;
            }
        }
        
        if !valid_refresh_rate {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported refresh rate: {} Hz",
                refresh_rate.rate
            )));
        }
        
        // Update configuration
        self.config.refresh_rate = refresh_rate;
        
        // Update properties
        self.info.properties.insert(
            "refresh_rate".to_string(),
            format!("{} Hz", refresh_rate.rate),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "RefreshRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("rate".to_string(), refresh_rate.rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_brightness(&mut self, brightness: f32) -> DeviceResult<()> {
        // Validate brightness
        if !(0.0..=1.0).contains(&brightness) {
            return Err(DeviceError::InvalidParameter(format!(
                "Brightness must be between 0.0 and 1.0: {}",
                brightness
            )));
        }
        
        // Update configuration
        self.config.brightness = brightness;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "BrightnessChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("brightness".to_string(), brightness.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    // Implement remaining DisplayDevice methods similar to VRLCDDisplay
    // with OLED-specific optimizations where appropriate
    
    fn set_contrast(&mut self, contrast: f32) -> DeviceResult<()> {
        // Validate contrast
        if !(0.0..=1.0).contains(&contrast) {
            return Err(DeviceError::InvalidParameter(format!(
                "Contrast must be between 0.0 and 1.0: {}",
                contrast
            )));
        }
        
        // Update configuration
        self.config.contrast = contrast;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ContrastChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("contrast".to_string(), contrast.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_gamma(&mut self, gamma: f32) -> DeviceResult<()> {
        // Validate gamma
        if !(1.0..=3.0).contains(&gamma) {
            return Err(DeviceError::InvalidParameter(format!(
                "Gamma must be between 1.0 and 3.0: {}",
                gamma
            )));
        }
        
        // Update configuration
        self.config.gamma = gamma;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "GammaChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("gamma".to_string(), gamma.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_color_mode(&mut self, color_mode: DisplayColorMode) -> DeviceResult<()> {
        // Update configuration
        self.config.color_mode = color_mode;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ColorModeChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("color_mode".to_string(), format!("{:?}", color_mode));
                data
            },
        });
        
        Ok(())
    }
    
    fn set_color_temperature(&mut self, color_temperature: DisplayColorTemperature) -> DeviceResult<()> {
        // Update configuration
        self.config.color_temperature = color_temperature;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ColorTemperatureChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("temperature".to_string(), color_temperature.temperature.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_physical_dimensions(&self) -> DeviceResult<(f32, f32)> {
        Ok(self.physical_dimensions)
    }
    
    fn get_pixel_density(&self) -> DeviceResult<f32> {
        let (width_mm, height_mm) = self.physical_dimensions;
        let width_inches = width_mm / 25.4;
        let height_inches = height_mm / 25.4;
        
        let diagonal_pixels = ((self.config.resolution.width.pow(2) + self.config.resolution.height.pow(2)) as f32).sqrt();
        let diagonal_inches = ((width_inches.powi(2) + height_inches.powi(2)) as f32).sqrt();
        
        Ok(diagonal_pixels / diagonal_inches)
    }
    
    fn get_panel_type(&self) -> DeviceResult<String> {
        Ok(self.panel_type.clone())
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        self.update_status();
        Ok(self.power_consumption)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        self.update_status();
        Ok(self.temperature)
    }
    
    fn calibrate(&mut self) -> DeviceResult<bool> {
        info!("Calibrating VR OLED Display: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Calibrating;
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "CalibrationStarted".to_string(),
            data: HashMap::new(),
        });
        
        // Simulate calibration delay
        std::thread::sleep(Duration::from_millis(500));
        
        // Update configuration with calibrated values
        self.config.gamma = 2.2;
        self.config.contrast = 0.7;
        self.config.brightness = 0.8;
        self.config.saturation = 0.6;
        self.config.sharpness = 0.7;
        
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
    
    fn run_test_pattern(&mut self, pattern_type: &str) -> DeviceResult<()> {
        info!("Running test pattern '{}' on VR OLED Display: {}", pattern_type, self.info.id);
        
        // Validate pattern type
        match pattern_type {
            "color_bars" | "grid" | "solid_white" | "solid_black" | "solid_red" | "solid_green" | "solid_blue" | "contrast" => {
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestPatternStarted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("pattern".to_string(), pattern_type.to_string());
                        data
                    },
                });
                
                // Simulate test pattern delay
                std::thread::sleep(Duration::from_millis(200));
                
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestPatternCompleted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("pattern".to_string(), pattern_type.to_string());
                        data
                    },
                });
                
                Ok(())
            },
            _ => Err(DeviceError::InvalidParameter(format!(
                "Unsupported test pattern: {}",
                pattern_type
            ))),
        }
    }
    
    fn clone_display_box(&self) -> Box<dyn DisplayDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_resolutions: self.available_resolutions.clone(),
            available_refresh_rates: self.available_refresh_rates.clone(),
            physical_dimensions: self.physical_dimensions,
            panel_type: self.panel_type.clone(),
            power_consumption: self.power_consumption,
            temperature: self.temperature,
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
    fn test_vrlcd_display_creation() {
        let display = VRLCDDisplay::new(
            "lcd1".to_string(),
            "VR LCD Display".to_string(),
            "Test Manufacturer".to_string(),
            "LCD-VR-2000".to_string(),
            DisplayResolution::new(1920, 1080),
            DisplayRefreshRate::fixed(90.0),
            (50.0, 30.0),
        );
        
        let info = display.info().unwrap();
        assert_eq!(info.id, "lcd1");
        assert_eq!(info.name, "VR LCD Display");
        assert_eq!(info.device_type, DeviceType::Display);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "LCD-VR-2000");
        assert_eq!(info.bus_type, DeviceBus::MIPI_DSI);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = display.get_config().unwrap();
        assert_eq!(config.resolution.width, 1920);
        assert_eq!(config.resolution.height, 1080);
        assert_eq!(config.refresh_rate.rate, 90.0);
    }
    
    #[test]
    fn test_vroled_display_creation() {
        let display = VROLEDDisplay::new(
            "oled1".to_string(),
            "VR OLED Display".to_string(),
            "Test Manufacturer".to_string(),
            "OLED-VR-3000".to_string(),
            DisplayResolution::new(2560, 1440),
            DisplayRefreshRate::fixed(120.0),
            (60.0, 35.0),
        );
        
        let info = display.info().unwrap();
        assert_eq!(info.id, "oled1");
        assert_eq!(info.name, "VR OLED Display");
        assert_eq!(info.device_type, DeviceType::Display);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "OLED-VR-3000");
        assert_eq!(info.bus_type, DeviceBus::MIPI_DSI);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = display.get_config().unwrap();
        assert_eq!(config.resolution.width, 2560);
        assert_eq!(config.resolution.height, 1440);
        assert_eq!(config.refresh_rate.rate, 120.0);
    }
    
    #[test]
    fn test_display_configuration() {
        let mut display = VRLCDDisplay::new(
            "lcd1".to_string(),
            "VR LCD Display".to_string(),
            "Test Manufacturer".to_string(),
            "LCD-VR-2000".to_string(),
            DisplayResolution::new(1920, 1080),
            DisplayRefreshRate::fixed(90.0),
            (50.0, 30.0),
        );
        
        // Test setting brightness
        display.set_brightness(0.5).unwrap();
        let config = display.get_config().unwrap();
        assert_eq!(config.brightness, 0.5);
        
        // Test setting invalid brightness
        assert!(display.set_brightness(1.5).is_err());
        
        // Test setting contrast
        display.set_contrast(0.7).unwrap();
        let config = display.get_config().unwrap();
        assert_eq!(config.contrast, 0.7);
        
        // Test setting color mode
        display.set_color_mode(DisplayColorMode::Cinema).unwrap();
        let config = display.get_config().unwrap();
        assert_eq!(config.color_mode, DisplayColorMode::Cinema);
    }
    
    #[test]
    fn test_display_events() {
        let mut display = VRLCDDisplay::new(
            "lcd1".to_string(),
            "VR LCD Display".to_string(),
            "Test Manufacturer".to_string(),
            "LCD-VR-2000".to_string(),
            DisplayResolution::new(1920, 1080),
            DisplayRefreshRate::fixed(90.0),
            (50.0, 30.0),
        );
        
        // Create a counter to track handler calls
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Register an event handler
        let handler: DeviceEventHandler = Box::new(move |event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        display.register_event_handler(handler).unwrap();
        
        // Trigger events
        display.initialize().unwrap();
        display.set_brightness(0.5).unwrap();
        display.set_contrast(0.7).unwrap();
        
        // Check that the handler was called
        assert!(counter.load(Ordering::SeqCst) > 0);
    }
}
