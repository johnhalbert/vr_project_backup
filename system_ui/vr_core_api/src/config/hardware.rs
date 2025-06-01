//! Hardware configuration module for the VR Core API.
//!
//! This module provides comprehensive configuration management for all hardware
//! components of the VR headset, including display, audio, tracking, power,
//! storage, and peripheral devices.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::hardware::{
    DeviceCapability, DeviceInfo, DeviceType, DisplayDevice, AudioDevice, 
    TrackingDevice, PowerDevice, StorageDevice, NetworkDevice
};
use super::{ConfigError, ConfigResult, validation};

/// Hardware configuration manager.
#[derive(Debug)]
pub struct HardwareConfig {
    /// Display configuration
    display: RwLock<DisplayConfig>,
    
    /// Audio configuration
    audio: RwLock<AudioConfig>,
    
    /// Tracking configuration
    tracking: RwLock<TrackingConfig>,
    
    /// Power configuration
    power: RwLock<PowerConfig>,
    
    /// Storage configuration
    storage: RwLock<StorageConfig>,
    
    /// Peripheral configuration
    peripherals: RwLock<PeripheralConfig>,
}

impl HardwareConfig {
    /// Create a new hardware configuration manager.
    pub fn new() -> Self {
        Self {
            display: RwLock::new(DisplayConfig::default()),
            audio: RwLock::new(AudioConfig::default()),
            tracking: RwLock::new(TrackingConfig::default()),
            power: RwLock::new(PowerConfig::default()),
            storage: RwLock::new(StorageConfig::default()),
            peripherals: RwLock::new(PeripheralConfig::default()),
        }
    }
    
    /// Load hardware configuration from TOML values.
    pub fn load_from_toml(&self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load display configuration
        if let Some(TomlValue::Table(display_table)) = config.get("display") {
            let mut display = self.display.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for display config".to_string())
            })?;
            display.load_from_toml(display_table)?;
        }
        
        // Load audio configuration
        if let Some(TomlValue::Table(audio_table)) = config.get("audio") {
            let mut audio = self.audio.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for audio config".to_string())
            })?;
            audio.load_from_toml(audio_table)?;
        }
        
        // Load tracking configuration
        if let Some(TomlValue::Table(tracking_table)) = config.get("tracking") {
            let mut tracking = self.tracking.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for tracking config".to_string())
            })?;
            tracking.load_from_toml(tracking_table)?;
        }
        
        // Load power configuration
        if let Some(TomlValue::Table(power_table)) = config.get("power") {
            let mut power = self.power.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for power config".to_string())
            })?;
            power.load_from_toml(power_table)?;
        }
        
        // Load storage configuration
        if let Some(TomlValue::Table(storage_table)) = config.get("storage") {
            let mut storage = self.storage.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for storage config".to_string())
            })?;
            storage.load_from_toml(storage_table)?;
        }
        
        // Load peripheral configuration
        if let Some(TomlValue::Table(peripherals_table)) = config.get("peripherals") {
            let mut peripherals = self.peripherals.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for peripherals config".to_string())
            })?;
            peripherals.load_from_toml(peripherals_table)?;
        }
        
        Ok(())
    }
    
    /// Save hardware configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save display configuration
        let display = self.display.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for display config".to_string())
        })?;
        config.insert("display".to_string(), TomlValue::Table(display.save_to_toml()?));
        
        // Save audio configuration
        let audio = self.audio.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for audio config".to_string())
        })?;
        config.insert("audio".to_string(), TomlValue::Table(audio.save_to_toml()?));
        
        // Save tracking configuration
        let tracking = self.tracking.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for tracking config".to_string())
        })?;
        config.insert("tracking".to_string(), TomlValue::Table(tracking.save_to_toml()?));
        
        // Save power configuration
        let power = self.power.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for power config".to_string())
        })?;
        config.insert("power".to_string(), TomlValue::Table(power.save_to_toml()?));
        
        // Save storage configuration
        let storage = self.storage.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for storage config".to_string())
        })?;
        config.insert("storage".to_string(), TomlValue::Table(storage.save_to_toml()?));
        
        // Save peripheral configuration
        let peripherals = self.peripherals.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for peripherals config".to_string())
        })?;
        config.insert("peripherals".to_string(), TomlValue::Table(peripherals.save_to_toml()?));
        
        Ok(config)
    }
    
    /// Get display configuration.
    pub fn display(&self) -> ConfigResult<DisplayConfig> {
        let display = self.display.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for display config".to_string())
        })?;
        Ok(display.clone())
    }
    
    /// Update display configuration.
    pub fn update_display(&self, config: DisplayConfig) -> ConfigResult<()> {
        let mut display = self.display.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for display config".to_string())
        })?;
        *display = config;
        Ok(())
    }
    
    /// Get audio configuration.
    pub fn audio(&self) -> ConfigResult<AudioConfig> {
        let audio = self.audio.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for audio config".to_string())
        })?;
        Ok(audio.clone())
    }
    
    /// Update audio configuration.
    pub fn update_audio(&self, config: AudioConfig) -> ConfigResult<()> {
        let mut audio = self.audio.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for audio config".to_string())
        })?;
        *audio = config;
        Ok(())
    }
    
    /// Get tracking configuration.
    pub fn tracking(&self) -> ConfigResult<TrackingConfig> {
        let tracking = self.tracking.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for tracking config".to_string())
        })?;
        Ok(tracking.clone())
    }
    
    /// Update tracking configuration.
    pub fn update_tracking(&self, config: TrackingConfig) -> ConfigResult<()> {
        let mut tracking = self.tracking.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for tracking config".to_string())
        })?;
        *tracking = config;
        Ok(())
    }
    
    /// Get power configuration.
    pub fn power(&self) -> ConfigResult<PowerConfig> {
        let power = self.power.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for power config".to_string())
        })?;
        Ok(power.clone())
    }
    
    /// Update power configuration.
    pub fn update_power(&self, config: PowerConfig) -> ConfigResult<()> {
        let mut power = self.power.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for power config".to_string())
        })?;
        *power = config;
        Ok(())
    }
    
    /// Get storage configuration.
    pub fn storage(&self) -> ConfigResult<StorageConfig> {
        let storage = self.storage.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for storage config".to_string())
        })?;
        Ok(storage.clone())
    }
    
    /// Update storage configuration.
    pub fn update_storage(&self, config: StorageConfig) -> ConfigResult<()> {
        let mut storage = self.storage.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for storage config".to_string())
        })?;
        *storage = config;
        Ok(())
    }
    
    /// Get peripheral configuration.
    pub fn peripherals(&self) -> ConfigResult<PeripheralConfig> {
        let peripherals = self.peripherals.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for peripherals config".to_string())
        })?;
        Ok(peripherals.clone())
    }
    
    /// Update peripheral configuration.
    pub fn update_peripherals(&self, config: PeripheralConfig) -> ConfigResult<()> {
        let mut peripherals = self.peripherals.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for peripherals config".to_string())
        })?;
        *peripherals = config;
        Ok(())
    }
    
    /// Apply hardware configuration to devices.
    pub fn apply_to_devices(&self, 
        display_devices: &[Arc<dyn DisplayDevice>],
        audio_devices: &[Arc<dyn AudioDevice>],
        tracking_devices: &[Arc<dyn TrackingDevice>],
        power_devices: &[Arc<dyn PowerDevice>],
        storage_devices: &[Arc<dyn StorageDevice>]
    ) -> ConfigResult<()> {
        // Apply display configuration
        let display_config = self.display()?;
        for device in display_devices {
            if let Err(e) = display_config.apply_to_device(device.as_ref()) {
                warn!("Failed to apply display config to device {}: {}", device.device_info().name, e);
            }
        }
        
        // Apply audio configuration
        let audio_config = self.audio()?;
        for device in audio_devices {
            if let Err(e) = audio_config.apply_to_device(device.as_ref()) {
                warn!("Failed to apply audio config to device {}: {}", device.device_info().name, e);
            }
        }
        
        // Apply tracking configuration
        let tracking_config = self.tracking()?;
        for device in tracking_devices {
            if let Err(e) = tracking_config.apply_to_device(device.as_ref()) {
                warn!("Failed to apply tracking config to device {}: {}", device.device_info().name, e);
            }
        }
        
        // Apply power configuration
        let power_config = self.power()?;
        for device in power_devices {
            if let Err(e) = power_config.apply_to_device(device.as_ref()) {
                warn!("Failed to apply power config to device {}: {}", device.device_info().name, e);
            }
        }
        
        // Apply storage configuration
        let storage_config = self.storage()?;
        for device in storage_devices {
            if let Err(e) = storage_config.apply_to_device(device.as_ref()) {
                warn!("Failed to apply storage config to device {}: {}", device.device_info().name, e);
            }
        }
        
        Ok(())
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Display configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Whether the display is enabled
    pub enabled: bool,
    
    /// Display brightness (0.0 - 1.0)
    pub brightness: f32,
    
    /// Display refresh rate in Hz
    pub refresh_rate: u32,
    
    /// Display resolution
    pub resolution: Resolution,
    
    /// Color settings
    pub color: ColorSettings,
    
    /// Distortion correction settings
    pub distortion: DistortionSettings,
    
    /// IPD (interpupillary distance) in millimeters
    pub ipd: f32,
    
    /// Eye relief distance in millimeters
    pub eye_relief: f32,
    
    /// Foveated rendering settings
    pub foveated_rendering: FoveatedRenderingSettings,
}

impl DisplayConfig {
    /// Load display configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load brightness
        if let Some(TomlValue::Float(brightness)) = config.get("brightness") {
            self.brightness = *brightness as f32;
            // Validate brightness range
            if self.brightness < 0.0 || self.brightness > 1.0 {
                return Err(ConfigError::ValidationError(
                    "Brightness must be between 0.0 and 1.0".to_string()
                ));
            }
        }
        
