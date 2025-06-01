//! User configuration module for the VR Core API.
//!
//! This module provides comprehensive configuration management for all user-related
//! settings, including profiles, notifications, privacy, appearance, input, and comfort.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use super::{ConfigError, ConfigResult, validation};

/// User configuration manager.
#[derive(Debug)]
pub struct UserConfig {
    /// Profile configuration
    profile: RwLock<ProfileConfig>,
    
    /// Notification configuration
    notification: RwLock<NotificationConfig>,
    
    /// Privacy configuration
    privacy: RwLock<PrivacyConfig>,
    
    /// Appearance configuration
    appearance: RwLock<AppearanceConfig>,
    
    /// Input configuration
    input: RwLock<InputConfig>,
    
    /// Comfort configuration
    comfort: RwLock<ComfortConfig>,
}

impl UserConfig {
    /// Create a new user configuration manager.
    pub fn new() -> Self {
        Self {
            profile: RwLock::new(ProfileConfig::default()),
            notification: RwLock::new(NotificationConfig::default()),
            privacy: RwLock::new(PrivacyConfig::default()),
            appearance: RwLock::new(AppearanceConfig::default()),
            input: RwLock::new(InputConfig::default()),
            comfort: RwLock::new(ComfortConfig::default()),
        }
    }
    
    /// Load user configuration from TOML values.
    pub fn load_from_toml(&self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load profile configuration
        if let Some(TomlValue::Table(profile_table)) = config.get("profile") {
            let mut profile = self.profile.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for profile config".to_string())
            })?;
            profile.load_from_toml(profile_table)?;
        }
        
        // Load notification configuration
        if let Some(TomlValue::Table(notification_table)) = config.get("notification") {
            let mut notification = self.notification.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for notification config".to_string())
            })?;
            notification.load_from_toml(notification_table)?;
        }
        
        // Load privacy configuration
        if let Some(TomlValue::Table(privacy_table)) = config.get("privacy") {
            let mut privacy = self.privacy.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for privacy config".to_string())
            })?;
            privacy.load_from_toml(privacy_table)?;
        }
        
        // Load appearance configuration
        if let Some(TomlValue::Table(appearance_table)) = config.get("appearance") {
            let mut appearance = self.appearance.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for appearance config".to_string())
            })?;
            appearance.load_from_toml(appearance_table)?;
        }
        
        // Load input configuration
        if let Some(TomlValue::Table(input_table)) = config.get("input") {
            let mut input = self.input.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for input config".to_string())
            })?;
            input.load_from_toml(input_table)?;
        }
        
        // Load comfort configuration
        if let Some(TomlValue::Table(comfort_table)) = config.get("comfort") {
            let mut comfort = self.comfort.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for comfort config".to_string())
            })?;
            comfort.load_from_toml(comfort_table)?;
        }
        
        Ok(())
    }
    
    /// Save user configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save profile configuration
        let profile = self.profile.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for profile config".to_string())
        })?;
        config.insert("profile".to_string(), TomlValue::Table(profile.save_to_toml()?));
        
        // Save notification configuration
        let notification = self.notification.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for notification config".to_string())
        })?;
        config.insert("notification".to_string(), TomlValue::Table(notification.save_to_toml()?));
        
        // Save privacy configuration
        let privacy = self.privacy.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for privacy config".to_string())
        })?;
        config.insert("privacy".to_string(), TomlValue::Table(privacy.save_to_toml()?));
        
        // Save appearance configuration
        let appearance = self.appearance.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for appearance config".to_string())
        })?;
        config.insert("appearance".to_string(), TomlValue::Table(appearance.save_to_toml()?));
        
        // Save input configuration
        let input = self.input.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for input config".to_string())
        })?;
        config.insert("input".to_string(), TomlValue::Table(input.save_to_toml()?));
        
        // Save comfort configuration
        let comfort = self.comfort.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for comfort config".to_string())
        })?;
        config.insert("comfort".to_string(), TomlValue::Table(comfort.save_to_toml()?));
        
        Ok(config)
    }
    
    /// Get profile configuration.
    pub fn profile(&self) -> ConfigResult<ProfileConfig> {
        let profile = self.profile.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for profile config".to_string())
        })?;
        Ok(profile.clone())
    }
    
    /// Update profile configuration.
    pub fn update_profile(&self, config: ProfileConfig) -> ConfigResult<()> {
        let mut profile = self.profile.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for profile config".to_string())
        })?;
        *profile = config;
        Ok(())
    }
    
    /// Get notification configuration.
    pub fn notification(&self) -> ConfigResult<NotificationConfig> {
        let notification = self.notification.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for notification config".to_string())
        })?;
        Ok(notification.clone())
    }
    
    /// Update notification configuration.
    pub fn update_notification(&self, config: NotificationConfig) -> ConfigResult<()> {
        let mut notification = self.notification.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for notification config".to_string())
        })?;
        *notification = config;
        Ok(())
    }
    
    /// Get privacy configuration.
    pub fn privacy(&self) -> ConfigResult<PrivacyConfig> {
        let privacy = self.privacy.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for privacy config".to_string())
        })?;
        Ok(privacy.clone())
    }
    
    /// Update privacy configuration.
    pub fn update_privacy(&self, config: PrivacyConfig) -> ConfigResult<()> {
        let mut privacy = self.privacy.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for privacy config".to_string())
        })?;
        *privacy = config;
        Ok(())
    }
    
    /// Get appearance configuration.
    pub fn appearance(&self) -> ConfigResult<AppearanceConfig> {
        let appearance = self.appearance.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for appearance config".to_string())
        })?;
        Ok(appearance.clone())
    }
    
    /// Update appearance configuration.
    pub fn update_appearance(&self, config: AppearanceConfig) -> ConfigResult<()> {
        let mut appearance = self.appearance.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for appearance config".to_string())
        })?;
        *appearance = config;
        Ok(())
    }
    
    /// Get input configuration.
    pub fn input(&self) -> ConfigResult<InputConfig> {
        let input = self.input.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for input config".to_string())
        })?;
        Ok(input.clone())
    }
    
    /// Update input configuration.
    pub fn update_input(&self, config: InputConfig) -> ConfigResult<()> {
        let mut input = self.input.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for input config".to_string())
        })?;
        *input = config;
        Ok(())
    }
    
    /// Get comfort configuration.
    pub fn comfort(&self) -> ConfigResult<ComfortConfig> {
        let comfort = self.comfort.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for comfort config".to_string())
        })?;
        Ok(comfort.clone())
    }
    
    /// Update comfort configuration.
    pub fn update_comfort(&self, config: ComfortConfig) -> ConfigResult<()> {
        let mut comfort = self.comfort.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for comfort config".to_string())
        })?;
        *comfort = config;
        Ok(())
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Current user ID
    pub current_user_id: String,
    
    /// User profiles (map of user ID to profile data)
    pub profiles: HashMap<String, UserProfile>,
    
    /// Whether to enable guest mode
    pub guest_mode: bool,
    
    /// Whether to enable profile switching
    pub profile_switching: bool,
    
    /// Whether to enable automatic profile selection
    pub auto_profile_selection: bool,
    
    /// Whether to enable profile synchronization
    pub profile_sync: bool,
    
    /// Profile sync interval in minutes
    pub profile_sync_interval_min: u32,
}

