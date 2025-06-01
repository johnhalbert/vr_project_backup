//! GPU optimization module for the VR headset system.
//!
//! This module provides GPU optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages GPU frequency scaling,
//! memory allocation, rendering parameters, and other GPU-related optimizations
//! to maximize performance for VR workloads.

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

/// GPU optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct GpuOptimizationManager {
    /// GPU optimization settings
    settings: GpuOptimizationSettings,
    
    /// GPU information
    info: GpuInfo,
    
    /// Current GPU optimization state
    state: GpuOptimizationState,
    
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
    
    /// Current GPU optimization settings
    settings: GpuOptimizationSettings,
}

/// GPU optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuOptimizationSettings {
    /// Whether GPU optimization is enabled
    pub enabled: bool,
    
    /// GPU governor to use
    pub governor: GpuGovernor,
    
    /// Maximum GPU frequency (in Hz)
    pub max_freq: u32,
    
    /// Minimum GPU frequency (in Hz)
    pub min_freq: u32,
    
    /// Whether to optimize GPU memory allocation
    pub optimize_memory: bool,
    
    /// Maximum GPU memory to allocate (in MB)
    pub max_memory: u32,
    
    /// Whether to optimize rendering parameters
    pub optimize_rendering: bool,
    
    /// Rendering quality level
    pub rendering_quality: RenderingQuality,
    
    /// Whether to enable GPU power management
    pub power_management: bool,
    
    /// Whether to enable thermal management
    pub thermal_management: bool,
    
    /// Maximum temperature before throttling (in Celsius)
    pub max_temperature: u8,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// GPU governor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuGovernor {
    /// Performance governor (maximum frequency)
    Performance,
    
    /// Powersave governor (minimum frequency)
    Powersave,
    
    /// Simple governor (fixed frequency)
    Simple,
    
    /// Ondemand governor (dynamic frequency scaling)
    Ondemand,
    
    /// Userspace governor (user-defined frequency)
    Userspace,
}

impl GpuGovernor {
    /// Convert GPU governor to string.
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Powersave => "powersave",
            Self::Simple => "simple",
            Self::Ondemand => "ondemand",
            Self::Userspace => "userspace",
        }
    }
    
    /// Parse GPU governor from string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "performance" => Ok(Self::Performance),
            "powersave" => Ok(Self::Powersave),
            "simple" => Ok(Self::Simple),
            "ondemand" => Ok(Self::Ondemand),
            "userspace" => Ok(Self::Userspace),
            _ => bail!("Unknown GPU governor: {}", s),
        }
    }
}

/// Rendering quality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingQuality {
    /// Low quality (maximum performance)
    Low,
    
    /// Medium quality (balanced)
    Medium,
    
    /// High quality (maximum quality)
    High,
    
    /// Ultra quality (maximum quality, no compromises)
    Ultra,
    
    /// Custom quality (user-defined)
    Custom,
}

/// GPU information.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// GPU model
    pub model: String,
    
    /// GPU vendor
    pub vendor: String,
    
    /// GPU driver version
    pub driver_version: String,
    
    /// Available GPU governors
    pub available_governors: Vec<GpuGovernor>,
    
    /// Available GPU frequencies (in Hz)
    pub available_frequencies: Vec<u32>,
    
    /// Maximum GPU frequency (in Hz)
    pub max_freq: u32,
    
    /// Minimum GPU frequency (in Hz)
    pub min_freq: u32,
    
    /// Total GPU memory (in MB)
    pub total_memory: u32,
    
    /// Whether the GPU supports frequency scaling
    pub supports_freq_scaling: bool,
    
    /// Whether the GPU supports memory management
    pub supports_memory_management: bool,
    
    /// Whether the GPU supports power management
    pub supports_power_management: bool,
    
    /// Whether the GPU supports thermal management
    pub supports_thermal_management: bool,
}

/// GPU optimization state.
#[derive(Debug, Clone)]
pub struct GpuOptimizationState {
    /// Current GPU governor
    pub governor: GpuGovernor,
    
    /// Current GPU frequency (in Hz)
    pub frequency: u32,
    
    /// Current GPU utilization (in percent)
    pub utilization: u8,
    
    /// Current GPU temperature (in Celsius)
    pub temperature: u8,
    
    /// Current GPU memory usage (in MB)
    pub memory_usage: u32,
    
    /// Current GPU power usage (in mW)
    pub power_usage: u32,
    
    /// Current rendering quality
    pub rendering_quality: RenderingQuality,
}

