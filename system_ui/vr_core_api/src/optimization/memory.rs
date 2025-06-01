//! Memory optimization module for the VR headset system.
//!
//! This module provides memory optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages memory allocation, paging,
//! swapping, and other memory-related optimizations to maximize performance for VR workloads.

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

/// Memory optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct MemoryOptimizationManager {
    /// Memory optimization settings
    settings: MemoryOptimizationSettings,
    
    /// Memory information
    info: MemoryInfo,
    
    /// Current memory optimization state
    state: MemoryOptimizationState,
    
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
    
    /// Current memory optimization settings
    settings: MemoryOptimizationSettings,
}

/// Memory optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationSettings {
    /// Whether memory optimization is enabled
    pub enabled: bool,
    
    /// Whether to optimize memory allocation
    pub optimize_allocation: bool,
    
    /// Whether to optimize paging
    pub optimize_paging: bool,
    
    /// Whether to optimize swapping
    pub optimize_swapping: bool,
    
    /// Whether to optimize huge pages
    pub optimize_huge_pages: bool,
    
    /// Whether to optimize memory compression
    pub optimize_compression: bool,
    
    /// Whether to optimize memory defragmentation
    pub optimize_defragmentation: bool,
    
    /// Whether to optimize memory priority
    pub optimize_priority: bool,
    
    /// Swappiness value (0-100)
    pub swappiness: u8,
    
    /// VFS cache pressure (0-100)
    pub vfs_cache_pressure: u8,
    
    /// Dirty ratio (0-100)
    pub dirty_ratio: u8,
    
    /// Dirty background ratio (0-100)
    pub dirty_background_ratio: u8,
    
    /// Minimum free memory (in MB)
    pub min_free_memory: u32,
    
    /// Maximum swap usage (in MB)
    pub max_swap_usage: u32,
    
    /// Number of huge pages to allocate
    pub nr_hugepages: u32,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// Memory information.
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Total physical memory (in MB)
    pub total_memory: u32,
    
    /// Total swap space (in MB)
    pub total_swap: u32,
    
    /// Page size (in KB)
    pub page_size: u32,
    
    /// Huge page size (in KB)
    pub huge_page_size: u32,
    
    /// Whether huge pages are supported
    pub supports_huge_pages: bool,
    
    /// Whether memory compression is supported
    pub supports_compression: bool,
    
    /// Whether memory defragmentation is supported
    pub supports_defragmentation: bool,
    
    /// Whether memory priority is supported
    pub supports_priority: bool,
}

/// Memory optimization state.
#[derive(Debug, Clone)]
pub struct MemoryOptimizationState {
    /// Free physical memory (in MB)
    pub free_memory: u32,
    
    /// Used physical memory (in MB)
    pub used_memory: u32,
    
    /// Cached memory (in MB)
    pub cached_memory: u32,
    
    /// Buffers memory (in MB)
    pub buffers_memory: u32,
    
    /// Free swap space (in MB)
    pub free_swap: u32,
    
    /// Used swap space (in MB)
    pub used_swap: u32,
    
    /// Current swappiness value
    pub swappiness: u8,
    
    /// Current VFS cache pressure
    pub vfs_cache_pressure: u8,
    
    /// Current dirty ratio
    pub dirty_ratio: u8,
    
    /// Current dirty background ratio
    pub dirty_background_ratio: u8,
    
    /// Current number of huge pages
    pub nr_hugepages: u32,
    
    /// Current number of allocated huge pages
    pub nr_hugepages_allocated: u32,
}

impl Default for MemoryOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            optimize_allocation: true,
            optimize_paging: true,
            optimize_swapping: true,
            optimize_huge_pages: true,
            optimize_compression: true,
            optimize_defragmentation: true,
            optimize_priority: true,
            swappiness: 10,
            vfs_cache_pressure: 50,
            dirty_ratio: 20,
            dirty_background_ratio: 10,
            min_free_memory: 1024, // 1 GB
            max_swap_usage: 4096,  // 4 GB
            nr_hugepages: 128,
            adaptive: true,
            adaptive_interval_ms: 1000,
        }
    }
}

impl MemoryOptimizationManager {
    /// Create a new memory optimization manager.
    pub fn new() -> Result<Self> {
        let info = Self::detect_memory_info()?;
        let settings = MemoryOptimizationSettings::default();
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
    
    /// Initialize memory optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing memory optimization for Orange Pi CM5");
        
        // Detect memory information
        self.info = Self::detect_memory_info()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("Memory optimization initialized successfully");
        Ok(())
    }
    
    /// Apply memory optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying memory optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply memory allocation optimization if enabled
        if self.settings.optimize_allocation {
            self.apply_allocation_optimization()?;
        }
        
