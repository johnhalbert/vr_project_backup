//! Secure storage module for the VR Core API.
//!
//! This module provides secure storage functionality for the VR Core API,
//! including encrypted file storage, secure key-value storage, and secure configuration.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use super::encryption::{EncryptionError, EncryptionKey, EncryptionProvider, KeyType};

/// Secure storage error
#[derive(Debug, Error)]
pub enum SecureStorageError {
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] EncryptionError),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Internal error
    #[error("Internal secure storage error: {0}")]
    InternalError(String),
}

/// Secure storage result
pub type Result<T> = std::result::Result<T, SecureStorageError>;

/// Secure storage trait
pub trait SecureStorage: Send + Sync {
    /// Store data
    fn store(&self, key: &str, data: &[u8]) -> Result<()>;
    
    /// Load data
    fn load(&self, key: &str) -> Result<Vec<u8>>;
    
    /// Delete data
    fn delete(&self, key: &str) -> Result<()>;
    
    /// Check if key exists
    fn exists(&self, key: &str) -> Result<bool>;
    
    /// List keys
    fn list_keys(&self) -> Result<Vec<String>>;
    
    /// Store serializable data
    fn store_serialized<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| SecureStorageError::SerializationError(e.to_string()))?;
        
        self.store(key, &serialized)
    }
    
    /// Load serializable data
    fn load_serialized<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let data = self.load(key)?;
        
        serde_json::from_slice(&data)
            .map_err(|e| SecureStorageError::SerializationError(e.to_string()))
    }
}

/// Memory secure storage
pub struct MemorySecureStorage {
    /// Encryption provider
    encryption_provider: Arc<dyn EncryptionProvider>,
    
    /// Encryption key ID
    encryption_key_id: String,
    
    /// Storage
    storage: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemorySecureStorage {
    /// Create a new MemorySecureStorage
    pub fn new(encryption_provider: Arc<dyn EncryptionProvider>, encryption_key_id: &str) -> Self {
        Self {
            encryption_provider,
            encryption_key_id: encryption_key_id.to_string(),
            storage: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new MemorySecureStorage with a new encryption key
    pub fn new_with_new_key(encryption_provider: Arc<dyn EncryptionProvider>) -> Result<Self> {
        let key = encryption_provider.generate_symmetric_key()?;
        
        Ok(Self::new(encryption_provider, &key.id))
    }
}

impl SecureStorage for MemorySecureStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        // Encrypt data
        let encrypted = self.encryption_provider.encrypt_symmetric(data, &self.encryption_key_id)?;
        
        // Store encrypted data
        let mut storage = self.storage.write().unwrap();
        storage.insert(key.to_string(), encrypted);
        
        Ok(())
    }
    
    fn load(&self, key: &str) -> Result<Vec<u8>> {
        // Get encrypted data
        let storage = self.storage.read().unwrap();
        let encrypted = storage.get(key)
            .cloned()
            .ok_or_else(|| SecureStorageError::KeyNotFound(key.to_string()))?;
        
        // Decrypt data
        let decrypted = self.encryption_provider.decrypt_symmetric(&encrypted, &self.encryption_key_id)?;
        
        Ok(decrypted)
    }
    
    fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.storage.write().unwrap();
        
        if storage.remove(key).is_none() {
            return Err(SecureStorageError::KeyNotFound(key.to_string()));
        }
        
        Ok(())
    }
    
    fn exists(&self, key: &str) -> Result<bool> {
        let storage = self.storage.read().unwrap();
        Ok(storage.contains_key(key))
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        let storage = self.storage.read().unwrap();
        Ok(storage.keys().cloned().collect())
    }
}

/// File secure storage
pub struct FileSecureStorage {
    /// Encryption provider
    encryption_provider: Arc<dyn EncryptionProvider>,
    
    /// Encryption key ID
    encryption_key_id: String,
    
    /// Storage directory
    storage_dir: PathBuf,
    
    /// Key index
    key_index: RwLock<HashMap<String, String>>,
}

