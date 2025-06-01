//! Default configurations for the VR Core API.
//!
//! This module provides default configuration values for the VR system,
//! including hardware settings, user preferences, and system defaults.

use std::collections::HashMap;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

/// Default configurations for the VR system.
#[derive(Debug, Clone)]
pub struct DefaultConfigs {
    /// Default system configuration
    system_defaults: HashMap<String, TomlValue>,
    
    /// Default user configuration
    user_defaults: HashMap<String, TomlValue>,
    
    /// Default development configuration
    dev_defaults: HashMap<String, TomlValue>,
}

impl DefaultConfigs {
    /// Create a new DefaultConfigs instance.
    pub fn new() -> Self {
        Self {
            system_defaults: Self::create_system_defaults(),
            user_defaults: Self::create_user_defaults(),
            dev_defaults: Self::create_dev_defaults(),
        }
    }
    
    /// Get the default configuration.
    pub fn get_default_config(&self) -> HashMap<String, TomlValue> {
        // Start with system defaults
        let mut config = self.system_defaults.clone();
        
        // Merge with user defaults
        for (key, value) in &self.user_defaults {
            config.insert(key.clone(), value.clone());
        }
        
        config
    }
    
    /// Get the development configuration.
    pub fn get_dev_config(&self) -> HashMap<String, TomlValue> {
        // Start with default config
        let mut config = self.get_default_config();
        
        // Merge with dev defaults
        for (key, value) in &self.dev_defaults {
            config.insert(key.clone(), value.clone());
        }
        
        config
    }
    
    /// Create system default configuration.
    fn create_system_defaults() -> HashMap<String, TomlValue> {
        let mut config = HashMap::new();
        
        // Version
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        // Display settings
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(true));
        display.insert("brightness".to_string(), TomlValue::Float(0.8));
        display.insert("refresh_rate".to_string(), TomlValue::Integer(90));
        
        let mut resolution = HashMap::new();
        resolution.insert("width".to_string(), TomlValue::Integer(1920));
        resolution.insert("height".to_string(), TomlValue::Integer(1080));
        display.insert("resolution".to_string(), TomlValue::Table(resolution));
        
        config.insert("display".to_string(), TomlValue::Table(display));
        
        // Audio settings
        let mut audio = HashMap::new();
        audio.insert("enabled".to_string(), TomlValue::Boolean(true));
        audio.insert("volume".to_string(), TomlValue::Float(0.5));
        audio.insert("muted".to_string(), TomlValue::Boolean(false));
        audio.insert("spatial_audio".to_string(), TomlValue::Boolean(true));
        
        config.insert("audio".to_string(), TomlValue::Table(audio));
        
        // Tracking settings
        let mut tracking = HashMap::new();
        tracking.insert("enabled".to_string(), TomlValue::Boolean(true));
        tracking.insert("mode".to_string(), TomlValue::String("6dof".to_string()));
        tracking.insert("camera_enabled".to_string(), TomlValue::Boolean(true));
        tracking.insert("imu_enabled".to_string(), TomlValue::Boolean(true));
        
        config.insert("tracking".to_string(), TomlValue::Table(tracking));
        
        // Power settings
        let mut power = HashMap::new();
        power.insert("mode".to_string(), TomlValue::String("normal".to_string()));
        power.insert("auto_sleep".to_string(), TomlValue::Boolean(true));
        power.insert("sleep_timeout_sec".to_string(), TomlValue::Integer(300));
        
        config.insert("power".to_string(), TomlValue::Table(power));
        
        // Network settings
        let mut network = HashMap::new();
        network.insert("wifi_enabled".to_string(), TomlValue::Boolean(true));
        network.insert("bluetooth_enabled".to_string(), TomlValue::Boolean(true));
        
        config.insert("network".to_string(), TomlValue::Table(network));
        
        // Security settings
        let mut security = HashMap::new();
        security.insert("encryption_enabled".to_string(), TomlValue::Boolean(true));
        security.insert("pin_required".to_string(), TomlValue::Boolean(false));
        
        config.insert("security".to_string(), TomlValue::Table(security));
        
        // Storage settings
        let mut storage = HashMap::new();
        storage.insert("auto_backup".to_string(), TomlValue::Boolean(true));
        storage.insert("backup_interval_days".to_string(), TomlValue::Integer(7));
        
        config.insert("storage".to_string(), TomlValue::Table(storage));
        
