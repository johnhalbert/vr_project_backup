//! Storage management interface for the VR headset.
//!
//! This module provides the implementation of storage devices and
//! the storage manager for capacity monitoring, file operations, and backup/restore.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};

/// Storage device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageCapability {
    /// Read operations
    Read,
    
    /// Write operations
    Write,
    
    /// Capacity monitoring
    CapacityMonitoring,
    
    /// Performance monitoring
    PerformanceMonitoring,
    
    /// Backup and restore
    BackupRestore,
    
    /// Encryption
    Encryption,
    
    /// Compression
    Compression,
    
    /// Deduplication
    Deduplication,
    
    /// Quota management
    QuotaManagement,
    
    /// Cache management
    CacheManagement,
    
    /// Custom capability
    Custom(u32),
}

/// Storage type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageType {
    /// Internal storage
    Internal,
    
    /// External storage
    External,
    
    /// Network storage
    Network,
    
    /// Cloud storage
    Cloud,
    
    /// Virtual storage
    Virtual,
    
    /// Custom storage type
    Custom(String),
}

/// Storage media type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageMediaType {
    /// Flash memory
    Flash,
    
    /// Hard disk drive
    HDD,
    
    /// Solid state drive
    SSD,
    
    /// RAM disk
    RAMDisk,
    
    /// SD card
    SDCard,
    
    /// USB drive
    USBDrive,
    
    /// Network attached storage
    NAS,
    
    /// Cloud storage
    Cloud,
    
    /// Custom media type
    Custom(String),
}

/// File system type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileSystemType {
    /// ext4
    Ext4,
    
    /// FAT32
    FAT32,
    
    /// exFAT
    ExFAT,
    
    /// NTFS
    NTFS,
    
    /// HFS+
    HFSPlus,
    
    /// APFS
    APFS,
    
    /// Btrfs
    Btrfs,
    
    /// ZFS
    ZFS,
    
    /// Custom file system
    Custom(String),
}

/// Storage information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Storage type
    pub storage_type: StorageType,
    
    /// Media type
    pub media_type: StorageMediaType,
    
    /// File system type
    pub file_system: FileSystemType,
    
    /// Total capacity in bytes
    pub total_capacity: u64,
    
    /// Mount point
    pub mount_point: PathBuf,
    
    /// Device path
    pub device_path: Option<PathBuf>,
    
    /// Serial number
    pub serial_number: Option<String>,
    
    /// Manufacturer
    pub manufacturer: Option<String>,
    
    /// Model
    pub model: Option<String>,
    
    /// Firmware version
    pub firmware_version: Option<String>,
    
    /// Is removable
    pub is_removable: bool,
    
    /// Is read-only
    pub is_read_only: bool,
    
    /// Is encrypted
    pub is_encrypted: bool,
}

impl StorageInfo {
    /// Create a new StorageInfo.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        storage_type: StorageType,
        media_type: StorageMediaType,
        file_system: FileSystemType,
        total_capacity: u64,
        mount_point: PathBuf,
        device_path: Option<PathBuf>,
        serial_number: Option<String>,
        manufacturer: Option<String>,
        model: Option<String>,
        firmware_version: Option<String>,
        is_removable: bool,
        is_read_only: bool,
        is_encrypted: bool,
    ) -> Self {
        Self {
            storage_type,
            media_type,
            file_system,
            total_capacity,
            mount_point,
            device_path,
            serial_number,
            manufacturer,
            model,
            firmware_version,
            is_removable,
            is_read_only,
            is_encrypted,
        }
    }
}

/// Storage status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageStatus {
    /// Used capacity in bytes
    pub used_capacity: u64,
    
    /// Free capacity in bytes
    pub free_capacity: u64,
    
    /// Available capacity in bytes (may differ from free due to reserved space)
    pub available_capacity: u64,
    
    /// Usage percentage (0-100)
    pub usage_percentage: f32,
    
    /// Inode usage percentage (0-100, if applicable)
    pub inode_usage_percentage: Option<f32>,
    
    /// Read speed in bytes per second
    pub read_speed: Option<u64>,
    
    /// Write speed in bytes per second
    pub write_speed: Option<u64>,
    
    /// Is mounted
    pub is_mounted: bool,
    
    /// Is healthy
    pub is_healthy: bool,
    
    /// Health status message
    pub health_status: Option<String>,
    
    /// Timestamp of the status update
    pub timestamp: SystemTime,
}