impl FileSecureStorage {
    /// Create a new FileSecureStorage
    pub fn new(
        encryption_provider: Arc<dyn EncryptionProvider>,
        encryption_key_id: &str,
        storage_dir: &Path,
    ) -> Result<Self> {
        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(storage_dir)?;
        }
        
        // Create key index file path
        let key_index_path = storage_dir.join("key_index.json");
        
        // Load key index if it exists
        let key_index = if key_index_path.exists() {
            let mut file = File::open(&key_index_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            serde_json::from_str(&contents)
                .map_err(|e| SecureStorageError::SerializationError(e.to_string()))?
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            encryption_provider,
            encryption_key_id: encryption_key_id.to_string(),
            storage_dir: storage_dir.to_path_buf(),
            key_index: RwLock::new(key_index),
        })
    }
    
    /// Create a new FileSecureStorage with a new encryption key
    pub fn new_with_new_key(
        encryption_provider: Arc<dyn EncryptionProvider>,
        storage_dir: &Path,
    ) -> Result<Self> {
        let key = encryption_provider.generate_symmetric_key()?;
        
        Self::new(encryption_provider, &key.id, storage_dir)
    }
    
    /// Save key index
    fn save_key_index(&self) -> Result<()> {
        let key_index = self.key_index.read().unwrap();
        let key_index_path = self.storage_dir.join("key_index.json");
        
        let contents = serde_json::to_string(&*key_index)
            .map_err(|e| SecureStorageError::SerializationError(e.to_string()))?;
        
        let mut file = File::create(&key_index_path)?;
        file.write_all(contents.as_bytes())?;
        
        Ok(())
    }
    
    /// Get file path for key
    fn get_file_path(&self, file_id: &str) -> PathBuf {
        self.storage_dir.join(file_id)
    }
}

impl SecureStorage for FileSecureStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        // Encrypt data
        let encrypted = self.encryption_provider.encrypt_symmetric(data, &self.encryption_key_id)?;
        
        // Generate file ID
        let file_id = format!("{}.bin", uuid::Uuid::new_v4());
        let file_path = self.get_file_path(&file_id);
        
        // Write encrypted data to file
        let mut file = File::create(&file_path)?;
        file.write_all(&encrypted)?;
        
        // Update key index
        {
            let mut key_index = self.key_index.write().unwrap();
            key_index.insert(key.to_string(), file_id);
        }
        
        // Save key index
        self.save_key_index()?;
        
        Ok(())
    }
    
    fn load(&self, key: &str) -> Result<Vec<u8>> {
        // Get file ID from key index
        let file_id = {
            let key_index = self.key_index.read().unwrap();
            key_index.get(key)
                .cloned()
                .ok_or_else(|| SecureStorageError::KeyNotFound(key.to_string()))?
        };
        
        // Get file path
        let file_path = self.get_file_path(&file_id);
        
        // Read encrypted data from file
        let mut file = File::open(&file_path)?;
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted)?;
        
        // Decrypt data
        let decrypted = self.encryption_provider.decrypt_symmetric(&encrypted, &self.encryption_key_id)?;
        
        Ok(decrypted)
    }
    
    fn delete(&self, key: &str) -> Result<()> {
        // Get file ID from key index
        let file_id = {
            let mut key_index = self.key_index.write().unwrap();
            key_index.remove(key)
                .ok_or_else(|| SecureStorageError::KeyNotFound(key.to_string()))?
        };
        
        // Get file path
        let file_path = self.get_file_path(&file_id);
        
        // Delete file
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        
        // Save key index
        self.save_key_index()?;
        
        Ok(())
    }
    
    fn exists(&self, key: &str) -> Result<bool> {
        let key_index = self.key_index.read().unwrap();
        Ok(key_index.contains_key(key))
    }
    
    fn list_keys(&self) -> Result<Vec<String>> {
        let key_index = self.key_index.read().unwrap();
        Ok(key_index.keys().cloned().collect())
    }
}

