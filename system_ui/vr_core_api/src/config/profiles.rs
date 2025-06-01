//! User profile management for the VR Core API.
//!
//! This module provides functionality for managing user profiles,
//! including creating, loading, saving, and deleting profiles.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

/// Error types for profile operations.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    /// Profile not found
    #[error("Profile not found: {0}")]
    NotFound(String),
    
    /// Profile already exists
    #[error("Profile already exists: {0}")]
    AlreadyExists(String),
    
    /// Invalid profile
    #[error("Invalid profile: {0}")]
    Invalid(String),
    
    /// Other profile error
    #[error("Profile error: {0}")]
    Other(String),
}

/// Result type for profile operations.
pub type ProfileResult<T> = Result<T, ProfileError>;

/// User profile metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// Profile name
    pub name: String,
    
    /// Profile description
    pub description: Option<String>,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
}

/// User profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Profile metadata
    pub metadata: ProfileMetadata,
    
    /// Profile configuration
    pub config: HashMap<String, TomlValue>,
}

impl UserProfile {
    /// Create a new UserProfile.
    pub fn new(name: &str, description: Option<&str>, config: HashMap<String, TomlValue>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let metadata = ProfileMetadata {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            modified_at: now,
        };
        
        Self {
            metadata,
            config,
        }
    }
    
    /// Update the profile configuration.
    pub fn update_config(&mut self, config: HashMap<String, TomlValue>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.metadata.modified_at = now;
        self.config = config;
    }
    
    /// Update the profile description.
    pub fn update_description(&mut self, description: Option<&str>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.metadata.modified_at = now;
        self.metadata.description = description.map(|s| s.to_string());
    }
}

/// Profile manager for the VR system.
#[derive(Debug)]
pub struct ProfileManager {
    /// Profiles directory
    profiles_dir: PathBuf,
    
    /// Loaded profiles
    profiles: HashMap<String, UserProfile>,
}

impl ProfileManager {
    /// Create a new ProfileManager with the specified profiles directory.
    pub fn new<P: AsRef<Path>>(profiles_dir: P) -> ProfileResult<Self> {
        let profiles_dir = profiles_dir.as_ref().to_path_buf();
        
        // Create profiles directory if it doesn't exist
        if !profiles_dir.exists() {
            fs::create_dir_all(&profiles_dir)?;
        }
        
        let mut manager = Self {
            profiles_dir,
            profiles: HashMap::new(),
        };
        
        // Load existing profiles
        manager.load_profiles()?;
        
        Ok(manager)
    }
    