impl Default for GpuOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            governor: GpuGovernor::Performance,
            max_freq: 800000000, // 800 MHz
            min_freq: 200000000, // 200 MHz
            optimize_memory: true,
            max_memory: 4096, // 4 GB
            optimize_rendering: true,
            rendering_quality: RenderingQuality::Medium,
            power_management: true,
            thermal_management: true,
            max_temperature: 85,
            adaptive: true,
            adaptive_interval_ms: 1000,
        }
    }
}

impl GpuOptimizationManager {
    /// Create a new GPU optimization manager.
    pub fn new() -> Result<Self> {
        let info = Self::detect_gpu_info()?;
        let settings = GpuOptimizationSettings::default();
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
    
    /// Initialize GPU optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing GPU optimization for Orange Pi CM5");
        
        // Detect GPU information
        self.info = Self::detect_gpu_info()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("GPU optimization initialized successfully");
        Ok(())
    }
    
    /// Apply GPU optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying GPU optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply GPU governor and frequency settings
        self.apply_governor_and_frequency_settings()?;
        
        // Apply memory optimization if enabled
        if self.settings.optimize_memory {
            self.apply_memory_optimization()?;
        }
        
        // Apply rendering optimization if enabled
        if self.settings.optimize_rendering {
            self.apply_rendering_optimization()?;
        }
        
        // Apply power management if enabled
        if self.settings.power_management {
            self.apply_power_management()?;
        }
        
        // Apply thermal management if enabled
        if self.settings.thermal_management {
            self.apply_thermal_management()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("GPU optimizations applied successfully");
        Ok(())
    }
    
    /// Reset GPU optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting GPU optimizations");
        
        // Reset GPU governor to default
        self.set_gpu_governor(GpuGovernor::Simple)?;
        
        // Reset GPU frequency to default
        self.set_gpu_max_freq(self.info.max_freq)?;
        self.set_gpu_min_freq(self.info.min_freq)?;
        
        // Reset memory optimization
        self.reset_memory_optimization()?;
        
        // Reset rendering optimization
        self.reset_rendering_optimization()?;
        
        // Reset power management
        self.reset_power_management()?;
        
