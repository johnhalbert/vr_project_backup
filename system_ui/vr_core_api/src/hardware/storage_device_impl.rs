//! Storage device implementation for the Hardware Access API.
//!
//! This module provides concrete implementations of storage devices for the VR headset,
//! including internal flash memory and external SD cards.

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};
use super::storage::{
    FileSystemType, PartitionInfo, StorageCapability, StorageDevice, StorageEncryption,
    StorageInfo, StoragePerformance, StorageState, StorageType,
};

/// VR Internal Flash Storage device implementation.
#[derive(Debug)]
pub struct VRInternalFlashDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Storage information
    storage_info: StorageInfo,
    
    /// Storage state
    storage_state: StorageState,
    
    /// Storage performance metrics
    performance: StoragePerformance,
    
    /// Partitions on the device
    partitions: Vec<PartitionInfo>,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRInternalFlashDevice {
    /// Create a new VRInternalFlashDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Storage,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::Internal,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::Storage,
                DeviceCapability::Encryption,
                DeviceCapability::PartitionManagement,
            ],
            state: DeviceState::Connected,
            description: Some("VR Internal Flash Storage".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add storage-specific properties
        info.properties.insert("storage_type".to_string(), "internal_flash".to_string());
        
        // Create storage information
        let storage_info = StorageInfo {
            storage_type: StorageType::InternalFlash,
            capacity_bytes: 128 * 1024 * 1024 * 1024, // 128 GB
            block_size_bytes: 4096,
            is_removable: false,
            is_readonly: false,
            encryption_support: vec![StorageEncryption::AES256],
            supported_filesystems: vec![FileSystemType::Ext4, FileSystemType::F2FS],
        };
        
        // Create storage state
        let storage_state = StorageState {
            available_bytes: 100 * 1024 * 1024 * 1024, // 100 GB available
            used_bytes: 28 * 1024 * 1024 * 1024, // 28 GB used
            is_mounted: true,
            mount_point: Some("/data".to_string()),
            filesystem_type: Some(FileSystemType::F2FS),
            encryption_status: StorageEncryption::None,
            health_status: 1.0, // 100% health
            temperature_c: Some(30.0),
        };
        
        // Create storage performance metrics
        let performance = StoragePerformance {
            read_speed_mbps: 500.0,
            write_speed_mbps: 200.0,
            iops_read: 50000,
            iops_write: 20000,
            latency_ms: 0.5,
        };
        
        // Create partitions
        let partitions = vec![
            PartitionInfo {
                id: "system".to_string(),
                label: Some("System".to_string()),
                filesystem_type: FileSystemType::Ext4,
                mount_point: Some("/system".to_string()),
                size_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
                used_bytes: 8 * 1024 * 1024 * 1024, // 8 GB used
                is_readonly: true,
                encryption: StorageEncryption::None,
            },
            PartitionInfo {
                id: "data".to_string(),
                label: Some("User Data".to_string()),
                filesystem_type: FileSystemType::F2FS,
                mount_point: Some("/data".to_string()),
                size_bytes: 118 * 1024 * 1024 * 1024, // 118 GB
                used_bytes: 20 * 1024 * 1024 * 1024, // 20 GB used
                is_readonly: false,
                encryption: StorageEncryption::None,
            },
        ];
        
        Self {
            info,
            storage_info,
            storage_state,
            performance,
            partitions,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate usage changes and temperature
        let elapsed = self.last_update.elapsed().as_secs_f32();
        
        // Simulate small writes over time
        let write_amount = (elapsed * 10.0 * 1024.0) as u64; // 10 KB/s
        self.storage_state.used_bytes += write_amount;
        self.storage_state.available_bytes -= write_amount;
        
        // Update data partition usage
        if let Some(data_partition) = self.partitions.iter_mut().find(|p| p.id == "data") {
            data_partition.used_bytes += write_amount;
        }
        
        // Simulate temperature changes
        let temp_change = (rand::random::<f32>() - 0.5) * 0.1; // Small random fluctuation
        if let Some(temp) = &mut self.storage_state.temperature_c {
            *temp += temp_change;
            *temp = temp.clamp(25.0, 45.0);
        }
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRInternalFlashDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Internal Flash: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Mount partitions (simulated)
        self.storage_state.is_mounted = true;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Internal Flash: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Unmount partitions (simulated)
        self.storage_state.is_mounted = false;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Internal Flash: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Re-initialize state (keep partitions)
        self.storage_state.is_mounted = true;
        self.storage_state.encryption_status = StorageEncryption::None;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
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
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(Self {
            info: self.info.clone(),
            storage_info: self.storage_info.clone(),
            storage_state: self.storage_state.clone(),
            performance: self.performance.clone(),
            partitions: self.partitions.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl StorageDevice for VRInternalFlashDevice {
    fn get_storage_info(&self) -> DeviceResult<StorageInfo> {
        Ok(self.storage_info.clone())
    }
    
    fn get_storage_state(&mut self) -> DeviceResult<StorageState> {
        // Update status
        self.update_status();
        
        Ok(self.storage_state.clone())
    }
    
    fn get_performance_metrics(&self) -> DeviceResult<StoragePerformance> {
        Ok(self.performance.clone())
    }
    
    fn mount(&mut self, _mount_point: Option<&str>) -> DeviceResult<()> {
        info!("Mounting VR Internal Flash: {}", self.info.id);
        
        // Check if already mounted
        if self.storage_state.is_mounted {
            return Ok(());
        }
        
        // Update state
        self.storage_state.is_mounted = true;
        self.storage_state.mount_point = Some("/data".to_string()); // Default mount point
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageMounted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("mount_point".to_string(), "/data".to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn unmount(&mut self) -> DeviceResult<()> {
        info!("Unmounting VR Internal Flash: {}", self.info.id);
        
        // Check if already unmounted
        if !self.storage_state.is_mounted {
            return Ok(());
        }
        
        // Update state
        self.storage_state.is_mounted = false;
        self.storage_state.mount_point = None;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageUnmounted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn format(&mut self, filesystem: FileSystemType, _label: Option<&str>) -> DeviceResult<()> {
        info!("Formatting VR Internal Flash: {} with {:?}", self.info.id, filesystem);
        
        // Check if supported filesystem
        if !self.storage_info.supported_filesystems.contains(&filesystem) {
            return Err(DeviceError::UnsupportedOperation(format!(
                "Filesystem {:?} not supported",
                filesystem
            )));
        }
        
        // Check if mounted
        if self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Cannot format while mounted".to_string()));
        }
        
        // Simulate format delay
        std::thread::sleep(Duration::from_secs(5));
        
        // Reset state after format
        self.storage_state.filesystem_type = Some(filesystem);
        self.storage_state.used_bytes = 0;
        self.storage_state.available_bytes = self.storage_info.capacity_bytes;
        self.storage_state.encryption_status = StorageEncryption::None;
        
        // Reset partitions (assuming format affects the whole device or main data partition)
        if let Some(data_partition) = self.partitions.iter_mut().find(|p| p.id == "data") {
            data_partition.filesystem_type = filesystem;
            data_partition.used_bytes = 0;
        }
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageFormatted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("filesystem".to_string(), format!("{:?}", filesystem));
                data
            },
        });
        
        Ok(())
    }
    
    fn read_data(&self, offset: u64, length: usize) -> DeviceResult<Vec<u8>> {
        info!("Reading {} bytes from offset {} on VR Internal Flash: {}", length, offset, self.info.id);
        
        // Check if mounted
        if !self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Device not mounted".to_string()));
        }
        
        // Check bounds
        if offset + length as u64 > self.storage_info.capacity_bytes {
            return Err(DeviceError::InvalidParameter("Read out of bounds".to_string()));
        }
        
        // Simulate read delay based on length and speed
        let delay_ms = (length as f32 / (self.performance.read_speed_mbps * 1024.0 * 1024.0)) * 1000.0;
        std::thread::sleep(Duration::from_millis(delay_ms as u64));
        
        // Return dummy data
        Ok(vec![0u8; length])
    }
    
    fn write_data(&mut self, offset: u64, data: &[u8]) -> DeviceResult<usize> {
        info!("Writing {} bytes to offset {} on VR Internal Flash: {}", data.len(), offset, self.info.id);
        
        // Check if mounted
        if !self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Device not mounted".to_string()));
        }
        
        // Check if readonly
        if self.storage_info.is_readonly {
            return Err(DeviceError::ReadOnly("Device is read-only".to_string()));
        }
        
        // Check bounds
        if offset + data.len() as u64 > self.storage_info.capacity_bytes {
            return Err(DeviceError::InvalidParameter("Write out of bounds".to_string()));
        }
        
        // Check available space
        if data.len() as u64 > self.storage_state.available_bytes {
            return Err(DeviceError::OutOfSpace("Not enough space available".to_string()));
        }
        
        // Simulate write delay based on length and speed
        let delay_ms = (data.len() as f32 / (self.performance.write_speed_mbps * 1024.0 * 1024.0)) * 1000.0;
        std::thread::sleep(Duration::from_millis(delay_ms as u64));
        
        // Update storage state
        self.storage_state.used_bytes += data.len() as u64;
        self.storage_state.available_bytes -= data.len() as u64;
        
        // Update data partition usage
        if let Some(data_partition) = self.partitions.iter_mut().find(|p| p.id == "data") {
            data_partition.used_bytes += data.len() as u64;
        }
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageDataWritten".to_string(),
            data: {
                let mut data_map = HashMap::new();
                data_map.insert("offset".to_string(), offset.to_string());
                data_map.insert("length".to_string(), data.len().to_string());
                data_map
            },
        });
        
        Ok(data.len())
    }
    
    fn flush(&mut self) -> DeviceResult<()> {
        info!("Flushing VR Internal Flash: {}", self.info.id);
        
        // Simulate flush delay
        std::thread::sleep(Duration::from_millis(10));
        
        Ok(())
    }
    
    fn get_partitions(&self) -> DeviceResult<Vec<PartitionInfo>> {
        Ok(self.partitions.clone())
    }
    
    fn create_partition(&mut self, _size_bytes: u64, _filesystem: FileSystemType, _label: Option<&str>) -> DeviceResult<PartitionInfo> {
        Err(DeviceError::UnsupportedOperation("Partition creation not supported on internal flash".to_string()))
    }
    
    fn delete_partition(&mut self, _partition_id: &str) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Partition deletion not supported on internal flash".to_string()))
    }
    
    fn resize_partition(&mut self, _partition_id: &str, _new_size_bytes: u64) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Partition resizing not supported on internal flash".to_string()))
    }
    
    fn set_encryption(&mut self, encryption: StorageEncryption, _passphrase: Option<&str>) -> DeviceResult<()> {
        info!("Setting encryption to {:?} on VR Internal Flash: {}", encryption, self.info.id);
        
        // Check if supported
        if !self.storage_info.encryption_support.contains(&encryption) && encryption != StorageEncryption::None {
            return Err(DeviceError::UnsupportedOperation(format!(
                "Encryption type {:?} not supported",
                encryption
            )));
        }
        
        // Check if mounted
        if self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Cannot change encryption while mounted".to_string()));
        }
        
        // Simulate encryption/decryption delay
        std::thread::sleep(Duration::from_secs(10));
        
        // Update state
        self.storage_state.encryption_status = encryption;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageEncryptionChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("status".to_string(), format!("{:?}", encryption));
                data
            },
        });
        
        Ok(())
    }
    
    fn unlock_encryption(&mut self, _passphrase: &str) -> DeviceResult<()> {
        info!("Unlocking encryption on VR Internal Flash: {}", self.info.id);
        
        // Check if encrypted
        if self.storage_state.encryption_status == StorageEncryption::None {
            return Err(DeviceError::InvalidState("Device is not encrypted".to_string()));
        }
        
        // Simulate unlock delay
        std::thread::sleep(Duration::from_millis(500));
        
        // Assume unlock is successful
        // In a real implementation, this would verify the passphrase
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageEncryptionUnlocked".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn lock_encryption(&mut self) -> DeviceResult<()> {
        info!("Locking encryption on VR Internal Flash: {}", self.info.id);
        
        // Check if encrypted
        if self.storage_state.encryption_status == StorageEncryption::None {
            return Err(DeviceError::InvalidState("Device is not encrypted".to_string()));
        }
        
        // Simulate lock delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageEncryptionLocked".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn has_storage_capability(&self, capability: StorageCapability) -> DeviceResult<bool> {
        match capability {
            StorageCapability::Read => Ok(true),
            StorageCapability::Write => Ok(!self.storage_info.is_readonly),
            StorageCapability::Mount => Ok(true),
            StorageCapability::Format => Ok(true),
            StorageCapability::PartitionManagement => Ok(false), // Not supported for internal flash
            StorageCapability::Encryption => Ok(!self.storage_info.encryption_support.is_empty()),
            StorageCapability::PerformanceMetrics => Ok(true),
            StorageCapability::HealthMonitoring => Ok(true),
            StorageCapability::TemperatureMonitoring => Ok(self.storage_state.temperature_c.is_some()),
        }
    }
    
    fn clone_storage_box(&self) -> Box<dyn StorageDevice> {
        Box::new(Self {
            info: self.info.clone(),
            storage_info: self.storage_info.clone(),
            storage_state: self.storage_state.clone(),
            performance: self.performance.clone(),
            partitions: self.partitions.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

/// VR SD Card Storage device implementation.
#[derive(Debug)]
pub struct VRSDCardDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Storage information
    storage_info: StorageInfo,
    
    /// Storage state
    storage_state: StorageState,
    
    /// Storage performance metrics
    performance: StoragePerformance,
    
    /// Partitions on the device
    partitions: Vec<PartitionInfo>,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRSDCardDevice {
    /// Create a new VRSDCardDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Storage,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::SDIO,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::Storage,
                DeviceCapability::PartitionManagement,
            ],
            state: DeviceState::Connected,
            description: Some("VR SD Card Storage".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add storage-specific properties
        info.properties.insert("storage_type".to_string(), "sd_card".to_string());
        
        // Create storage information
        let storage_info = StorageInfo {
            storage_type: StorageType::SDCard,
            capacity_bytes: 256 * 1024 * 1024 * 1024, // 256 GB
            block_size_bytes: 512,
            is_removable: true,
            is_readonly: false,
            encryption_support: vec![], // No hardware encryption support
            supported_filesystems: vec![FileSystemType::FAT32, FileSystemType::ExFAT, FileSystemType::Ext4],
        };
        
        // Create storage state
        let storage_state = StorageState {
            available_bytes: 250 * 1024 * 1024 * 1024, // 250 GB available
            used_bytes: 6 * 1024 * 1024 * 1024, // 6 GB used
            is_mounted: false,
            mount_point: None,
            filesystem_type: None,
            encryption_status: StorageEncryption::None,
            health_status: 1.0, // 100% health
            temperature_c: None, // SD cards typically don't report temperature
        };
        
        // Create storage performance metrics
        let performance = StoragePerformance {
            read_speed_mbps: 90.0,
            write_speed_mbps: 40.0,
            iops_read: 2000,
            iops_write: 500,
            latency_ms: 2.0,
        };
        
        // Create partitions (initially one partition covering the whole card)
        let partitions = vec![
            PartitionInfo {
                id: "sdcard1".to_string(),
                label: None,
                filesystem_type: FileSystemType::ExFAT, // Default for large SD cards
                mount_point: None,
                size_bytes: storage_info.capacity_bytes,
                used_bytes: storage_state.used_bytes,
                is_readonly: false,
                encryption: StorageEncryption::None,
            },
        ];
        
        Self {
            info,
            storage_info,
            storage_state,
            performance,
            partitions,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // SD card status doesn't change much passively
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRSDCardDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR SD Card: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR SD Card: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Unmount if mounted
        if self.storage_state.is_mounted {
            self.unmount()?;
        }
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR SD Card: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Re-initialize state
        self.storage_state.is_mounted = false;
        self.storage_state.mount_point = None;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
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
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(Self {
            info: self.info.clone(),
            storage_info: self.storage_info.clone(),
            storage_state: self.storage_state.clone(),
            performance: self.performance.clone(),
            partitions: self.partitions.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl StorageDevice for VRSDCardDevice {
    fn get_storage_info(&self) -> DeviceResult<StorageInfo> {
        Ok(self.storage_info.clone())
    }
    
    fn get_storage_state(&mut self) -> DeviceResult<StorageState> {
        // Update status
        self.update_status();
        
        Ok(self.storage_state.clone())
    }
    
    fn get_performance_metrics(&self) -> DeviceResult<StoragePerformance> {
        Ok(self.performance.clone())
    }
    
    fn mount(&mut self, mount_point: Option<&str>) -> DeviceResult<()> {
        info!("Mounting VR SD Card: {} at {:?}", self.info.id, mount_point);
        
        // Check if already mounted
        if self.storage_state.is_mounted {
            return Ok(());
        }
        
        // Determine mount point
        let mp = mount_point.unwrap_or("/mnt/sdcard").to_string();
        
        // Update state
        self.storage_state.is_mounted = true;
        self.storage_state.mount_point = Some(mp.clone());
        
        // Update partition mount point
        if let Some(partition) = self.partitions.first_mut() {
            partition.mount_point = Some(mp.clone());
        }
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageMounted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("mount_point".to_string(), mp);
                data
            },
        });
        
        Ok(())
    }
    
    fn unmount(&mut self) -> DeviceResult<()> {
        info!("Unmounting VR SD Card: {}", self.info.id);
        
        // Check if already unmounted
        if !self.storage_state.is_mounted {
            return Ok(());
        }
        
        // Update state
        self.storage_state.is_mounted = false;
        self.storage_state.mount_point = None;
        
        // Update partition mount point
        if let Some(partition) = self.partitions.first_mut() {
            partition.mount_point = None;
        }
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageUnmounted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn format(&mut self, filesystem: FileSystemType, label: Option<&str>) -> DeviceResult<()> {
        info!("Formatting VR SD Card: {} with {:?} (Label: {:?})", self.info.id, filesystem, label);
        
        // Check if supported filesystem
        if !self.storage_info.supported_filesystems.contains(&filesystem) {
            return Err(DeviceError::UnsupportedOperation(format!(
                "Filesystem {:?} not supported",
                filesystem
            )));
        }
        
        // Check if mounted
        if self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Cannot format while mounted".to_string()));
        }
        
        // Simulate format delay
        std::thread::sleep(Duration::from_secs(10));
        
        // Reset state after format
        self.storage_state.filesystem_type = Some(filesystem);
        self.storage_state.used_bytes = 0;
        self.storage_state.available_bytes = self.storage_info.capacity_bytes;
        
        // Reset partitions (assume format creates a single partition)
        self.partitions = vec![PartitionInfo {
            id: format!("{}_part1", self.info.id),
            label: label.map(|s| s.to_string()),
            filesystem_type: filesystem,
            mount_point: None,
            size_bytes: self.storage_info.capacity_bytes,
            used_bytes: 0,
            is_readonly: false,
            encryption: StorageEncryption::None,
        }];
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageFormatted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("filesystem".to_string(), format!("{:?}", filesystem));
                if let Some(l) = label { data.insert("label".to_string(), l.to_string()); }
                data
            },
        });
        
        Ok(())
    }
    
    fn read_data(&self, offset: u64, length: usize) -> DeviceResult<Vec<u8>> {
        info!("Reading {} bytes from offset {} on VR SD Card: {}", length, offset, self.info.id);
        
        // Check if mounted
        if !self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Device not mounted".to_string()));
        }
        
        // Check bounds
        if offset + length as u64 > self.storage_info.capacity_bytes {
            return Err(DeviceError::InvalidParameter("Read out of bounds".to_string()));
        }
        
        // Simulate read delay based on length and speed
        let delay_ms = (length as f32 / (self.performance.read_speed_mbps * 1024.0 * 1024.0)) * 1000.0;
        std::thread::sleep(Duration::from_millis(delay_ms as u64));
        
        // Return dummy data
        Ok(vec![0u8; length])
    }
    
    fn write_data(&mut self, offset: u64, data: &[u8]) -> DeviceResult<usize> {
        info!("Writing {} bytes to offset {} on VR SD Card: {}", data.len(), offset, self.info.id);
        
        // Check if mounted
        if !self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Device not mounted".to_string()));
        }
        
        // Check if readonly
        if self.storage_info.is_readonly {
            return Err(DeviceError::ReadOnly("Device is read-only".to_string()));
        }
        
        // Check bounds
        if offset + data.len() as u64 > self.storage_info.capacity_bytes {
            return Err(DeviceError::InvalidParameter("Write out of bounds".to_string()));
        }
        
        // Check available space
        if data.len() as u64 > self.storage_state.available_bytes {
            return Err(DeviceError::OutOfSpace("Not enough space available".to_string()));
        }
        
        // Simulate write delay based on length and speed
        let delay_ms = (data.len() as f32 / (self.performance.write_speed_mbps * 1024.0 * 1024.0)) * 1000.0;
        std::thread::sleep(Duration::from_millis(delay_ms as u64));
        
        // Update storage state
        self.storage_state.used_bytes += data.len() as u64;
        self.storage_state.available_bytes -= data.len() as u64;
        
        // Update partition usage
        if let Some(partition) = self.partitions.first_mut() {
            partition.used_bytes += data.len() as u64;
        }
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "StorageDataWritten".to_string(),
            data: {
                let mut data_map = HashMap::new();
                data_map.insert("offset".to_string(), offset.to_string());
                data_map.insert("length".to_string(), data.len().to_string());
                data_map
            },
        });
        
        Ok(data.len())
    }
    
    fn flush(&mut self) -> DeviceResult<()> {
        info!("Flushing VR SD Card: {}", self.info.id);
        
        // Simulate flush delay
        std::thread::sleep(Duration::from_millis(50));
        
        Ok(())
    }
    
    fn get_partitions(&self) -> DeviceResult<Vec<PartitionInfo>> {
        Ok(self.partitions.clone())
    }
    
    fn create_partition(&mut self, size_bytes: u64, filesystem: FileSystemType, label: Option<&str>) -> DeviceResult<PartitionInfo> {
        info!("Creating partition on VR SD Card: {} - Size: {}, FS: {:?}, Label: {:?}", 
              self.info.id, size_bytes, filesystem, label);
        
        // Check if mounted
        if self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Cannot manage partitions while mounted".to_string()));
        }
        
        // Check if supported filesystem
        if !self.storage_info.supported_filesystems.contains(&filesystem) {
            return Err(DeviceError::UnsupportedOperation(format!(
                "Filesystem {:?} not supported",
                filesystem
            )));
        }
        
        // Find available space (simplified: assumes space at the end)
        let current_total_partition_size: u64 = self.partitions.iter().map(|p| p.size_bytes).sum();
        let available_space = self.storage_info.capacity_bytes - current_total_partition_size;
        
        if size_bytes > available_space {
            return Err(DeviceError::OutOfSpace("Not enough space for new partition".to_string()));
        }
        
        // Simulate delay
        std::thread::sleep(Duration::from_secs(2));
        
        // Create new partition info
        let partition_id = format!("{}_part{}", self.info.id, self.partitions.len() + 1);
        let new_partition = PartitionInfo {
            id: partition_id.clone(),
            label: label.map(|s| s.to_string()),
            filesystem_type: filesystem,
            mount_point: None,
            size_bytes,
            used_bytes: 0,
            is_readonly: false,
            encryption: StorageEncryption::None,
        };
        
        self.partitions.push(new_partition.clone());
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "PartitionCreated".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("partition_id".to_string(), partition_id);
                data.insert("size_bytes".to_string(), size_bytes.to_string());
                data.insert("filesystem".to_string(), format!("{:?}", filesystem));
                if let Some(l) = label { data.insert("label".to_string(), l.to_string()); }
                data
            },
        });
        
        Ok(new_partition)
    }
    
    fn delete_partition(&mut self, partition_id: &str) -> DeviceResult<()> {
        info!("Deleting partition {} on VR SD Card: {}", partition_id, self.info.id);
        
        // Check if mounted
        if self.storage_state.is_mounted {
            return Err(DeviceError::InvalidState("Cannot manage partitions while mounted".to_string()));
        }
        
        // Find partition index
        let partition_index = self.partitions.iter().position(|p| p.id == partition_id);
        
        match partition_index {
            Some(index) => {
                // Simulate delay
                std::thread::sleep(Duration::from_secs(1));
                
                // Remove partition
                self.partitions.remove(index);
                
                // Update timestamp
                self.info.updated_at = chrono::Utc::now();
                
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "PartitionDeleted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("partition_id".to_string(), partition_id.to_string());
                        data
                    },
                });
                
                Ok(())
            }
            None => Err(DeviceError::NotFound(format!("Partition {} not found", partition_id))),
        }
    }
    
    fn resize_partition(&mut self, _partition_id: &str, _new_size_bytes: u64) -> DeviceResult<()> {
        // Resizing is complex to simulate accurately, mark as unsupported for now
        Err(DeviceError::UnsupportedOperation("Partition resizing not supported in this simulation".to_string()))
    }
    
    fn set_encryption(&mut self, _encryption: StorageEncryption, _passphrase: Option<&str>) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Encryption not supported on this SD card device".to_string()))
    }
    
    fn unlock_encryption(&mut self, _passphrase: &str) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Encryption not supported on this SD card device".to_string()))
    }
    
    fn lock_encryption(&mut self) -> DeviceResult<()> {
        Err(DeviceError::UnsupportedOperation("Encryption not supported on this SD card device".to_string()))
    }
    
    fn has_storage_capability(&self, capability: StorageCapability) -> DeviceResult<bool> {
        match capability {
            StorageCapability::Read => Ok(true),
            StorageCapability::Write => Ok(!self.storage_info.is_readonly),
            StorageCapability::Mount => Ok(true),
            StorageCapability::Format => Ok(true),
            StorageCapability::PartitionManagement => Ok(true),
            StorageCapability::Encryption => Ok(false), // Not supported
            StorageCapability::PerformanceMetrics => Ok(true),
            StorageCapability::HealthMonitoring => Ok(true),
            StorageCapability::TemperatureMonitoring => Ok(false), // Not supported
        }
    }
    
    fn clone_storage_box(&self) -> Box<dyn StorageDevice> {
        Box::new(Self {
            info: self.info.clone(),
            storage_info: self.storage_info.clone(),
            storage_state: self.storage_state.clone(),
            performance: self.performance.clone(),
            partitions: self.partitions.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_internal_flash_creation() {
        let flash = VRInternalFlashDevice::new(
            "flash1".to_string(),
            "VR Internal Flash".to_string(),
            "Test Manufacturer".to_string(),
            "FLASH-VR-128G".to_string(),
        );
        
        let info = flash.info().unwrap();
        assert_eq!(info.id, "flash1");
        assert_eq!(info.device_type, DeviceType::Storage);
        assert_eq!(info.bus_type, DeviceBus::Internal);
        assert_eq!(info.state, DeviceState::Connected);
        
        let storage_info = flash.get_storage_info().unwrap();
        assert_eq!(storage_info.storage_type, StorageType::InternalFlash);
        assert_eq!(storage_info.capacity_bytes, 128 * 1024 * 1024 * 1024);
        assert_eq!(storage_info.is_removable, false);
        assert!(storage_info.encryption_support.contains(&StorageEncryption::AES256));
    }
    
    #[test]
    fn test_sd_card_creation() {
        let sdcard = VRSDCardDevice::new(
            "sd1".to_string(),
            "VR SD Card".to_string(),
            "Test Manufacturer".to_string(),
            "SD-VR-256G".to_string(),
        );
        
        let info = sdcard.info().unwrap();
        assert_eq!(info.id, "sd1");
        assert_eq!(info.device_type, DeviceType::Storage);
        assert_eq!(info.bus_type, DeviceBus::SDIO);
        assert_eq!(info.state, DeviceState::Connected);
        
        let storage_info = sdcard.get_storage_info().unwrap();
        assert_eq!(storage_info.storage_type, StorageType::SDCard);
        assert_eq!(storage_info.capacity_bytes, 256 * 1024 * 1024 * 1024);
        assert_eq!(storage_info.is_removable, true);
        assert!(storage_info.encryption_support.is_empty());
    }
    
    #[test]
    fn test_internal_flash_read_write() {
        let mut flash = VRInternalFlashDevice::new(
            "flash1".to_string(),
            "VR Internal Flash".to_string(),
            "Test Manufacturer".to_string(),
            "FLASH-VR-128G".to_string(),
        );
        
        flash.initialize().unwrap();
        
        // Test write
        let data_to_write = vec![1u8; 1024];
        let bytes_written = flash.write_data(0, &data_to_write).unwrap();
        assert_eq!(bytes_written, 1024);
        
        // Test read
        let read_data = flash.read_data(0, 1024).unwrap();
        assert_eq!(read_data.len(), 1024);
        // Note: We don't check content as it's dummy data
    }
    
    #[test]
    fn test_sd_card_mount_format() {
        let mut sdcard = VRSDCardDevice::new(
            "sd1".to_string(),
            "VR SD Card".to_string(),
            "Test Manufacturer".to_string(),
            "SD-VR-256G".to_string(),
        );
        
        sdcard.initialize().unwrap();
        
        // Test mount
        assert_eq!(sdcard.get_storage_state().unwrap().is_mounted, false);
        sdcard.mount(Some("/mnt/custom_sd")).unwrap();
        let state = sdcard.get_storage_state().unwrap();
        assert_eq!(state.is_mounted, true);
        assert_eq!(state.mount_point, Some("/mnt/custom_sd".to_string()));
        
        // Test unmount
        sdcard.unmount().unwrap();
        assert_eq!(sdcard.get_storage_state().unwrap().is_mounted, false);
        
        // Test format
        sdcard.format(FileSystemType::ExFAT, Some("MySDCard")).unwrap();
        let state_after_format = sdcard.get_storage_state().unwrap();
        assert_eq!(state_after_format.filesystem_type, Some(FileSystemType::ExFAT));
        assert_eq!(state_after_format.used_bytes, 0);
        
        let partitions = sdcard.get_partitions().unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].label, Some("MySDCard".to_string()));
    }
    
    #[test]
    fn test_sd_card_partition_management() {
        let mut sdcard = VRSDCardDevice::new(
            "sd1".to_string(),
            "VR SD Card".to_string(),
            "Test Manufacturer".to_string(),
            "SD-VR-256G".to_string(),
        );
        
        sdcard.initialize().unwrap();
        
        // Create a new partition
        let new_partition_size = 10 * 1024 * 1024 * 1024; // 10 GB
        let new_partition = sdcard.create_partition(new_partition_size, FileSystemType::Ext4, Some("LinuxData")).unwrap();
        assert_eq!(new_partition.size_bytes, new_partition_size);
        assert_eq!(new_partition.filesystem_type, FileSystemType::Ext4);
        assert_eq!(new_partition.label, Some("LinuxData".to_string()));
        
        let partitions = sdcard.get_partitions().unwrap();
        assert_eq!(partitions.len(), 2);
        
        // Delete the new partition
        sdcard.delete_partition(&new_partition.id).unwrap();
        let partitions_after_delete = sdcard.get_partitions().unwrap();
        assert_eq!(partitions_after_delete.len(), 1);
    }
}
