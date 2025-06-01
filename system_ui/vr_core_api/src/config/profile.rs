use std::collections::HashMap;
use std::io;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

/// Profile type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileType {
    /// User profile
    User,
    /// System profile
    System,
    /// Guest profile
    Guest,
}

/// User profile for the VR headset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Profile ID
    pub id: String,
    
    /// Profile name
    pub name: String,
    
    /// Profile type
    pub profile_type: ProfileType,
    
    /// Profile creation time
    pub created_at: u64,
    
    /// Profile last modified time
    pub modified_at: u64,
    
    /// Profile metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Profile error.
#[derive(Debug, Error)]
pub enum ProfileError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Profile not found
    #[error("Profile not found: {0}")]
    NotFound(String),
    
    /// Invalid profile
    #[error("Invalid profile: {0}")]
    Invalid(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Profile result.
pub type ProfileResult<T> = Result<T, ProfileError>;
