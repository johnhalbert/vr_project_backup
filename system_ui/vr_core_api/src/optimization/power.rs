//! Power optimization module for the VR headset system.
//!
//! This module provides power optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages CPU frequency scaling,
//! peripheral power states, and sleep modes to maximize battery life and manage thermals.

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
use crate::optimization::cpu::CpuGovernor;

/// Power optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct PowerOptimizationManager {
    /// Power optimization settings
    settings: PowerOptimizationSettings,
    
    /// Power information
    info: PowerInfo,
    
    /// Current power optimization state
    state: PowerOptimizationState,
    
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
    
    /// Current power optimization settings
    settings: PowerOptimizationSettings,
}

/// Power optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerOptimizationSettings {
    /// Whether power optimization is enabled
    pub enabled: bool,
    
    /// CPU governor for power saving
    pub cpu_governor: CpuGovernor,
    
    /// Minimum CPU frequency (in MHz)
    pub min_cpu_freq_mhz: u32,
    
    /// Maximum CPU frequency (in MHz)
    pub max_cpu_freq_mhz: u32,
    
    /// Whether to enable GPU power saving
    pub gpu_power_saving: bool,
    
    /// Whether to enable peripheral power management
    pub peripheral_power_management: bool,
    
    /// Whether to enable USB power saving
    pub usb_power_saving: bool,
    
    /// Whether to enable Wi-Fi power saving
    pub wifi_power_saving: bool,
    
    /// Whether to enable Bluetooth power saving
    pub bluetooth_power_saving: bool,
    
    /// Whether to enable display power saving
    pub display_power_saving: bool,
    
    /// Whether to enable audio power saving
    pub audio_power_saving: bool,
    
    /// Whether to enable storage power saving
    pub storage_power_saving: bool,
    
    /// Whether to enable dynamic power management
    pub dynamic_power_management: bool,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// Power information.
#[derive(Debug, Clone)]
pub struct PowerInfo {
    /// Available CPU governors
    pub available_cpu_governors: Vec<CpuGovernor>,
    
    /// Minimum CPU frequency (in MHz)
    pub min_cpu_freq_mhz: u32,
    
    /// Maximum CPU frequency (in MHz)
    pub max_cpu_freq_mhz: u32,
    
    /// Whether GPU power saving is supported
    pub supports_gpu_power_saving: bool,
    
    /// Whether peripheral power management is supported
    pub supports_peripheral_power_management: bool,
    
    /// Whether USB power saving is supported
    pub supports_usb_power_saving: bool,
    
    /// Whether Wi-Fi power saving is supported
    pub supports_wifi_power_saving: bool,
    
    /// Whether Bluetooth power saving is supported
    pub supports_bluetooth_power_saving: bool,
    
    /// Whether display power saving is supported
    pub supports_display_power_saving: bool,
    
    /// Whether audio power saving is supported
    pub supports_audio_power_saving: bool,
    
    /// Whether storage power saving is supported
    pub supports_storage_power_saving: bool,
}

/// Power optimization state.
#[derive(Debug, Clone)]
pub struct PowerOptimizationState {
    /// Current CPU governor
    pub cpu_governor: CpuGovernor,
    
    /// Current minimum CPU frequency (in MHz)
    pub min_cpu_freq_mhz: u32,
    
    /// Current maximum CPU frequency (in MHz)
    pub max_cpu_freq_mhz: u32,
    
    /// Whether GPU power saving is enabled
    pub gpu_power_saving_enabled: bool,
    
    /// Whether peripheral power management is enabled
    pub peripheral_power_management_enabled: bool,
    
    /// Whether USB power saving is enabled
    pub usb_power_saving_enabled: bool,
    
    /// Whether Wi-Fi power saving is enabled
    pub wifi_power_saving_enabled: bool,
    
    /// Whether Bluetooth power saving is enabled
    pub bluetooth_power_saving_enabled: bool,
    
    /// Whether display power saving is enabled
    pub display_power_saving_enabled: bool,
    
    /// Whether audio power saving is enabled
    pub audio_power_saving_enabled: bool,
    
    /// Whether storage power saving is enabled
    pub storage_power_saving_enabled: bool,
}

impl Default for PowerOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_governor: CpuGovernor::Schedutil,
            min_cpu_freq_mhz: 408, // Default min freq for RK3588S
            max_cpu_freq_mhz: 2400, // Default max freq for RK3588S
            gpu_power_saving: true,
            peripheral_power_management: true,
            usb_power_saving: true,
            wifi_power_saving: true,
            bluetooth_power_saving: true,
            display_power_saving: true,
            audio_power_saving: true,
            storage_power_saving: true,
            dynamic_power_management: true,
            adaptive: true,
            adaptive_interval_ms: 10000,
        }
    }
}

impl PowerOptimizationManager {
    /// Create a new power optimization manager.
    pub fn new() -> Result<Self> {
        let info = Self::detect_power_info()?;
        let settings = PowerOptimizationSettings::default();
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
    
    /// Initialize power optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing power optimization for Orange Pi CM5");
        
        // Detect power information
        self.info = Self::detect_power_info()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("Power optimization initialized successfully");
        Ok(())
    }
    
