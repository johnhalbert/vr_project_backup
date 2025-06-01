//! TOML configuration parser for the VR headset.
//!
//! This module provides TOML parsing and serialization for configuration data,
//! with support for schema validation and error handling.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use toml::{self, de::Error as TomlError};

use super::schema::{Schema, SchemaError, SchemaRegistry, SchemaResult};

/// Parse a string as TOML and return a Value.
pub fn from_str<T>(s: &str) -> Result<T, TomlError>
where
    T: for<'de> Deserialize<'de>,
{
    toml::from_str(s)
}

/// Convert a value to a TOML string with pretty formatting.
pub fn to_string_pretty<T>(value: &T) -> Result<String, toml::ser::Error>
where
    T: Serialize,
{
    toml::to_string_pretty(value)
}

/// TOML configuration error.
#[derive(Debug)]
pub enum TomlConfigError {
    /// IO error
    Io(io::Error),
    
    /// TOML parsing error
    Parse(TomlError),
    
    /// Schema validation error
    Validation(Vec<SchemaError>),
    
    /// Configuration error
    Config(String),
}

impl std::fmt::Display for TomlConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TomlConfigError::Io(err) => write!(f, "IO error: {}", err),
            TomlConfigError::Parse(err) => write!(f, "TOML parsing error: {}", err),
            TomlConfigError::Validation(errs) => {
                writeln!(f, "Schema validation errors:")?;
                for (i, err) in errs.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, err)?;
                }
                Ok(())
            }
            TomlConfigError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for TomlConfigError {}

impl From<io::Error> for TomlConfigError {
    fn from(err: io::Error) -> Self {
        TomlConfigError::Io(err)
    }
}

impl From<TomlError> for TomlConfigError {
    fn from(err: TomlError) -> Self {
        TomlConfigError::Parse(err)
    }
}

impl From<Vec<SchemaError>> for TomlConfigError {
    fn from(errs: Vec<SchemaError>) -> Self {
        TomlConfigError::Validation(errs)
    }
}

/// TOML configuration result.
pub type TomlConfigResult<T> = Result<T, TomlConfigError>;

/// TOML configuration parser.
#[derive(Debug)]
pub struct TomlConfig {
    /// Configuration data
    data: Value,
    
    /// Schema registry
    schema_registry: Arc<RwLock<SchemaRegistry>>,
    
    /// Configuration file path
    file_path: Option<PathBuf>,
    
    /// Schema ID
    schema_id: Option<String>,
    
    /// Modified flag
    modified: bool,
}