impl StorageStatus {
    /// Create a new StorageStatus.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        used_capacity: u64,
        free_capacity: u64,
        available_capacity: u64,
        usage_percentage: f32,
        inode_usage_percentage: Option<f32>,
        read_speed: Option<u64>,
        write_speed: Option<u64>,
        is_mounted: bool,
        is_healthy: bool,
        health_status: Option<String>,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            used_capacity,
            free_capacity,
            available_capacity,
            usage_percentage,
            inode_usage_percentage,
            read_speed,
            write_speed,
            is_mounted,
            is_healthy,
            health_status,
            timestamp,
        }
    }
}

/// Directory entry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectoryEntryType {
    /// File
    File,
    
    /// Directory
    Directory,
    
    /// Symbolic link
    SymbolicLink,
    
    /// Block device
    BlockDevice,
    
    /// Character device
    CharacterDevice,
    
    /// FIFO
    FIFO,
    
    /// Socket
    Socket,
    
    /// Unknown
    Unknown,
}

/// Directory entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Entry name
    pub name: String,
    
    /// Entry path
    pub path: PathBuf,
    
    /// Entry type
    pub entry_type: DirectoryEntryType,
    
    /// Size in bytes (for files)
    pub size: Option<u64>,
    
    /// Last modified time
    pub modified_time: Option<SystemTime>,
    
    /// Last accessed time
    pub accessed_time: Option<SystemTime>,
    
    /// Creation time
    pub creation_time: Option<SystemTime>,
    
    /// Is hidden
    pub is_hidden: bool,
    
    /// Is system
    pub is_system: bool,
    
    /// Is read-only
    pub is_read_only: bool,
}

impl DirectoryEntry {
    /// Create a new DirectoryEntry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        path: PathBuf,
        entry_type: DirectoryEntryType,
        size: Option<u64>,
        modified_time: Option<SystemTime>,
        accessed_time: Option<SystemTime>,
        creation_time: Option<SystemTime>,
        is_hidden: bool,
        is_system: bool,
        is_read_only: bool,
    ) -> Self {
        Self {
            name,
            path,
            entry_type,
            size,
            modified_time,
            accessed_time,
            creation_time,
            is_hidden,
            is_system,
            is_read_only,
        }
    }
}

/// Backup information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup ID
    pub id: String,
    
    /// Backup name
    pub name: String,
    
    /// Backup description
    pub description: Option<String>,
    
    /// Backup creation time
    pub creation_time: SystemTime,
    
    /// Backup size in bytes
    pub size: u64,
    
    /// Source path
    pub source_path: PathBuf,
    
    /// Backup path
    pub backup_path: PathBuf,
    
    /// Is incremental
    pub is_incremental: bool,
    
    /// Parent backup ID (for incremental backups)
    pub parent_id: Option<String>,
    
    /// Is encrypted
    pub is_encrypted: bool,
    
    /// Is compressed
    pub is_compressed: bool,
    
    /// Backup type
    pub backup_type: String,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl BackupInfo {
    /// Create a new BackupInfo.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        creation_time: SystemTime,
        size: u64,
        source_path: PathBuf,
        backup_path: PathBuf,
        is_incremental: bool,
        parent_id: Option<String>,
        is_encrypted: bool,
        is_compressed: bool,
        backup_type: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            creation_time,
            size,
            source_path,
            backup_path,
            is_incremental,
            parent_id,
            is_encrypted,
            is_compressed,
            backup_type,
            metadata,
        }
    }
}

/// Quota information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotaInfo {
    /// Quota ID
    pub id: String,
    
    /// Quota name
    pub name: String,
    
    /// Path the quota applies to
    pub path: PathBuf,
    
    /// Size limit in bytes
    pub size_limit: u64,
    
    /// File count limit
    pub file_count_limit: Option<u64>,
    
    /// Current size in bytes
    pub current_size: u64,
    
    /// Current file count
    pub current_file_count: Option<u64>,
    
    /// Is enforced
    pub is_enforced: bool,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl QuotaInfo {
    /// Create a new QuotaInfo.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        path: PathBuf,
        size_limit: u64,
        file_count_limit: Option<u64>,
        current_size: u64,
        current_file_count: Option<u64>,
        is_enforced: bool,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            name,
            path,
            size_limit,
            file_count_limit,
            current_size,
            current_file_count,
            is_enforced,
            metadata,
        }
    }
}