    /// Apply power optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying power optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply CPU power optimizations
        self.apply_cpu_power_optimization()?;
        
        // Apply GPU power optimizations
        if self.settings.gpu_power_saving && self.info.supports_gpu_power_saving {
            self.apply_gpu_power_optimization()?;
        }
        
        // Apply peripheral power optimizations
        if self.settings.peripheral_power_management && self.info.supports_peripheral_power_management {
            self.apply_peripheral_power_optimization()?;
        }
        
        // Apply USB power optimizations
        if self.settings.usb_power_saving && self.info.supports_usb_power_saving {
            self.apply_usb_power_optimization()?;
        }
        
        // Apply Wi-Fi power optimizations
        if self.settings.wifi_power_saving && self.info.supports_wifi_power_saving {
            self.apply_wifi_power_optimization()?;
        }
        
        // Apply Bluetooth power optimizations
        if self.settings.bluetooth_power_saving && self.info.supports_bluetooth_power_saving {
            self.apply_bluetooth_power_optimization()?;
        }
        
        // Apply display power optimizations
        if self.settings.display_power_saving && self.info.supports_display_power_saving {
            self.apply_display_power_optimization()?;
        }
        
        // Apply audio power optimizations
        if self.settings.audio_power_saving && self.info.supports_audio_power_saving {
            self.apply_audio_power_optimization()?;
        }
        
        // Apply storage power optimizations
        if self.settings.storage_power_saving && self.info.supports_storage_power_saving {
            self.apply_storage_power_optimization()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("Power optimizations applied successfully");
        Ok(())
    }
    
    /// Reset power optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting power optimizations");
        
        // Reset CPU power settings
        self.reset_cpu_power_settings()?;
        
        // Reset GPU power settings
        self.reset_gpu_power_settings()?;
        
        // Reset peripheral power settings
        self.reset_peripheral_power_settings()?;
        
        // Reset USB power settings
        self.reset_usb_power_settings()?;
        
        // Reset Wi-Fi power settings
        self.reset_wifi_power_settings()?;
        
        // Reset Bluetooth power settings
        self.reset_bluetooth_power_settings()?;
        
        // Reset display power settings
        self.reset_display_power_settings()?;
        
        // Reset audio power settings
        self.reset_audio_power_settings()?;
        
        // Reset storage power settings
        self.reset_storage_power_settings()?;
        