        // Load refresh rate
        if let Some(TomlValue::Integer(refresh_rate)) = config.get("refresh_rate") {
            self.refresh_rate = *refresh_rate as u32;
            // Validate refresh rate range
            if self.refresh_rate < 60 || self.refresh_rate > 144 {
                return Err(ConfigError::ValidationError(
                    "Refresh rate must be between 60 and 144 Hz".to_string()
                ));
            }
        }
        
        // Load resolution
        if let Some(TomlValue::Table(resolution_table)) = config.get("resolution") {
            if let Some(TomlValue::Integer(width)) = resolution_table.get("width") {
                self.resolution.width = *width as u32;
            }
            if let Some(TomlValue::Integer(height)) = resolution_table.get("height") {
                self.resolution.height = *height as u32;
            }
            
            // Validate resolution
            if self.resolution.width < 800 || self.resolution.height < 600 {
                return Err(ConfigError::ValidationError(
                    "Resolution must be at least 800x600".to_string()
                ));
            }
        }
        
        // Load color settings
        if let Some(TomlValue::Table(color_table)) = config.get("color") {
            if let Some(TomlValue::Float(contrast)) = color_table.get("contrast") {
                self.color.contrast = *contrast as f32;
            }
            if let Some(TomlValue::Float(gamma)) = color_table.get("gamma") {
                self.color.gamma = *gamma as f32;
            }
            if let Some(TomlValue::Float(saturation)) = color_table.get("saturation") {
                self.color.saturation = *saturation as f32;
            }
            if let Some(TomlValue::Boolean(night_mode)) = color_table.get("night_mode") {
                self.color.night_mode = *night_mode;
            }
        }
        
        // Load distortion settings
        if let Some(TomlValue::Table(distortion_table)) = config.get("distortion") {
            if let Some(TomlValue::Boolean(enabled)) = distortion_table.get("enabled") {
                self.distortion.enabled = *enabled;
            }
            if let Some(TomlValue::Float(k1)) = distortion_table.get("k1") {
                self.distortion.k1 = *k1 as f32;
            }
            if let Some(TomlValue::Float(k2)) = distortion_table.get("k2") {
                self.distortion.k2 = *k2 as f32;
            }
        }
        
        // Load IPD
        if let Some(TomlValue::Float(ipd)) = config.get("ipd") {
            self.ipd = *ipd as f32;
            // Validate IPD range
            if self.ipd < 55.0 || self.ipd > 75.0 {
                return Err(ConfigError::ValidationError(
                    "IPD must be between 55 and 75 mm".to_string()
                ));
            }
        }
        
        // Load eye relief
        if let Some(TomlValue::Float(eye_relief)) = config.get("eye_relief") {
            self.eye_relief = *eye_relief as f32;
            // Validate eye relief range
            if self.eye_relief < 10.0 || self.eye_relief > 30.0 {
                return Err(ConfigError::ValidationError(
                    "Eye relief must be between 10 and 30 mm".to_string()
                ));
            }
        }
        
        // Load foveated rendering settings
        if let Some(TomlValue::Table(foveated_table)) = config.get("foveated_rendering") {
            if let Some(TomlValue::Boolean(enabled)) = foveated_table.get("enabled") {
                self.foveated_rendering.enabled = *enabled;
            }
            if let Some(TomlValue::Float(strength)) = foveated_table.get("strength") {
                self.foveated_rendering.strength = *strength as f32;
            }
            if let Some(TomlValue::Float(falloff)) = foveated_table.get("falloff") {
                self.foveated_rendering.falloff = *falloff as f32;
            }
        }
        
        Ok(())
    }
    
    /// Save display configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save brightness
        config.insert("brightness".to_string(), TomlValue::Float(self.brightness as f64));
        
        // Save refresh rate
        config.insert("refresh_rate".to_string(), TomlValue::Integer(self.refresh_rate as i64));
        
        // Save resolution
        let mut resolution_table = HashMap::new();
        resolution_table.insert("width".to_string(), TomlValue::Integer(self.resolution.width as i64));
        resolution_table.insert("height".to_string(), TomlValue::Integer(self.resolution.height as i64));
        config.insert("resolution".to_string(), TomlValue::Table(resolution_table));
        
        // Save color settings
        let mut color_table = HashMap::new();
        color_table.insert("contrast".to_string(), TomlValue::Float(self.color.contrast as f64));
        color_table.insert("gamma".to_string(), TomlValue::Float(self.color.gamma as f64));
        color_table.insert("saturation".to_string(), TomlValue::Float(self.color.saturation as f64));
        color_table.insert("night_mode".to_string(), TomlValue::Boolean(self.color.night_mode));
        config.insert("color".to_string(), TomlValue::Table(color_table));
        
        // Save distortion settings
        let mut distortion_table = HashMap::new();
        distortion_table.insert("enabled".to_string(), TomlValue::Boolean(self.distortion.enabled));
        distortion_table.insert("k1".to_string(), TomlValue::Float(self.distortion.k1 as f64));
        distortion_table.insert("k2".to_string(), TomlValue::Float(self.distortion.k2 as f64));
        config.insert("distortion".to_string(), TomlValue::Table(distortion_table));
        
        // Save IPD
        config.insert("ipd".to_string(), TomlValue::Float(self.ipd as f64));
        
        // Save eye relief
        config.insert("eye_relief".to_string(), TomlValue::Float(self.eye_relief as f64));
        
        // Save foveated rendering settings
        let mut foveated_table = HashMap::new();
        foveated_table.insert("enabled".to_string(), TomlValue::Boolean(self.foveated_rendering.enabled));
        foveated_table.insert("strength".to_string(), TomlValue::Float(self.foveated_rendering.strength as f64));
        foveated_table.insert("falloff".to_string(), TomlValue::Float(self.foveated_rendering.falloff as f64));
        config.insert("foveated_rendering".to_string(), TomlValue::Table(foveated_table));
        
        Ok(config)
    }
    
    /// Apply display configuration to a display device.
    pub fn apply_to_device(&self, device: &dyn DisplayDevice) -> ConfigResult<()> {
        // Apply enabled state
        if let Err(e) = device.set_enabled(self.enabled) {
            return Err(ConfigError::DeviceError(format!("Failed to set enabled state: {}", e)));
        }
        
        // Apply brightness
        if let Err(e) = device.set_brightness(self.brightness) {
            return Err(ConfigError::DeviceError(format!("Failed to set brightness: {}", e)));
        }
        
        // Apply refresh rate
        if let Err(e) = device.set_refresh_rate(self.refresh_rate) {
            return Err(ConfigError::DeviceError(format!("Failed to set refresh rate: {}", e)));
        }
        
        // Apply resolution
        if let Err(e) = device.set_resolution(self.resolution.width, self.resolution.height) {
            return Err(ConfigError::DeviceError(format!("Failed to set resolution: {}", e)));
        }
        
        // Apply color settings
        if let Err(e) = device.set_contrast(self.color.contrast) {
            return Err(ConfigError::DeviceError(format!("Failed to set contrast: {}", e)));
        }
        if let Err(e) = device.set_gamma(self.color.gamma) {
            return Err(ConfigError::DeviceError(format!("Failed to set gamma: {}", e)));
        }
        
        // Apply distortion correction
        if device.capabilities().contains(&DeviceCapability::DistortionCorrection) {
            if let Err(e) = device.set_distortion_parameters(self.distortion.k1, self.distortion.k2) {
                return Err(ConfigError::DeviceError(format!("Failed to set distortion parameters: {}", e)));
            }
        }
        
        // Apply IPD if supported
        if device.capabilities().contains(&DeviceCapability::IPDAdjustment) {
            if let Err(e) = device.set_ipd(self.ipd) {
                return Err(ConfigError::DeviceError(format!("Failed to set IPD: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 0.8,
            refresh_rate: 90,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            color: ColorSettings {
                contrast: 1.0,
                gamma: 1.0,
                saturation: 1.0,
                night_mode: false,
            },
            distortion: DistortionSettings {
                enabled: true,
                k1: 0.22,
                k2: 0.24,
            },
            ipd: 63.5,
            eye_relief: 20.0,
            foveated_rendering: FoveatedRenderingSettings {
                enabled: false,
                strength: 0.5,
                falloff: 0.2,
            },
        }
    }
}

/// Display resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// Width in pixels
    pub width: u32,
    
    /// Height in pixels
    pub height: u32,
}

/// Color settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSettings {
    /// Contrast (0.0 - 2.0)
    pub contrast: f32,
    
    /// Gamma (0.5 - 2.5)
    pub gamma: f32,
    
    /// Saturation (0.0 - 2.0)
    pub saturation: f32,
    
    /// Night mode (blue light reduction)
    pub night_mode: bool,
}

/// Distortion correction settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistortionSettings {
    /// Whether distortion correction is enabled
    pub enabled: bool,
    
    /// Radial distortion coefficient k1
    pub k1: f32,
    
    /// Radial distortion coefficient k2
    pub k2: f32,
}

/// Foveated rendering settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoveatedRenderingSettings {
    /// Whether foveated rendering is enabled
    pub enabled: bool,
    
    /// Foveation strength (0.0 - 1.0)
    pub strength: f32,
    
    /// Foveation falloff (0.0 - 1.0)
    pub falloff: f32,
}

/// Audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Whether audio is enabled
    pub enabled: bool,
    
    /// Master volume (0.0 - 1.0)
    pub volume: f32,
    
    /// Whether audio is muted
    pub muted: bool,
    
    /// Whether spatial audio is enabled
    pub spatial_audio: bool,
    
    /// Microphone settings
    pub microphone: MicrophoneSettings,
    
    /// Equalizer settings
    pub equalizer: EqualizerSettings,
    
    /// Audio latency in milliseconds
    pub latency: u32,
}

