//! Configuration validation for the VR Core API.
//!
//! This module provides validation functionality for configuration values,
//! including type checking, range validation, and custom validation rules.

use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use super::schema::{ConfigSchema, SchemaValidationError};

/// Error types for configuration validation.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Schema validation error
    #[error("Schema validation error: {0}")]
    Schema(#[from] SchemaValidationError),
    
    /// Semantic validation error
    #[error("Semantic validation error: {0}")]
    Semantic(String),
    
    /// Dependency validation error
    #[error("Dependency validation error: {0}")]
    Dependency(String),
    
    /// Other validation error
    #[error("Validation error: {0}")]
    Other(String),
}

/// Result type for validation operations.
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Configuration validator for the VR system.
#[derive(Debug)]
pub struct ConfigValidator {
    /// Configuration schema
    schema: Arc<ConfigSchema>,
    
    /// Custom validation rules
    validation_rules: Vec<Box<dyn Fn(&HashMap<String, TomlValue>) -> ValidationResult<()> + Send + Sync>>,
}

impl ConfigValidator {
    /// Create a new ConfigValidator with the specified schema.
    pub fn new(schema: Arc<ConfigSchema>) -> Self {
        let mut validator = Self {
            schema,
            validation_rules: Vec::new(),
        };
        
        // Add default validation rules
        validator.add_default_rules();
        
        validator
    }
    