/// Storage device trait.
pub trait StorageDevice: Device {
    /// Get the storage information.
    fn get_storage_info(&self) -> DeviceResult<StorageInfo>;
    
    /// Get the storage status.
    fn get_storage_status(&self) -> DeviceResult<StorageStatus>;
    
    /// List directory entries.
    fn list_directory(&self, path: &Path) -> DeviceResult<Vec<DirectoryEntry>>;
    
    /// Create a directory.
    fn create_directory(&self, path: &Path) -> DeviceResult<()>;
    
    /// Remove a directory.
    fn remove_directory(&self, path: &Path, recursive: bool) -> DeviceResult<()>;
    
    /// Remove a file.
    fn remove_file(&self, path: &Path) -> DeviceResult<()>;
    
    /// Copy a file.
    fn copy_file(&self, source: &Path, destination: &Path) -> DeviceResult<()>;
    
    /// Move a file.
    fn move_file(&self, source: &Path, destination: &Path) -> DeviceResult<()>;
    
    /// Get file information.
    fn get_file_info(&self, path: &Path) -> DeviceResult<DirectoryEntry>;
    
    /// Check if a path exists.
    fn path_exists(&self, path: &Path) -> DeviceResult<bool>;
    
    /// Get the free space in bytes.
    fn get_free_space(&self) -> DeviceResult<u64>;
    
    /// Get the total space in bytes.
    fn get_total_space(&self) -> DeviceResult<u64>;
    
    /// Get the used space in bytes.
    fn get_used_space(&self) -> DeviceResult<u64>;
    
    /// Create a backup.
    fn create_backup(
        &self,
        source_path: &Path,
        backup_name: &str,
        description: Option<&str>,
        is_incremental: bool,
        parent_id: Option<&str>,
        is_encrypted: bool,
        is_compressed: bool,
        backup_type: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<BackupInfo>;
    
    /// Restore a backup.
    fn restore_backup(
        &self,
        backup_id: &str,
        destination_path: &Path,
        overwrite: bool,
    ) -> DeviceResult<()>;
    
    /// List backups.
    fn list_backups(&self) -> DeviceResult<Vec<BackupInfo>>;
    
    /// Get backup information.
    fn get_backup_info(&self, backup_id: &str) -> DeviceResult<BackupInfo>;
    
    /// Delete a backup.
    fn delete_backup(&self, backup_id: &str) -> DeviceResult<()>;
    
    /// Set a quota.
    fn set_quota(
        &self,
        path: &Path,
        name: &str,
        size_limit: u64,
        file_count_limit: Option<u64>,
        is_enforced: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<QuotaInfo>;
    
    /// Get quota information.
    fn get_quota_info(&self, quota_id: &str) -> DeviceResult<QuotaInfo>;
    
    /// List quotas.
    fn list_quotas(&self) -> DeviceResult<Vec<QuotaInfo>>;
    
    /// Remove a quota.
    fn remove_quota(&self, quota_id: &str) -> DeviceResult<()>;
    
    /// Clean cache.
    fn clean_cache(&self, path: Option<&Path>) -> DeviceResult<u64>;
    
    /// Get cache size.
    fn get_cache_size(&self, path: Option<&Path>) -> DeviceResult<u64>;
    
    /// Mount the storage device.
    fn mount(&self) -> DeviceResult<()>;
    
    /// Unmount the storage device.
    fn unmount(&self) -> DeviceResult<()>;
    
    /// Check if the storage device is mounted.
    fn is_mounted(&self) -> DeviceResult<bool>;
    
    /// Run a file system check.
    fn check_file_system(&self) -> DeviceResult<bool>;
    
    /// Repair the file system.
    fn repair_file_system(&self) -> DeviceResult<bool>;
    
    /// Clone the storage device.
    fn clone_storage_box(&self) -> Box<dyn StorageDevice>;
}

/// Storage manager for managing storage devices.
#[derive(Debug)]
pub struct StorageManager {
    /// Storage devices by ID
    devices: HashMap<String, Arc<Mutex<Box<dyn StorageDevice>>>>,
    
    /// Primary storage device ID
    primary_device_id: Option<String>,
    
    /// Backup directory
    backup_directory: PathBuf,
    
    /// Cache directory
    cache_directory: PathBuf,
    
    /// Low space threshold percentage
    low_space_threshold: f32,
    
    /// Critical space threshold percentage
    critical_space_threshold: f32,
    
    /// Backup registry
    backup_registry: HashMap<String, BackupInfo>,
    
    /// Quota registry
    quota_registry: HashMap<String, QuotaInfo>,
    
    /// Cache size limit in bytes
    cache_size_limit: u64,
    
    /// Auto-clean cache when threshold is reached
    auto_clean_cache: bool,
    
    /// Auto-clean cache threshold percentage
    auto_clean_cache_threshold: f32,
}

impl StorageManager {
    /// Create a new StorageManager.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            primary_device_id: None,
            backup_directory: PathBuf::from("/var/backups/vr_headset"),
            cache_directory: PathBuf::from("/var/cache/vr_headset"),
            low_space_threshold: 15.0,
            critical_space_threshold: 5.0,
            backup_registry: HashMap::new(),
            quota_registry: HashMap::new(),
            cache_size_limit: 1024 * 1024 * 1024, // 1 GB
            auto_clean_cache: true,
            auto_clean_cache_threshold: 80.0,
        }
    }
    