        // Apply paging optimization if enabled
        if self.settings.optimize_paging {
            self.apply_paging_optimization()?;
        }
        
        // Apply swapping optimization if enabled
        if self.settings.optimize_swapping {
            self.apply_swapping_optimization()?;
        }
        
        // Apply huge pages optimization if enabled
        if self.settings.optimize_huge_pages {
            self.apply_huge_pages_optimization()?;
        }
        
        // Apply memory compression optimization if enabled
        if self.settings.optimize_compression {
            self.apply_compression_optimization()?;
        }
        
        // Apply memory defragmentation optimization if enabled
        if self.settings.optimize_defragmentation {
            self.apply_defragmentation_optimization()?;
        }
        
        // Apply memory priority optimization if enabled
        if self.settings.optimize_priority {
            self.apply_priority_optimization()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("Memory optimizations applied successfully");
        Ok(())
    }
    
    /// Reset memory optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting memory optimizations");
        
        // Reset swappiness
        self.set_swappiness(60)?;
        
        // Reset VFS cache pressure
        self.set_vfs_cache_pressure(100)?;
        
        // Reset dirty ratio
        self.set_dirty_ratio(20)?;
        
        // Reset dirty background ratio
        self.set_dirty_background_ratio(10)?;
        
        // Reset huge pages
        self.set_nr_hugepages(0)?;
        
        // Reset memory compression
        self.reset_compression()?;
        
        // Reset memory defragmentation
        self.reset_defragmentation()?;
        
        // Reset memory priority
        self.reset_priority()?;
        