/// User profile data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User name
    pub name: String,
    
    /// User avatar URL
    pub avatar_url: String,
    
    /// User preferences (map of preference key to value)
    pub preferences: HashMap<String, String>,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// User roles
    pub roles: Vec<String>,
    
    /// Last login timestamp
    pub last_login: Option<String>,
}

impl ProfileConfig {
    /// Load profile configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load current user ID
        if let Some(TomlValue::String(current_user_id)) = config.get("current_user_id") {
            self.current_user_id = current_user_id.clone();
        }
        
        // Load profiles
        if let Some(TomlValue::Table(profiles_table)) = config.get("profiles") {
            self.profiles.clear();
            for (user_id, profile_value) in profiles_table {
                if let TomlValue::Table(profile_data) = profile_value {
                    let mut profile = UserProfile::default();
                    profile.load_from_toml(profile_data)?;
                    self.profiles.insert(user_id.clone(), profile);
                }
            }
        }
        
        // Load guest mode
        if let Some(TomlValue::Boolean(guest_mode)) = config.get("guest_mode") {
            self.guest_mode = *guest_mode;
        }
        
        // Load profile switching
        if let Some(TomlValue::Boolean(profile_switching)) = config.get("profile_switching") {
            self.profile_switching = *profile_switching;
        }
        
        // Load auto profile selection
        if let Some(TomlValue::Boolean(auto_profile_selection)) = config.get("auto_profile_selection") {
            self.auto_profile_selection = *auto_profile_selection;
        }
        
        // Load profile sync
        if let Some(TomlValue::Boolean(profile_sync)) = config.get("profile_sync") {
            self.profile_sync = *profile_sync;
        }
        
        // Load profile sync interval
        if let Some(TomlValue::Integer(profile_sync_interval_min)) = config.get("profile_sync_interval_min") {
            self.profile_sync_interval_min = *profile_sync_interval_min as u32;
        }
        
        Ok(())
    }
    
    /// Save profile configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save current user ID
        config.insert("current_user_id".to_string(), TomlValue::String(self.current_user_id.clone()));
        
        // Save profiles
        let mut profiles_table = HashMap::new();
        for (user_id, profile) in &self.profiles {
            profiles_table.insert(user_id.clone(), TomlValue::Table(profile.save_to_toml()?));
        }
        config.insert("profiles".to_string(), TomlValue::Table(profiles_table));
        
        // Save guest mode
        config.insert("guest_mode".to_string(), TomlValue::Boolean(self.guest_mode));
        
        // Save profile switching
        config.insert("profile_switching".to_string(), TomlValue::Boolean(self.profile_switching));
        
        // Save auto profile selection
        config.insert("auto_profile_selection".to_string(), TomlValue::Boolean(self.auto_profile_selection));
        
        // Save profile sync
        config.insert("profile_sync".to_string(), TomlValue::Boolean(self.profile_sync));
        
        // Save profile sync interval
        config.insert("profile_sync_interval_min".to_string(), TomlValue::Integer(self.profile_sync_interval_min as i64));
        
        Ok(config)
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            current_user_id: "default".to_string(),
            profiles: HashMap::new(),
            guest_mode: false,
            profile_switching: true,
            auto_profile_selection: false,
            profile_sync: false,
            profile_sync_interval_min: 60,
        }
    }
}

impl UserProfile {
    /// Load user profile data from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load name
        if let Some(TomlValue::String(name)) = config.get("name") {
            self.name = name.clone();
        }
        
        // Load avatar URL
        if let Some(TomlValue::String(avatar_url)) = config.get("avatar_url") {
            self.avatar_url = avatar_url.clone();
        }
        
        // Load preferences
        if let Some(TomlValue::Table(preferences_table)) = config.get("preferences") {
            self.preferences.clear();
            for (key, value) in preferences_table {
                if let TomlValue::String(value_str) = value {
                    self.preferences.insert(key.clone(), value_str.clone());
                }
            }
        }
        
        // Load permissions
        if let Some(TomlValue::Array(permissions_array)) = config.get("permissions") {
            self.permissions.clear();
            for permission in permissions_array {
                if let TomlValue::String(permission_str) = permission {
                    self.permissions.push(permission_str.clone());
                }
            }
        }
        
        // Load roles
        if let Some(TomlValue::Array(roles_array)) = config.get("roles") {
            self.roles.clear();
            for role in roles_array {
                if let TomlValue::String(role_str) = role {
                    self.roles.push(role_str.clone());
                }
            }
        }
        
        // Load last login
        if let Some(TomlValue::String(last_login)) = config.get("last_login") {
            self.last_login = Some(last_login.clone());
        }
        
        Ok(())
    }
    
    /// Save user profile data to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save name
        config.insert("name".to_string(), TomlValue::String(self.name.clone()));
        
        // Save avatar URL
        config.insert("avatar_url".to_string(), TomlValue::String(self.avatar_url.clone()));
        
        // Save preferences
        let mut preferences_table = HashMap::new();
        for (key, value) in &self.preferences {
            preferences_table.insert(key.clone(), TomlValue::String(value.clone()));
        }
        config.insert("preferences".to_string(), TomlValue::Table(preferences_table));
        
        // Save permissions
        let permissions_array: Vec<TomlValue> = self.permissions.iter()
            .map(|permission| TomlValue::String(permission.clone()))
            .collect();
        config.insert("permissions".to_string(), TomlValue::Array(permissions_array));
        
        // Save roles
        let roles_array: Vec<TomlValue> = self.roles.iter()
            .map(|role| TomlValue::String(role.clone()))
            .collect();
        config.insert("roles".to_string(), TomlValue::Array(roles_array));
        
        // Save last login
        if let Some(last_login) = &self.last_login {
            config.insert("last_login".to_string(), TomlValue::String(last_login.clone()));
        }
        
        Ok(config)
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            name: "Default User".to_string(),
            avatar_url: String::new(),
            preferences: HashMap::new(),
            permissions: Vec::new(),
            roles: vec!["user".to_string()],
            last_login: None,
        }
    }
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether to enable notifications
    pub enabled: bool,
    
    /// Whether to enable sound notifications
    pub sound_enabled: bool,
    
    /// Notification sound file path
    pub sound_file: String,
    
    /// Whether to enable vibration notifications
    pub vibration_enabled: bool,
    
    /// Whether to enable LED notifications
    pub led_enabled: bool,
    
    /// Whether to enable in-VR notifications
    pub in_vr_notifications: bool,
    
    /// Whether to enable desktop notifications
    pub desktop_notifications: bool,
    
    /// Whether to enable mobile notifications
    pub mobile_notifications: bool,
    
    /// Notification priority level (low, medium, high)
    pub priority_level: String,
    
    /// Whether to enable do not disturb mode
    pub do_not_disturb: bool,
    
    /// Do not disturb start time (HH:MM)
    pub dnd_start_time: String,
    
    /// Do not disturb end time (HH:MM)
    pub dnd_end_time: String,
    
    /// Whether to enable app-specific notifications
    pub app_specific_notifications: bool,
    
    /// App notification settings (map of app ID to settings)
    pub app_settings: HashMap<String, AppNotificationSettings>,
}