    /// Add a storage device.
    pub fn add_device(&mut self, device: Box<dyn StorageDevice>) -> DeviceResult<()> {
        let info = device.info()?;
        let id = info.id.clone();
        
        // Check if this is the first device
        if self.devices.is_empty() {
            self.primary_device_id = Some(id.clone());
        }
        
        self.devices.insert(id, Arc::new(Mutex::new(device)));
        Ok(())
    }
    
    /// Remove a storage device.
    pub fn remove_device(&mut self, device_id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!(
                "Storage device not found: {}",
                device_id
            )));
        }
        
        // If this is the primary device, clear the primary device ID
        if let Some(primary_id) = &self.primary_device_id {
            if primary_id == device_id {
                self.primary_device_id = None;
                
                // Set a new primary device if there are other devices
                if !self.devices.is_empty() {
                    let next_id = self
                        .devices
                        .keys()
                        .find(|id| *id != device_id)
                        .unwrap()
                        .clone();
                    self.primary_device_id = Some(next_id);
                }
            }
        }
        
        self.devices.remove(device_id);
        Ok(())
    }
    
    /// Get a storage device.
    pub fn get_device(&self, device_id: &str) -> DeviceResult<Arc<Mutex<Box<dyn StorageDevice>>>> {
        self.devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Storage device not found: {}", device_id)))
    }
    
    /// Get the primary storage device.
    pub fn get_primary_device(&self) -> DeviceResult<Arc<Mutex<Box<dyn StorageDevice>>>> {
        if let Some(primary_id) = &self.primary_device_id {
            self.get_device(primary_id)
        } else {
            Err(DeviceError::NotFound(
                "No primary storage device set".to_string(),
            ))
        }
    }
    
    /// Set the primary storage device.
    pub fn set_primary_device(&mut self, device_id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!(
                "Storage device not found: {}",
                device_id
            )));
        }
        
        self.primary_device_id = Some(device_id.to_string());
        Ok(())
    }
    
    /// List all storage devices.
    pub fn list_devices(&self) -> DeviceResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        
        for device in self.devices.values() {
            let device = device.lock().unwrap();
            devices.push(device.info()?);
        }
        
        Ok(devices)
    }
    
    /// Get the backup directory.
    pub fn get_backup_directory(&self) -> &Path {
        &self.backup_directory
    }
    
    /// Set the backup directory.
    pub fn set_backup_directory(&mut self, path: PathBuf) {
        self.backup_directory = path;
    }
    
    /// Get the cache directory.
    pub fn get_cache_directory(&self) -> &Path {
        &self.cache_directory
    }
    
    /// Set the cache directory.
    pub fn set_cache_directory(&mut self, path: PathBuf) {
        self.cache_directory = path;
    }
    
    /// Get the low space threshold percentage.
    pub fn get_low_space_threshold(&self) -> f32 {
        self.low_space_threshold
    }
    
    /// Set the low space threshold percentage.
    pub fn set_low_space_threshold(&mut self, threshold: f32) {
        self.low_space_threshold = threshold;
    }
    
    /// Get the critical space threshold percentage.
    pub fn get_critical_space_threshold(&self) -> f32 {
        self.critical_space_threshold
    }
    
    /// Set the critical space threshold percentage.
    pub fn set_critical_space_threshold(&mut self, threshold: f32) {
        self.critical_space_threshold = threshold;
    }
    
    /// Check if the storage is low.
    pub fn is_storage_low(&self) -> DeviceResult<bool> {
        if let Some(primary_id) = &self.primary_device_id {
            let device = self.get_device(primary_id)?;
            let device = device.lock().unwrap();
            let status = device.get_storage_status()?;
            Ok(status.usage_percentage >= (100.0 - self.low_space_threshold))
        } else {
            Err(DeviceError::NotFound(
                "No primary storage device set".to_string(),
            ))
        }
    }
    
    /// Check if the storage is critically low.
    pub fn is_storage_critical(&self) -> DeviceResult<bool> {
        if let Some(primary_id) = &self.primary_device_id {
            let device = self.get_device(primary_id)?;
            let device = device.lock().unwrap();
            let status = device.get_storage_status()?;
            Ok(status.usage_percentage >= (100.0 - self.critical_space_threshold))
        } else {
            Err(DeviceError::NotFound(
                "No primary storage device set".to_string(),
            ))
        }
    }
    
    /// Get the total storage space across all devices.
    pub fn get_total_storage_space(&self) -> DeviceResult<u64> {
        let mut total = 0;
        
        for device in self.devices.values() {
            let device = device.lock().unwrap();
            total += device.get_total_space()?;
        }
        
        Ok(total)
    }
    
    /// Get the free storage space across all devices.
    pub fn get_free_storage_space(&self) -> DeviceResult<u64> {
        let mut free = 0;
        
        for device in self.devices.values() {
            let device = device.lock().unwrap();
            free += device.get_free_space()?;
        }
        
        Ok(free)
    }
    
    /// Get the used storage space across all devices.
    pub fn get_used_storage_space(&self) -> DeviceResult<u64> {
        let mut used = 0;
        
        for device in self.devices.values() {
            let device = device.lock().unwrap();
            used += device.get_used_space()?;
        }
        
        Ok(used)
    }
    
    /// Create a backup on the primary storage device.
    pub fn create_backup(
        &mut self,
        source_path: &Path,
        backup_name: &str,
        description: Option<&str>,
        is_incremental: bool,
        parent_id: Option<&str>,
        is_encrypted: bool,
        is_compressed: bool,
        backup_type: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<BackupInfo> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        let backup_info = primary_device.create_backup(
            source_path,
            backup_name,
            description,
            is_incremental,
            parent_id,
            is_encrypted,
            is_compressed,
            backup_type,
            metadata,
        )?;
        
        // Register the backup
        self.backup_registry
            .insert(backup_info.id.clone(), backup_info.clone());
        
        Ok(backup_info)
    }
    
    /// Restore a backup from the primary storage device.
    pub fn restore_backup(
        &self,
        backup_id: &str,
        destination_path: &Path,
        overwrite: bool,
    ) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        primary_device.restore_backup(backup_id, destination_path, overwrite)
    }
    
    /// List all backups.
    pub fn list_backups(&self) -> DeviceResult<Vec<BackupInfo>> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().unwrap();
        
        primary_device.list_backups()
    }
    
    /// Get backup information.
    pub fn get_backup_info(&self, backup_id: &str) -> DeviceResult<BackupInfo> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().unwrap();
        
        primary_device.get_backup_info(backup_id)
    }
    
    /// Delete a backup.
    pub fn delete_backup(&mut self, backup_id: &str) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        primary_device.delete_backup(backup_id)?;
        
        // Remove from registry
        self.backup_registry.remove(backup_id);
        
        Ok(())
    }
    
    /// Set a quota on the primary storage device.
    pub fn set_quota(
        &mut self,
        path: &Path,
        name: &str,
        size_limit: u64,
        file_count_limit: Option<u64>,
        is_enforced: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<QuotaInfo> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        let quota_info = primary_device.set_quota(
            path,
            name,
            size_limit,
            file_count_limit,
            is_enforced,
            metadata,
        )?;
        
        // Register the quota
        self.quota_registry
            .insert(quota_info.id.clone(), quota_info.clone());
        
        Ok(quota_info)
    }
    
    /// Get quota information.
    pub fn get_quota_info(&self, quota_id: &str) -> DeviceResult<QuotaInfo> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().unwrap();
        
        primary_device.get_quota_info(quota_id)
    }
    
    /// List all quotas.
    pub fn list_quotas(&self) -> DeviceResult<Vec<QuotaInfo>> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().unwrap();
        
        primary_device.list_quotas()
    }
    
    /// Remove a quota.
    pub fn remove_quota(&mut self, quota_id: &str) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        primary_device.remove_quota(quota_id)?;
        
        // Remove from registry
        self.quota_registry.remove(quota_id);
        
        Ok(())
    }
    
    /// Clean the cache on the primary storage device.
    pub fn clean_cache(&self, path: Option<&Path>) -> DeviceResult<u64> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().unwrap();
        
        primary_device.clean_cache(path)
    }
    
    /// Get the cache size on the primary storage device.
    pub fn get_cache_size(&self, path: Option<&Path>) -> DeviceResult<u64> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().unwrap();
        
        primary_device.get_cache_size(path)
    }
    
    /// Get the cache size limit.
    pub fn get_cache_size_limit(&self) -> u64 {
        self.cache_size_limit
    }
    
    /// Set the cache size limit.
    pub fn set_cache_size_limit(&mut self, limit: u64) {
        self.cache_size_limit = limit;
    }
    
    /// Check if auto-clean cache is enabled.
    pub fn is_auto_clean_cache_enabled(&self) -> bool {
        self.auto_clean_cache
    }
    
    /// Enable or disable auto-clean cache.
    pub fn set_auto_clean_cache(&mut self, enabled: bool) {
        self.auto_clean_cache = enabled;
    }
    
    /// Get the auto-clean cache threshold percentage.
    pub fn get_auto_clean_cache_threshold(&self) -> f32 {
        self.auto_clean_cache_threshold
    }
    
    /// Set the auto-clean cache threshold percentage.
    pub fn set_auto_clean_cache_threshold(&mut self, threshold: f32) {
        self.auto_clean_cache_threshold = threshold;
    }
    
    /// Check if the cache needs cleaning.
    pub fn needs_cache_cleaning(&self) -> DeviceResult<bool> {
        if !self.auto_clean_cache {
            return Ok(false);
        }
        
        let cache_size = self.get_cache_size(None)?;
        let threshold = (self.cache_size_limit as f32 * self.auto_clean_cache_threshold / 100.0) as u64;
        
        Ok(cache_size >= threshold)
    }
    
    /// Auto-clean the cache if needed.
    pub fn auto_clean_cache_if_needed(&self) -> DeviceResult<u64> {
        if self.needs_cache_cleaning()? {
            self.clean_cache(None)
        } else {
            Ok(0)
        }
    }
    
    /// Mount all storage devices.
    pub fn mount_all(&self) -> DeviceResult<()> {
        for device in self.devices.values() {
            let mut device = device.lock().unwrap();
            if let Err(e) = device.mount() {
                warn!("Failed to mount device: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Unmount all storage devices.
    pub fn unmount_all(&self) -> DeviceResult<()> {
        for device in self.devices.values() {
            let mut device = device.lock().unwrap();
            if let Err(e) = device.unmount() {
                warn!("Failed to unmount device: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Check all file systems.
    pub fn check_all_file_systems(&self) -> DeviceResult<HashMap<String, bool>> {
        let mut results = HashMap::new();
        
        for (id, device) in &self.devices {
            let mut device = device.lock().unwrap();
            match device.check_file_system() {
                Ok(result) => {
                    results.insert(id.clone(), result);
                }
                Err(e) => {
                    warn!("Failed to check file system for device {}: {}", id, e);
                    results.insert(id.clone(), false);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Repair all file systems.
    pub fn repair_all_file_systems(&self) -> DeviceResult<HashMap<String, bool>> {
        let mut results = HashMap::new();
        
        for (id, device) in &self.devices {
            let mut device = device.lock().unwrap();
            match device.repair_file_system() {
                Ok(result) => {
                    results.insert(id.clone(), result);
                }
                Err(e) => {
                    warn!("Failed to repair file system for device {}: {}", id, e);
                    results.insert(id.clone(), false);
                }
            }
        }
        
        Ok(results)
    }
}

/// Mock storage device for testing.
#[derive(Debug, Clone)]
pub struct MockStorageDevice {
    /// Device info
    pub info: DeviceInfo,
    
    /// Device state
    pub state: DeviceState,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Event handlers
    pub event_handlers: Vec<DeviceEventHandler>,
    
    /// Storage info
    pub storage_info: StorageInfo,
    
    /// Storage status
    pub storage_status: StorageStatus,
    
    /// Directory entries
    pub directory_entries: HashMap<PathBuf, Vec<DirectoryEntry>>,
    
    /// Backups
    pub backups: HashMap<String, BackupInfo>,
    
    /// Quotas
    pub quotas: HashMap<String, QuotaInfo>,
    
    /// Cache size
    pub cache_size: u64,
    
    /// Is mounted
    pub is_mounted: bool,
}

impl MockStorageDevice {
    /// Create a new MockStorageDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
        storage_type: StorageType,
        media_type: StorageMediaType,
        file_system: FileSystemType,
        total_capacity: u64,
        mount_point: PathBuf,
    ) -> Self {
        let mut info = DeviceInfo::new(
            id,
            name,
            DeviceType::Storage,
            manufacturer.clone(),
            model.clone(),
            DeviceBus::Virtual,
        );
        
        info.state = DeviceState::Connected;
        
        let storage_info = StorageInfo::new(
            storage_type,
            media_type,
            file_system,
            total_capacity,
            mount_point,
            None,
            None,
            Some(manufacturer),
            Some(model),
            None,
            false,
            false,
            false,
        );
        
        let now = SystemTime::now();
        
        let storage_status = StorageStatus::new(
            0,
            total_capacity,
            total_capacity,
            0.0,
            None,
            None,
            None,
            true,
            true,
            None,
            now,
        );
        
        Self {
            info,
            state: DeviceState::Connected,
            properties: HashMap::new(),
            event_handlers: Vec::new(),
            storage_info,
            storage_status,
            directory_entries: HashMap::new(),
            backups: HashMap::new(),
            quotas: HashMap::new(),
            cache_size: 0,
            is_mounted: true,
        }
    }
    
    /// Fire an event.
    pub fn fire_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for MockStorageDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        self.fire_event(DeviceEventType::Initialized);
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::ShuttingDown;
        self.info.state = DeviceState::ShuttingDown;
        self.fire_event(DeviceEventType::Shutdown);
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Initializing;
        self.info.state = DeviceState::Initializing;
        self.fire_event(DeviceEventType::Reset);
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous = self.state;
        self.state = state;
        self.info.state = state;
        self.fire_event(DeviceEventType::StateChanged {
            previous,
            current: state,
        });
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.has_capability(capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.properties.get(key).cloned();
        self.properties.insert(key.to_string(), value.to_string());
        self.fire_event(DeviceEventType::PropertyChanged {
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
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl StorageDevice for MockStorageDevice {
    fn get_storage_info(&self) -> DeviceResult<StorageInfo> {
        Ok(self.storage_info.clone())
    }
    
    fn get_storage_status(&self) -> DeviceResult<StorageStatus> {
        Ok(self.storage_status.clone())
    }
    
    fn list_directory(&self, path: &Path) -> DeviceResult<Vec<DirectoryEntry>> {
        self.directory_entries
            .get(path)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Directory not found: {:?}", path)))
    }
    
    fn create_directory(&self, _path: &Path) -> DeviceResult<()> {
        Ok(())
    }
    
    fn remove_directory(&self, _path: &Path, _recursive: bool) -> DeviceResult<()> {
        Ok(())
    }
    
    fn remove_file(&self, _path: &Path) -> DeviceResult<()> {
        Ok(())
    }
    
    fn copy_file(&self, _source: &Path, _destination: &Path) -> DeviceResult<()> {
        Ok(())
    }
    
    fn move_file(&self, _source: &Path, _destination: &Path) -> DeviceResult<()> {
        Ok(())
    }
    
    fn get_file_info(&self, path: &Path) -> DeviceResult<DirectoryEntry> {
        for entries in self.directory_entries.values() {
            for entry in entries {
                if entry.path == path {
                    return Ok(entry.clone());
                }
            }
        }
        
        Err(DeviceError::NotFound(format!("File not found: {:?}", path)))
    }
    
    fn path_exists(&self, path: &Path) -> DeviceResult<bool> {
        if self.directory_entries.contains_key(path) {
            return Ok(true);
        }
        
        for entries in self.directory_entries.values() {
            for entry in entries {
                if entry.path == path {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    fn get_free_space(&self) -> DeviceResult<u64> {
        Ok(self.storage_status.free_capacity)
    }
    
    fn get_total_space(&self) -> DeviceResult<u64> {
        Ok(self.storage_info.total_capacity)
    }
    
    fn get_used_space(&self) -> DeviceResult<u64> {
        Ok(self.storage_status.used_capacity)
    }
    
    fn create_backup(
        &self,
        _source_path: &Path,
        backup_name: &str,
        description: Option<&str>,
        is_incremental: bool,
        parent_id: Option<&str>,
        is_encrypted: bool,
        is_compressed: bool,
        backup_type: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<BackupInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();
        
        let backup_info = BackupInfo::new(
            id.clone(),
            backup_name.to_string(),
            description.map(|s| s.to_string()),
            now,
            0,
            PathBuf::from("/mock/source"),
            PathBuf::from("/mock/backup"),
            is_incremental,
            parent_id.map(|s| s.to_string()),
            is_encrypted,
            is_compressed,
            backup_type.to_string(),
            metadata.unwrap_or_default(),
        );
        
        Ok(backup_info)
    }
    
    fn restore_backup(
        &self,
        _backup_id: &str,
        _destination_path: &Path,
        _overwrite: bool,
    ) -> DeviceResult<()> {
        Ok(())
    }
    
    fn list_backups(&self) -> DeviceResult<Vec<BackupInfo>> {
        Ok(self.backups.values().cloned().collect())
    }
    
    fn get_backup_info(&self, backup_id: &str) -> DeviceResult<BackupInfo> {
        self.backups
            .get(backup_id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Backup not found: {}", backup_id)))
    }
    
    fn delete_backup(&self, _backup_id: &str) -> DeviceResult<()> {
        Ok(())
    }
    
    fn set_quota(
        &self,
        path: &Path,
        name: &str,
        size_limit: u64,
        file_count_limit: Option<u64>,
        is_enforced: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> DeviceResult<QuotaInfo> {
        let id = uuid::Uuid::new_v4().to_string();
        
        let quota_info = QuotaInfo::new(
            id.clone(),
            name.to_string(),
            path.to_path_buf(),
            size_limit,
            file_count_limit,
            0,
            None,
            is_enforced,
            metadata.unwrap_or_default(),
        );
        
        Ok(quota_info)
    }
    
    fn get_quota_info(&self, quota_id: &str) -> DeviceResult<QuotaInfo> {
        self.quotas
            .get(quota_id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Quota not found: {}", quota_id)))
    }
    
    fn list_quotas(&self) -> DeviceResult<Vec<QuotaInfo>> {
        Ok(self.quotas.values().cloned().collect())
    }
    
    fn remove_quota(&self, _quota_id: &str) -> DeviceResult<()> {
        Ok(())
    }
    
    fn clean_cache(&self, _path: Option<&Path>) -> DeviceResult<u64> {
        Ok(0)
    }
    
    fn get_cache_size(&self, _path: Option<&Path>) -> DeviceResult<u64> {
        Ok(self.cache_size)
    }
    
    fn mount(&self) -> DeviceResult<()> {
        Ok(())
    }
    
    fn unmount(&self) -> DeviceResult<()> {
        Ok(())
    }
    
    fn is_mounted(&self) -> DeviceResult<bool> {
        Ok(self.is_mounted)
    }
    
    fn check_file_system(&self) -> DeviceResult<bool> {
        Ok(true)
    }
    
    fn repair_file_system(&self) -> DeviceResult<bool> {
        Ok(true)
    }
    
    fn clone_storage_box(&self) -> Box<dyn StorageDevice> {
        Box::new(self.clone())
    }
}
