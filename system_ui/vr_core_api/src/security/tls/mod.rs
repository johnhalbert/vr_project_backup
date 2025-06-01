//! TLS/HTTPS support for the VR headset.
//!
//! This module provides TLS/HTTPS functionality for the VR headset,
//! including certificate management, TLS configuration, and HTTPS server.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{error};
use rustls::ServerConfig;
use thiserror::Error;

pub mod certificate;
pub mod config;
pub mod cookie;
pub mod server;

use certificate::{CertificateError, CertificateManager};
use config::{TlsConfig, TlsConfigError};
use cookie::CookieManager;
use server::HttpsServer;

/// TLS manager for the VR headset.
pub struct TlsManager {
    /// Certificate manager
    cert_manager: Arc<Mutex<CertificateManager>>,
    
    /// TLS configuration
    tls_config: Arc<Mutex<TlsConfig>>,
    
    /// Cookie manager
    cookie_manager: Arc<Mutex<CookieManager>>,
    
    /// TLS directory
    tls_dir: PathBuf,
}

impl TlsManager {
    /// Create a new TLS manager.
    pub fn new(tls_dir: PathBuf) -> Result<Self> {
        // Create the certificate manager
        let cert_manager = CertificateManager::new(tls_dir.join("certificates"))?;
        let cert_manager = Arc::new(Mutex::new(cert_manager));
        
        // Create the TLS configuration
        let tls_config = TlsConfig::new(tls_dir.join("config"))?;
        let tls_config = Arc::new(Mutex::new(tls_config));
        
        // Create the cookie manager
        let cookie_manager = CookieManager::new();
        let cookie_manager = Arc::new(Mutex::new(cookie_manager));
        
        Ok(Self {
            cert_manager,
            tls_config,
            cookie_manager,
            tls_dir,
        })
    }
    
    /// Initialize the TLS manager.
    pub fn initialize(&self) -> Result<()> {
        // Nothing to initialize
        Ok(())
    }
    
    /// Shutdown the TLS manager.
    pub fn shutdown(&self) -> Result<()> {
        // Nothing to shutdown
        Ok(())
    }
    
    /// Get the certificate manager.
    pub fn cert_manager(&self) -> Arc<Mutex<CertificateManager>> {
        Arc::clone(&self.cert_manager)
    }
    
    /// Get the TLS configuration.
    pub fn tls_config(&self) -> Arc<Mutex<TlsConfig>> {
        Arc::clone(&self.tls_config)
    }
    
    /// Get the cookie manager.
    pub fn cookie_manager(&self) -> Arc<Mutex<CookieManager>> {
        Arc::clone(&self.cookie_manager)
    }
    
    /// Create a server configuration.
    pub fn create_server_config(&self, cert_name: &str) -> Result<Arc<ServerConfig>> {
        let cert = self.cert_manager.lock().unwrap().get_certificate(cert_name)?;
        let config = self.tls_config.lock().unwrap().create_server_config(&cert)?;
        Ok(Arc::new(config))
    }
    
    /// Create an HTTPS server.
    pub fn create_https_server(&self, addr: std::net::SocketAddr, cert_name: &str) -> Result<HttpsServer> {
        let server_config = self.create_server_config(cert_name)?;
        let cookie_manager = self.cookie_manager.lock().unwrap().clone();
        
        let server = HttpsServer::new(addr, server_config)
            .with_cookie_manager(cookie_manager);
        
        Ok(server)
    }
}

/// TLS error.
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Certificate error: {0}")]
    Certificate(#[from] CertificateError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] TlsConfigError),
    
    #[error("Server error: {0}")]
    Server(#[from] server::ServerError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