    /// Load all profiles from the profiles directory.
    pub fn load_profiles(&mut self) -> ProfileResult<()> {
        self.profiles.clear();
        
        // Read profile files
        let entries = fs::read_dir(&self.profiles_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        match self.load_profile_from_file(&path) {
                            Ok(profile) => {
                                self.profiles.insert(name.to_string(), profile);
                            }
                            Err(err) => {
                                warn!("Failed to load profile {}: {}", name, err);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a profile from a file.
    fn load_profile_from_file<P: AsRef<Path>>(&self, path: P) -> ProfileResult<UserProfile> {
        let path = path.as_ref();
        
        // Read file
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse TOML
        let profile: UserProfile = toml::from_str(&contents)?;
        
        Ok(profile)
    }
    
    /// Save a profile to a file.
    fn save_profile_to_file(&self, profile: &UserProfile) -> ProfileResult<()> {
        let filename = format!("{}.toml", profile.metadata.name);
        let path = self.profiles_dir.join(filename);
        
        // Convert to TOML
        let toml_string = toml::to_string_pretty(profile)?;
        
        // Write to file
        let mut file = File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
        
        Ok(())
    }
    
    /// Get a list of all profile names.
    pub fn get_profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
    
    /// Get a list of all profiles.
    pub fn get_profiles(&self) -> Vec<UserProfile> {
        self.profiles.values().cloned().collect()
    }
    
    /// Get a profile by name.
    pub fn get_profile(&self, name: &str) -> ProfileResult<UserProfile> {
        match self.profiles.get(name) {
            Some(profile) => Ok(profile.clone()),
            None => Err(ProfileError::NotFound(name.to_string())),
        }
    }
    
    /// Create a new profile.
    pub fn create_profile(&mut self, name: &str, description: Option<&str>, config: HashMap<String, TomlValue>) -> ProfileResult<UserProfile> {
        // Check if profile already exists
        if self.profiles.contains_key(name) {
            return Err(ProfileError::AlreadyExists(name.to_string()));
        }
        
        // Create profile
        let profile = UserProfile::new(name, description, config);
        
        // Save profile to file
        self.save_profile_to_file(&profile)?;
        
        // Add to loaded profiles
        self.profiles.insert(name.to_string(), profile.clone());
        
        Ok(profile)
    }
    
    /// Update an existing profile.
    pub fn update_profile(&mut self, name: &str, config: HashMap<String, TomlValue>) -> ProfileResult<UserProfile> {
        // Check if profile exists
        let mut profile = match self.profiles.get(name) {
            Some(profile) => profile.clone(),
            None => return Err(ProfileError::NotFound(name.to_string())),
        };
        
        // Update profile
        profile.update_config(config);
        
        // Save profile to file
        self.save_profile_to_file(&profile)?;
        
        // Update loaded profile
        self.profiles.insert(name.to_string(), profile.clone());
        
        Ok(profile)
    }
    
    /// Update a profile description.
    pub fn update_profile_description(&mut self, name: &str, description: Option<&str>) -> ProfileResult<UserProfile> {
        // Check if profile exists
        let mut profile = match self.profiles.get(name) {
            Some(profile) => profile.clone(),
            None => return Err(ProfileError::NotFound(name.to_string())),
        };
        
        // Update profile
        profile.update_description(description);
        
        // Save profile to file
        self.save_profile_to_file(&profile)?;
        
        // Update loaded profile
        self.profiles.insert(name.to_string(), profile.clone());
        
        Ok(profile)
    }
    
    /// Delete a profile.
    pub fn delete_profile(&mut self, name: &str) -> ProfileResult<()> {
        // Check if profile exists
        if !self.profiles.contains_key(name) {
            return Err(ProfileError::NotFound(name.to_string()));
        }
        
        // Remove profile file
        let filename = format!("{}.toml", name);
        let path = self.profiles_dir.join(filename);
        fs::remove_file(path)?;
        
        // Remove from loaded profiles
        self.profiles.remove(name);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_profile_creation() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(temp_dir.path()).unwrap();
        
        let mut config = HashMap::new();
        config.insert("test".to_string(), TomlValue::String("value".to_string()));
        
        let profile = manager.create_profile("test_profile", Some("Test Profile"), config.clone()).unwrap();
        
        assert_eq!(profile.metadata.name, "test_profile");
        assert_eq!(profile.metadata.description, Some("Test Profile".to_string()));
        assert_eq!(profile.config, config);
    }
    
    #[test]
    fn test_profile_retrieval() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(temp_dir.path()).unwrap();
        
        let mut config = HashMap::new();
        config.insert("test".to_string(), TomlValue::String("value".to_string()));
        
        manager.create_profile("test_profile", Some("Test Profile"), config.clone()).unwrap();
        
        let profile = manager.get_profile("test_profile").unwrap();
        assert_eq!(profile.metadata.name, "test_profile");
        assert_eq!(profile.config, config);
        
        let names = manager.get_profile_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "test_profile");
    }
    
    #[test]
    fn test_profile_update() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(temp_dir.path()).unwrap();
        
        let mut config = HashMap::new();
        config.insert("test".to_string(), TomlValue::String("value".to_string()));
        
        manager.create_profile("test_profile", Some("Test Profile"), config.clone()).unwrap();
        
        let mut new_config = HashMap::new();
        new_config.insert("test".to_string(), TomlValue::String("new_value".to_string()));
        
        let updated_profile = manager.update_profile("test_profile", new_config.clone()).unwrap();
        assert_eq!(updated_profile.config, new_config);
        
        let retrieved_profile = manager.get_profile("test_profile").unwrap();
        assert_eq!(retrieved_profile.config, new_config);
    }
    
    #[test]
    fn test_profile_description_update() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(temp_dir.path()).unwrap();
        
        let mut config = HashMap::new();
        config.insert("test".to_string(), TomlValue::String("value".to_string()));
        
        manager.create_profile("test_profile", Some("Test Profile"), config.clone()).unwrap();
        
        let updated_profile = manager.update_profile_description("test_profile", Some("Updated Description")).unwrap();
        assert_eq!(updated_profile.metadata.description, Some("Updated Description".to_string()));
        
        let retrieved_profile = manager.get_profile("test_profile").unwrap();
        assert_eq!(retrieved_profile.metadata.description, Some("Updated Description".to_string()));
    }
    
    #[test]
    fn test_profile_deletion() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ProfileManager::new(temp_dir.path()).unwrap();
        
        let mut config = HashMap::new();
        config.insert("test".to_string(), TomlValue::String("value".to_string()));
        
        manager.create_profile("test_profile", Some("Test Profile"), config.clone()).unwrap();
        
        manager.delete_profile("test_profile").unwrap();
        
        let result = manager.get_profile("test_profile");
        assert!(result.is_err());
        
        let names = manager.get_profile_names();
        assert_eq!(names.len(), 0);
    }
}