impl TomlConfig {
    /// Create a new TOML configuration.
    pub fn new() -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
            schema_registry: Arc::new(RwLock::new(SchemaRegistry::new())),
            file_path: None,
            schema_id: None,
            modified: false,
        }
    }
    
    /// Create a new TOML configuration with a schema registry.
    pub fn with_schema_registry(schema_registry: Arc<RwLock<SchemaRegistry>>) -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
            schema_registry,
            file_path: None,
            schema_id: None,
            modified: false,
        }
    }
    
    /// Load configuration from a TOML file.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> TomlConfigResult<()> {
        let path = path.as_ref();
        
        // Read the file
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse the TOML
        let value: Value = from_str(&contents)?;
        
        // Update the configuration
        self.data = value;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;
        
        // Validate the configuration if a schema is set
        if let Some(schema_id) = &self.schema_id {
            self.validate_schema(schema_id)?;
        }
        
        Ok(())
    }
    
    /// Save configuration to a TOML file.
    pub fn save<P: AsRef<Path>>(&mut self, path: Option<P>) -> TomlConfigResult<()> {
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => match &self.file_path {
                Some(p) => p.clone(),
                None => return Err(TomlConfigError::Config("No file path specified".to_string())),
            },
        };
        
        // Validate the configuration if a schema is set
        if let Some(schema_id) = &self.schema_id {
            self.validate_schema(schema_id)?;
        }
        
        // Convert to TOML
        let toml_string = to_string_pretty(&self.data)
            .map_err(|e| TomlConfigError::Config(format!("Failed to serialize TOML: {}", e)))?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write to file
        let mut file = File::create(&path)?;
        file.write_all(toml_string.as_bytes())?;
        
        // Update the file path and modified flag
        self.file_path = Some(path);
        self.modified = false;
        
        Ok(())
    }
    
    /// Set the schema ID for validation.
    pub fn set_schema(&mut self, schema_id: &str) -> TomlConfigResult<()> {
        // Check if the schema exists
        let registry = self.schema_registry.read().unwrap();
        if !registry.contains(schema_id) {
            return Err(TomlConfigError::Config(format!("Schema '{}' not found", schema_id)));
        }
        
        self.schema_id = Some(schema_id.to_string());
        
        // Validate the configuration
        self.validate_schema(schema_id)?;
        
        Ok(())
    }
    
    /// Validate the configuration against a schema.
    pub fn validate_schema(&self, schema_id: &str) -> TomlConfigResult<()> {
        let registry = self.schema_registry.read().unwrap();
        
        let schema = match registry.get(schema_id) {
            Some(s) => s,
            None => return Err(TomlConfigError::Config(format!("Schema '{}' not found", schema_id))),
        };
        
        match schema.validate(&self.data) {
            Ok(_) => Ok(()),
            Err(errs) => Err(TomlConfigError::Validation(errs)),
        }
    }
    
    /// Get a configuration value.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> TomlConfigResult<T> {
        let value = self.get_value(path)?;
        
        serde_json::from_value(value.clone())
            .map_err(|e| TomlConfigError::Config(format!("Failed to deserialize value at '{}': {}", path, e)))
    }
    
    /// Get a configuration value as a JSON Value.
    pub fn get_value(&self, path: &str) -> TomlConfigResult<Value> {
        if path.is_empty() {
            return Ok(self.data.clone());
        }
        
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;
        
        for (i, part) in parts.iter().enumerate() {
            match current {
                Value::Object(obj) => {
                    match obj.get(*part) {
                        Some(value) => {
                            current = value;
                        }
                        None => {
                            let current_path = parts[..=i].join(".");
                            return Err(TomlConfigError::Config(format!("Path '{}' not found", current_path)));
                        }
                    }
                }
                Value::Array(arr) => {
                    match part.parse::<usize>() {
                        Ok(index) => {
                            if index < arr.len() {
                                current = &arr[index];
                            } else {
                                let current_path = parts[..=i].join(".");
                                return Err(TomlConfigError::Config(format!("Index {} out of bounds at '{}'", index, current_path)));
                            }
                        }
                        Err(_) => {
                            let current_path = parts[..i].join(".");
                            return Err(TomlConfigError::Config(format!("Expected array index at '{}', got '{}'", current_path, part)));
                        }
                    }
                }
                _ => {
                    let current_path = parts[..i].join(".");
                    return Err(TomlConfigError::Config(format!("Cannot access '{}' in non-object/non-array value at '{}'", part, current_path)));
                }
            }
        }
        
        Ok(current.clone())
    }
    
    /// Get a mutable reference to a configuration value.
    fn get_value_mut(&mut self, path: &str) -> Option<&mut Value> {
        if path.is_empty() {
            return Some(&mut self.data);
        }
        
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.data;
        
        for part in parts {
            match current {
                Value::Object(obj) => {
                    current = obj.get_mut(part)?;
                }
                Value::Array(arr) => {
                    let index = part.parse::<usize>().ok()?;
                    if index < arr.len() {
                        current = &mut arr[index];
                    } else {
                        return None;
                    }
                }
                _ => {
                    return None;
                }
            }
        }
        
        Some(current)
    }
    
    /// Set a configuration value.
    pub fn set<T: Serialize>(&mut self, path: &str, value: T) -> TomlConfigResult<()> {
        let value = serde_json::to_value(value)
            .map_err(|e| TomlConfigError::Config(format!("Failed to serialize value for '{}': {}", path, e)))?;
        
        self.set_value(path, value)
    }
    
    /// Set a configuration value from a JSON Value.
    pub fn set_value(&mut self, path: &str, value: Value) -> TomlConfigResult<()> {
        if path.is_empty() {
            self.data = value;
            self.modified = true;
            return Ok(());
        }
        
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.data;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                match current {
                    Value::Object(obj) => {
                        obj.insert(part.to_string(), value);
                        self.modified = true;
                        return Ok(());
                    }
                    Value::Array(arr) => {
                        match part.parse::<usize>() {
                            Ok(index) => {
                                if index < arr.len() {
                                    arr[index] = value;
                                    self.modified = true;
                                    return Ok(());
                                } else {
                                    let current_path = parts[..=i].join(".");
                                    return Err(TomlConfigError::Config(format!("Index {} out of bounds at '{}'", index, current_path)));
                                }
                            }
                            Err(_) => {
                                let current_path = parts[..i].join(".");
                                return Err(TomlConfigError::Config(format!("Expected array index at '{}', got '{}'", current_path, part)));
                            }
                        }
                    }
                    _ => {
                        let current_path = parts[..i].join(".");
                        return Err(TomlConfigError::Config(format!("Cannot set '{}' in non-object/non-array value at '{}'", part, current_path)));
                    }
                }
            } else {
                // Not the last part, navigate to the next level
                match current {
                    Value::Object(obj) => {
                        if !obj.contains_key(*part) {
                            // Create intermediate objects
                            obj.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                        }
                        
                        match obj.get_mut(*part) {
                            Some(next) => {
                                current = next;
                            }
                            None => {
                                let current_path = parts[..=i].join(".");
                                return Err(TomlConfigError::Config(format!("Failed to navigate to '{}'", current_path)));
                            }
                        }
                    }
                    Value::Array(arr) => {
                        match part.parse::<usize>() {
                            Ok(index) => {
                                if index < arr.len() {
                                    current = &mut arr[index];
                                } else {
                                    let current_path = parts[..=i].join(".");
                                    return Err(TomlConfigError::Config(format!("Index {} out of bounds at '{}'", index, current_path)));
                                }
                            }
                            Err(_) => {
                                let current_path = parts[..i].join(".");
                                return Err(TomlConfigError::Config(format!("Expected array index at '{}', got '{}'", current_path, part)));
                            }
                        }
                    }
                    _ => {
                        let current_path = parts[..i].join(".");
                        return Err(TomlConfigError::Config(format!("Cannot navigate to '{}' in non-object/non-array value at '{}'", part, current_path)));
                    }
                }
            }
        }
        
        // This should never happen
        Err(TomlConfigError::Config("Failed to set value".to_string()))
    }
    
    /// Remove a configuration value.
    pub fn remove(&mut self, path: &str) -> TomlConfigResult<Value> {
        if path.is_empty() {
            return Err(TomlConfigError::Config("Cannot remove root value".to_string()));
        }
        
        let parts: Vec<&str> = path.split('.').collect();
        let parent_path = parts[..parts.len() - 1].join(".");
        let last_part = parts[parts.len() - 1];
        
        let parent = if parent_path.is_empty() {
            &mut self.data
        } else {
            let parent_value = self.get_value(&parent_path)?;
            self.set_value(&parent_path, parent_value.clone())?;
            
            match self.get_value_mut(&parent_path) {
                Some(value) => value,
                None => return Err(TomlConfigError::Config(format!("Failed to get mutable reference to '{}'", parent_path))),
            }
        };
        
        match parent {
            Value::Object(obj) => {
                match obj.remove(last_part) {
                    Some(value) => {
                        self.modified = true;
                        Ok(value)
                    }
                    None => Err(TomlConfigError::Config(format!("Path '{}' not found", path))),
                }
            }
            Value::Array(arr) => {
                match last_part.parse::<usize>() {
                    Ok(index) => {
                        if index < arr.len() {
                            let value = arr.remove(index);
                            self.modified = true;
                            Ok(value)
                        } else {
                            Err(TomlConfigError::Config(format!("Index {} out of bounds at '{}'", index, path)))
                        }
                    }
                    Err(_) => {
                        Err(TomlConfigError::Config(format!("Expected array index at '{}', got '{}'", parent_path, last_part)))
                    }
                }
            }
            _ => {
                Err(TomlConfigError::Config(format!("Cannot remove '{}' from non-object/non-array value at '{}'", last_part, parent_path)))
            }
        }
    }
    
    /// Check if a configuration value exists.
    pub fn contains(&self, path: &str) -> bool {
        self.get_value(path).is_ok()
    }
    
    /// Get all keys at the root level.
    pub fn keys(&self) -> Vec<String> {
        match &self.data {
            Value::Object(obj) => obj.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }
    
    /// Get all keys at a specific path.
    pub fn keys_at(&self, path: &str) -> TomlConfigResult<Vec<String>> {
        let value = self.get_value(path)?;
        
        match value {
            Value::Object(obj) => Ok(obj.keys().cloned().collect()),
            _ => Err(TomlConfigError::Config(format!("Value at '{}' is not an object", path))),
        }
    }
    
    /// Get the configuration data.
    pub fn data(&self) -> &Value {
        &self.data
    }
    
    /// Get the configuration data as a mutable reference.
    pub fn data_mut(&mut self) -> &mut Value {
        &mut self.data
    }
    
    /// Get the file path.
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
    
    /// Check if the configuration has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Mark the configuration as modified.
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }
    
    /// Get the schema ID.
    pub fn schema_id(&self) -> Option<&str> {
        self.schema_id.as_deref()
    }
    
    /// Get the schema registry.
    pub fn schema_registry(&self) -> Arc<RwLock<SchemaRegistry>> {
        Arc::clone(&self.schema_registry)
    }
}