/// App-specific notification settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNotificationSettings {
    /// Whether notifications are enabled for this app
    pub enabled: bool,
    
    /// Whether sound is enabled for this app
    pub sound_enabled: bool,
    
    /// Whether vibration is enabled for this app
    pub vibration_enabled: bool,
    
    /// Notification priority level for this app
    pub priority_level: String,
}

impl NotificationConfig {
    /// Load notification configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load sound enabled
        if let Some(TomlValue::Boolean(sound_enabled)) = config.get("sound_enabled") {
            self.sound_enabled = *sound_enabled;
        }
        
        // Load sound file
        if let Some(TomlValue::String(sound_file)) = config.get("sound_file") {
            self.sound_file = sound_file.clone();
        }
        
        // Load vibration enabled
        if let Some(TomlValue::Boolean(vibration_enabled)) = config.get("vibration_enabled") {
            self.vibration_enabled = *vibration_enabled;
        }
        
        // Load LED enabled
        if let Some(TomlValue::Boolean(led_enabled)) = config.get("led_enabled") {
            self.led_enabled = *led_enabled;
        }
        
        // Load in-VR notifications
        if let Some(TomlValue::Boolean(in_vr_notifications)) = config.get("in_vr_notifications") {
            self.in_vr_notifications = *in_vr_notifications;
        }
        
        // Load desktop notifications
        if let Some(TomlValue::Boolean(desktop_notifications)) = config.get("desktop_notifications") {
            self.desktop_notifications = *desktop_notifications;
        }
        
        // Load mobile notifications
        if let Some(TomlValue::Boolean(mobile_notifications)) = config.get("mobile_notifications") {
            self.mobile_notifications = *mobile_notifications;
        }
        
        // Load priority level
        if let Some(TomlValue::String(priority_level)) = config.get("priority_level") {
            self.priority_level = priority_level.clone();
            // Validate priority level
            if self.priority_level != "low" && self.priority_level != "medium" && self.priority_level != "high" {
                return Err(ConfigError::ValidationError(
                    "Notification priority level must be 'low', 'medium', or 'high'".to_string()
                ));
            }
        }
        
        // Load do not disturb
        if let Some(TomlValue::Boolean(do_not_disturb)) = config.get("do_not_disturb") {
            self.do_not_disturb = *do_not_disturb;
        }
        
        // Load DND start time
        if let Some(TomlValue::String(dnd_start_time)) = config.get("dnd_start_time") {
            self.dnd_start_time = dnd_start_time.clone();
            // Validate DND start time format (HH:MM)
            if !validation::is_valid_time_format(&self.dnd_start_time) {
                return Err(ConfigError::ValidationError(
                    "DND start time must be in HH:MM format".to_string()
                ));
            }
        }
        
        // Load DND end time
        if let Some(TomlValue::String(dnd_end_time)) = config.get("dnd_end_time") {
            self.dnd_end_time = dnd_end_time.clone();
            // Validate DND end time format (HH:MM)
            if !validation::is_valid_time_format(&self.dnd_end_time) {
                return Err(ConfigError::ValidationError(
                    "DND end time must be in HH:MM format".to_string()
                ));
            }
        }
        
        // Load app-specific notifications
        if let Some(TomlValue::Boolean(app_specific_notifications)) = config.get("app_specific_notifications") {
            self.app_specific_notifications = *app_specific_notifications;
        }
        
        // Load app settings
        if let Some(TomlValue::Table(app_settings_table)) = config.get("app_settings") {
            self.app_settings.clear();
            for (app_id, settings_value) in app_settings_table {
                if let TomlValue::Table(settings_data) = settings_value {
                    let mut settings = AppNotificationSettings::default();
                    settings.load_from_toml(settings_data)?;
                    self.app_settings.insert(app_id.clone(), settings);
                }
            }
        }
        
        Ok(())
    }
    
    /// Save notification configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save sound enabled
        config.insert("sound_enabled".to_string(), TomlValue::Boolean(self.sound_enabled));
        
        // Save sound file
        config.insert("sound_file".to_string(), TomlValue::String(self.sound_file.clone()));
        
        // Save vibration enabled
        config.insert("vibration_enabled".to_string(), TomlValue::Boolean(self.vibration_enabled));
        
        // Save LED enabled
        config.insert("led_enabled".to_string(), TomlValue::Boolean(self.led_enabled));
        
        // Save in-VR notifications
        config.insert("in_vr_notifications".to_string(), TomlValue::Boolean(self.in_vr_notifications));
        
        // Save desktop notifications
        config.insert("desktop_notifications".to_string(), TomlValue::Boolean(self.desktop_notifications));
        
        // Save mobile notifications
        config.insert("mobile_notifications".to_string(), TomlValue::Boolean(self.mobile_notifications));
        
        // Save priority level
        config.insert("priority_level".to_string(), TomlValue::String(self.priority_level.clone()));
        
        // Save do not disturb
        config.insert("do_not_disturb".to_string(), TomlValue::Boolean(self.do_not_disturb));
        
        // Save DND start time
        config.insert("dnd_start_time".to_string(), TomlValue::String(self.dnd_start_time.clone()));
        
        // Save DND end time
        config.insert("dnd_end_time".to_string(), TomlValue::String(self.dnd_end_time.clone()));
        
        // Save app-specific notifications
        config.insert("app_specific_notifications".to_string(), TomlValue::Boolean(self.app_specific_notifications));
        
        // Save app settings
        let mut app_settings_table = HashMap::new();
        for (app_id, settings) in &self.app_settings {
            app_settings_table.insert(app_id.clone(), TomlValue::Table(settings.save_to_toml()?));
        }
        config.insert("app_settings".to_string(), TomlValue::Table(app_settings_table));
        
        Ok(config)
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            sound_file: "/path/to/default/notification.wav".to_string(),
            vibration_enabled: true,
            led_enabled: true,
            in_vr_notifications: true,
            desktop_notifications: false,
            mobile_notifications: false,
            priority_level: "medium".to_string(),
            do_not_disturb: false,
            dnd_start_time: "22:00".to_string(),
            dnd_end_time: "07:00".to_string(),
            app_specific_notifications: true,
            app_settings: HashMap::new(),
        }
    }
}