    /// Add a custom validation rule.
    pub fn add_rule<F>(&mut self, rule: F)
    where
        F: Fn(&HashMap<String, TomlValue>) -> ValidationResult<()> + Send + Sync + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }
    
    /// Validate a configuration against the schema and custom rules.
    pub fn validate(&self, config: &HashMap<String, TomlValue>) -> ValidationResult<()> {
        // Validate against schema
        self.schema.validate(config).map_err(ValidationError::Schema)?;
        
        // Apply custom validation rules
        for rule in &self.validation_rules {
            rule(config)?;
        }
        
        Ok(())
    }
    
    /// Add default validation rules.
    fn add_default_rules(&mut self) {
        // Rule: If power.mode is "low_power", display.brightness should be <= 0.6
        self.add_rule(|config| {
            if let (Some(TomlValue::Table(power)), Some(TomlValue::Table(display))) = (config.get("power"), config.get("display")) {
                if let (Some(TomlValue::String(mode)), Some(TomlValue::Float(brightness))) = (power.get("mode"), display.get("brightness")) {
                    if mode == "low_power" && *brightness > 0.6 {
                        return Err(ValidationError::Semantic(
                            "In low_power mode, display brightness should not exceed 0.6".to_string(),
                        ));
                    }
                }
            }
            Ok(())
        });
        
        // Rule: If tracking.enabled is false, both tracking.camera_enabled and tracking.imu_enabled should be false
        self.add_rule(|config| {
            if let Some(TomlValue::Table(tracking)) = config.get("tracking") {
                if let Some(TomlValue::Boolean(enabled)) = tracking.get("enabled") {
                    if !enabled {
                        if let Some(TomlValue::Boolean(camera_enabled)) = tracking.get("camera_enabled") {
                            if *camera_enabled {
                                return Err(ValidationError::Dependency(
                                    "When tracking is disabled, camera tracking should also be disabled".to_string(),
                                ));
                            }
                        }
                        
                        if let Some(TomlValue::Boolean(imu_enabled)) = tracking.get("imu_enabled") {
                            if *imu_enabled {
                                return Err(ValidationError::Dependency(
                                    "When tracking is disabled, IMU tracking should also be disabled".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        
        // Rule: If audio.muted is true, audio.volume should be 0.0
        self.add_rule(|config| {
            if let Some(TomlValue::Table(audio)) = config.get("audio") {
                if let (Some(TomlValue::Boolean(muted)), Some(TomlValue::Float(volume))) = (audio.get("muted"), audio.get("volume")) {
                    if *muted && *volume > 0.0 {
                        return Err(ValidationError::Semantic(
                            "When audio is muted, volume should be 0.0".to_string(),
                        ));
                    }
                }
            }
            Ok(())
        });
        
        // Rule: If security.encryption_enabled is true, security.pin_required should also be true
        self.add_rule(|config| {
            if let Some(TomlValue::Table(security)) = config.get("security") {
                if let (Some(TomlValue::Boolean(encryption_enabled)), Some(TomlValue::Boolean(pin_required))) = 
                    (security.get("encryption_enabled"), security.get("pin_required")) {
                    if *encryption_enabled && !pin_required {
                        return Err(ValidationError::Dependency(
                            "When encryption is enabled, PIN should be required".to_string(),
                        ));
                    }
                }
            }
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::ConfigSchema;
    
    #[test]
    fn test_validator_creation() {
        let schema = Arc::new(ConfigSchema::default());
        let validator = ConfigValidator::new(schema);
        
        assert!(!validator.validation_rules.is_empty());
    }
    
    #[test]
    fn test_validation_success() {
        let schema = Arc::new(ConfigSchema::default());
        let validator = ConfigValidator::new(schema);
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(true));
        display.insert("brightness".to_string(), TomlValue::Float(0.7));
        config.insert("display".to_string(), TomlValue::Table(display));
        
        let mut power = HashMap::new();
        power.insert("mode".to_string(), TomlValue::String("normal".to_string()));
        config.insert("power".to_string(), TomlValue::Table(power));
        
        assert!(validator.validate(&config).is_ok());
    }
    
    #[test]
    fn test_validation_semantic_error() {
        let schema = Arc::new(ConfigSchema::default());
        let validator = ConfigValidator::new(schema);
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(true));
        display.insert("brightness".to_string(), TomlValue::Float(0.8)); // Too high for low_power mode
        config.insert("display".to_string(), TomlValue::Table(display));
        
        let mut power = HashMap::new();
        power.insert("mode".to_string(), TomlValue::String("low_power".to_string()));
        config.insert("power".to_string(), TomlValue::Table(power));
        
        let result = validator.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(ValidationError::Semantic(_)) => {
                // Expected error
            }
            _ => panic!("Expected Semantic validation error"),
        }
    }
    
    #[test]
    fn test_validation_dependency_error() {
        let schema = Arc::new(ConfigSchema::default());
        let validator = ConfigValidator::new(schema);
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut tracking = HashMap::new();
        tracking.insert("enabled".to_string(), TomlValue::Boolean(false));
        tracking.insert("camera_enabled".to_string(), TomlValue::Boolean(true)); // Should be false when tracking is disabled
        config.insert("tracking".to_string(), TomlValue::Table(tracking));
        
        let result = validator.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(ValidationError::Dependency(_)) => {
                // Expected error
            }
            _ => panic!("Expected Dependency validation error"),
        }
    }
    
    #[test]
    fn test_custom_validation_rule() {
        let schema = Arc::new(ConfigSchema::default());
        let mut validator = ConfigValidator::new(schema);
        
        // Add custom rule: network.wifi_enabled and network.bluetooth_enabled cannot both be false
        validator.add_rule(|config| {
            if let Some(TomlValue::Table(network)) = config.get("network") {
                if let (Some(TomlValue::Boolean(wifi)), Some(TomlValue::Boolean(bluetooth))) = 
                    (network.get("wifi_enabled"), network.get("bluetooth_enabled")) {
                    if !wifi && !bluetooth {
                        return Err(ValidationError::Semantic(
                            "At least one of WiFi or Bluetooth must be enabled".to_string(),
                        ));
                    }
                }
            }
            Ok(())
        });
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut network = HashMap::new();
        network.insert("wifi_enabled".to_string(), TomlValue::Boolean(false));
        network.insert("bluetooth_enabled".to_string(), TomlValue::Boolean(false));
        config.insert("network".to_string(), TomlValue::Table(network));
        
        let result = validator.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(ValidationError::Semantic(_)) => {
                // Expected error
            }
            _ => panic!("Expected Semantic validation error"),
        }
    }
}
