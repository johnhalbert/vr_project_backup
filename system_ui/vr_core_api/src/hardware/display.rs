//! Display device interface for the VR headset.
//!
//! This module provides the implementation of display devices for the VR headset,
//! including management of brightness, refresh rate, resolution, and other display properties.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};

/// Display device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayCapability {
    /// Variable refresh rate
    VariableRefreshRate,
    
    /// HDR support
    HDR,
    
    /// Low persistence
    LowPersistence,
    
    /// Local dimming
    LocalDimming,
    
    /// Adaptive brightness
    AdaptiveBrightness,
    
    /// Blue light filter
    BlueLightFilter,
    
    /// Color temperature adjustment
    ColorTemperature,
    
    /// Gamma adjustment
    GammaAdjustment,
    
    /// Contrast adjustment
    ContrastAdjustment,
    
    /// Saturation adjustment
    SaturationAdjustment,
    
    /// Sharpness adjustment
    SharpnessAdjustment,
    
    /// Custom capability
    Custom(u32),
}

/// Display resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayResolution {
    /// Width in pixels
    pub width: u32,
    
    /// Height in pixels
    pub height: u32,
}

impl DisplayResolution {
    /// Create a new DisplayResolution.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    
    /// Get the aspect ratio.
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    
    /// Get the total number of pixels.
    pub fn total_pixels(&self) -> u32 {
        self.width * self.height
    }
}

/// Display refresh rate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DisplayRefreshRate {
    /// Refresh rate in Hz
    pub rate: f32,
    
    /// Whether this is a variable refresh rate
    pub variable: bool,
    
    /// Minimum refresh rate (for variable refresh rate)
    pub min_rate: Option<f32>,
    
    /// Maximum refresh rate (for variable refresh rate)
    pub max_rate: Option<f32>,
}

impl DisplayRefreshRate {
    /// Create a new fixed DisplayRefreshRate.
    pub fn fixed(rate: f32) -> Self {
        Self {
            rate,
            variable: false,
            min_rate: None,
            max_rate: None,
        }
    }
    
    /// Create a new variable DisplayRefreshRate.
    pub fn variable(min_rate: f32, max_rate: f32) -> Self {
        Self {
            rate: max_rate,
            variable: true,
            min_rate: Some(min_rate),
            max_rate: Some(max_rate),
        }
    }
}

/// Display color mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayColorMode {
    /// Standard color mode
    Standard,
    
    /// Vivid color mode
    Vivid,
    
    /// Natural color mode
    Natural,
    
    /// Cinema color mode
    Cinema,
    
    /// Game color mode
    Game,
    
    /// Reading color mode
    Reading,
    
    /// Night color mode
    Night,
    
    /// Custom color mode
    Custom,
}

/// Display color temperature.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DisplayColorTemperature {
    /// Color temperature in Kelvin
    pub temperature: u32,
}

impl DisplayColorTemperature {
    /// Create a new DisplayColorTemperature.
    pub fn new(temperature: u32) -> Self {
        Self { temperature }
    }
    
    /// Warm color temperature (2700K).
    pub fn warm() -> Self {
        Self { temperature: 2700 }
    }
    
    /// Standard color temperature (6500K).
    pub fn standard() -> Self {
        Self { temperature: 6500 }
    }
    
    /// Cool color temperature (9300K).
    pub fn cool() -> Self {
        Self { temperature: 9300 }
    }
}