impl AppNotificationSettings {
    /// Load app notification settings from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load sound enabled
        if let Some(TomlValue::Boolean(sound_enabled)) = config.get("sound_enabled") {
            self.sound_enabled = *sound_enabled;
        }
        
        // Load vibration enabled
        if let Some(TomlValue::Boolean(vibration_enabled)) = config.get("vibration_enabled") {
            self.vibration_enabled = *vibration_enabled;
        }
        
        // Load priority level
        if let Some(TomlValue::String(priority_level)) = config.get("priority_level") {
            self.priority_level = priority_level.clone();
            // Validate priority level
            if self.priority_level != "low" && self.priority_level != "medium" && self.priority_level != "high" {
                return Err(ConfigError::ValidationError(
                    "App notification priority level must be 'low', 'medium', or 'high'".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Save app notification settings to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save sound enabled
        config.insert("sound_enabled".to_string(), TomlValue::Boolean(self.sound_enabled));
        
        // Save vibration enabled
        config.insert("vibration_enabled".to_string(), TomlValue::Boolean(self.vibration_enabled));
        
        // Save priority level
        config.insert("priority_level".to_string(), TomlValue::String(self.priority_level.clone()));
        
        Ok(config)
    }
}

impl Default for AppNotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            vibration_enabled: true,
            priority_level: "medium".to_string(),
        }
    }
}

/// Privacy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Whether to enable data collection
    pub data_collection: bool,
    
    /// Whether to enable personalized ads
    pub personalized_ads: bool,
    
    /// Whether to enable location tracking
    pub location_tracking: bool,
    
    /// Whether to enable activity tracking
    pub activity_tracking: bool,
    
    /// Whether to enable microphone access for voice commands
    pub microphone_access: bool,
    
    /// Whether to enable camera access for tracking
    pub camera_access: bool,
    
    /// Whether to enable sharing usage data
    pub share_usage_data: bool,
    
    /// Whether to enable sharing crash reports
    pub share_crash_reports: bool,
    
    /// Whether to enable incognito mode
    pub incognito_mode: bool,
    
    /// Whether to clear browsing data on exit
    pub clear_data_on_exit: bool,
    
    /// Whether to enable tracking protection
    pub tracking_protection: bool,
    
    /// Whether to enable ad blocking
    pub ad_blocking: bool,
    
    /// Whether to enable privacy dashboard
    pub privacy_dashboard: bool,
    
    /// Whether to enable privacy notifications
    pub privacy_notifications: bool,
}

impl PrivacyConfig {
    /// Load privacy configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load data collection
        if let Some(TomlValue::Boolean(data_collection)) = config.get("data_collection") {
            self.data_collection = *data_collection;
        }
        
        // Load personalized ads
        if let Some(TomlValue::Boolean(personalized_ads)) = config.get("personalized_ads") {
            self.personalized_ads = *personalized_ads;
        }
        
        // Load location tracking
        if let Some(TomlValue::Boolean(location_tracking)) = config.get("location_tracking") {
            self.location_tracking = *location_tracking;
        }
        
        // Load activity tracking
        if let Some(TomlValue::Boolean(activity_tracking)) = config.get("activity_tracking") {
            self.activity_tracking = *activity_tracking;
        }
        
        // Load microphone access
        if let Some(TomlValue::Boolean(microphone_access)) = config.get("microphone_access") {
            self.microphone_access = *microphone_access;
        }
        
        // Load camera access
        if let Some(TomlValue::Boolean(camera_access)) = config.get("camera_access") {
            self.camera_access = *camera_access;
        }
        
        // Load share usage data
        if let Some(TomlValue::Boolean(share_usage_data)) = config.get("share_usage_data") {
            self.share_usage_data = *share_usage_data;
        }
        
        // Load share crash reports
        if let Some(TomlValue::Boolean(share_crash_reports)) = config.get("share_crash_reports") {
            self.share_crash_reports = *share_crash_reports;
        }
        
        // Load incognito mode
        if let Some(TomlValue::Boolean(incognito_mode)) = config.get("incognito_mode") {
            self.incognito_mode = *incognito_mode;
        }
        
        // Load clear data on exit
        if let Some(TomlValue::Boolean(clear_data_on_exit)) = config.get("clear_data_on_exit") {
            self.clear_data_on_exit = *clear_data_on_exit;
        }
        
        // Load tracking protection
        if let Some(TomlValue::Boolean(tracking_protection)) = config.get("tracking_protection") {
            self.tracking_protection = *tracking_protection;
        }
        
        // Load ad blocking
        if let Some(TomlValue::Boolean(ad_blocking)) = config.get("ad_blocking") {
            self.ad_blocking = *ad_blocking;
        }
        
        // Load privacy dashboard
        if let Some(TomlValue::Boolean(privacy_dashboard)) = config.get("privacy_dashboard") {
            self.privacy_dashboard = *privacy_dashboard;
        }
        
        // Load privacy notifications
        if let Some(TomlValue::Boolean(privacy_notifications)) = config.get("privacy_notifications") {
            self.privacy_notifications = *privacy_notifications;
        }
        
        Ok(())
    }
    
    /// Save privacy configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save data collection
        config.insert("data_collection".to_string(), TomlValue::Boolean(self.data_collection));
        
        // Save personalized ads
        config.insert("personalized_ads".to_string(), TomlValue::Boolean(self.personalized_ads));
        
        // Save location tracking
        config.insert("location_tracking".to_string(), TomlValue::Boolean(self.location_tracking));
        
        // Save activity tracking
        config.insert("activity_tracking".to_string(), TomlValue::Boolean(self.activity_tracking));
        
        // Save microphone access
        config.insert("microphone_access".to_string(), TomlValue::Boolean(self.microphone_access));
        
        // Save camera access
        config.insert("camera_access".to_string(), TomlValue::Boolean(self.camera_access));
        
        // Save share usage data
        config.insert("share_usage_data".to_string(), TomlValue::Boolean(self.share_usage_data));
        
        // Save share crash reports
        config.insert("share_crash_reports".to_string(), TomlValue::Boolean(self.share_crash_reports));
        
        // Save incognito mode
        config.insert("incognito_mode".to_string(), TomlValue::Boolean(self.incognito_mode));
        
        // Save clear data on exit
        config.insert("clear_data_on_exit".to_string(), TomlValue::Boolean(self.clear_data_on_exit));
        
        // Save tracking protection
        config.insert("tracking_protection".to_string(), TomlValue::Boolean(self.tracking_protection));
        
        // Save ad blocking
        config.insert("ad_blocking".to_string(), TomlValue::Boolean(self.ad_blocking));
        
        // Save privacy dashboard
        config.insert("privacy_dashboard".to_string(), TomlValue::Boolean(self.privacy_dashboard));
        
        // Save privacy notifications
        config.insert("privacy_notifications".to_string(), TomlValue::Boolean(self.privacy_notifications));
        
        Ok(config)
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            data_collection: true,
            personalized_ads: true,
            location_tracking: true,
            activity_tracking: true,
            microphone_access: true,
            camera_access: true,
            share_usage_data: true,
            share_crash_reports: true,
            incognito_mode: false,
            clear_data_on_exit: false,
            tracking_protection: true,
            ad_blocking: false,
            privacy_dashboard: true,
            privacy_notifications: true,
        }
    }
}

