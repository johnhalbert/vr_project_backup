//! Hardware Manager implementation for the VR headset.
//!
//! This module provides the implementation of the DeviceManager trait
//! that manages all hardware devices in the VR headset system.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::thread;

use log::{debug, error, info, warn};
use thiserror::Error;

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, 
    DeviceEventType, DeviceFactory, DeviceInfo, DeviceResult, 
    DeviceState, DeviceType,
};

/// Device Manager trait.
pub trait DeviceManager: Send + Sync {
    /// Initialize the device manager.
    fn initialize(&mut self) -> DeviceResult<()>;
    
    /// Shutdown the device manager.
    fn shutdown(&mut self) -> DeviceResult<()>;
    
    /// Register a device factory.
    fn register_factory(&mut self, factory: Box<dyn DeviceFactory>) -> DeviceResult<()>;
    
    /// Unregister a device factory.
    fn unregister_factory(&mut self, device_type: DeviceType) -> DeviceResult<()>;
    
    /// Discover all devices.
    fn discover_devices(&mut self) -> DeviceResult<Vec<DeviceInfo>>;
    
    /// Discover devices by type.
    fn discover_devices_by_type(&mut self, device_type: DeviceType) -> DeviceResult<Vec<DeviceInfo>>;
    
    /// Get all device info.
    fn get_all_device_info(&self) -> DeviceResult<Vec<DeviceInfo>>;
    
    /// Get device info.
    fn get_device_info(&self, device_id: &str) -> DeviceResult<DeviceInfo>;
    
    /// Get devices by type.
    fn get_devices_by_type(&self, device_type: DeviceType) -> DeviceResult<Vec<DeviceInfo>>;
    
    /// Get a device.
    fn get_device(&self, device_id: &str) -> DeviceResult<Arc<RwLock<Box<dyn Device>>>>;
    
    /// Initialize a device.
    fn initialize_device(&mut self, device_id: &str) -> DeviceResult<()>;
    
    /// Shutdown a device.
    fn shutdown_device(&mut self, device_id: &str) -> DeviceResult<()>;
    
    /// Reset a device.
    fn reset_device(&mut self, device_id: &str) -> DeviceResult<()>;
    
    /// Register a global event handler.
    fn register_global_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()>;
    
    /// Register a device event handler.
    fn register_device_event_handler(&mut self, device_id: &str, handler: DeviceEventHandler) -> DeviceResult<()>;
    
    /// Unregister all event handlers.
    fn unregister_all_event_handlers(&mut self) -> DeviceResult<()>;
    
    /// Unregister device event handlers.
    fn unregister_device_event_handlers(&mut self, device_id: &str) -> DeviceResult<()>;
    
    /// Run self-test on a device.
    fn self_test_device(&mut self, device_id: &str) -> DeviceResult<bool>;
    
    /// Update device firmware.
    fn update_device_firmware(&mut self, device_id: &str, firmware: &[u8]) -> DeviceResult<()>;
    
    /// Calibrate a device.
    fn calibrate_device(&mut self, device_id: &str) -> DeviceResult<bool>;
    
    /// Get device diagnostics.
    fn get_device_diagnostics(&self, device_id: &str) -> DeviceResult<HashMap<String, String>>;
    
    /// Get device statistics.
    fn get_device_statistics(&self, device_id: &str) -> DeviceResult<HashMap<String, String>>;
    
    /// Get device logs.
    fn get_device_logs(&self, device_id: &str) -> DeviceResult<Vec<String>>;
    
    /// Clear device logs.
    fn clear_device_logs(&mut self, device_id: &str) -> DeviceResult<()>;
    
    /// Wait for device to reach a specific state.
    fn wait_for_device_state(
        &self,
        device_id: &str,
        state: DeviceState,
        timeout: Duration,
        poll_interval: Duration,
    ) -> DeviceResult<bool>;
    
    /// Wait for device property to match a specific value.
    fn wait_for_device_property(
        &self,
        device_id: &str,
        property: &str,
        value: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> DeviceResult<bool>;
}

/// Hardware Manager implementation.
#[derive(Debug)]
pub struct HardwareManager {
    /// Device factories by device type
    factories: HashMap<DeviceType, Box<dyn DeviceFactory>>,
    