/// Display configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display resolution
    pub resolution: DisplayResolution,
    
    /// Display refresh rate
    pub refresh_rate: DisplayRefreshRate,
    
    /// Display brightness (0.0 - 1.0)
    pub brightness: f32,
    
    /// Display contrast (0.0 - 1.0)
    pub contrast: f32,
    
    /// Display gamma (typically 1.0 - 3.0)
    pub gamma: f32,
    
    /// Display color mode
    pub color_mode: DisplayColorMode,
    
    /// Display color temperature
    pub color_temperature: DisplayColorTemperature,
    
    /// Display saturation (0.0 - 1.0)
    pub saturation: f32,
    
    /// Display sharpness (0.0 - 1.0)
    pub sharpness: f32,
    
    /// Blue light filter level (0.0 - 1.0)
    pub blue_light_filter: f32,
    
    /// Low persistence mode enabled
    pub low_persistence: bool,
    
    /// Local dimming enabled
    pub local_dimming: bool,
    
    /// Adaptive brightness enabled
    pub adaptive_brightness: bool,
    
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl DisplayConfig {
    /// Create a new DisplayConfig with default values.
    pub fn new(resolution: DisplayResolution, refresh_rate: DisplayRefreshRate) -> Self {
        Self {
            resolution,
            refresh_rate,
            brightness: 0.8,
            contrast: 0.5,
            gamma: 2.2,
            color_mode: DisplayColorMode::Standard,
            color_temperature: DisplayColorTemperature::standard(),
            saturation: 0.5,
            sharpness: 0.5,
            blue_light_filter: 0.0,
            low_persistence: true,
            local_dimming: false,
            adaptive_brightness: false,
            custom_settings: HashMap::new(),
        }
    }
    
    /// Create a new DisplayConfig optimized for VR.
    pub fn vr_optimized(resolution: DisplayResolution, refresh_rate: DisplayRefreshRate) -> Self {
        Self {
            resolution,
            refresh_rate,
            brightness: 0.9,
            contrast: 0.6,
            gamma: 2.2,
            color_mode: DisplayColorMode::Game,
            color_temperature: DisplayColorTemperature::standard(),
            saturation: 0.6,
            sharpness: 0.7,
            blue_light_filter: 0.2,
            low_persistence: true,
            local_dimming: true,
            adaptive_brightness: false,
            custom_settings: HashMap::new(),
        }
    }
}

/// Display device trait.
pub trait DisplayDevice: Device {
    /// Get the display configuration.
    fn get_config(&self) -> DeviceResult<DisplayConfig>;
    
    /// Set the display configuration.
    fn set_config(&mut self, config: &DisplayConfig) -> DeviceResult<()>;
    
    /// Get the available display resolutions.
    fn get_available_resolutions(&self) -> DeviceResult<Vec<DisplayResolution>>;
    
    /// Get the available display refresh rates.
    fn get_available_refresh_rates(&self) -> DeviceResult<Vec<DisplayRefreshRate>>;
    
    /// Set the display resolution.
    fn set_resolution(&mut self, resolution: DisplayResolution) -> DeviceResult<()>;
    
    /// Set the display refresh rate.
    fn set_refresh_rate(&mut self, refresh_rate: DisplayRefreshRate) -> DeviceResult<()>;
    
    /// Set the display brightness.
    fn set_brightness(&mut self, brightness: f32) -> DeviceResult<()>;
    
    /// Set the display contrast.
    fn set_contrast(&mut self, contrast: f32) -> DeviceResult<()>;
    
    /// Set the display gamma.
    fn set_gamma(&mut self, gamma: f32) -> DeviceResult<()>;
    
    /// Set the display color mode.
    fn set_color_mode(&mut self, color_mode: DisplayColorMode) -> DeviceResult<()>;
    
    /// Set the display color temperature.
    fn set_color_temperature(&mut self, color_temperature: DisplayColorTemperature) -> DeviceResult<()>;
    
    /// Set the display saturation.
    fn set_saturation(&mut self, saturation: f32) -> DeviceResult<()>;
    
    /// Set the display sharpness.
    fn set_sharpness(&mut self, sharpness: f32) -> DeviceResult<()>;
    
    /// Set the blue light filter level.
    fn set_blue_light_filter(&mut self, level: f32) -> DeviceResult<()>;
    
