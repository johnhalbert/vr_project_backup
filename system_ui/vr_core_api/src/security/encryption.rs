//! Encryption module for the VR Core API.
//!
//! This module provides encryption functionality for the VR Core API,
//! including symmetric and asymmetric encryption, key management, and secure communication.

use std::fmt;
use std::sync::{Arc, Mutex};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use log::{debug, error, info, warn};
use rand::Rng;
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Encryption error
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    /// Key generation error
    #[error("Key generation error: {0}")]
    KeyGenerationError(String),
    
    /// Key loading error
    #[error("Key loading error: {0}")]
    KeyLoadingError(String),
    
    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    /// Internal error
    #[error("Internal encryption error: {0}")]
    InternalError(String),
}

/// Encryption result
pub type Result<T> = std::result::Result<T, EncryptionError>;

/// Encryption key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptionKey {
    /// Key ID
    pub id: String,
    
    /// Key type
    pub key_type: KeyType,
    
    /// Key data
    pub data: Vec<u8>,
    
    /// Key metadata
    pub metadata: KeyMetadata,
}

impl EncryptionKey {
    /// Create a new symmetric encryption key
    pub fn new_symmetric() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let key_id = format!("sym-{}", uuid::Uuid::new_v4());
        let key_data: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        Ok(Self {
            id: key_id,
            key_type: KeyType::Symmetric,
            data: key_data,
            metadata: KeyMetadata::new(),
        })
    }
    
    /// Create a new asymmetric encryption key pair
    pub fn new_asymmetric() -> Result<(Self, Self)> {
        let mut rng = rand::thread_rng();
        let key_id = format!("asym-{}", uuid::Uuid::new_v4());
        
        // Generate RSA key pair
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| EncryptionError::KeyGenerationError(e.to_string()))?;
        let public_key = RsaPublicKey::from(&private_key);
        
        // Encode keys
        let private_key_data = private_key.to_pkcs1_der()
            .map_err(|e| EncryptionError::KeyGenerationError(e.to_string()))?
            .to_vec();
        let public_key_data = public_key.to_pkcs1_der()
            .map_err(|e| EncryptionError::KeyGenerationError(e.to_string()))?
            .to_vec();
        
        let private_key = Self {
            id: format!("{}-private", key_id),
            key_type: KeyType::AsymmetricPrivate,
            data: private_key_data,
            metadata: KeyMetadata::new(),
        };
        
        let public_key = Self {
            id: format!("{}-public", key_id),
            key_type: KeyType::AsymmetricPublic,
            data: public_key_data,
            metadata: KeyMetadata::new(),
        };
        
        Ok((private_key, public_key))
    }
    
    /// Create a new key from raw data
    pub fn from_raw(key_type: KeyType, data: Vec<u8>) -> Self {
        let key_id = match key_type {
            KeyType::Symmetric => format!("sym-{}", uuid::Uuid::new_v4()),
            KeyType::AsymmetricPrivate => format!("asym-{}-private", uuid::Uuid::new_v4()),
            KeyType::AsymmetricPublic => format!("asym-{}-public", uuid::Uuid::new_v4()),
        };
        
        Self {
            id: key_id,
            key_type,
            data,
            metadata: KeyMetadata::new(),
        }
    }
    
    /// Get key as AES-GCM key
    pub fn as_aes_gcm_key(&self) -> Result<Aes256Gcm> {
        if self.key_type != KeyType::Symmetric {
            return Err(EncryptionError::KeyLoadingError(
                "Key is not a symmetric key".to_string(),
            ));
        }
        
        if self.data.len() != 32 {
            return Err(EncryptionError::KeyLoadingError(
                format!("Invalid key length: {}", self.data.len()),
            ));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&self.data);
        let cipher = Aes256Gcm::new(key);
        
        Ok(cipher)
    }
    
    /// Get key as RSA private key
    pub fn as_rsa_private_key(&self) -> Result<RsaPrivateKey> {
        if self.key_type != KeyType::AsymmetricPrivate {
            return Err(EncryptionError::KeyLoadingError(
                "Key is not an asymmetric private key".to_string(),
            ));
        }
        
        RsaPrivateKey::from_pkcs1_der(&self.data)
            .map_err(|e| EncryptionError::KeyLoadingError(e.to_string()))
    }
    
    /// Get key as RSA public key
    pub fn as_rsa_public_key(&self) -> Result<RsaPublicKey> {
        if self.key_type != KeyType::AsymmetricPublic {
            return Err(EncryptionError::KeyLoadingError(
                "Key is not an asymmetric public key".to_string(),
            ));
        }
        
        RsaPublicKey::from_pkcs1_der(&self.data)
            .map_err(|e| EncryptionError::KeyLoadingError(e.to_string()))
    }
}