/// Appearance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// UI theme (light, dark, system)
    pub theme: String,
    
    /// Accent color (hex code)
    pub accent_color: String,
    
    /// Font size (small, medium, large)
    pub font_size: String,
    
    /// Font family
    pub font_family: String,
    
    /// Whether to enable custom backgrounds
    pub custom_backgrounds: bool,
    
    /// Background image URL
    pub background_image_url: String,
    
    /// Whether to enable UI scaling
    pub ui_scaling: bool,
    
    /// UI scale factor (0.8-1.5)
    pub ui_scale_factor: f32,
    
    /// Whether to enable animations
    pub animations: bool,
    
    /// Whether to enable transparency effects
    pub transparency_effects: bool,
    
    /// Whether to enable custom icons
    pub custom_icons: bool,
    
    /// Icon pack name
    pub icon_pack: String,
    
    /// Whether to enable custom cursors
    pub custom_cursors: bool,
    
    /// Cursor theme name
    pub cursor_theme: String,
}

impl AppearanceConfig {
    /// Load appearance configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load theme
        if let Some(TomlValue::String(theme)) = config.get("theme") {
            self.theme = theme.clone();
            // Validate theme
            if self.theme != "light" && self.theme != "dark" && self.theme != "system" {
                return Err(ConfigError::ValidationError(
                    "Theme must be 'light', 'dark', or 'system'".to_string()
                ));
            }
        }
        
        // Load accent color
        if let Some(TomlValue::String(accent_color)) = config.get("accent_color") {
            self.accent_color = accent_color.clone();
            // Validate accent color format (hex code)
            if !validation::is_valid_hex_color(&self.accent_color) {
                return Err(ConfigError::ValidationError(
                    "Accent color must be a valid hex code".to_string()
                ));
            }
        }
        
        // Load font size
        if let Some(TomlValue::String(font_size)) = config.get("font_size") {
            self.font_size = font_size.clone();
            // Validate font size
            if self.font_size != "small" && self.font_size != "medium" && self.font_size != "large" {
                return Err(ConfigError::ValidationError(
                    "Font size must be 'small', 'medium', or 'large'".to_string()
                ));
            }
        }
        
        // Load font family
        if let Some(TomlValue::String(font_family)) = config.get("font_family") {
            self.font_family = font_family.clone();
        }
        
        // Load custom backgrounds
        if let Some(TomlValue::Boolean(custom_backgrounds)) = config.get("custom_backgrounds") {
            self.custom_backgrounds = *custom_backgrounds;
        }
        
        // Load background image URL
        if let Some(TomlValue::String(background_image_url)) = config.get("background_image_url") {
            self.background_image_url = background_image_url.clone();
        }
        
        // Load UI scaling
        if let Some(TomlValue::Boolean(ui_scaling)) = config.get("ui_scaling") {
            self.ui_scaling = *ui_scaling;
        }
        
        // Load UI scale factor
        if let Some(TomlValue::Float(ui_scale_factor)) = config.get("ui_scale_factor") {
            self.ui_scale_factor = *ui_scale_factor as f32;
            // Validate UI scale factor
            if self.ui_scale_factor < 0.8 || self.ui_scale_factor > 1.5 {
                return Err(ConfigError::ValidationError(
                    "UI scale factor must be between 0.8 and 1.5".to_string()
                ));
            }
        }
        
        // Load animations
        if let Some(TomlValue::Boolean(animations)) = config.get("animations") {
            self.animations = *animations;
        }
        
        // Load transparency effects
        if let Some(TomlValue::Boolean(transparency_effects)) = config.get("transparency_effects") {
            self.transparency_effects = *transparency_effects;
        }
        
        // Load custom icons
        if let Some(TomlValue::Boolean(custom_icons)) = config.get("custom_icons") {
            self.custom_icons = *custom_icons;
        }
        
        // Load icon pack
        if let Some(TomlValue::String(icon_pack)) = config.get("icon_pack") {
            self.icon_pack = icon_pack.clone();
        }
        
        // Load custom cursors
        if let Some(TomlValue::Boolean(custom_cursors)) = config.get("custom_cursors") {
            self.custom_cursors = *custom_cursors;
        }
        
        // Load cursor theme
        if let Some(TomlValue::String(cursor_theme)) = config.get("cursor_theme") {
            self.cursor_theme = cursor_theme.clone();
        }
        
        Ok(())
    }
    
    /// Save appearance configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save theme
        config.insert("theme".to_string(), TomlValue::String(self.theme.clone()));
        
        // Save accent color
        config.insert("accent_color".to_string(), TomlValue::String(self.accent_color.clone()));
        
        // Save font size
        config.insert("font_size".to_string(), TomlValue::String(self.font_size.clone()));
        
        // Save font family
        config.insert("font_family".to_string(), TomlValue::String(self.font_family.clone()));
        
        // Save custom backgrounds
        config.insert("custom_backgrounds".to_string(), TomlValue::Boolean(self.custom_backgrounds));
        
        // Save background image URL
        config.insert("background_image_url".to_string(), TomlValue::String(self.background_image_url.clone()));
        
        // Save UI scaling
        config.insert("ui_scaling".to_string(), TomlValue::Boolean(self.ui_scaling));
        
        // Save UI scale factor
        config.insert("ui_scale_factor".to_string(), TomlValue::Float(self.ui_scale_factor as f64));
        
        // Save animations
        config.insert("animations".to_string(), TomlValue::Boolean(self.animations));
        
        // Save transparency effects
        config.insert("transparency_effects".to_string(), TomlValue::Boolean(self.transparency_effects));
        
        // Save custom icons
        config.insert("custom_icons".to_string(), TomlValue::Boolean(self.custom_icons));
        
        // Save icon pack
        config.insert("icon_pack".to_string(), TomlValue::String(self.icon_pack.clone()));
        
        // Save custom cursors
        config.insert("custom_cursors".to_string(), TomlValue::Boolean(self.custom_cursors));
        
        // Save cursor theme
        config.insert("cursor_theme".to_string(), TomlValue::String(self.cursor_theme.clone()));
        
        Ok(config)
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            accent_color: "#007bff".to_string(),
            font_size: "medium".to_string(),
            font_family: "Roboto".to_string(),
            custom_backgrounds: false,
            background_image_url: String::new(),
            ui_scaling: false,
            ui_scale_factor: 1.0,
            animations: true,
            transparency_effects: true,
            custom_icons: false,
            icon_pack: "default".to_string(),
            custom_cursors: false,
            cursor_theme: "default".to_string(),
        }
    }
}