impl AudioConfig {
    /// Load audio configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load volume
        if let Some(TomlValue::Float(volume)) = config.get("volume") {
            self.volume = *volume as f32;
            // Validate volume range
            if self.volume < 0.0 || self.volume > 1.0 {
                return Err(ConfigError::ValidationError(
                    "Volume must be between 0.0 and 1.0".to_string()
                ));
            }
        }
        
        // Load muted
        if let Some(TomlValue::Boolean(muted)) = config.get("muted") {
            self.muted = *muted;
        }
        
        // Load spatial audio
        if let Some(TomlValue::Boolean(spatial_audio)) = config.get("spatial_audio") {
            self.spatial_audio = *spatial_audio;
        }
        
        // Load microphone settings
        if let Some(TomlValue::Table(microphone_table)) = config.get("microphone") {
            if let Some(TomlValue::Boolean(enabled)) = microphone_table.get("enabled") {
                self.microphone.enabled = *enabled;
            }
            if let Some(TomlValue::Float(gain)) = microphone_table.get("gain") {
                self.microphone.gain = *gain as f32;
            }
            if let Some(TomlValue::Boolean(noise_suppression)) = microphone_table.get("noise_suppression") {
                self.microphone.noise_suppression = *noise_suppression;
            }
            if let Some(TomlValue::Boolean(echo_cancellation)) = microphone_table.get("echo_cancellation") {
                self.microphone.echo_cancellation = *echo_cancellation;
            }
            if let Some(TomlValue::Boolean(auto_gain)) = microphone_table.get("auto_gain") {
                self.microphone.auto_gain = *auto_gain;
            }
        }
        
        // Load equalizer settings
        if let Some(TomlValue::Table(equalizer_table)) = config.get("equalizer") {
            if let Some(TomlValue::Boolean(enabled)) = equalizer_table.get("enabled") {
                self.equalizer.enabled = *enabled;
            }
            if let Some(TomlValue::String(preset)) = equalizer_table.get("preset") {
                self.equalizer.preset = preset.clone();
            }
            if let Some(TomlValue::Array(bands)) = equalizer_table.get("bands") {
                self.equalizer.bands.clear();
                for (i, band) in bands.iter().enumerate() {
                    if let TomlValue::Float(gain) = band {
                        self.equalizer.bands.push(*gain as f32);
                    }
                }
            }
        }
        
