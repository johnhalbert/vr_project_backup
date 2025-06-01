//! Authentication module for the VR Core API.
//!
//! This module provides authentication functionality for the VR Core API,
//! including authentication providers, tokens, and credentials.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{self, Config as Argon2Config, ThreadMode, Variant, Version};
use log::{debug, error, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::config::profiles::UserProfile;

/// Authentication error
#[derive(Debug, Error)]
pub enum AuthError {
    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    /// Invalid token
    #[error("Invalid token")]
    InvalidToken,
    
    /// Token expired
    #[error("Token expired")]
    TokenExpired,
    
    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    /// Internal error
    #[error("Internal authentication error: {0}")]
    InternalError(String),
}

/// Authentication result
pub type Result<T> = std::result::Result<T, AuthError>;

/// Authentication token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    /// Token ID
    pub id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Token value
    pub value: String,
    
    /// Expiration time (Unix timestamp in seconds)
    pub expires_at: u64,
    
    /// Token type
    pub token_type: TokenType,
    
    /// Token scope
    pub scope: Vec<String>,
}

impl AuthToken {
    /// Create a new authentication token
    pub fn new(user_id: &str, token_type: TokenType, scope: Vec<String>, expires_in: Duration) -> Self {
        let id = Uuid::new_v4().to_string();
        let value = Self::generate_token_value();
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() + expires_in.as_secs();
        
        Self {
            id,
            user_id: user_id.to_string(),
            value,
            expires_at,
            token_type,
            scope,
        }
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.expires_at < now
    }
    
    /// Check if token has scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scope.contains(&scope.to_string())
    }
    
    /// Generate token value
    fn generate_token_value() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64::encode(&random_bytes)
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// Access token
    Access,
    
    /// Refresh token
    Refresh,
    
    /// Device token
    Device,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Access => write!(f, "access"),
            TokenType::Refresh => write!(f, "refresh"),
            TokenType::Device => write!(f, "device"),
        }
    }
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Username
    pub username: String,
    
    /// Password
    pub password: Option<String>,
    
    /// Token
    pub token: Option<String>,
    
    /// Device ID
    pub device_id: Option<String>,
}