/// Key type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    /// Symmetric key
    Symmetric,
    
    /// Asymmetric private key
    AsymmetricPrivate,
    
    /// Asymmetric public key
    AsymmetricPublic,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyType::Symmetric => write!(f, "symmetric"),
            KeyType::AsymmetricPrivate => write!(f, "asymmetric_private"),
            KeyType::AsymmetricPublic => write!(f, "asymmetric_public"),
        }
    }
}

/// Key metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Creation time (Unix timestamp in seconds)
    pub created_at: u64,
    
    /// Last used time (Unix timestamp in seconds)
    pub last_used_at: Option<u64>,
    
    /// Description
    pub description: Option<String>,
    
    /// Tags
    pub tags: Vec<String>,
}

impl KeyMetadata {
    /// Create new key metadata
    pub fn new() -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            created_at,
            last_used_at: None,
            description: None,
            tags: Vec::new(),
        }
    }
    
    /// Update last used time
    pub fn update_last_used(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.last_used_at = Some(now);
    }
    
    /// Set description
    pub fn set_description(&mut self, description: &str) {
        self.description = Some(description.to_string());
    }
    
    /// Add tag
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
    
    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

/// Encryption provider trait
pub trait EncryptionProvider: Send + Sync {
    /// Encrypt data with symmetric key
    fn encrypt_symmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;
    
    /// Decrypt data with symmetric key
    fn decrypt_symmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;
    
    /// Encrypt data with asymmetric key
    fn encrypt_asymmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;
    
    /// Decrypt data with asymmetric key
    fn decrypt_asymmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;
    
    /// Generate symmetric key
    fn generate_symmetric_key(&self) -> Result<EncryptionKey>;
    
    /// Generate asymmetric key pair
    fn generate_asymmetric_key_pair(&self) -> Result<(EncryptionKey, EncryptionKey)>;
    
    /// Get key by ID
    fn get_key(&self, key_id: &str) -> Result<EncryptionKey>;
    
    /// Store key
    fn store_key(&self, key: &EncryptionKey) -> Result<()>;
    
    /// Delete key
    fn delete_key(&self, key_id: &str) -> Result<()>;
    
    /// List keys
    fn list_keys(&self) -> Result<Vec<EncryptionKey>>;
    
    /// Hash data
    fn hash_data(&self, data: &[u8]) -> Vec<u8>;
    
    /// Verify hash
    fn verify_hash(&self, data: &[u8], hash: &[u8]) -> bool;
}

/// Local encryption provider
pub struct LocalEncryptionProvider {
    /// Keys
    keys: Mutex<std::collections::HashMap<String, EncryptionKey>>,
}

impl LocalEncryptionProvider {
    /// Create a new LocalEncryptionProvider
    pub fn new() -> Self {
        Self {
            keys: Mutex::new(std::collections::HashMap::new()),
        }
    }
    
    /// Generate nonce for AES-GCM
    fn generate_nonce() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut nonce = vec![0u8; 12];
        rng.fill(&mut nonce[..]);
        nonce
    }
}

impl EncryptionProvider for LocalEncryptionProvider {
    fn encrypt_symmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        // Get key
        let key = self.get_key(key_id)?;
        
        // Update last used time
        {
            let mut keys = self.keys.lock().unwrap();
            if let Some(stored_key) = keys.get_mut(key_id) {
                stored_key.metadata.update_last_used();
            }
        }
        
        // Get cipher
        let cipher = key.as_aes_gcm_key()?;
        