        config
    }
    
    /// Create user default configuration.
    fn create_user_defaults() -> HashMap<String, TomlValue> {
        let mut config = HashMap::new();
        
        // User interface settings
        let mut ui = HashMap::new();
        ui.insert("theme".to_string(), TomlValue::String("default".to_string()));
        ui.insert("language".to_string(), TomlValue::String("en".to_string()));
        ui.insert("hand".to_string(), TomlValue::String("right".to_string()));
        ui.insert("pointer_speed".to_string(), TomlValue::Float(1.0));
        
        config.insert("ui".to_string(), TomlValue::Table(ui));
        
        // Accessibility settings
        let mut accessibility = HashMap::new();
        accessibility.insert("high_contrast".to_string(), TomlValue::Boolean(false));
        accessibility.insert("large_text".to_string(), TomlValue::Boolean(false));
        accessibility.insert("reduced_motion".to_string(), TomlValue::Boolean(false));
        
        config.insert("accessibility".to_string(), TomlValue::Table(accessibility));
        
        // Notification settings
        let mut notifications = HashMap::new();
        notifications.insert("enabled".to_string(), TomlValue::Boolean(true));
        notifications.insert("do_not_disturb".to_string(), TomlValue::Boolean(false));
        
        config.insert("notifications".to_string(), TomlValue::Table(notifications));
        
        // Privacy settings
        let mut privacy = HashMap::new();
        privacy.insert("analytics_enabled".to_string(), TomlValue::Boolean(false));
        privacy.insert("location_enabled".to_string(), TomlValue::Boolean(true));
        privacy.insert("camera_privacy".to_string(), TomlValue::Boolean(true));
        privacy.insert("microphone_privacy".to_string(), TomlValue::Boolean(true));
        
        config.insert("privacy".to_string(), TomlValue::Table(privacy));
        
        config
    }
    
    /// Create development default configuration.
    fn create_dev_defaults() -> HashMap<String, TomlValue> {
        let mut config = HashMap::new();
        
        // Development settings
        let mut dev = HashMap::new();
        dev.insert("debug_mode".to_string(), TomlValue::Boolean(true));
        dev.insert("verbose_logging".to_string(), TomlValue::Boolean(true));
        dev.insert("performance_overlay".to_string(), TomlValue::Boolean(true));
        
        config.insert("dev".to_string(), TomlValue::Table(dev));
        
        // Testing settings
        let mut testing = HashMap::new();
        testing.insert("mock_sensors".to_string(), TomlValue::Boolean(true));
        testing.insert("test_mode".to_string(), TomlValue::Boolean(true));
        
        config.insert("testing".to_string(), TomlValue::Table(testing));
        
        config
    }
    
    /// Get a specific preset configuration.
    pub fn get_preset_config(&self, preset: &str) -> Option<HashMap<String, TomlValue>> {
        match preset {
            "power_saving" => Some(self.create_power_saving_preset()),
            "performance" => Some(self.create_performance_preset()),
            "cinema" => Some(self.create_cinema_preset()),
            "gaming" => Some(self.create_gaming_preset()),
            "accessibility" => Some(self.create_accessibility_preset()),
            _ => None,
        }
    }
    
    /// Create power saving preset configuration.
    fn create_power_saving_preset(&self) -> HashMap<String, TomlValue> {
        let mut config = self.get_default_config();
        
        // Modify display settings
        if let Some(TomlValue::Table(display)) = config.get_mut("display") {
            display.insert("brightness".to_string(), TomlValue::Float(0.5));
            display.insert("refresh_rate".to_string(), TomlValue::Integer(60));
        }
        
        // Modify power settings
        if let Some(TomlValue::Table(power)) = config.get_mut("power") {
            power.insert("mode".to_string(), TomlValue::String("low_power".to_string()));
            power.insert("sleep_timeout_sec".to_string(), TomlValue::Integer(60));
        }
        
        // Modify network settings
        if let Some(TomlValue::Table(network)) = config.get_mut("network") {
            network.insert("bluetooth_enabled".to_string(), TomlValue::Boolean(false));
        }
        
        config
    }
    
    /// Create performance preset configuration.
    fn create_performance_preset(&self) -> HashMap<String, TomlValue> {
        let mut config = self.get_default_config();
        
        // Modify display settings
        if let Some(TomlValue::Table(display)) = config.get_mut("display") {
            display.insert("brightness".to_string(), TomlValue::Float(1.0));
            display.insert("refresh_rate".to_string(), TomlValue::Integer(120));
        }
        
        // Modify power settings
        if let Some(TomlValue::Table(power)) = config.get_mut("power") {
            power.insert("mode".to_string(), TomlValue::String("performance".to_string()));
            power.insert("auto_sleep".to_string(), TomlValue::Boolean(false));
        }
        
        config
    }
    
    /// Create cinema preset configuration.
    fn create_cinema_preset(&self) -> HashMap<String, TomlValue> {
        let mut config = self.get_default_config();
        
        // Modify display settings
        if let Some(TomlValue::Table(display)) = config.get_mut("display") {
            display.insert("brightness".to_string(), TomlValue::Float(0.7));
            display.insert("refresh_rate".to_string(), TomlValue::Integer(60));
        }
        
        // Modify audio settings
        if let Some(TomlValue::Table(audio)) = config.get_mut("audio") {
            audio.insert("volume".to_string(), TomlValue::Float(0.8));
            audio.insert("spatial_audio".to_string(), TomlValue::Boolean(true));
        }
        
        // Modify notifications
        if let Some(TomlValue::Table(notifications)) = config.get_mut("notifications") {
            notifications.insert("do_not_disturb".to_string(), TomlValue::Boolean(true));
        }
        
        config
    }
    
    /// Create gaming preset configuration.
    fn create_gaming_preset(&self) -> HashMap<String, TomlValue> {
        let mut config = self.get_default_config();
        
        // Modify display settings
        if let Some(TomlValue::Table(display)) = config.get_mut("display") {
            display.insert("brightness".to_string(), TomlValue::Float(0.9));
            display.insert("refresh_rate".to_string(), TomlValue::Integer(120));
        }
        
        // Modify audio settings
        if let Some(TomlValue::Table(audio)) = config.get_mut("audio") {
            audio.insert("volume".to_string(), TomlValue::Float(0.7));
            audio.insert("spatial_audio".to_string(), TomlValue::Boolean(true));
        }
        
        // Modify power settings
        if let Some(TomlValue::Table(power)) = config.get_mut("power") {
            power.insert("mode".to_string(), TomlValue::String("performance".to_string()));
        }
        
        // Modify tracking settings
        if let Some(TomlValue::Table(tracking)) = config.get_mut("tracking") {
            tracking.insert("mode".to_string(), TomlValue::String("6dof".to_string()));
        }
        
        config
    }
    
    /// Create accessibility preset configuration.
    fn create_accessibility_preset(&self) -> HashMap<String, TomlValue> {
        let mut config = self.get_default_config();
        
        // Modify accessibility settings
        if let Some(TomlValue::Table(accessibility)) = config.get_mut("accessibility") {
            accessibility.insert("high_contrast".to_string(), TomlValue::Boolean(true));
            accessibility.insert("large_text".to_string(), TomlValue::Boolean(true));
            accessibility.insert("reduced_motion".to_string(), TomlValue::Boolean(true));
        }
        
        // Modify UI settings
        if let Some(TomlValue::Table(ui)) = config.get_mut("ui") {
            ui.insert("pointer_speed".to_string(), TomlValue::Float(0.7));
        }
        
        // Modify audio settings
        if let Some(TomlValue::Table(audio)) = config.get_mut("audio") {
            audio.insert("volume".to_string(), TomlValue::Float(0.8));
        }
        
        config
    }
}

