//! Configuration schema definition and validation for the VR Core API.
//!
//! This module provides the schema definition for configuration files,
//! including field types, constraints, and validation rules.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

/// Error types for schema validation.
#[derive(Debug, thiserror::Error)]
pub enum SchemaValidationError {
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Type mismatch
    #[error("Type mismatch for field {0}: expected {1}, got {2}")]
    TypeMismatch(String, String, String),
    
    /// Value out of range
    #[error("Value out of range for field {0}: {1}")]
    OutOfRange(String, String),
    
    /// Invalid value
    #[error("Invalid value for field {0}: {1}")]
    InvalidValue(String, String),
    
    /// Other validation error
    #[error("Validation error: {0}")]
    Other(String),
}

/// Schema field types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaType {
    /// String type
    String,
    /// Integer type
    Integer,
    /// Float type
    Float,
    /// Boolean type
    Boolean,
    /// Array type with element type
    Array(Box<SchemaType>),
    /// Table type with field schemas
    Table(HashMap<String, SchemaField>),
    /// Any type
    Any,
}

impl fmt::Display for SchemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaType::String => write!(f, "string"),
            SchemaType::Integer => write!(f, "integer"),
            SchemaType::Float => write!(f, "float"),
            SchemaType::Boolean => write!(f, "boolean"),
            SchemaType::Array(elem_type) => write!(f, "array of {}", elem_type),
            SchemaType::Table(_) => write!(f, "table"),
            SchemaType::Any => write!(f, "any"),
        }
    }
}

/// Schema field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field type
    pub field_type: SchemaType,
    
    /// Whether the field is required
    pub required: bool,
    
    /// Default value
    pub default: Option<TomlValue>,
    
    /// Description
    pub description: Option<String>,
    
    /// Minimum value (for numeric types)
    pub min: Option<TomlValue>,
    
    /// Maximum value (for numeric types)
    pub max: Option<TomlValue>,
    
    /// Allowed values (for enum-like fields)
    pub allowed_values: Option<Vec<TomlValue>>,
    
    /// Pattern (for string fields)
    pub pattern: Option<String>,
    
    /// Custom validation function
    #[serde(skip)]
    pub validator: Option<Arc<dyn Fn(&TomlValue) -> Result<(), String> + Send + Sync>>,
}

impl SchemaField {
    /// Create a new schema field.
    pub fn new(field_type: SchemaType) -> Self {
        Self {
            field_type,
            required: false,
            default: None,
            description: None,
            min: None,
            max: None,
            allowed_values: None,
            pattern: None,
            validator: None,
        }
    }
    
    /// Set the field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Set the default value.
    pub fn default(mut self, value: TomlValue) -> Self {
        self.default = Some(value);
        self
    }
    
    /// Set the description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set the minimum value.
    pub fn min(mut self, min: TomlValue) -> Self {
        self.min = Some(min);
        self
    }
    
    /// Set the maximum value.
    pub fn max(mut self, max: TomlValue) -> Self {
        self.max = Some(max);
        self
    }
    
    /// Set the allowed values.
    pub fn allowed_values(mut self, values: Vec<TomlValue>) -> Self {
        self.allowed_values = Some(values);
        self
    }
    
    /// Set the pattern.
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
    