        // Load latency
        if let Some(TomlValue::Integer(latency)) = config.get("latency") {
            self.latency = *latency as u32;
            // Validate latency range
            if self.latency < 10 || self.latency > 500 {
                return Err(ConfigError::ValidationError(
                    "Latency must be between 10 and 500 ms".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Save audio configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save volume
        config.insert("volume".to_string(), TomlValue::Float(self.volume as f64));
        
        // Save muted
        config.insert("muted".to_string(), TomlValue::Boolean(self.muted));
        
        // Save spatial audio
        config.insert("spatial_audio".to_string(), TomlValue::Boolean(self.spatial_audio));
        
        // Save microphone settings
        let mut microphone_table = HashMap::new();
        microphone_table.insert("enabled".to_string(), TomlValue::Boolean(self.microphone.enabled));
        microphone_table.insert("gain".to_string(), TomlValue::Float(self.microphone.gain as f64));
        microphone_table.insert("noise_suppression".to_string(), TomlValue::Boolean(self.microphone.noise_suppression));
        microphone_table.insert("echo_cancellation".to_string(), TomlValue::Boolean(self.microphone.echo_cancellation));
        microphone_table.insert("auto_gain".to_string(), TomlValue::Boolean(self.microphone.auto_gain));
        config.insert("microphone".to_string(), TomlValue::Table(microphone_table));
        
        // Save equalizer settings
        let mut equalizer_table = HashMap::new();
        equalizer_table.insert("enabled".to_string(), TomlValue::Boolean(self.equalizer.enabled));
        equalizer_table.insert("preset".to_string(), TomlValue::String(self.equalizer.preset.clone()));
        
        let bands: Vec<TomlValue> = self.equalizer.bands.iter()
            .map(|&gain| TomlValue::Float(gain as f64))
            .collect();
        equalizer_table.insert("bands".to_string(), TomlValue::Array(bands));
        
        config.insert("equalizer".to_string(), TomlValue::Table(equalizer_table));
        
        // Save latency
        config.insert("latency".to_string(), TomlValue::Integer(self.latency as i64));
        
        Ok(config)
    }
    
    /// Apply audio configuration to an audio device.
    pub fn apply_to_device(&self, device: &dyn AudioDevice) -> ConfigResult<()> {
        // Apply enabled state
        if let Err(e) = device.set_enabled(self.enabled) {
            return Err(ConfigError::DeviceError(format!("Failed to set enabled state: {}", e)));
        }
        
        // Apply volume
        if let Err(e) = device.set_volume(self.volume) {
            return Err(ConfigError::DeviceError(format!("Failed to set volume: {}", e)));
        }
        
        // Apply muted state
        if let Err(e) = device.set_muted(self.muted) {
            return Err(ConfigError::DeviceError(format!("Failed to set muted state: {}", e)));
        }
        
        // Apply spatial audio if supported
        if device.capabilities().contains(&DeviceCapability::SpatialAudio) {
            if let Err(e) = device.set_spatial_audio(self.spatial_audio) {
                return Err(ConfigError::DeviceError(format!("Failed to set spatial audio: {}", e)));
            }
        }
        
        // Apply microphone settings if this is a microphone
        if device.device_type() == DeviceType::Microphone {
            if let Err(e) = device.set_microphone_enabled(self.microphone.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set microphone enabled: {}", e)));
            }
            
            if let Err(e) = device.set_microphone_gain(self.microphone.gain) {
                return Err(ConfigError::DeviceError(format!("Failed to set microphone gain: {}", e)));
            }
            
            if device.capabilities().contains(&DeviceCapability::NoiseSuppression) {
                if let Err(e) = device.set_noise_suppression(self.microphone.noise_suppression) {
                    return Err(ConfigError::DeviceError(format!("Failed to set noise suppression: {}", e)));
                }
            }
            
            if device.capabilities().contains(&DeviceCapability::EchoCancellation) {
                if let Err(e) = device.set_echo_cancellation(self.microphone.echo_cancellation) {
                    return Err(ConfigError::DeviceError(format!("Failed to set echo cancellation: {}", e)));
                }
            }
        }
        
        // Apply equalizer settings if supported
        if device.capabilities().contains(&DeviceCapability::Equalizer) {
            if let Err(e) = device.set_equalizer_enabled(self.equalizer.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set equalizer enabled: {}", e)));
            }
            
            if let Err(e) = device.set_equalizer_preset(&self.equalizer.preset) {
                return Err(ConfigError::DeviceError(format!("Failed to set equalizer preset: {}", e)));
            }
            
            if let Err(e) = device.set_equalizer_bands(&self.equalizer.bands) {
                return Err(ConfigError::DeviceError(format!("Failed to set equalizer bands: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
            muted: false,
            spatial_audio: true,
            microphone: MicrophoneSettings {
                enabled: true,
                gain: 0.7,
                noise_suppression: true,
                echo_cancellation: true,
                auto_gain: true,
            },
            equalizer: EqualizerSettings {
                enabled: false,
                preset: "flat".to_string(),
                bands: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
            latency: 20,
        }
    }
}

/// Microphone settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneSettings {
    /// Whether the microphone is enabled
    pub enabled: bool,
    
    /// Microphone gain (0.0 - 1.0)
    pub gain: f32,
    
    /// Whether noise suppression is enabled
    pub noise_suppression: bool,
    
    /// Whether echo cancellation is enabled
    pub echo_cancellation: bool,
    
    /// Whether automatic gain control is enabled
    pub auto_gain: bool,
}

/// Equalizer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerSettings {
    /// Whether the equalizer is enabled
    pub enabled: bool,
    
    /// Equalizer preset name
    pub preset: String,
    
    /// Equalizer band gains (-1.0 to 1.0)
    pub bands: Vec<f32>,
}

/// Tracking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    /// Whether tracking is enabled
    pub enabled: bool,
    
    /// Tracking mode (3dof, 6dof)
    pub mode: String,
    
    /// Whether camera tracking is enabled
    pub camera_enabled: bool,
    
    /// Whether IMU tracking is enabled
    pub imu_enabled: bool,
    
    /// Tracking quality (low, medium, high)
    pub quality: String,
    
    /// Prediction time in milliseconds
    pub prediction_ms: u32,
    
    /// Tracking boundary settings
    pub boundary: BoundarySettings,
    
    /// Camera settings
    pub camera: CameraSettings,
    
    /// IMU settings
    pub imu: IMUSettings,
    
    /// Controller settings
    pub controller: ControllerSettings,
}

impl TrackingConfig {
    /// Load tracking configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load mode
        if let Some(TomlValue::String(mode)) = config.get("mode") {
            self.mode = mode.clone();
            // Validate mode
            if self.mode != "3dof" && self.mode != "6dof" {
                return Err(ConfigError::ValidationError(
                    "Tracking mode must be '3dof' or '6dof'".to_string()
                ));
            }
        }
        
        // Load camera enabled
        if let Some(TomlValue::Boolean(camera_enabled)) = config.get("camera_enabled") {
            self.camera_enabled = *camera_enabled;
        }
        
        // Load IMU enabled
        if let Some(TomlValue::Boolean(imu_enabled)) = config.get("imu_enabled") {
            self.imu_enabled = *imu_enabled;
        }
        
        // Load quality
        if let Some(TomlValue::String(quality)) = config.get("quality") {
            self.quality = quality.clone();
            // Validate quality
            if self.quality != "low" && self.quality != "medium" && self.quality != "high" {
                return Err(ConfigError::ValidationError(
                    "Tracking quality must be 'low', 'medium', or 'high'".to_string()
                ));
            }
        }
        
        // Load prediction time
        if let Some(TomlValue::Integer(prediction_ms)) = config.get("prediction_ms") {
            self.prediction_ms = *prediction_ms as u32;
            // Validate prediction time range
            if self.prediction_ms > 100 {
                return Err(ConfigError::ValidationError(
                    "Prediction time must be between 0 and 100 ms".to_string()
                ));
            }
        }
        
        // Load boundary settings
        if let Some(TomlValue::Table(boundary_table)) = config.get("boundary") {
            if let Some(TomlValue::Boolean(enabled)) = boundary_table.get("enabled") {
                self.boundary.enabled = *enabled;
            }
            if let Some(TomlValue::String(mode)) = boundary_table.get("mode") {
                self.boundary.mode = mode.clone();
            }
            if let Some(TomlValue::Float(visibility)) = boundary_table.get("visibility") {
                self.boundary.visibility = *visibility as f32;
            }
            if let Some(TomlValue::Float(proximity_warning)) = boundary_table.get("proximity_warning") {
                self.boundary.proximity_warning = *proximity_warning as f32;
            }
        }
        
        // Load camera settings
        if let Some(TomlValue::Table(camera_table)) = config.get("camera") {
            if let Some(TomlValue::Integer(exposure)) = camera_table.get("exposure") {
                self.camera.exposure = *exposure as u32;
            }
            if let Some(TomlValue::Integer(gain)) = camera_table.get("gain") {
                self.camera.gain = *gain as u32;
            }
            if let Some(TomlValue::Integer(fps)) = camera_table.get("fps") {
                self.camera.fps = *fps as u32;
            }
        }
        
        // Load IMU settings
        if let Some(TomlValue::Table(imu_table)) = config.get("imu") {
            if let Some(TomlValue::Integer(sample_rate)) = imu_table.get("sample_rate") {
                self.imu.sample_rate = *sample_rate as u32;
            }
            if let Some(TomlValue::Float(accel_filter)) = imu_table.get("accel_filter") {
                self.imu.accel_filter = *accel_filter as f32;
            }
            if let Some(TomlValue::Float(gyro_filter)) = imu_table.get("gyro_filter") {
                self.imu.gyro_filter = *gyro_filter as f32;
            }
        }
        
        // Load controller settings
        if let Some(TomlValue::Table(controller_table)) = config.get("controller") {
            if let Some(TomlValue::Boolean(enabled)) = controller_table.get("enabled") {
                self.controller.enabled = *enabled;
            }
            if let Some(TomlValue::String(hand_preference)) = controller_table.get("hand_preference") {
                self.controller.hand_preference = hand_preference.clone();
            }
            if let Some(TomlValue::Float(haptic_strength)) = controller_table.get("haptic_strength") {
                self.controller.haptic_strength = *haptic_strength as f32;
            }
        }
        
        Ok(())
    }
    
    /// Save tracking configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save mode
        config.insert("mode".to_string(), TomlValue::String(self.mode.clone()));
        
        // Save camera enabled
        config.insert("camera_enabled".to_string(), TomlValue::Boolean(self.camera_enabled));
        
        // Save IMU enabled
        config.insert("imu_enabled".to_string(), TomlValue::Boolean(self.imu_enabled));
        
        // Save quality
        config.insert("quality".to_string(), TomlValue::String(self.quality.clone()));
        
        // Save prediction time
        config.insert("prediction_ms".to_string(), TomlValue::Integer(self.prediction_ms as i64));
        
        // Save boundary settings
        let mut boundary_table = HashMap::new();
        boundary_table.insert("enabled".to_string(), TomlValue::Boolean(self.boundary.enabled));
        boundary_table.insert("mode".to_string(), TomlValue::String(self.boundary.mode.clone()));
        boundary_table.insert("visibility".to_string(), TomlValue::Float(self.boundary.visibility as f64));
        boundary_table.insert("proximity_warning".to_string(), TomlValue::Float(self.boundary.proximity_warning as f64));
        config.insert("boundary".to_string(), TomlValue::Table(boundary_table));
        
        // Save camera settings
        let mut camera_table = HashMap::new();
        camera_table.insert("exposure".to_string(), TomlValue::Integer(self.camera.exposure as i64));
        camera_table.insert("gain".to_string(), TomlValue::Integer(self.camera.gain as i64));
        camera_table.insert("fps".to_string(), TomlValue::Integer(self.camera.fps as i64));
        config.insert("camera".to_string(), TomlValue::Table(camera_table));
        
        // Save IMU settings
        let mut imu_table = HashMap::new();
        imu_table.insert("sample_rate".to_string(), TomlValue::Integer(self.imu.sample_rate as i64));
        imu_table.insert("accel_filter".to_string(), TomlValue::Float(self.imu.accel_filter as f64));
        imu_table.insert("gyro_filter".to_string(), TomlValue::Float(self.imu.gyro_filter as f64));
        config.insert("imu".to_string(), TomlValue::Table(imu_table));
        
        // Save controller settings
        let mut controller_table = HashMap::new();
        controller_table.insert("enabled".to_string(), TomlValue::Boolean(self.controller.enabled));
        controller_table.insert("hand_preference".to_string(), TomlValue::String(self.controller.hand_preference.clone()));
        controller_table.insert("haptic_strength".to_string(), TomlValue::Float(self.controller.haptic_strength as f64));
        config.insert("controller".to_string(), TomlValue::Table(controller_table));
        
        Ok(config)
    }
    
    /// Apply tracking configuration to a tracking device.
    pub fn apply_to_device(&self, device: &dyn TrackingDevice) -> ConfigResult<()> {
        // Apply enabled state
        if let Err(e) = device.set_enabled(self.enabled) {
            return Err(ConfigError::DeviceError(format!("Failed to set enabled state: {}", e)));
        }
        
        // Apply tracking mode if supported
        if device.capabilities().contains(&DeviceCapability::TrackingModeSelection) {
            if let Err(e) = device.set_tracking_mode(&self.mode) {
                return Err(ConfigError::DeviceError(format!("Failed to set tracking mode: {}", e)));
            }
        }
        
        // Apply camera settings if this is a camera
        if device.device_type() == DeviceType::Camera {
            if let Err(e) = device.set_camera_enabled(self.camera_enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set camera enabled: {}", e)));
            }
            
            if let Err(e) = device.set_camera_exposure(self.camera.exposure) {
                return Err(ConfigError::DeviceError(format!("Failed to set camera exposure: {}", e)));
            }
            
            if let Err(e) = device.set_camera_gain(self.camera.gain) {
                return Err(ConfigError::DeviceError(format!("Failed to set camera gain: {}", e)));
            }
            
            if let Err(e) = device.set_camera_fps(self.camera.fps) {
                return Err(ConfigError::DeviceError(format!("Failed to set camera FPS: {}", e)));
            }
        }
        
        // Apply IMU settings if this is an IMU
        if device.device_type() == DeviceType::IMU {
            if let Err(e) = device.set_imu_enabled(self.imu_enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set IMU enabled: {}", e)));
            }
            
            if let Err(e) = device.set_imu_sample_rate(self.imu.sample_rate) {
                return Err(ConfigError::DeviceError(format!("Failed to set IMU sample rate: {}", e)));
            }
            
            if let Err(e) = device.set_imu_filter_params(self.imu.accel_filter, self.imu.gyro_filter) {
                return Err(ConfigError::DeviceError(format!("Failed to set IMU filter parameters: {}", e)));
            }
        }
        
        // Apply prediction settings if supported
        if device.capabilities().contains(&DeviceCapability::MotionPrediction) {
            if let Err(e) = device.set_prediction_time(self.prediction_ms) {
                return Err(ConfigError::DeviceError(format!("Failed to set prediction time: {}", e)));
            }
        }
        
        // Apply boundary settings if supported
        if device.capabilities().contains(&DeviceCapability::BoundarySystem) {
            if let Err(e) = device.set_boundary_enabled(self.boundary.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set boundary enabled: {}", e)));
            }
            
            if let Err(e) = device.set_boundary_mode(&self.boundary.mode) {
                return Err(ConfigError::DeviceError(format!("Failed to set boundary mode: {}", e)));
            }
            
            if let Err(e) = device.set_boundary_visibility(self.boundary.visibility) {
                return Err(ConfigError::DeviceError(format!("Failed to set boundary visibility: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "6dof".to_string(),
            camera_enabled: true,
            imu_enabled: true,
            quality: "high".to_string(),
            prediction_ms: 20,
            boundary: BoundarySettings {
                enabled: true,
                mode: "room".to_string(),
                visibility: 0.7,
                proximity_warning: 0.3,
            },
            camera: CameraSettings {
                exposure: 16000,
                gain: 4,
                fps: 60,
            },
            imu: IMUSettings {
                sample_rate: 1000,
                accel_filter: 0.2,
                gyro_filter: 0.1,
            },
            controller: ControllerSettings {
                enabled: true,
                hand_preference: "right".to_string(),
                haptic_strength: 0.7,
            },
        }
    }
}

/// Boundary settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySettings {
    /// Whether the boundary is enabled
    pub enabled: bool,
    
    /// Boundary mode (room, stationary)
    pub mode: String,
    
    /// Boundary visibility (0.0 - 1.0)
    pub visibility: f32,
    
    /// Proximity warning threshold in meters
    pub proximity_warning: f32,
}

/// Camera settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    /// Camera exposure time in microseconds
    pub exposure: u32,
    
    /// Camera gain
    pub gain: u32,
    
    /// Camera frames per second
    pub fps: u32,
}

/// IMU settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMUSettings {
    /// IMU sample rate in Hz
    pub sample_rate: u32,
    
    /// Accelerometer filter strength (0.0 - 1.0)
    pub accel_filter: f32,
    
    /// Gyroscope filter strength (0.0 - 1.0)
    pub gyro_filter: f32,
}

/// Controller settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerSettings {
    /// Whether controllers are enabled
    pub enabled: bool,
    
    /// Hand preference (left, right)
    pub hand_preference: String,
    
    /// Haptic feedback strength (0.0 - 1.0)
    pub haptic_strength: f32,
}

/// Power configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Power mode (normal, performance, low_power)
    pub mode: String,
    
    /// Whether auto sleep is enabled
    pub auto_sleep: bool,
    
    /// Sleep timeout in seconds
    pub sleep_timeout_sec: u32,
    
    /// Whether to dim the display before sleep
    pub dim_before_sleep: bool,
    
    /// Display dim timeout in seconds
    pub dim_timeout_sec: u32,
    
    /// CPU power settings
    pub cpu: CPUPowerSettings,
    
    /// GPU power settings
    pub gpu: GPUPowerSettings,
    
    /// Thermal settings
    pub thermal: ThermalSettings,
    
    /// Battery settings
    pub battery: BatterySettings,
}

impl PowerConfig {
    /// Load power configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load mode
        if let Some(TomlValue::String(mode)) = config.get("mode") {
            self.mode = mode.clone();
            // Validate mode
            if self.mode != "normal" && self.mode != "performance" && self.mode != "low_power" {
                return Err(ConfigError::ValidationError(
                    "Power mode must be 'normal', 'performance', or 'low_power'".to_string()
                ));
            }
        }
        
        // Load auto sleep
        if let Some(TomlValue::Boolean(auto_sleep)) = config.get("auto_sleep") {
            self.auto_sleep = *auto_sleep;
        }
        
        // Load sleep timeout
        if let Some(TomlValue::Integer(sleep_timeout_sec)) = config.get("sleep_timeout_sec") {
            self.sleep_timeout_sec = *sleep_timeout_sec as u32;
        }
        
        // Load dim before sleep
        if let Some(TomlValue::Boolean(dim_before_sleep)) = config.get("dim_before_sleep") {
            self.dim_before_sleep = *dim_before_sleep;
        }
        
        // Load dim timeout
        if let Some(TomlValue::Integer(dim_timeout_sec)) = config.get("dim_timeout_sec") {
            self.dim_timeout_sec = *dim_timeout_sec as u32;
        }
        
        // Load CPU settings
        if let Some(TomlValue::Table(cpu_table)) = config.get("cpu") {
            if let Some(TomlValue::Integer(min_freq)) = cpu_table.get("min_freq") {
                self.cpu.min_freq = *min_freq as u32;
            }
            if let Some(TomlValue::Integer(max_freq)) = cpu_table.get("max_freq") {
                self.cpu.max_freq = *max_freq as u32;
            }
            if let Some(TomlValue::String(governor)) = cpu_table.get("governor") {
                self.cpu.governor = governor.clone();
            }
        }
        
        // Load GPU settings
        if let Some(TomlValue::Table(gpu_table)) = config.get("gpu") {
            if let Some(TomlValue::Integer(min_freq)) = gpu_table.get("min_freq") {
                self.gpu.min_freq = *min_freq as u32;
            }
            if let Some(TomlValue::Integer(max_freq)) = gpu_table.get("max_freq") {
                self.gpu.max_freq = *max_freq as u32;
            }
            if let Some(TomlValue::Boolean(dynamic_freq)) = gpu_table.get("dynamic_freq") {
                self.gpu.dynamic_freq = *dynamic_freq;
            }
        }
        
        // Load thermal settings
        if let Some(TomlValue::Table(thermal_table)) = config.get("thermal") {
            if let Some(TomlValue::Integer(throttling_temp)) = thermal_table.get("throttling_temp") {
                self.thermal.throttling_temp = *throttling_temp as u32;
            }
            if let Some(TomlValue::Integer(critical_temp)) = thermal_table.get("critical_temp") {
                self.thermal.critical_temp = *critical_temp as u32;
            }
            if let Some(TomlValue::Boolean(fan_control)) = thermal_table.get("fan_control") {
                self.thermal.fan_control = *fan_control;
            }
            if let Some(TomlValue::Integer(fan_speed)) = thermal_table.get("fan_speed") {
                self.thermal.fan_speed = *fan_speed as u32;
            }
        }
        
        // Load battery settings
        if let Some(TomlValue::Table(battery_table)) = config.get("battery") {
            if let Some(TomlValue::Boolean(show_percentage)) = battery_table.get("show_percentage") {
                self.battery.show_percentage = *show_percentage;
            }
            if let Some(TomlValue::Integer(low_battery_warning)) = battery_table.get("low_battery_warning") {
                self.battery.low_battery_warning = *low_battery_warning as u32;
            }
            if let Some(TomlValue::Boolean(power_saving_mode)) = battery_table.get("power_saving_mode") {
                self.battery.power_saving_mode = *power_saving_mode;
            }
            if let Some(TomlValue::Integer(power_saving_threshold)) = battery_table.get("power_saving_threshold") {
                self.battery.power_saving_threshold = *power_saving_threshold as u32;
            }
        }
        
        Ok(())
    }
    
    /// Save power configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save mode
        config.insert("mode".to_string(), TomlValue::String(self.mode.clone()));
        
        // Save auto sleep
        config.insert("auto_sleep".to_string(), TomlValue::Boolean(self.auto_sleep));
        
        // Save sleep timeout
        config.insert("sleep_timeout_sec".to_string(), TomlValue::Integer(self.sleep_timeout_sec as i64));
        
        // Save dim before sleep
        config.insert("dim_before_sleep".to_string(), TomlValue::Boolean(self.dim_before_sleep));
        
        // Save dim timeout
        config.insert("dim_timeout_sec".to_string(), TomlValue::Integer(self.dim_timeout_sec as i64));
        
        // Save CPU settings
        let mut cpu_table = HashMap::new();
        cpu_table.insert("min_freq".to_string(), TomlValue::Integer(self.cpu.min_freq as i64));
        cpu_table.insert("max_freq".to_string(), TomlValue::Integer(self.cpu.max_freq as i64));
        cpu_table.insert("governor".to_string(), TomlValue::String(self.cpu.governor.clone()));
        config.insert("cpu".to_string(), TomlValue::Table(cpu_table));
        
        // Save GPU settings
        let mut gpu_table = HashMap::new();
        gpu_table.insert("min_freq".to_string(), TomlValue::Integer(self.gpu.min_freq as i64));
        gpu_table.insert("max_freq".to_string(), TomlValue::Integer(self.gpu.max_freq as i64));
        gpu_table.insert("dynamic_freq".to_string(), TomlValue::Boolean(self.gpu.dynamic_freq));
        config.insert("gpu".to_string(), TomlValue::Table(gpu_table));
        
        // Save thermal settings
        let mut thermal_table = HashMap::new();
        thermal_table.insert("throttling_temp".to_string(), TomlValue::Integer(self.thermal.throttling_temp as i64));
        thermal_table.insert("critical_temp".to_string(), TomlValue::Integer(self.thermal.critical_temp as i64));
        thermal_table.insert("fan_control".to_string(), TomlValue::Boolean(self.thermal.fan_control));
        thermal_table.insert("fan_speed".to_string(), TomlValue::Integer(self.thermal.fan_speed as i64));
        config.insert("thermal".to_string(), TomlValue::Table(thermal_table));
        
        // Save battery settings
        let mut battery_table = HashMap::new();
        battery_table.insert("show_percentage".to_string(), TomlValue::Boolean(self.battery.show_percentage));
        battery_table.insert("low_battery_warning".to_string(), TomlValue::Integer(self.battery.low_battery_warning as i64));
        battery_table.insert("power_saving_mode".to_string(), TomlValue::Boolean(self.battery.power_saving_mode));
        battery_table.insert("power_saving_threshold".to_string(), TomlValue::Integer(self.battery.power_saving_threshold as i64));
        config.insert("battery".to_string(), TomlValue::Table(battery_table));
        
        Ok(config)
    }
    
