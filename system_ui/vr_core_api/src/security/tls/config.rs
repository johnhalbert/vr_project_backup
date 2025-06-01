//! TLS configuration for the VR headset.
//!
//! This module provides TLS configuration functionality for the VR headset,
//! including cipher suite selection, protocol version, and other TLS settings.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use log::error;
use rustls::{Certificate, PrivateKey, ServerConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::certificate::StoredCertificate;

/// TLS configuration error.
#[derive(Debug, Error)]
pub enum TlsConfigError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// TLS configuration result type.
pub type TlsConfigResult<T> = Result<T, TlsConfigError>;

/// TLS configuration.
pub struct TlsConfig {
    /// Configuration directory
    config_dir: PathBuf,
    
    /// Configuration
    config: Arc<RwLock<TlsConfigData>>,
}

impl TlsConfig {
    /// Create a new TLS configuration.
    pub fn new(config_dir: PathBuf) -> TlsConfigResult<Self> {
        // Create the configuration directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        // Load or create the configuration
        let config_path = config_dir.join("tls_config.json");
        let config = if config_path.exists() {
            // Load the configuration
            let config_str = fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_str)
                .map_err(|e| TlsConfigError::Serialization(format!("Failed to parse TLS configuration: {}", e)))?
        } else {
            // Create a default configuration
            let config = TlsConfigData::default();
            
            // Save the configuration
            let config_str = serde_json::to_string_pretty(&config)
                .map_err(|e| TlsConfigError::Serialization(format!("Failed to serialize TLS configuration: {}", e)))?;
            
            let mut file = File::create(&config_path)?;
            file.write_all(config_str.as_bytes())?;
            
            config
        };
        
        Ok(Self {
            config_dir,
            config: Arc::new(RwLock::new(config)),
        })
    }
    
    /// Get the TLS configuration.
    pub fn get_config(&self) -> TlsConfigData {
        self.config.read().unwrap().clone()
    }
    
    /// Set the TLS configuration.
    pub fn set_config(&self, config: TlsConfigData) -> TlsConfigResult<()> {
        // Update the configuration
        *self.config.write().unwrap() = config.clone();
        
        // Save the configuration
        let config_path = self.config_dir.join("tls_config.json");
        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| TlsConfigError::Serialization(format!("Failed to serialize TLS configuration: {}", e)))?;
        
        let mut file = File::create(&config_path)?;
        file.write_all(config_str.as_bytes())?;
        
        Ok(())
    }
    
    /// Create a server configuration.
    pub fn create_server_config(&self, cert: &StoredCertificate) -> TlsConfigResult<ServerConfig> {
        // Get the configuration
        let config = self.get_config();
        
        // Create the certificate chain
        let cert_chain = vec![Certificate(cert.certificate.clone())];
        
        // Create the private key
        let private_key = PrivateKey(cert.private_key.clone());
        
        // Create the server configuration
        let mut server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| TlsConfigError::Config(format!("Failed to create server configuration: {}", e)))?;
        
        // Configure the server
        server_config.alpn_protocols = config.alpn_protocols.iter().map(|p| p.as_bytes().to_vec()).collect();
        
        Ok(server_config)
    }
}

/// TLS configuration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfigData {
    /// ALPN protocols
    pub alpn_protocols: Vec<String>,
    
    /// Cipher suites
    pub cipher_suites: Vec<String>,
    
    /// Protocol versions
    pub protocol_versions: Vec<String>,
    
    /// Session ticket lifetime
    pub session_ticket_lifetime: u32,
    
    /// Session ticket key rotation
    pub session_ticket_key_rotation: u32,
}

impl Default for TlsConfigData {
    fn default() -> Self {
        Self {
            alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
            cipher_suites: vec![
                "TLS13_AES_256_GCM_SHA384".to_string(),
                "TLS13_AES_128_GCM_SHA256".to_string(),
                "TLS13_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            protocol_versions: vec!["TLSv1.3".to_string(), "TLSv1.2".to_string()],
            session_ticket_lifetime: 86400,
            session_ticket_key_rotation: 43200,
        }
    }
}