impl Credentials {
    /// Create new credentials with username and password
    pub fn new_with_password(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: Some(password.to_string()),
            token: None,
            device_id: None,
        }
    }
    
    /// Create new credentials with token
    pub fn new_with_token(username: &str, token: &str) -> Self {
        Self {
            username: username.to_string(),
            password: None,
            token: Some(token.to_string()),
            device_id: None,
        }
    }
    
    /// Create new credentials with device ID
    pub fn new_with_device_id(username: &str, device_id: &str) -> Self {
        Self {
            username: username.to_string(),
            password: None,
            token: None,
            device_id: Some(device_id.to_string()),
        }
    }
    
    /// Convert credentials to token
    pub fn to_token(&self) -> Result<String> {
        if let Some(token) = &self.token {
            Ok(token.clone())
        } else if let Some(password) = &self.password {
            // In a real implementation, this would involve a server request
            // For now, just create a simple token
            let token_data = format!("{}:{}", self.username, password);
            Ok(base64::encode(token_data))
        } else if let Some(device_id) = &self.device_id {
            // In a real implementation, this would involve a server request
            // For now, just create a simple token
            let token_data = format!("{}:{}", self.username, device_id);
            Ok(base64::encode(token_data))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

/// Authentication provider trait
pub trait AuthenticationProvider: Send + Sync {
    /// Authenticate user with credentials
    fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    
    /// Verify token
    fn verify_token(&self, token: &str) -> Result<bool>;
    
    /// Get token
    fn get_token(&self, token: &str) -> Result<AuthToken>;
    
    /// Revoke token
    fn revoke_token(&self, token: &str) -> Result<()>;
    
    /// Refresh token
    fn refresh_token(&self, token: &str) -> Result<AuthToken>;
    
    /// Register user
    fn register_user(&self, username: &str, password: &str) -> Result<()>;
    
    /// Change password
    fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<()>;
    
    /// Reset password
    fn reset_password(&self, username: &str) -> Result<String>;
}

/// Local authentication provider
pub struct LocalAuthProvider {
    /// User store
    user_store: Arc<dyn UserStore>,
    
    /// Token store
    token_store: Arc<dyn TokenStore>,
}

impl LocalAuthProvider {
    /// Create a new LocalAuthProvider
    pub fn new(user_store: Arc<dyn UserStore>, token_store: Arc<dyn TokenStore>) -> Self {
        Self {
            user_store,
            token_store,
        }
    }
    
    /// Hash password
    fn hash_password(password: &str, salt: &[u8]) -> Result<String> {
        let config = Argon2Config {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 4096,
            time_cost: 3,
            lanes: 4,
            thread_mode: ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: 32,
        };
        
        argon2::hash_encoded(password.as_bytes(), salt, &config)
            .map_err(|e| AuthError::InternalError(format!("Failed to hash password: {}", e)))
    }
    
    /// Verify password
    fn verify_password(hash: &str, password: &str) -> Result<bool> {
        argon2::verify_encoded(hash, password.as_bytes())
            .map_err(|e| AuthError::InternalError(format!("Failed to verify password: {}", e)))
    }
}

impl AuthenticationProvider for LocalAuthProvider {
    fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Get user
        let user = self.user_store.get_user(&credentials.username)?;
        
        // Authenticate
        if let Some(password) = &credentials.password {
            // Password authentication
            if let Some(hash) = &user.password_hash {
                if Self::verify_password(hash, password)? {
                    // Create token
                    let token = AuthToken::new(
                        &user.id,
                        TokenType::Access,
                        vec!["*".to_string()],
                        Duration::from_secs(3600),
                    );
                    
                    // Store token
                    self.token_store.store_token(&token)?;
                    
                    return Ok(token);
                }
            }
        } else if let Some(token_value) = &credentials.token {
            // Token authentication
            if let Ok(token) = self.token_store.get_token(token_value) {
                if !token.is_expired() && token.user_id == user.id {
                    return Ok(token);
                }
            }
        } else if let Some(device_id) = &credentials.device_id {
            // Device authentication
            if let Some(stored_device_id) = &user.device_id {
                if device_id == stored_device_id {
                    // Create token
                    let token = AuthToken::new(
                        &user.id,
                        TokenType::Device,
                        vec!["device".to_string()],
                        Duration::from_secs(86400 * 30), // 30 days
                    );
                    
                    // Store token
                    self.token_store.store_token(&token)?;
                    
                    return Ok(token);
                }
            }
        }
        
        Err(AuthError::InvalidCredentials)
    }
    
    fn verify_token(&self, token: &str) -> Result<bool> {
        match self.token_store.get_token(token) {
            Ok(token) => Ok(!token.is_expired()),
            Err(_) => Ok(false),
        }
    }
    
    fn get_token(&self, token: &str) -> Result<AuthToken> {
        self.token_store.get_token(token)
    }
    
    fn revoke_token(&self, token: &str) -> Result<()> {
        self.token_store.revoke_token(token)
    }
    
    fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        // Get token
        let old_token = self.token_store.get_token(token)?;
        
        // Check if token is a refresh token
        if old_token.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidToken);
        }
        
        // Check if token is expired
        if old_token.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        
        // Create new token
        let new_token = AuthToken::new(
            &old_token.user_id,
            TokenType::Access,
            old_token.scope.clone(),
            Duration::from_secs(3600),
        );
        
        // Store new token
        self.token_store.store_token(&new_token)?;
        
        // Revoke old token
        self.token_store.revoke_token(&old_token.value)?;
        
        Ok(new_token)
    }
    
    fn register_user(&self, username: &str, password: &str) -> Result<()> {
        // Check if user already exists
        if self.user_store.user_exists(username)? {
            return Err(AuthError::InternalError(format!("User {} already exists", username)));
        }
        
        // Generate salt
        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);
        
        // Hash password
        let password_hash = Self::hash_password(password, &salt)?;
        
        // Create user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: Some(password_hash),
            device_id: None,
            profile: None,
        };
        
        // Store user
        self.user_store.store_user(&user)?;
        
        Ok(())
    }
    
    fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<()> {
        // Get user
        let mut user = self.user_store.get_user(username)?;
        
        // Verify old password
        if let Some(hash) = &user.password_hash {
            if !Self::verify_password(hash, old_password)? {
                return Err(AuthError::InvalidCredentials);
            }
        } else {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Generate salt
        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);
        
        // Hash new password
        let password_hash = Self::hash_password(new_password, &salt)?;
        
        // Update user
        user.password_hash = Some(password_hash);
        
        // Store user
        self.user_store.store_user(&user)?;
        
        // Revoke all tokens for user
        self.token_store.revoke_all_tokens_for_user(&user.id)?;
        
        Ok(())
    }
    
    fn reset_password(&self, username: &str) -> Result<String> {
        // Get user
        let mut user = self.user_store.get_user(username)?;
        
        // Generate new password
        let new_password = Self::generate_random_password();
        
        // Generate salt
        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);
        
        // Hash new password
        let password_hash = Self::hash_password(&new_password, &salt)?;
        
        // Update user
        user.password_hash = Some(password_hash);
        
        // Store user
        self.user_store.store_user(&user)?;
        
        // Revoke all tokens for user
        self.token_store.revoke_all_tokens_for_user(&user.id)?;
        
        Ok(new_password)
    }
}