        // Generate nonce
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionError(e.to_string()))?;
        
        // Combine nonce and ciphertext
        let mut result = nonce_bytes;
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    fn decrypt_symmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        // Check data length
        if data.len() < 12 {
            return Err(EncryptionError::DecryptionError(
                "Invalid data length".to_string(),
            ));
        }
        
        // Get key
        let key = self.get_key(key_id)?;
        
        // Update last used time
        {
            let mut keys = self.keys.lock().unwrap();
            if let Some(stored_key) = keys.get_mut(key_id) {
                stored_key.metadata.update_last_used();
            }
        }
        
        // Get cipher
        let cipher = key.as_aes_gcm_key()?;
        
        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&data[0..12]);
        let ciphertext = &data[12..];
        
        // Decrypt data
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionError(e.to_string()))?;
        
        Ok(plaintext)
    }
    
    fn encrypt_asymmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        // Get key
        let key = self.get_key(key_id)?;
        
        // Update last used time
        {
            let mut keys = self.keys.lock().unwrap();
            if let Some(stored_key) = keys.get_mut(key_id) {
                stored_key.metadata.update_last_used();
            }
        }
        
        // Get public key
        let public_key = key.as_rsa_public_key()?;
        
        // Encrypt data
        let mut rng = rand::thread_rng();
        let padding = rsa::pkcs1v15::Pkcs1v15Encrypt;
        
        let ciphertext = public_key.encrypt(&mut rng, padding, data)
            .map_err(|e| EncryptionError::EncryptionError(e.to_string()))?;
        
        Ok(ciphertext)
    }
    
    fn decrypt_asymmetric(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        // Get key
        let key = self.get_key(key_id)?;
        
        // Update last used time
        {
            let mut keys = self.keys.lock().unwrap();
            if let Some(stored_key) = keys.get_mut(key_id) {
                stored_key.metadata.update_last_used();
            }
        }
        
        // Get private key
        let private_key = key.as_rsa_private_key()?;
        
        // Decrypt data
        let padding = rsa::pkcs1v15::Pkcs1v15Encrypt;
        
        let plaintext = private_key.decrypt(padding, data)
            .map_err(|e| EncryptionError::DecryptionError(e.to_string()))?;
        
        Ok(plaintext)
    }
    
    fn generate_symmetric_key(&self) -> Result<EncryptionKey> {
        let key = EncryptionKey::new_symmetric()?;
        self.store_key(&key)?;
        Ok(key)
    }
    
    fn generate_asymmetric_key_pair(&self) -> Result<(EncryptionKey, EncryptionKey)> {
        let (private_key, public_key) = EncryptionKey::new_asymmetric()?;
        self.store_key(&private_key)?;
        self.store_key(&public_key)?;
        Ok((private_key, public_key))
    }
    
    fn get_key(&self, key_id: &str) -> Result<EncryptionKey> {
        let keys = self.keys.lock().unwrap();
        
        keys.get(key_id)
            .cloned()
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id.to_string()))
    }
    
    fn store_key(&self, key: &EncryptionKey) -> Result<()> {
        let mut keys = self.keys.lock().unwrap();
        keys.insert(key.id.clone(), key.clone());
        Ok(())
    }
    
    fn delete_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.lock().unwrap();
        
        if keys.remove(key_id).is_none() {
            return Err(EncryptionError::KeyNotFound(key_id.to_string()));
        }
        
        Ok(())
    }
    
    fn list_keys(&self) -> Result<Vec<EncryptionKey>> {
        let keys = self.keys.lock().unwrap();
        Ok(keys.values().cloned().collect())
    }
    
    fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    
    fn verify_hash(&self, data: &[u8], hash: &[u8]) -> bool {
        let calculated_hash = self.hash_data(data);
        calculated_hash == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symmetric_key_generation() {
        let key = EncryptionKey::new_symmetric().unwrap();
        assert_eq!(key.key_type, KeyType::Symmetric);
        assert_eq!(key.data.len(), 32);
    }
    
    #[test]
    fn test_asymmetric_key_generation() {
        let (private_key, public_key) = EncryptionKey::new_asymmetric().unwrap();
        assert_eq!(private_key.key_type, KeyType::AsymmetricPrivate);
        assert_eq!(public_key.key_type, KeyType::AsymmetricPublic);
    }
    
    #[test]
    fn test_symmetric_encryption() {
        let provider = LocalEncryptionProvider::new();
        let key = provider.generate_symmetric_key().unwrap();
        
        let data = b"Hello, world!";
        let encrypted = provider.encrypt_symmetric(data, &key.id).unwrap();
        let decrypted = provider.decrypt_symmetric(&encrypted, &key.id).unwrap();
        
        assert_eq!(decrypted, data);
    }
    
    #[test]
    fn test_asymmetric_encryption() {
        let provider = LocalEncryptionProvider::new();
        let (private_key, public_key) = provider.generate_asymmetric_key_pair().unwrap();
        
        let data = b"Hello, world!";
        let encrypted = provider.encrypt_asymmetric(data, &public_key.id).unwrap();
        let decrypted = provider.decrypt_asymmetric(&encrypted, &private_key.id).unwrap();
        
        assert_eq!(decrypted, data);
    }
    
    #[test]
    fn test_hash_verification() {
        let provider = LocalEncryptionProvider::new();
        
        let data = b"Hello, world!";
        let hash = provider.hash_data(data);
        
        assert!(provider.verify_hash(data, &hash));
        assert!(!provider.verify_hash(b"Modified data", &hash));
    }
}