    /// Set the custom validator.
    pub fn validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&TomlValue) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }
    
    /// Validate a value against this schema field.
    pub fn validate(&self, key: &str, value: &TomlValue) -> Result<(), SchemaValidationError> {
        // Check type
        self.validate_type(key, value)?;
        
        // Check constraints
        self.validate_constraints(key, value)?;
        
        // Check custom validator
        if let Some(validator) = &self.validator {
            if let Err(err) = validator(value) {
                return Err(SchemaValidationError::InvalidValue(key.to_string(), err));
            }
        }
        
        Ok(())
    }
    
    /// Validate the type of a value.
    fn validate_type(&self, key: &str, value: &TomlValue) -> Result<(), SchemaValidationError> {
        match &self.field_type {
            SchemaType::String => {
                if !value.is_str() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        "string".to_string(),
                        format!("{:?}", value),
                    ));
                }
            }
            SchemaType::Integer => {
                if !value.is_integer() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        "integer".to_string(),
                        format!("{:?}", value),
                    ));
                }
            }
            SchemaType::Float => {
                if !value.is_float() && !value.is_integer() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        "float".to_string(),
                        format!("{:?}", value),
                    ));
                }
            }
            SchemaType::Boolean => {
                if !value.is_bool() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        "boolean".to_string(),
                        format!("{:?}", value),
                    ));
                }
            }
            SchemaType::Array(elem_type) => {
                if !value.is_array() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        format!("array of {}", elem_type),
                        format!("{:?}", value),
                    ));
                }
                
                // Check array elements
                if let TomlValue::Array(array) = value {
                    for (i, elem) in array.iter().enumerate() {
                        let elem_key = format!("{}[{}]", key, i);
                        let elem_field = SchemaField::new((**elem_type).clone());
                        elem_field.validate_type(&elem_key, elem)?;
                    }
                }
            }
            SchemaType::Table(fields) => {
                if !value.is_table() {
                    return Err(SchemaValidationError::TypeMismatch(
                        key.to_string(),
                        "table".to_string(),
                        format!("{:?}", value),
                    ));
                }
                
                // Check table fields
                if let TomlValue::Table(table) = value {
                    for (field_key, field) in fields {
                        let full_key = if key.is_empty() {
                            field_key.clone()
                        } else {
                            format!("{}.{}", key, field_key)
                        };
                        
                        match table.get(field_key) {
                            Some(field_value) => {
                                field.validate(&full_key, field_value)?;
                            }
                            None => {
                                if field.required {
                                    return Err(SchemaValidationError::MissingField(full_key));
                                }
                            }
                        }
                    }
                }
            }
            SchemaType::Any => {
                // Any type is always valid
            }
        }
        
        Ok(())
    }
    
    /// Validate constraints on a value.
    fn validate_constraints(&self, key: &str, value: &TomlValue) -> Result<(), SchemaValidationError> {
        // Check min/max for numeric types
        if value.is_integer() || value.is_float() {
            if let Some(min) = &self.min {
                if (min.is_integer() && value.as_integer().unwrap() < min.as_integer().unwrap())
                    || (min.is_float() && value.as_float().unwrap() < min.as_float().unwrap())
                {
                    return Err(SchemaValidationError::OutOfRange(
                        key.to_string(),
                        format!("value {} is less than minimum {}", value, min),
                    ));
                }
            }
            
            if let Some(max) = &self.max {
                if (max.is_integer() && value.as_integer().unwrap() > max.as_integer().unwrap())
                    || (max.is_float() && value.as_float().unwrap() > max.as_float().unwrap())
                {
                    return Err(SchemaValidationError::OutOfRange(
                        key.to_string(),
                        format!("value {} is greater than maximum {}", value, max),
                    ));
                }
            }
        }
        
        // Check allowed values
        if let Some(allowed) = &self.allowed_values {
            if !allowed.contains(value) {
                return Err(SchemaValidationError::InvalidValue(
                    key.to_string(),
                    format!("value {} is not in allowed values {:?}", value, allowed),
                ));
            }
        }
        
        // Check pattern for string types
        if value.is_str() {
            if let Some(pattern) = &self.pattern {
                let regex = regex::Regex::new(pattern).map_err(|err| {
                    SchemaValidationError::Other(format!("Invalid regex pattern: {}", err))
                })?;
                
                if !regex.is_match(value.as_str().unwrap()) {
                    return Err(SchemaValidationError::InvalidValue(
                        key.to_string(),
                        format!("value {} does not match pattern {}", value, pattern),
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Configuration schema for the VR system.
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    /// Root schema fields
    pub fields: HashMap<String, SchemaField>,
}

impl ConfigSchema {
    /// Create a new empty schema.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
    
    /// Add a field to the schema.
    pub fn add_field(&mut self, key: &str, field: SchemaField) {
        self.fields.insert(key.to_string(), field);
    }
    
    /// Validate a configuration against this schema.
    pub fn validate(&self, config: &HashMap<String, TomlValue>) -> Result<(), SchemaValidationError> {
        for (key, field) in &self.fields {
            match config.get(key) {
                Some(value) => {
                    field.validate(key, value)?;
                }
                None => {
                    if field.required {
                        return Err(SchemaValidationError::MissingField(key.clone()));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the default configuration based on this schema.
    pub fn get_defaults(&self) -> HashMap<String, TomlValue> {
        let mut defaults = HashMap::new();
        
        for (key, field) in &self.fields {
            if let Some(default) = &field.default {
                defaults.insert(key.clone(), default.clone());
            } else if let SchemaType::Table(subfields) = &field.field_type {
                // Recursively get defaults for tables
                let mut table_defaults = HashMap::new();
                for (subkey, subfield) in subfields {
                    if let Some(default) = &subfield.default {
                        table_defaults.insert(subkey.clone(), default.clone());
                    }
                }
                
                if !table_defaults.is_empty() {
                    defaults.insert(key.clone(), TomlValue::Table(table_defaults));
                }
            }
        }
        
        defaults
    }
}

impl Default for ConfigSchema {
    fn default() -> Self {
        let mut schema = Self::new();
        
        // Add version field
        schema.add_field(
            "version",
            SchemaField::new(SchemaType::String)
                .description("Configuration version")
                .default(TomlValue::String("1.0.0".to_string())),
        );
        
        // Add display section
        let mut display_fields = HashMap::new();
        display_fields.insert(
            "enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether the display is enabled")
                .default(TomlValue::Boolean(true)),
        );
        display_fields.insert(
            "brightness".to_string(),
            SchemaField::new(SchemaType::Float)
                .description("Display brightness (0.0 - 1.0)")
                .min(TomlValue::Float(0.0))
                .max(TomlValue::Float(1.0))
                .default(TomlValue::Float(0.8)),
        );
        display_fields.insert(
            "refresh_rate".to_string(),
            SchemaField::new(SchemaType::Integer)
                .description("Display refresh rate in Hz")
                .min(TomlValue::Integer(60))
                .max(TomlValue::Integer(144))
                .default(TomlValue::Integer(90)),
        );
        display_fields.insert(
            "resolution".to_string(),
            SchemaField::new(SchemaType::Table({
                let mut res_fields = HashMap::new();
                res_fields.insert(
                    "width".to_string(),
                    SchemaField::new(SchemaType::Integer)
                        .description("Display width in pixels")
                        .min(TomlValue::Integer(800))
                        .default(TomlValue::Integer(1920)),
                );
                res_fields.insert(
                    "height".to_string(),
                    SchemaField::new(SchemaType::Integer)
                        .description("Display height in pixels")
                        .min(TomlValue::Integer(600))
                        .default(TomlValue::Integer(1080)),
                );
                res_fields
            }))
            .description("Display resolution"),
        );
        
        schema.add_field(
            "display",
            SchemaField::new(SchemaType::Table(display_fields))
                .description("Display settings"),
        );
        
        // Add audio section
        let mut audio_fields = HashMap::new();
        audio_fields.insert(
            "enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether audio is enabled")
                .default(TomlValue::Boolean(true)),
        );
        audio_fields.insert(
            "volume".to_string(),
            SchemaField::new(SchemaType::Float)
                .description("Audio volume (0.0 - 1.0)")
                .min(TomlValue::Float(0.0))
                .max(TomlValue::Float(1.0))
                .default(TomlValue::Float(0.5)),
        );
        audio_fields.insert(
            "muted".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether audio is muted")
                .default(TomlValue::Boolean(false)),
        );
        audio_fields.insert(
            "spatial_audio".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether spatial audio is enabled")
                .default(TomlValue::Boolean(true)),
        );
        
        schema.add_field(
            "audio",
            SchemaField::new(SchemaType::Table(audio_fields))
                .description("Audio settings"),
        );
        
        // Add tracking section
        let mut tracking_fields = HashMap::new();
        tracking_fields.insert(
            "enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether tracking is enabled")
                .default(TomlValue::Boolean(true)),
        );
        tracking_fields.insert(
            "mode".to_string(),
            SchemaField::new(SchemaType::String)
                .description("Tracking mode")
                .allowed_values(vec![
                    TomlValue::String("6dof".to_string()),
                    TomlValue::String("3dof".to_string()),
                ])
                .default(TomlValue::String("6dof".to_string())),
        );
        tracking_fields.insert(
            "camera_enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether camera tracking is enabled")
                .default(TomlValue::Boolean(true)),
        );
        tracking_fields.insert(
            "imu_enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether IMU tracking is enabled")
                .default(TomlValue::Boolean(true)),
        );
        
        schema.add_field(
            "tracking",
            SchemaField::new(SchemaType::Table(tracking_fields))
                .description("Tracking settings"),
        );
        
        // Add power section
        let mut power_fields = HashMap::new();
        power_fields.insert(
            "mode".to_string(),
            SchemaField::new(SchemaType::String)
                .description("Power mode")
                .allowed_values(vec![
                    TomlValue::String("normal".to_string()),
                    TomlValue::String("low_power".to_string()),
                    TomlValue::String("performance".to_string()),
                ])
                .default(TomlValue::String("normal".to_string())),
        );
        power_fields.insert(
            "auto_sleep".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether auto sleep is enabled")
                .default(TomlValue::Boolean(true)),
        );
        power_fields.insert(
            "sleep_timeout_sec".to_string(),
            SchemaField::new(SchemaType::Integer)
                .description("Sleep timeout in seconds")
                .min(TomlValue::Integer(30))
                .max(TomlValue::Integer(3600))
                .default(TomlValue::Integer(300)),
        );
        
        schema.add_field(
            "power",
            SchemaField::new(SchemaType::Table(power_fields))
                .description("Power settings"),
        );
        
        // Add network section
        let mut network_fields = HashMap::new();
        network_fields.insert(
            "wifi_enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether WiFi is enabled")
                .default(TomlValue::Boolean(true)),
        );
        network_fields.insert(
            "bluetooth_enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether Bluetooth is enabled")
                .default(TomlValue::Boolean(true)),
        );
        
        schema.add_field(
            "network",
            SchemaField::new(SchemaType::Table(network_fields))
                .description("Network settings"),
        );
        
        // Add security section
        let mut security_fields = HashMap::new();
        security_fields.insert(
            "encryption_enabled".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether encryption is enabled")
                .default(TomlValue::Boolean(true)),
        );
        security_fields.insert(
            "pin_required".to_string(),
            SchemaField::new(SchemaType::Boolean)
                .description("Whether PIN is required")
                .default(TomlValue::Boolean(false)),
        );
        
        schema.add_field(
            "security",
            SchemaField::new(SchemaType::Table(security_fields))
                .description("Security settings"),
        );
        
        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_validation_success() {
        let schema = ConfigSchema::default();
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(true));
        display.insert("brightness".to_string(), TomlValue::Float(0.7));
        display.insert("refresh_rate".to_string(), TomlValue::Integer(90));
        
        let mut resolution = HashMap::new();
        resolution.insert("width".to_string(), TomlValue::Integer(1920));
        resolution.insert("height".to_string(), TomlValue::Integer(1080));
        display.insert("resolution".to_string(), TomlValue::Table(resolution));
        
        config.insert("display".to_string(), TomlValue::Table(display));
        
        assert!(schema.validate(&config).is_ok());
    }
    
    #[test]
    fn test_schema_validation_type_mismatch() {
        let schema = ConfigSchema::default();
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::String("true".to_string())); // Should be boolean
        display.insert("brightness".to_string(), TomlValue::Float(0.7));
        
        config.insert("display".to_string(), TomlValue::Table(display));
        
        let result = schema.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(SchemaValidationError::TypeMismatch(field, expected, _)) => {
                assert_eq!(field, "display.enabled");
                assert_eq!(expected, "boolean");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }
    
    #[test]
    fn test_schema_validation_out_of_range() {
        let schema = ConfigSchema::default();
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut display = HashMap::new();
        display.insert("enabled".to_string(), TomlValue::Boolean(true));
        display.insert("brightness".to_string(), TomlValue::Float(1.5)); // Out of range
        
        config.insert("display".to_string(), TomlValue::Table(display));
        
        let result = schema.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(SchemaValidationError::OutOfRange(field, _)) => {
                assert_eq!(field, "display.brightness");
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }
    
    #[test]
    fn test_schema_validation_invalid_value() {
        let schema = ConfigSchema::default();
        
        let mut config = HashMap::new();
        config.insert("version".to_string(), TomlValue::String("1.0.0".to_string()));
        
        let mut tracking = HashMap::new();
        tracking.insert("enabled".to_string(), TomlValue::Boolean(true));
        tracking.insert("mode".to_string(), TomlValue::String("invalid".to_string())); // Invalid value
        
        config.insert("tracking".to_string(), TomlValue::Table(tracking));
        
        let result = schema.validate(&config);
        assert!(result.is_err());
        
        match result {
            Err(SchemaValidationError::InvalidValue(field, _)) => {
                assert_eq!(field, "tracking.mode");
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }
    
    #[test]
    fn test_get_defaults() {
        let schema = ConfigSchema::default();
        let defaults = schema.get_defaults();
        
        assert_eq!(defaults.get("version"), Some(&TomlValue::String("1.0.0".to_string())));
        
        if let Some(TomlValue::Table(display)) = defaults.get("display") {
            assert_eq!(display.get("enabled"), Some(&TomlValue::Boolean(true)));
            assert_eq!(display.get("brightness"), Some(&TomlValue::Float(0.8)));
            assert_eq!(display.get("refresh_rate"), Some(&TomlValue::Integer(90)));
        } else {
            panic!("Expected display table");
        }
    }
}
