//! Storage optimization module for the VR headset system.
//!
//! This module provides storage optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages I/O scheduling, caching,
//! and other storage-related optimizations to maximize performance for VR workloads.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Result, Context, anyhow, bail};
use log::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::optimization::OptimizationSettings;

/// Storage optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct StorageOptimizationManager {
    /// Storage optimization settings
    settings: StorageOptimizationSettings,
    
    /// Storage information
    info: StorageInfo,
    
    /// Current storage optimization state
    state: StorageOptimizationState,
    
    /// Last optimization time
    last_optimization_time: Instant,
    
    /// Whether optimization is currently running
    is_running: bool,
    
    /// Background optimization thread handle
    background_thread: Option<thread::JoinHandle<()>>,
    
    /// Shared state for background thread
    shared_state: Arc<Mutex<SharedState>>,
}

/// Shared state for background optimization thread.
#[derive(Debug)]
struct SharedState {
    /// Whether the background thread should stop
    should_stop: bool,
    
    /// Current storage optimization settings
    settings: StorageOptimizationSettings,
}

/// Storage optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimizationSettings {
    /// Whether storage optimization is enabled
    pub enabled: bool,
    
    /// I/O scheduler to use
    pub io_scheduler: IoScheduler,
    
    /// Whether to optimize read-ahead
    pub optimize_readahead: bool,
    
    /// Read-ahead size (in KB)
    pub readahead_kb: u32,
    
    /// Whether to optimize I/O priority
    pub optimize_io_priority: bool,
    
    /// Whether to optimize file system cache
    pub optimize_fs_cache: bool,
    
    /// Whether to optimize file system mount options
    pub optimize_mount_options: bool,
    
    /// Whether to optimize file system trim
    pub optimize_trim: bool,
    
    /// Whether to optimize file system journaling
    pub optimize_journaling: bool,
    
    /// Whether to optimize file system compression
    pub optimize_compression: bool,
    
    /// Whether to optimize file system defragmentation
    pub optimize_defragmentation: bool,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// I/O scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoScheduler {
    /// Completely Fair Queuing scheduler
    Cfq,
    
    /// Deadline scheduler
    Deadline,
    
    /// NOOP scheduler
    Noop,
    
    /// Budget Fair Queuing scheduler
    Bfq,
    
    /// Kyber scheduler
    Kyber,
    
    /// Multi-Queue scheduler
    Mq,
}

impl IoScheduler {
    /// Convert I/O scheduler to string.
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Cfq => "cfq",
            Self::Deadline => "deadline",
            Self::Noop => "noop",
            Self::Bfq => "bfq",
            Self::Kyber => "kyber",
            Self::Mq => "mq-deadline",
        }
    }
    
    /// Parse I/O scheduler from string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "cfq" => Ok(Self::Cfq),
            "deadline" => Ok(Self::Deadline),
            "noop" => Ok(Self::Noop),
            "bfq" => Ok(Self::Bfq),
            "kyber" => Ok(Self::Kyber),
            "mq-deadline" => Ok(Self::Mq),
            _ => bail!("Unknown I/O scheduler: {}", s),
        }
    }
}

/// Storage information.
#[derive(Debug, Clone)]
pub struct StorageInfo {
    /// Storage devices
    pub devices: Vec<StorageDevice>,
    
    /// File systems
    pub filesystems: Vec<FileSystem>,
    
    /// Available I/O schedulers
    pub available_schedulers: Vec<IoScheduler>,
    
    /// Whether I/O priority is supported
    pub supports_io_priority: bool,
    
    /// Whether file system trim is supported
    pub supports_trim: bool,
    
    /// Whether file system compression is supported
    pub supports_compression: bool,
    
    /// Whether file system defragmentation is supported
    pub supports_defragmentation: bool,
}

/// Storage device information.
#[derive(Debug, Clone)]
pub struct StorageDevice {
    /// Device name
    pub name: String,
    
    /// Device path
    pub path: PathBuf,
    
    /// Device type
    pub device_type: StorageDeviceType,
    
    /// Device size (in MB)
    pub size: u64,
    
    /// Current I/O scheduler
    pub scheduler: IoScheduler,
    
    /// Available I/O schedulers
    pub available_schedulers: Vec<IoScheduler>,
    
    /// Current read-ahead size (in KB)
    pub readahead_kb: u32,
    
    /// Whether device supports trim
    pub supports_trim: bool,
}

/// Storage device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageDeviceType {
    /// Hard disk drive
    Hdd,
    
    /// Solid state drive
    Ssd,
    
    /// eMMC storage
    Emmc,
    
    /// SD card
    Sd,
    
    /// NVMe storage
    Nvme,
    
    /// Unknown storage type
    Unknown,
}

/// File system information.
#[derive(Debug, Clone)]
pub struct FileSystem {
    /// Mount point
    pub mount_point: PathBuf,
    
    /// Device path
    pub device: PathBuf,
    
    /// File system type
    pub fs_type: String,
    
    /// Mount options
    pub mount_options: Vec<String>,
    
    /// Total size (in MB)
    pub total_size: u64,
    
    /// Free size (in MB)
    pub free_size: u64,
    
    /// Whether file system supports trim
    pub supports_trim: bool,
    
    /// Whether file system supports compression
    pub supports_compression: bool,
    
    /// Whether file system supports defragmentation
    pub supports_defragmentation: bool,
}

/// Storage optimization state.
#[derive(Debug, Clone)]
pub struct StorageOptimizationState {
    /// Current I/O scheduler for each device
    pub io_schedulers: Vec<(String, IoScheduler)>,
    
    /// Current read-ahead size for each device (in KB)
    pub readahead_sizes: Vec<(String, u32)>,
    
    /// Current I/O statistics for each device
    pub io_stats: Vec<(String, IoStats)>,
    
    /// Current file system cache statistics
    pub fs_cache_stats: FsCacheStats,
}