        info!("GPU optimizations reset successfully");
        Ok(())
    }
    
    /// Update GPU optimization settings.
    pub fn update_settings(&mut self, settings: GpuOptimizationSettings) -> Result<()> {
        info!("Updating GPU optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("GPU optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current GPU optimization settings.
    pub fn get_settings(&self) -> GpuOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current GPU optimization state.
    pub fn get_state(&self) -> GpuOptimizationState {
        self.state.clone()
    }
    
    /// Get GPU information.
    pub fn get_info(&self) -> GpuInfo {
        self.info.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background GPU optimization thread");
        
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
        
        info!("Background GPU optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background GPU optimization thread");
        
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
        
        info!("Background GPU optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive optimization.
    fn perform_adaptive_optimization(info: &GpuInfo, settings: &GpuOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive GPU optimization");
        
        // Get current GPU state
        let state = Self::get_current_state(info)?;
        
        // Check GPU temperature and adjust frequency if necessary
        if settings.thermal_management {
            if state.temperature > settings.max_temperature {
                // Reduce frequency if temperature is too high
                let current_freq = state.frequency;
                let new_freq = (current_freq as f32 * 0.9) as u32;
                
                if new_freq >= info.min_freq {
                    if let Err(e) = Self::set_gpu_max_freq_static(new_freq) {
                        warn!("Error setting GPU frequency: {}", e);
                    }
                }
            }
        }
        
        // Check GPU utilization and adjust governor if necessary
        let governor = if state.utilization > 80 {
            GpuGovernor::Performance
        } else if state.utilization < 20 {
            GpuGovernor::Powersave
        } else {
            GpuGovernor::Ondemand
        };
        
        if governor != state.governor {
            if let Err(e) = Self::set_gpu_governor_static(governor) {
                warn!("Error setting GPU governor: {}", e);
            }
        }
        
        debug!("Adaptive GPU optimization completed");
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
                self.settings.governor = GpuGovernor::Performance;
                self.settings.max_freq = self.info.max_freq;
                self.settings.min_freq = self.info.max_freq;
                self.settings.optimize_memory = true;
                self.settings.max_memory = self.info.total_memory;
                self.settings.optimize_rendering = true;
                self.settings.rendering_quality = RenderingQuality::Medium;
                self.settings.power_management = false;
                self.settings.thermal_management = false;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.governor = GpuGovernor::Ondemand;
                self.settings.max_freq = self.info.max_freq;
                self.settings.min_freq = self.info.min_freq;
                self.settings.optimize_memory = true;
                self.settings.max_memory = self.info.total_memory / 2;
                self.settings.optimize_rendering = true;
                self.settings.rendering_quality = RenderingQuality::Low;
                self.settings.power_management = true;
                self.settings.thermal_management = true;
            },
            super::OptimizationMode::Latency => {
                self.settings.governor = GpuGovernor::Performance;
                self.settings.max_freq = self.info.max_freq;
                self.settings.min_freq = self.info.max_freq;
                self.settings.optimize_memory = true;
                self.settings.max_memory = self.info.total_memory;
                self.settings.optimize_rendering = true;
                self.settings.rendering_quality = RenderingQuality::Low;
                self.settings.power_management = false;
                self.settings.thermal_management = false;
            },
            super::OptimizationMode::Thermal => {
                self.settings.governor = GpuGovernor::Ondemand;
                self.settings.max_freq = self.info.max_freq * 9 / 10;
                self.settings.min_freq = self.info.min_freq;
                self.settings.optimize_memory = true;
                self.settings.max_memory = self.info.total_memory * 3 / 4;
                self.settings.optimize_rendering = true;
                self.settings.rendering_quality = RenderingQuality::Low;
                self.settings.power_management = true;
                self.settings.thermal_management = true;
                self.settings.max_temperature = 75;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.governor = GpuGovernor::Performance;
            self.settings.max_freq = self.info.max_freq;
            self.settings.min_freq = self.info.max_freq;
            self.settings.optimize_memory = true;
            self.settings.max_memory = self.info.total_memory;
            self.settings.optimize_rendering = true;
            self.settings.rendering_quality = RenderingQuality::Low;
        }
    }
    
    /// Apply GPU governor and frequency settings.
    fn apply_governor_and_frequency_settings(&self) -> Result<()> {
        info!("Applying GPU governor and frequency settings");
        
        // Set GPU governor
        self.set_gpu_governor(self.settings.governor)?;
        
        // Set GPU frequency limits
        self.set_gpu_max_freq(self.settings.max_freq)?;
        self.set_gpu_min_freq(self.settings.min_freq)?;
        
        Ok(())
    }
    
    /// Apply memory optimization.
    fn apply_memory_optimization(&self) -> Result<()> {
        info!("Applying GPU memory optimization");
        
        // Set maximum GPU memory
        let mem_path = Path::new("/sys/class/devfreq/mali/max_mem");
        if mem_path.exists() {
            fs::write(mem_path, self.settings.max_memory.to_string())
                .context("Failed to set maximum GPU memory")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_mem_path = Path::new("/sys/class/misc/mali0/device/mem_pool_max_size");
            if alt_mem_path.exists() {
                fs::write(alt_mem_path, (self.settings.max_memory * 1024 * 1024).to_string())
                    .context("Failed to set maximum GPU memory")?;
            } else {
                warn!("GPU memory optimization not supported");
            }
        }
        
        // Set memory management policy
        let policy_path = Path::new("/sys/class/devfreq/mali/mem_policy");
        if policy_path.exists() {
            fs::write(policy_path, "demand")
                .context("Failed to set GPU memory policy")?;
        }
        
        Ok(())
    }
    
    /// Reset memory optimization.
    fn reset_memory_optimization(&self) -> Result<()> {
        info!("Resetting GPU memory optimization");
        
        // Reset maximum GPU memory
        let mem_path = Path::new("/sys/class/devfreq/mali/max_mem");
        if mem_path.exists() {
            fs::write(mem_path, self.info.total_memory.to_string())
                .context("Failed to reset maximum GPU memory")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_mem_path = Path::new("/sys/class/misc/mali0/device/mem_pool_max_size");
            if alt_mem_path.exists() {
                fs::write(alt_mem_path, (self.info.total_memory * 1024 * 1024).to_string())
                    .context("Failed to reset maximum GPU memory")?;
            }
        }
        
        // Reset memory management policy
        let policy_path = Path::new("/sys/class/devfreq/mali/mem_policy");
        if policy_path.exists() {
            fs::write(policy_path, "default")
                .context("Failed to reset GPU memory policy")?;
        }
        
        Ok(())
    }
    
    /// Apply rendering optimization.
    fn apply_rendering_optimization(&self) -> Result<()> {
        info!("Applying GPU rendering optimization");
        
        // Set rendering quality parameters
        let quality = match self.settings.rendering_quality {
            RenderingQuality::Low => {
                // Low quality settings
                let params = [
                    ("msaa", "0"),
                    ("aniso", "0"),
                    ("trilinear", "0"),
                    ("vsync", "0"),
                    ("texture_quality", "low"),
                    ("shader_quality", "low"),
                    ("shadow_quality", "low"),
                    ("effect_quality", "low"),
                ];
                
                params
            },
            RenderingQuality::Medium => {
                // Medium quality settings
                let params = [
                    ("msaa", "2"),
                    ("aniso", "4"),
                    ("trilinear", "1"),
                    ("vsync", "1"),
                    ("texture_quality", "medium"),
                    ("shader_quality", "medium"),
                    ("shadow_quality", "medium"),
                    ("effect_quality", "medium"),
                ];
                
                params
            },
            RenderingQuality::High => {
                // High quality settings
                let params = [
                    ("msaa", "4"),
                    ("aniso", "8"),
                    ("trilinear", "1"),
                    ("vsync", "1"),
                    ("texture_quality", "high"),
                    ("shader_quality", "high"),
                    ("shadow_quality", "high"),
                    ("effect_quality", "high"),
                ];
                
                params
            },
            RenderingQuality::Ultra => {
                // Ultra quality settings
                let params = [
                    ("msaa", "8"),
                    ("aniso", "16"),
                    ("trilinear", "1"),
                    ("vsync", "1"),
                    ("texture_quality", "ultra"),
                    ("shader_quality", "ultra"),
                    ("shadow_quality", "ultra"),
                    ("effect_quality", "ultra"),
                ];
                
                params
            },
            RenderingQuality::Custom => {
                // Custom quality settings (not implemented)
                let params = [
                    ("msaa", "2"),
                    ("aniso", "4"),
                    ("trilinear", "1"),
                    ("vsync", "1"),
                    ("texture_quality", "medium"),
                    ("shader_quality", "medium"),
                    ("shadow_quality", "medium"),
                    ("effect_quality", "medium"),
                ];
                
                params
            },
        };
        
        // Apply rendering quality parameters
        let config_dir = Path::new("/etc/vr/rendering");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .context("Failed to create rendering configuration directory")?;
        }
        
        let config_path = config_dir.join("quality.conf");
        let mut config = String::new();
        
        for (key, value) in quality {
            config.push_str(&format!("{}={}\n", key, value));
        }
        
        fs::write(&config_path, config)
            .context("Failed to write rendering configuration")?;
        
        Ok(())
    }
    
    /// Reset rendering optimization.
    fn reset_rendering_optimization(&self) -> Result<()> {
        info!("Resetting GPU rendering optimization");
        
        // Set default rendering quality parameters
        let params = [
            ("msaa", "2"),
            ("aniso", "4"),
            ("trilinear", "1"),
            ("vsync", "1"),
            ("texture_quality", "medium"),
            ("shader_quality", "medium"),
            ("shadow_quality", "medium"),
            ("effect_quality", "medium"),
        ];
        
        // Apply default rendering quality parameters
        let config_dir = Path::new("/etc/vr/rendering");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .context("Failed to create rendering configuration directory")?;
        }
        
        let config_path = config_dir.join("quality.conf");
        let mut config = String::new();
        
        for (key, value) in params {
            config.push_str(&format!("{}={}\n", key, value));
        }
        
        fs::write(&config_path, config)
            .context("Failed to write rendering configuration")?;
        
        Ok(())
    }
    
    /// Apply power management.
    fn apply_power_management(&self) -> Result<()> {
        info!("Applying GPU power management");
        
        // Set power management policy
        let policy_path = Path::new("/sys/class/devfreq/mali/power_policy");
        if policy_path.exists() {
            fs::write(policy_path, "coarse_demand")
                .context("Failed to set GPU power policy")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_policy_path = Path::new("/sys/class/misc/mali0/device/power_policy");
            if alt_policy_path.exists() {
                fs::write(alt_policy_path, "coarse_demand")
                    .context("Failed to set GPU power policy")?;
            } else {
                warn!("GPU power management not supported");
            }
        }
        
        // Set power management parameters
        let params_path = Path::new("/sys/class/devfreq/mali/power_params");
        if params_path.exists() {
            fs::write(params_path, "adaptive")
                .context("Failed to set GPU power parameters")?;
        }
        
        Ok(())
    }
    
    /// Reset power management.
    fn reset_power_management(&self) -> Result<()> {
        info!("Resetting GPU power management");
        
        // Reset power management policy
        let policy_path = Path::new("/sys/class/devfreq/mali/power_policy");
        if policy_path.exists() {
            fs::write(policy_path, "always_on")
                .context("Failed to reset GPU power policy")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_policy_path = Path::new("/sys/class/misc/mali0/device/power_policy");
            if alt_policy_path.exists() {
                fs::write(alt_policy_path, "always_on")
                    .context("Failed to reset GPU power policy")?;
            }
        }
        
        // Reset power management parameters
        let params_path = Path::new("/sys/class/devfreq/mali/power_params");
        if params_path.exists() {
            fs::write(params_path, "default")
                .context("Failed to reset GPU power parameters")?;
        }
        
        Ok(())
    }
    
    /// Apply thermal management.
    fn apply_thermal_management(&self) -> Result<()> {
        info!("Applying GPU thermal management");
        
        // Set thermal trip points
        let thermal_dir = Path::new("/sys/class/thermal");
        if thermal_dir.exists() && thermal_dir.is_dir() {
            for entry in fs::read_dir(thermal_dir)
                .context("Failed to read thermal directory")? {
                let entry = entry.context("Failed to read thermal directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(zone) = path.file_name()
                        .and_then(|n| n.to_str())
                        .filter(|s| s.starts_with("thermal_zone")) {
                        
                        // Check if this thermal zone is for GPU
                        let type_path = path.join("type");
                        if type_path.exists() {
                            let zone_type = fs::read_to_string(&type_path)
                                .unwrap_or_else(|_| "unknown".to_string())
                                .trim()
                                .to_string();
                            
                            if zone_type.contains("gpu") || zone_type.contains("mali") {
                                // Set trip point temperature
                                let trip_point_path = path.join("trip_point_0_temp");
                                if trip_point_path.exists() {
                                    fs::write(&trip_point_path, (self.settings.max_temperature * 1000).to_string())
                                        .unwrap_or_else(|e| warn!("Failed to set trip point temperature for {}: {}", zone, e));
                                }
                                
                                // Set trip point type
                                let trip_type_path = path.join("trip_point_0_type");
                                if trip_type_path.exists() {
                                    fs::write(&trip_type_path, "passive")
                                        .unwrap_or_else(|e| warn!("Failed to set trip point type for {}: {}", zone, e));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Set GPU governor.
    fn set_gpu_governor(&self, governor: GpuGovernor) -> Result<()> {
        debug!("Setting GPU governor to {}", governor.to_str());
        
        let governor_path = Path::new("/sys/class/devfreq/mali/governor");
        if governor_path.exists() {
            fs::write(&governor_path, governor.to_str())
                .context("Failed to set GPU governor")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_governor_path = Path::new("/sys/class/misc/mali0/device/devfreq/governor");
            if alt_governor_path.exists() {
                fs::write(&alt_governor_path, governor.to_str())
                    .context("Failed to set GPU governor")?;
            } else {
                warn!("GPU governor setting not supported");
            }
        }
        
        Ok(())
    }
    
    /// Set GPU governor (static method).
    fn set_gpu_governor_static(governor: GpuGovernor) -> Result<()> {
        debug!("Setting GPU governor to {}", governor.to_str());
        
        let governor_path = Path::new("/sys/class/devfreq/mali/governor");
        if governor_path.exists() {
            fs::write(&governor_path, governor.to_str())
                .context("Failed to set GPU governor")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_governor_path = Path::new("/sys/class/misc/mali0/device/devfreq/governor");
            if alt_governor_path.exists() {
                fs::write(&alt_governor_path, governor.to_str())
                    .context("Failed to set GPU governor")?;
            } else {
                warn!("GPU governor setting not supported");
            }
        }
        
        Ok(())
    }
    
    /// Set GPU maximum frequency.
    fn set_gpu_max_freq(&self, freq: u32) -> Result<()> {
        debug!("Setting GPU maximum frequency to {} Hz", freq);
        
        let freq_path = Path::new("/sys/class/devfreq/mali/max_freq");
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context("Failed to set GPU maximum frequency")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_freq_path = Path::new("/sys/class/misc/mali0/device/devfreq/max_freq");
            if alt_freq_path.exists() {
                fs::write(&alt_freq_path, freq.to_string())
                    .context("Failed to set GPU maximum frequency")?;
            } else {
                warn!("GPU frequency setting not supported");
            }
        }
        
        Ok(())
    }
    
    /// Set GPU maximum frequency (static method).
    fn set_gpu_max_freq_static(freq: u32) -> Result<()> {
        debug!("Setting GPU maximum frequency to {} Hz", freq);
        
        let freq_path = Path::new("/sys/class/devfreq/mali/max_freq");
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context("Failed to set GPU maximum frequency")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_freq_path = Path::new("/sys/class/misc/mali0/device/devfreq/max_freq");
            if alt_freq_path.exists() {
                fs::write(&alt_freq_path, freq.to_string())
                    .context("Failed to set GPU maximum frequency")?;
            } else {
                warn!("GPU frequency setting not supported");
            }
        }
        
        Ok(())
    }
    
    /// Set GPU minimum frequency.
    fn set_gpu_min_freq(&self, freq: u32) -> Result<()> {
        debug!("Setting GPU minimum frequency to {} Hz", freq);
        
        let freq_path = Path::new("/sys/class/devfreq/mali/min_freq");
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context("Failed to set GPU minimum frequency")?;
        } else {
            // Try alternative path for Mali GPU
            let alt_freq_path = Path::new("/sys/class/misc/mali0/device/devfreq/min_freq");
            if alt_freq_path.exists() {
                fs::write(&alt_freq_path, freq.to_string())
                    .context("Failed to set GPU minimum frequency")?;
            } else {
                warn!("GPU frequency setting not supported");
            }
        }
        
        Ok(())
    }
    
    /// Detect GPU information.
    fn detect_gpu_info() -> Result<GpuInfo> {
        info!("Detecting GPU information");
        
        // Default values for Orange Pi CM5 (Mali-G610 MC4)
        let mut model = "Mali-G610 MC4".to_string();
        let mut vendor = "ARM".to_string();
        let mut driver_version = "Unknown".to_string();
        let mut available_governors = vec![
            GpuGovernor::Performance,
            GpuGovernor::Simple,
            GpuGovernor::Powersave,
        ];
        let mut available_frequencies = vec![
            200000000, // 200 MHz
            300000000, // 300 MHz
            400000000, // 400 MHz
            500000000, // 500 MHz
            600000000, // 600 MHz
            700000000, // 700 MHz
            800000000, // 800 MHz
        ];
        let mut max_freq = 800000000; // 800 MHz
        let mut min_freq = 200000000; // 200 MHz
        let mut total_memory = 4096; // 4 GB
        let mut supports_freq_scaling = true;
        let mut supports_memory_management = true;
        let mut supports_power_management = true;
        let mut supports_thermal_management = true;
        
        // Try to get GPU information from sysfs
        let mali_dir = Path::new("/sys/class/misc/mali0");
        if mali_dir.exists() && mali_dir.is_dir() {
            // Get GPU model
            let model_path = mali_dir.join("device/modalias");
            if model_path.exists() {
                let model_str = fs::read_to_string(&model_path)
                    .unwrap_or_else(|_| "unknown".to_string())
                    .trim()
                    .to_string();
                
                if model_str.contains("mali") {
                    model = model_str;
                }
            }
            
            // Get GPU driver version
            let version_path = mali_dir.join("device/driver/module/version");
            if version_path.exists() {
                driver_version = fs::read_to_string(&version_path)
                    .unwrap_or_else(|_| "Unknown".to_string())
                    .trim()
                    .to_string();
            }
            
            // Get GPU memory
            let mem_path = mali_dir.join("device/mem_pool_size");
            if mem_path.exists() {
                let mem_str = fs::read_to_string(&mem_path)
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .to_string();
                
                if let Ok(mem) = mem_str.parse::<u64>() {
                    total_memory = (mem / (1024 * 1024)) as u32; // Convert to MB
                }
            }
        }
        
        // Try to get GPU frequency information from devfreq
        let devfreq_dir = Path::new("/sys/class/devfreq/mali");
        if devfreq_dir.exists() && devfreq_dir.is_dir() {
            // Get available frequencies
            let avail_freq_path = devfreq_dir.join("available_frequencies");
            if avail_freq_path.exists() {
                let freq_str = fs::read_to_string(&avail_freq_path)
                    .unwrap_or_else(|_| "".to_string())
                    .trim()
                    .to_string();
                
                if !freq_str.is_empty() {
                    let mut freqs = Vec::new();
                    
                    for freq in freq_str.split_whitespace() {
                        if let Ok(f) = freq.parse::<u32>() {
                            freqs.push(f);
                        }
                    }
                    
                    if !freqs.is_empty() {
                        available_frequencies = freqs;
                        max_freq = *available_frequencies.iter().max().unwrap_or(&max_freq);
                        min_freq = *available_frequencies.iter().min().unwrap_or(&min_freq);
                    }
                }
            }
            
            // Get available governors
            let avail_gov_path = devfreq_dir.join("available_governors");
            if avail_gov_path.exists() {
                let gov_str = fs::read_to_string(&avail_gov_path)
                    .unwrap_or_else(|_| "".to_string())
                    .trim()
                    .to_string();
                
                if !gov_str.is_empty() {
                    let mut govs = Vec::new();
                    
                    for gov in gov_str.split_whitespace() {
                        if let Ok(g) = GpuGovernor::from_str(gov) {
                            govs.push(g);
                        }
                    }
                    
                    if !govs.is_empty() {
                        available_governors = govs;
                    }
                }
            }
            
            // Check if frequency scaling is supported
            supports_freq_scaling = devfreq_dir.join("max_freq").exists() && devfreq_dir.join("min_freq").exists();
            
            // Check if memory management is supported
            supports_memory_management = devfreq_dir.join("max_mem").exists() || mali_dir.join("device/mem_pool_max_size").exists();
            
            // Check if power management is supported
            supports_power_management = devfreq_dir.join("power_policy").exists() || mali_dir.join("device/power_policy").exists();
        }
        
        // Check if thermal management is supported
        let thermal_dir = Path::new("/sys/class/thermal");
        if thermal_dir.exists() && thermal_dir.is_dir() {
            for entry in fs::read_dir(thermal_dir)
                .unwrap_or_else(|_| fs::read_dir(Path::new("/")).unwrap()) {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        if let Some(zone) = path.file_name()
                            .and_then(|n| n.to_str())
                            .filter(|s| s.starts_with("thermal_zone")) {
                            
                            // Check if this thermal zone is for GPU
                            let type_path = path.join("type");
                            if type_path.exists() {
                                let zone_type = fs::read_to_string(&type_path)
                                    .unwrap_or_else(|_| "unknown".to_string())
                                    .trim()
                                    .to_string();
                                
                                if zone_type.contains("gpu") || zone_type.contains("mali") {
                                    supports_thermal_management = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(GpuInfo {
            model,
            vendor,
            driver_version,
            available_governors,
            available_frequencies,
            max_freq,
            min_freq,
            total_memory,
            supports_freq_scaling,
            supports_memory_management,
            supports_power_management,
            supports_thermal_management,
        })
    }
    
    /// Get current GPU optimization state.
    fn get_current_state(info: &GpuInfo) -> Result<GpuOptimizationState> {
        debug!("Getting current GPU optimization state");
        
        // Default values
        let mut governor = GpuGovernor::Simple;
        let mut frequency = 0;
        let mut utilization = 0;
        let mut temperature = 0;
        let mut memory_usage = 0;
        let mut power_usage = 0;
        let mut rendering_quality = RenderingQuality::Medium;
        
        // Try to get GPU governor
        let governor_path = Path::new("/sys/class/devfreq/mali/governor");
        if governor_path.exists() {
            let governor_str = fs::read_to_string(&governor_path)
                .unwrap_or_else(|_| "unknown".to_string())
                .trim()
                .to_string();
            
            governor = GpuGovernor::from_str(&governor_str).unwrap_or(GpuGovernor::Simple);
        } else {
            // Try alternative path for Mali GPU
            let alt_governor_path = Path::new("/sys/class/misc/mali0/device/devfreq/governor");
            if alt_governor_path.exists() {
                let governor_str = fs::read_to_string(&alt_governor_path)
                    .unwrap_or_else(|_| "unknown".to_string())
                    .trim()
                    .to_string();
                
                governor = GpuGovernor::from_str(&governor_str).unwrap_or(GpuGovernor::Simple);
            }
        }
        
        // Try to get GPU frequency
        let freq_path = Path::new("/sys/class/devfreq/mali/cur_freq");
        if freq_path.exists() {
            let freq_str = fs::read_to_string(&freq_path)
                .unwrap_or_else(|_| "0".to_string())
                .trim()
                .to_string();
            
            frequency = freq_str.parse::<u32>().unwrap_or(0);
        } else {
            // Try alternative path for Mali GPU
            let alt_freq_path = Path::new("/sys/class/misc/mali0/device/devfreq/cur_freq");
            if alt_freq_path.exists() {
                let freq_str = fs::read_to_string(&alt_freq_path)
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .to_string();
                
                frequency = freq_str.parse::<u32>().unwrap_or(0);
            }
        }
        
        // Try to get GPU utilization
        let util_path = Path::new("/sys/class/devfreq/mali/utilization");
        if util_path.exists() {
            let util_str = fs::read_to_string(&util_path)
                .unwrap_or_else(|_| "0".to_string())
                .trim()
                .to_string();
            
            utilization = util_str.parse::<u8>().unwrap_or(0);
        } else {
            // Try alternative path for Mali GPU
            let alt_util_path = Path::new("/sys/class/misc/mali0/device/utilization");
            if alt_util_path.exists() {
                let util_str = fs::read_to_string(&alt_util_path)
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .to_string();
                
                utilization = util_str.parse::<u8>().unwrap_or(0);
            }
        }
        
        // Try to get GPU temperature
        let thermal_dir = Path::new("/sys/class/thermal");
        if thermal_dir.exists() && thermal_dir.is_dir() {
            for entry in fs::read_dir(thermal_dir)
                .unwrap_or_else(|_| fs::read_dir(Path::new("/")).unwrap()) {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        if let Some(zone) = path.file_name()
                            .and_then(|n| n.to_str())
                            .filter(|s| s.starts_with("thermal_zone")) {
                            
                            // Check if this thermal zone is for GPU
                            let type_path = path.join("type");
                            if type_path.exists() {
                                let zone_type = fs::read_to_string(&type_path)
                                    .unwrap_or_else(|_| "unknown".to_string())
                                    .trim()
                                    .to_string();
                                
                                if zone_type.contains("gpu") || zone_type.contains("mali") {
                                    // This is a GPU thermal zone, read temperature
                                    let temp_path = path.join("temp");
                                    if temp_path.exists() {
                                        let temp = fs::read_to_string(&temp_path)
                                            .unwrap_or_else(|_| "0".to_string())
                                            .trim()
                                            .parse::<u32>()
                                            .unwrap_or(0);
                                        
                                        // Temperature is in millidegrees Celsius, convert to degrees
                                        temperature = (temp / 1000) as u8;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Try to get GPU memory usage
        let mem_path = Path::new("/sys/class/devfreq/mali/mem_usage");
        if mem_path.exists() {
            let mem_str = fs::read_to_string(&mem_path)
                .unwrap_or_else(|_| "0".to_string())
                .trim()
                .to_string();
            
            memory_usage = mem_str.parse::<u32>().unwrap_or(0);
        } else {
            // Try alternative path for Mali GPU
            let alt_mem_path = Path::new("/sys/class/misc/mali0/device/mem_used");
            if alt_mem_path.exists() {
                let mem_str = fs::read_to_string(&alt_mem_path)
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .to_string();
                
                let mem = mem_str.parse::<u64>().unwrap_or(0);
                memory_usage = (mem / (1024 * 1024)) as u32; // Convert to MB
            }
        }
        
        // Try to get GPU power usage
        let power_path = Path::new("/sys/class/devfreq/mali/power");
        if power_path.exists() {
            let power_str = fs::read_to_string(&power_path)
                .unwrap_or_else(|_| "0".to_string())
                .trim()
                .to_string();
            
            power_usage = power_str.parse::<u32>().unwrap_or(0);
        }
        
        // Try to get rendering quality
        let config_path = Path::new("/etc/vr/rendering/quality.conf");
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)
                .unwrap_or_else(|_| "".to_string());
            
            // Parse rendering quality from configuration
            let mut msaa = 0;
            let mut aniso = 0;
            let mut texture_quality = "medium";
            
            for line in config.lines() {
                if line.starts_with("msaa=") {
                    msaa = line[5..].parse::<u32>().unwrap_or(0);
                } else if line.starts_with("aniso=") {
                    aniso = line[6..].parse::<u32>().unwrap_or(0);
                } else if line.starts_with("texture_quality=") {
                    texture_quality = &line[16..];
                }
            }
            
            // Determine rendering quality based on parameters
            rendering_quality = if msaa <= 0 && aniso <= 0 && texture_quality == "low" {
                RenderingQuality::Low
            } else if msaa <= 2 && aniso <= 4 && texture_quality == "medium" {
                RenderingQuality::Medium
            } else if msaa <= 4 && aniso <= 8 && texture_quality == "high" {
                RenderingQuality::High
            } else if msaa > 4 && aniso > 8 && texture_quality == "ultra" {
                RenderingQuality::Ultra
            } else {
                RenderingQuality::Custom
            };
        }
        
        Ok(GpuOptimizationState {
            governor,
            frequency,
            utilization,
            temperature,
            memory_usage,
            power_usage,
            rendering_quality,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_governor_conversion() {
        assert_eq!(GpuGovernor::Performance.to_str(), "performance");
        assert_eq!(GpuGovernor::Powersave.to_str(), "powersave");
        assert_eq!(GpuGovernor::Simple.to_str(), "simple");
        assert_eq!(GpuGovernor::Ondemand.to_str(), "ondemand");
        assert_eq!(GpuGovernor::Userspace.to_str(), "userspace");
        
        assert_eq!(GpuGovernor::from_str("performance").unwrap(), GpuGovernor::Performance);
        assert_eq!(GpuGovernor::from_str("powersave").unwrap(), GpuGovernor::Powersave);
        assert_eq!(GpuGovernor::from_str("simple").unwrap(), GpuGovernor::Simple);
        assert_eq!(GpuGovernor::from_str("ondemand").unwrap(), GpuGovernor::Ondemand);
        assert_eq!(GpuGovernor::from_str("userspace").unwrap(), GpuGovernor::Userspace);
        
        assert!(GpuGovernor::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_gpu_optimization_settings_default() {
        let settings = GpuOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert_eq!(settings.governor, GpuGovernor::Performance);
        assert_eq!(settings.max_freq, 800000000);
        assert_eq!(settings.min_freq, 200000000);
        assert!(settings.optimize_memory);
        assert_eq!(settings.max_memory, 4096);
        assert!(settings.optimize_rendering);
        assert_eq!(settings.rendering_quality, RenderingQuality::Medium);
        assert!(settings.power_management);
        assert!(settings.thermal_management);
        assert_eq!(settings.max_temperature, 85);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 1000);
    }
}