        info!("Power optimizations reset successfully");
        Ok(())
    }
    
    /// Update power optimization settings.
    pub fn update_settings(&mut self, settings: PowerOptimizationSettings) -> Result<()> {
        info!("Updating power optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("Power optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current power optimization settings.
    pub fn get_settings(&self) -> PowerOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current power optimization state.
    pub fn get_state(&self) -> PowerOptimizationState {
        self.state.clone()
    }
    
    /// Get power information.
    pub fn get_info(&self) -> PowerInfo {
        self.info.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background power optimization thread");
        
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
                        error!("Error performing adaptive power optimization: {}", e);
                    }
                    
                    last_optimization_time = now;
                }
                
                // Sleep for a short time
                thread::sleep(Duration::from_millis(1000)); // Check less frequently than other optimizations
            }
        });
        
        self.background_thread = Some(thread);
        self.is_running = true;
        
        info!("Background power optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background power optimization thread");
        
        // Signal thread to stop
        {
            let mut state = self.shared_state.lock().unwrap();
            state.should_stop = true;
        }
        
        // Wait for thread to finish
        if let Some(thread) = self.background_thread.take() {
            if let Err(e) = thread.join() {
                error!("Error joining background power thread: {:?}", e);
            }
        }
        
        self.is_running = false;
        
        info!("Background power optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive power optimization.
    fn perform_adaptive_optimization(info: &PowerInfo, settings: &PowerOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive power optimization");
        
        // Get current system load
        let load_avg = Self::get_load_average()?;
        
        // Get current CPU temperature
        let cpu_temp = Self::get_cpu_temperature()?;
        
        // Adjust CPU governor based on load and temperature
        let mut target_governor = settings.cpu_governor;
        
        if load_avg > 3.0 || cpu_temp > 75.0 {
            // High load or temperature: prioritize performance
            target_governor = CpuGovernor::Performance;
        } else if load_avg < 1.0 && cpu_temp < 50.0 {
            // Low load and temperature: prioritize power saving
            target_governor = CpuGovernor::Powersave;
        } else {
            // Moderate load and temperature: use balanced governor
            target_governor = CpuGovernor::Schedutil;
        }
        
        // Apply target governor if different from current setting
        let current_governor = Self::get_current_cpu_governor(0)?;
        if target_governor != current_governor {
            Self::set_cpu_governor_for_all_cores(target_governor)?;
        }
        
        // Adjust peripheral power saving based on activity (placeholder)
        // TODO: Implement logic to detect peripheral activity
        
        debug!("Adaptive power optimization completed");
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
                self.settings.cpu_governor = CpuGovernor::Performance;
                self.settings.min_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
                self.settings.max_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
                self.settings.gpu_power_saving = false;
                self.settings.peripheral_power_management = false;
                self.settings.usb_power_saving = false;
                self.settings.wifi_power_saving = false;
                self.settings.bluetooth_power_saving = false;
                self.settings.display_power_saving = false;
                self.settings.audio_power_saving = false;
                self.settings.storage_power_saving = false;
                self.settings.dynamic_power_management = false;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.cpu_governor = CpuGovernor::Powersave;
                self.settings.min_cpu_freq_mhz = self.info.min_cpu_freq_mhz;
                self.settings.max_cpu_freq_mhz = self.info.max_cpu_freq_mhz / 2; // Limit max freq
                self.settings.gpu_power_saving = true;
                self.settings.peripheral_power_management = true;
                self.settings.usb_power_saving = true;
                self.settings.wifi_power_saving = true;
                self.settings.bluetooth_power_saving = true;
                self.settings.display_power_saving = true;
                self.settings.audio_power_saving = true;
                self.settings.storage_power_saving = true;
                self.settings.dynamic_power_management = true;
            },
            super::OptimizationMode::Latency => {
                self.settings.cpu_governor = CpuGovernor::Performance;
                self.settings.min_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
                self.settings.max_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
                self.settings.gpu_power_saving = false;
                self.settings.peripheral_power_management = false;
                self.settings.usb_power_saving = false;
                self.settings.wifi_power_saving = false;
                self.settings.bluetooth_power_saving = false;
                self.settings.display_power_saving = false;
                self.settings.audio_power_saving = false;
                self.settings.storage_power_saving = false;
                self.settings.dynamic_power_management = false;
            },
            super::OptimizationMode::Thermal => {
                self.settings.cpu_governor = CpuGovernor::Powersave;
                self.settings.min_cpu_freq_mhz = self.info.min_cpu_freq_mhz;
                self.settings.max_cpu_freq_mhz = self.info.max_cpu_freq_mhz / 2; // Limit max freq
                self.settings.gpu_power_saving = true;
                self.settings.peripheral_power_management = true;
                self.settings.usb_power_saving = true;
                self.settings.wifi_power_saving = true;
                self.settings.bluetooth_power_saving = true;
                self.settings.display_power_saving = true;
                self.settings.audio_power_saving = true;
                self.settings.storage_power_saving = true;
                self.settings.dynamic_power_management = true;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.cpu_governor = CpuGovernor::Performance;
            self.settings.min_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
            self.settings.max_cpu_freq_mhz = self.info.max_cpu_freq_mhz;
            self.settings.gpu_power_saving = false;
            self.settings.peripheral_power_management = false;
            self.settings.usb_power_saving = false;
            self.settings.wifi_power_saving = false;
            self.settings.bluetooth_power_saving = false;
            self.settings.display_power_saving = false;
            self.settings.audio_power_saving = false;
            self.settings.storage_power_saving = false;
            self.settings.dynamic_power_management = false;
        }
    }
    
    /// Apply CPU power optimizations.
    fn apply_cpu_power_optimization(&self) -> Result<()> {
        info!("Applying CPU power optimizations");
        
        // Set CPU governor
        Self::set_cpu_governor_for_all_cores(self.settings.cpu_governor)?;
        
        // Set CPU frequency limits
        Self::set_cpu_freq_limits_for_all_cores(
            self.settings.min_cpu_freq_mhz * 1000, // Convert to kHz
            self.settings.max_cpu_freq_mhz * 1000, // Convert to kHz
        )?;
        
        Ok(())
    }
    
    /// Apply GPU power optimizations.
    fn apply_gpu_power_optimization(&self) -> Result<()> {
        info!("Applying GPU power optimizations");
        
        // Set GPU frequency scaling governor (if available)
        let gpu_gov_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/governor");
        if gpu_gov_path.exists() {
            let governor = if self.settings.gpu_power_saving { "powersave" } else { "performance" };
            fs::write(&gpu_gov_path, governor)
                .context("Failed to set GPU governor")?;
        } else {
            warn!("GPU governor setting not supported");
        }
        
        // Set GPU frequency limits (if available)
        let gpu_min_freq_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/min_freq");
        let gpu_max_freq_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/max_freq");
        
        if gpu_min_freq_path.exists() && gpu_max_freq_path.exists() {
            let min_freq = if self.settings.gpu_power_saving { "200000000" } else { "600000000" }; // Example frequencies
            let max_freq = if self.settings.gpu_power_saving { "400000000" } else { "600000000" }; // Example frequencies
            
            fs::write(&gpu_min_freq_path, min_freq)
                .context("Failed to set GPU min frequency")?;
            fs::write(&gpu_max_freq_path, max_freq)
                .context("Failed to set GPU max frequency")?;
        } else {
            warn!("GPU frequency limit setting not supported");
        }
        
        Ok(())
    }
    
    /// Apply peripheral power optimizations.
    fn apply_peripheral_power_optimization(&self) -> Result<()> {
        info!("Applying peripheral power optimizations");
        
        // Enable runtime PM for all devices
        let control_path = Path::new("/sys/bus/platform/devices");
        if control_path.exists() {
            for entry in fs::read_dir(control_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let power_control_path = path.join("power/control");
                    if power_control_path.exists() {
                        let value = if self.settings.peripheral_power_management { "auto" } else { "on" };
                        fs::write(&power_control_path, value)
                            .unwrap_or_else(|e| warn!("Failed to set power control for {}: {}", path.display(), e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply USB power optimizations.
    fn apply_usb_power_optimization(&self) -> Result<()> {
        info!("Applying USB power optimizations");
        
        // Enable USB autosuspend
        let control_path = Path::new("/sys/bus/usb/devices");
        if control_path.exists() {
            for entry in fs::read_dir(control_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let power_control_path = path.join("power/control");
                    if power_control_path.exists() {
                        let value = if self.settings.usb_power_saving { "auto" } else { "on" };
                        fs::write(&power_control_path, value)
                            .unwrap_or_else(|e| warn!("Failed to set USB power control for {}: {}", path.display(), e));
                    }
                    
                    let autosuspend_path = path.join("power/autosuspend");
                    if autosuspend_path.exists() {
                        let value = if self.settings.usb_power_saving { "2" } else { "-1" }; // 2 seconds delay
                        fs::write(&autosuspend_path, value)
                            .unwrap_or_else(|e| warn!("Failed to set USB autosuspend for {}: {}", path.display(), e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply Wi-Fi power optimizations.
    fn apply_wifi_power_optimization(&self) -> Result<()> {
        info!("Applying Wi-Fi power optimizations");
        
        // Find Wi-Fi interface
        let wifi_interface = Self::find_wifi_interface()?;
        
        if let Some(interface) = wifi_interface {
            // Set Wi-Fi power management
            let output = Command::new("iw")
                .arg("dev")
                .arg(&interface)
                .arg("set")
                .arg("power_save")
                .arg(if self.settings.wifi_power_saving { "on" } else { "off" })
                .output()
                .context("Failed to execute iw command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set Wi-Fi power management for {}: {}", interface, error);
            }
        } else {
            warn!("No Wi-Fi interface found for power optimization");
        }
        
        Ok(())
    }
    
    /// Apply Bluetooth power optimizations.
    fn apply_bluetooth_power_optimization(&self) -> Result<()> {
        info!("Applying Bluetooth power optimizations");
        
        // Bluetooth power management is typically handled by the kernel
        // No specific user-space commands are usually needed
        
        Ok(())
    }
    
    /// Apply display power optimizations.
    fn apply_display_power_optimization(&self) -> Result<()> {
        info!("Applying display power optimizations");
        
        // Set display blanking timeout (using xset if X server is running)
        let output = Command::new("xset")
            .arg("q")
            .output();
            
        if output.is_ok() && output.unwrap().status.success() {
            let timeout = if self.settings.display_power_saving { "60" } else { "0" }; // 60 seconds or disable
            let output = Command::new("xset")
                .arg("s")
                .arg(timeout)
                .output()
                .context("Failed to execute xset command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set display blanking timeout: {}", error);
            }
        } else {
            // Set console blanking timeout
            let timeout = if self.settings.display_power_saving { "1" } else { "0" }; // 1 minute or disable
            let output = Command::new("setterm")
                .arg("--blank")
                .arg(timeout)
                .output()
                .context("Failed to execute setterm command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set console blanking timeout: {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Apply audio power optimizations.
    fn apply_audio_power_optimization(&self) -> Result<()> {
        info!("Applying audio power optimizations");
        
        // Enable audio codec power saving
        let timeout_path = Path::new("/sys/module/snd_hda_intel/parameters/power_save");
        if timeout_path.exists() {
            let value = if self.settings.audio_power_saving { "1" } else { "0" };
            fs::write(&timeout_path, value)
                .context("Failed to set audio power save timeout")?;
        } else {
            warn!("Audio power saving setting not supported");
        }
        
        Ok(())
    }
    
    /// Apply storage power optimizations.
    fn apply_storage_power_optimization(&self) -> Result<()> {
        info!("Applying storage power optimizations");
        
        // Set HDD spindown timeout (if applicable)
        let output = Command::new("hdparm")
            .arg("-S")
            .arg(if self.settings.storage_power_saving { "12" } else { "0" }) // 1 minute or disable
            .arg("/dev/sda") // Assuming primary storage is /dev/sda
            .output();
            
        if output.is_ok() && !output.unwrap().status.success() {
            // Ignore errors if hdparm is not installed or device is not HDD
        }
        
        // Enable NVMe APST (Autonomous Power State Transition)
        let output = Command::new("nvme")
            .arg("set-feature")
            .arg("/dev/nvme0") // Assuming primary NVMe is /dev/nvme0
            .arg("-f")
            .arg("0x0c") // Power Management feature
            .arg("-v")
            .arg(if self.settings.storage_power_saving { "1" } else { "0" }) // Enable/disable APST
            .output();
            
        if output.is_ok() && !output.unwrap().status.success() {
            // Ignore errors if nvme-cli is not installed or device is not NVMe
        }
        
        Ok(())
    }
    
    /// Reset CPU power settings.
    fn reset_cpu_power_settings(&self) -> Result<()> {
        info!("Resetting CPU power settings");
        
        // Reset CPU governor to default (usually ondemand or schedutil)
        Self::set_cpu_governor_for_all_cores(CpuGovernor::Schedutil)?;
        
        // Reset CPU frequency limits to hardware defaults
        Self::set_cpu_freq_limits_for_all_cores(
            self.info.min_cpu_freq_mhz * 1000,
            self.info.max_cpu_freq_mhz * 1000,
        )?;
        
        Ok(())
    }
    
    /// Reset GPU power settings.
    fn reset_gpu_power_settings(&self) -> Result<()> {
        info!("Resetting GPU power settings");
        
        // Reset GPU frequency scaling governor
        let gpu_gov_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/governor");
        if gpu_gov_path.exists() {
            fs::write(&gpu_gov_path, "performance")
                .context("Failed to reset GPU governor")?;
        }
        
        // Reset GPU frequency limits
        let gpu_min_freq_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/min_freq");
        let gpu_max_freq_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/max_freq");
        
        if gpu_min_freq_path.exists() && gpu_max_freq_path.exists() {
            let available_freqs_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/available_frequencies");
            if available_freqs_path.exists() {
                let freqs_str = fs::read_to_string(available_freqs_path)?;
                let freqs: Vec<&str> = freqs_str.split_whitespace().collect();
                if !freqs.is_empty() {
                    fs::write(&gpu_min_freq_path, freqs[0])?;
                    fs::write(&gpu_max_freq_path, freqs[freqs.len() - 1])?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Reset peripheral power settings.
    fn reset_peripheral_power_settings(&self) -> Result<()> {
        info!("Resetting peripheral power settings");
        
        // Disable runtime PM for all devices
        let control_path = Path::new("/sys/bus/platform/devices");
        if control_path.exists() {
            for entry in fs::read_dir(control_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let power_control_path = path.join("power/control");
                    if power_control_path.exists() {
                        fs::write(&power_control_path, "on")
                            .unwrap_or_else(|e| warn!("Failed to reset power control for {}: {}", path.display(), e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Reset USB power settings.
    fn reset_usb_power_settings(&self) -> Result<()> {
        info!("Resetting USB power settings");
        
        // Disable USB autosuspend
        let control_path = Path::new("/sys/bus/usb/devices");
        if control_path.exists() {
            for entry in fs::read_dir(control_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let power_control_path = path.join("power/control");
                    if power_control_path.exists() {
                        fs::write(&power_control_path, "on")
                            .unwrap_or_else(|e| warn!("Failed to reset USB power control for {}: {}", path.display(), e));
                    }
                    
                    let autosuspend_path = path.join("power/autosuspend");
                    if autosuspend_path.exists() {
                        fs::write(&autosuspend_path, "-1")
                            .unwrap_or_else(|e| warn!("Failed to reset USB autosuspend for {}: {}", path.display(), e));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Reset Wi-Fi power settings.
    fn reset_wifi_power_settings(&self) -> Result<()> {
        info!("Resetting Wi-Fi power settings");
        
        // Find Wi-Fi interface
        let wifi_interface = Self::find_wifi_interface()?;
        
        if let Some(interface) = wifi_interface {
            // Disable Wi-Fi power management
            let output = Command::new("iw")
                .arg("dev")
                .arg(&interface)
                .arg("set")
                .arg("power_save")
                .arg("off")
                .output()
                .context("Failed to execute iw command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to reset Wi-Fi power management for {}: {}", interface, error);
            }
        }
        
        Ok(())
    }
    
    /// Reset Bluetooth power settings.
    fn reset_bluetooth_power_settings(&self) -> Result<()> {
        info!("Resetting Bluetooth power settings");
        // No specific reset needed
        Ok(())
    }
    
    /// Reset display power settings.
    fn reset_display_power_settings(&self) -> Result<()> {
        info!("Resetting display power settings");
        
        // Disable display blanking timeout (using xset if X server is running)
        let output = Command::new("xset")
            .arg("q")
            .output();
            
        if output.is_ok() && output.unwrap().status.success() {
            let output = Command::new("xset")
                .arg("s")
                .arg("0")
                .output()
                .context("Failed to execute xset command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to reset display blanking timeout: {}", error);
            }
        } else {
            // Disable console blanking timeout
            let output = Command::new("setterm")
                .arg("--blank")
                .arg("0")
                .output()
                .context("Failed to execute setterm command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to reset console blanking timeout: {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Reset audio power settings.
    fn reset_audio_power_settings(&self) -> Result<()> {
        info!("Resetting audio power settings");
        
        // Disable audio codec power saving
        let timeout_path = Path::new("/sys/module/snd_hda_intel/parameters/power_save");
        if timeout_path.exists() {
            fs::write(&timeout_path, "0")
                .context("Failed to reset audio power save timeout")?;
        } else {
            warn!("Audio power saving setting not supported");
        }
        
        Ok(())
    }
    
    /// Reset storage power settings.
    fn reset_storage_power_settings(&self) -> Result<()> {
        info!("Resetting storage power settings");
        
        // Disable HDD spindown timeout
        let output = Command::new("hdparm")
            .arg("-S")
            .arg("0")
            .arg("/dev/sda")
            .output();
            
        if output.is_ok() && !output.unwrap().status.success() {
            // Ignore errors
        }
        
        // Disable NVMe APST
        let output = Command::new("nvme")
            .arg("set-feature")
            .arg("/dev/nvme0")
            .arg("-f")
            .arg("0x0c")
            .arg("-v")
            .arg("0")
            .output();
            
        if output.is_ok() && !output.unwrap().status.success() {
            // Ignore errors
        }
        
        Ok(())
    }
    
    /// Set CPU governor for a specific core.
    fn set_cpu_governor(core: u32, governor: CpuGovernor) -> Result<()> {
        debug!("Setting CPU governor for core {} to {}", core, governor.to_str());
        
        let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", core);
        let path = Path::new(&governor_path);
        
        if path.exists() {
            fs::write(path, governor.to_str())
                .context(format!("Failed to set CPU governor for core {}", core))?;
        } else {
            warn!("CPU governor setting not supported for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU governor for all cores.
    fn set_cpu_governor_for_all_cores(governor: CpuGovernor) -> Result<()> {
        info!("Setting CPU governor for all cores to {}", governor.to_str());
        
        let cpu_count = Self::get_cpu_count()?;
        for core in 0..cpu_count {
            Self::set_cpu_governor(core, governor)?;
        }
        
        Ok(())
    }
    
    /// Set CPU frequency limits for a specific core.
    fn set_cpu_freq_limits(core: u32, min_freq_khz: u32, max_freq_khz: u32) -> Result<()> {
        debug!("Setting CPU frequency limits for core {} to [{}, {}] kHz", core, min_freq_khz, max_freq_khz);
        
        let min_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq", core);
        let max_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq", core);
        
        let min_path = Path::new(&min_freq_path);
        let max_path = Path::new(&max_freq_path);
        
        if min_path.exists() {
            fs::write(min_path, min_freq_khz.to_string())
                .context(format!("Failed to set min CPU frequency for core {}", core))?;
        } else {
            warn!("Min CPU frequency setting not supported for core {}", core);
        }
        
        if max_path.exists() {
            fs::write(max_path, max_freq_khz.to_string())
                .context(format!("Failed to set max CPU frequency for core {}", core))?;
        } else {
            warn!("Max CPU frequency setting not supported for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU frequency limits for all cores.
    fn set_cpu_freq_limits_for_all_cores(min_freq_khz: u32, max_freq_khz: u32) -> Result<()> {
        info!("Setting CPU frequency limits for all cores to [{}, {}] kHz", min_freq_khz, max_freq_khz);
        
        let cpu_count = Self::get_cpu_count()?;
        for core in 0..cpu_count {
            Self::set_cpu_freq_limits(core, min_freq_khz, max_freq_khz)?;
        }
        
        Ok(())
    }
    
    /// Get the number of CPU cores.
    fn get_cpu_count() -> Result<u32> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")
            .context("Failed to read /proc/cpuinfo")?;
        
        let count = cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count();
            
        Ok(count as u32)
    }
    
    /// Get current CPU governor for a specific core.
    fn get_current_cpu_governor(core: u32) -> Result<CpuGovernor> {
        let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", core);
        let path = Path::new(&governor_path);
        
        if path.exists() {
            let governor_str = fs::read_to_string(path)
                .context(format!("Failed to read CPU governor for core {}", core))?
                .trim()
                .to_string();
            
            CpuGovernor::from_str(&governor_str)
                .map_err(|_| anyhow!("Unknown CPU governor: {}", governor_str))
        } else {
            Ok(CpuGovernor::Schedutil) // Default if not supported
        }
    }
    
    /// Get current CPU frequency limits for a specific core.
    fn get_current_cpu_freq_limits(core: u32) -> Result<(u32, u32)> {
        let min_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq", core);
        let max_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq", core);
        
        let min_path = Path::new(&min_freq_path);
        let max_path = Path::new(&max_freq_path);
        
        let min_freq_khz = if min_path.exists() {
            fs::read_to_string(min_path)?
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
        } else {
            0
        };
        
        let max_freq_khz = if max_path.exists() {
            fs::read_to_string(max_path)?
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
        } else {
            0
        };
        
        Ok((min_freq_khz, max_freq_khz))
    }
    
    /// Get current GPU power saving state.
    fn get_current_gpu_power_saving() -> Result<bool> {
        let gpu_gov_path = Path::new("/sys/class/devfreq/ff9a0000.gpu/governor");
        if gpu_gov_path.exists() {
            let governor = fs::read_to_string(gpu_gov_path)?;
            Ok(governor.trim() == "powersave")
        } else {
            Ok(false) // Assume not enabled if not supported
        }
    }
    
    /// Get current peripheral power management state.
    fn get_current_peripheral_power_management() -> Result<bool> {
        // Check a sample device
        let control_path = Path::new("/sys/bus/platform/devices/ff100000.i2c/power/control");
        if control_path.exists() {
            let state = fs::read_to_string(control_path)?;
            Ok(state.trim() == "auto")
        } else {
            Ok(false) // Assume not enabled if not supported
        }
    }
    
    /// Get current USB power saving state.
    fn get_current_usb_power_saving() -> Result<bool> {
        // Check a sample USB device
        let control_path = Path::new("/sys/bus/usb/devices/1-1/power/control"); // Example path
        if control_path.exists() {
            let state = fs::read_to_string(control_path)?;
            Ok(state.trim() == "auto")
        } else {
            Ok(false) // Assume not enabled if not supported
        }
    }
    
    /// Get current Wi-Fi power saving state.
    fn get_current_wifi_power_saving() -> Result<bool> {
        let wifi_interface = Self::find_wifi_interface()?;
        
        if let Some(interface) = wifi_interface {
            let output = Command::new("iw")
                .arg("dev")
                .arg(&interface)
                .arg("get")
                .arg("power_save")
                .output()
                .context("Failed to execute iw command")?;
            
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                Ok(output_str.contains("on"))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    /// Get current Bluetooth power saving state.
    fn get_current_bluetooth_power_saving() -> Result<bool> {
        // Bluetooth power saving is typically managed by the kernel
        Ok(true) // Assume enabled by default
    }
    
    /// Get current display power saving state.
    fn get_current_display_power_saving() -> Result<bool> {
        // Check xset state
        let output = Command::new("xset")
            .arg("q")
            .output();
            
        if output.is_ok() && output.unwrap().status.success() {
            let output_str = String::from_utf8_lossy(&output.unwrap().stdout);
            if let Some(timeout_line) = output_str.lines().find(|l| l.contains("timeout:")) {
                if let Some(timeout_str) = timeout_line.split_whitespace().nth(1) {
                    if let Ok(timeout) = timeout_str.parse::<u32>() {
                        return Ok(timeout > 0);
                    }
                }
            }
        } else {
            // Check console blanking state
            // Cannot reliably get this state, assume default
        }
        
        Ok(true) // Assume enabled by default
    }
    
    /// Get current audio power saving state.
    fn get_current_audio_power_saving() -> Result<bool> {
        let timeout_path = Path::new("/sys/module/snd_hda_intel/parameters/power_save");
        if timeout_path.exists() {
            let state = fs::read_to_string(timeout_path)?;
            Ok(state.trim() == "1")
        } else {
            Ok(false) // Assume not enabled if not supported
        }
    }
    
    /// Get current storage power saving state.
    fn get_current_storage_power_saving() -> Result<bool> {
        // Cannot reliably get this state, assume default
        Ok(true) // Assume enabled by default
    }
    
    /// Detect power information.
    fn detect_power_info() -> Result<PowerInfo> {
        info!("Detecting power information");
        
        // Detect available CPU governors
        let mut available_cpu_governors = Vec::new();
        let governors_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors");
        if governors_path.exists() {
            let governors_str = fs::read_to_string(governors_path)?;
            for gov_str in governors_str.split_whitespace() {
                if let Ok(gov) = CpuGovernor::from_str(gov_str) {
                    available_cpu_governors.push(gov);
                }
            }
        }
        if available_cpu_governors.is_empty() {
            available_cpu_governors.push(CpuGovernor::Schedutil); // Default
        }
        
        // Detect CPU frequency limits
        let min_freq_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq");
        let max_freq_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq");
        
        let min_cpu_freq_mhz = if min_freq_path.exists() {
            fs::read_to_string(min_freq_path)?
                .trim()
                .parse::<u32>()
                .unwrap_or(408000) / 1000 // Default min freq for RK3588S
        } else {
            408
        };
        
        let max_cpu_freq_mhz = if max_freq_path.exists() {
            fs::read_to_string(max_freq_path)?
                .trim()
                .parse::<u32>()
                .unwrap_or(2400000) / 1000 // Default max freq for RK3588S
        } else {
            2400
        };
        
        // Check GPU power saving support
        let supports_gpu_power_saving = Path::new("/sys/class/devfreq/ff9a0000.gpu/governor").exists();
        
        // Check peripheral power management support
        let supports_peripheral_power_management = Path::new("/sys/bus/platform/devices/ff100000.i2c/power/control").exists();
        
        // Check USB power saving support
        let supports_usb_power_saving = Path::new("/sys/bus/usb/devices/1-1/power/control").exists();
        
        // Check Wi-Fi power saving support
        let supports_wifi_power_saving = Self::find_wifi_interface()?.is_some();
        
        // Check Bluetooth power saving support
        let supports_bluetooth_power_saving = Path::new("/sys/class/bluetooth/hci0").exists();
        
        // Check display power saving support
        let supports_display_power_saving = Command::new("xset").arg("q").output().is_ok()
            || Command::new("setterm").arg("--version").output().is_ok();
            
        // Check audio power saving support
        let supports_audio_power_saving = Path::new("/sys/module/snd_hda_intel/parameters/power_save").exists();
        
        // Check storage power saving support
        let supports_storage_power_saving = Command::new("hdparm").arg("-V").output().is_ok()
            || Command::new("nvme").arg("--version").output().is_ok();
            
        Ok(PowerInfo {
            available_cpu_governors,
            min_cpu_freq_mhz,
            max_cpu_freq_mhz,
            supports_gpu_power_saving,
            supports_peripheral_power_management,
            supports_usb_power_saving,
            supports_wifi_power_saving,
            supports_bluetooth_power_saving,
            supports_display_power_saving,
            supports_audio_power_saving,
            supports_storage_power_saving,
        })
    }
    
    /// Get current power optimization state.
    fn get_current_state(info: &PowerInfo) -> Result<PowerOptimizationState> {
        debug!("Getting current power optimization state");
        
        let cpu_governor = Self::get_current_cpu_governor(0)?;
        let (min_freq_khz, max_freq_khz) = Self::get_current_cpu_freq_limits(0)?;
        let gpu_power_saving_enabled = Self::get_current_gpu_power_saving()?;
        let peripheral_power_management_enabled = Self::get_current_peripheral_power_management()?;
        let usb_power_saving_enabled = Self::get_current_usb_power_saving()?;
        let wifi_power_saving_enabled = Self::get_current_wifi_power_saving()?;
        let bluetooth_power_saving_enabled = Self::get_current_bluetooth_power_saving()?;
        let display_power_saving_enabled = Self::get_current_display_power_saving()?;
        let audio_power_saving_enabled = Self::get_current_audio_power_saving()?;
        let storage_power_saving_enabled = Self::get_current_storage_power_saving()?;
        
        Ok(PowerOptimizationState {
            cpu_governor,
            min_cpu_freq_mhz: min_freq_khz / 1000,
            max_cpu_freq_mhz: max_freq_khz / 1000,
            gpu_power_saving_enabled,
            peripheral_power_management_enabled,
            usb_power_saving_enabled,
            wifi_power_saving_enabled,
            bluetooth_power_saving_enabled,
            display_power_saving_enabled,
            audio_power_saving_enabled,
            storage_power_saving_enabled,
        })
    }
    
    /// Find the first Wi-Fi interface name.
    fn find_wifi_interface() -> Result<Option<String>> {
        let output = Command::new("ip")
            .arg("-o")
            .arg("link")
            .arg("show")
            .output()
            .context("Failed to execute ip command")?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_end_matches(":");
                    if name.starts_with("wl") {
                        return Ok(Some(name.to_string()));
                    }
                }
            }
        }
        Ok(None)
    }
    
    /// Get system load average (1-minute).
    fn get_load_average() -> Result<f32> {
        let loadavg_str = fs::read_to_string("/proc/loadavg")
            .context("Failed to read /proc/loadavg")?;
        let parts: Vec<&str> = loadavg_str.split_whitespace().collect();
        if parts.len() >= 1 {
            parts[0].parse::<f32>().context("Failed to parse load average")
        } else {
            bail!("Invalid format in /proc/loadavg")
        }
    }
    
    /// Get CPU temperature.
    fn get_cpu_temperature() -> Result<f32> {
        // Try different paths for CPU temperature
        let paths = [
            "/sys/class/thermal/thermal_zone0/temp",
            "/sys/class/thermal/thermal_zone1/temp",
            "/sys/class/hwmon/hwmon0/temp1_input",
            "/sys/class/hwmon/hwmon1/temp1_input",
        ];
        
        for path_str in paths {
            let path = Path::new(path_str);
            if path.exists() {
                let temp_str = fs::read_to_string(path)?;
                if let Ok(temp_milli_c) = temp_str.trim().parse::<f32>() {
                    return Ok(temp_milli_c / 1000.0);
                }
            }
        }
        
        bail!("Could not find CPU temperature sensor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_optimization_settings_default() {
        let settings = PowerOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert_eq!(settings.cpu_governor, CpuGovernor::Schedutil);
        assert_eq!(settings.min_cpu_freq_mhz, 408);
        assert_eq!(settings.max_cpu_freq_mhz, 2400);
        assert!(settings.gpu_power_saving);
        assert!(settings.peripheral_power_management);
        assert!(settings.usb_power_saving);
        assert!(settings.wifi_power_saving);
        assert!(settings.bluetooth_power_saving);
        assert!(settings.display_power_saving);
        assert!(settings.audio_power_saving);
        assert!(settings.storage_power_saving);
        assert!(settings.dynamic_power_management);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 10000);
    }
}