    /// Apply power configuration to a power device.
    pub fn apply_to_device(&self, device: &dyn PowerDevice) -> ConfigResult<()> {
        // Apply power mode
        if let Err(e) = device.set_power_mode(&self.mode) {
            return Err(ConfigError::DeviceError(format!("Failed to set power mode: {}", e)));
        }
        
        // Apply auto sleep settings
        if let Err(e) = device.set_auto_sleep(self.auto_sleep) {
            return Err(ConfigError::DeviceError(format!("Failed to set auto sleep: {}", e)));
        }
        
        if let Err(e) = device.set_sleep_timeout(self.sleep_timeout_sec) {
            return Err(ConfigError::DeviceError(format!("Failed to set sleep timeout: {}", e)));
        }
        
        // Apply CPU settings if supported
        if device.capabilities().contains(&DeviceCapability::CPUFrequencyControl) {
            if let Err(e) = device.set_cpu_freq_range(self.cpu.min_freq, self.cpu.max_freq) {
                return Err(ConfigError::DeviceError(format!("Failed to set CPU frequency range: {}", e)));
            }
            
            if let Err(e) = device.set_cpu_governor(&self.cpu.governor) {
                return Err(ConfigError::DeviceError(format!("Failed to set CPU governor: {}", e)));
            }
        }
        
        // Apply GPU settings if supported
        if device.capabilities().contains(&DeviceCapability::GPUFrequencyControl) {
            if let Err(e) = device.set_gpu_freq_range(self.gpu.min_freq, self.gpu.max_freq) {
                return Err(ConfigError::DeviceError(format!("Failed to set GPU frequency range: {}", e)));
            }
            
            if let Err(e) = device.set_gpu_dynamic_freq(self.gpu.dynamic_freq) {
                return Err(ConfigError::DeviceError(format!("Failed to set GPU dynamic frequency: {}", e)));
            }
        }
        
        // Apply thermal settings if supported
        if device.capabilities().contains(&DeviceCapability::ThermalControl) {
            if let Err(e) = device.set_thermal_thresholds(self.thermal.throttling_temp, self.thermal.critical_temp) {
                return Err(ConfigError::DeviceError(format!("Failed to set thermal thresholds: {}", e)));
            }
            
            if let Err(e) = device.set_fan_control(self.thermal.fan_control) {
                return Err(ConfigError::DeviceError(format!("Failed to set fan control: {}", e)));
            }
            
            if let Err(e) = device.set_fan_speed(self.thermal.fan_speed) {
                return Err(ConfigError::DeviceError(format!("Failed to set fan speed: {}", e)));
            }
        }
        
        // Apply battery settings if supported
        if device.capabilities().contains(&DeviceCapability::BatteryManagement) {
            if let Err(e) = device.set_low_battery_warning(self.battery.low_battery_warning) {
                return Err(ConfigError::DeviceError(format!("Failed to set low battery warning: {}", e)));
            }
            
            if let Err(e) = device.set_battery_power_saving(self.battery.power_saving_mode, self.battery.power_saving_threshold) {
                return Err(ConfigError::DeviceError(format!("Failed to set battery power saving: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            mode: "normal".to_string(),
            auto_sleep: true,
            sleep_timeout_sec: 300,
            dim_before_sleep: true,
            dim_timeout_sec: 60,
            cpu: CPUPowerSettings {
                min_freq: 300000,
                max_freq: 2000000,
                governor: "ondemand".to_string(),
            },
            gpu: GPUPowerSettings {
                min_freq: 200000,
                max_freq: 800000,
                dynamic_freq: true,
            },
            thermal: ThermalSettings {
                throttling_temp: 80,
                critical_temp: 90,
                fan_control: true,
                fan_speed: 50,
            },
            battery: BatterySettings {
                show_percentage: true,
                low_battery_warning: 15,
                power_saving_mode: true,
                power_saving_threshold: 20,
            },
        }
    }
}

/// CPU power settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUPowerSettings {
    /// Minimum CPU frequency in kHz
    pub min_freq: u32,
    
    /// Maximum CPU frequency in kHz
    pub max_freq: u32,
    
    /// CPU governor (ondemand, performance, powersave)
    pub governor: String,
}

/// GPU power settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUPowerSettings {
    /// Minimum GPU frequency in kHz
    pub min_freq: u32,
    
    /// Maximum GPU frequency in kHz
    pub max_freq: u32,
    
    /// Whether to use dynamic frequency scaling
    pub dynamic_freq: bool,
}

/// Thermal settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSettings {
    /// Temperature at which to start throttling in Celsius
    pub throttling_temp: u32,
    
    /// Critical temperature at which to shut down in Celsius
    pub critical_temp: u32,
    
    /// Whether to enable fan control
    pub fan_control: bool,
    
    /// Fan speed percentage (0-100)
    pub fan_speed: u32,
}

/// Battery settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySettings {
    /// Whether to show battery percentage
    pub show_percentage: bool,
    