/// Input configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Controller mapping profile
    pub controller_profile: String,
    
    /// Whether to enable haptic feedback
    pub haptic_feedback: bool,
    
    /// Haptic feedback intensity (0-100)
    pub haptic_intensity: u32,
    
    /// Whether to enable thumbstick deadzone
    pub thumbstick_deadzone: bool,
    
    /// Thumbstick deadzone value (0-50)
    pub thumbstick_deadzone_value: u32,
    
    /// Whether to enable trigger deadzone
    pub trigger_deadzone: bool,
    
    /// Trigger deadzone value (0-50)
    pub trigger_deadzone_value: u32,
    
    /// Whether to enable controller vibration
    pub controller_vibration: bool,
    
    /// Controller vibration intensity (0-100)
    pub vibration_intensity: u32,
    
    /// Whether to enable custom button mapping
    pub custom_button_mapping: bool,
    
    /// Button mapping configuration (map of button name to action)
    pub button_mapping: HashMap<String, String>,
    
    /// Whether to enable gesture controls
    pub gesture_controls: bool,
    
    /// Gesture mapping configuration (map of gesture name to action)
    pub gesture_mapping: HashMap<String, String>,
    
    /// Whether to enable voice commands
    pub voice_commands: bool,
    
    /// Voice command mapping configuration (map of phrase to action)
    pub voice_command_mapping: HashMap<String, String>,
}

impl InputConfig {
    /// Load input configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load controller profile
        if let Some(TomlValue::String(controller_profile)) = config.get("controller_profile") {
            self.controller_profile = controller_profile.clone();
        }
        
        // Load haptic feedback
        if let Some(TomlValue::Boolean(haptic_feedback)) = config.get("haptic_feedback") {
            self.haptic_feedback = *haptic_feedback;
        }
        