/// I/O statistics.
#[derive(Debug, Clone)]
pub struct IoStats {
    /// Number of read I/Os processed
    pub reads_completed: u64,
    
    /// Number of read I/Os merged with in-queue I/O
    pub reads_merged: u64,
    
    /// Number of sectors read
    pub sectors_read: u64,
    
    /// Time spent reading (in milliseconds)
    pub read_time_ms: u64,
    
    /// Number of write I/Os processed
    pub writes_completed: u64,
    
    /// Number of write I/Os merged with in-queue I/O
    pub writes_merged: u64,
    
    /// Number of sectors written
    pub sectors_written: u64,
    
    /// Time spent writing (in milliseconds)
    pub write_time_ms: u64,
    
    /// Number of I/Os currently in progress
    pub ios_in_progress: u64,
    
    /// Time spent doing I/Os (in milliseconds)
    pub io_time_ms: u64,
    
    /// Weighted time spent doing I/Os (in milliseconds)
    pub weighted_io_time_ms: u64,
}

/// File system cache statistics.
#[derive(Debug, Clone)]
pub struct FsCacheStats {
    /// Total cache size (in MB)
    pub total_cache_size: u32,
    
    /// Dirty cache size (in MB)
    pub dirty_cache_size: u32,
    
    /// Writeback cache size (in MB)
    pub writeback_cache_size: u32,
}

impl Default for StorageOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            io_scheduler: IoScheduler::Bfq,
            optimize_readahead: true,
            readahead_kb: 512,
            optimize_io_priority: true,
            optimize_fs_cache: true,
            optimize_mount_options: true,
            optimize_trim: true,
            optimize_journaling: true,
            optimize_compression: false,
            optimize_defragmentation: true,
            adaptive: true,
            adaptive_interval_ms: 5000,
        }
    }
}

impl StorageOptimizationManager {
    /// Create a new storage optimization manager.
    pub fn new() -> Result<Self> {
        let info = Self::detect_storage_info()?;
        let settings = StorageOptimizationSettings::default();
        let state = Self::get_current_state(&info)?;
        let shared_state = Arc::new(Mutex::new(SharedState {
            should_stop: false,
            settings: settings.clone(),
        }));
        
        Ok(Self {
            settings,
            info,
            state,
            last_optimization_time: Instant::now(),
            is_running: false,
            background_thread: None,
            shared_state,
        })
    }
    
    /// Initialize storage optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing storage optimization for Orange Pi CM5");
        
