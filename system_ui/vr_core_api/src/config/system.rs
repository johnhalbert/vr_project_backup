//! System configuration module for the VR Core API.
//!
//! This module provides comprehensive configuration management for all system
//! components of the VR headset, including performance, update, security,
//! accessibility, language, and time settings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use super::{ConfigError, ConfigResult, validation};

/// System configuration manager.
#[derive(Debug)]
pub struct SystemConfig {
    /// Performance configuration
    performance: RwLock<PerformanceConfig>,
    
    /// Update configuration
    update: RwLock<UpdateConfig>,
    
    /// Security configuration
    security: RwLock<SecurityConfig>,
    
    /// Accessibility configuration
    accessibility: RwLock<AccessibilityConfig>,
    
    /// Language configuration
    language: RwLock<LanguageConfig>,
    
    /// Time configuration
    time: RwLock<TimeConfig>,
}

impl SystemConfig {
    /// Create a new system configuration manager.
    pub fn new() -> Self {
        Self {
            performance: RwLock::new(PerformanceConfig::default()),
            update: RwLock::new(UpdateConfig::default()),
            security: RwLock::new(SecurityConfig::default()),
            accessibility: RwLock::new(AccessibilityConfig::default()),
            language: RwLock::new(LanguageConfig::default()),
            time: RwLock::new(TimeConfig::default()),
        }
    }
    
    /// Load system configuration from TOML values.
    pub fn load_from_toml(&self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load performance configuration
        if let Some(TomlValue::Table(performance_table)) = config.get("performance") {
            let mut performance = self.performance.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for performance config".to_string())
            })?;
            performance.load_from_toml(performance_table)?;
        }
        
        // Load update configuration
        if let Some(TomlValue::Table(update_table)) = config.get("update") {
            let mut update = self.update.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for update config".to_string())
            })?;
            update.load_from_toml(update_table)?;
        }
        
        // Load security configuration
        if let Some(TomlValue::Table(security_table)) = config.get("security") {
            let mut security = self.security.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for security config".to_string())
            })?;
            security.load_from_toml(security_table)?;
        }
        
        // Load accessibility configuration
        if let Some(TomlValue::Table(accessibility_table)) = config.get("accessibility") {
            let mut accessibility = self.accessibility.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for accessibility config".to_string())
            })?;
            accessibility.load_from_toml(accessibility_table)?;
        }
        
        // Load language configuration
        if let Some(TomlValue::Table(language_table)) = config.get("language") {
            let mut language = self.language.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for language config".to_string())
            })?;
            language.load_from_toml(language_table)?;
        }
        
        // Load time configuration
        if let Some(TomlValue::Table(time_table)) = config.get("time") {
            let mut time = self.time.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for time config".to_string())
            })?;
            time.load_from_toml(time_table)?;
        }
        
        Ok(())
    }
    
    /// Save system configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save performance configuration
        let performance = self.performance.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for performance config".to_string())
        })?;
        config.insert("performance".to_string(), TomlValue::Table(performance.save_to_toml()?));
        
        // Save update configuration
        let update = self.update.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for update config".to_string())
        })?;
        config.insert("update".to_string(), TomlValue::Table(update.save_to_toml()?));
        
        // Save security configuration
        let security = self.security.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for security config".to_string())
        })?;
        config.insert("security".to_string(), TomlValue::Table(security.save_to_toml()?));
        
        // Save accessibility configuration
        let accessibility = self.accessibility.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for accessibility config".to_string())
        })?;
        config.insert("accessibility".to_string(), TomlValue::Table(accessibility.save_to_toml()?));
        
        // Save language configuration
        let language = self.language.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for language config".to_string())
        })?;
        config.insert("language".to_string(), TomlValue::Table(language.save_to_toml()?));
        
        // Save time configuration
        let time = self.time.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for time config".to_string())
        })?;
        config.insert("time".to_string(), TomlValue::Table(time.save_to_toml()?));
        
        Ok(config)
    }
    
    /// Get performance configuration.
    pub fn performance(&self) -> ConfigResult<PerformanceConfig> {
        let performance = self.performance.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for performance config".to_string())
        })?;
        Ok(performance.clone())
    }
    
    /// Update performance configuration.
    pub fn update_performance(&self, config: PerformanceConfig) -> ConfigResult<()> {
        let mut performance = self.performance.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for performance config".to_string())
        })?;
        *performance = config;
        Ok(())
    }
    
    /// Get update configuration.
    pub fn update(&self) -> ConfigResult<UpdateConfig> {
        let update = self.update.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for update config".to_string())
        })?;
        Ok(update.clone())
    }
    
    /// Update update configuration.
    pub fn update_update(&self, config: UpdateConfig) -> ConfigResult<()> {
        let mut update = self.update.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for update config".to_string())
        })?;
        *update = config;
        Ok(())
    }
    
    /// Get security configuration.
    pub fn security(&self) -> ConfigResult<SecurityConfig> {
        let security = self.security.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for security config".to_string())
        })?;
        Ok(security.clone())
    }
    
    /// Update security configuration.
    pub fn update_security(&self, config: SecurityConfig) -> ConfigResult<()> {
        let mut security = self.security.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for security config".to_string())
        })?;
        *security = config;
        Ok(())
    }
    
    /// Get accessibility configuration.
    pub fn accessibility(&self) -> ConfigResult<AccessibilityConfig> {
        let accessibility = self.accessibility.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for accessibility config".to_string())
        })?;
        Ok(accessibility.clone())
    }
    
    /// Update accessibility configuration.
    pub fn update_accessibility(&self, config: AccessibilityConfig) -> ConfigResult<()> {
        let mut accessibility = self.accessibility.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for accessibility config".to_string())
        })?;
        *accessibility = config;
        Ok(())
    }
    
    /// Get language configuration.
    pub fn language(&self) -> ConfigResult<LanguageConfig> {
        let language = self.language.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for language config".to_string())
        })?;
        Ok(language.clone())
    }
    
    /// Update language configuration.
    pub fn update_language(&self, config: LanguageConfig) -> ConfigResult<()> {
        let mut language = self.language.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for language config".to_string())
        })?;
        *language = config;
        Ok(())
    }
    
    /// Get time configuration.
    pub fn time(&self) -> ConfigResult<TimeConfig> {
        let time = self.time.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for time config".to_string())
        })?;
        Ok(time.clone())
    }
    
    /// Update time configuration.
    pub fn update_time(&self, config: TimeConfig) -> ConfigResult<()> {
        let mut time = self.time.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for time config".to_string())
        })?;
        *time = config;
        Ok(())
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Performance mode (balanced, performance, power_saving, custom)
    pub mode: String,
    
    /// CPU performance level (0-100)
    pub cpu_level: u32,
    
    /// GPU performance level (0-100)
    pub gpu_level: u32,
    
    /// Whether to enable dynamic foveated rendering
    pub dynamic_foveated_rendering: bool,
    
    /// Whether to enable fixed foveated rendering
    pub fixed_foveated_rendering: bool,
    
    /// Fixed foveated rendering level (1-4)
    pub fixed_foveated_level: u32,
    
    /// Whether to enable motion smoothing
    pub motion_smoothing: bool,
    
    /// Whether to enable supersampling
    pub supersampling: bool,
    
    /// Supersampling factor (0.5-2.0)
    pub supersampling_factor: f32,
    
    /// Whether to enable thermal throttling
    pub thermal_throttling: bool,
    
    /// Thermal throttling threshold in Celsius
    pub thermal_threshold_celsius: u32,
    
    /// Whether to enable performance overlay
    pub performance_overlay: bool,
    
    /// Performance overlay level (basic, advanced, developer)
    pub performance_overlay_level: String,
    
    /// Whether to enable frame timing
    pub frame_timing: bool,
    
    /// Target frame rate
    pub target_framerate: u32,
    
    /// Whether to enable asynchronous reprojection
    pub async_reprojection: bool,
    
    /// Whether to enable interleaved reprojection
    pub interleaved_reprojection: bool,
    
    /// Whether to enable always-on reprojection
    pub always_on_reprojection: bool,
    
    /// Whether to enable GPU priority
    pub gpu_priority: bool,
}