    /// Enable or disable low persistence mode.
    fn set_low_persistence(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Enable or disable local dimming.
    fn set_local_dimming(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Enable or disable adaptive brightness.
    fn set_adaptive_brightness(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Get the display physical dimensions in millimeters.
    fn get_physical_dimensions(&self) -> DeviceResult<(f32, f32)>;
    
    /// Get the display pixel density in pixels per inch.
    fn get_pixel_density(&self) -> DeviceResult<f32>;
    
    /// Get the display panel type.
    fn get_panel_type(&self) -> DeviceResult<String>;
    
    /// Get the display power consumption in watts.
    fn get_power_consumption(&self) -> DeviceResult<f32>;
    
    /// Get the display temperature in Celsius.
    fn get_temperature(&self) -> DeviceResult<f32>;
    
    /// Perform display calibration.
    fn calibrate(&mut self) -> DeviceResult<bool>;
    
    /// Run display test pattern.
    fn run_test_pattern(&mut self, pattern_type: &str) -> DeviceResult<()>;
    
    /// Clone as DisplayDevice
    fn clone_display_box(&self) -> Box<dyn DisplayDevice>;
}

/// Display manager for managing multiple displays.
#[derive(Debug)]
pub struct DisplayManager {
    /// Display devices by ID
    displays: HashMap<String, Box<dyn DisplayDevice>>,
    
    /// Primary display ID
    primary_display_id: Option<String>,
    
    /// Secondary display ID
    secondary_display_id: Option<String>,
    
    /// Display synchronization enabled
    sync_enabled: bool,
}

impl DisplayManager {
    /// Create a new DisplayManager.
    pub fn new() -> Self {
        Self {
            displays: HashMap::new(),
            primary_display_id: None,
            secondary_display_id: None,
            sync_enabled: false,
        }
    }
    
    /// Initialize the display manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing DisplayManager");
        Ok(())
    }
    
    /// Shutdown the display manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down DisplayManager");
        
        // Shutdown all displays
        for (id, display) in &mut self.displays {
            info!("Shutting down display {}", id);
            
            if let Err(e) = display.shutdown() {
                warn!("Failed to shutdown display {}: {}", id, e);
            }
        }
        
        self.displays.clear();
        self.primary_display_id = None;
        self.secondary_display_id = None;
        
        Ok(())
    }
    
    /// Register a display device.
    pub fn register_display(
        &mut self,
        id: &str,
        display: Box<dyn DisplayDevice>,
    ) -> DeviceResult<()> {
        info!("Registering display {}", id);
        
        self.displays.insert(id.to_string(), display);
        
        // If this is the first display, set it as primary
        if self.primary_display_id.is_none() {
            self.set_primary_display(id)?;
        } else if self.secondary_display_id.is_none() {
            // If this is the second display, set it as secondary
            self.set_secondary_display(id)?;
        }
        
        Ok(())
    }
    
    /// Unregister a display device.
    pub fn unregister_display(&mut self, id: &str) -> DeviceResult<()> {
        info!("Unregistering display {}", id);
        
        if self.displays.remove(id).is_none() {
            return Err(DeviceError::NotFound(format!("Display {} not found", id)));
        }
        
        // Update primary/secondary display IDs if necessary
        if Some(id.to_string()) == self.primary_display_id {
            self.primary_display_id = None;
            
            // If there's a secondary display, make it primary
            if let Some(secondary_id) = self.secondary_display_id.take() {
                self.primary_display_id = Some(secondary_id);
            }
        } else if Some(id.to_string()) == self.secondary_display_id {
            self.secondary_display_id = None;
        }
        
        Ok(())
    }
    
    /// Get a display device.
    pub fn get_display(&self, id: &str) -> DeviceResult<Box<dyn DisplayDevice>> {
        match self.displays.get(id) {
            Some(display) => Ok(display.clone_display_box()),
            None => Err(DeviceError::NotFound(format!("Display {} not found", id)))
        }
    }
    
    /// Get all display devices.
    pub fn get_all_displays(&self) -> DeviceResult<Vec<DeviceInfo>> {
        let mut result = Vec::new();
        for display in self.displays.values() {
            result.push(display.info()?);
        }
        Ok(result)
    }
    
    /// Get the primary display device.
    pub fn get_primary_display(&self) -> DeviceResult<Box<dyn DisplayDevice>> {
        if let Some(id) = &self.primary_display_id {
            self.get_display(id)
        } else {
            Err(DeviceError::NotFound("No primary display set".to_string()))
        }
    }
    
    /// Get the secondary display device.
    pub fn get_secondary_display(&self) -> DeviceResult<Box<dyn DisplayDevice>> {
        if let Some(id) = &self.secondary_display_id {
            self.get_display(id)
        } else {
            Err(DeviceError::NotFound("No secondary display set".to_string()))
        }
    }
    
    /// Set the primary display device.
    pub fn set_primary_display(&mut self, id: &str) -> DeviceResult<()> {
        if !self.displays.contains_key(id) {
            return Err(DeviceError::NotFound(format!("Display {} not found", id)));
        }
        
        info!("Setting {} as primary display", id);
        
        // If the new primary display is currently the secondary display,
        // clear the secondary display ID
        if Some(id.to_string()) == self.secondary_display_id {
            self.secondary_display_id = None;
        }
        
        // If there's already a primary display, make it the secondary display
        if let Some(current_primary) = self.primary_display_id.take() {
            if self.secondary_display_id.is_none() && current_primary != id {
                self.secondary_display_id = Some(current_primary);
            }
        }
        
        self.primary_display_id = Some(id.to_string());
        Ok(())
    }
    
    /// Set the secondary display device.
    pub fn set_secondary_display(&mut self, id: &str) -> DeviceResult<()> {
        if !self.displays.contains_key(id) {
            return Err(DeviceError::NotFound(format!("Display {} not found", id)));
        }
        
        // Can't set the primary display as the secondary display
        if Some(id.to_string()) == self.primary_display_id {
            return Err(DeviceError::InvalidParameter(
                "Cannot set primary display as secondary display".to_string(),
            ));
        }
        
        info!("Setting {} as secondary display", id);
        self.secondary_display_id = Some(id.to_string());
        Ok(())
    }
    
    /// Enable or disable display synchronization.
    pub fn set_sync_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        info!("Setting display synchronization to {}", enabled);
        self.sync_enabled = enabled;
        
        // If enabling sync, ensure both displays have the same configuration
        if enabled {
            self.synchronize_displays()?;
        }
        
        Ok(())
    }
    