/// Secure configuration
pub struct SecureConfig {
    /// Secure storage
    storage: Arc<dyn SecureStorage>,
    
    /// Configuration cache
    cache: RwLock<HashMap<String, serde_json::Value>>,
}

impl SecureConfig {
    /// Create a new SecureConfig
    pub fn new(storage: Arc<dyn SecureStorage>) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// Set configuration value
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        // Serialize value
        let json_value = serde_json::to_value(value)
            .map_err(|e| SecureStorageError::SerializationError(e.to_string()))?;
        
        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.to_string(), json_value.clone());
        }
        
        // Store in secure storage
        self.storage.store_serialized(key, &json_value)
    }
    
    /// Get configuration value
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(value) = cache.get(key) {
                return serde_json::from_value(value.clone())
                    .map_err(|e| SecureStorageError::SerializationError(e.to_string()));
            }
        }
        
        // Load from secure storage
        let json_value: serde_json::Value = self.storage.load_serialized(key)?;
        
        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.to_string(), json_value.clone());
        }
        
        // Deserialize value
        serde_json::from_value(json_value)
            .map_err(|e| SecureStorageError::SerializationError(e.to_string()))
    }
    
    /// Remove configuration value
    pub fn remove(&self, key: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.remove(key);
        }
        
        // Remove from secure storage
        self.storage.delete(key)
    }
    
    /// Check if configuration key exists
    pub fn exists(&self, key: &str) -> Result<bool> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if cache.contains_key(key) {
                return Ok(true);
            }
        }
        
        // Check secure storage
        self.storage.exists(key)
    }
    
    /// List configuration keys
    pub fn list_keys(&self) -> Result<Vec<String>> {
        self.storage.list_keys()
    }
    
    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::encryption::LocalEncryptionProvider;
    
    #[test]
    fn test_memory_secure_storage() {
        let encryption_provider = Arc::new(LocalEncryptionProvider::new());
        let key = encryption_provider.generate_symmetric_key().unwrap();
        let storage = MemorySecureStorage::new(encryption_provider, &key.id);
        
        // Store data
        let data = b"Hello, world!";
        storage.store("test_key", data).unwrap();
        
        // Check if key exists
        assert!(storage.exists("test_key").unwrap());
        assert!(!storage.exists("nonexistent_key").unwrap());
        
        // Load data
        let loaded = storage.load("test_key").unwrap();
        assert_eq!(loaded, data);
        
        // List keys
        let keys = storage.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test_key");
        
        // Delete data
        storage.delete("test_key").unwrap();
        assert!(!storage.exists("test_key").unwrap());
    }
    
    #[test]
    fn test_serialized_storage() {
        let encryption_provider = Arc::new(LocalEncryptionProvider::new());
        let key = encryption_provider.generate_symmetric_key().unwrap();
        let storage = MemorySecureStorage::new(encryption_provider, &key.id);
        
        // Test data
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            field1: String,
            field2: i32,
        }
        
        let data = TestData {
            field1: "test".to_string(),
            field2: 42,
        };
        
        // Store serialized data
        storage.store_serialized("test_key", &data).unwrap();
        
        // Load serialized data
        let loaded: TestData = storage.load_serialized("test_key").unwrap();
        assert_eq!(loaded, data);
    }
    
    #[test]
    fn test_secure_config() {
        let encryption_provider = Arc::new(LocalEncryptionProvider::new());
        let key = encryption_provider.generate_symmetric_key().unwrap();
        let storage = Arc::new(MemorySecureStorage::new(encryption_provider, &key.id));
        let config = SecureConfig::new(storage);
        
        // Set configuration value
        config.set("test_key", &"test_value").unwrap();
        
        // Get configuration value
        let value: String = config.get("test_key").unwrap();
        assert_eq!(value, "test_value");
        
        // Check if key exists
        assert!(config.exists("test_key").unwrap());
        
        // Remove configuration value
        config.remove("test_key").unwrap();
        assert!(!config.exists("test_key").unwrap());
    }
}
