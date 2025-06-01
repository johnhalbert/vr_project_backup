//! Configuration management module for the VR Core API.
//!
//! This module provides functionality for managing configuration settings,
//! including loading, saving, validating, and versioning configuration data.
//! It supports TOML-based configuration files and user profiles.

mod schema;
mod validation;
mod versioning;
mod profiles;
mod defaults;

pub use schema::{ConfigSchema, SchemaField, SchemaType, SchemaValidationError};
pub use validation::{ConfigValidator, ValidationError, ValidationResult};
pub use versioning::{ConfigVersion, VersionInfo, VersionMigration, MigrationError};
pub use profiles::{UserProfile, ProfileManager, ProfileError};
pub use defaults::DefaultConfigs;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::{self, Value as TomlValue};

/// Error types for configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    /// Schema validation error
    #[error("Schema validation error: {0}")]
    SchemaValidation(#[from] SchemaValidationError),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// Version migration error
    #[error("Version migration error: {0}")]
    VersionMigration(#[from] MigrationError),
    
    /// Profile error
    #[error("Profile error: {0}")]
    Profile(#[from] ProfileError),
    
    /// Configuration not found
    #[error("Configuration not found: {0}")]
    NotFound(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    
    /// Configuration already exists
    #[error("Configuration already exists: {0}")]
    AlreadyExists(String),
    
    /// Other error
    #[error("Configuration error: {0}")]
    Other(String),
}

/// Result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration manager for handling VR system configurations.
#[derive(Debug)]
pub struct ConfigManager {
    /// Base directory for configuration files
    config_dir: PathBuf,
    
    /// Current configuration values
    config_values: RwLock<HashMap<String, TomlValue>>,
    
    /// Configuration schema
    schema: Arc<ConfigSchema>,
    
    /// Configuration validator
    validator: Arc<ConfigValidator>,
    
    /// Version information
    version_info: VersionInfo,
    
    /// Profile manager
    profile_manager: Arc<Mutex<ProfileManager>>,
    
    /// Default configurations
    defaults: DefaultConfigs,
}

impl ConfigManager {
    /// Create a new ConfigManager with the specified configuration directory.
    pub fn new<P: AsRef<Path>>(config_dir: P) -> ConfigResult<Self> {
        let config_dir = config_dir.as_ref().to_path_buf();
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        // Create schema
        let schema = Arc::new(ConfigSchema::default());
        
        // Create validator
        let validator = Arc::new(ConfigValidator::new(schema.clone()));
        
        // Create version info
        let version_info = VersionInfo::current();
        
        // Create profile manager
        let profile_manager = Arc::new(Mutex::new(ProfileManager::new(config_dir.join("profiles"))?));
        
        // Create default configs
        let defaults = DefaultConfigs::new();
        
        // Create config manager
        let manager = Self {
            config_dir,
            config_values: RwLock::new(HashMap::new()),
            schema,
            validator,
            version_info,
            profile_manager,
            defaults,
        };
        
        // Load default configuration
        manager.load_defaults()?;
        
        Ok(manager)
    }
    
    /// Load the default configuration.
    pub fn load_defaults(&self) -> ConfigResult<()> {
        let default_config = self.defaults.get_default_config();
        
        // Validate default configuration
        self.validator.validate(&default_config)?;
        
        // Set default configuration
        let mut config_values = self.config_values.write().unwrap();
        *config_values = default_config;
        
        Ok(())
    }
    
    /// Load configuration from a file.
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let path = path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Err(ConfigError::NotFound(format!("File not found: {}", path.display())));
        }
        
        // Read file
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse TOML
        let config: HashMap<String, TomlValue> = toml::from_str(&contents)?;
        
        // Check version and migrate if necessary
        let config = self.check_and_migrate_version(config)?;
        
        // Validate configuration
        self.validator.validate(&config)?;
        
        // Set configuration
        let mut config_values = self.config_values.write().unwrap();
        *config_values = config;
        
        Ok(())
    }
    
    /// Save configuration to a file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        let path = path.as_ref();
        
        // Get configuration
        let config_values = self.config_values.read().unwrap();
        
        // Convert to TOML
        let toml_string = toml::to_string_pretty(&*config_values)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Write to file
        let mut file = File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
        
        Ok(())
    }
    
    /// Get a configuration value.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ConfigResult<T> {
        let config_values = self.config_values.read().unwrap();
        
        // Split key by dots
        let parts: Vec<&str> = key.split('.').collect();
        
        // Navigate through the configuration
        let mut current = &TomlValue::Table(config_values.clone());
        for part in &parts {
            match current {
                TomlValue::Table(table) => {
                    match table.get(*part) {
                        Some(value) => current = value,
                        None => return Err(ConfigError::NotFound(format!("Key not found: {}", key))),
                    }
                }
                _ => return Err(ConfigError::Invalid(format!("Invalid key path: {}", key))),
            }
        }
        
        // Deserialize value
        match toml::Value::try_from(current.clone()) {
            Ok(value) => match T::deserialize(value) {
                Ok(value) => Ok(value),
                Err(err) => Err(ConfigError::Invalid(format!("Failed to deserialize value: {}", err))),
            },
            Err(err) => Err(ConfigError::Invalid(format!("Failed to convert value: {}", err))),
        }
    }
    
    /// Set a configuration value.
    pub fn set<T: Serialize>(&self, key: &str, value: T) -> ConfigResult<()> {
        // Serialize value to TOML
        let toml_value = toml::Value::try_from(value)
            .map_err(|err| ConfigError::Invalid(format!("Failed to serialize value: {}", err)))?;
        
        // Split key by dots
        let parts: Vec<&str> = key.split('.').collect();
        
        // Get mutable reference to config values
        let mut config_values = self.config_values.write().unwrap();
        
        // Navigate through the configuration and set the value
        if parts.len() == 1 {
            // Simple key
            config_values.insert(parts[0].to_string(), toml_value);
        } else {
            // Nested key
            let mut current = config_values.clone();
            let last_index = parts.len() - 1;
            
            // Navigate to the parent of the target key
            for i in 0..last_index {
                let part = parts[i];
                
                // Ensure the path exists
                if !current.contains_key(part) {
                    current.insert(part.to_string(), TomlValue::Table(HashMap::new()));
                }
                
                // Get the next level
                match current.get_mut(part) {
                    Some(TomlValue::Table(table)) => {
                        current = table.clone();
                    }
                    _ => {
                        // Replace with a table
                        current.insert(part.to_string(), TomlValue::Table(HashMap::new()));
                        if let Some(TomlValue::Table(table)) = current.get(part) {
                            current = table.clone();
                        } else {
                            return Err(ConfigError::Invalid(format!("Failed to create path: {}", key)));
                        }
                    }
                }
            }
            
            // Set the value at the target key
            current.insert(parts[last_index].to_string(), toml_value);
            
            // Rebuild the configuration
            let mut result = config_values.clone();
            for i in (0..last_index).rev() {
                let part = parts[i];
                let mut parent = HashMap::new();
                
                if i > 0 {
                    if let Some(TomlValue::Table(table)) = result.get(parts[i-1]) {
                        parent = table.clone();
                    }
                } else {
                    parent = result.clone();
                }
                
                parent.insert(part.to_string(), TomlValue::Table(current.clone()));
                current = parent;
            }
            
            *config_values = current;
        }
        
        // Validate the updated configuration
        self.validator.validate(&config_values)?;
        
        Ok(())
    }
    
    /// Reset configuration to defaults.
    pub fn reset_to_defaults(&self) -> ConfigResult<()> {
        self.load_defaults()
    }
    
    /// Get the configuration schema.
    pub fn get_schema(&self) -> Arc<ConfigSchema> {
        self.schema.clone()
    }
    
    /// Get the configuration validator.
    pub fn get_validator(&self) -> Arc<ConfigValidator> {
        self.validator.clone()
    }
    
    /// Get the version information.
    pub fn get_version_info(&self) -> VersionInfo {
        self.version_info.clone()
    }
    
    /// Get the profile manager.
    pub fn get_profile_manager(&self) -> Arc<Mutex<ProfileManager>> {
        self.profile_manager.clone()
    }
    
    /// Load a user profile.
    pub fn load_profile(&self, profile_name: &str) -> ConfigResult<()> {
        let profile_manager = self.profile_manager.lock().unwrap();
        let profile = profile_manager.get_profile(profile_name)?;
        
        // Validate profile configuration
        self.validator.validate(&profile.config)?;
        
        // Set configuration
        let mut config_values = self.config_values.write().unwrap();
        *config_values = profile.config;
        
        Ok(())
    }
    
    /// Save current configuration as a user profile.
    pub fn save_as_profile(&self, profile_name: &str, description: Option<&str>) -> ConfigResult<()> {
        let config_values = self.config_values.read().unwrap();
        let mut profile_manager = self.profile_manager.lock().unwrap();
        
        profile_manager.create_profile(profile_name, description, config_values.clone())?;
        
        Ok(())
    }
    
    /// Check version and migrate configuration if necessary.
    fn check_and_migrate_version(&self, config: HashMap<String, TomlValue>) -> ConfigResult<HashMap<String, TomlValue>> {
        // Check if version is present
        let version = match config.get("version") {
            Some(TomlValue::String(version)) => version.clone(),
            Some(_) => return Err(ConfigError::Invalid("Invalid version format".to_string())),
            None => return Ok(config), // No version, assume current
        };
        
        // Parse version
        let config_version = ConfigVersion::parse(&version)
            .map_err(|err| ConfigError::Invalid(format!("Invalid version: {}", err)))?;
        
        // Check if migration is needed
        if config_version == self.version_info.version {
            return Ok(config);
        }
        
        // Migrate configuration
        let migrated_config = self.version_info.migrate(config, config_version)?;
        
        Ok(migrated_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        assert!(config_manager.config_dir.exists());
    }
    
    #[test]
    fn test_load_defaults() {
        let temp_dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Reset to defaults
        config_manager.reset_to_defaults().unwrap();
        
        // Check if defaults are loaded
        let display_enabled: bool = config_manager.get("display.enabled").unwrap();
        assert!(display_enabled);
    }
    
    #[test]
    fn test_set_and_get() {
        let temp_dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Set a value
        config_manager.set("test.value", 42).unwrap();
        
        // Get the value
        let value: i64 = config_manager.get("test.value").unwrap();
        assert_eq!(value, 42);
    }
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Set some values
        config_manager.set("test.string", "hello").unwrap();
        config_manager.set("test.number", 42).unwrap();
        config_manager.set("test.boolean", true).unwrap();
        
        // Save to file
        let config_file = temp_dir.path().join("test_config.toml");
        config_manager.save_to_file(&config_file).unwrap();
        
        // Create a new config manager
        let new_config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Load from file
        new_config_manager.load_from_file(&config_file).unwrap();
        
        // Check values
        let string: String = new_config_manager.get("test.string").unwrap();
        let number: i64 = new_config_manager.get("test.number").unwrap();
        let boolean: bool = new_config_manager.get("test.boolean").unwrap();
        
        assert_eq!(string, "hello");
        assert_eq!(number, 42);
        assert_eq!(boolean, true);
    }
    
    #[test]
    fn test_profile_management() {
        let temp_dir = tempdir().unwrap();
        let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
        
        // Set some values
        config_manager.set("display.brightness", 0.8).unwrap();
        config_manager.set("audio.volume", 0.5).unwrap();
        
        // Save as profile
        config_manager.save_as_profile("test_profile", Some("Test Profile")).unwrap();
        
        // Change values
        config_manager.set("display.brightness", 0.6).unwrap();
        config_manager.set("audio.volume", 0.3).unwrap();
        
        // Load profile
        config_manager.load_profile("test_profile").unwrap();
        
        // Check values
        let brightness: f64 = config_manager.get("display.brightness").unwrap();
        let volume: f64 = config_manager.get("audio.volume").unwrap();
        
        assert_eq!(brightness, 0.8);
        assert_eq!(volume, 0.5);
    }
}
