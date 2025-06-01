//! Hardware access API for the VR headset.
//!
//! This module provides hardware access functionality for the VR headset,
//! including device discovery, management, and control.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use log::{debug, error, info, warn};
use thiserror::Error;

pub mod device;
pub mod manager;
pub mod display;
pub mod audio;
pub mod tracking;
pub mod power;
pub mod storage;
pub mod network;

pub use device::{Device, DeviceInfo, DeviceType, DeviceState, DeviceError};
use display::DisplayManager;
use audio::AudioManager;
use tracking::TrackingManager;
use power::PowerManager;
use storage::StorageManager;
use network::NetworkManager;
use crate::config::ConfigManager;

/// Hardware manager for the VR headset.
pub struct HardwareManager {
    /// Devices
    devices: Arc<RwLock<HashMap<String, Box<dyn Device>>>>,
    
    /// Display manager
    display_manager: Arc<Mutex<DisplayManager>>,
    
    /// Audio manager
    audio_manager: Arc<Mutex<AudioManager>>,
    
    /// Tracking manager
    tracking_manager: Arc<Mutex<TrackingManager>>,
    
    /// Power manager
    power_manager: Arc<Mutex<PowerManager>>,
    
    /// Storage manager
    storage_manager: Arc<Mutex<StorageManager>>,
    
    /// Network manager
    network_manager: Arc<Mutex<NetworkManager>>,
}

impl HardwareManager {
    /// Create a new hardware manager.
    pub fn new(_config: &Arc<Mutex<ConfigManager>>) -> Result<Self> {
        // Create the display manager
        let display_manager = DisplayManager::new()?;
        let display_manager = Arc::new(Mutex::new(display_manager));
        
        // Create the audio manager
        let audio_manager = AudioManager::new()?;
        let audio_manager = Arc::new(Mutex::new(audio_manager));
        
        // Create the tracking manager
        let tracking_manager = TrackingManager::new()?;
        let tracking_manager = Arc::new(Mutex::new(tracking_manager));
        
        // Create the power manager
        let power_manager = PowerManager::new();
        let power_manager = Arc::new(Mutex::new(power_manager));
        
        // Create the storage manager
        let storage_manager = StorageManager::new();
        let storage_manager = Arc::new(Mutex::new(storage_manager));
        
        // Create the network manager
        let network_manager = NetworkManager::new();
        let network_manager = Arc::new(Mutex::new(network_manager));
        
        Ok(Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            display_manager,
            audio_manager,
            tracking_manager,
            power_manager,
            storage_manager,
            network_manager,
        })
    }
    
    /// Initialize the hardware manager.
    pub fn initialize(&mut self) -> Result<()> {
        // Initialize the display manager
        self.display_manager.lock().unwrap().initialize()?;
        
        // Initialize the audio manager
        self.audio_manager.lock().unwrap().initialize()?;
        
        // Initialize the tracking manager
        self.tracking_manager.lock().unwrap().initialize()?;
        
        // Initialize the power manager
        self.power_manager.lock().unwrap().initialize()?;
        
        // Initialize the storage manager
        self.storage_manager.lock().unwrap().initialize()?;
        
        // Initialize the network manager
        self.network_manager.lock().unwrap().initialize()?;
        
        // Discover devices
        self.discover_devices()?;
        
        Ok(())
    }
    
    /// Shutdown the hardware manager.
    pub fn shutdown(&mut self) -> Result<()> {
        // Shutdown the network manager
        if let Err(e) = self.network_manager.lock().unwrap().shutdown() {
            error!("Error shutting down network manager: {}", e);
        }
        
        // Shutdown the storage manager
        if let Err(e) = self.storage_manager.lock().unwrap().shutdown() {
            error!("Error shutting down storage manager: {}", e);
        }
        
        // Shutdown the power manager
        if let Err(e) = self.power_manager.lock().unwrap().shutdown() {
            error!("Error shutting down power manager: {}", e);
        }
        
        // Shutdown the tracking manager
        if let Err(e) = self.tracking_manager.lock().unwrap().shutdown() {
            error!("Error shutting down tracking manager: {}", e);
        }
        
        // Shutdown the audio manager
        if let Err(e) = self.audio_manager.lock().unwrap().shutdown() {
            error!("Error shutting down audio manager: {}", e);
        }
        
        // Shutdown the display manager
        if let Err(e) = self.display_manager.lock().unwrap().shutdown() {
            error!("Error shutting down display manager: {}", e);
        }
        
        Ok(())
    }
    
    /// Discover devices.
    fn discover_devices(&self) -> Result<()> {
        // TODO: Implement device discovery
        Ok(())
    }
    
    /// Get a device.
    pub fn get_device(&self, id: &str) -> Option<Box<dyn Device>> {
        let devices = self.devices.read().unwrap();
        
        match devices.get(id) {
            Some(device) => Some(device.clone_box()),
            None => None
        }
    }
    
    /// Get all devices.
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().unwrap();
        
        let mut result = Vec::new();
        for device in devices.values() {
            if let Ok(info) = device.info() {
                result.push(info);
            }
        }
        result
    }
    
    /// Get devices by type.
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<DeviceInfo> {
        let devices = self.devices.read().unwrap();
        
        let mut result = Vec::new();
        for device in devices.values() {
            if let Ok(info) = device.info() {
                if info.device_type == device_type {
                    result.push(info);
                }
            }
        }
        result
    }
    
    /// Add a device.
    pub fn add_device(&self, device: Box<dyn Device>) -> Result<()> {
        let mut devices = self.devices.write().unwrap();
        
        let info = device.info()?;
        let id = info.id.clone();
        devices.insert(id, device);
        
        Ok(())
    }
    
    /// Remove a device.
    pub fn remove_device(&self, id: &str) -> Result<()> {
        let mut devices = self.devices.write().unwrap();
        
        if devices.remove(id).is_none() {
            return Err(anyhow::anyhow!("Device not found: {}", id));
        }
        
        Ok(())
    }
    
    /// Get the display manager.
    pub fn display_manager(&self) -> Arc<Mutex<DisplayManager>> {
        Arc::clone(&self.display_manager)
    }
    
    /// Get the audio manager.
    pub fn audio_manager(&self) -> Arc<Mutex<AudioManager>> {
        Arc::clone(&self.audio_manager)
    }
    
    /// Get the tracking manager.
    pub fn tracking_manager(&self) -> Arc<Mutex<TrackingManager>> {
        Arc::clone(&self.tracking_manager)
    }
    
    /// Get the power manager.
    pub fn power_manager(&self) -> Arc<Mutex<PowerManager>> {
        Arc::clone(&self.power_manager)
    }
    
    /// Get the storage manager.
    pub fn storage_manager(&self) -> Arc<Mutex<StorageManager>> {
        Arc::clone(&self.storage_manager)
    }
    
    /// Get the network manager.
    pub fn network_manager(&self) -> Arc<Mutex<NetworkManager>> {
        Arc::clone(&self.network_manager)
    }
}

/// Hardware error.
#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("Device error: {0}")]
    Device(String),
    
    #[error("Display error: {0}")]
    Display(String),
    
    #[error("Audio error: {0}")]
    Audio(String),
    
    #[error("Tracking error: {0}")]
    Tracking(String),
    
    #[error("Power error: {0}")]
    Power(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