impl LocalAuthProvider {
    /// Generate random password
    fn generate_random_password() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const PASSWORD_LEN: usize = 12;
        
        let mut rng = rand::thread_rng();
        let password: String = (0..PASSWORD_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        password
    }
}

/// User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: String,
    
    /// Username
    pub username: String,
    
    /// Password hash
    pub password_hash: Option<String>,
    
    /// Device ID
    pub device_id: Option<String>,
    
    /// User profile
    pub profile: Option<UserProfile>,
}

/// User store trait
pub trait UserStore: Send + Sync {
    /// Get user by username
    fn get_user(&self, username: &str) -> Result<User>;
    
    /// Store user
    fn store_user(&self, user: &User) -> Result<()>;
    
    /// Delete user
    fn delete_user(&self, username: &str) -> Result<()>;
    
    /// Check if user exists
    fn user_exists(&self, username: &str) -> Result<bool>;
}

/// Token store trait
pub trait TokenStore: Send + Sync {
    /// Get token by value
    fn get_token(&self, token: &str) -> Result<AuthToken>;
    
    /// Store token
    fn store_token(&self, token: &AuthToken) -> Result<()>;
    
    /// Revoke token
    fn revoke_token(&self, token: &str) -> Result<()>;
    
    /// Revoke all tokens for user
    fn revoke_all_tokens_for_user(&self, user_id: &str) -> Result<()>;
}

/// Memory user store
pub struct MemoryUserStore {
    /// Users
    users: std::sync::RwLock<HashMap<String, User>>,
}