impl PerformanceConfig {
    /// Load performance configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load mode
        if let Some(TomlValue::String(mode)) = config.get("mode") {
            self.mode = mode.clone();
            // Validate mode
            if self.mode != "balanced" && self.mode != "performance" && self.mode != "power_saving" && self.mode != "custom" {
                return Err(ConfigError::ValidationError(
                    "Performance mode must be 'balanced', 'performance', 'power_saving', or 'custom'".to_string()
                ));
            }
        }
        
        // Load CPU level
        if let Some(TomlValue::Integer(cpu_level)) = config.get("cpu_level") {
            self.cpu_level = *cpu_level as u32;
            // Validate CPU level
            if self.cpu_level > 100 {
                return Err(ConfigError::ValidationError(
                    "CPU level must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load GPU level
        if let Some(TomlValue::Integer(gpu_level)) = config.get("gpu_level") {
            self.gpu_level = *gpu_level as u32;
            // Validate GPU level
            if self.gpu_level > 100 {
                return Err(ConfigError::ValidationError(
                    "GPU level must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load dynamic foveated rendering
        if let Some(TomlValue::Boolean(dynamic_foveated_rendering)) = config.get("dynamic_foveated_rendering") {
            self.dynamic_foveated_rendering = *dynamic_foveated_rendering;
        }
        
        // Load fixed foveated rendering
        if let Some(TomlValue::Boolean(fixed_foveated_rendering)) = config.get("fixed_foveated_rendering") {
            self.fixed_foveated_rendering = *fixed_foveated_rendering;
        }
        
        // Load fixed foveated level
        if let Some(TomlValue::Integer(fixed_foveated_level)) = config.get("fixed_foveated_level") {
            self.fixed_foveated_level = *fixed_foveated_level as u32;
            // Validate fixed foveated level
            if self.fixed_foveated_level < 1 || self.fixed_foveated_level > 4 {
                return Err(ConfigError::ValidationError(
                    "Fixed foveated level must be between 1 and 4".to_string()
                ));
            }
        }
        
        // Load motion smoothing
        if let Some(TomlValue::Boolean(motion_smoothing)) = config.get("motion_smoothing") {
            self.motion_smoothing = *motion_smoothing;
        }
        
        // Load supersampling
        if let Some(TomlValue::Boolean(supersampling)) = config.get("supersampling") {
            self.supersampling = *supersampling;
        }
        
        // Load supersampling factor
        if let Some(TomlValue::Float(supersampling_factor)) = config.get("supersampling_factor") {
            self.supersampling_factor = *supersampling_factor as f32;
            // Validate supersampling factor
            if self.supersampling_factor < 0.5 || self.supersampling_factor > 2.0 {
                return Err(ConfigError::ValidationError(
                    "Supersampling factor must be between 0.5 and 2.0".to_string()
                ));
            }
        }
        
        // Load thermal throttling
        if let Some(TomlValue::Boolean(thermal_throttling)) = config.get("thermal_throttling") {
            self.thermal_throttling = *thermal_throttling;
        }
        
        // Load thermal threshold
        if let Some(TomlValue::Integer(thermal_threshold_celsius)) = config.get("thermal_threshold_celsius") {
            self.thermal_threshold_celsius = *thermal_threshold_celsius as u32;
            // Validate thermal threshold
            if self.thermal_threshold_celsius < 60 || self.thermal_threshold_celsius > 90 {
                return Err(ConfigError::ValidationError(
                    "Thermal threshold must be between 60 and 90 degrees Celsius".to_string()
                ));
            }
        }
        
        // Load performance overlay
        if let Some(TomlValue::Boolean(performance_overlay)) = config.get("performance_overlay") {
            self.performance_overlay = *performance_overlay;
        }
        
        // Load performance overlay level
        if let Some(TomlValue::String(performance_overlay_level)) = config.get("performance_overlay_level") {
            self.performance_overlay_level = performance_overlay_level.clone();
            // Validate performance overlay level
            if self.performance_overlay_level != "basic" && self.performance_overlay_level != "advanced" && self.performance_overlay_level != "developer" {
                return Err(ConfigError::ValidationError(
                    "Performance overlay level must be 'basic', 'advanced', or 'developer'".to_string()
                ));
            }
        }
        
        // Load frame timing
        if let Some(TomlValue::Boolean(frame_timing)) = config.get("frame_timing") {
            self.frame_timing = *frame_timing;
        }
        
        // Load target framerate
        if let Some(TomlValue::Integer(target_framerate)) = config.get("target_framerate") {
            self.target_framerate = *target_framerate as u32;
            // Validate target framerate
            if self.target_framerate != 60 && self.target_framerate != 72 && self.target_framerate != 90 && self.target_framerate != 120 && self.target_framerate != 144 {
                return Err(ConfigError::ValidationError(
                    "Target framerate must be 60, 72, 90, 120, or 144".to_string()
                ));
            }
        }
        
        // Load async reprojection
        if let Some(TomlValue::Boolean(async_reprojection)) = config.get("async_reprojection") {
            self.async_reprojection = *async_reprojection;
        }
        
        // Load interleaved reprojection
        if let Some(TomlValue::Boolean(interleaved_reprojection)) = config.get("interleaved_reprojection") {
            self.interleaved_reprojection = *interleaved_reprojection;
        }
        
        // Load always-on reprojection
        if let Some(TomlValue::Boolean(always_on_reprojection)) = config.get("always_on_reprojection") {
            self.always_on_reprojection = *always_on_reprojection;
        }
        
        // Load GPU priority
        if let Some(TomlValue::Boolean(gpu_priority)) = config.get("gpu_priority") {
            self.gpu_priority = *gpu_priority;
        }
        
        Ok(())
    }
    
    /// Save performance configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save mode
        config.insert("mode".to_string(), TomlValue::String(self.mode.clone()));
        
        // Save CPU level
        config.insert("cpu_level".to_string(), TomlValue::Integer(self.cpu_level as i64));
        
        // Save GPU level
        config.insert("gpu_level".to_string(), TomlValue::Integer(self.gpu_level as i64));
        
        // Save dynamic foveated rendering
        config.insert("dynamic_foveated_rendering".to_string(), TomlValue::Boolean(self.dynamic_foveated_rendering));
        
        // Save fixed foveated rendering
        config.insert("fixed_foveated_rendering".to_string(), TomlValue::Boolean(self.fixed_foveated_rendering));
        
        // Save fixed foveated level
        config.insert("fixed_foveated_level".to_string(), TomlValue::Integer(self.fixed_foveated_level as i64));
        
        // Save motion smoothing
        config.insert("motion_smoothing".to_string(), TomlValue::Boolean(self.motion_smoothing));
        
        // Save supersampling
        config.insert("supersampling".to_string(), TomlValue::Boolean(self.supersampling));
        
        // Save supersampling factor
        config.insert("supersampling_factor".to_string(), TomlValue::Float(self.supersampling_factor as f64));
        
        // Save thermal throttling
        config.insert("thermal_throttling".to_string(), TomlValue::Boolean(self.thermal_throttling));
        
        // Save thermal threshold
        config.insert("thermal_threshold_celsius".to_string(), TomlValue::Integer(self.thermal_threshold_celsius as i64));
        
        // Save performance overlay
        config.insert("performance_overlay".to_string(), TomlValue::Boolean(self.performance_overlay));
        
        // Save performance overlay level
        config.insert("performance_overlay_level".to_string(), TomlValue::String(self.performance_overlay_level.clone()));
        
        // Save frame timing
        config.insert("frame_timing".to_string(), TomlValue::Boolean(self.frame_timing));
        
        // Save target framerate
        config.insert("target_framerate".to_string(), TomlValue::Integer(self.target_framerate as i64));
        
        // Save async reprojection
        config.insert("async_reprojection".to_string(), TomlValue::Boolean(self.async_reprojection));
        
        // Save interleaved reprojection
        config.insert("interleaved_reprojection".to_string(), TomlValue::Boolean(self.interleaved_reprojection));
        
        // Save always-on reprojection
        config.insert("always_on_reprojection".to_string(), TomlValue::Boolean(self.always_on_reprojection));
        
        // Save GPU priority
        config.insert("gpu_priority".to_string(), TomlValue::Boolean(self.gpu_priority));
        
        Ok(config)
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            mode: "balanced".to_string(),
            cpu_level: 80,
            gpu_level: 80,
            dynamic_foveated_rendering: true,
            fixed_foveated_rendering: false,
            fixed_foveated_level: 2,
            motion_smoothing: true,
            supersampling: false,
            supersampling_factor: 1.0,
            thermal_throttling: true,
            thermal_threshold_celsius: 80,
            performance_overlay: false,
            performance_overlay_level: "basic".to_string(),
            frame_timing: false,
            target_framerate: 90,
            async_reprojection: true,
            interleaved_reprojection: false,
            always_on_reprojection: false,
            gpu_priority: true,
        }
    }
}

/// Update configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Whether to enable automatic updates
    pub auto_update: bool,
    
    /// Whether to check for updates on startup
    pub check_on_startup: bool,
    
    /// Update check interval in hours
    pub check_interval_hours: u32,
    
    /// Whether to download updates automatically
    pub auto_download: bool,
    
    /// Whether to install updates automatically
    pub auto_install: bool,
    
    /// Whether to install updates on shutdown
    pub install_on_shutdown: bool,
    
    /// Whether to enable beta updates
    pub beta_updates: bool,
    
    /// Whether to enable developer updates
    pub developer_updates: bool,
    
    /// Update channel (stable, beta, developer)
    pub channel: String,
    
    /// Whether to enable background updates
    pub background_updates: bool,
    
    /// Whether to enable update notifications
    pub update_notifications: bool,
    
    /// Whether to enable bandwidth limiting for updates
    pub bandwidth_limit: bool,
    
    /// Bandwidth limit in Mbps
    pub bandwidth_limit_mbps: u32,
    
    /// Whether to enable scheduled updates
    pub scheduled_updates: bool,
    
    /// Scheduled update time (HH:MM)
    pub scheduled_time: String,
    
    /// Scheduled update days (comma-separated list of days)
    pub scheduled_days: String,
}

impl UpdateConfig {
    /// Load update configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load auto update
        if let Some(TomlValue::Boolean(auto_update)) = config.get("auto_update") {
            self.auto_update = *auto_update;
        }
        
        // Load check on startup
        if let Some(TomlValue::Boolean(check_on_startup)) = config.get("check_on_startup") {
            self.check_on_startup = *check_on_startup;
        }
        
        // Load check interval
        if let Some(TomlValue::Integer(check_interval_hours)) = config.get("check_interval_hours") {
            self.check_interval_hours = *check_interval_hours as u32;
        }
        
        // Load auto download
        if let Some(TomlValue::Boolean(auto_download)) = config.get("auto_download") {
            self.auto_download = *auto_download;
        }
        
        // Load auto install
        if let Some(TomlValue::Boolean(auto_install)) = config.get("auto_install") {
            self.auto_install = *auto_install;
        }
        
        // Load install on shutdown
        if let Some(TomlValue::Boolean(install_on_shutdown)) = config.get("install_on_shutdown") {
            self.install_on_shutdown = *install_on_shutdown;
        }
        
        // Load beta updates
        if let Some(TomlValue::Boolean(beta_updates)) = config.get("beta_updates") {
            self.beta_updates = *beta_updates;
        }
        
        // Load developer updates
        if let Some(TomlValue::Boolean(developer_updates)) = config.get("developer_updates") {
            self.developer_updates = *developer_updates;
        }
        
        // Load channel
        if let Some(TomlValue::String(channel)) = config.get("channel") {
            self.channel = channel.clone();
            // Validate channel
            if self.channel != "stable" && self.channel != "beta" && self.channel != "developer" {
                return Err(ConfigError::ValidationError(
                    "Update channel must be 'stable', 'beta', or 'developer'".to_string()
                ));
            }
        }
        
        // Load background updates
        if let Some(TomlValue::Boolean(background_updates)) = config.get("background_updates") {
            self.background_updates = *background_updates;
        }
        
        // Load update notifications
        if let Some(TomlValue::Boolean(update_notifications)) = config.get("update_notifications") {
            self.update_notifications = *update_notifications;
        }
        
        // Load bandwidth limit
        if let Some(TomlValue::Boolean(bandwidth_limit)) = config.get("bandwidth_limit") {
            self.bandwidth_limit = *bandwidth_limit;
        }
        
        // Load bandwidth limit Mbps
        if let Some(TomlValue::Integer(bandwidth_limit_mbps)) = config.get("bandwidth_limit_mbps") {
            self.bandwidth_limit_mbps = *bandwidth_limit_mbps as u32;
        }
        
        // Load scheduled updates
        if let Some(TomlValue::Boolean(scheduled_updates)) = config.get("scheduled_updates") {
            self.scheduled_updates = *scheduled_updates;
        }
        
        // Load scheduled time
        if let Some(TomlValue::String(scheduled_time)) = config.get("scheduled_time") {
            self.scheduled_time = scheduled_time.clone();
            // Validate scheduled time format (HH:MM)
            if !validation::is_valid_time_format(&self.scheduled_time) {
                return Err(ConfigError::ValidationError(
                    "Scheduled time must be in HH:MM format".to_string()
                ));
            }
        }
        
        // Load scheduled days
        if let Some(TomlValue::String(scheduled_days)) = config.get("scheduled_days") {
            self.scheduled_days = scheduled_days.clone();
            // Validate scheduled days
            for day in self.scheduled_days.split(',') {
                let day = day.trim();
                if day != "monday" && day != "tuesday" && day != "wednesday" && day != "thursday" && day != "friday" && day != "saturday" && day != "sunday" {
                    return Err(ConfigError::ValidationError(
                        "Scheduled days must be a comma-separated list of valid days of the week".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Save update configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save auto update
        config.insert("auto_update".to_string(), TomlValue::Boolean(self.auto_update));
        
        // Save check on startup
        config.insert("check_on_startup".to_string(), TomlValue::Boolean(self.check_on_startup));
        
        // Save check interval
        config.insert("check_interval_hours".to_string(), TomlValue::Integer(self.check_interval_hours as i64));
        
        // Save auto download
        config.insert("auto_download".to_string(), TomlValue::Boolean(self.auto_download));
        
        // Save auto install
        config.insert("auto_install".to_string(), TomlValue::Boolean(self.auto_install));
        
        // Save install on shutdown
        config.insert("install_on_shutdown".to_string(), TomlValue::Boolean(self.install_on_shutdown));
        
        // Save beta updates
        config.insert("beta_updates".to_string(), TomlValue::Boolean(self.beta_updates));
        
        // Save developer updates
        config.insert("developer_updates".to_string(), TomlValue::Boolean(self.developer_updates));
        
        // Save channel
        config.insert("channel".to_string(), TomlValue::String(self.channel.clone()));
        
        // Save background updates
        config.insert("background_updates".to_string(), TomlValue::Boolean(self.background_updates));
        
        // Save update notifications
        config.insert("update_notifications".to_string(), TomlValue::Boolean(self.update_notifications));
        
        // Save bandwidth limit
        config.insert("bandwidth_limit".to_string(), TomlValue::Boolean(self.bandwidth_limit));
        
        // Save bandwidth limit Mbps
        config.insert("bandwidth_limit_mbps".to_string(), TomlValue::Integer(self.bandwidth_limit_mbps as i64));
        
        // Save scheduled updates
        config.insert("scheduled_updates".to_string(), TomlValue::Boolean(self.scheduled_updates));
        
        // Save scheduled time
        config.insert("scheduled_time".to_string(), TomlValue::String(self.scheduled_time.clone()));
        
        // Save scheduled days
        config.insert("scheduled_days".to_string(), TomlValue::String(self.scheduled_days.clone()));
        
        Ok(config)
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_update: true,
            check_on_startup: true,
            check_interval_hours: 24,
            auto_download: true,
            auto_install: false,
            install_on_shutdown: true,
            beta_updates: false,
            developer_updates: false,
            channel: "stable".to_string(),
            background_updates: true,
            update_notifications: true,
            bandwidth_limit: false,
            bandwidth_limit_mbps: 10,
            scheduled_updates: false,
            scheduled_time: "03:00".to_string(),
            scheduled_days: "monday,wednesday,friday".to_string(),
        }
    }
}

/// Security configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to enable PIN protection
    pub pin_protection: bool,
    
    /// PIN code (hashed)
    pub pin_hash: String,
    
    /// Whether to enable biometric authentication
    pub biometric_auth: bool,
    
    /// Whether to enable automatic locking
    pub auto_lock: bool,
    
    /// Auto-lock timeout in minutes
    pub auto_lock_timeout_min: u32,
    
    /// Whether to enable app permissions
    pub app_permissions: bool,
    
    /// Whether to enable camera permissions
    pub camera_permissions: bool,
    
    /// Whether to enable microphone permissions
    pub microphone_permissions: bool,
    
    /// Whether to enable location permissions
    pub location_permissions: bool,
    
    /// Whether to enable contacts permissions
    pub contacts_permissions: bool,
    
    /// Whether to enable storage permissions
    pub storage_permissions: bool,
    
    /// Whether to enable secure boot
    pub secure_boot: bool,
    
    /// Whether to enable data encryption
    pub data_encryption: bool,
    
    /// Whether to enable remote wipe
    pub remote_wipe: bool,
    
    /// Whether to enable USB debugging
    pub usb_debugging: bool,
    
    /// Whether to enable developer mode
    pub developer_mode: bool,
    
    /// Whether to enable unknown sources
    pub unknown_sources: bool,
    
    /// Whether to enable factory reset protection
    pub factory_reset_protection: bool,
    
    /// Whether to enable screen lock
    pub screen_lock: bool,
    
    /// Screen lock type (none, pattern, pin, password)
    pub screen_lock_type: String,
}

impl SecurityConfig {
    /// Load security configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load PIN protection
        if let Some(TomlValue::Boolean(pin_protection)) = config.get("pin_protection") {
            self.pin_protection = *pin_protection;
        }
        
        // Load PIN hash
        if let Some(TomlValue::String(pin_hash)) = config.get("pin_hash") {
            self.pin_hash = pin_hash.clone();
        }
        
        // Load biometric authentication
        if let Some(TomlValue::Boolean(biometric_auth)) = config.get("biometric_auth") {
            self.biometric_auth = *biometric_auth;
        }
        
        // Load auto lock
        if let Some(TomlValue::Boolean(auto_lock)) = config.get("auto_lock") {
            self.auto_lock = *auto_lock;
        }
        
        // Load auto lock timeout
        if let Some(TomlValue::Integer(auto_lock_timeout_min)) = config.get("auto_lock_timeout_min") {
            self.auto_lock_timeout_min = *auto_lock_timeout_min as u32;
        }
        
        // Load app permissions
        if let Some(TomlValue::Boolean(app_permissions)) = config.get("app_permissions") {
            self.app_permissions = *app_permissions;
        }
        
        // Load camera permissions
        if let Some(TomlValue::Boolean(camera_permissions)) = config.get("camera_permissions") {
            self.camera_permissions = *camera_permissions;
        }
        
        // Load microphone permissions
        if let Some(TomlValue::Boolean(microphone_permissions)) = config.get("microphone_permissions") {
            self.microphone_permissions = *microphone_permissions;
        }
        
        // Load location permissions
        if let Some(TomlValue::Boolean(location_permissions)) = config.get("location_permissions") {
            self.location_permissions = *location_permissions;
        }
        
        // Load contacts permissions
        if let Some(TomlValue::Boolean(contacts_permissions)) = config.get("contacts_permissions") {
            self.contacts_permissions = *contacts_permissions;
        }
        
        // Load storage permissions
        if let Some(TomlValue::Boolean(storage_permissions)) = config.get("storage_permissions") {
            self.storage_permissions = *storage_permissions;
        }
        
        // Load secure boot
        if let Some(TomlValue::Boolean(secure_boot)) = config.get("secure_boot") {
            self.secure_boot = *secure_boot;
        }
        
        // Load data encryption
        if let Some(TomlValue::Boolean(data_encryption)) = config.get("data_encryption") {
            self.data_encryption = *data_encryption;
        }
        
        // Load remote wipe
        if let Some(TomlValue::Boolean(remote_wipe)) = config.get("remote_wipe") {
            self.remote_wipe = *remote_wipe;
        }
        
        // Load USB debugging
        if let Some(TomlValue::Boolean(usb_debugging)) = config.get("usb_debugging") {
            self.usb_debugging = *usb_debugging;
        }
        
        // Load developer mode
        if let Some(TomlValue::Boolean(developer_mode)) = config.get("developer_mode") {
            self.developer_mode = *developer_mode;
        }
        
        // Load unknown sources
        if let Some(TomlValue::Boolean(unknown_sources)) = config.get("unknown_sources") {
            self.unknown_sources = *unknown_sources;
        }
        
        // Load factory reset protection
        if let Some(TomlValue::Boolean(factory_reset_protection)) = config.get("factory_reset_protection") {
            self.factory_reset_protection = *factory_reset_protection;
        }
        
        // Load screen lock
        if let Some(TomlValue::Boolean(screen_lock)) = config.get("screen_lock") {
            self.screen_lock = *screen_lock;
        }
        
        // Load screen lock type
        if let Some(TomlValue::String(screen_lock_type)) = config.get("screen_lock_type") {
            self.screen_lock_type = screen_lock_type.clone();
            // Validate screen lock type
            if self.screen_lock_type != "none" && self.screen_lock_type != "pattern" && self.screen_lock_type != "pin" && self.screen_lock_type != "password" {
                return Err(ConfigError::ValidationError(
                    "Screen lock type must be 'none', 'pattern', 'pin', or 'password'".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Save security configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save PIN protection
        config.insert("pin_protection".to_string(), TomlValue::Boolean(self.pin_protection));
        
        // Save PIN hash
        config.insert("pin_hash".to_string(), TomlValue::String(self.pin_hash.clone()));
        
        // Save biometric authentication
        config.insert("biometric_auth".to_string(), TomlValue::Boolean(self.biometric_auth));
        
        // Save auto lock
        config.insert("auto_lock".to_string(), TomlValue::Boolean(self.auto_lock));
        
        // Save auto lock timeout
        config.insert("auto_lock_timeout_min".to_string(), TomlValue::Integer(self.auto_lock_timeout_min as i64));
        
        // Save app permissions
        config.insert("app_permissions".to_string(), TomlValue::Boolean(self.app_permissions));
        
        // Save camera permissions
        config.insert("camera_permissions".to_string(), TomlValue::Boolean(self.camera_permissions));
        
        // Save microphone permissions
        config.insert("microphone_permissions".to_string(), TomlValue::Boolean(self.microphone_permissions));
        
        // Save location permissions
        config.insert("location_permissions".to_string(), TomlValue::Boolean(self.location_permissions));
        
        // Save contacts permissions
        config.insert("contacts_permissions".to_string(), TomlValue::Boolean(self.contacts_permissions));
        
        // Save storage permissions
        config.insert("storage_permissions".to_string(), TomlValue::Boolean(self.storage_permissions));
        
        // Save secure boot
        config.insert("secure_boot".to_string(), TomlValue::Boolean(self.secure_boot));
        
        // Save data encryption
        config.insert("data_encryption".to_string(), TomlValue::Boolean(self.data_encryption));
        
        // Save remote wipe
        config.insert("remote_wipe".to_string(), TomlValue::Boolean(self.remote_wipe));
        
        // Save USB debugging
        config.insert("usb_debugging".to_string(), TomlValue::Boolean(self.usb_debugging));
        
        // Save developer mode
        config.insert("developer_mode".to_string(), TomlValue::Boolean(self.developer_mode));
        
        // Save unknown sources
        config.insert("unknown_sources".to_string(), TomlValue::Boolean(self.unknown_sources));
        
        // Save factory reset protection
        config.insert("factory_reset_protection".to_string(), TomlValue::Boolean(self.factory_reset_protection));
        
        // Save screen lock
        config.insert("screen_lock".to_string(), TomlValue::Boolean(self.screen_lock));
        
        // Save screen lock type
        config.insert("screen_lock_type".to_string(), TomlValue::String(self.screen_lock_type.clone()));
        
        Ok(config)
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pin_protection: false,
            pin_hash: String::new(),
            biometric_auth: false,
            auto_lock: true,
            auto_lock_timeout_min: 5,
            app_permissions: true,
            camera_permissions: true,
            microphone_permissions: true,
            location_permissions: true,
            contacts_permissions: true,
            storage_permissions: true,
            secure_boot: true,
            data_encryption: true,
            remote_wipe: false,
            usb_debugging: false,
            developer_mode: false,
            unknown_sources: false,
            factory_reset_protection: true,
            screen_lock: false,
            screen_lock_type: "none".to_string(),
        }
    }
}

/// Accessibility configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Whether to enable high contrast mode
    pub high_contrast: bool,
    
    /// Whether to enable color correction
    pub color_correction: bool,
    
    /// Color correction type (deuteranomaly, protanomaly, tritanomaly)
    pub color_correction_type: String,
    
    /// Whether to enable text-to-speech
    pub text_to_speech: bool,
    
    /// Text-to-speech rate (0.5-2.0)
    pub tts_rate: f32,
    
    /// Text-to-speech pitch (0.5-2.0)
    pub tts_pitch: f32,
    
    /// Whether to enable screen reader
    pub screen_reader: bool,
    
    /// Whether to enable magnification
    pub magnification: bool,
    
    /// Magnification level (1.0-5.0)
    pub magnification_level: f32,
    
    /// Whether to enable subtitles
    pub subtitles: bool,
    
    /// Subtitle size (small, medium, large)
    pub subtitle_size: String,
    
    /// Whether to enable mono audio
    pub mono_audio: bool,
    
    /// Whether to enable audio balance
    pub audio_balance: bool,
    
    /// Audio balance (-1.0 to 1.0, 0.0 is centered)
    pub audio_balance_value: f32,
    
    /// Whether to enable reduced motion
    pub reduced_motion: bool,
    
    /// Whether to enable reduced transparency
    pub reduced_transparency: bool,
    
    /// Whether to enable touch accommodations
    pub touch_accommodations: bool,
    
    /// Touch hold duration in milliseconds
    pub touch_hold_duration_ms: u32,
    
    /// Whether to enable switch control
    pub switch_control: bool,
    
    /// Whether to enable voice control
    pub voice_control: bool,
    
    /// Whether to enable gesture navigation
    pub gesture_navigation: bool,
}

impl AccessibilityConfig {
    /// Load accessibility configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load high contrast
        if let Some(TomlValue::Boolean(high_contrast)) = config.get("high_contrast") {
            self.high_contrast = *high_contrast;
        }
        
        // Load color correction
        if let Some(TomlValue::Boolean(color_correction)) = config.get("color_correction") {
            self.color_correction = *color_correction;
        }
        
        // Load color correction type
        if let Some(TomlValue::String(color_correction_type)) = config.get("color_correction_type") {
            self.color_correction_type = color_correction_type.clone();
            // Validate color correction type
            if self.color_correction_type != "deuteranomaly" && self.color_correction_type != "protanomaly" && self.color_correction_type != "tritanomaly" {
                return Err(ConfigError::ValidationError(
                    "Color correction type must be 'deuteranomaly', 'protanomaly', or 'tritanomaly'".to_string()
                ));
            }
        }
        
        // Load text-to-speech
        if let Some(TomlValue::Boolean(text_to_speech)) = config.get("text_to_speech") {
            self.text_to_speech = *text_to_speech;
        }
        
        // Load TTS rate
        if let Some(TomlValue::Float(tts_rate)) = config.get("tts_rate") {
            self.tts_rate = *tts_rate as f32;
            // Validate TTS rate
            if self.tts_rate < 0.5 || self.tts_rate > 2.0 {
                return Err(ConfigError::ValidationError(
                    "Text-to-speech rate must be between 0.5 and 2.0".to_string()
                ));
            }
        }
        
        // Load TTS pitch
        if let Some(TomlValue::Float(tts_pitch)) = config.get("tts_pitch") {
            self.tts_pitch = *tts_pitch as f32;
            // Validate TTS pitch
            if self.tts_pitch < 0.5 || self.tts_pitch > 2.0 {
                return Err(ConfigError::ValidationError(
                    "Text-to-speech pitch must be between 0.5 and 2.0".to_string()
                ));
            }
        }
        
        // Load screen reader
        if let Some(TomlValue::Boolean(screen_reader)) = config.get("screen_reader") {
            self.screen_reader = *screen_reader;
        }
        
        // Load magnification
        if let Some(TomlValue::Boolean(magnification)) = config.get("magnification") {
            self.magnification = *magnification;
        }
        
        // Load magnification level
        if let Some(TomlValue::Float(magnification_level)) = config.get("magnification_level") {
            self.magnification_level = *magnification_level as f32;
            // Validate magnification level
            if self.magnification_level < 1.0 || self.magnification_level > 5.0 {
                return Err(ConfigError::ValidationError(
                    "Magnification level must be between 1.0 and 5.0".to_string()
                ));
            }
        }
        
        // Load subtitles
        if let Some(TomlValue::Boolean(subtitles)) = config.get("subtitles") {
            self.subtitles = *subtitles;
        }
        
        // Load subtitle size
        if let Some(TomlValue::String(subtitle_size)) = config.get("subtitle_size") {
            self.subtitle_size = subtitle_size.clone();
            // Validate subtitle size
            if self.subtitle_size != "small" && self.subtitle_size != "medium" && self.subtitle_size != "large" {
                return Err(ConfigError::ValidationError(
                    "Subtitle size must be 'small', 'medium', or 'large'".to_string()
                ));
            }
        }
        
        // Load mono audio
        if let Some(TomlValue::Boolean(mono_audio)) = config.get("mono_audio") {
            self.mono_audio = *mono_audio;
        }
        
        // Load audio balance
        if let Some(TomlValue::Boolean(audio_balance)) = config.get("audio_balance") {
            self.audio_balance = *audio_balance;
        }
        
        // Load audio balance value
        if let Some(TomlValue::Float(audio_balance_value)) = config.get("audio_balance_value") {
            self.audio_balance_value = *audio_balance_value as f32;
            // Validate audio balance value
            if self.audio_balance_value < -1.0 || self.audio_balance_value > 1.0 {
                return Err(ConfigError::ValidationError(
                    "Audio balance value must be between -1.0 and 1.0".to_string()
                ));
            }
        }
        
        // Load reduced motion
        if let Some(TomlValue::Boolean(reduced_motion)) = config.get("reduced_motion") {
            self.reduced_motion = *reduced_motion;
        }
        
        // Load reduced transparency
        if let Some(TomlValue::Boolean(reduced_transparency)) = config.get("reduced_transparency") {
            self.reduced_transparency = *reduced_transparency;
        }
        
        // Load touch accommodations
        if let Some(TomlValue::Boolean(touch_accommodations)) = config.get("touch_accommodations") {
            self.touch_accommodations = *touch_accommodations;
        }
        
        // Load touch hold duration
        if let Some(TomlValue::Integer(touch_hold_duration_ms)) = config.get("touch_hold_duration_ms") {
            self.touch_hold_duration_ms = *touch_hold_duration_ms as u32;
        }
        
        // Load switch control
        if let Some(TomlValue::Boolean(switch_control)) = config.get("switch_control") {
            self.switch_control = *switch_control;
        }
        
        // Load voice control
        if let Some(TomlValue::Boolean(voice_control)) = config.get("voice_control") {
            self.voice_control = *voice_control;
        }
        
        // Load gesture navigation
        if let Some(TomlValue::Boolean(gesture_navigation)) = config.get("gesture_navigation") {
            self.gesture_navigation = *gesture_navigation;
        }
        
        Ok(())
    }
    
    /// Save accessibility configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save high contrast
        config.insert("high_contrast".to_string(), TomlValue::Boolean(self.high_contrast));
        
        // Save color correction
        config.insert("color_correction".to_string(), TomlValue::Boolean(self.color_correction));
        
        // Save color correction type
        config.insert("color_correction_type".to_string(), TomlValue::String(self.color_correction_type.clone()));
        
        // Save text-to-speech
        config.insert("text_to_speech".to_string(), TomlValue::Boolean(self.text_to_speech));
        
        // Save TTS rate
        config.insert("tts_rate".to_string(), TomlValue::Float(self.tts_rate as f64));
        
        // Save TTS pitch
        config.insert("tts_pitch".to_string(), TomlValue::Float(self.tts_pitch as f64));
        
        // Save screen reader
        config.insert("screen_reader".to_string(), TomlValue::Boolean(self.screen_reader));
        
        // Save magnification
        config.insert("magnification".to_string(), TomlValue::Boolean(self.magnification));
        
        // Save magnification level
        config.insert("magnification_level".to_string(), TomlValue::Float(self.magnification_level as f64));
        
        // Save subtitles
        config.insert("subtitles".to_string(), TomlValue::Boolean(self.subtitles));
        
        // Save subtitle size
        config.insert("subtitle_size".to_string(), TomlValue::String(self.subtitle_size.clone()));
        
        // Save mono audio
        config.insert("mono_audio".to_string(), TomlValue::Boolean(self.mono_audio));
        
        // Save audio balance
        config.insert("audio_balance".to_string(), TomlValue::Boolean(self.audio_balance));
        
        // Save audio balance value
        config.insert("audio_balance_value".to_string(), TomlValue::Float(self.audio_balance_value as f64));
        
        // Save reduced motion
        config.insert("reduced_motion".to_string(), TomlValue::Boolean(self.reduced_motion));
        
        // Save reduced transparency
        config.insert("reduced_transparency".to_string(), TomlValue::Boolean(self.reduced_transparency));
        
        // Save touch accommodations
        config.insert("touch_accommodations".to_string(), TomlValue::Boolean(self.touch_accommodations));
        
        // Save touch hold duration
        config.insert("touch_hold_duration_ms".to_string(), TomlValue::Integer(self.touch_hold_duration_ms as i64));
        
        // Save switch control
        config.insert("switch_control".to_string(), TomlValue::Boolean(self.switch_control));
        
        // Save voice control
        config.insert("voice_control".to_string(), TomlValue::Boolean(self.voice_control));
        
        // Save gesture navigation
        config.insert("gesture_navigation".to_string(), TomlValue::Boolean(self.gesture_navigation));
        
        Ok(config)
    }
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            high_contrast: false,
            color_correction: false,
            color_correction_type: "deuteranomaly".to_string(),
            text_to_speech: false,
            tts_rate: 1.0,
            tts_pitch: 1.0,
            screen_reader: false,
            magnification: false,
            magnification_level: 1.5,
            subtitles: false,
            subtitle_size: "medium".to_string(),
            mono_audio: false,
            audio_balance: false,
            audio_balance_value: 0.0,
            reduced_motion: false,
            reduced_transparency: false,
            touch_accommodations: false,
            touch_hold_duration_ms: 500,
            switch_control: false,
            voice_control: false,
            gesture_navigation: true,
        }
    }
}

/// Language configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// System language
    pub system_language: String,
    
    /// Keyboard language
    pub keyboard_language: String,
    
    /// Voice recognition language
    pub voice_language: String,
    
    /// Whether to enable automatic language detection
    pub auto_detect: bool,
    
    /// Secondary languages
    pub secondary_languages: Vec<String>,
    
    /// Whether to enable spell checking
    pub spell_check: bool,
    
    /// Whether to enable autocorrect
    pub autocorrect: bool,
    
    /// Whether to enable predictive text
    pub predictive_text: bool,
    
    /// Whether to enable swipe typing
    pub swipe_typing: bool,
    
    /// Whether to enable voice typing
    pub voice_typing: bool,
    
    /// Whether to enable multilingual support
    pub multilingual: bool,
    
    /// Whether to enable translation
    pub translation: bool,
    
    /// Translation target language
    pub translation_target: String,
}

impl LanguageConfig {
    /// Load language configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load system language
        if let Some(TomlValue::String(system_language)) = config.get("system_language") {
            self.system_language = system_language.clone();
            // Validate system language
            if !validation::is_valid_language_code(&self.system_language) {
                return Err(ConfigError::ValidationError(
                    "System language must be a valid language code".to_string()
                ));
            }
        }
        
        // Load keyboard language
        if let Some(TomlValue::String(keyboard_language)) = config.get("keyboard_language") {
            self.keyboard_language = keyboard_language.clone();
            // Validate keyboard language
            if !validation::is_valid_language_code(&self.keyboard_language) {
                return Err(ConfigError::ValidationError(
                    "Keyboard language must be a valid language code".to_string()
                ));
            }
        }
        
        // Load voice language
        if let Some(TomlValue::String(voice_language)) = config.get("voice_language") {
            self.voice_language = voice_language.clone();
            // Validate voice language
            if !validation::is_valid_language_code(&self.voice_language) {
                return Err(ConfigError::ValidationError(
                    "Voice language must be a valid language code".to_string()
                ));
            }
        }
        
        // Load auto detect
        if let Some(TomlValue::Boolean(auto_detect)) = config.get("auto_detect") {
            self.auto_detect = *auto_detect;
        }
        
        // Load secondary languages
        if let Some(TomlValue::Array(languages)) = config.get("secondary_languages") {
            self.secondary_languages.clear();
            for language in languages {
                if let TomlValue::String(language_str) = language {
                    // Validate language code
                    if !validation::is_valid_language_code(language_str) {
                        return Err(ConfigError::ValidationError(
                            format!("Invalid language code: {}", language_str)
                        ));
                    }
                    self.secondary_languages.push(language_str.clone());
                }
            }
        }
        
        // Load spell check
        if let Some(TomlValue::Boolean(spell_check)) = config.get("spell_check") {
            self.spell_check = *spell_check;
        }
        
        // Load autocorrect
        if let Some(TomlValue::Boolean(autocorrect)) = config.get("autocorrect") {
            self.autocorrect = *autocorrect;
        }
        
        // Load predictive text
        if let Some(TomlValue::Boolean(predictive_text)) = config.get("predictive_text") {
            self.predictive_text = *predictive_text;
        }
        
        // Load swipe typing
        if let Some(TomlValue::Boolean(swipe_typing)) = config.get("swipe_typing") {
            self.swipe_typing = *swipe_typing;
        }
        
        // Load voice typing
        if let Some(TomlValue::Boolean(voice_typing)) = config.get("voice_typing") {
            self.voice_typing = *voice_typing;
        }
        
        // Load multilingual
        if let Some(TomlValue::Boolean(multilingual)) = config.get("multilingual") {
            self.multilingual = *multilingual;
        }
        
        // Load translation
        if let Some(TomlValue::Boolean(translation)) = config.get("translation") {
            self.translation = *translation;
        }
        
        // Load translation target
        if let Some(TomlValue::String(translation_target)) = config.get("translation_target") {
            self.translation_target = translation_target.clone();
            // Validate translation target
            if !validation::is_valid_language_code(&self.translation_target) {
                return Err(ConfigError::ValidationError(
                    "Translation target must be a valid language code".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Save language configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save system language
        config.insert("system_language".to_string(), TomlValue::String(self.system_language.clone()));
        
        // Save keyboard language
        config.insert("keyboard_language".to_string(), TomlValue::String(self.keyboard_language.clone()));
        
        // Save voice language
        config.insert("voice_language".to_string(), TomlValue::String(self.voice_language.clone()));
        
        // Save auto detect
        config.insert("auto_detect".to_string(), TomlValue::Boolean(self.auto_detect));
        
        // Save secondary languages
        let secondary_languages: Vec<TomlValue> = self.secondary_languages.iter()
            .map(|language| TomlValue::String(language.clone()))
            .collect();
        config.insert("secondary_languages".to_string(), TomlValue::Array(secondary_languages));
        
        // Save spell check
        config.insert("spell_check".to_string(), TomlValue::Boolean(self.spell_check));
        
        // Save autocorrect
        config.insert("autocorrect".to_string(), TomlValue::Boolean(self.autocorrect));
        
        // Save predictive text
        config.insert("predictive_text".to_string(), TomlValue::Boolean(self.predictive_text));
        
        // Save swipe typing
        config.insert("swipe_typing".to_string(), TomlValue::Boolean(self.swipe_typing));
        
        // Save voice typing
        config.insert("voice_typing".to_string(), TomlValue::Boolean(self.voice_typing));
        
        // Save multilingual
        config.insert("multilingual".to_string(), TomlValue::Boolean(self.multilingual));
        
        // Save translation
        config.insert("translation".to_string(), TomlValue::Boolean(self.translation));
        
        // Save translation target
        config.insert("translation_target".to_string(), TomlValue::String(self.translation_target.clone()));
        
        Ok(config)
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            system_language: "en-US".to_string(),
            keyboard_language: "en-US".to_string(),
            voice_language: "en-US".to_string(),
            auto_detect: true,
            secondary_languages: Vec::new(),
            spell_check: true,
            autocorrect: true,
            predictive_text: true,
            swipe_typing: true,
            voice_typing: true,
            multilingual: false,
            translation: false,
            translation_target: "en-US".to_string(),
        }
    }
}

/// Time configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    /// Whether to use 24-hour format
    pub use_24_hour: bool,
    
    /// Whether to use automatic time zone
    pub auto_timezone: bool,
    
    /// Time zone
    pub timezone: String,
    
    /// Whether to use automatic time
    pub auto_time: bool,
    
    /// Whether to use automatic date
    pub auto_date: bool,
    
    /// Date format (MM/DD/YYYY, DD/MM/YYYY, YYYY-MM-DD)
    pub date_format: String,
    
    /// Time format (HH:MM, HH:MM:SS)
    pub time_format: String,
    
    /// Whether to show seconds
    pub show_seconds: bool,
    
    /// Whether to show AM/PM
    pub show_am_pm: bool,
    
    /// Whether to show day of week
    pub show_day_of_week: bool,
    
    /// Whether to show date in status bar
    pub show_date_in_status: bool,
    
    /// Whether to show time in status bar
    pub show_time_in_status: bool,
    
    /// Whether to use NTP
    pub use_ntp: bool,
    
    /// NTP server
    pub ntp_server: String,
}

impl TimeConfig {
    /// Load time configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load 24-hour format
        if let Some(TomlValue::Boolean(use_24_hour)) = config.get("use_24_hour") {
            self.use_24_hour = *use_24_hour;
        }
        
        // Load auto timezone
        if let Some(TomlValue::Boolean(auto_timezone)) = config.get("auto_timezone") {
            self.auto_timezone = *auto_timezone;
        }
        
        // Load timezone
        if let Some(TomlValue::String(timezone)) = config.get("timezone") {
            self.timezone = timezone.clone();
            // Validate timezone
            if !validation::is_valid_timezone(&self.timezone) {
                return Err(ConfigError::ValidationError(
                    "Timezone must be a valid IANA timezone".to_string()
                ));
            }
        }
        
        // Load auto time
        if let Some(TomlValue::Boolean(auto_time)) = config.get("auto_time") {
            self.auto_time = *auto_time;
        }
        
        // Load auto date
        if let Some(TomlValue::Boolean(auto_date)) = config.get("auto_date") {
            self.auto_date = *auto_date;
        }
        
        // Load date format
        if let Some(TomlValue::String(date_format)) = config.get("date_format") {
            self.date_format = date_format.clone();
            // Validate date format
            if self.date_format != "MM/DD/YYYY" && self.date_format != "DD/MM/YYYY" && self.date_format != "YYYY-MM-DD" {
                return Err(ConfigError::ValidationError(
                    "Date format must be 'MM/DD/YYYY', 'DD/MM/YYYY', or 'YYYY-MM-DD'".to_string()
                ));
            }
        }
        
        // Load time format
        if let Some(TomlValue::String(time_format)) = config.get("time_format") {
            self.time_format = time_format.clone();
            // Validate time format
            if self.time_format != "HH:MM" && self.time_format != "HH:MM:SS" {
                return Err(ConfigError::ValidationError(
                    "Time format must be 'HH:MM' or 'HH:MM:SS'".to_string()
                ));
            }
        }
        
        // Load show seconds
        if let Some(TomlValue::Boolean(show_seconds)) = config.get("show_seconds") {
            self.show_seconds = *show_seconds;
        }
        
        // Load show AM/PM
        if let Some(TomlValue::Boolean(show_am_pm)) = config.get("show_am_pm") {
            self.show_am_pm = *show_am_pm;
        }
        
        // Load show day of week
        if let Some(TomlValue::Boolean(show_day_of_week)) = config.get("show_day_of_week") {
            self.show_day_of_week = *show_day_of_week;
        }
        
        // Load show date in status
        if let Some(TomlValue::Boolean(show_date_in_status)) = config.get("show_date_in_status") {
            self.show_date_in_status = *show_date_in_status;
        }
        
        // Load show time in status
        if let Some(TomlValue::Boolean(show_time_in_status)) = config.get("show_time_in_status") {
            self.show_time_in_status = *show_time_in_status;
        }
        
        // Load use NTP
        if let Some(TomlValue::Boolean(use_ntp)) = config.get("use_ntp") {
            self.use_ntp = *use_ntp;
        }
        
        // Load NTP server
        if let Some(TomlValue::String(ntp_server)) = config.get("ntp_server") {
            self.ntp_server = ntp_server.clone();
        }
        
        Ok(())
    }
    
    /// Save time configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save 24-hour format
        config.insert("use_24_hour".to_string(), TomlValue::Boolean(self.use_24_hour));
        
        // Save auto timezone
        config.insert("auto_timezone".to_string(), TomlValue::Boolean(self.auto_timezone));
        
        // Save timezone
        config.insert("timezone".to_string(), TomlValue::String(self.timezone.clone()));
        
        // Save auto time
        config.insert("auto_time".to_string(), TomlValue::Boolean(self.auto_time));
        
        // Save auto date
        config.insert("auto_date".to_string(), TomlValue::Boolean(self.auto_date));
        
        // Save date format
        config.insert("date_format".to_string(), TomlValue::String(self.date_format.clone()));
        
        // Save time format
        config.insert("time_format".to_string(), TomlValue::String(self.time_format.clone()));
        
        // Save show seconds
        config.insert("show_seconds".to_string(), TomlValue::Boolean(self.show_seconds));
        
        // Save show AM/PM
        config.insert("show_am_pm".to_string(), TomlValue::Boolean(self.show_am_pm));
        
        // Save show day of week
        config.insert("show_day_of_week".to_string(), TomlValue::Boolean(self.show_day_of_week));
        
        // Save show date in status
        config.insert("show_date_in_status".to_string(), TomlValue::Boolean(self.show_date_in_status));
        
        // Save show time in status
        config.insert("show_time_in_status".to_string(), TomlValue::Boolean(self.show_time_in_status));
        
        // Save use NTP
        config.insert("use_ntp".to_string(), TomlValue::Boolean(self.use_ntp));
        
        // Save NTP server
        config.insert("ntp_server".to_string(), TomlValue::String(self.ntp_server.clone()));
        
        Ok(config)
    }
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            use_24_hour: false,
            auto_timezone: true,
            timezone: "America/New_York".to_string(),
            auto_time: true,
            auto_date: true,
            date_format: "MM/DD/YYYY".to_string(),
            time_format: "HH:MM".to_string(),
            show_seconds: false,
            show_am_pm: true,
            show_day_of_week: true,
            show_date_in_status: true,
            show_time_in_status: true,
            use_ntp: true,
            ntp_server: "pool.ntp.org".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_config_load_save() {
        let mut config = PerformanceConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("mode".to_string(), TomlValue::String("performance".to_string()));
        toml_values.insert("cpu_level".to_string(), TomlValue::Integer(90));
        toml_values.insert("gpu_level".to_string(), TomlValue::Integer(95));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.mode, "performance");
        assert_eq!(config.cpu_level, 90);
        assert_eq!(config.gpu_level, 95);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("mode"), Some(&TomlValue::String("performance".to_string())));
        assert_eq!(saved.get("cpu_level"), Some(&TomlValue::Integer(90)));
        assert_eq!(saved.get("gpu_level"), Some(&TomlValue::Integer(95)));
    }
    
    #[test]
    fn test_update_config_load_save() {
        let mut config = UpdateConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("auto_update".to_string(), TomlValue::Boolean(false));
        toml_values.insert("check_on_startup".to_string(), TomlValue::Boolean(false));
        toml_values.insert("check_interval_hours".to_string(), TomlValue::Integer(12));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.auto_update, false);
        assert_eq!(config.check_on_startup, false);
        assert_eq!(config.check_interval_hours, 12);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("auto_update"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("check_on_startup"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("check_interval_hours"), Some(&TomlValue::Integer(12)));
    }
    
    #[test]
    fn test_security_config_load_save() {
        let mut config = SecurityConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("pin_protection".to_string(), TomlValue::Boolean(true));
        toml_values.insert("pin_hash".to_string(), TomlValue::String("abcdef123456".to_string()));
        toml_values.insert("biometric_auth".to_string(), TomlValue::Boolean(true));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.pin_protection, true);
        assert_eq!(config.pin_hash, "abcdef123456");
        assert_eq!(config.biometric_auth, true);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("pin_protection"), Some(&TomlValue::Boolean(true)));
        assert_eq!(saved.get("pin_hash"), Some(&TomlValue::String("abcdef123456".to_string())));
        assert_eq!(saved.get("biometric_auth"), Some(&TomlValue::Boolean(true)));
    }
    
    #[test]
    fn test_system_config_load_save() {
        let config = SystemConfig::new();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        
        let mut performance = HashMap::new();
        performance.insert("mode".to_string(), TomlValue::String("performance".to_string()));
        performance.insert("cpu_level".to_string(), TomlValue::Integer(90));
        toml_values.insert("performance".to_string(), TomlValue::Table(performance));
        
        let mut update = HashMap::new();
        update.insert("auto_update".to_string(), TomlValue::Boolean(false));
        update.insert("check_on_startup".to_string(), TomlValue::Boolean(false));
        toml_values.insert("update".to_string(), TomlValue::Table(update));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify performance values were saved correctly
        if let Some(TomlValue::Table(performance)) = saved.get("performance") {
            assert_eq!(performance.get("mode"), Some(&TomlValue::String("performance".to_string())));
            assert_eq!(performance.get("cpu_level"), Some(&TomlValue::Integer(90)));
        } else {
            panic!("Expected performance table");
        }
        
        // Verify update values were saved correctly
        if let Some(TomlValue::Table(update)) = saved.get("update") {
            assert_eq!(update.get("auto_update"), Some(&TomlValue::Boolean(false)));
            assert_eq!(update.get("check_on_startup"), Some(&TomlValue::Boolean(false)));
        } else {
            panic!("Expected update table");
        }
    }
}