        info!("Memory optimizations reset successfully");
        Ok(())
    }
    
    /// Update memory optimization settings.
    pub fn update_settings(&mut self, settings: MemoryOptimizationSettings) -> Result<()> {
        info!("Updating memory optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("Memory optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current memory optimization settings.
    pub fn get_settings(&self) -> MemoryOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current memory optimization state.
    pub fn get_state(&self) -> MemoryOptimizationState {
        self.state.clone()
    }
    
    /// Get memory information.
    pub fn get_info(&self) -> MemoryInfo {
        self.info.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background memory optimization thread");
        
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
        
        info!("Background memory optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background memory optimization thread");
        
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
        
        info!("Background memory optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive optimization.
    fn perform_adaptive_optimization(info: &MemoryInfo, settings: &MemoryOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive memory optimization");
        
        // Get current memory state
        let state = Self::get_current_state(info)?;
        
        // Check free memory and adjust swappiness if necessary
        let free_memory_percent = (state.free_memory as f32 / info.total_memory as f32) * 100.0;
        
        if free_memory_percent < 10.0 {
            // Low free memory, increase swappiness
            let new_swappiness = (state.swappiness as u32 + 10).min(100) as u8;
            if new_swappiness != state.swappiness {
                if let Err(e) = Self::set_swappiness_static(new_swappiness) {
                    warn!("Error setting swappiness: {}", e);
                }
            }
            
            // Increase VFS cache pressure
            let new_vfs_cache_pressure = (state.vfs_cache_pressure as u32 + 10).min(100) as u8;
            if new_vfs_cache_pressure != state.vfs_cache_pressure {
                if let Err(e) = Self::set_vfs_cache_pressure_static(new_vfs_cache_pressure) {
                    warn!("Error setting VFS cache pressure: {}", e);
                }
            }
        } else if free_memory_percent > 30.0 {
            // High free memory, decrease swappiness
            let new_swappiness = state.swappiness.saturating_sub(10).max(settings.swappiness);
            if new_swappiness != state.swappiness {
                if let Err(e) = Self::set_swappiness_static(new_swappiness) {
                    warn!("Error setting swappiness: {}", e);
                }
            }
            
            // Decrease VFS cache pressure
            let new_vfs_cache_pressure = state.vfs_cache_pressure.saturating_sub(10).max(settings.vfs_cache_pressure);
            if new_vfs_cache_pressure != state.vfs_cache_pressure {
                if let Err(e) = Self::set_vfs_cache_pressure_static(new_vfs_cache_pressure) {
                    warn!("Error setting VFS cache pressure: {}", e);
                }
            }
        }
        
        // Check swap usage and adjust if necessary
        let swap_usage_percent = if info.total_swap > 0 {
            (state.used_swap as f32 / info.total_swap as f32) * 100.0
        } else {
            0.0
        };
        
        if swap_usage_percent > 80.0 {
            // High swap usage, try to reduce it
            if let Err(e) = Self::drop_caches() {
                warn!("Error dropping caches: {}", e);
            }
        }
        
        debug!("Adaptive memory optimization completed");
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
                self.settings.optimize_allocation = true;
                self.settings.optimize_paging = true;
                self.settings.optimize_swapping = true;
                self.settings.optimize_huge_pages = true;
                self.settings.optimize_compression = true;
                self.settings.optimize_defragmentation = true;
                self.settings.optimize_priority = true;
                self.settings.swappiness = 10;
                self.settings.vfs_cache_pressure = 50;
                self.settings.dirty_ratio = 30;
                self.settings.dirty_background_ratio = 15;
                self.settings.min_free_memory = 2048; // 2 GB
                self.settings.max_swap_usage = 2048;  // 2 GB
                self.settings.nr_hugepages = 256;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.optimize_allocation = true;
                self.settings.optimize_paging = true;
                self.settings.optimize_swapping = true;
                self.settings.optimize_huge_pages = false;
                self.settings.optimize_compression = true;
                self.settings.optimize_defragmentation = false;
                self.settings.optimize_priority = true;
                self.settings.swappiness = 60;
                self.settings.vfs_cache_pressure = 100;
                self.settings.dirty_ratio = 20;
                self.settings.dirty_background_ratio = 10;
                self.settings.min_free_memory = 512;  // 512 MB
                self.settings.max_swap_usage = 4096; // 4 GB
                self.settings.nr_hugepages = 0;
            },
            super::OptimizationMode::Latency => {
                self.settings.optimize_allocation = true;
                self.settings.optimize_paging = true;
                self.settings.optimize_swapping = true;
                self.settings.optimize_huge_pages = true;
                self.settings.optimize_compression = false;
                self.settings.optimize_defragmentation = true;
                self.settings.optimize_priority = true;
                self.settings.swappiness = 0;
                self.settings.vfs_cache_pressure = 50;
                self.settings.dirty_ratio = 30;
                self.settings.dirty_background_ratio = 15;
                self.settings.min_free_memory = 4096; // 4 GB
                self.settings.max_swap_usage = 0;     // No swap
                self.settings.nr_hugepages = 512;
            },
            super::OptimizationMode::Thermal => {
                self.settings.optimize_allocation = true;
                self.settings.optimize_paging = true;
                self.settings.optimize_swapping = true;
                self.settings.optimize_huge_pages = false;
                self.settings.optimize_compression = true;
                self.settings.optimize_defragmentation = false;
                self.settings.optimize_priority = true;
                self.settings.swappiness = 60;
                self.settings.vfs_cache_pressure = 100;
                self.settings.dirty_ratio = 20;
                self.settings.dirty_background_ratio = 10;
                self.settings.min_free_memory = 1024; // 1 GB
                self.settings.max_swap_usage = 4096; // 4 GB
                self.settings.nr_hugepages = 0;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.optimize_allocation = true;
            self.settings.optimize_paging = true;
            self.settings.optimize_swapping = true;
            self.settings.optimize_huge_pages = true;
            self.settings.optimize_compression = true;
            self.settings.optimize_defragmentation = true;
            self.settings.optimize_priority = true;
            self.settings.swappiness = 0;
            self.settings.vfs_cache_pressure = 50;
            self.settings.dirty_ratio = 40;
            self.settings.dirty_background_ratio = 20;
            self.settings.min_free_memory = 4096; // 4 GB
            self.settings.max_swap_usage = 0;     // No swap
            self.settings.nr_hugepages = 1024;
        }
    }
    
    /// Apply memory allocation optimization.
    fn apply_allocation_optimization(&self) -> Result<()> {
        info!("Applying memory allocation optimization");
        
        // Set minimum free memory
        let min_free_path = Path::new("/proc/sys/vm/min_free_kbytes");
        if min_free_path.exists() {
            fs::write(min_free_path, (self.settings.min_free_memory * 1024).to_string())
                .context("Failed to set minimum free memory")?;
        }
        
        // Set watermark scale factor
        let watermark_path = Path::new("/proc/sys/vm/watermark_scale_factor");
        if watermark_path.exists() {
            fs::write(watermark_path, "150")
                .context("Failed to set watermark scale factor")?;
        }
        
        // Set overcommit memory policy
        let overcommit_path = Path::new("/proc/sys/vm/overcommit_memory");
        if overcommit_path.exists() {
            fs::write(overcommit_path, "1") // Allow overcommit
                .context("Failed to set overcommit memory policy")?;
        }
        
        // Set overcommit ratio
        let overcommit_ratio_path = Path::new("/proc/sys/vm/overcommit_ratio");
        if overcommit_ratio_path.exists() {
            fs::write(overcommit_ratio_path, "80")
                .context("Failed to set overcommit ratio")?;
        }
        
        // Set page cluster
        let page_cluster_path = Path::new("/proc/sys/vm/page-cluster");
        if page_cluster_path.exists() {
            fs::write(page_cluster_path, "3")
                .context("Failed to set page cluster")?;
        }
        
        // Set zone reclaim mode
        let zone_reclaim_path = Path::new("/proc/sys/vm/zone_reclaim_mode");
        if zone_reclaim_path.exists() {
            fs::write(zone_reclaim_path, "0") // Disable zone reclaim
                .context("Failed to set zone reclaim mode")?;
        }
        
        Ok(())
    }
    
    /// Apply paging optimization.
    fn apply_paging_optimization(&self) -> Result<()> {
        info!("Applying paging optimization");
        
        // Set dirty ratio
        self.set_dirty_ratio(self.settings.dirty_ratio)?;
        
        // Set dirty background ratio
        self.set_dirty_background_ratio(self.settings.dirty_background_ratio)?;
        
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
        
        // Set laptop mode
        let laptop_mode_path = Path::new("/proc/sys/vm/laptop_mode");
        if laptop_mode_path.exists() {
            fs::write(laptop_mode_path, "0") // Disable laptop mode
                .context("Failed to set laptop mode")?;
        }
        
        // Set VFS cache pressure
        self.set_vfs_cache_pressure(self.settings.vfs_cache_pressure)?;
        
        Ok(())
    }
    
    /// Apply swapping optimization.
    fn apply_swapping_optimization(&self) -> Result<()> {
        info!("Applying swapping optimization");
        
        // Set swappiness
        self.set_swappiness(self.settings.swappiness)?;
        
        // Set maximum swap usage
        if self.settings.max_swap_usage == 0 {
            // Disable swap
            let output = Command::new("swapoff")
                .arg("-a")
                .output()
                .context("Failed to execute swapoff command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to disable swap: {}", error);
            }
        } else {
            // Enable swap
            let output = Command::new("swapon")
                .arg("-a")
                .output()
                .context("Failed to execute swapon command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to enable swap: {}", error);
            }
            
            // Set maximum swap usage
            // Note: There's no direct way to limit swap usage in Linux,
            // but we can adjust swappiness and other parameters to influence it
        }
        
        Ok(())
    }
    
    /// Apply huge pages optimization.
    fn apply_huge_pages_optimization(&self) -> Result<()> {
        info!("Applying huge pages optimization");
        
        // Set number of huge pages
        self.set_nr_hugepages(self.settings.nr_hugepages)?;
        
        // Set huge pages defrag
        let defrag_path = Path::new("/proc/sys/vm/compact_memory");
        if defrag_path.exists() {
            fs::write(defrag_path, "1")
                .context("Failed to trigger memory compaction")?;
        }
        
        // Set transparent huge pages
        let thp_path = Path::new("/sys/kernel/mm/transparent_hugepage/enabled");
        if thp_path.exists() {
            fs::write(thp_path, "always")
                .context("Failed to set transparent huge pages")?;
        }
        
        // Set transparent huge page defrag
        let thp_defrag_path = Path::new("/sys/kernel/mm/transparent_hugepage/defrag");
        if thp_defrag_path.exists() {
            fs::write(thp_defrag_path, "always")
                .context("Failed to set transparent huge page defrag")?;
        }
        
        // Set transparent huge page khugepaged
        let khugepaged_path = Path::new("/sys/kernel/mm/transparent_hugepage/khugepaged/defrag");
        if khugepaged_path.exists() {
            fs::write(khugepaged_path, "1")
                .context("Failed to enable khugepaged defrag")?;
        }
        
        Ok(())
    }
    
    /// Apply memory compression optimization.
    fn apply_compression_optimization(&self) -> Result<()> {
        info!("Applying memory compression optimization");
        
        // Check if zswap is available
        let zswap_enabled_path = Path::new("/sys/module/zswap/parameters/enabled");
        if zswap_enabled_path.exists() {
            // Enable zswap
            fs::write(zswap_enabled_path, "Y")
                .context("Failed to enable zswap")?;
            
            // Set zswap compression algorithm
            let zswap_compressor_path = Path::new("/sys/module/zswap/parameters/compressor");
            if zswap_compressor_path.exists() {
                fs::write(zswap_compressor_path, "lz4")
                    .context("Failed to set zswap compression algorithm")?;
            }
            
            // Set zswap pool size
            let zswap_max_pool_path = Path::new("/sys/module/zswap/parameters/max_pool_percent");
            if zswap_max_pool_path.exists() {
                fs::write(zswap_max_pool_path, "20")
                    .context("Failed to set zswap pool size")?;
            }
        } else {
            // Check if zram is available
            let output = Command::new("modprobe")
                .arg("zram")
                .output()
                .context("Failed to execute modprobe command")?;
            
            if output.status.success() {
                // Configure zram
                let zram_size = self.info.total_memory / 4; // 25% of RAM
                
                // Create zram device
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(format!("echo lz4 > /sys/block/zram0/comp_algorithm"))
                    .output()
                    .context("Failed to set zram compression algorithm")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to set zram compression algorithm: {}", error);
                }
                
                // Set zram size
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(format!("echo {}M > /sys/block/zram0/disksize", zram_size))
                    .output()
                    .context("Failed to set zram size")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to set zram size: {}", error);
                }
                
                // Enable zram as swap
                let output = Command::new("mkswap")
                    .arg("/dev/zram0")
                    .output()
                    .context("Failed to execute mkswap command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to create zram swap: {}", error);
                }
                
                // Enable zram swap
                let output = Command::new("swapon")
                    .arg("/dev/zram0")
                    .output()
                    .context("Failed to enable zram swap")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to enable zram swap: {}", error);
                }
            } else {
                warn!("Memory compression not supported");
            }
        }
        
        Ok(())
    }
    
    /// Reset memory compression.
    fn reset_compression(&self) -> Result<()> {
        info!("Resetting memory compression");
        
        // Check if zswap is available
        let zswap_enabled_path = Path::new("/sys/module/zswap/parameters/enabled");
        if zswap_enabled_path.exists() {
            // Disable zswap
            fs::write(zswap_enabled_path, "N")
                .context("Failed to disable zswap")?;
        }
        
        // Check if zram is in use
        let zram_path = Path::new("/dev/zram0");
        if zram_path.exists() {
            // Disable zram swap
            let output = Command::new("swapoff")
                .arg("/dev/zram0")
                .output()
                .context("Failed to disable zram swap")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to disable zram swap: {}", error);
            }
            
            // Remove zram device
            let output = Command::new("sh")
                .arg("-c")
                .arg("echo 1 > /sys/block/zram0/reset")
                .output()
                .context("Failed to reset zram device")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to reset zram device: {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Apply memory defragmentation optimization.
    fn apply_defragmentation_optimization(&self) -> Result<()> {
        info!("Applying memory defragmentation optimization");
        
        // Trigger memory compaction
        let compact_path = Path::new("/proc/sys/vm/compact_memory");
        if compact_path.exists() {
            fs::write(compact_path, "1")
                .context("Failed to trigger memory compaction")?;
        }
        
        // Set compaction proactiveness
        let proactiveness_path = Path::new("/proc/sys/vm/compaction_proactiveness");
        if proactiveness_path.exists() {
            fs::write(proactiveness_path, "20")
                .context("Failed to set compaction proactiveness")?;
        }
        
        // Set extfrag threshold
        let extfrag_path = Path::new("/proc/sys/vm/extfrag_threshold");
        if extfrag_path.exists() {
            fs::write(extfrag_path, "500")
                .context("Failed to set extfrag threshold")?;
        }
        
        // Set kswapd parameters
        let kswapd_threads_path = Path::new("/proc/sys/vm/kswapd_threads");
        if kswapd_threads_path.exists() {
            fs::write(kswapd_threads_path, "4")
                .context("Failed to set kswapd threads")?;
        }
        
        Ok(())
    }
    
    /// Reset memory defragmentation.
    fn reset_defragmentation(&self) -> Result<()> {
        info!("Resetting memory defragmentation");
        
        // Reset compaction proactiveness
        let proactiveness_path = Path::new("/proc/sys/vm/compaction_proactiveness");
        if proactiveness_path.exists() {
            fs::write(proactiveness_path, "0")
                .context("Failed to reset compaction proactiveness")?;
        }
        
        // Reset extfrag threshold
        let extfrag_path = Path::new("/proc/sys/vm/extfrag_threshold");
        if extfrag_path.exists() {
            fs::write(extfrag_path, "500")
                .context("Failed to reset extfrag threshold")?;
        }
        
        // Reset kswapd parameters
        let kswapd_threads_path = Path::new("/proc/sys/vm/kswapd_threads");
        if kswapd_threads_path.exists() {
            fs::write(kswapd_threads_path, "2")
                .context("Failed to reset kswapd threads")?;
        }
        
        Ok(())
    }
    
    /// Apply memory priority optimization.
    fn apply_priority_optimization(&self) -> Result<()> {
        info!("Applying memory priority optimization");
        
        // Find VR processes
        let vr_processes = self.find_vr_processes()?;
        
        for pid in vr_processes {
            // Set OOM score adjustment
            let oom_path = Path::new("/proc").join(pid.to_string()).join("oom_score_adj");
            if oom_path.exists() {
                fs::write(&oom_path, "-1000") // Never kill
                    .unwrap_or_else(|e| warn!("Failed to set OOM score for process {}: {}", pid, e));
            }
            
            // Set memory priority using cgroups
            let cgroup_path = Path::new("/sys/fs/cgroup/memory/vr");
            if !cgroup_path.exists() {
                fs::create_dir_all(cgroup_path)
                    .unwrap_or_else(|e| warn!("Failed to create memory cgroup: {}", e));
            }
            
            // Set memory limit
            let limit_path = cgroup_path.join("memory.limit_in_bytes");
            if limit_path.exists() {
                fs::write(&limit_path, "-1") // No limit
                    .unwrap_or_else(|e| warn!("Failed to set memory limit: {}", e));
            }
            
            // Set memory swappiness
            let swappiness_path = cgroup_path.join("memory.swappiness");
            if swappiness_path.exists() {
                fs::write(&swappiness_path, "0") // No swap
                    .unwrap_or_else(|e| warn!("Failed to set memory swappiness: {}", e));
            }
            
            // Add process to cgroup
            let tasks_path = cgroup_path.join("tasks");
            if tasks_path.exists() {
                fs::write(&tasks_path, pid.to_string())
                    .unwrap_or_else(|e| warn!("Failed to add process {} to memory cgroup: {}", pid, e));
            }
        }
        
        // Set memory priority for system processes
        let system_cgroup_path = Path::new("/sys/fs/cgroup/memory/system");
        if !system_cgroup_path.exists() {
            fs::create_dir_all(system_cgroup_path)
                .unwrap_or_else(|e| warn!("Failed to create system memory cgroup: {}", e));
        }
        
        // Set memory limit
        let limit_path = system_cgroup_path.join("memory.limit_in_bytes");
        if limit_path.exists() {
            let limit = (self.info.total_memory as u64 * 1024 * 1024 * 3) / 4; // 75% of RAM
            fs::write(&limit_path, limit.to_string())
                .unwrap_or_else(|e| warn!("Failed to set system memory limit: {}", e));
        }
        
        // Set memory swappiness
        let swappiness_path = system_cgroup_path.join("memory.swappiness");
        if swappiness_path.exists() {
            fs::write(&swappiness_path, "60")
                .unwrap_or_else(|e| warn!("Failed to set system memory swappiness: {}", e));
        }
        
        Ok(())
    }
    
    /// Reset memory priority.
    fn reset_priority(&self) -> Result<()> {
        info!("Resetting memory priority");
        
        // Find VR processes
        let vr_processes = self.find_vr_processes()?;
        
        for pid in vr_processes {
            // Reset OOM score adjustment
            let oom_path = Path::new("/proc").join(pid.to_string()).join("oom_score_adj");
            if oom_path.exists() {
                fs::write(&oom_path, "0") // Default
                    .unwrap_or_else(|e| warn!("Failed to reset OOM score for process {}: {}", pid, e));
            }
        }
        
        // Remove memory cgroups
        let vr_cgroup_path = Path::new("/sys/fs/cgroup/memory/vr");
        if vr_cgroup_path.exists() {
            // Move all processes to root cgroup
            let tasks_path = vr_cgroup_path.join("tasks");
            if tasks_path.exists() {
                if let Ok(tasks) = fs::read_to_string(&tasks_path) {
                    for pid in tasks.lines() {
                        let root_tasks_path = Path::new("/sys/fs/cgroup/memory/tasks");
                        if root_tasks_path.exists() {
                            fs::write(&root_tasks_path, pid)
                                .unwrap_or_else(|e| warn!("Failed to move process {} to root cgroup: {}", pid, e));
                        }
                    }
                }
            }
            
            // Remove cgroup
            fs::remove_dir(vr_cgroup_path)
                .unwrap_or_else(|e| warn!("Failed to remove VR memory cgroup: {}", e));
        }
        
        let system_cgroup_path = Path::new("/sys/fs/cgroup/memory/system");
        if system_cgroup_path.exists() {
            // Move all processes to root cgroup
            let tasks_path = system_cgroup_path.join("tasks");
            if tasks_path.exists() {
                if let Ok(tasks) = fs::read_to_string(&tasks_path) {
                    for pid in tasks.lines() {
                        let root_tasks_path = Path::new("/sys/fs/cgroup/memory/tasks");
                        if root_tasks_path.exists() {
                            fs::write(&root_tasks_path, pid)
                                .unwrap_or_else(|e| warn!("Failed to move process {} to root cgroup: {}", pid, e));
                        }
                    }
                }
            }
            
            // Remove cgroup
            fs::remove_dir(system_cgroup_path)
                .unwrap_or_else(|e| warn!("Failed to remove system memory cgroup: {}", e));
        }
        
        Ok(())
    }
    
    /// Set swappiness.
    fn set_swappiness(&self, swappiness: u8) -> Result<()> {
        debug!("Setting swappiness to {}", swappiness);
        
        let swappiness_path = Path::new("/proc/sys/vm/swappiness");
        if swappiness_path.exists() {
            fs::write(&swappiness_path, swappiness.to_string())
                .context("Failed to set swappiness")?;
        } else {
            warn!("Swappiness setting not supported");
        }
        
        Ok(())
    }
    
    /// Set swappiness (static method).
    fn set_swappiness_static(swappiness: u8) -> Result<()> {
        debug!("Setting swappiness to {}", swappiness);
        
        let swappiness_path = Path::new("/proc/sys/vm/swappiness");
        if swappiness_path.exists() {
            fs::write(&swappiness_path, swappiness.to_string())
                .context("Failed to set swappiness")?;
        } else {
            warn!("Swappiness setting not supported");
        }
        
        Ok(())
    }
    
    /// Set VFS cache pressure.
    fn set_vfs_cache_pressure(&self, pressure: u8) -> Result<()> {
        debug!("Setting VFS cache pressure to {}", pressure);
        
        let pressure_path = Path::new("/proc/sys/vm/vfs_cache_pressure");
        if pressure_path.exists() {
            fs::write(&pressure_path, pressure.to_string())
                .context("Failed to set VFS cache pressure")?;
        } else {
            warn!("VFS cache pressure setting not supported");
        }
        
        Ok(())
    }
    
    /// Set VFS cache pressure (static method).
    fn set_vfs_cache_pressure_static(pressure: u8) -> Result<()> {
        debug!("Setting VFS cache pressure to {}", pressure);
        
        let pressure_path = Path::new("/proc/sys/vm/vfs_cache_pressure");
        if pressure_path.exists() {
            fs::write(&pressure_path, pressure.to_string())
                .context("Failed to set VFS cache pressure")?;
        } else {
            warn!("VFS cache pressure setting not supported");
        }
        
        Ok(())
    }
    
    /// Set dirty ratio.
    fn set_dirty_ratio(&self, ratio: u8) -> Result<()> {
        debug!("Setting dirty ratio to {}", ratio);
        
        let ratio_path = Path::new("/proc/sys/vm/dirty_ratio");
        if ratio_path.exists() {
            fs::write(&ratio_path, ratio.to_string())
                .context("Failed to set dirty ratio")?;
        } else {
            warn!("Dirty ratio setting not supported");
        }
        
        Ok(())
    }
    
    /// Set dirty background ratio.
    fn set_dirty_background_ratio(&self, ratio: u8) -> Result<()> {
        debug!("Setting dirty background ratio to {}", ratio);
        
        let ratio_path = Path::new("/proc/sys/vm/dirty_background_ratio");
        if ratio_path.exists() {
            fs::write(&ratio_path, ratio.to_string())
                .context("Failed to set dirty background ratio")?;
        } else {
            warn!("Dirty background ratio setting not supported");
        }
        
        Ok(())
    }
    
    /// Set number of huge pages.
    fn set_nr_hugepages(&self, pages: u32) -> Result<()> {
        debug!("Setting number of huge pages to {}", pages);
        
        let hugepages_path = Path::new("/proc/sys/vm/nr_hugepages");
        if hugepages_path.exists() {
            fs::write(&hugepages_path, pages.to_string())
                .context("Failed to set number of huge pages")?;
        } else {
            warn!("Huge pages setting not supported");
        }
        
        Ok(())
    }
    
    /// Drop caches.
    fn drop_caches() -> Result<()> {
        debug!("Dropping caches");
        
        let drop_caches_path = Path::new("/proc/sys/vm/drop_caches");
        if drop_caches_path.exists() {
            fs::write(&drop_caches_path, "3") // Drop page cache, dentries and inodes
                .context("Failed to drop caches")?;
        } else {
            warn!("Drop caches not supported");
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
    
    /// Detect memory information.
    fn detect_memory_info() -> Result<MemoryInfo> {
        info!("Detecting memory information");
        
        // Read memory information from /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo")
            .context("Failed to read memory information")?;
        
        let mut total_memory = 0;
        let mut total_swap = 0;
        let mut page_size = 4; // Default page size is 4 KB
        let mut huge_page_size = 2048; // Default huge page size is 2 MB
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(mem) = parts[1].parse::<u64>() {
                        total_memory = (mem / 1024) as u32; // Convert to MB
                    }
                }
            } else if line.starts_with("SwapTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(swap) = parts[1].parse::<u64>() {
                        total_swap = (swap / 1024) as u32; // Convert to MB
                    }
                }
            } else if line.starts_with("Hugepagesize:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(size) = parts[1].parse::<u32>() {
                        huge_page_size = size;
                    }
                }
            }
        }
        
        // Get page size
        let output = Command::new("getconf")
            .arg("PAGE_SIZE")
            .output()
            .context("Failed to execute getconf command")?;
        
        if output.status.success() {
            let page_size_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(size) = page_size_str.parse::<u32>() {
                page_size = size / 1024; // Convert to KB
            }
        }
        
        // Check if huge pages are supported
        let supports_huge_pages = Path::new("/proc/sys/vm/nr_hugepages").exists();
        
        // Check if memory compression is supported
        let supports_compression = Path::new("/sys/module/zswap").exists() || Path::new("/sys/module/zram").exists();
        
        // Check if memory defragmentation is supported
        let supports_defragmentation = Path::new("/proc/sys/vm/compact_memory").exists();
        
        // Check if memory priority is supported
        let supports_priority = Path::new("/sys/fs/cgroup/memory").exists();
        
        Ok(MemoryInfo {
            total_memory,
            total_swap,
            page_size,
            huge_page_size,
            supports_huge_pages,
            supports_compression,
            supports_defragmentation,
            supports_priority,
        })
    }
    
    /// Get current memory optimization state.
    fn get_current_state(info: &MemoryInfo) -> Result<MemoryOptimizationState> {
        debug!("Getting current memory optimization state");
        
        // Read memory information from /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo")
            .context("Failed to read memory information")?;
        
        let mut free_memory = 0;
        let mut cached_memory = 0;
        let mut buffers_memory = 0;
        let mut free_swap = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("MemFree:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(mem) = parts[1].parse::<u64>() {
                        free_memory = (mem / 1024) as u32; // Convert to MB
                    }
                }
            } else if line.starts_with("Cached:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(mem) = parts[1].parse::<u64>() {
                        cached_memory = (mem / 1024) as u32; // Convert to MB
                    }
                }
            } else if line.starts_with("Buffers:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(mem) = parts[1].parse::<u64>() {
                        buffers_memory = (mem / 1024) as u32; // Convert to MB
                    }
                }
            } else if line.starts_with("SwapFree:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(swap) = parts[1].parse::<u64>() {
                        free_swap = (swap / 1024) as u32; // Convert to MB
                    }
                }
            }
        }
        
        let used_memory = info.total_memory.saturating_sub(free_memory);
        let used_swap = info.total_swap.saturating_sub(free_swap);
        
        // Read swappiness
        let swappiness = if let Ok(value) = fs::read_to_string("/proc/sys/vm/swappiness") {
            value.trim().parse::<u8>().unwrap_or(60)
        } else {
            60
        };
        
        // Read VFS cache pressure
        let vfs_cache_pressure = if let Ok(value) = fs::read_to_string("/proc/sys/vm/vfs_cache_pressure") {
            value.trim().parse::<u8>().unwrap_or(100)
        } else {
            100
        };
        
        // Read dirty ratio
        let dirty_ratio = if let Ok(value) = fs::read_to_string("/proc/sys/vm/dirty_ratio") {
            value.trim().parse::<u8>().unwrap_or(20)
        } else {
            20
        };
        
        // Read dirty background ratio
        let dirty_background_ratio = if let Ok(value) = fs::read_to_string("/proc/sys/vm/dirty_background_ratio") {
            value.trim().parse::<u8>().unwrap_or(10)
        } else {
            10
        };
        
        // Read number of huge pages
        let nr_hugepages = if let Ok(value) = fs::read_to_string("/proc/sys/vm/nr_hugepages") {
            value.trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };
        
        // Read number of allocated huge pages
        let nr_hugepages_allocated = if let Ok(value) = fs::read_to_string("/sys/devices/system/node/node0/hugepages/hugepages-2048kB/nr_hugepages") {
            value.trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };
        
        Ok(MemoryOptimizationState {
            free_memory,
            used_memory,
            cached_memory,
            buffers_memory,
            free_swap,
            used_swap,
            swappiness,
            vfs_cache_pressure,
            dirty_ratio,
            dirty_background_ratio,
            nr_hugepages,
            nr_hugepages_allocated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_optimization_settings_default() {
        let settings = MemoryOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert!(settings.optimize_allocation);
        assert!(settings.optimize_paging);
        assert!(settings.optimize_swapping);
        assert!(settings.optimize_huge_pages);
        assert!(settings.optimize_compression);
        assert!(settings.optimize_defragmentation);
        assert!(settings.optimize_priority);
        assert_eq!(settings.swappiness, 10);
        assert_eq!(settings.vfs_cache_pressure, 50);
        assert_eq!(settings.dirty_ratio, 20);
        assert_eq!(settings.dirty_background_ratio, 10);
        assert_eq!(settings.min_free_memory, 1024);
        assert_eq!(settings.max_swap_usage, 4096);
        assert_eq!(settings.nr_hugepages, 128);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 1000);
    }
}