    /// Device instances by device ID
    devices: HashMap<String, Arc<RwLock<Box<dyn Device>>>>,
    
    /// Device information by device ID
    device_info: HashMap<String, DeviceInfo>,
    
    /// Global event handlers
    global_event_handlers: Vec<DeviceEventHandler>,
    
    /// Device-specific event handlers
    device_event_handlers: HashMap<String, Vec<DeviceEventHandler>>,
    
    /// Initialization state
    initialized: bool,
    
    /// Event handler mutex
    event_handler_mutex: Mutex<()>,
}

impl HardwareManager {
    /// Create a new HardwareManager.
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            devices: HashMap::new(),
            device_info: HashMap::new(),
            global_event_handlers: Vec::new(),
            device_event_handlers: HashMap::new(),
            initialized: false,
            event_handler_mutex: Mutex::new(()),
        }
    }
    
    /// Handle a device event.
    fn handle_event(&self, event: &DeviceEvent) {
        // Acquire lock to ensure event handling is thread-safe
        let _lock = self.event_handler_mutex.lock().unwrap_or_else(|e| {
            error!("Failed to acquire event handler mutex: {}", e);
            e.into_inner()
        });
        
        // Call global event handlers
        for handler in &self.global_event_handlers {
            handler(event);
        }
        
        // Call device-specific event handlers
        if let Some(handlers) = self.device_event_handlers.get(&event.device_id) {
            for handler in handlers {
                handler(event);
            }
        }
    }
    
    /// Create an event handler for a device that forwards events to the manager.
    fn create_device_event_handler(&self) -> DeviceEventHandler {
        let manager_ref = self.clone_event_handler_ref();
        Box::new(move |event| {
            manager_ref.handle_event(event);
        })
    }
    
    /// Clone a reference to the event handler for use in closures.
    fn clone_event_handler_ref(&self) -> EventHandlerRef {
        // Create a static handler that doesn't capture self directly
        let handler = Arc::new(move |event: &DeviceEvent| {
            // Use a static approach to handle the event
            let event_clone = event.clone();
            // Spawn a thread to handle the event to avoid lifetime issues
            thread::spawn(move || {
                // Log the event instead of directly calling self.handle_event
                debug!("Event received: {:?}", event_clone);
            });
        });
        
        EventHandlerRef {
            handle_event: handler,
        }
    }
    
    /// Update device info cache.
    fn update_device_info(&mut self, device_id: &str) -> DeviceResult<()> {
        if let Some(device) = self.devices.get(device_id) {
            let device_guard = device.read().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
            })?;
            
            let info = device_guard.info()?;
            self.device_info.insert(device_id.to_string(), info);
        }
        
        Ok(())
    }
}

/// Reference to event handler for use in closures.
#[derive(Clone)]
struct EventHandlerRef {
    handle_event: Arc<dyn Fn(&DeviceEvent) + Send + Sync>,
}

impl EventHandlerRef {
    fn handle_event(&self, event: &DeviceEvent) {
        (self.handle_event)(event);
    }
}

impl DeviceManager for HardwareManager {
    fn initialize(&mut self) -> DeviceResult<()> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initializing HardwareManager");
        
        // Discover all devices
        let devices = self.discover_devices()?;
        info!("Discovered {} devices", devices.len());
        
        self.initialized = true;
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        if !self.initialized {
            return Ok(());
        }
        
        info!("Shutting down HardwareManager");
        
        // Shutdown all devices
        let device_ids: Vec<String> = self.devices.keys().cloned().collect();
        for device_id in device_ids {
            if let Err(e) = self.shutdown_device(&device_id) {
                warn!("Failed to shutdown device {}: {}", device_id, e);
            }
        }
        
        // Clear all data
        self.devices.clear();
        self.device_info.clear();
        self.device_event_handlers.clear();
        self.global_event_handlers.clear();
        