        // Detect storage information
        self.info = Self::detect_storage_info()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("Storage optimization initialized successfully");
        Ok(())
    }
    
    /// Apply storage optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying storage optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply I/O scheduler optimization
        self.apply_io_scheduler_optimization()?;
        
        // Apply read-ahead optimization if enabled
        if self.settings.optimize_readahead {
            self.apply_readahead_optimization()?;
        }
        
        // Apply I/O priority optimization if enabled
        if self.settings.optimize_io_priority {
            self.apply_io_priority_optimization()?;
        }
        
        // Apply file system cache optimization if enabled
        if self.settings.optimize_fs_cache {
            self.apply_fs_cache_optimization()?;
        }
        
        // Apply mount options optimization if enabled
        if self.settings.optimize_mount_options {
            self.apply_mount_options_optimization()?;
        }
        
        // Apply trim optimization if enabled
        if self.settings.optimize_trim {
            self.apply_trim_optimization()?;
        }
        
        // Apply journaling optimization if enabled
        if self.settings.optimize_journaling {
            self.apply_journaling_optimization()?;
        }
        
        // Apply compression optimization if enabled
        if self.settings.optimize_compression {
            self.apply_compression_optimization()?;
        }
        
        // Apply defragmentation optimization if enabled
        if self.settings.optimize_defragmentation {
            self.apply_defragmentation_optimization()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("Storage optimizations applied successfully");
        Ok(())
    }
    
    /// Reset storage optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting storage optimizations");
        
        // Reset I/O scheduler
        for device in &self.info.devices {
            self.set_io_scheduler(&device.path, IoScheduler::Cfq)?;
        }
        
        // Reset read-ahead
        for device in &self.info.devices {
            self.set_readahead(&device.path, 128)?;
        }
        
        // Reset file system cache
        self.reset_fs_cache()?;
        
        // Reset mount options
        self.reset_mount_options()?;
        
        info!("Storage optimizations reset successfully");
        Ok(())
    }
    
    /// Update storage optimization settings.
    pub fn update_settings(&mut self, settings: StorageOptimizationSettings) -> Result<()> {
        info!("Updating storage optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("Storage optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current storage optimization settings.
    pub fn get_settings(&self) -> StorageOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current storage optimization state.
    pub fn get_state(&self) -> StorageOptimizationState {
        self.state.clone()
    }
    
    /// Get storage information.
    pub fn get_info(&self) -> StorageInfo {
        self.info.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background storage optimization thread");
        
        let shared_state = self.shared_state.clone();
        let info = self.info.clone();
        
        let thread = thread::spawn(move || {
            let mut last_optimization_time = Instant::now();
            
            loop {
                // Check if thread should stop
                {
                    let state = shared_state.lock().unwrap();
                    if state.should_stop {
                        break;
                    }
                }
                
                // Get current time
                let now = Instant::now();
                
                // Check if it's time to optimize
                let settings = {
                    let state = shared_state.lock().unwrap();
                    state.settings.clone()
                };
                
                if now.duration_since(last_optimization_time) >= Duration::from_millis(settings.adaptive_interval_ms) {
                    // Perform adaptive optimization
                    if let Err(e) = Self::perform_adaptive_optimization(&info, &settings) {
                        error!("Error performing adaptive optimization: {}", e);
                    }
                    
                    last_optimization_time = now;
                }
                
                // Sleep for a short time
                thread::sleep(Duration::from_millis(100));
            }
        });
        
        self.background_thread = Some(thread);
        self.is_running = true;
        
        info!("Background storage optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background storage optimization thread");
        
        // Signal thread to stop
        {
            let mut state = self.shared_state.lock().unwrap();
            state.should_stop = true;
        }
        
        // Wait for thread to finish
        if let Some(thread) = self.background_thread.take() {
            if let Err(e) = thread.join() {
                error!("Error joining background thread: {:?}", e);
            }
        }
        
        self.is_running = false;
        
        info!("Background storage optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive optimization.
    fn perform_adaptive_optimization(info: &StorageInfo, settings: &StorageOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive storage optimization");
        
        // Get current storage state
        let state = Self::get_current_state(info)?;
        
        // Check I/O statistics and adjust scheduler if necessary
        for (device_name, io_stats) in &state.io_stats {
            // Find device
            if let Some(device) = info.devices.iter().find(|d| d.name == *device_name) {
                // Calculate I/O load
                let io_load = io_stats.ios_in_progress as f32;
                
                // Adjust scheduler based on I/O load
                let scheduler = if io_load > 10.0 {
                    // High I/O load, use deadline scheduler for better throughput
                    IoScheduler::Deadline
                } else if io_load > 5.0 {
                    // Medium I/O load, use BFQ scheduler for balance
                    IoScheduler::Bfq
                } else {
                    // Low I/O load, use CFQ scheduler for fairness
                    IoScheduler::Cfq
                };
                
                // Check if scheduler needs to be changed
                if let Some((_, current_scheduler)) = state.io_schedulers.iter().find(|(name, _)| name == device_name) {
                    if *current_scheduler != scheduler && device.available_schedulers.contains(&scheduler) {
                        if let Err(e) = Self::set_io_scheduler_static(&device.path, scheduler) {
                            warn!("Error setting I/O scheduler for {}: {}", device_name, e);
                        }
                    }
                }
                
                // Adjust read-ahead based on I/O patterns
                let read_ratio = if io_stats.reads_completed + io_stats.writes_completed > 0 {
                    io_stats.reads_completed as f32 / (io_stats.reads_completed + io_stats.writes_completed) as f32
                } else {
                    0.5
                };
                
                let readahead = if read_ratio > 0.8 {
                    // Mostly reads, use larger read-ahead
                    1024
                } else if read_ratio > 0.5 {
                    // Balanced reads/writes, use medium read-ahead
                    512
                } else {
                    // Mostly writes, use smaller read-ahead
                    256
                };
                
                // Check if read-ahead needs to be changed
                if let Some((_, current_readahead)) = state.readahead_sizes.iter().find(|(name, _)| name == device_name) {
                    if *current_readahead != readahead {
                        if let Err(e) = Self::set_readahead_static(&device.path, readahead) {
                            warn!("Error setting read-ahead for {}: {}", device_name, e);
                        }
                    }
                }
            }
        }
        
        debug!("Adaptive storage optimization completed");
        Ok(())
    }
    
    /// Update settings based on global optimization settings.
    fn update_settings_from_global(&mut self, global_settings: &super::OptimizationSettings) {
        // Update enabled state
        self.settings.enabled = global_settings.enabled;
        
        // Update adaptive state
        self.settings.adaptive = global_settings.adaptive;
        
        // Update settings based on optimization mode
        match global_settings.mode {
            super::OptimizationMode::Performance => {
                self.settings.io_scheduler = IoScheduler::Deadline;
                self.settings.optimize_readahead = true;
                self.settings.readahead_kb = 1024;
                self.settings.optimize_io_priority = true;
                self.settings.optimize_fs_cache = true;
                self.settings.optimize_mount_options = true;
                self.settings.optimize_trim = true;
                self.settings.optimize_journaling = true;
                self.settings.optimize_compression = false;
                self.settings.optimize_defragmentation = true;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.io_scheduler = IoScheduler::Bfq;
                self.settings.optimize_readahead = true;
                self.settings.readahead_kb = 256;
                self.settings.optimize_io_priority = true;
                self.settings.optimize_fs_cache = true;
                self.settings.optimize_mount_options = true;
                self.settings.optimize_trim = true;
                self.settings.optimize_journaling = true;
                self.settings.optimize_compression = true;
                self.settings.optimize_defragmentation = false;
            },
            super::OptimizationMode::Latency => {
                self.settings.io_scheduler = IoScheduler::Deadline;
                self.settings.optimize_readahead = true;
                self.settings.readahead_kb = 128;
                self.settings.optimize_io_priority = true;
                self.settings.optimize_fs_cache = true;
                self.settings.optimize_mount_options = true;
                self.settings.optimize_trim = true;
                self.settings.optimize_journaling = false;
                self.settings.optimize_compression = false;
                self.settings.optimize_defragmentation = false;
            },
            super::OptimizationMode::Thermal => {
                self.settings.io_scheduler = IoScheduler::Cfq;
                self.settings.optimize_readahead = true;
                self.settings.readahead_kb = 512;
                self.settings.optimize_io_priority = true;
                self.settings.optimize_fs_cache = true;
                self.settings.optimize_mount_options = true;
                self.settings.optimize_trim = true;
                self.settings.optimize_journaling = true;
                self.settings.optimize_compression = true;
                self.settings.optimize_defragmentation = false;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.io_scheduler = IoScheduler::Deadline;
            self.settings.optimize_readahead = true;
            self.settings.readahead_kb = 2048;
            self.settings.optimize_io_priority = true;
            self.settings.optimize_fs_cache = true;
            self.settings.optimize_mount_options = true;
            self.settings.optimize_trim = true;
            self.settings.optimize_journaling = false;
            self.settings.optimize_compression = false;
            self.settings.optimize_defragmentation = true;
        }
    }
    
    /// Apply I/O scheduler optimization.
    fn apply_io_scheduler_optimization(&self) -> Result<()> {
        info!("Applying I/O scheduler optimization");
        
        for device in &self.info.devices {
            // Skip devices that don't support the selected scheduler
            if !device.available_schedulers.contains(&self.settings.io_scheduler) {
                warn!("Device {} does not support {} scheduler", device.name, self.settings.io_scheduler.to_str());
                continue;
            }
            
            // Set I/O scheduler
            self.set_io_scheduler(&device.path, self.settings.io_scheduler)?;
        }
        
        Ok(())
    }
    
    /// Apply read-ahead optimization.
    fn apply_readahead_optimization(&self) -> Result<()> {
        info!("Applying read-ahead optimization");
        
        for device in &self.info.devices {
            // Set read-ahead size
            self.set_readahead(&device.path, self.settings.readahead_kb)?;
        }
        
        Ok(())
    }
    
    /// Apply I/O priority optimization.
    fn apply_io_priority_optimization(&self) -> Result<()> {
        info!("Applying I/O priority optimization");
        
        // Find VR processes
        let vr_processes = self.find_vr_processes()?;
        
        for pid in vr_processes {
            // Set I/O priority using ionice
            let output = Command::new("ionice")
                .arg("-c")
                .arg("1") // Real-time class
                .arg("-n")
                .arg("0") // Highest priority
                .arg("-p")
                .arg(pid.to_string())
                .output()
                .context("Failed to execute ionice command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set I/O priority for process {}: {}", pid, error);
            }
        }
        
        // Set I/O priority for system processes
        let output = Command::new("ionice")
            .arg("-c")
            .arg("2") // Best-effort class
            .arg("-n")
            .arg("7") // Lowest priority
            .arg("-p")
            .arg("1") // Init process
            .output()
            .context("Failed to execute ionice command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set I/O priority for system processes: {}", error);
        }
        
        Ok(())
    }
    
    /// Apply file system cache optimization.
    fn apply_fs_cache_optimization(&self) -> Result<()> {
        info!("Applying file system cache optimization");
        
        // Set dirty ratio
        let dirty_ratio_path = Path::new("/proc/sys/vm/dirty_ratio");
        if dirty_ratio_path.exists() {
            fs::write(dirty_ratio_path, "20")
                .context("Failed to set dirty ratio")?;
        }
        
        // Set dirty background ratio
        let dirty_background_ratio_path = Path::new("/proc/sys/vm/dirty_background_ratio");
        if dirty_background_ratio_path.exists() {
            fs::write(dirty_background_ratio_path, "10")
                .context("Failed to set dirty background ratio")?;
        }
        
        // Set dirty expire centisecs
        let dirty_expire_path = Path::new("/proc/sys/vm/dirty_expire_centisecs");
        if dirty_expire_path.exists() {
            fs::write(dirty_expire_path, "3000") // 30 seconds
                .context("Failed to set dirty expire centisecs")?;
        }
        
        // Set dirty writeback centisecs
        let dirty_writeback_path = Path::new("/proc/sys/vm/dirty_writeback_centisecs");
        if dirty_writeback_path.exists() {
            fs::write(dirty_writeback_path, "500") // 5 seconds
                .context("Failed to set dirty writeback centisecs")?;
        }
        
        // Set VFS cache pressure
        let vfs_cache_pressure_path = Path::new("/proc/sys/vm/vfs_cache_pressure");
        if vfs_cache_pressure_path.exists() {
            fs::write(vfs_cache_pressure_path, "50")
                .context("Failed to set VFS cache pressure")?;
        }
        
        // Set page cache limit
        let page_cache_limit_path = Path::new("/proc/sys/vm/pagecache_limit_mb");
        if page_cache_limit_path.exists() {
            // Set page cache limit to 25% of total memory
            let meminfo = fs::read_to_string("/proc/meminfo")
                .context("Failed to read memory information")?;
            
            let mut total_memory = 0;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(mem) = parts[1].parse::<u64>() {
                            total_memory = (mem / 1024) as u32; // Convert to MB
                            break;
                        }
                    }
                }
            }
            
            if total_memory > 0 {
                let page_cache_limit = total_memory / 4;
                fs::write(page_cache_limit_path, page_cache_limit.to_string())
                    .context("Failed to set page cache limit")?;
            }
        }
        
        Ok(())
    }
    
    /// Reset file system cache.
    fn reset_fs_cache(&self) -> Result<()> {
        info!("Resetting file system cache");
        
        // Reset dirty ratio
        let dirty_ratio_path = Path::new("/proc/sys/vm/dirty_ratio");
        if dirty_ratio_path.exists() {
            fs::write(dirty_ratio_path, "20")
                .context("Failed to reset dirty ratio")?;
        }
        
        // Reset dirty background ratio
        let dirty_background_ratio_path = Path::new("/proc/sys/vm/dirty_background_ratio");
        if dirty_background_ratio_path.exists() {
            fs::write(dirty_background_ratio_path, "10")
                .context("Failed to reset dirty background ratio")?;
        }
        
        // Reset dirty expire centisecs
        let dirty_expire_path = Path::new("/proc/sys/vm/dirty_expire_centisecs");
        if dirty_expire_path.exists() {
            fs::write(dirty_expire_path, "3000") // 30 seconds
                .context("Failed to reset dirty expire centisecs")?;
        }
        
        // Reset dirty writeback centisecs
        let dirty_writeback_path = Path::new("/proc/sys/vm/dirty_writeback_centisecs");
        if dirty_writeback_path.exists() {
            fs::write(dirty_writeback_path, "500") // 5 seconds
                .context("Failed to reset dirty writeback centisecs")?;
        }
        
        // Reset VFS cache pressure
        let vfs_cache_pressure_path = Path::new("/proc/sys/vm/vfs_cache_pressure");
        if vfs_cache_pressure_path.exists() {
            fs::write(vfs_cache_pressure_path, "100")
                .context("Failed to reset VFS cache pressure")?;
        }
        
        // Reset page cache limit
        let page_cache_limit_path = Path::new("/proc/sys/vm/pagecache_limit_mb");
        if page_cache_limit_path.exists() {
            fs::write(page_cache_limit_path, "0") // No limit
                .context("Failed to reset page cache limit")?;
        }
        
        Ok(())
    }
    
    /// Apply mount options optimization.
    fn apply_mount_options_optimization(&self) -> Result<()> {
        info!("Applying mount options optimization");
        
        for filesystem in &self.info.filesystems {
            // Skip special file systems
            if filesystem.fs_type == "proc" || filesystem.fs_type == "sysfs" || filesystem.fs_type == "devtmpfs" {
                continue;
            }
            
            // Get current mount options
            let mut options = filesystem.mount_options.clone();
            
            // Add or update mount options
            let mut has_noatime = false;
            let mut has_nodiratime = false;
            let mut has_data_mode = false;
            
            for option in &options {
                if option == "noatime" {
                    has_noatime = true;
                } else if option == "nodiratime" {
                    has_nodiratime = true;
                } else if option.starts_with("data=") {
                    has_data_mode = true;
                }
            }
            
            if !has_noatime {
                options.push("noatime".to_string());
            }
            
            if !has_nodiratime {
                options.push("nodiratime".to_string());
            }
            
            if !has_data_mode && filesystem.fs_type == "ext4" {
                options.push("data=writeback".to_string());
            }
            
            // Remount file system with new options
            let options_str = options.join(",");
            let output = Command::new("mount")
                .arg("-o")
                .arg(format!("remount,{}", options_str))
                .arg(&filesystem.mount_point)
                .output()
                .context("Failed to execute mount command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to remount {} with options {}: {}", filesystem.mount_point.display(), options_str, error);
            }
        }
        
        Ok(())
    }
    
    /// Reset mount options.
    fn reset_mount_options(&self) -> Result<()> {
        info!("Resetting mount options");
        
        for filesystem in &self.info.filesystems {
            // Skip special file systems
            if filesystem.fs_type == "proc" || filesystem.fs_type == "sysfs" || filesystem.fs_type == "devtmpfs" {
                continue;
            }
            
            // Get current mount options
            let mut options = filesystem.mount_options.clone();
            
            // Remove optimized mount options
            options.retain(|option| option != "noatime" && option != "nodiratime" && !option.starts_with("data="));
            
            // Add default options
            if filesystem.fs_type == "ext4" {
                options.push("data=ordered".to_string());
            }
            
            // Remount file system with default options
            let options_str = options.join(",");
            let output = Command::new("mount")
                .arg("-o")
                .arg(format!("remount,{}", options_str))
                .arg(&filesystem.mount_point)
                .output()
                .context("Failed to execute mount command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to remount {} with options {}: {}", filesystem.mount_point.display(), options_str, error);
            }
        }
        
        Ok(())
    }
    
    /// Apply trim optimization.
    fn apply_trim_optimization(&self) -> Result<()> {
        info!("Applying trim optimization");
        
        // Check if any device supports trim
        let has_trim_support = self.info.devices.iter().any(|d| d.supports_trim);
        
        if has_trim_support {
            // Enable fstrim timer
            let output = Command::new("systemctl")
                .arg("enable")
                .arg("fstrim.timer")
                .output()
                .context("Failed to execute systemctl command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to enable fstrim timer: {}", error);
            }
            
            // Start fstrim timer
            let output = Command::new("systemctl")
                .arg("start")
                .arg("fstrim.timer")
                .output()
                .context("Failed to execute systemctl command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to start fstrim timer: {}", error);
            }
            
            // Run fstrim once
            let output = Command::new("fstrim")
                .arg("-av")
                .output()
                .context("Failed to execute fstrim command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to run fstrim: {}", error);
            }
        } else {
            warn!("No devices support trim");
        }
        
        Ok(())
    }
    
    /// Apply journaling optimization.
    fn apply_journaling_optimization(&self) -> Result<()> {
        info!("Applying journaling optimization");
        
        for filesystem in &self.info.filesystems {
            // Skip non-ext4 file systems
            if filesystem.fs_type != "ext4" {
                continue;
            }
            
            // Set journal commit interval
            let output = Command::new("tune2fs")
                .arg("-o")
                .arg("journal_data_writeback")
                .arg("-J")
                .arg("journal_commit_time=5")
                .arg(filesystem.device.to_str().unwrap_or(""))
                .output()
                .context("Failed to execute tune2fs command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set journal options for {}: {}", filesystem.device.display(), error);
            }
        }
        
        Ok(())
    }
    
    /// Apply compression optimization.
    fn apply_compression_optimization(&self) -> Result<()> {
        info!("Applying compression optimization");
        
        for filesystem in &self.info.filesystems {
            // Skip file systems that don't support compression
            if !filesystem.supports_compression {
                continue;
            }
            
            if filesystem.fs_type == "btrfs" {
                // Enable compression for Btrfs
                let output = Command::new("btrfs")
                    .arg("filesystem")
                    .arg("defrag")
                    .arg("-r")
                    .arg("-czstd")
                    .arg(filesystem.mount_point.to_str().unwrap_or(""))
                    .output()
                    .context("Failed to execute btrfs command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to enable compression for {}: {}", filesystem.mount_point.display(), error);
                }
            } else if filesystem.fs_type == "f2fs" {
                // Enable compression for F2FS
                let output = Command::new("mount")
                    .arg("-o")
                    .arg("remount,compress_algorithm=lz4")
                    .arg(filesystem.mount_point.to_str().unwrap_or(""))
                    .output()
                    .context("Failed to execute mount command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to enable compression for {}: {}", filesystem.mount_point.display(), error);
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply defragmentation optimization.
    fn apply_defragmentation_optimization(&self) -> Result<()> {
        info!("Applying defragmentation optimization");
        
        for filesystem in &self.info.filesystems {
            // Skip file systems that don't support defragmentation
            if !filesystem.supports_defragmentation {
                continue;
            }
            
            if filesystem.fs_type == "ext4" {
                // Defragment ext4 file system
                let output = Command::new("e4defrag")
                    .arg("-c")
                    .arg(filesystem.mount_point.to_str().unwrap_or(""))
                    .output()
                    .context("Failed to execute e4defrag command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to defragment {}: {}", filesystem.mount_point.display(), error);
                }
            } else if filesystem.fs_type == "btrfs" {
                // Defragment Btrfs file system
                let output = Command::new("btrfs")
                    .arg("filesystem")
                    .arg("defrag")
                    .arg("-r")
                    .arg(filesystem.mount_point.to_str().unwrap_or(""))
                    .output()
                    .context("Failed to execute btrfs command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to defragment {}: {}", filesystem.mount_point.display(), error);
                }
            } else if filesystem.fs_type == "f2fs" {
                // Defragment F2FS file system
                let output = Command::new("fsck.f2fs")
                    .arg("-d")
                    .arg("0")
                    .arg(filesystem.device.to_str().unwrap_or(""))
                    .output()
                    .context("Failed to execute fsck.f2fs command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to defragment {}: {}", filesystem.mount_point.display(), error);
                }
            }
        }
        
        Ok(())
    }
    
    /// Set I/O scheduler.
    fn set_io_scheduler(&self, device_path: &Path, scheduler: IoScheduler) -> Result<()> {
        debug!("Setting I/O scheduler for {} to {}", device_path.display(), scheduler.to_str());
        
        let device_name = device_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid device path"))?;
        
        let scheduler_path = Path::new("/sys/block")
            .join(device_name)
            .join("queue/scheduler");
        
        if scheduler_path.exists() {
            fs::write(&scheduler_path, scheduler.to_str())
                .context("Failed to set I/O scheduler")?;
        } else {
            warn!("I/O scheduler setting not supported for {}", device_path.display());
        }
        
        Ok(())
    }
    
    /// Set I/O scheduler (static method).
    fn set_io_scheduler_static(device_path: &Path, scheduler: IoScheduler) -> Result<()> {
        debug!("Setting I/O scheduler for {} to {}", device_path.display(), scheduler.to_str());
        
        let device_name = device_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid device path"))?;
        
        let scheduler_path = Path::new("/sys/block")
            .join(device_name)
            .join("queue/scheduler");
        
        if scheduler_path.exists() {
            fs::write(&scheduler_path, scheduler.to_str())
                .context("Failed to set I/O scheduler")?;
        } else {
            warn!("I/O scheduler setting not supported for {}", device_path.display());
        }
        
        Ok(())
    }
    
    /// Set read-ahead.
    fn set_readahead(&self, device_path: &Path, readahead_kb: u32) -> Result<()> {
        debug!("Setting read-ahead for {} to {} KB", device_path.display(), readahead_kb);
        
        let device_name = device_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid device path"))?;
        
        let readahead_path = Path::new("/sys/block")
            .join(device_name)
            .join("queue/read_ahead_kb");
        
        if readahead_path.exists() {
            fs::write(&readahead_path, readahead_kb.to_string())
                .context("Failed to set read-ahead")?;
        } else {
            warn!("Read-ahead setting not supported for {}", device_path.display());
        }
        
        Ok(())
    }
    
    /// Set read-ahead (static method).
    fn set_readahead_static(device_path: &Path, readahead_kb: u32) -> Result<()> {
        debug!("Setting read-ahead for {} to {} KB", device_path.display(), readahead_kb);
        
        let device_name = device_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid device path"))?;
        
        let readahead_path = Path::new("/sys/block")
            .join(device_name)
            .join("queue/read_ahead_kb");
        
        if readahead_path.exists() {
            fs::write(&readahead_path, readahead_kb.to_string())
                .context("Failed to set read-ahead")?;
        } else {
            warn!("Read-ahead setting not supported for {}", device_path.display());
        }
        
        Ok(())
    }
    
    /// Find VR processes.
    fn find_vr_processes(&self) -> Result<Vec<u32>> {
        debug!("Finding VR processes");
        
        let mut vr_processes = Vec::new();
        
        // VR process patterns
        let vr_patterns = [
            "vr", "slam", "tracking", "mapping", "feature",
            "imu", "camera", "display", "tpu",
        ];
        
        // Read process list
        let proc_dir = Path::new("/proc");
        if proc_dir.exists() && proc_dir.is_dir() {
            for entry in fs::read_dir(proc_dir)
                .context("Failed to read process directory")? {
                let entry = entry.context("Failed to read process directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(pid_str) = path.file_name()
                        .and_then(|n| n.to_str())
                        .filter(|s| s.chars().all(|c| c.is_digit(10))) {
                        
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            // Read process name
                            let comm_path = path.join("comm");
                            if comm_path.exists() {
                                let comm = fs::read_to_string(&comm_path)
                                    .unwrap_or_else(|_| "".to_string())
                                    .trim()
                                    .to_string();
                                
                                // Check if process name matches VR patterns
                                if vr_patterns.iter().any(|p| comm.contains(p)) {
                                    vr_processes.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(vr_processes)
    }
    
    /// Detect storage information.
    fn detect_storage_info() -> Result<StorageInfo> {
        info!("Detecting storage information");
        
        let mut devices = Vec::new();
        let mut filesystems = Vec::new();
        let mut available_schedulers = Vec::new();
        let mut supports_io_priority = false;
        let mut supports_trim = false;
        let mut supports_compression = false;
        let mut supports_defragmentation = false;
        
        // Detect block devices
        let block_dir = Path::new("/sys/block");
        if block_dir.exists() && block_dir.is_dir() {
            for entry in fs::read_dir(block_dir)
                .context("Failed to read block directory")? {
                let entry = entry.context("Failed to read block directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(device_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip loop and ram devices
                        if device_name.starts_with("loop") || device_name.starts_with("ram") {
                            continue;
                        }
                        
                        let device_path = Path::new("/dev").join(device_name);
                        
                        // Get device type
                        let device_type = if device_name.starts_with("sd") {
                            // Check if it's an SSD or HDD
                            let rotational_path = path.join("queue/rotational");
                            if rotational_path.exists() {
                                let rotational = fs::read_to_string(&rotational_path)
                                    .unwrap_or_else(|_| "1".to_string())
                                    .trim()
                                    .to_string();
                                
                                if rotational == "0" {
                                    StorageDeviceType::Ssd
                                } else {
                                    StorageDeviceType::Hdd
                                }
                            } else {
                                StorageDeviceType::Hdd
                            }
                        } else if device_name.starts_with("nvme") {
                            StorageDeviceType::Nvme
                        } else if device_name.starts_with("mmcblk") {
                            // Check if it's eMMC or SD card
                            if device_name.contains("boot") {
                                StorageDeviceType::Emmc
                            } else {
                                StorageDeviceType::Sd
                            }
                        } else {
                            StorageDeviceType::Unknown
                        };
                        
                        // Get device size
                        let size_path = path.join("size");
                        let size = if size_path.exists() {
                            let size_str = fs::read_to_string(&size_path)
                                .unwrap_or_else(|_| "0".to_string())
                                .trim()
                                .to_string();
                            
                            if let Ok(sectors) = size_str.parse::<u64>() {
                                // Convert sectors to MB (sector size is 512 bytes)
                                sectors * 512 / (1024 * 1024)
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        
                        // Get available schedulers
                        let scheduler_path = path.join("queue/scheduler");
                        let device_schedulers = if scheduler_path.exists() {
                            let scheduler_str = fs::read_to_string(&scheduler_path)
                                .unwrap_or_else(|_| "".to_string())
                                .trim()
                                .to_string();
                            
                            let mut schedulers = Vec::new();
                            
                            for scheduler in scheduler_str.split_whitespace() {
                                let scheduler = scheduler.trim_matches(|c| c == '[' || c == ']');
                                if let Ok(s) = IoScheduler::from_str(scheduler) {
                                    schedulers.push(s);
                                    
                                    // Add to global list if not already present
                                    if !available_schedulers.contains(&s) {
                                        available_schedulers.push(s);
                                    }
                                }
                            }
                            
                            schedulers
                        } else {
                            Vec::new()
                        };
                        
                        // Get current scheduler
                        let current_scheduler = if scheduler_path.exists() {
                            let scheduler_str = fs::read_to_string(&scheduler_path)
                                .unwrap_or_else(|_| "".to_string())
                                .trim()
                                .to_string();
                            
                            let current = scheduler_str
                                .split_whitespace()
                                .find(|s| s.starts_with("[") && s.ends_with("]"))
                                .map(|s| s.trim_matches(|c| c == '[' || c == ']'))
                                .unwrap_or("cfq");
                            
                            IoScheduler::from_str(current).unwrap_or(IoScheduler::Cfq)
                        } else {
                            IoScheduler::Cfq
                        };
                        
                        // Get read-ahead size
                        let readahead_path = path.join("queue/read_ahead_kb");
                        let readahead_kb = if readahead_path.exists() {
                            let readahead_str = fs::read_to_string(&readahead_path)
                                .unwrap_or_else(|_| "128".to_string())
                                .trim()
                                .to_string();
                            
                            readahead_str.parse::<u32>().unwrap_or(128)
                        } else {
                            128
                        };
                        
                        // Check if device supports trim
                        let discard_path = path.join("queue/discard_granularity");
                        let device_supports_trim = if discard_path.exists() {
                            let discard_str = fs::read_to_string(&discard_path)
                                .unwrap_or_else(|_| "0".to_string())
                                .trim()
                                .to_string();
                            
                            discard_str != "0"
                        } else {
                            false
                        };
                        
                        // Update global trim support
                        if device_supports_trim {
                            supports_trim = true;
                        }
                        
                        devices.push(StorageDevice {
                            name: device_name.to_string(),
                            path: device_path,
                            device_type,
                            size,
                            scheduler: current_scheduler,
                            available_schedulers: device_schedulers,
                            readahead_kb,
                            supports_trim: device_supports_trim,
                        });
                    }
                }
            }
        }
        
        // Detect file systems
        let mounts = fs::read_to_string("/proc/mounts")
            .context("Failed to read mounts information")?;
        
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let device = parts[0];
                let mount_point = parts[1];
                let fs_type = parts[2];
                let options = parts[3];
                
                // Skip special file systems
                if fs_type == "proc" || fs_type == "sysfs" || fs_type == "devtmpfs" || fs_type == "tmpfs" {
                    continue;
                }
                
                // Parse mount options
                let mount_options: Vec<String> = options.split(',').map(|s| s.to_string()).collect();
                
                // Get file system size and free space
                let mut total_size = 0;
                let mut free_size = 0;
                
                let output = Command::new("df")
                    .arg("-B")
                    .arg("1M")
                    .arg(mount_point)
                    .output()
                    .context("Failed to execute df command")?;
                
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = output_str.lines().collect();
                    
                    if lines.len() >= 2 {
                        let fields: Vec<&str> = lines[1].split_whitespace().collect();
                        if fields.len() >= 4 {
                            if let Ok(size) = fields[1].parse::<u64>() {
                                total_size = size;
                            }
                            
                            if let Ok(avail) = fields[3].parse::<u64>() {
                                free_size = avail;
                            }
                        }
                    }
                }
                
                // Check if file system supports trim
                let fs_supports_trim = supports_trim && (fs_type == "ext4" || fs_type == "btrfs" || fs_type == "f2fs" || fs_type == "xfs");
                
                // Check if file system supports compression
                let fs_supports_compression = fs_type == "btrfs" || fs_type == "f2fs";
                
                // Update global compression support
                if fs_supports_compression {
                    supports_compression = true;
                }
                
                // Check if file system supports defragmentation
                let fs_supports_defragmentation = fs_type == "ext4" || fs_type == "btrfs" || fs_type == "f2fs";
                
                // Update global defragmentation support
                if fs_supports_defragmentation {
                    supports_defragmentation = true;
                }
                
                filesystems.push(FileSystem {
                    mount_point: PathBuf::from(mount_point),
                    device: PathBuf::from(device),
                    fs_type: fs_type.to_string(),
                    mount_options,
                    total_size,
                    free_size,
                    supports_trim: fs_supports_trim,
                    supports_compression: fs_supports_compression,
                    supports_defragmentation: fs_supports_defragmentation,
                });
            }
        }
        
        // Check if I/O priority is supported
        let output = Command::new("ionice")
            .arg("--help")
            .output()
            .context("Failed to execute ionice command")?;
        
        supports_io_priority = output.status.success();
        
        Ok(StorageInfo {
            devices,
            filesystems,
            available_schedulers,
            supports_io_priority,
            supports_trim,
            supports_compression,
            supports_defragmentation,
        })
    }
    
    /// Get current storage optimization state.
    fn get_current_state(info: &StorageInfo) -> Result<StorageOptimizationState> {
        debug!("Getting current storage optimization state");
        
        let mut io_schedulers = Vec::new();
        let mut readahead_sizes = Vec::new();
        let mut io_stats = Vec::new();
        
        // Get current I/O schedulers and read-ahead sizes
        for device in &info.devices {
            io_schedulers.push((device.name.clone(), device.scheduler));
            readahead_sizes.push((device.name.clone(), device.readahead_kb));
            
            // Get I/O statistics
            let stat_path = Path::new("/sys/block")
                .join(&device.name)
                .join("stat");
            
            if stat_path.exists() {
                let stat_str = fs::read_to_string(&stat_path)
                    .unwrap_or_else(|_| "".to_string())
                    .trim()
                    .to_string();
                
                let fields: Vec<&str> = stat_str.split_whitespace().collect();
                
                if fields.len() >= 11 {
                    let reads_completed = fields[0].parse::<u64>().unwrap_or(0);
                    let reads_merged = fields[1].parse::<u64>().unwrap_or(0);
                    let sectors_read = fields[2].parse::<u64>().unwrap_or(0);
                    let read_time_ms = fields[3].parse::<u64>().unwrap_or(0);
                    let writes_completed = fields[4].parse::<u64>().unwrap_or(0);
                    let writes_merged = fields[5].parse::<u64>().unwrap_or(0);
                    let sectors_written = fields[6].parse::<u64>().unwrap_or(0);
                    let write_time_ms = fields[7].parse::<u64>().unwrap_or(0);
                    let ios_in_progress = fields[8].parse::<u64>().unwrap_or(0);
                    let io_time_ms = fields[9].parse::<u64>().unwrap_or(0);
                    let weighted_io_time_ms = fields[10].parse::<u64>().unwrap_or(0);
                    
                    io_stats.push((device.name.clone(), IoStats {
                        reads_completed,
                        reads_merged,
                        sectors_read,
                        read_time_ms,
                        writes_completed,
                        writes_merged,
                        sectors_written,
                        write_time_ms,
                        ios_in_progress,
                        io_time_ms,
                        weighted_io_time_ms,
                    }));
                }
            }
        }
        
        // Get file system cache statistics
        let meminfo = fs::read_to_string("/proc/meminfo")
            .context("Failed to read memory information")?;
        
        let mut total_cache_size = 0;
        let mut dirty_cache_size = 0;
        let mut writeback_cache_size = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("Cached:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(size) = parts[1].parse::<u32>() {
                        total_cache_size = size / 1024; // Convert to MB
                    }
                }
            } else if line.starts_with("Dirty:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(size) = parts[1].parse::<u32>() {
                        dirty_cache_size = size / 1024; // Convert to MB
                    }
                }
            } else if line.starts_with("Writeback:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(size) = parts[1].parse::<u32>() {
                        writeback_cache_size = size / 1024; // Convert to MB
                    }
                }
            }
        }
        
        let fs_cache_stats = FsCacheStats {
            total_cache_size,
            dirty_cache_size,
            writeback_cache_size,
        };
        
        Ok(StorageOptimizationState {
            io_schedulers,
            readahead_sizes,
            io_stats,
            fs_cache_stats,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_io_scheduler_conversion() {
        assert_eq!(IoScheduler::Cfq.to_str(), "cfq");
        assert_eq!(IoScheduler::Deadline.to_str(), "deadline");
        assert_eq!(IoScheduler::Noop.to_str(), "noop");
        assert_eq!(IoScheduler::Bfq.to_str(), "bfq");
        assert_eq!(IoScheduler::Kyber.to_str(), "kyber");
        assert_eq!(IoScheduler::Mq.to_str(), "mq-deadline");
        
        assert_eq!(IoScheduler::from_str("cfq").unwrap(), IoScheduler::Cfq);
        assert_eq!(IoScheduler::from_str("deadline").unwrap(), IoScheduler::Deadline);
        assert_eq!(IoScheduler::from_str("noop").unwrap(), IoScheduler::Noop);
        assert_eq!(IoScheduler::from_str("bfq").unwrap(), IoScheduler::Bfq);
        assert_eq!(IoScheduler::from_str("kyber").unwrap(), IoScheduler::Kyber);
        assert_eq!(IoScheduler::from_str("mq-deadline").unwrap(), IoScheduler::Mq);
        
        assert!(IoScheduler::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_storage_optimization_settings_default() {
        let settings = StorageOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert_eq!(settings.io_scheduler, IoScheduler::Bfq);
        assert!(settings.optimize_readahead);
        assert_eq!(settings.readahead_kb, 512);
        assert!(settings.optimize_io_priority);
        assert!(settings.optimize_fs_cache);
        assert!(settings.optimize_mount_options);
        assert!(settings.optimize_trim);
        assert!(settings.optimize_journaling);
        assert!(!settings.optimize_compression);
        assert!(settings.optimize_defragmentation);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 5000);
    }
}
