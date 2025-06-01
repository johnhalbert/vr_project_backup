//! Device manager for the Hardware Access API.
//!
//! This module provides a centralized device management system for hardware devices,
//! allowing for device discovery, registration, and access.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};
use uuid::Uuid;

use super::device::{Device, DeviceCapability, DeviceError, DeviceInfo, DeviceResult, DeviceState, DeviceType};
use super::device_event_manager::DeviceEventManager;

/// Device discovery handler function type.
pub type DeviceDiscoveryHandler = Box<dyn Fn() -> DeviceResult<Vec<Box<dyn Device>>> + Send + Sync>;

/// Device manager for managing hardware devices.
pub struct DeviceManager {
    /// Devices by ID
    devices: RwLock<HashMap<String, Box<dyn Device>>>,
    
    /// Device discovery handlers by type
    discovery_handlers: RwLock<HashMap<DeviceType, Vec<DeviceDiscoveryHandler>>>,
    
    /// Device event manager
    event_manager: Arc<DeviceEventManager>,
    
    /// Auto-discovery enabled
    auto_discovery_enabled: bool,
    
    /// Device metadata
    device_metadata: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl DeviceManager {
    /// Create a new DeviceManager.
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            discovery_handlers: RwLock::new(HashMap::new()),
            event_manager: Arc::new(DeviceEventManager::default()),
            auto_discovery_enabled: false,
            device_metadata: RwLock::new(HashMap::new()),
        }
    }
    
    /// Initialize the device manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing DeviceManager");
        
        // Enable auto-discovery
        self.auto_discovery_enabled = true;
        
        // Run initial device discovery
        self.discover_devices()?;
        
        Ok(())
    }
    
    /// Shutdown the device manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down DeviceManager");
        
        // Disable auto-discovery
        self.auto_discovery_enabled = false;
        
        // Shutdown all devices
        let mut devices = self.devices.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on devices".to_string())
        })?;
        
        for (id, device) in devices.iter_mut() {
            info!("Shutting down device {}", id);
            
            if let Err(e) = device.shutdown() {
                warn!("Failed to shutdown device {}: {}", id, e);
            }
        }
        
        devices.clear();
        
        Ok(())
    }
    
    /// Register a device.
    pub fn register_device(&self, device: Box<dyn Device>) -> DeviceResult<String> {
        let device_info = device.info()?;
        let device_id = device_info.id.clone();
        
        info!("Registering device: {} ({})", device_info.name, device_id);
        
        let mut devices = self.devices.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on devices".to_string())
        })?;
        
        // Check if device already exists
        if devices.contains_key(&device_id) {
            return Err(DeviceError::InvalidParameter(format!("Device with ID {} already exists", device_id)));
        }
        
        // Add device to registry
        devices.insert(device_id.clone(), device);
        
        Ok(device_id)
    }
    
    /// Unregister a device.
    pub fn unregister_device(&self, device_id: &str) -> DeviceResult<()> {
        info!("Unregistering device: {}", device_id);
        
        let mut devices = self.devices.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on devices".to_string())
        })?;
        
        // Check if device exists
        if !devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!("Device with ID {} not found", device_id)));
        }
        
        // Remove device from registry
        devices.remove(device_id);
        
        // Remove device metadata
        let mut metadata = self.device_metadata.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on device metadata".to_string())
        })?;
        
        metadata.remove(device_id);
        
        Ok(())
    }
    
    /// Get a device by ID.
    pub fn get_device(&self, device_id: &str) -> DeviceResult<Box<dyn Device>> {
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        match devices.get(device_id) {
            Some(device) => Ok(device.clone_box()),
            None => Err(DeviceError::NotFound(format!("Device with ID {} not found", device_id))),
        }
    }
    
    /// Get all devices.
    pub fn get_all_devices(&self) -> DeviceResult<Vec<DeviceInfo>> {
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        let mut result = Vec::new();
        for device in devices.values() {
            match device.info() {
                Ok(info) => result.push(info),
                Err(e) => warn!("Failed to get device info: {}", e),
            }
        }
        
        Ok(result)
    }
    
    /// Get devices by type.
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> DeviceResult<Vec<DeviceInfo>> {
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        let mut result = Vec::new();
        for device in devices.values() {
            match device.info() {
                Ok(info) => {
                    if info.device_type == device_type {
                        result.push(info);
                    }
                },
                Err(e) => warn!("Failed to get device info: {}", e),
            }
        }
        
        Ok(result)
    }
    
    /// Get devices by capability.
    pub fn get_devices_by_capability(&self, capability: DeviceCapability) -> DeviceResult<Vec<DeviceInfo>> {
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        let mut result = Vec::new();
        for device in devices.values() {
            match device.info() {
                Ok(info) => {
                    if info.has_capability(capability.clone()) {
                        result.push(info);
                    }
                },
                Err(e) => warn!("Failed to get device info: {}", e),
            }
        }
        
        Ok(result)
    }
    
    /// Register a device discovery handler.
    pub fn register_discovery_handler(
        &self,
        device_type: DeviceType,
        handler: DeviceDiscoveryHandler,
    ) -> DeviceResult<()> {
        info!("Registering discovery handler for device type: {:?}", device_type);
        
        let mut discovery_handlers = self.discovery_handlers.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on discovery handlers".to_string())
        })?;
        
        // Get or create handlers for this device type
        let handlers = discovery_handlers.entry(device_type).or_insert_with(Vec::new);
        
        // Add handler
        handlers.push(handler);
        
        Ok(())
    }
    
    /// Discover devices using registered discovery handlers.
    pub fn discover_devices(&self) -> DeviceResult<Vec<String>> {
        info!("Discovering devices");
        
        let discovery_handlers = self.discovery_handlers.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on discovery handlers".to_string())
        })?;
        
        let mut discovered_device_ids = Vec::new();
        
        // Run all discovery handlers
        for (device_type, handlers) in discovery_handlers.iter() {
            info!("Running discovery handlers for device type: {:?}", device_type);
            
            for handler in handlers {
                match handler() {
                    Ok(devices) => {
                        for device in devices {
                            match self.register_device(device) {
                                Ok(device_id) => {
                                    discovered_device_ids.push(device_id);
                                },
                                Err(e) => {
                                    warn!("Failed to register discovered device: {}", e);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Device discovery handler failed: {}", e);
                    }
                }
            }
        }
        
        info!("Discovered {} devices", discovered_device_ids.len());
        
        Ok(discovered_device_ids)
    }
    
    /// Set device metadata.
    pub fn set_device_metadata(&self, device_id: &str, key: &str, value: &str) -> DeviceResult<()> {
        // Check if device exists
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        if !devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!("Device with ID {} not found", device_id)));
        }
        
        // Set metadata
        let mut metadata = self.device_metadata.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on device metadata".to_string())
        })?;
        
        let device_metadata = metadata.entry(device_id.to_string()).or_insert_with(HashMap::new);
        device_metadata.insert(key.to_string(), value.to_string());
        
        Ok(())
    }
    
    /// Get device metadata.
    pub fn get_device_metadata(&self, device_id: &str, key: &str) -> DeviceResult<Option<String>> {
        // Check if device exists
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        if !devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!("Device with ID {} not found", device_id)));
        }
        
        // Get metadata
        let metadata = self.device_metadata.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on device metadata".to_string())
        })?;
        
        let value = metadata
            .get(device_id)
            .and_then(|device_metadata| device_metadata.get(key))
            .map(|value| value.clone());
        
        Ok(value)
    }
    
    /// Get all device metadata.
    pub fn get_all_device_metadata(&self, device_id: &str) -> DeviceResult<HashMap<String, String>> {
        // Check if device exists
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        if !devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!("Device with ID {} not found", device_id)));
        }
        
        // Get metadata
        let metadata = self.device_metadata.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on device metadata".to_string())
        })?;
        
        let device_metadata = metadata
            .get(device_id)
            .map(|device_metadata| device_metadata.clone())
            .unwrap_or_else(HashMap::new);
        
        Ok(device_metadata)
    }
    
    /// Get the device event manager.
    pub fn get_event_manager(&self) -> Arc<DeviceEventManager> {
        Arc::clone(&self.event_manager)
    }
    
    /// Enable or disable auto-discovery.
    pub fn set_auto_discovery_enabled(&mut self, enabled: bool) {
        self.auto_discovery_enabled = enabled;
    }
    
    /// Check if auto-discovery is enabled.
    pub fn is_auto_discovery_enabled(&self) -> bool {
        self.auto_discovery_enabled
    }
    
    /// Get the number of registered devices.
    pub fn get_device_count(&self) -> DeviceResult<usize> {
        let devices = self.devices.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on devices".to_string())
        })?;
        
        Ok(devices.len())
    }
    
    /// Get the number of registered discovery handlers.
    pub fn get_discovery_handler_count(&self) -> DeviceResult<usize> {
        let discovery_handlers = self.discovery_handlers.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on discovery handlers".to_string())
        })?;
        
        let mut count = 0;
        for handlers in discovery_handlers.values() {
            count += handlers.len();
        }
        
        Ok(count)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::device::{DeviceBus, DeviceState};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // Mock device implementation for testing
    #[derive(Debug)]
    struct MockDevice {
        info: DeviceInfo,
    }
    
    impl MockDevice {
        fn new(id: &str, device_type: DeviceType) -> Self {
            let now = chrono::Utc::now();
            let info = DeviceInfo {
                id: id.to_string(),
                name: format!("Mock {}", id),
                device_type,
                manufacturer: "Mock Manufacturer".to_string(),
                model: "Mock Model".to_string(),
                serial_number: Some("12345".to_string()),
                firmware_version: Some("1.0.0".to_string()),
                driver_version: Some("1.0.0".to_string()),
                bus_type: DeviceBus::Virtual,
                bus_address: None,
                capabilities: Vec::new(),
                state: DeviceState::Ready,
                description: Some("Mock device for testing".to_string()),
                properties: HashMap::new(),
                created_at: now,
                updated_at: now,
            };
            
            Self { info }
        }
    }
    
    impl Device for MockDevice {
        fn info(&self) -> DeviceResult<DeviceInfo> {
            Ok(self.info.clone())
        }
        
        fn initialize(&mut self) -> DeviceResult<()> {
            Ok(())
        }
        
        fn shutdown(&mut self) -> DeviceResult<()> {
            Ok(())
        }
        
        fn reset(&mut self) -> DeviceResult<()> {
            Ok(())
        }
        
        fn is_connected(&self) -> DeviceResult<bool> {
            Ok(true)
        }
        
        fn state(&self) -> DeviceResult<DeviceState> {
            Ok(self.info.state)
        }
        
        fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
            self.info.state = state;
            Ok(())
        }
        
        fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
            Ok(self.info.capabilities.contains(&capability))
        }
        
        fn properties(&self) -> DeviceResult<HashMap<String, String>> {
            Ok(self.info.properties.clone())
        }
        
        fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
            Ok(self.info.properties.get(key).cloned())
        }
        
        fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
            self.info.properties.insert(key.to_string(), value.to_string());
            Ok(())
        }
        
        fn register_event_handler(&mut self, _handler: DeviceEventHandler) -> DeviceResult<()> {
            Ok(())
        }
        
        fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
            Ok(())
        }
        
        fn clone_box(&self) -> Box<dyn Device> {
            Box::new(MockDevice {
                info: self.info.clone(),
            })
        }
    }
    
    #[test]
    fn test_device_registration() {
        let manager = DeviceManager::new();
        
        // Register a device
        let device = Box::new(MockDevice::new("device1", DeviceType::Sensor));
        let device_id = manager.register_device(device).unwrap();
        
        // Check that the device was registered
        assert_eq!(device_id, "device1");
        assert_eq!(manager.get_device_count().unwrap(), 1);
        
        // Get the device
        let device = manager.get_device(&device_id).unwrap();
        let info = device.info().unwrap();
        assert_eq!(info.id, "device1");
        assert_eq!(info.device_type, DeviceType::Sensor);
        
        // Unregister the device
        manager.unregister_device(&device_id).unwrap();
        assert_eq!(manager.get_device_count().unwrap(), 0);
        
        // Check that the device is no longer registered
        assert!(manager.get_device(&device_id).is_err());
    }
    
    #[test]
    fn test_device_discovery() {
        let manager = DeviceManager::new();
        
        // Register a discovery handler
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        let handler: DeviceDiscoveryHandler = Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            
            let mut devices: Vec<Box<dyn Device>> = Vec::new();
            devices.push(Box::new(MockDevice::new("discovered1", DeviceType::Sensor)));
            devices.push(Box::new(MockDevice::new("discovered2", DeviceType::Sensor)));
            
            Ok(devices)
        });
        
        manager.register_discovery_handler(DeviceType::Sensor, handler).unwrap();
        
        // Run device discovery
        let discovered_ids = manager.discover_devices().unwrap();
        
        // Check that the handler was called
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        
        // Check that the devices were discovered and registered
        assert_eq!(discovered_ids.len(), 2);
        assert_eq!(manager.get_device_count().unwrap(), 2);
        
        // Check that we can get the discovered devices
        let device1 = manager.get_device("discovered1").unwrap();
        let device2 = manager.get_device("discovered2").unwrap();
        
        let info1 = device1.info().unwrap();
        let info2 = device2.info().unwrap();
        
        assert_eq!(info1.id, "discovered1");
        assert_eq!(info2.id, "discovered2");
    }
    
    #[test]
    fn test_device_metadata() {
        let manager = DeviceManager::new();
        
        // Register a device
        let device = Box::new(MockDevice::new("device1", DeviceType::Sensor));
        let device_id = manager.register_device(device).unwrap();
        
        // Set metadata
        manager.set_device_metadata(&device_id, "key1", "value1").unwrap();
        manager.set_device_metadata(&device_id, "key2", "value2").unwrap();
        
        // Get metadata
        let value1 = manager.get_device_metadata(&device_id, "key1").unwrap();
        let value2 = manager.get_device_metadata(&device_id, "key2").unwrap();
        let value3 = manager.get_device_metadata(&device_id, "key3").unwrap();
        
        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value2".to_string()));
        assert_eq!(value3, None);
        
        // Get all metadata
        let all_metadata = manager.get_all_device_metadata(&device_id).unwrap();
        
        assert_eq!(all_metadata.len(), 2);
        assert_eq!(all_metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(all_metadata.get("key2"), Some(&"value2".to_string()));
    }
}