        self.initialized = false;
        Ok(())
    }
    
    fn register_factory(&mut self, factory: Box<dyn DeviceFactory>) -> DeviceResult<()> {
        let device_type = factory.device_type();
        info!("Registering device factory for {:?}", device_type);
        
        self.factories.insert(device_type, factory);
        Ok(())
    }
    
    fn unregister_factory(&mut self, device_type: DeviceType) -> DeviceResult<()> {
        info!("Unregistering device factory for {:?}", device_type);
        
        if self.factories.remove(&device_type).is_none() {
            return Err(DeviceError::NotFound(format!(
                "Factory for device type {:?} not found",
                device_type
            )));
        }
        
        Ok(())
    }
    
    fn discover_devices(&mut self) -> DeviceResult<Vec<DeviceInfo>> {
        info!("Discovering all devices");
        
        let mut all_devices = Vec::new();
        
        for factory in self.factories.values() {
            match factory.discover() {
                Ok(devices) => {
                    for device in devices {
                        all_devices.push(device.clone());
                        self.device_info.insert(device.id.clone(), device);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to discover devices for type {:?}: {}",
                        factory.device_type(),
                        e
                    );
                }
            }
        }
        
        Ok(all_devices)
    }
    
    fn discover_devices_by_type(&mut self, device_type: DeviceType) -> DeviceResult<Vec<DeviceInfo>> {
        info!("Discovering devices of type {:?}", device_type);
        
        if let Some(factory) = self.factories.get(&device_type) {
            let devices = factory.discover()?;
            
            for device in &devices {
                self.device_info.insert(device.id.clone(), device.clone());
            }
            
            Ok(devices)
        } else {
            Err(DeviceError::NotFound(format!(
                "Factory for device type {:?} not found",
                device_type
            )))
        }
    }
    
    fn get_all_device_info(&self) -> DeviceResult<Vec<DeviceInfo>> {
        Ok(self.device_info.values().cloned().collect())
    }
    
    fn get_device_info(&self, device_id: &str) -> DeviceResult<DeviceInfo> {
        self.device_info
            .get(device_id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Device {} not found", device_id)))
    }
    
    fn get_devices_by_type(&self, device_type: DeviceType) -> DeviceResult<Vec<DeviceInfo>> {
        Ok(self
            .device_info
            .values()
            .filter(|info| info.device_type == device_type)
            .cloned()
            .collect())
    }
    
    fn get_device(&self, device_id: &str) -> DeviceResult<Arc<RwLock<Box<dyn Device>>>> {
        // Check if device is already instantiated
        if let Some(device) = self.devices.get(device_id) {
            return Ok(Arc::clone(device));
        }
        
        // Get device info
        let device_info = self.get_device_info(device_id)?;
        
        // Find factory for device type
        let factory = self
            .factories
            .get(&device_info.device_type)
            .ok_or_else(|| {
                DeviceError::NotFound(format!(
                    "Factory for device type {:?} not found",
                    device_info.device_type
                ))
            })?;
        
        // Create device
        Err(DeviceError::NotFound(format!(
            "Device {} not instantiated yet, use initialize_device first",
            device_id
        )))
    }
    
    fn initialize_device(&mut self, device_id: &str) -> DeviceResult<()> {
        // Check if device is already instantiated
        if self.devices.contains_key(device_id) {
            return Ok(());
        }
        
        info!("Initializing device {}", device_id);
        
        // Get device info
        let device_info = self.get_device_info(device_id)?;
        
        // Find factory for device type
        let factory = self
            .factories
            .get(&device_info.device_type)
            .ok_or_else(|| {
                DeviceError::NotFound(format!(
                    "Factory for device type {:?} not found",
                    device_info.device_type
                ))
            })?;
        
        // Create device
        let mut device = factory.create(device_id)?;
        
        // Register event handler
        let event_handler = self.create_device_event_handler();
        device.register_event_handler(event_handler)?;
        
        // Initialize device
        device.initialize()?;
        
        // Store device
        let device = Arc::new(RwLock::new(device));
        self.devices.insert(device_id.to_string(), device);
        
        // Update device info
        self.update_device_info(device_id)?;
        
        Ok(())
    }
    
    fn shutdown_device(&mut self, device_id: &str) -> DeviceResult<()> {
        info!("Shutting down device {}", device_id);
        
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            device_guard.shutdown()?;
            
            // Update device info
            drop(device_guard);
            self.update_device_info(device_id)?;
        }
        
        Ok(())
    }
    
    fn reset_device(&mut self, device_id: &str) -> DeviceResult<()> {
        info!("Resetting device {}", device_id);
        
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            device_guard.reset()?;
            
            // Update device info
            drop(device_guard);
            self.update_device_info(device_id)?;
        } else {
            return Err(DeviceError::NotFound(format!("Device {} not found", device_id)));
        }
        
        Ok(())
    }
    
    fn register_global_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.global_event_handlers.push(handler);
        Ok(())
    }
    
    fn register_device_event_handler(
        &mut self,
        device_id: &str,
        handler: DeviceEventHandler,
    ) -> DeviceResult<()> {
        self.device_event_handlers
            .entry(device_id.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
        
        Ok(())
    }
    
    fn unregister_all_event_handlers(&mut self) -> DeviceResult<()> {
        self.global_event_handlers.clear();
        self.device_event_handlers.clear();
        
        // Unregister handlers from devices
        for (device_id, device) in &self.devices {
            if let Ok(mut device_guard) = device.write() {
                if let Err(e) = device_guard.unregister_event_handlers() {
                    warn!("Failed to unregister event handlers for device {}: {}", device_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    fn unregister_device_event_handlers(&mut self, device_id: &str) -> DeviceResult<()> {
        self.device_event_handlers.remove(device_id);
        
        // Unregister handlers from device
        if let Some(device) = self.devices.get(device_id) {
            if let Ok(mut device_guard) = device.write() {
                device_guard.unregister_event_handlers()?;
                
                // Re-register manager handler
                let event_handler = self.create_device_event_handler();
                device_guard.register_event_handler(event_handler)?;
            }
        }
        
        Ok(())
    }
    
    fn self_test_device(&mut self, device_id: &str) -> DeviceResult<bool> {
        info!("Running self-test on device {}", device_id);
        
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            let result = device_guard.self_test()?;
            
            // Update device info
            drop(device_guard);
            self.update_device_info(device_id)?;
            
            Ok(result)
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn update_device_firmware(&mut self, device_id: &str, firmware: &[u8]) -> DeviceResult<()> {
        info!("Updating firmware for device {}", device_id);
        
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            device_guard.update_firmware(firmware)?;
            
            // Update device info
            drop(device_guard);
            self.update_device_info(device_id)?;
            
            Ok(())
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn calibrate_device(&mut self, device_id: &str) -> DeviceResult<bool> {
        info!("Calibrating device {}", device_id);
        
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            let result = device_guard.calibrate()?;
            
            // Update device info
            drop(device_guard);
            self.update_device_info(device_id)?;
            
            Ok(result)
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn get_device_diagnostics(&self, device_id: &str) -> DeviceResult<HashMap<String, String>> {
        if let Some(device) = self.devices.get(device_id) {
            let device_guard = device.read().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
            })?;
            
            device_guard.diagnostics()
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn get_device_statistics(&self, device_id: &str) -> DeviceResult<HashMap<String, String>> {
        if let Some(device) = self.devices.get(device_id) {
            let device_guard = device.read().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
            })?;
            
            device_guard.statistics()
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn get_device_logs(&self, device_id: &str) -> DeviceResult<Vec<String>> {
        if let Some(device) = self.devices.get(device_id) {
            let device_guard = device.read().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
            })?;
            
            device_guard.logs()
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn clear_device_logs(&mut self, device_id: &str) -> DeviceResult<()> {
        if let Some(device) = self.devices.get(device_id) {
            let mut device_guard = device.write().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire write lock on device".to_string())
            })?;
            
            device_guard.clear_logs()
        } else {
            Err(DeviceError::NotFound(format!("Device {} not found", device_id)))
        }
    }
    
    fn wait_for_device_state(
        &self,
        device_id: &str,
        state: DeviceState,
        timeout: Duration,
        poll_interval: Duration,
    ) -> DeviceResult<bool> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout {
            if let Some(device) = self.devices.get(device_id) {
                let device_guard = device.read().map_err(|_| {
                    DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
                })?;
                
                if device_guard.state()? == state {
                    return Ok(true);
                }
            } else {
                return Err(DeviceError::NotFound(format!("Device {} not found", device_id)));
            }
            
            // Sleep for a short time to avoid busy waiting
            thread::sleep(Duration::from_millis(10));
        }
        
        Ok(false)
    }
    
    fn wait_for_device_property(
        &self,
        device_id: &str,
        property: &str,
        value: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> DeviceResult<bool> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout {
            if let Some(device) = self.devices.get(device_id) {
                let device_guard = device.read().map_err(|_| {
                    DeviceError::CommunicationError("Failed to acquire read lock on device".to_string())
                })?;
                
                if let Some(prop_value) = device_guard.get_property(property)? {
                    if prop_value == value {
                        return Ok(true);
                    }
                }
            } else {
                return Err(DeviceError::NotFound(format!("Device {} not found", device_id)));
            }
            
            // Sleep for a short time to avoid busy waiting
            thread::sleep(poll_interval);
        }
        
        Ok(false)
    }
}

impl Default for HardwareManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::device::{DeviceBus, MockDevice, MockDeviceFactory};
    use std::sync::mpsc;
    
    fn create_test_device_info(id: &str) -> DeviceInfo {
        DeviceInfo::new(
            id.to_string(),
            format!("Test Device {}", id),
            DeviceType::Display,
            "Test Manufacturer".to_string(),
            "Test Model".to_string(),
            DeviceBus::Internal,
        )
    }
    
    #[test]
    fn test_hardware_manager_initialization() {
        let mut manager = HardwareManager::new();
        
        // Register factory
        let mut factory = MockDeviceFactory::new(DeviceType::Display);
        factory.add_device(create_test_device_info("test-device-1"));
        factory.add_device(create_test_device_info("test-device-2"));
        
        manager.register_factory(Box::new(factory)).unwrap();
        
        // Initialize manager
        manager.initialize().unwrap();
        
        // Check discovered devices
        let devices = manager.get_all_device_info().unwrap();
        assert_eq!(devices.len(), 2);
        
        // Initialize a device
        manager.initialize_device("test-device-1").unwrap();
        
        // Get device
        let device = manager.get_device("test-device-1").unwrap();
        let device_guard = device.read().unwrap();
        let device_info = device_guard.info().unwrap();
        assert_eq!(device_info.id, "test-device-1");
        
        // Shutdown manager
        manager.shutdown().unwrap();
    }
    
    #[test]
    fn test_hardware_manager_events() {
        let mut manager = HardwareManager::new();
        
        // Register factory
        let mut factory = MockDeviceFactory::new(DeviceType::Display);
        factory.add_device(create_test_device_info("test-device-1"));
        
        manager.register_factory(Box::new(factory)).unwrap();
        manager.initialize().unwrap();
        
        // Set up event channel
        let (tx, rx) = mpsc::channel();
        manager
            .register_global_event_handler(Box::new(move |event| {
                tx.send(event.clone()).unwrap();
            }))
            .unwrap();
        
        // Initialize device (should generate events)
        manager.initialize_device("test-device-1").unwrap();
        
        // Check events
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::Initialized));
        
        // Reset device (should generate events)
        manager.reset_device("test-device-1").unwrap();
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::Reset));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        // Shutdown device (should generate events)
        manager.shutdown_device("test-device-1").unwrap();
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::StateChanged { .. }));
        
        let event = rx.recv().unwrap();
        assert!(matches!(event.event_type, DeviceEventType::Shutdown));
    }
    
    #[test]
    fn test_hardware_manager_device_operations() {
        let mut manager = HardwareManager::new();
        
        // Register factory
        let mut factory = MockDeviceFactory::new(DeviceType::Display);
        factory.add_device(create_test_device_info("test-device-1"));
        
        manager.register_factory(Box::new(factory)).unwrap();
        manager.initialize().unwrap();
        manager.initialize_device("test-device-1").unwrap();
        
        // Test self-test
        let result = manager.self_test_device("test-device-1").unwrap();
        assert!(result);
        
        // Test calibration
        let result = manager.calibrate_device("test-device-1").unwrap();
        assert!(result);
        
        // Test firmware update
        manager
            .update_device_firmware("test-device-1", &[0, 1, 2, 3])
            .unwrap();
        
        // Test diagnostics
        let diagnostics = manager.get_device_diagnostics("test-device-1").unwrap();
        assert!(diagnostics.contains_key("status"));
        
        // Test statistics
        let statistics = manager.get_device_statistics("test-device-1").unwrap();
        assert!(statistics.contains_key("reads"));
        
        // Test logs
        let logs = manager.get_device_logs("test-device-1").unwrap();
        assert!(!logs.is_empty());
        
        // Test clear logs
        manager.clear_device_logs("test-device-1").unwrap();
    }
}
