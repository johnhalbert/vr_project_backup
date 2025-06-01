//! CPU optimization module for the VR headset system.
//!
//! This module provides CPU optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages CPU frequency scaling,
//! core affinity, process priority, and other CPU-related optimizations to
//! maximize performance for VR workloads.

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

/// CPU optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct CpuOptimizationManager {
    /// CPU optimization settings
    settings: CpuOptimizationSettings,
    
    /// CPU topology information
    topology: CpuTopology,
    
    /// Current CPU optimization state
    state: CpuOptimizationState,
    
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
    
    /// Current CPU optimization settings
    settings: CpuOptimizationSettings,
}

/// CPU optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuOptimizationSettings {
    /// Whether CPU optimization is enabled
    pub enabled: bool,
    
    /// CPU governor to use for VR cores
    pub vr_governor: CpuGovernor,
    
    /// CPU governor to use for system cores
    pub system_governor: CpuGovernor,
    
    /// Maximum frequency for VR cores (in kHz)
    pub vr_max_freq: u32,
    
    /// Minimum frequency for VR cores (in kHz)
    pub vr_min_freq: u32,
    
    /// Maximum frequency for system cores (in kHz)
    pub system_max_freq: u32,
    
    /// Minimum frequency for system cores (in kHz)
    pub system_min_freq: u32,
    
    /// Whether to isolate VR cores
    pub isolate_vr_cores: bool,
    
    /// Whether to use real-time scheduling for VR processes
    pub use_rt_scheduling: bool,
    
    /// Real-time priority for VR processes
    pub rt_priority: u8,
    
    /// Nice value for VR processes
    pub nice_value: i8,
    
    /// Whether to optimize IRQ handling
    pub optimize_irq: bool,
    
    /// Whether to disable CPU idle states for VR cores
    pub disable_idle_states: bool,
    
    /// Whether to optimize CPU cache
    pub optimize_cache: bool,
    
    /// Whether to enable thermal management
    pub thermal_management: bool,
    
    /// Maximum temperature before throttling (in Celsius)
    pub max_temperature: u8,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// CPU governor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuGovernor {
    /// Performance governor (maximum frequency)
    Performance,
    
    /// Powersave governor (minimum frequency)
    Powersave,
    
    /// Ondemand governor (dynamic frequency scaling)
    Ondemand,
    
    /// Conservative governor (gradual frequency scaling)
    Conservative,
    
    /// Schedutil governor (scheduler-driven frequency scaling)
    Schedutil,
    
    /// Userspace governor (user-defined frequency)
    Userspace,
}

impl CpuGovernor {
    /// Convert CPU governor to string.
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Powersave => "powersave",
            Self::Ondemand => "ondemand",
            Self::Conservative => "conservative",
            Self::Schedutil => "schedutil",
            Self::Userspace => "userspace",
        }
    }
    
    /// Parse CPU governor from string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "performance" => Ok(Self::Performance),
            "powersave" => Ok(Self::Powersave),
            "ondemand" => Ok(Self::Ondemand),
            "conservative" => Ok(Self::Conservative),
            "schedutil" => Ok(Self::Schedutil),
            "userspace" => Ok(Self::Userspace),
            _ => bail!("Unknown CPU governor: {}", s),
        }
    }
}

/// CPU topology information.
#[derive(Debug, Clone)]
pub struct CpuTopology {
    /// Number of CPU cores
    pub num_cores: usize,
    
    /// Number of CPU clusters
    pub num_clusters: usize,
    
    /// Cores per cluster
    pub cores_per_cluster: Vec<usize>,
    
    /// Core IDs for VR workloads
    pub vr_cores: Vec<usize>,
    
    /// Core IDs for system workloads
    pub system_cores: Vec<usize>,
    
    /// Available CPU governors
    pub available_governors: Vec<CpuGovernor>,
    
    /// Available CPU frequencies (in kHz)
    pub available_frequencies: Vec<u32>,
    
    /// Maximum CPU frequency (in kHz)
    pub max_freq: u32,
    
    /// Minimum CPU frequency (in kHz)
    pub min_freq: u32,
}

/// CPU optimization state.
#[derive(Debug, Clone)]
pub struct CpuOptimizationState {
    /// Current CPU governor for each core
    pub governors: Vec<CpuGovernor>,
    
    /// Current CPU frequency for each core (in kHz)
    pub frequencies: Vec<u32>,
    
    /// Current CPU temperature for each core (in Celsius)
    pub temperatures: Vec<u8>,
    
    /// Current CPU utilization for each core (in percent)
    pub utilizations: Vec<u8>,
    
    /// Whether each core is isolated
    pub isolated: Vec<bool>,
    
    /// Whether each core has idle states disabled
    pub idle_disabled: Vec<bool>,
    
    /// Current IRQ affinity
    pub irq_affinity: Vec<usize>,
    
    /// Current real-time processes
    pub rt_processes: Vec<u32>,
}

impl Default for CpuOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            vr_governor: CpuGovernor::Performance,
            system_governor: CpuGovernor::Schedutil,
            vr_max_freq: 2400000,
            vr_min_freq: 2400000,
            system_max_freq: 2400000,
            system_min_freq: 1200000,
            isolate_vr_cores: true,
            use_rt_scheduling: true,
            rt_priority: 80,
            nice_value: -20,
            optimize_irq: true,
            disable_idle_states: true,
            optimize_cache: true,
            thermal_management: true,
            max_temperature: 85,
            adaptive: true,
            adaptive_interval_ms: 1000,
        }
    }
}

impl CpuOptimizationManager {
    /// Create a new CPU optimization manager.
    pub fn new() -> Result<Self> {
        let topology = Self::detect_cpu_topology()?;
        let settings = CpuOptimizationSettings::default();
        let state = Self::get_current_state(&topology)?;
        let shared_state = Arc::new(Mutex::new(SharedState {
            should_stop: false,
            settings: settings.clone(),
        }));
        
        Ok(Self {
            settings,
            topology,
            state,
            last_optimization_time: Instant::now(),
            is_running: false,
            background_thread: None,
            shared_state,
        })
    }
    