    /// Battery percentage at which to show low battery warning
    pub low_battery_warning: u32,
    
    /// Whether to enable power saving mode at low battery
    pub power_saving_mode: bool,
    
    /// Battery percentage at which to enable power saving mode
    pub power_saving_threshold: u32,
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Whether automatic backup is enabled
    pub auto_backup: bool,
    
    /// Backup interval in days
    pub backup_interval_days: u32,
    
    /// Maximum number of backups to keep
    pub max_backups: u32,
    
    /// Whether to compress backups
    pub compress_backups: bool,
    
    /// Whether to encrypt backups
    pub encrypt_backups: bool,
    
    /// Cache settings
    pub cache: CacheSettings,
    
    /// Log settings
    pub logs: LogSettings,
    
    /// Storage quota settings
    pub quota: QuotaSettings,
}

impl StorageConfig {
    /// Load storage configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load auto backup
        if let Some(TomlValue::Boolean(auto_backup)) = config.get("auto_backup") {
            self.auto_backup = *auto_backup;
        }
        
        // Load backup interval
        if let Some(TomlValue::Integer(backup_interval_days)) = config.get("backup_interval_days") {
            self.backup_interval_days = *backup_interval_days as u32;
        }
        
        // Load max backups
        if let Some(TomlValue::Integer(max_backups)) = config.get("max_backups") {
            self.max_backups = *max_backups as u32;
        }
        
        // Load compress backups
        if let Some(TomlValue::Boolean(compress_backups)) = config.get("compress_backups") {
            self.compress_backups = *compress_backups;
        }
        
        // Load encrypt backups
        if let Some(TomlValue::Boolean(encrypt_backups)) = config.get("encrypt_backups") {
            self.encrypt_backups = *encrypt_backups;
        }
        
        // Load cache settings
        if let Some(TomlValue::Table(cache_table)) = config.get("cache") {
            if let Some(TomlValue::Integer(max_size_mb)) = cache_table.get("max_size_mb") {
                self.cache.max_size_mb = *max_size_mb as u32;
            }
            if let Some(TomlValue::Boolean(auto_clean)) = cache_table.get("auto_clean") {
                self.cache.auto_clean = *auto_clean;
            }
            if let Some(TomlValue::Integer(clean_threshold_mb)) = cache_table.get("clean_threshold_mb") {
                self.cache.clean_threshold_mb = *clean_threshold_mb as u32;
            }
            if let Some(TomlValue::Integer(max_age_days)) = cache_table.get("max_age_days") {
                self.cache.max_age_days = *max_age_days as u32;
            }
        }
        
        // Load log settings
        if let Some(TomlValue::Table(logs_table)) = config.get("logs") {
            if let Some(TomlValue::Integer(max_size_mb)) = logs_table.get("max_size_mb") {
                self.logs.max_size_mb = *max_size_mb as u32;
            }
            if let Some(TomlValue::Integer(rotation_count)) = logs_table.get("rotation_count") {
                self.logs.rotation_count = *rotation_count as u32;
            }
            if let Some(TomlValue::Integer(max_age_days)) = logs_table.get("max_age_days") {
                self.logs.max_age_days = *max_age_days as u32;
            }
        }
        
        // Load quota settings
        if let Some(TomlValue::Table(quota_table)) = config.get("quota") {
            if let Some(TomlValue::Boolean(enabled)) = quota_table.get("enabled") {
                self.quota.enabled = *enabled;
            }
            if let Some(TomlValue::Integer(app_data_max_mb)) = quota_table.get("app_data_max_mb") {
                self.quota.app_data_max_mb = *app_data_max_mb as u32;
            }
            if let Some(TomlValue::Integer(user_data_max_mb)) = quota_table.get("user_data_max_mb") {
                self.quota.user_data_max_mb = *user_data_max_mb as u32;
            }
            if let Some(TomlValue::Boolean(warn_on_threshold)) = quota_table.get("warn_on_threshold") {
                self.quota.warn_on_threshold = *warn_on_threshold;
            }
            if let Some(TomlValue::Integer(warning_threshold_percent)) = quota_table.get("warning_threshold_percent") {
                self.quota.warning_threshold_percent = *warning_threshold_percent as u32;
            }
        }
        