impl Default for DefaultConfigs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_creation() {
        let defaults = DefaultConfigs::new();
        let config = defaults.get_default_config();
        
        assert!(config.contains_key("version"));
        assert!(config.contains_key("display"));
        assert!(config.contains_key("audio"));
        assert!(config.contains_key("tracking"));
        assert!(config.contains_key("power"));
        assert!(config.contains_key("network"));
        assert!(config.contains_key("security"));
        assert!(config.contains_key("ui"));
        assert!(config.contains_key("accessibility"));
    }
    
    #[test]
    fn test_dev_config_creation() {
        let defaults = DefaultConfigs::new();
        let config = defaults.get_dev_config();
        
        assert!(config.contains_key("dev"));
        assert!(config.contains_key("testing"));
        
        if let Some(TomlValue::Table(dev)) = config.get("dev") {
            assert_eq!(dev.get("debug_mode"), Some(&TomlValue::Boolean(true)));
        } else {
            panic!("Expected dev table");
        }
    }
    
    #[test]
    fn test_preset_configs() {
        let defaults = DefaultConfigs::new();
        
        // Test power saving preset
        let power_saving = defaults.get_preset_config("power_saving").unwrap();
        if let Some(TomlValue::Table(power)) = power_saving.get("power") {
            assert_eq!(power.get("mode"), Some(&TomlValue::String("low_power".to_string())));
        } else {
            panic!("Expected power table");
        }
        
        // Test performance preset
        let performance = defaults.get_preset_config("performance").unwrap();
        if let Some(TomlValue::Table(power)) = performance.get("power") {
            assert_eq!(power.get("mode"), Some(&TomlValue::String("performance".to_string())));
        } else {
            panic!("Expected power table");
        }
        
        // Test cinema preset
        let cinema = defaults.get_preset_config("cinema").unwrap();
        if let Some(TomlValue::Table(notifications)) = cinema.get("notifications") {
            assert_eq!(notifications.get("do_not_disturb"), Some(&TomlValue::Boolean(true)));
        } else {
            panic!("Expected notifications table");
        }
        
        // Test gaming preset
        let gaming = defaults.get_preset_config("gaming").unwrap();
        if let Some(TomlValue::Table(display)) = gaming.get("display") {
            assert_eq!(display.get("refresh_rate"), Some(&TomlValue::Integer(120)));
        } else {
            panic!("Expected display table");
        }
        
        // Test accessibility preset
        let accessibility = defaults.get_preset_config("accessibility").unwrap();
        if let Some(TomlValue::Table(accessibility_table)) = accessibility.get("accessibility") {
            assert_eq!(accessibility_table.get("high_contrast"), Some(&TomlValue::Boolean(true)));
        } else {
            panic!("Expected accessibility table");
        }
    }
}