    /// Initialize CPU optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing CPU optimization for Orange Pi CM5");
        
        // Detect CPU topology
        self.topology = Self::detect_cpu_topology()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.topology)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("CPU optimization initialized successfully");
        Ok(())
    }
    
    /// Apply CPU optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying CPU optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply CPU governor and frequency settings
        self.apply_governor_and_frequency_settings()?;
        
        // Apply CPU isolation if enabled
        if self.settings.isolate_vr_cores {
            self.apply_cpu_isolation()?;
        }
        
        // Apply real-time scheduling if enabled
        if self.settings.use_rt_scheduling {
            self.apply_rt_scheduling()?;
        }
        
        // Apply IRQ optimization if enabled
        if self.settings.optimize_irq {
            self.apply_irq_optimization()?;
        }
        
        // Apply idle states optimization if enabled
        if self.settings.disable_idle_states {
            self.apply_idle_states_optimization()?;
        }
        
        // Apply cache optimization if enabled
        if self.settings.optimize_cache {
            self.apply_cache_optimization()?;
        }
        
        // Apply thermal management if enabled
        if self.settings.thermal_management {
            self.apply_thermal_management()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.topology)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("CPU optimizations applied successfully");
        Ok(())
    }
    
    /// Reset CPU optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting CPU optimizations");
        
        // Reset CPU governor to default
        for core in 0..self.topology.num_cores {
            let governor = if self.topology.vr_cores.contains(&core) {
                CpuGovernor::Schedutil
            } else {
                CpuGovernor::Schedutil
            };
            
            self.set_cpu_governor(core, governor)?;
        }
        
        // Reset CPU frequency to default
        for core in 0..self.topology.num_cores {
            self.set_cpu_max_freq(core, self.topology.max_freq)?;
            self.set_cpu_min_freq(core, self.topology.min_freq)?;
        }
        
        // Reset CPU isolation
        self.reset_cpu_isolation()?;
        
        // Reset IRQ affinity
        self.reset_irq_affinity()?;
        
        // Reset idle states
        self.reset_idle_states()?;
        
        info!("CPU optimizations reset successfully");
        Ok(())
    }
    
    /// Update CPU optimization settings.
    pub fn update_settings(&mut self, settings: CpuOptimizationSettings) -> Result<()> {
        info!("Updating CPU optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("CPU optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current CPU optimization settings.
    pub fn get_settings(&self) -> CpuOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current CPU optimization state.
    pub fn get_state(&self) -> CpuOptimizationState {
        self.state.clone()
    }
    
    /// Get CPU topology information.
    pub fn get_topology(&self) -> CpuTopology {
        self.topology.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background CPU optimization thread");
        
        let shared_state = self.shared_state.clone();
        let topology = self.topology.clone();
        
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
                    if let Err(e) = Self::perform_adaptive_optimization(&topology, &settings) {
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
        
        info!("Background CPU optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background CPU optimization thread");
        
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
        
        info!("Background CPU optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive optimization.
    fn perform_adaptive_optimization(topology: &CpuTopology, settings: &CpuOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive CPU optimization");
        
        // Get current CPU state
        let state = Self::get_current_state(topology)?;
        
        // Check CPU temperature and adjust frequency if necessary
        if settings.thermal_management {
            for (i, temp) in state.temperatures.iter().enumerate() {
                if *temp > settings.max_temperature {
                    // Reduce frequency if temperature is too high
                    let current_freq = state.frequencies[i];
                    let new_freq = (current_freq as f32 * 0.9) as u32;
                    
                    if new_freq >= topology.min_freq {
                        if let Err(e) = Self::set_cpu_max_freq_static(i, new_freq) {
                            warn!("Error setting CPU frequency: {}", e);
                        }
                    }
                }
            }
        }
        
        // Check CPU utilization and adjust governor if necessary
        for (i, util) in state.utilizations.iter().enumerate() {
            if topology.vr_cores.contains(&i) {
                // VR cores should always use performance governor
                continue;
            }
            
            // For system cores, adjust governor based on utilization
            let governor = if *util > 80 {
                CpuGovernor::Performance
            } else if *util < 20 {
                CpuGovernor::Powersave
            } else {
                CpuGovernor::Schedutil
            };
            
            if governor != state.governors[i] {
                if let Err(e) = Self::set_cpu_governor_static(i, governor) {
                    warn!("Error setting CPU governor: {}", e);
                }
            }
        }
        
        debug!("Adaptive CPU optimization completed");
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
                self.settings.vr_governor = CpuGovernor::Performance;
                self.settings.system_governor = CpuGovernor::Performance;
                self.settings.vr_max_freq = self.topology.max_freq;
                self.settings.vr_min_freq = self.topology.max_freq;
                self.settings.system_max_freq = self.topology.max_freq;
                self.settings.system_min_freq = self.topology.max_freq / 2;
                self.settings.isolate_vr_cores = true;
                self.settings.use_rt_scheduling = true;
                self.settings.rt_priority = 80;
                self.settings.nice_value = -20;
                self.settings.optimize_irq = true;
                self.settings.disable_idle_states = true;
                self.settings.optimize_cache = true;
                self.settings.thermal_management = false;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.vr_governor = CpuGovernor::Schedutil;
                self.settings.system_governor = CpuGovernor::Powersave;
                self.settings.vr_max_freq = self.topology.max_freq;
                self.settings.vr_min_freq = self.topology.min_freq;
                self.settings.system_max_freq = self.topology.max_freq / 2;
                self.settings.system_min_freq = self.topology.min_freq;
                self.settings.isolate_vr_cores = true;
                self.settings.use_rt_scheduling = true;
                self.settings.rt_priority = 80;
                self.settings.nice_value = -20;
                self.settings.optimize_irq = true;
                self.settings.disable_idle_states = false;
                self.settings.optimize_cache = true;
                self.settings.thermal_management = true;
            },
            super::OptimizationMode::Latency => {
                self.settings.vr_governor = CpuGovernor::Performance;
                self.settings.system_governor = CpuGovernor::Schedutil;
                self.settings.vr_max_freq = self.topology.max_freq;
                self.settings.vr_min_freq = self.topology.max_freq;
                self.settings.system_max_freq = self.topology.max_freq;
                self.settings.system_min_freq = self.topology.min_freq;
                self.settings.isolate_vr_cores = true;
                self.settings.use_rt_scheduling = true;
                self.settings.rt_priority = 99;
                self.settings.nice_value = -20;
                self.settings.optimize_irq = true;
                self.settings.disable_idle_states = true;
                self.settings.optimize_cache = true;
                self.settings.thermal_management = false;
            },
            super::OptimizationMode::Thermal => {
                self.settings.vr_governor = CpuGovernor::Schedutil;
                self.settings.system_governor = CpuGovernor::Schedutil;
                self.settings.vr_max_freq = self.topology.max_freq * 9 / 10;
                self.settings.vr_min_freq = self.topology.min_freq;
                self.settings.system_max_freq = self.topology.max_freq * 8 / 10;
                self.settings.system_min_freq = self.topology.min_freq;
                self.settings.isolate_vr_cores = true;
                self.settings.use_rt_scheduling = true;
                self.settings.rt_priority = 80;
                self.settings.nice_value = -20;
                self.settings.optimize_irq = true;
                self.settings.disable_idle_states = false;
                self.settings.optimize_cache = true;
                self.settings.thermal_management = true;
                self.settings.max_temperature = 75;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.vr_governor = CpuGovernor::Performance;
            self.settings.vr_max_freq = self.topology.max_freq;
            self.settings.vr_min_freq = self.topology.max_freq;
            self.settings.isolate_vr_cores = true;
            self.settings.use_rt_scheduling = true;
            self.settings.rt_priority = 99;
            self.settings.nice_value = -20;
            self.settings.optimize_irq = true;
            self.settings.disable_idle_states = true;
            self.settings.optimize_cache = true;
        }
    }
    
    /// Apply CPU governor and frequency settings.
    fn apply_governor_and_frequency_settings(&self) -> Result<()> {
        info!("Applying CPU governor and frequency settings");
        
        for core in 0..self.topology.num_cores {
            let (governor, max_freq, min_freq) = if self.topology.vr_cores.contains(&core) {
                (self.settings.vr_governor, self.settings.vr_max_freq, self.settings.vr_min_freq)
            } else {
                (self.settings.system_governor, self.settings.system_max_freq, self.settings.system_min_freq)
            };
            
            // Set CPU governor
            self.set_cpu_governor(core, governor)?;
            
            // Set CPU frequency limits
            self.set_cpu_max_freq(core, max_freq)?;
            self.set_cpu_min_freq(core, min_freq)?;
        }
        
        Ok(())
    }
    
    /// Apply CPU isolation.
    fn apply_cpu_isolation(&self) -> Result<()> {
        info!("Applying CPU isolation");
        
        // Create isolcpus kernel parameter
        let vr_cores_str = self.topology.vr_cores.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        // Apply CPU isolation using sysfs
        let isolcpus_path = Path::new("/sys/devices/system/cpu/isolated");
        if isolcpus_path.exists() {
            fs::write(isolcpus_path, &vr_cores_str)
                .context("Failed to write to isolated cpus")?;
        } else {
            // If sysfs interface is not available, use kernel command line
            warn!("CPU isolation sysfs interface not available, using kernel command line");
            
            // Check if kernel command line already has isolcpus
            let cmdline = fs::read_to_string("/proc/cmdline")
                .context("Failed to read kernel command line")?;
            
            if !cmdline.contains("isolcpus=") {
                warn!("Kernel command line does not contain isolcpus parameter");
                warn!("Please add 'isolcpus={}' to kernel command line for proper CPU isolation", vr_cores_str);
            }
        }
        
        // Set CPU affinity for all processes to system cores
        let system_cores_str = self.topology.system_cores.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        // Use taskset to set default CPU affinity
        let output = Command::new("taskset")
            .arg("-p")
            .arg(format!("0x{:x}", self.calculate_cpu_mask(&self.topology.system_cores)))
            .arg("1")
            .output()
            .context("Failed to execute taskset command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set default CPU affinity: {}", error);
        }
        
        Ok(())
    }
    
    /// Reset CPU isolation.
    fn reset_cpu_isolation(&self) -> Result<()> {
        info!("Resetting CPU isolation");
        
        // Clear isolcpus using sysfs
        let isolcpus_path = Path::new("/sys/devices/system/cpu/isolated");
        if isolcpus_path.exists() {
            fs::write(isolcpus_path, "")
                .context("Failed to clear isolated cpus")?;
        }
        
        // Reset CPU affinity for all processes to all cores
        let all_cores_mask = (1 << self.topology.num_cores) - 1;
        
        let output = Command::new("taskset")
            .arg("-p")
            .arg(format!("0x{:x}", all_cores_mask))
            .arg("1")
            .output()
            .context("Failed to execute taskset command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to reset CPU affinity: {}", error);
        }
        
        Ok(())
    }
    
    /// Apply real-time scheduling.
    fn apply_rt_scheduling(&self) -> Result<()> {
        info!("Applying real-time scheduling");
        
        // Find VR processes
        let vr_processes = self.find_vr_processes()?;
        
        for pid in vr_processes {
            // Set real-time priority
            let output = Command::new("chrt")
                .arg("-f")
                .arg("-p")
                .arg(self.settings.rt_priority.to_string())
                .arg(pid.to_string())
                .output()
                .context("Failed to execute chrt command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set real-time priority for process {}: {}", pid, error);
            }
            
            // Set nice value
            let output = Command::new("renice")
                .arg("-n")
                .arg(self.settings.nice_value.to_string())
                .arg("-p")
                .arg(pid.to_string())
                .output()
                .context("Failed to execute renice command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set nice value for process {}: {}", pid, error);
            }
            
            // Set CPU affinity to VR cores
            let vr_cores_mask = self.calculate_cpu_mask(&self.topology.vr_cores);
            
            let output = Command::new("taskset")
                .arg("-p")
                .arg(format!("0x{:x}", vr_cores_mask))
                .arg(pid.to_string())
                .output()
                .context("Failed to execute taskset command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set CPU affinity for process {}: {}", pid, error);
            }
        }
        
        Ok(())
    }
    
    /// Apply IRQ optimization.
    fn apply_irq_optimization(&self) -> Result<()> {
        info!("Applying IRQ optimization");
        
        // Set default IRQ affinity to system cores
        let system_cores_mask = self.calculate_cpu_mask(&self.topology.system_cores);
        
        let default_irq_path = Path::new("/proc/irq/default_smp_affinity");
        if default_irq_path.exists() {
            fs::write(default_irq_path, format!("{:x}", system_cores_mask))
                .context("Failed to set default IRQ affinity")?;
        }
        
        // Set IRQ affinity for all IRQs to system cores
        let irq_dir = Path::new("/proc/irq");
        if irq_dir.exists() && irq_dir.is_dir() {
            for entry in fs::read_dir(irq_dir)
                .context("Failed to read IRQ directory")? {
                let entry = entry.context("Failed to read IRQ directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(irq_num) = path.file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|s| s.parse::<u32>().ok()) {
                        
                        let affinity_path = path.join("smp_affinity");
                        if affinity_path.exists() {
                            fs::write(&affinity_path, format!("{:x}", system_cores_mask))
                                .unwrap_or_else(|e| warn!("Failed to set IRQ affinity for IRQ {}: {}", irq_num, e));
                        }
                    }
                }
            }
        }
        
        // Set VR-specific IRQs to VR cores
        let vr_irqs = self.find_vr_irqs()?;
        let vr_cores_mask = self.calculate_cpu_mask(&self.topology.vr_cores);
        
        for irq in vr_irqs {
            let affinity_path = Path::new("/proc/irq").join(irq.to_string()).join("smp_affinity");
            if affinity_path.exists() {
                fs::write(&affinity_path, format!("{:x}", vr_cores_mask))
                    .unwrap_or_else(|e| warn!("Failed to set IRQ affinity for VR IRQ {}: {}", irq, e));
            }
        }
        
        // Disable IRQ balancing
        let output = Command::new("systemctl")
            .arg("stop")
            .arg("irqbalance.service")
            .output()
            .context("Failed to execute systemctl command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to stop IRQ balancing service: {}", error);
        }
        
        Ok(())
    }
    
    /// Reset IRQ affinity.
    fn reset_irq_affinity(&self) -> Result<()> {
        info!("Resetting IRQ affinity");
        
        // Calculate mask for all cores
        let all_cores_mask = (1 << self.topology.num_cores) - 1;
        
        // Set default IRQ affinity to all cores
        let default_irq_path = Path::new("/proc/irq/default_smp_affinity");
        if default_irq_path.exists() {
            fs::write(default_irq_path, format!("{:x}", all_cores_mask))
                .context("Failed to reset default IRQ affinity")?;
        }
        
        // Reset IRQ affinity for all IRQs to all cores
        let irq_dir = Path::new("/proc/irq");
        if irq_dir.exists() && irq_dir.is_dir() {
            for entry in fs::read_dir(irq_dir)
                .context("Failed to read IRQ directory")? {
                let entry = entry.context("Failed to read IRQ directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(irq_num) = path.file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|s| s.parse::<u32>().ok()) {
                        
                        let affinity_path = path.join("smp_affinity");
                        if affinity_path.exists() {
                            fs::write(&affinity_path, format!("{:x}", all_cores_mask))
                                .unwrap_or_else(|e| warn!("Failed to reset IRQ affinity for IRQ {}: {}", irq_num, e));
                        }
                    }
                }
            }
        }
        
        // Enable IRQ balancing
        let output = Command::new("systemctl")
            .arg("start")
            .arg("irqbalance.service")
            .output()
            .context("Failed to execute systemctl command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to start IRQ balancing service: {}", error);
        }
        
        Ok(())
    }
    
    /// Apply idle states optimization.
    fn apply_idle_states_optimization(&self) -> Result<()> {
        info!("Applying idle states optimization");
        
        for core in &self.topology.vr_cores {
            // Disable all idle states for VR cores
            let cpu_path = Path::new("/sys/devices/system/cpu").join(format!("cpu{}", core));
            let cpuidle_path = cpu_path.join("cpuidle");
            
            if cpuidle_path.exists() && cpuidle_path.is_dir() {
                for entry in fs::read_dir(&cpuidle_path)
                    .context("Failed to read cpuidle directory")? {
                    let entry = entry.context("Failed to read cpuidle directory entry")?;
                    let path = entry.path();
                    
                    if path.is_dir() {
                        if let Some(state) = path.file_name()
                            .and_then(|n| n.to_str())
                            .filter(|s| s.starts_with("state")) {
                            
                            let disable_path = path.join("disable");
                            if disable_path.exists() {
                                fs::write(&disable_path, "1")
                                    .unwrap_or_else(|e| warn!("Failed to disable idle state {} for core {}: {}", state, core, e));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Reset idle states.
    fn reset_idle_states(&self) -> Result<()> {
        info!("Resetting idle states");
        
        for core in 0..self.topology.num_cores {
            // Enable all idle states for all cores
            let cpu_path = Path::new("/sys/devices/system/cpu").join(format!("cpu{}", core));
            let cpuidle_path = cpu_path.join("cpuidle");
            
            if cpuidle_path.exists() && cpuidle_path.is_dir() {
                for entry in fs::read_dir(&cpuidle_path)
                    .context("Failed to read cpuidle directory")? {
                    let entry = entry.context("Failed to read cpuidle directory entry")?;
                    let path = entry.path();
                    
                    if path.is_dir() {
                        if let Some(state) = path.file_name()
                            .and_then(|n| n.to_str())
                            .filter(|s| s.starts_with("state")) {
                            
                            let disable_path = path.join("disable");
                            if disable_path.exists() {
                                fs::write(&disable_path, "0")
                                    .unwrap_or_else(|e| warn!("Failed to enable idle state {} for core {}: {}", state, core, e));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply cache optimization.
    fn apply_cache_optimization(&self) -> Result<()> {
        info!("Applying cache optimization");
        
        // Set cache coherency policy
        let coherency_path = Path::new("/sys/kernel/mm/coherent_pool/coherent_pool_size");
        if coherency_path.exists() {
            fs::write(coherency_path, "16777216") // 16MB
                .unwrap_or_else(|e| warn!("Failed to set cache coherency pool size: {}", e));
        }
        
        // Set cache allocation policy
        let allocation_path = Path::new("/sys/kernel/mm/transparent_hugepage/enabled");
        if allocation_path.exists() {
            fs::write(allocation_path, "madvise")
                .unwrap_or_else(|e| warn!("Failed to set transparent hugepage policy: {}", e));
        }
        
        // Set cache defrag policy
        let defrag_path = Path::new("/sys/kernel/mm/transparent_hugepage/defrag");
        if defrag_path.exists() {
            fs::write(defrag_path, "madvise")
                .unwrap_or_else(|e| warn!("Failed to set transparent hugepage defrag policy: {}", e));
        }
        
        // Set cache shmem policy
        let shmem_path = Path::new("/sys/kernel/mm/transparent_hugepage/shmem_enabled");
        if shmem_path.exists() {
            fs::write(shmem_path, "advise")
                .unwrap_or_else(|e| warn!("Failed to set transparent hugepage shmem policy: {}", e));
        }
        
        // Set cache khugepaged settings
        let khugepaged_dir = Path::new("/sys/kernel/mm/transparent_hugepage/khugepaged");
        if khugepaged_dir.exists() && khugepaged_dir.is_dir() {
            // Set scan sleep milliseconds
            let scan_sleep_path = khugepaged_dir.join("scan_sleep_millisecs");
            if scan_sleep_path.exists() {
                fs::write(&scan_sleep_path, "10000") // 10 seconds
                    .unwrap_or_else(|e| warn!("Failed to set khugepaged scan sleep: {}", e));
            }
            
            // Set alloc sleep milliseconds
            let alloc_sleep_path = khugepaged_dir.join("alloc_sleep_millisecs");
            if alloc_sleep_path.exists() {
                fs::write(&alloc_sleep_path, "60000") // 60 seconds
                    .unwrap_or_else(|e| warn!("Failed to set khugepaged alloc sleep: {}", e));
            }
            
            // Set max ptes to scan
            let max_ptes_path = khugepaged_dir.join("max_ptes_none");
            if max_ptes_path.exists() {
                fs::write(&max_ptes_path, "511")
                    .unwrap_or_else(|e| warn!("Failed to set khugepaged max ptes: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Apply thermal management.
    fn apply_thermal_management(&self) -> Result<()> {
        info!("Applying thermal management");
        
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
        
        // Set CPU frequency scaling governor for thermal management
        let thermal_governor_path = Path::new("/sys/class/thermal/thermal_zone0/policy");
        if thermal_governor_path.exists() {
            fs::write(thermal_governor_path, "step_wise")
                .unwrap_or_else(|e| warn!("Failed to set thermal governor: {}", e));
        }
        
        Ok(())
    }
    
    /// Set CPU governor for a specific core.
    fn set_cpu_governor(&self, core: usize, governor: CpuGovernor) -> Result<()> {
        debug!("Setting CPU governor for core {} to {}", core, governor.to_str());
        
        let governor_path = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", core))
            .join("cpufreq")
            .join("scaling_governor");
        
        if governor_path.exists() {
            fs::write(&governor_path, governor.to_str())
                .context(format!("Failed to set CPU governor for core {}", core))?;
        } else {
            warn!("CPU governor path does not exist for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU governor for a specific core (static method).
    fn set_cpu_governor_static(core: usize, governor: CpuGovernor) -> Result<()> {
        debug!("Setting CPU governor for core {} to {}", core, governor.to_str());
        
        let governor_path = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", core))
            .join("cpufreq")
            .join("scaling_governor");
        
        if governor_path.exists() {
            fs::write(&governor_path, governor.to_str())
                .context(format!("Failed to set CPU governor for core {}", core))?;
        } else {
            warn!("CPU governor path does not exist for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU maximum frequency for a specific core.
    fn set_cpu_max_freq(&self, core: usize, freq: u32) -> Result<()> {
        debug!("Setting CPU maximum frequency for core {} to {} kHz", core, freq);
        
        let freq_path = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", core))
            .join("cpufreq")
            .join("scaling_max_freq");
        
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context(format!("Failed to set CPU maximum frequency for core {}", core))?;
        } else {
            warn!("CPU frequency path does not exist for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU maximum frequency for a specific core (static method).
    fn set_cpu_max_freq_static(core: usize, freq: u32) -> Result<()> {
        debug!("Setting CPU maximum frequency for core {} to {} kHz", core, freq);
        
        let freq_path = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", core))
            .join("cpufreq")
            .join("scaling_max_freq");
        
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context(format!("Failed to set CPU maximum frequency for core {}", core))?;
        } else {
            warn!("CPU frequency path does not exist for core {}", core);
        }
        
        Ok(())
    }
    
    /// Set CPU minimum frequency for a specific core.
    fn set_cpu_min_freq(&self, core: usize, freq: u32) -> Result<()> {
        debug!("Setting CPU minimum frequency for core {} to {} kHz", core, freq);
        
        let freq_path = Path::new("/sys/devices/system/cpu")
            .join(format!("cpu{}", core))
            .join("cpufreq")
            .join("scaling_min_freq");
        
        if freq_path.exists() {
            fs::write(&freq_path, freq.to_string())
                .context(format!("Failed to set CPU minimum frequency for core {}", core))?;
        } else {
            warn!("CPU frequency path does not exist for core {}", core);
        }
        
        Ok(())
    }
    
    /// Detect CPU topology.
    fn detect_cpu_topology() -> Result<CpuTopology> {
        info!("Detecting CPU topology");
        
        // Count CPU cores
        let cpu_dir = Path::new("/sys/devices/system/cpu");
        let mut num_cores = 0;
        
        if cpu_dir.exists() && cpu_dir.is_dir() {
            for entry in fs::read_dir(cpu_dir)
                .context("Failed to read CPU directory")? {
                let entry = entry.context("Failed to read CPU directory entry")?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(cpu) = path.file_name()
                        .and_then(|n| n.to_str())
                        .filter(|s| s.starts_with("cpu"))
                        .and_then(|s| s[3..].parse::<usize>().ok()) {
                        
                        num_cores = num_cores.max(cpu + 1);
                    }
                }
            }
        }
        
        if num_cores == 0 {
            bail!("Failed to detect CPU cores");
        }
        
        // Detect CPU clusters
        let mut cores_per_cluster = Vec::new();
        let mut current_cluster = 0;
        let mut current_cluster_cores = 0;
        
        for core in 0..num_cores {
            let package_path = Path::new("/sys/devices/system/cpu")
                .join(format!("cpu{}", core))
                .join("topology")
                .join("physical_package_id");
            
            if package_path.exists() {
                let package = fs::read_to_string(&package_path)
                    .context(format!("Failed to read physical package ID for core {}", core))?
                    .trim()
                    .parse::<usize>()
                    .context(format!("Failed to parse physical package ID for core {}", core))?;
                
                if package != current_cluster {
                    if current_cluster_cores > 0 {
                        cores_per_cluster.push(current_cluster_cores);
                    }
                    
                    current_cluster = package;
                    current_cluster_cores = 1;
                } else {
                    current_cluster_cores += 1;
                }
            } else {
                // If topology information is not available, assume all cores are in one cluster
                current_cluster_cores += 1;
            }
        }
        
        if current_cluster_cores > 0 {
            cores_per_cluster.push(current_cluster_cores);
        }
        
        if cores_per_cluster.is_empty() {
            // If cluster detection failed, assume all cores are in one cluster
            cores_per_cluster.push(num_cores);
        }
        
        // Determine VR cores and system cores
        // For Orange Pi CM5, we'll use the first 2 cores for VR workloads
        let vr_cores = (0..2).collect::<Vec<_>>();
        let system_cores = (2..num_cores).collect::<Vec<_>>();
        
        // Detect available CPU governors
        let mut available_governors = Vec::new();
        let governors_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors");
        
        if governors_path.exists() {
            let governors_str = fs::read_to_string(governors_path)
                .context("Failed to read available CPU governors")?;
            
            for governor in governors_str.split_whitespace() {
                if let Ok(gov) = CpuGovernor::from_str(governor) {
                    available_governors.push(gov);
                }
            }
        } else {
            // If governors information is not available, assume common governors
            available_governors = vec![
                CpuGovernor::Performance,
                CpuGovernor::Powersave,
                CpuGovernor::Ondemand,
                CpuGovernor::Conservative,
                CpuGovernor::Schedutil,
            ];
        }
        
        // Detect available CPU frequencies
        let mut available_frequencies = Vec::new();
        let mut max_freq = 0;
        let mut min_freq = u32::MAX;
        
        let frequencies_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_frequencies");
        
        if frequencies_path.exists() {
            let frequencies_str = fs::read_to_string(frequencies_path)
                .context("Failed to read available CPU frequencies")?;
            
            for freq in frequencies_str.split_whitespace() {
                if let Ok(f) = freq.parse::<u32>() {
                    available_frequencies.push(f);
                    max_freq = max_freq.max(f);
                    min_freq = min_freq.min(f);
                }
            }
        } else {
            // If frequencies information is not available, try to read min and max frequencies
            let max_freq_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq");
            let min_freq_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq");
            
            if max_freq_path.exists() && min_freq_path.exists() {
                max_freq = fs::read_to_string(max_freq_path)
                    .context("Failed to read maximum CPU frequency")?
                    .trim()
                    .parse::<u32>()
                    .context("Failed to parse maximum CPU frequency")?;
                
                min_freq = fs::read_to_string(min_freq_path)
                    .context("Failed to read minimum CPU frequency")?
                    .trim()
                    .parse::<u32>()
                    .context("Failed to parse minimum CPU frequency")?;
                
                // Generate a range of frequencies
                let step = (max_freq - min_freq) / 10;
                if step > 0 {
                    for f in (min_freq..=max_freq).step_by(step as usize) {
                        available_frequencies.push(f);
                    }
                } else {
                    available_frequencies.push(min_freq);
                    available_frequencies.push(max_freq);
                }
            } else {
                // If no frequency information is available, use default values for Orange Pi CM5
                min_freq = 1200000; // 1.2 GHz
                max_freq = 2400000; // 2.4 GHz
                
                available_frequencies = vec![
                    1200000, // 1.2 GHz
                    1400000, // 1.4 GHz
                    1600000, // 1.6 GHz
                    1800000, // 1.8 GHz
                    2000000, // 2.0 GHz
                    2200000, // 2.2 GHz
                    2400000, // 2.4 GHz
                ];
            }
        }
        
        Ok(CpuTopology {
            num_cores,
            num_clusters: cores_per_cluster.len(),
            cores_per_cluster,
            vr_cores,
            system_cores,
            available_governors,
            available_frequencies,
            max_freq,
            min_freq,
        })
    }
    
    /// Get current CPU optimization state.
    fn get_current_state(topology: &CpuTopology) -> Result<CpuOptimizationState> {
        debug!("Getting current CPU optimization state");
        
        let mut governors = Vec::with_capacity(topology.num_cores);
        let mut frequencies = Vec::with_capacity(topology.num_cores);
        let mut temperatures = Vec::with_capacity(topology.num_cores);
        let mut utilizations = Vec::with_capacity(topology.num_cores);
        let mut isolated = Vec::with_capacity(topology.num_cores);
        let mut idle_disabled = Vec::with_capacity(topology.num_cores);
        
        // Get CPU governors and frequencies
        for core in 0..topology.num_cores {
            let governor_path = Path::new("/sys/devices/system/cpu")
                .join(format!("cpu{}", core))
                .join("cpufreq")
                .join("scaling_governor");
            
            let freq_path = Path::new("/sys/devices/system/cpu")
                .join(format!("cpu{}", core))
                .join("cpufreq")
                .join("scaling_cur_freq");
            
            let governor = if governor_path.exists() {
                let governor_str = fs::read_to_string(&governor_path)
                    .unwrap_or_else(|_| "unknown".to_string())
                    .trim()
                    .to_string();
                
                CpuGovernor::from_str(&governor_str).unwrap_or(CpuGovernor::Performance)
            } else {
                CpuGovernor::Performance
            };
            
            let frequency = if freq_path.exists() {
                fs::read_to_string(&freq_path)
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(0)
            } else {
                0
            };
            
            governors.push(governor);
            frequencies.push(frequency);
        }
        
        // Get CPU temperatures
        let thermal_dir = Path::new("/sys/class/thermal");
        if thermal_dir.exists() && thermal_dir.is_dir() {
            for core in 0..topology.num_cores {
                let mut temperature = 0;
                
                // Try to find thermal zone for this CPU core
                for entry in fs::read_dir(thermal_dir)
                    .unwrap_or_else(|_| fs::read_dir(Path::new("/")).unwrap()) {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        
                        if path.is_dir() {
                            if let Some(zone) = path.file_name()
                                .and_then(|n| n.to_str())
                                .filter(|s| s.starts_with("thermal_zone")) {
                                
                                // Check if this thermal zone is for a CPU
                                let type_path = path.join("type");
                                if type_path.exists() {
                                    let zone_type = fs::read_to_string(&type_path)
                                        .unwrap_or_else(|_| "unknown".to_string())
                                        .trim()
                                        .to_string();
                                    
                                    if zone_type.contains("cpu") || zone_type.contains("x86_pkg_temp") {
                                        // This is a CPU thermal zone, read temperature
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
                
                temperatures.push(temperature);
            }
        } else {
            // If thermal information is not available, use default values
            temperatures = vec![50; topology.num_cores];
        }
        
        // Get CPU utilization
        let stat_before = Self::read_proc_stat()?;
        thread::sleep(Duration::from_millis(100));
        let stat_after = Self::read_proc_stat()?;
        
        for core in 0..topology.num_cores {
            let utilization = if core < stat_before.len() && core < stat_after.len() {
                let before = &stat_before[core];
                let after = &stat_after[core];
                
                let before_idle = before.idle + before.iowait;
                let after_idle = after.idle + after.iowait;
                
                let before_total = before.user + before.nice + before.system + before.idle + before.iowait + before.irq + before.softirq + before.steal;
                let after_total = after.user + after.nice + after.system + after.idle + after.iowait + after.irq + after.softirq + after.steal;
                
                let idle_delta = after_idle - before_idle;
                let total_delta = after_total - before_total;
                
                if total_delta > 0 {
                    let utilization = 100 - (100 * idle_delta / total_delta);
                    utilization as u8
                } else {
                    0
                }
            } else {
                0
            };
            
            utilizations.push(utilization);
        }
        
        // Get CPU isolation status
        let isolated_str = fs::read_to_string("/sys/devices/system/cpu/isolated")
            .unwrap_or_else(|_| "".to_string())
            .trim()
            .to_string();
        
        let isolated_cores = if !isolated_str.is_empty() {
            let mut cores = Vec::new();
            
            for range in isolated_str.split(',') {
                if range.contains('-') {
                    let parts: Vec<&str> = range.split('-').collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(end)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                            for core in start..=end {
                                cores.push(core);
                            }
                        }
                    }
                } else {
                    if let Ok(core) = range.parse::<usize>() {
                        cores.push(core);
                    }
                }
            }
            
            cores
        } else {
            Vec::new()
        };
        
        for core in 0..topology.num_cores {
            isolated.push(isolated_cores.contains(&core));
        }
        
        // Get CPU idle states status
        for core in 0..topology.num_cores {
            let mut is_disabled = false;
            
            let cpuidle_path = Path::new("/sys/devices/system/cpu")
                .join(format!("cpu{}", core))
                .join("cpuidle");
            
            if cpuidle_path.exists() && cpuidle_path.is_dir() {
                let mut all_disabled = true;
                let mut has_states = false;
                
                for entry in fs::read_dir(&cpuidle_path)
                    .unwrap_or_else(|_| fs::read_dir(Path::new("/")).unwrap()) {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        
                        if path.is_dir() {
                            if let Some(state) = path.file_name()
                                .and_then(|n| n.to_str())
                                .filter(|s| s.starts_with("state")) {
                                
                                has_states = true;
                                
                                let disable_path = path.join("disable");
                                if disable_path.exists() {
                                    let disabled = fs::read_to_string(&disable_path)
                                        .unwrap_or_else(|_| "0".to_string())
                                        .trim()
                                        .parse::<u32>()
                                        .unwrap_or(0);
                                    
                                    if disabled == 0 {
                                        all_disabled = false;
                                    }
                                }
                            }
                        }
                    }
                }
                
                is_disabled = has_states && all_disabled;
            }
            
            idle_disabled.push(is_disabled);
        }
        
        // Get IRQ affinity
        let mut irq_affinity = Vec::new();
        
        let default_irq_path = Path::new("/proc/irq/default_smp_affinity");
        if default_irq_path.exists() {
            let affinity_str = fs::read_to_string(default_irq_path)
                .unwrap_or_else(|_| "".to_string())
                .trim()
                .to_string();
            
            if !affinity_str.is_empty() {
                let affinity_mask = u64::from_str_radix(&affinity_str, 16).unwrap_or(0);
                
                for core in 0..topology.num_cores {
                    if (affinity_mask & (1 << core)) != 0 {
                        irq_affinity.push(core);
                    }
                }
            }
        }
        
        // Get real-time processes
        let rt_processes = Self::find_rt_processes()?;
        
        Ok(CpuOptimizationState {
            governors,
            frequencies,
            temperatures,
            utilizations,
            isolated,
            idle_disabled,
            irq_affinity,
            rt_processes,
        })
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
    
    /// Find VR IRQs.
    fn find_vr_irqs(&self) -> Result<Vec<u32>> {
        debug!("Finding VR IRQs");
        
        let mut vr_irqs = Vec::new();
        
        // VR IRQ patterns
        let vr_patterns = [
            "bno085", "ov9281", "vr", "camera", "imu",
            "display", "gpu", "tpu", "dsi",
        ];
        
        // Read IRQ list
        let interrupts = fs::read_to_string("/proc/interrupts")
            .context("Failed to read interrupts")?;
        
        for line in interrupts.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                if let Ok(irq) = parts[0].trim().parse::<u32>() {
                    let desc = parts[1..].join(":");
                    
                    // Check if IRQ description matches VR patterns
                    if vr_patterns.iter().any(|p| desc.contains(p)) {
                        vr_irqs.push(irq);
                    }
                }
            }
        }
        
        Ok(vr_irqs)
    }
    
    /// Find real-time processes.
    fn find_rt_processes() -> Result<Vec<u32>> {
        debug!("Finding real-time processes");
        
        let mut rt_processes = Vec::new();
        
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
                            // Read process scheduling info
                            let sched_path = path.join("sched");
                            if sched_path.exists() {
                                let sched = fs::read_to_string(&sched_path)
                                    .unwrap_or_else(|_| "".to_string());
                                
                                // Check if process has real-time priority
                                if sched.contains("policy:SCHED_FIFO") || sched.contains("policy:SCHED_RR") {
                                    rt_processes.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(rt_processes)
    }
    
    /// Read /proc/stat for CPU utilization.
    fn read_proc_stat() -> Result<Vec<CpuStat>> {
        let stat = fs::read_to_string("/proc/stat")
            .context("Failed to read /proc/stat")?;
        
        let mut cpu_stats = Vec::new();
        
        for line in stat.lines() {
            if line.starts_with("cpu") && !line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 8 {
                    let cpu_stat = CpuStat {
                        user: parts[1].parse::<u64>().unwrap_or(0),
                        nice: parts[2].parse::<u64>().unwrap_or(0),
                        system: parts[3].parse::<u64>().unwrap_or(0),
                        idle: parts[4].parse::<u64>().unwrap_or(0),
                        iowait: parts[5].parse::<u64>().unwrap_or(0),
                        irq: parts[6].parse::<u64>().unwrap_or(0),
                        softirq: parts[7].parse::<u64>().unwrap_or(0),
                        steal: if parts.len() >= 9 { parts[8].parse::<u64>().unwrap_or(0) } else { 0 },
                    };
                    
                    cpu_stats.push(cpu_stat);
                }
            }
        }
        
        Ok(cpu_stats)
    }
    
    /// Calculate CPU mask from core list.
    fn calculate_cpu_mask(&self, cores: &[usize]) -> u64 {
        let mut mask = 0;
        
        for core in cores {
            mask |= 1 << core;
        }
        
        mask
    }
}

/// CPU statistics from /proc/stat.
#[derive(Debug, Clone)]
struct CpuStat {
    /// User time
    user: u64,
    
    /// Nice time
    nice: u64,
    
    /// System time
    system: u64,
    
    /// Idle time
    idle: u64,
    
    /// I/O wait time
    iowait: u64,
    
    /// IRQ time
    irq: u64,
    
    /// Soft IRQ time
    softirq: u64,
    
    /// Steal time
    steal: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_governor_conversion() {
        assert_eq!(CpuGovernor::Performance.to_str(), "performance");
        assert_eq!(CpuGovernor::Powersave.to_str(), "powersave");
        assert_eq!(CpuGovernor::Ondemand.to_str(), "ondemand");
        assert_eq!(CpuGovernor::Conservative.to_str(), "conservative");
        assert_eq!(CpuGovernor::Schedutil.to_str(), "schedutil");
        assert_eq!(CpuGovernor::Userspace.to_str(), "userspace");
        
        assert_eq!(CpuGovernor::from_str("performance").unwrap(), CpuGovernor::Performance);
        assert_eq!(CpuGovernor::from_str("powersave").unwrap(), CpuGovernor::Powersave);
        assert_eq!(CpuGovernor::from_str("ondemand").unwrap(), CpuGovernor::Ondemand);
        assert_eq!(CpuGovernor::from_str("conservative").unwrap(), CpuGovernor::Conservative);
        assert_eq!(CpuGovernor::from_str("schedutil").unwrap(), CpuGovernor::Schedutil);
        assert_eq!(CpuGovernor::from_str("userspace").unwrap(), CpuGovernor::Userspace);
        
        assert!(CpuGovernor::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_cpu_optimization_settings_default() {
        let settings = CpuOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert_eq!(settings.vr_governor, CpuGovernor::Performance);
        assert_eq!(settings.system_governor, CpuGovernor::Schedutil);
        assert_eq!(settings.vr_max_freq, 2400000);
        assert_eq!(settings.vr_min_freq, 2400000);
        assert_eq!(settings.system_max_freq, 2400000);
        assert_eq!(settings.system_min_freq, 1200000);
        assert!(settings.isolate_vr_cores);
        assert!(settings.use_rt_scheduling);
        assert_eq!(settings.rt_priority, 80);
        assert_eq!(settings.nice_value, -20);
        assert!(settings.optimize_irq);
        assert!(settings.disable_idle_states);
        assert!(settings.optimize_cache);
        assert!(settings.thermal_management);
        assert_eq!(settings.max_temperature, 85);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 1000);
    }
}
