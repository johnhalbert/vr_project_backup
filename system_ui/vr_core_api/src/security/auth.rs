//! Authentication and authorization module for the VR headset.
//!
//! This module provides authentication and authorization functionality for the VR headset,
//! including user management, role-based access control, and session management.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use argon2::{self, Config as Argon2Config};
use log::error;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Authentication result.
pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Authentication error.
#[derive(Debug, Error)]
pub enum AuthError {
    
    /// User error
    #[error("User error: {0}")]
    User(String),
    
    /// Session error
    #[error("Session error: {0}")]
    Session(String),
    
    /// Permission error
    #[error("Permission error: {0}")]
    Permission(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Not authenticated error
    #[error("Not authenticated")]
    NotAuthenticated,
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Authentication context.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Session ID
    pub session_id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Username
    pub username: String,
    
    /// Roles
    pub roles: Vec<String>,
}

/// Authentication manager.
pub struct AuthManager {
    /// Users
    users: Arc<RwLock<HashMap<String, User>>>,
    
    /// Sessions
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    
    /// Roles
    roles: Arc<RwLock<HashMap<String, Role>>>,
    
    /// Authentication directory
    auth_dir: PathBuf,
}

impl AuthManager {
    /// Create a new authentication manager.
    pub fn new(auth_dir: PathBuf) -> Result<Self> {
        // Create the authentication directory if it doesn't exist
        if !auth_dir.exists() {
            fs::create_dir_all(&auth_dir)?;
        }
        
        // Create the users directory if it doesn't exist
        let users_dir = auth_dir.join("users");
        if !users_dir.exists() {
            fs::create_dir_all(&users_dir)?;
        }
        
        // Create the roles directory if it doesn't exist
        let roles_dir = auth_dir.join("roles");
        if !roles_dir.exists() {
            fs::create_dir_all(&roles_dir)?;
        }
        
        // Load users
        let mut users = HashMap::new();
        for entry in fs::read_dir(&users_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let user_str = fs::read_to_string(&path)?;
                let user: User = serde_json::from_str(&user_str)
                    .map_err(|e| AuthError::Serialization(format!("Failed to parse user: {}", e)))?;
                
                users.insert(user.username.clone(), user);
            }
        }
        
        // Load roles
        let mut roles = HashMap::new();
        for entry in fs::read_dir(&roles_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let role_str = fs::read_to_string(&path)?;
                let role: Role = serde_json::from_str(&role_str)
                    .map_err(|e| AuthError::Serialization(format!("Failed to parse role: {}", e)))?;
                
                roles.insert(role.name.clone(), role);
            }
        }
        
        // Create default roles if they don't exist
        if !roles.contains_key("admin") {
            let admin_role = Role {
                name: "admin".to_string(),
                permissions: vec![
                    "admin".to_string(),
                    "user".to_string(),
                    "guest".to_string(),
                ],
            };
            
            roles.insert(admin_role.name.clone(), admin_role);
        }
        
        if !roles.contains_key("user") {
            let user_role = Role {
                name: "user".to_string(),
                permissions: vec![
                    "user".to_string(),
                    "guest".to_string(),
                ],
            };
            
            roles.insert(user_role.name.clone(), user_role);
        }
        
        if !roles.contains_key("guest") {
            let guest_role = Role {
                name: "guest".to_string(),
                permissions: vec![
                    "guest".to_string(),
                ],
            };
            
            roles.insert(guest_role.name.clone(), guest_role);
        }
        
        // Save default roles
        for role in roles.values() {
            let role_path = roles_dir.join(format!("{}.json", role.name));
            let role_str = serde_json::to_string_pretty(role)
                .map_err(|e| AuthError::Serialization(format!("Failed to serialize role: {}", e)))?;
            
            let mut file = File::create(&role_path)?;
            file.write_all(role_str.as_bytes())?;
        }
        
        // Create default admin user if no users exist
        if users.is_empty() {
            let admin_user = User {
                id: Uuid::new_v4().to_string(),
                username: "admin".to_string(),
                password_hash: Self::hash_password("admin")?,
                roles: vec!["admin".to_string()],
                created_at: SystemTime::now(),
                last_login: None,
            };
            
            users.insert(admin_user.username.clone(), admin_user);
            
            // Save the admin user
            let user_path = users_dir.join("admin.json");
            let user_str = serde_json::to_string_pretty(&users["admin"])
                .map_err(|e| AuthError::Serialization(format!("Failed to serialize user: {}", e)))?;
            
            let mut file = File::create(&user_path)?;
            file.write_all(user_str.as_bytes())?;
        }
        