    /// Check if display synchronization is enabled.
    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled
    }
    
    /// Synchronize display configurations.
    pub fn synchronize_displays(&mut self) -> DeviceResult<()> {
        info!("Synchronizing displays");
        
        // Need both primary and secondary displays
        if self.primary_display_id.is_none() || self.secondary_display_id.is_none() {
            return Err(DeviceError::InvalidState(
                "Both primary and secondary displays must be set for synchronization".to_string(),
            ));
        }
        
        // Get the primary display configuration
        let primary_id = self.primary_display_id.as_ref().unwrap();
        let primary_display = self.get_display(primary_id)?;
        let primary_config = primary_display.get_config()?;
        
        // Apply the primary display configuration to the secondary display
        let secondary_id = self.secondary_display_id.as_ref().unwrap();
        let mut secondary_display = self.get_display(secondary_id)?;
        secondary_display.set_config(&primary_config)?;
        
        Ok(())
    }
    
    /// Get the number of displays.
    pub fn get_display_count(&self) -> usize {
        self.displays.len()
    }
    
    /// Check if a display exists.
    pub fn has_display(&self, id: &str) -> bool {
        self.displays.contains_key(id)
    }
    
    /// Set the configuration for all displays.
    pub fn set_all_displays_config(&mut self, config: &DisplayConfig) -> DeviceResult<()> {
        info!("Setting configuration for all displays");
        
        for (id, display) in &mut self.displays {
            info!("Setting configuration for display {}", id);
            display.set_config(config)?;
        }
        
        Ok(())
    }
    
    /// Set the brightness for all displays.
    pub fn set_all_displays_brightness(&mut self, brightness: f32) -> DeviceResult<()> {
        info!("Setting brightness {} for all displays", brightness);
        
        for (id, display) in &mut self.displays {
            info!("Setting brightness for display {}", id);
            display.set_brightness(brightness)?;
        }
        
        Ok(())
    }
    
    /// Set the refresh rate for all displays.
    pub fn set_all_displays_refresh_rate(&mut self, refresh_rate: DisplayRefreshRate) -> DeviceResult<()> {
        info!("Setting refresh rate {:?} for all displays", refresh_rate);
        
        for (id, display) in &mut self.displays {
            info!("Setting refresh rate for display {}", id);
            display.set_refresh_rate(refresh_rate)?;
        }
        
        Ok(())
    }
    
    /// Set the resolution for all displays.
    pub fn set_all_displays_resolution(&mut self, resolution: DisplayResolution) -> DeviceResult<()> {
        info!("Setting resolution {:?} for all displays", resolution);
        
        for (id, display) in &mut self.displays {
            info!("Setting resolution for display {}", id);
            display.set_resolution(resolution)?;
        }
        
        Ok(())
    }
    
    /// Run a test pattern on all displays.
    pub fn run_all_displays_test_pattern(&mut self, pattern_type: &str) -> DeviceResult<()> {
        info!("Running test pattern {} on all displays", pattern_type);
        
        for (id, display) in &mut self.displays {
            info!("Running test pattern on display {}", id);
            display.run_test_pattern(pattern_type)?;
        }
        
        Ok(())
    }
}