        Ok(())
    }
    
    /// Save storage configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save auto backup
        config.insert("auto_backup".to_string(), TomlValue::Boolean(self.auto_backup));
        
        // Save backup interval
        config.insert("backup_interval_days".to_string(), TomlValue::Integer(self.backup_interval_days as i64));
        
        // Save max backups
        config.insert("max_backups".to_string(), TomlValue::Integer(self.max_backups as i64));
        
        // Save compress backups
        config.insert("compress_backups".to_string(), TomlValue::Boolean(self.compress_backups));
        
        // Save encrypt backups
        config.insert("encrypt_backups".to_string(), TomlValue::Boolean(self.encrypt_backups));
        
        // Save cache settings
        let mut cache_table = HashMap::new();
        cache_table.insert("max_size_mb".to_string(), TomlValue::Integer(self.cache.max_size_mb as i64));
        cache_table.insert("auto_clean".to_string(), TomlValue::Boolean(self.cache.auto_clean));
        cache_table.insert("clean_threshold_mb".to_string(), TomlValue::Integer(self.cache.clean_threshold_mb as i64));
        cache_table.insert("max_age_days".to_string(), TomlValue::Integer(self.cache.max_age_days as i64));
        config.insert("cache".to_string(), TomlValue::Table(cache_table));
        
        // Save log settings
        let mut logs_table = HashMap::new();
        logs_table.insert("max_size_mb".to_string(), TomlValue::Integer(self.logs.max_size_mb as i64));
        logs_table.insert("rotation_count".to_string(), TomlValue::Integer(self.logs.rotation_count as i64));
        logs_table.insert("max_age_days".to_string(), TomlValue::Integer(self.logs.max_age_days as i64));
        config.insert("logs".to_string(), TomlValue::Table(logs_table));
        
        // Save quota settings
        let mut quota_table = HashMap::new();
        quota_table.insert("enabled".to_string(), TomlValue::Boolean(self.quota.enabled));
        quota_table.insert("app_data_max_mb".to_string(), TomlValue::Integer(self.quota.app_data_max_mb as i64));
        quota_table.insert("user_data_max_mb".to_string(), TomlValue::Integer(self.quota.user_data_max_mb as i64));
        quota_table.insert("warn_on_threshold".to_string(), TomlValue::Boolean(self.quota.warn_on_threshold));
        quota_table.insert("warning_threshold_percent".to_string(), TomlValue::Integer(self.quota.warning_threshold_percent as i64));
        config.insert("quota".to_string(), TomlValue::Table(quota_table));
        
        Ok(config)
    }
    
    /// Apply storage configuration to a storage device.
    pub fn apply_to_device(&self, device: &dyn StorageDevice) -> ConfigResult<()> {
        // Apply backup settings if supported
        if device.capabilities().contains(&DeviceCapability::BackupManagement) {
            if let Err(e) = device.set_auto_backup(self.auto_backup) {
                return Err(ConfigError::DeviceError(format!("Failed to set auto backup: {}", e)));
            }
            
            if let Err(e) = device.set_backup_interval(self.backup_interval_days) {
                return Err(ConfigError::DeviceError(format!("Failed to set backup interval: {}", e)));
            }
            
            if let Err(e) = device.set_max_backups(self.max_backups) {
                return Err(ConfigError::DeviceError(format!("Failed to set max backups: {}", e)));
            }
            
            if let Err(e) = device.set_backup_compression(self.compress_backups) {
                return Err(ConfigError::DeviceError(format!("Failed to set backup compression: {}", e)));
            }
            
            if let Err(e) = device.set_backup_encryption(self.encrypt_backups) {
                return Err(ConfigError::DeviceError(format!("Failed to set backup encryption: {}", e)));
            }
        }
        
        // Apply cache settings if supported
        if device.capabilities().contains(&DeviceCapability::CacheManagement) {
            if let Err(e) = device.set_cache_size(self.cache.max_size_mb) {
                return Err(ConfigError::DeviceError(format!("Failed to set cache size: {}", e)));
            }
            
            if let Err(e) = device.set_cache_auto_clean(self.cache.auto_clean, self.cache.clean_threshold_mb) {
                return Err(ConfigError::DeviceError(format!("Failed to set cache auto clean: {}", e)));
            }
            
            if let Err(e) = device.set_cache_max_age(self.cache.max_age_days) {
                return Err(ConfigError::DeviceError(format!("Failed to set cache max age: {}", e)));
            }
        }
        
        // Apply log settings if supported
        if device.capabilities().contains(&DeviceCapability::LogManagement) {
            if let Err(e) = device.set_log_max_size(self.logs.max_size_mb) {
                return Err(ConfigError::DeviceError(format!("Failed to set log max size: {}", e)));
            }
            
            if let Err(e) = device.set_log_rotation(self.logs.rotation_count) {
                return Err(ConfigError::DeviceError(format!("Failed to set log rotation: {}", e)));
            }
            
            if let Err(e) = device.set_log_max_age(self.logs.max_age_days) {
                return Err(ConfigError::DeviceError(format!("Failed to set log max age: {}", e)));
            }
        }
        
        // Apply quota settings if supported
        if device.capabilities().contains(&DeviceCapability::StorageQuota) {
            if let Err(e) = device.set_quota_enabled(self.quota.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set quota enabled: {}", e)));
            }
            
            if let Err(e) = device.set_app_data_quota(self.quota.app_data_max_mb) {
                return Err(ConfigError::DeviceError(format!("Failed to set app data quota: {}", e)));
            }
            
            if let Err(e) = device.set_user_data_quota(self.quota.user_data_max_mb) {
                return Err(ConfigError::DeviceError(format!("Failed to set user data quota: {}", e)));
            }
            
            if let Err(e) = device.set_quota_warning(self.quota.warn_on_threshold, self.quota.warning_threshold_percent) {
                return Err(ConfigError::DeviceError(format!("Failed to set quota warning: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_interval_days: 7,
            max_backups: 5,
            compress_backups: true,
            encrypt_backups: false,
            cache: CacheSettings {
                max_size_mb: 1024,
                auto_clean: true,
                clean_threshold_mb: 768,
                max_age_days: 30,
            },
            logs: LogSettings {
                max_size_mb: 100,
                rotation_count: 5,
                max_age_days: 90,
            },
            quota: QuotaSettings {
                enabled: true,
                app_data_max_mb: 4096,
                user_data_max_mb: 8192,
                warn_on_threshold: true,
                warning_threshold_percent: 90,
            },
        }
    }
}

/// Cache settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Maximum cache size in MB
    pub max_size_mb: u32,
    
    /// Whether to automatically clean the cache
    pub auto_clean: bool,
    
    /// Cache size threshold for cleaning in MB
    pub clean_threshold_mb: u32,
    
    /// Maximum age of cache items in days
    pub max_age_days: u32,
}

/// Log settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    /// Maximum log size in MB
    pub max_size_mb: u32,
    
    /// Number of log rotations to keep
    pub rotation_count: u32,
    
    /// Maximum age of logs in days
    pub max_age_days: u32,
}

/// Storage quota settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSettings {
    /// Whether storage quotas are enabled
    pub enabled: bool,
    
    /// Maximum app data size in MB
    pub app_data_max_mb: u32,
    
    /// Maximum user data size in MB
    pub user_data_max_mb: u32,
    
    /// Whether to warn when approaching quota
    pub warn_on_threshold: bool,
    
    /// Warning threshold as percentage of quota
    pub warning_threshold_percent: u32,
}

/// Peripheral configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralConfig {
    /// Controller settings
    pub controllers: ControllersSettings,
    
    /// External display settings
    pub external_display: ExternalDisplaySettings,
    
    /// USB device settings
    pub usb: USBSettings,
    
    /// Bluetooth device settings
    pub bluetooth: BluetoothSettings,
}

impl PeripheralConfig {
    /// Load peripheral configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load controller settings
        if let Some(TomlValue::Table(controllers_table)) = config.get("controllers") {
            if let Some(TomlValue::Boolean(enabled)) = controllers_table.get("enabled") {
                self.controllers.enabled = *enabled;
            }
            if let Some(TomlValue::String(primary_hand)) = controllers_table.get("primary_hand") {
                self.controllers.primary_hand = primary_hand.clone();
                // Validate primary hand
                if self.controllers.primary_hand != "left" && self.controllers.primary_hand != "right" {
                    return Err(ConfigError::ValidationError(
                        "Primary hand must be 'left' or 'right'".to_string()
                    ));
                }
            }
            if let Some(TomlValue::Float(haptic_strength)) = controllers_table.get("haptic_strength") {
                self.controllers.haptic_strength = *haptic_strength as f32;
                // Validate haptic strength range
                if self.controllers.haptic_strength < 0.0 || self.controllers.haptic_strength > 1.0 {
                    return Err(ConfigError::ValidationError(
                        "Haptic strength must be between 0.0 and 1.0".to_string()
                    ));
                }
            }
            if let Some(TomlValue::Boolean(auto_sleep)) = controllers_table.get("auto_sleep") {
                self.controllers.auto_sleep = *auto_sleep;
            }
            if let Some(TomlValue::Integer(sleep_timeout_sec)) = controllers_table.get("sleep_timeout_sec") {
                self.controllers.sleep_timeout_sec = *sleep_timeout_sec as u32;
            }
            if let Some(TomlValue::String(tracking_mode)) = controllers_table.get("tracking_mode") {
                self.controllers.tracking_mode = tracking_mode.clone();
                // Validate tracking mode
                if self.controllers.tracking_mode != "3dof" && self.controllers.tracking_mode != "6dof" {
                    return Err(ConfigError::ValidationError(
                        "Controller tracking mode must be '3dof' or '6dof'".to_string()
                    ));
                }
            }
        }
        
        // Load external display settings
        if let Some(TomlValue::Table(external_display_table)) = config.get("external_display") {
            if let Some(TomlValue::Boolean(enabled)) = external_display_table.get("enabled") {
                self.external_display.enabled = *enabled;
            }
            if let Some(TomlValue::String(mode)) = external_display_table.get("mode") {
                self.external_display.mode = mode.clone();
                // Validate mode
                if self.external_display.mode != "mirror" && self.external_display.mode != "extend" && self.external_display.mode != "second_screen" {
                    return Err(ConfigError::ValidationError(
                        "External display mode must be 'mirror', 'extend', or 'second_screen'".to_string()
                    ));
                }
            }
            if let Some(TomlValue::Integer(refresh_rate)) = external_display_table.get("refresh_rate") {
                self.external_display.refresh_rate = *refresh_rate as u32;
            }
            if let Some(TomlValue::Table(resolution_table)) = external_display_table.get("resolution") {
                if let Some(TomlValue::Integer(width)) = resolution_table.get("width") {
                    self.external_display.resolution.width = *width as u32;
                }
                if let Some(TomlValue::Integer(height)) = resolution_table.get("height") {
                    self.external_display.resolution.height = *height as u32;
                }
            }
        }
        
        // Load USB settings
        if let Some(TomlValue::Table(usb_table)) = config.get("usb") {
            if let Some(TomlValue::Boolean(enabled)) = usb_table.get("enabled") {
                self.usb.enabled = *enabled;
            }
            if let Some(TomlValue::String(mode)) = usb_table.get("mode") {
                self.usb.mode = mode.clone();
                // Validate mode
                if self.usb.mode != "mtp" && self.usb.mode != "storage" && self.usb.mode != "developer" {
                    return Err(ConfigError::ValidationError(
                        "USB mode must be 'mtp', 'storage', or 'developer'".to_string()
                    ));
                }
            }
            if let Some(TomlValue::Boolean(auto_connect)) = usb_table.get("auto_connect") {
                self.usb.auto_connect = *auto_connect;
            }
            if let Some(TomlValue::Boolean(charge_only)) = usb_table.get("charge_only") {
                self.usb.charge_only = *charge_only;
            }
        }
        
