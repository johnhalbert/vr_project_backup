//! Backup and restore functionality for the VR headset.
//!
//! This module provides comprehensive backup and restore capabilities
//! for configuration data, user profiles, and system settings.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use zip::read::ZipArchive;

use super::profile::{ProfileError, ProfileResult};
use super::schema::SchemaRegistry;
use super::toml::{TomlConfig, TomlConfigError, TomlConfigResult};

/// Backup type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackupType {
    /// Full system backup
    Full,
    /// User profiles only
    Profiles,
    /// System settings only
    Settings,
    /// Custom backup
    Custom,
}

/// Configuration backup for the VR headset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    /// Backup ID
    pub id: String,
    
    /// Backup name
    pub name: String,
    
    /// Backup description
    pub description: String,
    
    /// Backup creation time
    pub created_at: u64,
    
    /// Backup version
    pub version: String,
    
    /// Backup type
    pub backup_type: BackupType,
    
    /// Included profiles
    pub profiles: Vec<String>,
    
    /// System information
    pub system_info: HashMap<String, Value>,
}

/// Backup metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup ID
    pub id: String,
    
    /// Backup name
    pub name: String,
    
    /// Backup description
    pub description: String,
    
    /// Backup creation time
    pub created_at: u64,
    
    /// Backup version
    pub version: String,
    
    /// Backup type
    pub backup_type: BackupType,
    
    /// Included profiles
    pub profiles: Vec<String>,
    
    /// System information
    pub system_info: HashMap<String, Value>,
}

/// Backup error.
#[derive(Debug)]
pub enum BackupError {
    /// IO error
    Io(io::Error),
    
    /// Zip error
    Zip(zip::result::ZipError),
    
    /// JSON error
    Json(serde_json::Error),
    
    /// TOML error
    Toml(toml::ser::Error),
    
    /// Profile error
    Profile(ProfileError),
    
    /// Backup error
    Backup(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::Io(err) => write!(f, "IO error: {}", err),
            BackupError::Zip(err) => write!(f, "Zip error: {}", err),
            BackupError::Json(err) => write!(f, "JSON error: {}", err),
            BackupError::Toml(err) => write!(f, "TOML error: {}", err),
            BackupError::Profile(err) => write!(f, "Profile error: {}", err),
            BackupError::Backup(msg) => write!(f, "Backup error: {}", msg),
        }
    }
}

impl std::error::Error for BackupError {}

impl From<io::Error> for BackupError {
    fn from(err: io::Error) -> Self {
        BackupError::Io(err)
    }
}

impl From<zip::result::ZipError> for BackupError {
    fn from(err: zip::result::ZipError) -> Self {
        BackupError::Zip(err)
    }
}

impl From<serde_json::Error> for BackupError {
    fn from(err: serde_json::Error) -> Self {
        BackupError::Json(err)
    }
}

impl From<toml::ser::Error> for BackupError {
    fn from(err: toml::ser::Error) -> Self {
        BackupError::Toml(err)
    }
}

impl From<ProfileError> for BackupError {
    fn from(err: ProfileError) -> Self {
        BackupError::Profile(err)
    }
}

/// Backup result.
pub type BackupResult<T> = Result<T, BackupError>;
