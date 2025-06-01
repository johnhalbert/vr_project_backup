//! Configuration versioning for the VR Core API.
//!
//! This module provides functionality for managing configuration versions,
//! including version parsing, comparison, and migration between versions.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

/// Error types for version operations.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    /// Invalid version format
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),
    
    /// Version not supported
    #[error("Version not supported: {0}")]
    NotSupported(String),
    
    /// Other version error
    #[error("Version error: {0}")]
    Other(String),
}

/// Error types for migration operations.
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    /// Migration path not found
    #[error("Migration path not found from {0} to {1}")]
    PathNotFound(String, String),
    
    /// Migration failed
    #[error("Migration failed: {0}")]
    Failed(String),
    
    /// Version error
    #[error("Version error: {0}")]
    Version(#[from] VersionError),
    
    /// Other migration error
    #[error("Migration error: {0}")]
    Other(String),
}

/// Configuration version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConfigVersion {
    /// Major version
    pub major: u32,
    
    /// Minor version
    pub minor: u32,
    
    /// Patch version
    pub patch: u32,
}

impl ConfigVersion {
    /// Create a new ConfigVersion.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
    
    /// Parse a version string.
    pub fn parse(version: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = version.split('.').collect();
        
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(format!(
                "Version must have 3 parts: {}", version
            )));
        }
        
        let major = parts[0].parse::<u32>().map_err(|_| {
            VersionError::InvalidFormat(format!("Invalid major version: {}", parts[0]))
        })?;
        
        let minor = parts[1].parse::<u32>().map_err(|_| {
            VersionError::InvalidFormat(format!("Invalid minor version: {}", parts[1]))
        })?;
        
        let patch = parts[2].parse::<u32>().map_err(|_| {
            VersionError::InvalidFormat(format!("Invalid patch version: {}", parts[2]))
        })?;
        
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
    
    /// Get the version as a string.
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
    
    /// Check if this version is compatible with another version.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl fmt::Display for ConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for ConfigVersion {
    type Err = VersionError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Version migration function type.
pub type MigrationFn = fn(HashMap<String, TomlValue>) -> Result<HashMap<String, TomlValue>, MigrationError>;

/// Version migration definition.
#[derive(Clone)]
pub struct VersionMigration {
    /// Source version
    pub from: ConfigVersion,
    
    /// Target version
    pub to: ConfigVersion,
    
    /// Migration function
    #[serde(skip)]
    pub migrate: MigrationFn,
}

/// Version information for the VR system.
#[derive(Clone)]
pub struct VersionInfo {
    /// Current version
    pub version: ConfigVersion,
    
    /// Available migrations
    pub migrations: Vec<VersionMigration>,
}

impl VersionInfo {
    /// Get the current version information.
    pub fn current() -> Self {
        let version = ConfigVersion::new(1, 0, 0);
        
        let migrations = vec![
            // Add migrations here as needed
        ];
        
        Self {
            version,
            migrations,
        }
    }
    
    /// Migrate a configuration from one version to another.
    pub fn migrate(&self, config: HashMap<String, TomlValue>, from_version: ConfigVersion) -> Result<HashMap<String, TomlValue>, MigrationError> {
        // If already at current version, no migration needed
        if from_version == self.version {
            return Ok(config);
        }
        
        // Find migration path
        let path = self.find_migration_path(&from_version, &self.version)?;
        
        // Apply migrations in sequence
        let mut current_config = config;
        for migration in path {
            info!("Migrating configuration from {} to {}", migration.from, migration.to);
            current_config = (migration.migrate)(current_config)?;
        }
        
        // Update version in the migrated config
        current_config.insert("version".to_string(), TomlValue::String(self.version.to_string()));
        
        Ok(current_config)
    }
    
    /// Find a migration path between two versions.
    fn find_migration_path(&self, from: &ConfigVersion, to: &ConfigVersion) -> Result<Vec<VersionMigration>, MigrationError> {
        // Simple case: direct migration available
        if let Some(migration) = self.migrations.iter().find(|m| &m.from == from && &m.to == to) {
            return Ok(vec![migration.clone()]);
        }
        
        // Complex case: need to find a path
        let mut path = Vec::new();
        let mut current = from.clone();
        
        while current != *to {
            // Find next step in the path
            let next = self.migrations.iter().find(|m| &m.from == &current);
            
            match next {
                Some(migration) => {
                    path.push(migration.clone());
                    current = migration.to.clone();
                }
                None => {
                    return Err(MigrationError::PathNotFound(
                        from.to_string(),
                        to.to_string(),
                    ));
                }
            }
        }
        
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let version = ConfigVersion::parse("1.2.3").unwrap();
        
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }
    
    #[test]
    fn test_version_parsing_error() {
        let result = ConfigVersion::parse("1.2");
        assert!(result.is_err());
        
        let result = ConfigVersion::parse("1.2.3.4");
        assert!(result.is_err());
        
        let result = ConfigVersion::parse("1.x.3");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = ConfigVersion::new(1, 0, 0);
        let v2 = ConfigVersion::new(1, 1, 0);
        let v3 = ConfigVersion::new(2, 0, 0);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
        
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }
    
    #[test]
    fn test_version_to_string() {
        let version = ConfigVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }
    
    #[test]
    fn test_current_version() {
        let info = VersionInfo::current();
        assert_eq!(info.version.major, 1);
        assert_eq!(info.version.minor, 0);
        assert_eq!(info.version.patch, 0);
    }
}