        // Load haptic intensity
        if let Some(TomlValue::Integer(haptic_intensity)) = config.get("haptic_intensity") {
            self.haptic_intensity = *haptic_intensity as u32;
            // Validate haptic intensity
            if self.haptic_intensity > 100 {
                return Err(ConfigError::ValidationError(
                    "Haptic intensity must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load thumbstick deadzone
        if let Some(TomlValue::Boolean(thumbstick_deadzone)) = config.get("thumbstick_deadzone") {
            self.thumbstick_deadzone = *thumbstick_deadzone;
        }
        
        // Load thumbstick deadzone value
        if let Some(TomlValue::Integer(thumbstick_deadzone_value)) = config.get("thumbstick_deadzone_value") {
            self.thumbstick_deadzone_value = *thumbstick_deadzone_value as u32;
            // Validate thumbstick deadzone value
            if self.thumbstick_deadzone_value > 50 {
                return Err(ConfigError::ValidationError(
                    "Thumbstick deadzone value must be between 0 and 50".to_string()
                ));
            }
        }
        
        // Load trigger deadzone
        if let Some(TomlValue::Boolean(trigger_deadzone)) = config.get("trigger_deadzone") {
            self.trigger_deadzone = *trigger_deadzone;
        }
        
        // Load trigger deadzone value
        if let Some(TomlValue::Integer(trigger_deadzone_value)) = config.get("trigger_deadzone_value") {
            self.trigger_deadzone_value = *trigger_deadzone_value as u32;
            // Validate trigger deadzone value
            if self.trigger_deadzone_value > 50 {
                return Err(ConfigError::ValidationError(
                    "Trigger deadzone value must be between 0 and 50".to_string()
                ));
            }
        }
        
        // Load controller vibration
        if let Some(TomlValue::Boolean(controller_vibration)) = config.get("controller_vibration") {
            self.controller_vibration = *controller_vibration;
        }
        
        // Load vibration intensity
        if let Some(TomlValue::Integer(vibration_intensity)) = config.get("vibration_intensity") {
            self.vibration_intensity = *vibration_intensity as u32;
            // Validate vibration intensity
            if self.vibration_intensity > 100 {
                return Err(ConfigError::ValidationError(
                    "Vibration intensity must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load custom button mapping
        if let Some(TomlValue::Boolean(custom_button_mapping)) = config.get("custom_button_mapping") {
            self.custom_button_mapping = *custom_button_mapping;
        }
        
        // Load button mapping
        if let Some(TomlValue::Table(button_mapping_table)) = config.get("button_mapping") {
            self.button_mapping.clear();
            for (key, value) in button_mapping_table {
                if let TomlValue::String(value_str) = value {
                    self.button_mapping.insert(key.clone(), value_str.clone());
                }
            }
        }
        
        // Load gesture controls
        if let Some(TomlValue::Boolean(gesture_controls)) = config.get("gesture_controls") {
            self.gesture_controls = *gesture_controls;
        }
        
        // Load gesture mapping
        if let Some(TomlValue::Table(gesture_mapping_table)) = config.get("gesture_mapping") {
            self.gesture_mapping.clear();
            for (key, value) in gesture_mapping_table {
                if let TomlValue::String(value_str) = value {
                    self.gesture_mapping.insert(key.clone(), value_str.clone());
                }
            }
        }
        
        // Load voice commands
        if let Some(TomlValue::Boolean(voice_commands)) = config.get("voice_commands") {
            self.voice_commands = *voice_commands;
        }
        
        // Load voice command mapping
        if let Some(TomlValue::Table(voice_command_mapping_table)) = config.get("voice_command_mapping") {
            self.voice_command_mapping.clear();
            for (key, value) in voice_command_mapping_table {
                if let TomlValue::String(value_str) = value {
                    self.voice_command_mapping.insert(key.clone(), value_str.clone());
                }
            }
        }
        
        Ok(())
    }
    
    /// Save input configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save controller profile
        config.insert("controller_profile".to_string(), TomlValue::String(self.controller_profile.clone()));
        
        // Save haptic feedback
        config.insert("haptic_feedback".to_string(), TomlValue::Boolean(self.haptic_feedback));
        
        // Save haptic intensity
        config.insert("haptic_intensity".to_string(), TomlValue::Integer(self.haptic_intensity as i64));
        
        // Save thumbstick deadzone
        config.insert("thumbstick_deadzone".to_string(), TomlValue::Boolean(self.thumbstick_deadzone));
        
        // Save thumbstick deadzone value
        config.insert("thumbstick_deadzone_value".to_string(), TomlValue::Integer(self.thumbstick_deadzone_value as i64));
        
        // Save trigger deadzone
        config.insert("trigger_deadzone".to_string(), TomlValue::Boolean(self.trigger_deadzone));
        
        // Save trigger deadzone value
        config.insert("trigger_deadzone_value".to_string(), TomlValue::Integer(self.trigger_deadzone_value as i64));
        
        // Save controller vibration
        config.insert("controller_vibration".to_string(), TomlValue::Boolean(self.controller_vibration));
        
        // Save vibration intensity
        config.insert("vibration_intensity".to_string(), TomlValue::Integer(self.vibration_intensity as i64));
        
        // Save custom button mapping
        config.insert("custom_button_mapping".to_string(), TomlValue::Boolean(self.custom_button_mapping));
        
        // Save button mapping
        let mut button_mapping_table = HashMap::new();
        for (key, value) in &self.button_mapping {
            button_mapping_table.insert(key.clone(), TomlValue::String(value.clone()));
        }
        config.insert("button_mapping".to_string(), TomlValue::Table(button_mapping_table));
        
        // Save gesture controls
        config.insert("gesture_controls".to_string(), TomlValue::Boolean(self.gesture_controls));
        
        // Save gesture mapping
        let mut gesture_mapping_table = HashMap::new();
        for (key, value) in &self.gesture_mapping {
            gesture_mapping_table.insert(key.clone(), TomlValue::String(value.clone()));
        }
        config.insert("gesture_mapping".to_string(), TomlValue::Table(gesture_mapping_table));
        
        // Save voice commands
        config.insert("voice_commands".to_string(), TomlValue::Boolean(self.voice_commands));
        
        // Save voice command mapping
        let mut voice_command_mapping_table = HashMap::new();
        for (key, value) in &self.voice_command_mapping {
            voice_command_mapping_table.insert(key.clone(), TomlValue::String(value.clone()));
        }
        config.insert("voice_command_mapping".to_string(), TomlValue::Table(voice_command_mapping_table));
        
        Ok(config)
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            controller_profile: "default".to_string(),
            haptic_feedback: true,
            haptic_intensity: 80,
            thumbstick_deadzone: true,
            thumbstick_deadzone_value: 10,
            trigger_deadzone: true,
            trigger_deadzone_value: 5,
            controller_vibration: true,
            vibration_intensity: 80,
            custom_button_mapping: false,
            button_mapping: HashMap::new(),
            gesture_controls: false,
            gesture_mapping: HashMap::new(),
            voice_commands: false,
            voice_command_mapping: HashMap::new(),
        }
    }
}

/// Comfort configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfortConfig {
    /// Interpupillary distance (IPD) in mm
    pub ipd_mm: f32,
    
    /// Lens distance setting (1-5)
    pub lens_distance: u32,
    
    /// Whether to enable blue light filter
    pub blue_light_filter: bool,
    
    /// Blue light filter intensity (0-100)
    pub blue_light_intensity: u32,
    
    /// Whether to enable motion sickness reduction
    pub motion_sickness_reduction: bool,
    
    /// Motion sickness reduction mode (vignette, teleport, snap_turn)
    pub motion_sickness_mode: String,
    
    /// Vignette intensity (0-100)
    pub vignette_intensity: u32,
    
    /// Snap turn angle (15, 30, 45, 90)
    pub snap_turn_angle: u32,
    
    /// Whether to enable seated mode
    pub seated_mode: bool,
    
    /// Whether to enable room scale mode
    pub room_scale_mode: bool,
    
    /// Whether to enable passthrough mode
    pub passthrough_mode: bool,
    
    /// Passthrough mode type (mono, stereo, color)
    pub passthrough_type: String,
    
    /// Whether to enable eye tracking
    pub eye_tracking: bool,
    
    /// Whether to enable dynamic IPD adjustment
    pub dynamic_ipd: bool,
    
    /// Whether to enable comfort reminders
    pub comfort_reminders: bool,
    
    /// Comfort reminder interval in minutes
    pub reminder_interval_min: u32,
}

impl ComfortConfig {
    /// Load comfort configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load IPD
        if let Some(TomlValue::Float(ipd_mm)) = config.get("ipd_mm") {
            self.ipd_mm = *ipd_mm as f32;
            // Validate IPD
            if self.ipd_mm < 50.0 || self.ipd_mm > 80.0 {
                return Err(ConfigError::ValidationError(
                    "IPD must be between 50.0 and 80.0 mm".to_string()
                ));
            }
        }
        
        // Load lens distance
        if let Some(TomlValue::Integer(lens_distance)) = config.get("lens_distance") {
            self.lens_distance = *lens_distance as u32;
            // Validate lens distance
            if self.lens_distance < 1 || self.lens_distance > 5 {
                return Err(ConfigError::ValidationError(
                    "Lens distance must be between 1 and 5".to_string()
                ));
            }
        }
        
        // Load blue light filter
        if let Some(TomlValue::Boolean(blue_light_filter)) = config.get("blue_light_filter") {
            self.blue_light_filter = *blue_light_filter;
        }
        
        // Load blue light intensity
        if let Some(TomlValue::Integer(blue_light_intensity)) = config.get("blue_light_intensity") {
            self.blue_light_intensity = *blue_light_intensity as u32;
            // Validate blue light intensity
            if self.blue_light_intensity > 100 {
                return Err(ConfigError::ValidationError(
                    "Blue light intensity must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load motion sickness reduction
        if let Some(TomlValue::Boolean(motion_sickness_reduction)) = config.get("motion_sickness_reduction") {
            self.motion_sickness_reduction = *motion_sickness_reduction;
        }
        
        // Load motion sickness mode
        if let Some(TomlValue::String(motion_sickness_mode)) = config.get("motion_sickness_mode") {
            self.motion_sickness_mode = motion_sickness_mode.clone();
            // Validate motion sickness mode
            if self.motion_sickness_mode != "vignette" && self.motion_sickness_mode != "teleport" && self.motion_sickness_mode != "snap_turn" {
                return Err(ConfigError::ValidationError(
                    "Motion sickness mode must be 'vignette', 'teleport', or 'snap_turn'".to_string()
                ));
            }
        }
        
        // Load vignette intensity
        if let Some(TomlValue::Integer(vignette_intensity)) = config.get("vignette_intensity") {
            self.vignette_intensity = *vignette_intensity as u32;
            // Validate vignette intensity
            if self.vignette_intensity > 100 {
                return Err(ConfigError::ValidationError(
                    "Vignette intensity must be between 0 and 100".to_string()
                ));
            }
        }
        
        // Load snap turn angle
        if let Some(TomlValue::Integer(snap_turn_angle)) = config.get("snap_turn_angle") {
            self.snap_turn_angle = *snap_turn_angle as u32;
            // Validate snap turn angle
            if self.snap_turn_angle != 15 && self.snap_turn_angle != 30 && self.snap_turn_angle != 45 && self.snap_turn_angle != 90 {
                return Err(ConfigError::ValidationError(
                    "Snap turn angle must be 15, 30, 45, or 90".to_string()
                ));
            }
        }
        
        // Load seated mode
        if let Some(TomlValue::Boolean(seated_mode)) = config.get("seated_mode") {
            self.seated_mode = *seated_mode;
        }
        
        // Load room scale mode
        if let Some(TomlValue::Boolean(room_scale_mode)) = config.get("room_scale_mode") {
            self.room_scale_mode = *room_scale_mode;
        }
        
        // Load passthrough mode
        if let Some(TomlValue::Boolean(passthrough_mode)) = config.get("passthrough_mode") {
            self.passthrough_mode = *passthrough_mode;
        }
        
        // Load passthrough type
        if let Some(TomlValue::String(passthrough_type)) = config.get("passthrough_type") {
            self.passthrough_type = passthrough_type.clone();
            // Validate passthrough type
            if self.passthrough_type != "mono" && self.passthrough_type != "stereo" && self.passthrough_type != "color" {
                return Err(ConfigError::ValidationError(
                    "Passthrough type must be 'mono', 'stereo', or 'color'".to_string()
                ));
            }
        }
        
        // Load eye tracking
        if let Some(TomlValue::Boolean(eye_tracking)) = config.get("eye_tracking") {
            self.eye_tracking = *eye_tracking;
        }
        
        // Load dynamic IPD
        if let Some(TomlValue::Boolean(dynamic_ipd)) = config.get("dynamic_ipd") {
            self.dynamic_ipd = *dynamic_ipd;
        }
        
        // Load comfort reminders
        if let Some(TomlValue::Boolean(comfort_reminders)) = config.get("comfort_reminders") {
            self.comfort_reminders = *comfort_reminders;
        }
        
        // Load reminder interval
        if let Some(TomlValue::Integer(reminder_interval_min)) = config.get("reminder_interval_min") {
            self.reminder_interval_min = *reminder_interval_min as u32;
        }
        
        Ok(())
    }
    
    /// Save comfort configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save IPD
        config.insert("ipd_mm".to_string(), TomlValue::Float(self.ipd_mm as f64));
        
        // Save lens distance
        config.insert("lens_distance".to_string(), TomlValue::Integer(self.lens_distance as i64));
        
        // Save blue light filter
        config.insert("blue_light_filter".to_string(), TomlValue::Boolean(self.blue_light_filter));
        
        // Save blue light intensity
        config.insert("blue_light_intensity".to_string(), TomlValue::Integer(self.blue_light_intensity as i64));
        
        // Save motion sickness reduction
        config.insert("motion_sickness_reduction".to_string(), TomlValue::Boolean(self.motion_sickness_reduction));
        
        // Save motion sickness mode
        config.insert("motion_sickness_mode".to_string(), TomlValue::String(self.motion_sickness_mode.clone()));
        
        // Save vignette intensity
        config.insert("vignette_intensity".to_string(), TomlValue::Integer(self.vignette_intensity as i64));
        
        // Save snap turn angle
        config.insert("snap_turn_angle".to_string(), TomlValue::Integer(self.snap_turn_angle as i64));
        
        // Save seated mode
        config.insert("seated_mode".to_string(), TomlValue::Boolean(self.seated_mode));
        
        // Save room scale mode
        config.insert("room_scale_mode".to_string(), TomlValue::Boolean(self.room_scale_mode));
        
        // Save passthrough mode
        config.insert("passthrough_mode".to_string(), TomlValue::Boolean(self.passthrough_mode));
        
        // Save passthrough type
        config.insert("passthrough_type".to_string(), TomlValue::String(self.passthrough_type.clone()));
        
        // Save eye tracking
        config.insert("eye_tracking".to_string(), TomlValue::Boolean(self.eye_tracking));
        
        // Save dynamic IPD
        config.insert("dynamic_ipd".to_string(), TomlValue::Boolean(self.dynamic_ipd));
        
        // Save comfort reminders
        config.insert("comfort_reminders".to_string(), TomlValue::Boolean(self.comfort_reminders));
        
        // Save reminder interval
        config.insert("reminder_interval_min".to_string(), TomlValue::Integer(self.reminder_interval_min as i64));
        
        Ok(config)
    }
}

impl Default for ComfortConfig {
    fn default() -> Self {
        Self {
            ipd_mm: 64.0,
            lens_distance: 3,
            blue_light_filter: false,
            blue_light_intensity: 50,
            motion_sickness_reduction: true,
            motion_sickness_mode: "vignette".to_string(),
            vignette_intensity: 30,
            snap_turn_angle: 45,
            seated_mode: false,
            room_scale_mode: true,
            passthrough_mode: true,
            passthrough_type: "stereo".to_string(),
            eye_tracking: false,
            dynamic_ipd: false,
            comfort_reminders: true,
            reminder_interval_min: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_config_load_save() {
        let mut config = ProfileConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("current_user_id".to_string(), TomlValue::String("user1".to_string()));
        toml_values.insert("guest_mode".to_string(), TomlValue::Boolean(true));
        
        let mut profiles_table = HashMap::new();
        let mut user1_profile = HashMap::new();
        user1_profile.insert("name".to_string(), TomlValue::String("User One".to_string()));
        profiles_table.insert("user1".to_string(), TomlValue::Table(user1_profile));
        toml_values.insert("profiles".to_string(), TomlValue::Table(profiles_table));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.current_user_id, "user1");
        assert_eq!(config.guest_mode, true);
        assert!(config.profiles.contains_key("user1"));
        assert_eq!(config.profiles.get("user1").unwrap().name, "User One");
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("current_user_id"), Some(&TomlValue::String("user1".to_string())));
        assert_eq!(saved.get("guest_mode"), Some(&TomlValue::Boolean(true)));
        assert!(saved.get("profiles").unwrap().as_table().unwrap().contains_key("user1"));
    }
    
    #[test]
    fn test_notification_config_load_save() {
        let mut config = NotificationConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("sound_enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("priority_level".to_string(), TomlValue::String("high".to_string()));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, false);
        assert_eq!(config.sound_enabled, false);
        assert_eq!(config.priority_level, "high");
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("sound_enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("priority_level"), Some(&TomlValue::String("high".to_string())));
    }
    
    #[test]
    fn test_user_config_load_save() {
        let config = UserConfig::new();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        
        let mut profile = HashMap::new();
        profile.insert("current_user_id".to_string(), TomlValue::String("user1".to_string()));
        toml_values.insert("profile".to_string(), TomlValue::Table(profile));
        
        let mut notification = HashMap::new();
        notification.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("notification".to_string(), TomlValue::Table(notification));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify profile values were saved correctly
        if let Some(TomlValue::Table(profile)) = saved.get("profile") {
            assert_eq!(profile.get("current_user_id"), Some(&TomlValue::String("user1".to_string())));
        } else {
            panic!("Expected profile table");
        }
        
        // Verify notification values were saved correctly
        if let Some(TomlValue::Table(notification)) = saved.get("notification") {
            assert_eq!(notification.get("enabled"), Some(&TomlValue::Boolean(false)));
        } else {
            panic!("Expected notification table");
        }
    }
}
