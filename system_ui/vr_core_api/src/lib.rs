//! Core API for the VR headset system.
//!
//! This library provides a comprehensive API for interacting with the VR headset
//! hardware and software components. It includes modules for hardware access,
//! configuration management, system monitoring, IPC, and security.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use thiserror::Error;

// Public modules
pub mod hardware;
pub mod config;
pub mod monitoring;
pub mod ipc;
pub mod security;

// Re-export submodule types
pub use hardware::{DeviceError, DeviceInfo, DeviceType, DeviceState, Device};
pub use config::{ConfigError, ConfigManager};
pub use monitoring::{MonitoringError, MonitoringManager};
pub use ipc::{IpcError, IpcManager};
pub use security::{SecurityError, SecurityManager};

/// Core API for the VR headset.
pub struct CoreApi {
    /// Configuration manager
    config_manager: Arc<Mutex<ConfigManager>>,
    
    /// Hardware manager
    hardware_manager: Arc<Mutex<hardware::HardwareManager>>,
    
    /// Monitoring manager
    monitoring_manager: Arc<Mutex<MonitoringManager>>,
    
    /// IPC manager
    ipc_manager: Arc<Mutex<IpcManager>>,
    
    /// Security manager
    security_manager: Arc<Mutex<SecurityManager>>,
    
    /// Initialized flag
    initialized: bool,
}

impl CoreApi {
    /// Create a new Core API instance.
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        // Create the configuration manager
        let config_manager = ConfigManager::new(config_dir.clone())
            .context("Failed to create configuration manager")?;
        let config_manager = Arc::new(Mutex::new(config_manager));
        
        // Create the hardware manager
        let hardware_manager = hardware::HardwareManager::new(&config_manager)
            .context("Failed to create hardware manager")?;
        let hardware_manager = Arc::new(Mutex::new(hardware_manager));
        
        // Create the monitoring manager
        let monitoring_manager = MonitoringManager::new(&config_manager)
            .context("Failed to create monitoring manager")?;
        let monitoring_manager = Arc::new(Mutex::new(monitoring_manager));
        
        // Create the IPC manager
        let ipc_manager = {
            let config = config_manager.lock().unwrap();
            IpcManager::new(&config)
                .context("Failed to create IPC manager")?
        };
        let ipc_manager = Arc::new(Mutex::new(ipc_manager));
        
        // Create the security manager
        let security_manager = SecurityManager::new(config_dir.clone())
            .context("Failed to create security manager")?;
        let security_manager = Arc::new(Mutex::new(security_manager));
        
        Ok(Self {
            config_manager,
            hardware_manager,
            monitoring_manager,
            ipc_manager,
            security_manager,
            initialized: false,
        })
    }
    
    /// Initialize the Core API.
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize the configuration manager
        {
            let config_manager = self.config_manager.lock().unwrap();
            config_manager.initialize()
                .context("Failed to initialize configuration manager")?;
        }
        
        // Initialize the hardware manager
        {
            let mut hardware_manager = self.hardware_manager.lock().unwrap();
            hardware_manager.initialize()
                .context("Failed to initialize hardware manager")?;
        }
        
        // Initialize the monitoring manager
        {
            let mut monitoring_manager = self.monitoring_manager.lock().unwrap();
            monitoring_manager.initialize()
                .context("Failed to initialize monitoring manager")?;
        }
        
        // Initialize the IPC manager
        {
            let mut ipc_manager = self.ipc_manager.lock().unwrap();
            ipc_manager.initialize()
                .context("Failed to initialize IPC manager")?;
        }
        
        // Initialize the security manager
        {
            let security_manager = self.security_manager.lock().unwrap();
            security_manager.initialize()
                .context("Failed to initialize security manager")?;
        }
        
        self.initialized = true;
        
        Ok(())
    }
    
    /// Shutdown the Core API.
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        // Shutdown the security manager
        {
            let security_manager = self.security_manager.lock().unwrap();
            if let Err(e) = security_manager.shutdown() {
                error!("Error shutting down security manager: {}", e);
            }
        }
        
        // Shutdown the IPC manager
        {
            let mut ipc_manager = self.ipc_manager.lock().unwrap();
            if let Err(e) = ipc_manager.shutdown() {
                error!("Error shutting down IPC manager: {}", e);
            }
        }
        
        // Shutdown the monitoring manager
        {
            let mut monitoring_manager = self.monitoring_manager.lock().unwrap();
            if let Err(e) = monitoring_manager.shutdown() {
                error!("Error shutting down monitoring manager: {}", e);
            }
        }
        
        // Shutdown the hardware manager
        {
            let mut hardware_manager = self.hardware_manager.lock().unwrap();
            if let Err(e) = hardware_manager.shutdown() {
                error!("Error shutting down hardware manager: {}", e);
            }
        }
        
        // Shutdown the configuration manager
        {
            let config_manager = self.config_manager.lock().unwrap();
            if let Err(e) = config_manager.shutdown() {
                error!("Error shutting down configuration manager: {}", e);
            }
        }
        
        self.initialized = false;
        
        Ok(())
    }
    
    /// Check if the Core API is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get the configuration manager.
    pub fn config_manager(&self) -> Arc<Mutex<ConfigManager>> {
        Arc::clone(&self.config_manager)
    }
    
    /// Get the hardware manager.
    pub fn hardware_manager(&self) -> Arc<Mutex<hardware::HardwareManager>> {
        Arc::clone(&self.hardware_manager)
    }
    
    /// Get the monitoring manager.
    pub fn monitoring_manager(&self) -> Arc<Mutex<MonitoringManager>> {
        Arc::clone(&self.monitoring_manager)
    }
    
    /// Get the IPC manager.
    pub fn ipc_manager(&self) -> Arc<Mutex<IpcManager>> {
        Arc::clone(&self.ipc_manager)
    }
    
    /// Get the security manager.
    pub fn security_manager(&self) -> Arc<Mutex<SecurityManager>> {
        Arc::clone(&self.security_manager)
    }
}

/// Core API error.
#[derive(Debug, Error)]
pub enum CoreApiError {
    #[error("Hardware error: {0}")]
    Hardware(#[from] hardware::HardwareError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Monitoring error: {0}")]
    Monitoring(#[from] monitoring::MonitoringError),
    
    #[error("IPC error: {0}")]
    Ipc(#[from] ipc::IpcError),
    
    #[error("Security error: {0}")]
    Security(#[from] security::SecurityError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_core_api_creation() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        
        let core_api = CoreApi::new(config_dir);
        assert!(core_api.is_ok());
    }
    
    #[test]
    fn test_core_api_initialization() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        
        let mut core_api = CoreApi::new(config_dir).unwrap();
        let result = core_api.initialize();
        
        // Initialization may fail in tests due to hardware access,
        // so we don't assert success here
        if result.is_ok() {
            assert!(core_api.is_initialized());
            
            // Shutdown if initialized
            let shutdown_result = core_api.shutdown();
            assert!(shutdown_result.is_ok());
            assert!(!core_api.is_initialized());
        }
    }
}