        Ok(Self {
            users: Arc::new(RwLock::new(users)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(roles)),
            auth_dir,
        })
    }
    
    /// Initialize the authentication manager.
    pub fn initialize(&self) -> Result<()> {
        // Nothing to initialize
        Ok(())
    }
    
    /// Shutdown the authentication manager.
    pub fn shutdown(&self) -> Result<()> {
        // Nothing to shutdown
        Ok(())
    }
    
    /// Create a user.
    pub fn create_user(&self, username: &str, password: &str, roles: &[&str]) -> Result<User> {
        // Check if the user already exists
        {
            let users = self.users.read().unwrap();
            
            if users.contains_key(username) {
                return Err(AuthError::User(format!("User already exists: {}", username)).into());
            }
        }
        
        // Check if the roles exist
        {
            let roles_map = self.roles.read().unwrap();
            
            for role in roles {
                if !roles_map.contains_key(*role) {
                    return Err(AuthError::User(format!("Role does not exist: {}", role)).into());
                }
            }
        }
        
        // Create the user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: Self::hash_password(password)?,
            roles: roles.iter().map(|r| r.to_string()).collect(),
            created_at: SystemTime::now(),
            last_login: None,
        };
        
        // Save the user
        let user_path = self.auth_dir.join("users").join(format!("{}.json", username));
        let user_str = serde_json::to_string_pretty(&user)
            .map_err(|e| AuthError::Serialization(format!("Failed to serialize user: {}", e)))?;
        
        let mut file = File::create(&user_path)?;
        file.write_all(user_str.as_bytes())?;
        
        // Add the user to the map
        let mut users = self.users.write().unwrap();
        users.insert(username.to_string(), user.clone());
        
        Ok(user)
    }
    
    /// Authenticate a user.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Session> {
        // Get the user
        let user = {
            let users = self.users.read().unwrap();
            
            users.get(username)
                .cloned()
                .ok_or_else(|| AuthError::User(format!("User not found: {}", username)))?
        };
        
        // Verify the password
        if !Self::verify_password(password, &user.password_hash)? {
            return Err(AuthError::User("Invalid password".to_string()).into());
        }
        
        // Create a session
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
        };
        
        // Add the session to the map
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        
        // Update the user's last login
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get_mut(username) {
            user.last_login = Some(SystemTime::now());
            
            // Save the user
            let user_path = self.auth_dir.join("users").join(format!("{}.json", username));
            let user_str = serde_json::to_string_pretty(user)
                .map_err(|e| AuthError::Serialization(format!("Failed to serialize user: {}", e)))?;
            
            let mut file = File::create(&user_path)?;
            file.write_all(user_str.as_bytes())?;
        }
        
        Ok(session)
    }
    
    /// Validate a session.
    pub fn validate_session(&self, session_id: &str) -> Result<Session> {
        // Get the session
        let session = {
            let sessions = self.sessions.read().unwrap();
            
            sessions.get(session_id)
                .cloned()
                .ok_or_else(|| AuthError::Session(format!("Session not found: {}", session_id)))?
        };
        
        // Check if the session is expired
        if SystemTime::now() > session.expires_at {
            // Remove the session
            let mut sessions = self.sessions.write().unwrap();
            sessions.remove(session_id);
            
            return Err(AuthError::Session("Session expired".to_string()).into());
        }
        
        Ok(session)
    }
    
    /// Get authentication context from session.
    pub fn get_auth_context(&self, session_id: &str) -> Result<AuthContext> {
        // Validate the session
        let session = self.validate_session(session_id)?;
        
        // Get the user
        let user = {
            let users = self.users.read().unwrap();
            
            let mut user = None;
            for u in users.values() {
                if u.id == session.user_id {
                    user = Some(u.clone());
                    break;
                }
            }
            
            user.ok_or_else(|| AuthError::User(format!("User not found: {}", session.user_id)))?
        };
        
        // Create the auth context
        let auth_context = AuthContext {
            session_id: session.id,
            user_id: user.id,
            username: user.username,
            roles: user.roles,
        };
        
        Ok(auth_context)
    }
    
    /// Check if a user has a permission.
    pub fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        // Get the user
        let user = {
            let users = self.users.read().unwrap();
            
            let mut user = None;
            for u in users.values() {
                if u.id == user_id {
                    user = Some(u.clone());
                    break;
                }
            }
            
            user.ok_or_else(|| AuthError::User(format!("User not found: {}", user_id)))?
        };
        
        // Get the roles
        let roles = self.roles.read().unwrap();
        
        // Check if the user has the permission
        for role_name in &user.roles {
            if let Some(role) = roles.get(role_name) {
                if role.permissions.contains(&permission.to_string()) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Hash a password.
    fn hash_password(password: &str) -> Result<String> {
        let salt = Self::generate_salt();
        let config = Argon2Config::default();
        
        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .map_err(|e| AuthError::Internal(format!("Failed to hash password: {}", e)))?;
        
        Ok(hash)
    }
    
    /// Verify a password.
    fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let result = argon2::verify_encoded(hash, password.as_bytes())
            .map_err(|e| AuthError::Internal(format!("Failed to verify password: {}", e)))?;
        
        Ok(result)
    }
    
    /// Generate a salt.
    fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }
}

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: String,
    
    /// Username
    pub username: String,
    
    /// Password hash
    pub password_hash: String,
    
    /// Roles
    pub roles: Vec<String>,
    
    /// Created at
    pub created_at: SystemTime,
    
    /// Last login
    pub last_login: Option<SystemTime>,
}

/// Session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Created at
    pub created_at: SystemTime,
    
    /// Expires at
    pub expires_at: SystemTime,
}

/// Role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    
    /// Permissions
    pub permissions: Vec<String>,
}