/// Mock display device for testing.
#[derive(Debug, Clone)]
pub struct MockDisplayDevice {
    /// Device info
    pub info: DeviceInfo,
    
    /// Device state
    pub state: DeviceState,
    
    /// Display configuration
    pub config: DisplayConfig,
    
    /// Available resolutions
    pub available_resolutions: Vec<DisplayResolution>,
    
    /// Available refresh rates
    pub available_refresh_rates: Vec<DisplayRefreshRate>,
    
    /// Physical dimensions in millimeters
    pub physical_dimensions: (f32, f32),
    
    /// Panel type
    pub panel_type: String,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Event handlers
    pub event_handlers: Vec<DeviceEventHandler>,
}

impl MockDisplayDevice {
    /// Create a new MockDisplayDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let mut info = DeviceInfo::new(
            id,
            name,
            DeviceType::Display,
            manufacturer,
            model,
            DeviceBus::Internal,
        );
        
        info.state = DeviceState::Connected;
        
        // Add display capabilities
        info.add_capability(DeviceCapability::PowerControl);
        
        // Create default configuration
        let resolution = DisplayResolution::new(1920, 1080);
        let refresh_rate = DisplayRefreshRate::fixed(60.0);
        let config = DisplayConfig::new(resolution, refresh_rate);
        
        // Create available resolutions
        let available_resolutions = vec![
            DisplayResolution::new(1280, 720),
            DisplayResolution::new(1920, 1080),
            DisplayResolution::new(2560, 1440),
            DisplayResolution::new(3840, 2160),
        ];
        
        // Create available refresh rates
        let available_refresh_rates = vec![
            DisplayRefreshRate::fixed(30.0),
            DisplayRefreshRate::fixed(60.0),
            DisplayRefreshRate::fixed(90.0),
            DisplayRefreshRate::fixed(120.0),
            DisplayRefreshRate::variable(30.0, 120.0),
        ];
        
        Self {
            info,
            state: DeviceState::Connected,
            config,
            available_resolutions,
            available_refresh_rates,
            physical_dimensions: (300.0, 170.0),
            panel_type: "OLED".to_string(),
            properties: HashMap::new(),
            event_handlers: Vec::new(),
        }
    }
    
    /// Fire an event.
    pub fn fire_event(&self, event_type: DeviceEventType) {
        let event = super::device::DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for MockDisplayDevice {
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

impl DisplayDevice for MockDisplayDevice {
    fn get_config(&self) -> DeviceResult<DisplayConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &DisplayConfig) -> DeviceResult<()> {
        self.config = config.clone();
        Ok(())
    }
    
    fn get_available_resolutions(&self) -> DeviceResult<Vec<DisplayResolution>> {
        Ok(self.available_resolutions.clone())
    }
    
    fn get_available_refresh_rates(&self) -> DeviceResult<Vec<DisplayRefreshRate>> {
        Ok(self.available_refresh_rates.clone())
    }
    
    fn set_resolution(&mut self, resolution: DisplayResolution) -> DeviceResult<()> {
        self.config.resolution = resolution;
        Ok(())
    }
    
    fn set_refresh_rate(&mut self, refresh_rate: DisplayRefreshRate) -> DeviceResult<()> {
        self.config.refresh_rate = refresh_rate;
        Ok(())
    }
    
    fn set_brightness(&mut self, brightness: f32) -> DeviceResult<()> {
        self.config.brightness = brightness;
        Ok(())
    }
    
    fn set_contrast(&mut self, contrast: f32) -> DeviceResult<()> {
        self.config.contrast = contrast;
        Ok(())
    }
    
    fn set_gamma(&mut self, gamma: f32) -> DeviceResult<()> {
        self.config.gamma = gamma;
        Ok(())
    }
    
    fn set_color_mode(&mut self, color_mode: DisplayColorMode) -> DeviceResult<()> {
        self.config.color_mode = color_mode;
        Ok(())
    }
    
    fn set_color_temperature(&mut self, color_temperature: DisplayColorTemperature) -> DeviceResult<()> {
        self.config.color_temperature = color_temperature;
        Ok(())
    }
    
    fn set_saturation(&mut self, saturation: f32) -> DeviceResult<()> {
        self.config.saturation = saturation;
        Ok(())
    }
    
    fn set_sharpness(&mut self, sharpness: f32) -> DeviceResult<()> {
        self.config.sharpness = sharpness;
        Ok(())
    }
    
    fn set_blue_light_filter(&mut self, level: f32) -> DeviceResult<()> {
        self.config.blue_light_filter = level;
        Ok(())
    }
    
    fn set_low_persistence(&mut self, enabled: bool) -> DeviceResult<()> {
        self.config.low_persistence = enabled;
        Ok(())
    }
    
    fn set_local_dimming(&mut self, enabled: bool) -> DeviceResult<()> {
        self.config.local_dimming = enabled;
        Ok(())
    }
    
    fn set_adaptive_brightness(&mut self, enabled: bool) -> DeviceResult<()> {
        self.config.adaptive_brightness = enabled;
        Ok(())
    }
    
    fn get_physical_dimensions(&self) -> DeviceResult<(f32, f32)> {
        Ok(self.physical_dimensions)
    }
    
    fn get_pixel_density(&self) -> DeviceResult<f32> {
        let (width_mm, height_mm) = self.physical_dimensions;
        let width_inches = width_mm / 25.4;
        let height_inches = height_mm / 25.4;
        let diagonal_inches = (width_inches.powi(2) + height_inches.powi(2)).sqrt();
        let diagonal_pixels = (self.config.resolution.width.pow(2) as f32
            + self.config.resolution.height.pow(2) as f32)
            .sqrt();
        Ok(diagonal_pixels / diagonal_inches)
    }
    
    fn get_panel_type(&self) -> DeviceResult<String> {
        Ok(self.panel_type.clone())
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        // Mock power consumption based on brightness
        Ok(5.0 + 15.0 * self.config.brightness)
    }
    
    fn get_temperature(&self) -> DeviceResult<f32> {
        // Mock temperature
        Ok(35.0 + 10.0 * self.config.brightness)
    }
    
    fn calibrate(&mut self) -> DeviceResult<bool> {
        self.fire_event(DeviceEventType::CalibrationStarted);
        self.fire_event(DeviceEventType::CalibrationProgress {
            progress: 50,
            status: "Calibrating display...".to_string(),
        });
        self.fire_event(DeviceEventType::CalibrationCompleted {
            success: true,
            status: "Display calibration completed successfully".to_string(),
        });
        Ok(true)
    }
    
    fn run_test_pattern(&mut self, pattern_type: &str) -> DeviceResult<()> {
        info!("Running test pattern: {}", pattern_type);
        Ok(())
    }
    
    fn clone_display_box(&self) -> Box<dyn DisplayDevice> {
        Box::new(self.clone())
    }
}