        // Load Bluetooth settings
        if let Some(TomlValue::Table(bluetooth_table)) = config.get("bluetooth") {
            if let Some(TomlValue::Boolean(enabled)) = bluetooth_table.get("enabled") {
                self.bluetooth.enabled = *enabled;
            }
            if let Some(TomlValue::Boolean(discoverable)) = bluetooth_table.get("discoverable") {
                self.bluetooth.discoverable = *discoverable;
            }
            if let Some(TomlValue::Integer(discoverable_timeout_sec)) = bluetooth_table.get("discoverable_timeout_sec") {
                self.bluetooth.discoverable_timeout_sec = *discoverable_timeout_sec as u32;
            }
            if let Some(TomlValue::Boolean(auto_connect_known)) = bluetooth_table.get("auto_connect_known") {
                self.bluetooth.auto_connect_known = *auto_connect_known;
            }
            if let Some(TomlValue::Array(allowed_devices)) = bluetooth_table.get("allowed_devices") {
                self.bluetooth.allowed_devices.clear();
                for device in allowed_devices {
                    if let TomlValue::String(device_str) = device {
                        self.bluetooth.allowed_devices.push(device_str.clone());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Save peripheral configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save controller settings
        let mut controllers_table = HashMap::new();
        controllers_table.insert("enabled".to_string(), TomlValue::Boolean(self.controllers.enabled));
        controllers_table.insert("primary_hand".to_string(), TomlValue::String(self.controllers.primary_hand.clone()));
        controllers_table.insert("haptic_strength".to_string(), TomlValue::Float(self.controllers.haptic_strength as f64));
        controllers_table.insert("auto_sleep".to_string(), TomlValue::Boolean(self.controllers.auto_sleep));
        controllers_table.insert("sleep_timeout_sec".to_string(), TomlValue::Integer(self.controllers.sleep_timeout_sec as i64));
        controllers_table.insert("tracking_mode".to_string(), TomlValue::String(self.controllers.tracking_mode.clone()));
        config.insert("controllers".to_string(), TomlValue::Table(controllers_table));
        
        // Save external display settings
        let mut external_display_table = HashMap::new();
        external_display_table.insert("enabled".to_string(), TomlValue::Boolean(self.external_display.enabled));
        external_display_table.insert("mode".to_string(), TomlValue::String(self.external_display.mode.clone()));
        external_display_table.insert("refresh_rate".to_string(), TomlValue::Integer(self.external_display.refresh_rate as i64));
        
        let mut resolution_table = HashMap::new();
        resolution_table.insert("width".to_string(), TomlValue::Integer(self.external_display.resolution.width as i64));
        resolution_table.insert("height".to_string(), TomlValue::Integer(self.external_display.resolution.height as i64));
        external_display_table.insert("resolution".to_string(), TomlValue::Table(resolution_table));
        
        config.insert("external_display".to_string(), TomlValue::Table(external_display_table));
        
        // Save USB settings
        let mut usb_table = HashMap::new();
        usb_table.insert("enabled".to_string(), TomlValue::Boolean(self.usb.enabled));
        usb_table.insert("mode".to_string(), TomlValue::String(self.usb.mode.clone()));
        usb_table.insert("auto_connect".to_string(), TomlValue::Boolean(self.usb.auto_connect));
        usb_table.insert("charge_only".to_string(), TomlValue::Boolean(self.usb.charge_only));
        config.insert("usb".to_string(), TomlValue::Table(usb_table));
        
        // Save Bluetooth settings
        let mut bluetooth_table = HashMap::new();
        bluetooth_table.insert("enabled".to_string(), TomlValue::Boolean(self.bluetooth.enabled));
        bluetooth_table.insert("discoverable".to_string(), TomlValue::Boolean(self.bluetooth.discoverable));
        bluetooth_table.insert("discoverable_timeout_sec".to_string(), TomlValue::Integer(self.bluetooth.discoverable_timeout_sec as i64));
        bluetooth_table.insert("auto_connect_known".to_string(), TomlValue::Boolean(self.bluetooth.auto_connect_known));
        
        let allowed_devices: Vec<TomlValue> = self.bluetooth.allowed_devices.iter()
            .map(|device| TomlValue::String(device.clone()))
            .collect();
        bluetooth_table.insert("allowed_devices".to_string(), TomlValue::Array(allowed_devices));
        
        config.insert("bluetooth".to_string(), TomlValue::Table(bluetooth_table));
        
        Ok(config)
    }
    
    /// Apply peripheral configuration to devices.
    pub fn apply_to_devices(&self) -> ConfigResult<()> {
        // Peripheral configuration is typically applied through the hardware manager
        // rather than directly to devices, as it may involve multiple device types.
        // This method is a placeholder for future implementation.
        Ok(())
    }
}

impl Default for PeripheralConfig {
    fn default() -> Self {
        Self {
            controllers: ControllersSettings {
                enabled: true,
                primary_hand: "right".to_string(),
                haptic_strength: 0.7,
                auto_sleep: true,
                sleep_timeout_sec: 300,
                tracking_mode: "6dof".to_string(),
            },
            external_display: ExternalDisplaySettings {
                enabled: false,
                mode: "mirror".to_string(),
                refresh_rate: 60,
                resolution: Resolution {
                    width: 1920,
                    height: 1080,
                },
            },
            usb: USBSettings {
                enabled: true,
                mode: "mtp".to_string(),
                auto_connect: true,
                charge_only: false,
            },
            bluetooth: BluetoothSettings {
                enabled: true,
                discoverable: false,
                discoverable_timeout_sec: 120,
                auto_connect_known: true,
                allowed_devices: Vec::new(),
            },
        }
    }
}

/// Controllers settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllersSettings {
    /// Whether controllers are enabled
    pub enabled: bool,
    
    /// Primary hand (left, right)
    pub primary_hand: String,
    
    /// Haptic feedback strength (0.0 - 1.0)
    pub haptic_strength: f32,
    
    /// Whether controllers auto sleep
    pub auto_sleep: bool,
    
    /// Controller sleep timeout in seconds
    pub sleep_timeout_sec: u32,
    
    /// Controller tracking mode (3dof, 6dof)
    pub tracking_mode: String,
}

/// External display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDisplaySettings {
    /// Whether external display is enabled
    pub enabled: bool,
    
    /// Display mode (mirror, extend, second_screen)
    pub mode: String,
    
    /// Refresh rate in Hz
    pub refresh_rate: u32,
    
    /// Resolution
    pub resolution: Resolution,
}

/// USB settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USBSettings {
    /// Whether USB is enabled
    pub enabled: bool,
    
    /// USB mode (mtp, storage, developer)
    pub mode: String,
    
    /// Whether to auto connect when plugged in
    pub auto_connect: bool,
    
    /// Whether to only charge (no data)
    pub charge_only: bool,
}

/// Bluetooth settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothSettings {
    /// Whether Bluetooth is enabled
    pub enabled: bool,
    
    /// Whether device is discoverable
    pub discoverable: bool,
    
    /// Discoverable timeout in seconds
    pub discoverable_timeout_sec: u32,
    
    /// Whether to auto connect to known devices
    pub auto_connect_known: bool,
    
    /// List of allowed device MAC addresses
    pub allowed_devices: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_config_load_save() {
        let mut config = DisplayConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("brightness".to_string(), TomlValue::Float(0.5));
        toml_values.insert("refresh_rate".to_string(), TomlValue::Integer(120));
        
        let mut resolution = HashMap::new();
        resolution.insert("width".to_string(), TomlValue::Integer(2560));
        resolution.insert("height".to_string(), TomlValue::Integer(1440));
        toml_values.insert("resolution".to_string(), TomlValue::Table(resolution));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, false);
        assert_eq!(config.brightness, 0.5);
        assert_eq!(config.refresh_rate, 120);
        assert_eq!(config.resolution.width, 2560);
        assert_eq!(config.resolution.height, 1440);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("brightness"), Some(&TomlValue::Float(0.5)));
        assert_eq!(saved.get("refresh_rate"), Some(&TomlValue::Integer(120)));
        
        if let Some(TomlValue::Table(res)) = saved.get("resolution") {
            assert_eq!(res.get("width"), Some(&TomlValue::Integer(2560)));
            assert_eq!(res.get("height"), Some(&TomlValue::Integer(1440)));
        } else {
            panic!("Expected resolution table");
        }
    }
    
    #[test]
    fn test_audio_config_load_save() {
        let mut config = AudioConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(true));
        toml_values.insert("volume".to_string(), TomlValue::Float(0.8));
        toml_values.insert("muted".to_string(), TomlValue::Boolean(true));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, true);
        assert_eq!(config.volume, 0.8);
        assert_eq!(config.muted, true);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(true)));
        assert_eq!(saved.get("volume"), Some(&TomlValue::Float(0.8)));
        assert_eq!(saved.get("muted"), Some(&TomlValue::Boolean(true)));
    }
    
    #[test]
    fn test_tracking_config_load_save() {
        let mut config = TrackingConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(true));
        toml_values.insert("mode".to_string(), TomlValue::String("3dof".to_string()));
        toml_values.insert("camera_enabled".to_string(), TomlValue::Boolean(false));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, true);
        assert_eq!(config.mode, "3dof");
        assert_eq!(config.camera_enabled, false);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(true)));
        assert_eq!(saved.get("mode"), Some(&TomlValue::String("3dof".to_string())));
        assert_eq!(saved.get("camera_enabled"), Some(&TomlValue::Boolean(false)));
    }
    
    #[test]
    fn test_hardware_config_load_save() {
        let config = HardwareConfig::new();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(false));
        display.insert("brightness".to_string(), TomlValue::Float(0.5));
        toml_values.insert("display".to_string(), TomlValue::Table(display));
        
        let mut audio = HashMap::new();
        audio.insert("volume".to_string(), TomlValue::Float(0.8));
        toml_values.insert("audio".to_string(), TomlValue::Table(audio));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify display values were saved correctly
        if let Some(TomlValue::Table(display)) = saved.get("display") {
            assert_eq!(display.get("enabled"), Some(&TomlValue::Boolean(false)));
            assert_eq!(display.get("brightness"), Some(&TomlValue::Float(0.5)));
        } else {
            panic!("Expected display table");
        }
        
        // Verify audio values were saved correctly
        if let Some(TomlValue::Table(audio)) = saved.get("audio") {
            assert_eq!(audio.get("volume"), Some(&TomlValue::Float(0.8)));
        } else {
            panic!("Expected audio table");
        }
    }
}