impl MemoryUserStore {
    /// Create a new MemoryUserStore
    pub fn new() -> Self {
        Self {
            users: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl UserStore for MemoryUserStore {
    fn get_user(&self, username: &str) -> Result<User> {
        let users = self.users.read().unwrap();
        
        users.get(username)
            .cloned()
            .ok_or_else(|| AuthError::UserNotFound(username.to_string()))
    }
    
    fn store_user(&self, user: &User) -> Result<()> {
        let mut users = self.users.write().unwrap();
        
        users.insert(user.username.clone(), user.clone());
        
        Ok(())
    }
    
    fn delete_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.write().unwrap();
        
        if users.remove(username).is_none() {
            return Err(AuthError::UserNotFound(username.to_string()));
        }
        
        Ok(())
    }
    
    fn user_exists(&self, username: &str) -> Result<bool> {
        let users = self.users.read().unwrap();
        
        Ok(users.contains_key(username))
    }
}

/// Memory token store
pub struct MemoryTokenStore {
    /// Tokens
    tokens: std::sync::RwLock<HashMap<String, AuthToken>>,
}

impl MemoryTokenStore {
    /// Create a new MemoryTokenStore
    pub fn new() -> Self {
        Self {
            tokens: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl TokenStore for MemoryTokenStore {
    fn get_token(&self, token: &str) -> Result<AuthToken> {
        let tokens = self.tokens.read().unwrap();
        
        tokens.get(token)
            .cloned()
            .ok_or_else(|| AuthError::InvalidToken)
    }
    
    fn store_token(&self, token: &AuthToken) -> Result<()> {
        let mut tokens = self.tokens.write().unwrap();
        
        tokens.insert(token.value.clone(), token.clone());
        
        Ok(())
    }
    
    fn revoke_token(&self, token: &str) -> Result<()> {
        let mut tokens = self.tokens.write().unwrap();
        
        if tokens.remove(token).is_none() {
            return Err(AuthError::InvalidToken);
        }
        
        Ok(())
    }
    
    fn revoke_all_tokens_for_user(&self, user_id: &str) -> Result<()> {
        let mut tokens = self.tokens.write().unwrap();
        
        tokens.retain(|_, token| token.user_id != user_id);
        
        Ok(())
    }
}

/// Mock authentication provider for testing
#[cfg(test)]
pub struct MockAuthenticationProvider;

#[cfg(test)]
impl MockAuthenticationProvider {
    /// Create a new MockAuthenticationProvider
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl AuthenticationProvider for MockAuthenticationProvider {
    fn authenticate(&self, _credentials: &Credentials) -> Result<AuthToken> {
        Ok(AuthToken {
            id: "mock_token_id".to_string(),
            user_id: "mock_user_id".to_string(),
            value: "mock_token_value".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 3600,
            token_type: TokenType::Access,
            scope: vec!["*".to_string()],
        })
    }
    
    fn verify_token(&self, _token: &str) -> Result<bool> {
        Ok(true)
    }
    
    fn get_token(&self, _token: &str) -> Result<AuthToken> {
        Ok(AuthToken {
            id: "mock_token_id".to_string(),
            user_id: "mock_user_id".to_string(),
            value: "mock_token_value".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 3600,
            token_type: TokenType::Access,
            scope: vec!["*".to_string()],
        })
    }
    
    fn revoke_token(&self, _token: &str) -> Result<()> {
        Ok(())
    }
    
    fn refresh_token(&self, _token: &str) -> Result<AuthToken> {
        Ok(AuthToken {
            id: "mock_token_id".to_string(),
            user_id: "mock_user_id".to_string(),
            value: "mock_token_value".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 3600,
            token_type: TokenType::Access,
            scope: vec!["*".to_string()],
        })
    }
    
    fn register_user(&self, _username: &str, _password: &str) -> Result<()> {
        Ok(())
    }
    
    fn change_password(&self, _username: &str, _old_password: &str, _new_password: &str) -> Result<()> {
        Ok(())
    }
    
    fn reset_password(&self, _username: &str) -> Result<String> {
        Ok("new_password".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_auth_token_creation() {
        let token = AuthToken::new(
            "user123",
            TokenType::Access,
            vec!["read".to_string(), "write".to_string()],
            Duration::from_secs(3600),
        );
        
        assert_eq!(token.user_id, "user123");
        assert_eq!(token.token_type, TokenType::Access);
        assert_eq!(token.scope, vec!["read".to_string(), "write".to_string()]);
        assert!(!token.is_expired());
    }
    
    #[test]
    fn test_auth_token_expiration() {
        let token = AuthToken {
            id: "token123".to_string(),
            user_id: "user123".to_string(),
            value: "value123".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - 1,
            token_type: TokenType::Access,
            scope: vec!["read".to_string()],
        };
        
        assert!(token.is_expired());
    }
    
    #[test]
    fn test_auth_token_scope() {
        let token = AuthToken::new(
            "user123",
            TokenType::Access,
            vec!["read".to_string(), "write".to_string()],
            Duration::from_secs(3600),
        );
        
        assert!(token.has_scope("read"));
        assert!(token.has_scope("write"));
        assert!(!token.has_scope("admin"));
    }
    
    #[test]
    fn test_credentials_to_token() {
        let creds = Credentials::new_with_password("user123", "password123");
        let token = creds.to_token().unwrap();
        assert!(!token.is_empty());
        
        let creds = Credentials::new_with_token("user123", "token123");
        let token = creds.to_token().unwrap();
        assert_eq!(token, "token123");
        
        let creds = Credentials::new_with_device_id("user123", "device123");
        let token = creds.to_token().unwrap();
        assert!(!token.is_empty());
        
        let creds = Credentials {
            username: "user123".to_string(),
            password: None,
            token: None,
            device_id: None,
        };
        assert!(creds.to_token().is_err());
    }
    
    // Additional tests would be added here
}
