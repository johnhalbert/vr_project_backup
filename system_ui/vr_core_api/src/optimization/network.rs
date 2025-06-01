//! Network optimization module for the VR headset system.
//!
//! This module provides network optimization capabilities specifically tailored for
//! the Orange Pi CM5 platform with 16GB RAM. It manages TCP/IP settings, QoS,
//! and other network-related optimizations to maximize performance for VR workloads.

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

/// Network optimization manager for the Orange Pi CM5 platform.
#[derive(Debug)]
pub struct NetworkOptimizationManager {
    /// Network optimization settings
    settings: NetworkOptimizationSettings,
    
    /// Network information
    info: NetworkInfo,
    
    /// Current network optimization state
    state: NetworkOptimizationState,
    
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
    
    /// Current network optimization settings
    settings: NetworkOptimizationSettings,
}

/// Network optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizationSettings {
    /// Whether network optimization is enabled
    pub enabled: bool,
    
    /// Whether to optimize TCP settings
    pub optimize_tcp: bool,
    
    /// Whether to optimize UDP settings
    pub optimize_udp: bool,
    
    /// Whether to optimize IP settings
    pub optimize_ip: bool,
    
    /// Whether to optimize QoS
    pub optimize_qos: bool,
    
    /// Whether to optimize DNS
    pub optimize_dns: bool,
    
    /// Whether to optimize network interfaces
    pub optimize_interfaces: bool,
    
    /// Whether to optimize Wi-Fi
    pub optimize_wifi: bool,
    
    /// Whether to optimize Bluetooth
    pub optimize_bluetooth: bool,
    
    /// Whether to optimize network buffer sizes
    pub optimize_buffers: bool,
    
    /// TCP congestion control algorithm
    pub tcp_congestion_control: TcpCongestionControl,
    
    /// TCP receive window size (in KB)
    pub tcp_rmem_max: u32,
    
    /// TCP send window size (in KB)
    pub tcp_wmem_max: u32,
    
    /// UDP receive buffer size (in KB)
    pub udp_rmem_max: u32,
    
    /// UDP send buffer size (in KB)
    pub udp_wmem_max: u32,
    
    /// IP default TTL
    pub ip_default_ttl: u8,
    
    /// Whether to enable TCP fast open
    pub tcp_fastopen: bool,
    
    /// Whether to enable TCP window scaling
    pub tcp_window_scaling: bool,
    
    /// Whether to enable TCP timestamps
    pub tcp_timestamps: bool,
    
    /// Whether to enable TCP SACK
    pub tcp_sack: bool,
    
    /// Whether to enable TCP low latency
    pub tcp_low_latency: bool,
    
    /// Whether to enable QoS for VR traffic
    pub qos_for_vr: bool,
    
    /// Whether to prioritize VR traffic
    pub prioritize_vr_traffic: bool,
    
    /// Whether to use adaptive optimization
    pub adaptive: bool,
    
    /// Interval for adaptive optimization (in milliseconds)
    pub adaptive_interval_ms: u64,
}

/// TCP congestion control algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TcpCongestionControl {
    /// Cubic congestion control
    Cubic,
    
    /// BBR congestion control
    Bbr,
    
    /// Reno congestion control
    Reno,
    
    /// Vegas congestion control
    Vegas,
    
    /// Westwood congestion control
    Westwood,
    
    /// Illinois congestion control
    Illinois,
}

impl TcpCongestionControl {
    /// Convert TCP congestion control algorithm to string.
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Cubic => "cubic",
            Self::Bbr => "bbr",
            Self::Reno => "reno",
            Self::Vegas => "vegas",
            Self::Westwood => "westwood",
            Self::Illinois => "illinois",
        }
    }
    
    /// Parse TCP congestion control algorithm from string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "cubic" => Ok(Self::Cubic),
            "bbr" => Ok(Self::Bbr),
            "reno" => Ok(Self::Reno),
            "vegas" => Ok(Self::Vegas),
            "westwood" => Ok(Self::Westwood),
            "illinois" => Ok(Self::Illinois),
            _ => bail!("Unknown TCP congestion control algorithm: {}", s),
        }
    }
}

/// Network information.
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
    
    /// Available TCP congestion control algorithms
    pub available_congestion_controls: Vec<TcpCongestionControl>,
    
    /// Whether QoS is supported
    pub supports_qos: bool,
    
    /// Whether Wi-Fi is available
    pub has_wifi: bool,
    
    /// Whether Bluetooth is available
    pub has_bluetooth: bool,
}

/// Network interface information.
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    
    /// Interface type
    pub interface_type: NetworkInterfaceType,
    
    /// MAC address
    pub mac_address: String,
    
    /// IP addresses
    pub ip_addresses: Vec<String>,
    
    /// Link speed (in Mbps)
    pub link_speed: u32,
    
    /// MTU
    pub mtu: u32,
    
    /// Whether interface is up
    pub is_up: bool,
    
    /// Whether interface supports QoS
    pub supports_qos: bool,
    
    /// TX queue length
    pub txqueuelen: u32,
}

/// Network interface type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkInterfaceType {
    /// Ethernet interface
    Ethernet,
    
    /// Wi-Fi interface
    Wifi,
    
    /// Loopback interface
    Loopback,
    
    /// Virtual interface
    Virtual,
    
    /// Unknown interface type
    Unknown,
}

/// Network optimization state.
#[derive(Debug, Clone)]
pub struct NetworkOptimizationState {
    /// Current TCP congestion control algorithm
    pub tcp_congestion_control: TcpCongestionControl,
    
    /// Current TCP receive window size (in KB)
    pub tcp_rmem_max: u32,
    
    /// Current TCP send window size (in KB)
    pub tcp_wmem_max: u32,
    
    /// Current UDP receive buffer size (in KB)
    pub udp_rmem_max: u32,
    
    /// Current UDP send buffer size (in KB)
    pub udp_wmem_max: u32,
    
    /// Current IP default TTL
    pub ip_default_ttl: u8,
    
    /// Whether TCP fast open is enabled
    pub tcp_fastopen: bool,
    
    /// Whether TCP window scaling is enabled
    pub tcp_window_scaling: bool,
    
    /// Whether TCP timestamps are enabled
    pub tcp_timestamps: bool,
    
    /// Whether TCP SACK is enabled
    pub tcp_sack: bool,
    
    /// Whether TCP low latency is enabled
    pub tcp_low_latency: bool,
    
    /// Whether QoS for VR traffic is enabled
    pub qos_for_vr: bool,
    
    /// Network interface states
    pub interface_states: Vec<NetworkInterfaceState>,
}

/// Network interface state.
#[derive(Debug, Clone)]
pub struct NetworkInterfaceState {
    /// Interface name
    pub name: String,
    
    /// Whether interface is up
    pub is_up: bool,
    
    /// Current MTU
    pub mtu: u32,
    
    /// Current TX queue length
    pub txqueuelen: u32,
    
    /// Current link speed (in Mbps)
    pub link_speed: u32,
    
    /// Current RX bytes
    pub rx_bytes: u64,
    
    /// Current TX bytes
    pub tx_bytes: u64,
    
    /// Current RX packets
    pub rx_packets: u64,
    
    /// Current TX packets
    pub tx_packets: u64,
    
    /// Current RX errors
    pub rx_errors: u64,
    
    /// Current TX errors
    pub tx_errors: u64,
    
    /// Current RX dropped
    pub rx_dropped: u64,
    
    /// Current TX dropped
    pub tx_dropped: u64,
}

impl Default for NetworkOptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            optimize_tcp: true,
            optimize_udp: true,
            optimize_ip: true,
            optimize_qos: true,
            optimize_dns: true,
            optimize_interfaces: true,
            optimize_wifi: true,
            optimize_bluetooth: true,
            optimize_buffers: true,
            tcp_congestion_control: TcpCongestionControl::Bbr,
            tcp_rmem_max: 16777216 / 1024, // 16 MB in KB
            tcp_wmem_max: 16777216 / 1024, // 16 MB in KB
            udp_rmem_max: 8388608 / 1024,  // 8 MB in KB
            udp_wmem_max: 8388608 / 1024,  // 8 MB in KB
            ip_default_ttl: 64,
            tcp_fastopen: true,
            tcp_window_scaling: true,
            tcp_timestamps: true,
            tcp_sack: true,
            tcp_low_latency: true,
            qos_for_vr: true,
            prioritize_vr_traffic: true,
            adaptive: true,
            adaptive_interval_ms: 5000,
        }
    }
}

impl NetworkOptimizationManager {
    /// Create a new network optimization manager.
    pub fn new() -> Result<Self> {
        let info = Self::detect_network_info()?;
        let settings = NetworkOptimizationSettings::default();
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
    
    /// Initialize network optimization.
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing network optimization for Orange Pi CM5");
        
        // Detect network information
        self.info = Self::detect_network_info()?;
        
        // Get current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Start background thread if adaptive optimization is enabled
        if self.settings.adaptive {
            self.start_background_thread()?;
        }
        
        info!("Network optimization initialized successfully");
        Ok(())
    }
    
    /// Apply network optimizations.
    pub fn apply_optimizations(&mut self, global_settings: &super::OptimizationSettings) -> Result<()> {
        if !self.settings.enabled || !global_settings.enabled {
            return Ok(());
        }
        
        info!("Applying network optimizations");
        
        // Update settings based on global settings
        self.update_settings_from_global(global_settings);
        
        // Apply TCP optimizations if enabled
        if self.settings.optimize_tcp {
            self.apply_tcp_optimization()?;
        }
        
        // Apply UDP optimizations if enabled
        if self.settings.optimize_udp {
            self.apply_udp_optimization()?;
        }
        
        // Apply IP optimizations if enabled
        if self.settings.optimize_ip {
            self.apply_ip_optimization()?;
        }
        
        // Apply QoS optimizations if enabled
        if self.settings.optimize_qos {
            self.apply_qos_optimization()?;
        }
        
        // Apply DNS optimizations if enabled
        if self.settings.optimize_dns {
            self.apply_dns_optimization()?;
        }
        
        // Apply interface optimizations if enabled
        if self.settings.optimize_interfaces {
            self.apply_interface_optimization()?;
        }
        
        // Apply Wi-Fi optimizations if enabled
        if self.settings.optimize_wifi && self.info.has_wifi {
            self.apply_wifi_optimization()?;
        }
        
        // Apply Bluetooth optimizations if enabled
        if self.settings.optimize_bluetooth && self.info.has_bluetooth {
            self.apply_bluetooth_optimization()?;
        }
        
        // Apply buffer optimizations if enabled
        if self.settings.optimize_buffers {
            self.apply_buffer_optimization()?;
        }
        
        // Update current state
        self.state = Self::get_current_state(&self.info)?;
        
        // Update last optimization time
        self.last_optimization_time = Instant::now();
        
        info!("Network optimizations applied successfully");
        Ok(())
    }
    
    /// Reset network optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        info!("Resetting network optimizations");
        
        // Reset TCP settings
        self.reset_tcp_settings()?;
        
        // Reset UDP settings
        self.reset_udp_settings()?;
        
        // Reset IP settings
        self.reset_ip_settings()?;
        
        // Reset QoS settings
        self.reset_qos_settings()?;
        
        // Reset interface settings
        self.reset_interface_settings()?;
        
        info!("Network optimizations reset successfully");
        Ok(())
    }
    
    /// Update network optimization settings.
    pub fn update_settings(&mut self, settings: NetworkOptimizationSettings) -> Result<()> {
        info!("Updating network optimization settings");
        
        // Update settings
        self.settings = settings;
        
        // Update shared state
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.settings = self.settings.clone();
        
        // Apply optimizations with new settings
        self.apply_optimizations(&super::OptimizationSettings::default())?;
        
        info!("Network optimization settings updated successfully");
        Ok(())
    }
    
    /// Get current network optimization settings.
    pub fn get_settings(&self) -> NetworkOptimizationSettings {
        self.settings.clone()
    }
    
    /// Get current network optimization state.
    pub fn get_state(&self) -> NetworkOptimizationState {
        self.state.clone()
    }
    
    /// Get network information.
    pub fn get_info(&self) -> NetworkInfo {
        self.info.clone()
    }
    
    /// Start background optimization thread.
    fn start_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_some() {
            return Ok(());
        }
        
        info!("Starting background network optimization thread");
        
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
        
        info!("Background network optimization thread started");
        Ok(())
    }
    
    /// Stop background optimization thread.
    pub fn stop_background_thread(&mut self) -> Result<()> {
        if self.background_thread.is_none() {
            return Ok(());
        }
        
        info!("Stopping background network optimization thread");
        
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
        
        info!("Background network optimization thread stopped");
        Ok(())
    }
    
    /// Perform adaptive optimization.
    fn perform_adaptive_optimization(info: &NetworkInfo, settings: &NetworkOptimizationSettings) -> Result<()> {
        debug!("Performing adaptive network optimization");
        
        // Get current network state
        let state = Self::get_current_state(info)?;
        
        // Check network interface states
        for interface_state in &state.interface_states {
            // Skip loopback interface
            if interface_state.name == "lo" {
                continue;
            }
            
            // Calculate packet loss rate
            let total_rx = interface_state.rx_packets + interface_state.rx_dropped + interface_state.rx_errors;
            let total_tx = interface_state.tx_packets + interface_state.tx_dropped + interface_state.tx_errors;
            
            let rx_loss_rate = if total_rx > 0 {
                (interface_state.rx_dropped + interface_state.rx_errors) as f32 / total_rx as f32
            } else {
                0.0
            };
            
            let tx_loss_rate = if total_tx > 0 {
                (interface_state.tx_dropped + interface_state.tx_errors) as f32 / total_tx as f32
            } else {
                0.0
            };
            
            // Adjust MTU if packet loss is high
            if rx_loss_rate > 0.01 || tx_loss_rate > 0.01 {
                // Find interface
                if let Some(interface) = info.interfaces.iter().find(|i| i.name == interface_state.name) {
                    // Reduce MTU by 10%
                    let new_mtu = (interface_state.mtu as f32 * 0.9) as u32;
                    
                    // Ensure MTU is at least 1280 (minimum for IPv6)
                    let new_mtu = new_mtu.max(1280);
                    
                    if new_mtu != interface_state.mtu {
                        if let Err(e) = Self::set_interface_mtu(&interface_state.name, new_mtu) {
                            warn!("Error setting MTU for {}: {}", interface_state.name, e);
                        }
                    }
                }
            }
            
            // Adjust TX queue length if TX errors are high
            if interface_state.tx_errors > 0 {
                // Find interface
                if let Some(interface) = info.interfaces.iter().find(|i| i.name == interface_state.name) {
                    // Increase TX queue length by 50%
                    let new_txqueuelen = (interface_state.txqueuelen as f32 * 1.5) as u32;
                    
                    if new_txqueuelen != interface_state.txqueuelen {
                        if let Err(e) = Self::set_interface_txqueuelen(&interface_state.name, new_txqueuelen) {
                            warn!("Error setting TX queue length for {}: {}", interface_state.name, e);
                        }
                    }
                }
            }
        }
        
        debug!("Adaptive network optimization completed");
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
                self.settings.optimize_tcp = true;
                self.settings.optimize_udp = true;
                self.settings.optimize_ip = true;
                self.settings.optimize_qos = true;
                self.settings.optimize_dns = true;
                self.settings.optimize_interfaces = true;
                self.settings.optimize_wifi = true;
                self.settings.optimize_bluetooth = true;
                self.settings.optimize_buffers = true;
                self.settings.tcp_congestion_control = TcpCongestionControl::Bbr;
                self.settings.tcp_rmem_max = 16777216 / 1024; // 16 MB in KB
                self.settings.tcp_wmem_max = 16777216 / 1024; // 16 MB in KB
                self.settings.udp_rmem_max = 8388608 / 1024;  // 8 MB in KB
                self.settings.udp_wmem_max = 8388608 / 1024;  // 8 MB in KB
                self.settings.ip_default_ttl = 64;
                self.settings.tcp_fastopen = true;
                self.settings.tcp_window_scaling = true;
                self.settings.tcp_timestamps = true;
                self.settings.tcp_sack = true;
                self.settings.tcp_low_latency = true;
                self.settings.qos_for_vr = true;
                self.settings.prioritize_vr_traffic = true;
            },
            super::OptimizationMode::Efficiency => {
                self.settings.optimize_tcp = true;
                self.settings.optimize_udp = true;
                self.settings.optimize_ip = true;
                self.settings.optimize_qos = true;
                self.settings.optimize_dns = true;
                self.settings.optimize_interfaces = true;
                self.settings.optimize_wifi = true;
                self.settings.optimize_bluetooth = true;
                self.settings.optimize_buffers = true;
                self.settings.tcp_congestion_control = TcpCongestionControl::Cubic;
                self.settings.tcp_rmem_max = 8388608 / 1024; // 8 MB in KB
                self.settings.tcp_wmem_max = 8388608 / 1024; // 8 MB in KB
                self.settings.udp_rmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.udp_wmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.ip_default_ttl = 64;
                self.settings.tcp_fastopen = true;
                self.settings.tcp_window_scaling = true;
                self.settings.tcp_timestamps = true;
                self.settings.tcp_sack = true;
                self.settings.tcp_low_latency = false;
                self.settings.qos_for_vr = true;
                self.settings.prioritize_vr_traffic = true;
            },
            super::OptimizationMode::Latency => {
                self.settings.optimize_tcp = true;
                self.settings.optimize_udp = true;
                self.settings.optimize_ip = true;
                self.settings.optimize_qos = true;
                self.settings.optimize_dns = true;
                self.settings.optimize_interfaces = true;
                self.settings.optimize_wifi = true;
                self.settings.optimize_bluetooth = true;
                self.settings.optimize_buffers = true;
                self.settings.tcp_congestion_control = TcpCongestionControl::Bbr;
                self.settings.tcp_rmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.tcp_wmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.udp_rmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.udp_wmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.ip_default_ttl = 64;
                self.settings.tcp_fastopen = true;
                self.settings.tcp_window_scaling = true;
                self.settings.tcp_timestamps = false;
                self.settings.tcp_sack = true;
                self.settings.tcp_low_latency = true;
                self.settings.qos_for_vr = true;
                self.settings.prioritize_vr_traffic = true;
            },
            super::OptimizationMode::Thermal => {
                self.settings.optimize_tcp = true;
                self.settings.optimize_udp = true;
                self.settings.optimize_ip = true;
                self.settings.optimize_qos = true;
                self.settings.optimize_dns = true;
                self.settings.optimize_interfaces = true;
                self.settings.optimize_wifi = true;
                self.settings.optimize_bluetooth = true;
                self.settings.optimize_buffers = true;
                self.settings.tcp_congestion_control = TcpCongestionControl::Cubic;
                self.settings.tcp_rmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.tcp_wmem_max = 4194304 / 1024; // 4 MB in KB
                self.settings.udp_rmem_max = 2097152 / 1024; // 2 MB in KB
                self.settings.udp_wmem_max = 2097152 / 1024; // 2 MB in KB
                self.settings.ip_default_ttl = 64;
                self.settings.tcp_fastopen = true;
                self.settings.tcp_window_scaling = true;
                self.settings.tcp_timestamps = true;
                self.settings.tcp_sack = true;
                self.settings.tcp_low_latency = false;
                self.settings.qos_for_vr = true;
                self.settings.prioritize_vr_traffic = true;
            },
            super::OptimizationMode::Balanced | super::OptimizationMode::Custom => {
                // Keep current settings
            },
        }
        
        // Apply aggressive settings if enabled
        if global_settings.aggressive {
            self.settings.tcp_congestion_control = TcpCongestionControl::Bbr;
            self.settings.tcp_rmem_max = 33554432 / 1024; // 32 MB in KB
            self.settings.tcp_wmem_max = 33554432 / 1024; // 32 MB in KB
            self.settings.udp_rmem_max = 16777216 / 1024; // 16 MB in KB
            self.settings.udp_wmem_max = 16777216 / 1024; // 16 MB in KB
            self.settings.tcp_low_latency = true;
            self.settings.qos_for_vr = true;
            self.settings.prioritize_vr_traffic = true;
        }
    }
    
    /// Apply TCP optimizations.
    fn apply_tcp_optimization(&self) -> Result<()> {
        info!("Applying TCP optimizations");
        
        // Set TCP congestion control algorithm
        self.set_tcp_congestion_control(self.settings.tcp_congestion_control)?;
        
        // Set TCP receive window size
        self.set_tcp_rmem_max(self.settings.tcp_rmem_max * 1024)?;
        
        // Set TCP send window size
        self.set_tcp_wmem_max(self.settings.tcp_wmem_max * 1024)?;
        
        // Set TCP fast open
        self.set_tcp_fastopen(self.settings.tcp_fastopen)?;
        
        // Set TCP window scaling
        self.set_tcp_window_scaling(self.settings.tcp_window_scaling)?;
        
        // Set TCP timestamps
        self.set_tcp_timestamps(self.settings.tcp_timestamps)?;
        
        // Set TCP SACK
        self.set_tcp_sack(self.settings.tcp_sack)?;
        
        // Set TCP low latency
        self.set_tcp_low_latency(self.settings.tcp_low_latency)?;
        
        // Set other TCP parameters
        self.set_tcp_parameters()?;
        
        Ok(())
    }
    
    /// Apply UDP optimizations.
    fn apply_udp_optimization(&self) -> Result<()> {
        info!("Applying UDP optimizations");
        
        // Set UDP receive buffer size
        self.set_udp_rmem_max(self.settings.udp_rmem_max * 1024)?;
        
        // Set UDP send buffer size
        self.set_udp_wmem_max(self.settings.udp_wmem_max * 1024)?;
        
        // Set other UDP parameters
        self.set_udp_parameters()?;
        
        Ok(())
    }
    
    /// Apply IP optimizations.
    fn apply_ip_optimization(&self) -> Result<()> {
        info!("Applying IP optimizations");
        
        // Set IP default TTL
        self.set_ip_default_ttl(self.settings.ip_default_ttl)?;
        
        // Set other IP parameters
        self.set_ip_parameters()?;
        
        Ok(())
    }
    
    /// Apply QoS optimizations.
    fn apply_qos_optimization(&self) -> Result<()> {
        info!("Applying QoS optimizations");
        
        if !self.info.supports_qos {
            warn!("QoS not supported");
            return Ok(());
        }
        
        if self.settings.qos_for_vr {
            // Set up QoS for VR traffic
            self.setup_qos_for_vr()?;
        }
        
        if self.settings.prioritize_vr_traffic {
            // Prioritize VR traffic
            self.prioritize_vr_traffic()?;
        }
        
        Ok(())
    }
    
    /// Apply DNS optimizations.
    fn apply_dns_optimization(&self) -> Result<()> {
        info!("Applying DNS optimizations");
        
        // Set up DNS caching
        self.setup_dns_caching()?;
        
        // Set up DNS prefetching
        self.setup_dns_prefetching()?;
        
        Ok(())
    }
    
    /// Apply interface optimizations.
    fn apply_interface_optimization(&self) -> Result<()> {
        info!("Applying interface optimizations");
        
        for interface in &self.info.interfaces {
            // Skip loopback interface
            if interface.interface_type == NetworkInterfaceType::Loopback {
                continue;
            }
            
            // Set MTU
            let mtu = match interface.interface_type {
                NetworkInterfaceType::Ethernet => 1500,
                NetworkInterfaceType::Wifi => 1500,
                NetworkInterfaceType::Virtual => 1500,
                NetworkInterfaceType::Loopback => 65536,
                NetworkInterfaceType::Unknown => 1500,
            };
            
            self.set_interface_mtu(&interface.name, mtu)?;
            
            // Set TX queue length
            let txqueuelen = match interface.interface_type {
                NetworkInterfaceType::Ethernet => 1000,
                NetworkInterfaceType::Wifi => 1000,
                NetworkInterfaceType::Virtual => 1000,
                NetworkInterfaceType::Loopback => 1000,
                NetworkInterfaceType::Unknown => 1000,
            };
            
            self.set_interface_txqueuelen(&interface.name, txqueuelen)?;
            
            // Set interface parameters
            self.set_interface_parameters(&interface.name)?;
        }
        
        Ok(())
    }
    
    /// Apply Wi-Fi optimizations.
    fn apply_wifi_optimization(&self) -> Result<()> {
        info!("Applying Wi-Fi optimizations");
        
        // Find Wi-Fi interface
        let wifi_interface = self.info.interfaces.iter()
            .find(|i| i.interface_type == NetworkInterfaceType::Wifi);
        
        if let Some(interface) = wifi_interface {
            // Set Wi-Fi power management
            self.set_wifi_power_management(&interface.name, false)?;
            
            // Set Wi-Fi parameters
            self.set_wifi_parameters(&interface.name)?;
        } else {
            warn!("No Wi-Fi interface found");
        }
        
        Ok(())
    }
    
    /// Apply Bluetooth optimizations.
    fn apply_bluetooth_optimization(&self) -> Result<()> {
        info!("Applying Bluetooth optimizations");
        
        // Set Bluetooth parameters
        self.set_bluetooth_parameters()?;
        
        Ok(())
    }
    
    /// Apply buffer optimizations.
    fn apply_buffer_optimization(&self) -> Result<()> {
        info!("Applying buffer optimizations");
        
        // Set socket buffer parameters
        self.set_socket_buffer_parameters()?;
        
        Ok(())
    }
    
    /// Reset TCP settings.
    fn reset_tcp_settings(&self) -> Result<()> {
        info!("Resetting TCP settings");
        
        // Reset TCP congestion control algorithm
        self.set_tcp_congestion_control(TcpCongestionControl::Cubic)?;
        
        // Reset TCP receive window size
        self.set_tcp_rmem_max(4194304)?; // 4 MB
        
        // Reset TCP send window size
        self.set_tcp_wmem_max(4194304)?; // 4 MB
        
        // Reset TCP fast open
        self.set_tcp_fastopen(false)?;
        
        // Reset TCP window scaling
        self.set_tcp_window_scaling(true)?;
        
        // Reset TCP timestamps
        self.set_tcp_timestamps(true)?;
        
        // Reset TCP SACK
        self.set_tcp_sack(true)?;
        
        // Reset TCP low latency
        self.set_tcp_low_latency(false)?;
        
        Ok(())
    }
    
    /// Reset UDP settings.
    fn reset_udp_settings(&self) -> Result<()> {
        info!("Resetting UDP settings");
        
        // Reset UDP receive buffer size
        self.set_udp_rmem_max(4194304)?; // 4 MB
        
        // Reset UDP send buffer size
        self.set_udp_wmem_max(4194304)?; // 4 MB
        
        Ok(())
    }
    
    /// Reset IP settings.
    fn reset_ip_settings(&self) -> Result<()> {
        info!("Resetting IP settings");
        
        // Reset IP default TTL
        self.set_ip_default_ttl(64)?;
        
        Ok(())
    }
    
    /// Reset QoS settings.
    fn reset_qos_settings(&self) -> Result<()> {
        info!("Resetting QoS settings");
        
        if !self.info.supports_qos {
            return Ok(());
        }
        
        // Remove QoS rules
        let output = Command::new("tc")
            .arg("qdisc")
            .arg("del")
            .arg("dev")
            .arg("eth0")
            .arg("root")
            .output()
            .context("Failed to execute tc command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if !error.contains("No such file or directory") {
                warn!("Failed to remove QoS rules: {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Reset interface settings.
    fn reset_interface_settings(&self) -> Result<()> {
        info!("Resetting interface settings");
        
        for interface in &self.info.interfaces {
            // Skip loopback interface
            if interface.interface_type == NetworkInterfaceType::Loopback {
                continue;
            }
            
            // Reset MTU
            let mtu = match interface.interface_type {
                NetworkInterfaceType::Ethernet => 1500,
                NetworkInterfaceType::Wifi => 1500,
                NetworkInterfaceType::Virtual => 1500,
                NetworkInterfaceType::Loopback => 65536,
                NetworkInterfaceType::Unknown => 1500,
            };
            
            self.set_interface_mtu(&interface.name, mtu)?;
            
            // Reset TX queue length
            let txqueuelen = match interface.interface_type {
                NetworkInterfaceType::Ethernet => 1000,
                NetworkInterfaceType::Wifi => 1000,
                NetworkInterfaceType::Virtual => 1000,
                NetworkInterfaceType::Loopback => 1000,
                NetworkInterfaceType::Unknown => 1000,
            };
            
            self.set_interface_txqueuelen(&interface.name, txqueuelen)?;
        }
        
        Ok(())
    }
    
    /// Set TCP congestion control algorithm.
    fn set_tcp_congestion_control(&self, algorithm: TcpCongestionControl) -> Result<()> {
        debug!("Setting TCP congestion control algorithm to {}", algorithm.to_str());
        
        // Check if algorithm is available
        if !self.info.available_congestion_controls.contains(&algorithm) {
            warn!("TCP congestion control algorithm {} not available", algorithm.to_str());
            return Ok(());
        }
        
        let algorithm_path = Path::new("/proc/sys/net/ipv4/tcp_congestion_control");
        if algorithm_path.exists() {
            fs::write(&algorithm_path, algorithm.to_str())
                .context("Failed to set TCP congestion control algorithm")?;
        } else {
            warn!("TCP congestion control setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP receive window size.
    fn set_tcp_rmem_max(&self, size: u32) -> Result<()> {
        debug!("Setting TCP receive window size to {} bytes", size);
        
        let rmem_path = Path::new("/proc/sys/net/core/rmem_max");
        if rmem_path.exists() {
            fs::write(&rmem_path, size.to_string())
                .context("Failed to set TCP receive window size")?;
        } else {
            warn!("TCP receive window size setting not supported");
        }
        
        // Also set TCP specific memory
        let tcp_rmem_path = Path::new("/proc/sys/net/ipv4/tcp_rmem");
        if tcp_rmem_path.exists() {
            let min = 4096;
            let default = size / 2;
            fs::write(&tcp_rmem_path, format!("{} {} {}", min, default, size))
                .context("Failed to set TCP receive memory")?;
        }
        
        Ok(())
    }
    
    /// Set TCP send window size.
    fn set_tcp_wmem_max(&self, size: u32) -> Result<()> {
        debug!("Setting TCP send window size to {} bytes", size);
        
        let wmem_path = Path::new("/proc/sys/net/core/wmem_max");
        if wmem_path.exists() {
            fs::write(&wmem_path, size.to_string())
                .context("Failed to set TCP send window size")?;
        } else {
            warn!("TCP send window size setting not supported");
        }
        
        // Also set TCP specific memory
        let tcp_wmem_path = Path::new("/proc/sys/net/ipv4/tcp_wmem");
        if tcp_wmem_path.exists() {
            let min = 4096;
            let default = size / 2;
            fs::write(&tcp_wmem_path, format!("{} {} {}", min, default, size))
                .context("Failed to set TCP send memory")?;
        }
        
        Ok(())
    }
    
    /// Set UDP receive buffer size.
    fn set_udp_rmem_max(&self, size: u32) -> Result<()> {
        debug!("Setting UDP receive buffer size to {} bytes", size);
        
        // UDP uses the same core settings as TCP
        let rmem_path = Path::new("/proc/sys/net/core/rmem_max");
        if rmem_path.exists() {
            // Read current value
            let current = fs::read_to_string(&rmem_path)
                .context("Failed to read TCP receive window size")?
                .trim()
                .parse::<u32>()
                .context("Failed to parse TCP receive window size")?;
            
            // Set to the maximum of current and requested size
            let max_size = current.max(size);
            fs::write(&rmem_path, max_size.to_string())
                .context("Failed to set UDP receive buffer size")?;
        } else {
            warn!("UDP receive buffer size setting not supported");
        }
        
        Ok(())
    }
    
    /// Set UDP send buffer size.
    fn set_udp_wmem_max(&self, size: u32) -> Result<()> {
        debug!("Setting UDP send buffer size to {} bytes", size);
        
        // UDP uses the same core settings as TCP
        let wmem_path = Path::new("/proc/sys/net/core/wmem_max");
        if wmem_path.exists() {
            // Read current value
            let current = fs::read_to_string(&wmem_path)
                .context("Failed to read TCP send window size")?
                .trim()
                .parse::<u32>()
                .context("Failed to parse TCP send window size")?;
            
            // Set to the maximum of current and requested size
            let max_size = current.max(size);
            fs::write(&wmem_path, max_size.to_string())
                .context("Failed to set UDP send buffer size")?;
        } else {
            warn!("UDP send buffer size setting not supported");
        }
        
        Ok(())
    }
    
    /// Set IP default TTL.
    fn set_ip_default_ttl(&self, ttl: u8) -> Result<()> {
        debug!("Setting IP default TTL to {}", ttl);
        
        let ttl_path = Path::new("/proc/sys/net/ipv4/ip_default_ttl");
        if ttl_path.exists() {
            fs::write(&ttl_path, ttl.to_string())
                .context("Failed to set IP default TTL")?;
        } else {
            warn!("IP default TTL setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP fast open.
    fn set_tcp_fastopen(&self, enabled: bool) -> Result<()> {
        debug!("Setting TCP fast open to {}", enabled);
        
        let fastopen_path = Path::new("/proc/sys/net/ipv4/tcp_fastopen");
        if fastopen_path.exists() {
            let value = if enabled { "3" } else { "0" };
            fs::write(&fastopen_path, value)
                .context("Failed to set TCP fast open")?;
        } else {
            warn!("TCP fast open setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP window scaling.
    fn set_tcp_window_scaling(&self, enabled: bool) -> Result<()> {
        debug!("Setting TCP window scaling to {}", enabled);
        
        let window_scaling_path = Path::new("/proc/sys/net/ipv4/tcp_window_scaling");
        if window_scaling_path.exists() {
            let value = if enabled { "1" } else { "0" };
            fs::write(&window_scaling_path, value)
                .context("Failed to set TCP window scaling")?;
        } else {
            warn!("TCP window scaling setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP timestamps.
    fn set_tcp_timestamps(&self, enabled: bool) -> Result<()> {
        debug!("Setting TCP timestamps to {}", enabled);
        
        let timestamps_path = Path::new("/proc/sys/net/ipv4/tcp_timestamps");
        if timestamps_path.exists() {
            let value = if enabled { "1" } else { "0" };
            fs::write(&timestamps_path, value)
                .context("Failed to set TCP timestamps")?;
        } else {
            warn!("TCP timestamps setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP SACK.
    fn set_tcp_sack(&self, enabled: bool) -> Result<()> {
        debug!("Setting TCP SACK to {}", enabled);
        
        let sack_path = Path::new("/proc/sys/net/ipv4/tcp_sack");
        if sack_path.exists() {
            let value = if enabled { "1" } else { "0" };
            fs::write(&sack_path, value)
                .context("Failed to set TCP SACK")?;
        } else {
            warn!("TCP SACK setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP low latency.
    fn set_tcp_low_latency(&self, enabled: bool) -> Result<()> {
        debug!("Setting TCP low latency to {}", enabled);
        
        let low_latency_path = Path::new("/proc/sys/net/ipv4/tcp_low_latency");
        if low_latency_path.exists() {
            let value = if enabled { "1" } else { "0" };
            fs::write(&low_latency_path, value)
                .context("Failed to set TCP low latency")?;
        } else {
            warn!("TCP low latency setting not supported");
        }
        
        Ok(())
    }
    
    /// Set TCP parameters.
    fn set_tcp_parameters(&self) -> Result<()> {
        debug!("Setting TCP parameters");
        
        // Set TCP FIN timeout
        let fin_timeout_path = Path::new("/proc/sys/net/ipv4/tcp_fin_timeout");
        if fin_timeout_path.exists() {
            fs::write(&fin_timeout_path, "15")
                .context("Failed to set TCP FIN timeout")?;
        }
        
        // Set TCP keepalive time
        let keepalive_time_path = Path::new("/proc/sys/net/ipv4/tcp_keepalive_time");
        if keepalive_time_path.exists() {
            fs::write(&keepalive_time_path, "600")
                .context("Failed to set TCP keepalive time")?;
        }
        
        // Set TCP keepalive probes
        let keepalive_probes_path = Path::new("/proc/sys/net/ipv4/tcp_keepalive_probes");
        if keepalive_probes_path.exists() {
            fs::write(&keepalive_probes_path, "5")
                .context("Failed to set TCP keepalive probes")?;
        }
        
        // Set TCP keepalive interval
        let keepalive_intvl_path = Path::new("/proc/sys/net/ipv4/tcp_keepalive_intvl");
        if keepalive_intvl_path.exists() {
            fs::write(&keepalive_intvl_path, "15")
                .context("Failed to set TCP keepalive interval")?;
        }
        
        // Set TCP retries
        let retries_path = Path::new("/proc/sys/net/ipv4/tcp_retries2");
        if retries_path.exists() {
            fs::write(&retries_path, "5")
                .context("Failed to set TCP retries")?;
        }
        
        // Set TCP slow start after idle
        let slow_start_path = Path::new("/proc/sys/net/ipv4/tcp_slow_start_after_idle");
        if slow_start_path.exists() {
            fs::write(&slow_start_path, "0")
                .context("Failed to set TCP slow start after idle")?;
        }
        
        // Set TCP no metrics save
        let metrics_path = Path::new("/proc/sys/net/ipv4/tcp_no_metrics_save");
        if metrics_path.exists() {
            fs::write(&metrics_path, "1")
                .context("Failed to set TCP no metrics save")?;
        }
        
        // Set TCP max SYN backlog
        let backlog_path = Path::new("/proc/sys/net/ipv4/tcp_max_syn_backlog");
        if backlog_path.exists() {
            fs::write(&backlog_path, "4096")
                .context("Failed to set TCP max SYN backlog")?;
        }
        
        // Set TCP max TWRecycle
        let tw_recycle_path = Path::new("/proc/sys/net/ipv4/tcp_tw_recycle");
        if tw_recycle_path.exists() {
            fs::write(&tw_recycle_path, "0")
                .context("Failed to set TCP max TWRecycle")?;
        }
        
        // Set TCP max TWReuse
        let tw_reuse_path = Path::new("/proc/sys/net/ipv4/tcp_tw_reuse");
        if tw_reuse_path.exists() {
            fs::write(&tw_reuse_path, "1")
                .context("Failed to set TCP max TWReuse")?;
        }
        
        Ok(())
    }
    
    /// Set UDP parameters.
    fn set_udp_parameters(&self) -> Result<()> {
        debug!("Setting UDP parameters");
        
        // Set UDP memory
        let udp_mem_path = Path::new("/proc/sys/net/ipv4/udp_mem");
        if udp_mem_path.exists() {
            let min = 4096;
            let pressure = 8388608 / 4096; // 8 MB in pages
            let max = 16777216 / 4096;     // 16 MB in pages
            fs::write(&udp_mem_path, format!("{} {} {}", min, pressure, max))
                .context("Failed to set UDP memory")?;
        }
        
        Ok(())
    }
    
    /// Set IP parameters.
    fn set_ip_parameters(&self) -> Result<()> {
        debug!("Setting IP parameters");
        
        // Set IP forward
        let forward_path = Path::new("/proc/sys/net/ipv4/ip_forward");
        if forward_path.exists() {
            fs::write(&forward_path, "0")
                .context("Failed to set IP forward")?;
        }
        
        // Set IP local port range
        let port_range_path = Path::new("/proc/sys/net/ipv4/ip_local_port_range");
        if port_range_path.exists() {
            fs::write(&port_range_path, "32768 60999")
                .context("Failed to set IP local port range")?;
        }
        
        // Set IP no pmtu discovery
        let pmtu_path = Path::new("/proc/sys/net/ipv4/ip_no_pmtu_disc");
        if pmtu_path.exists() {
            fs::write(&pmtu_path, "0")
                .context("Failed to set IP no pmtu discovery")?;
        }
        
        Ok(())
    }
    
    /// Set up QoS for VR traffic.
    fn setup_qos_for_vr(&self) -> Result<()> {
        debug!("Setting up QoS for VR traffic");
        
        // Find network interfaces
        for interface in &self.info.interfaces {
            // Skip loopback interface
            if interface.interface_type == NetworkInterfaceType::Loopback {
                continue;
            }
            
            // Skip interfaces that don't support QoS
            if !interface.supports_qos {
                continue;
            }
            
            // Set up HTB qdisc
            let output = Command::new("tc")
                .arg("qdisc")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("root")
                .arg("handle")
                .arg("1:")
                .arg("htb")
                .arg("default")
                .arg("30")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                if !error.contains("File exists") {
                    warn!("Failed to set up HTB qdisc: {}", error);
                }
            }
            
            // Set up VR traffic class
            let output = Command::new("tc")
                .arg("class")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:")
                .arg("classid")
                .arg("1:10")
                .arg("htb")
                .arg("rate")
                .arg("100mbit")
                .arg("ceil")
                .arg("100mbit")
                .arg("prio")
                .arg("1")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up VR traffic class: {}", error);
            }
            
            // Set up other traffic class
            let output = Command::new("tc")
                .arg("class")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:")
                .arg("classid")
                .arg("1:30")
                .arg("htb")
                .arg("rate")
                .arg("50mbit")
                .arg("ceil")
                .arg("100mbit")
                .arg("prio")
                .arg("2")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up other traffic class: {}", error);
            }
            
            // Set up SFQ qdisc for VR traffic
            let output = Command::new("tc")
                .arg("qdisc")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:10")
                .arg("handle")
                .arg("10:")
                .arg("sfq")
                .arg("perturb")
                .arg("10")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up SFQ qdisc for VR traffic: {}", error);
            }
            
            // Set up SFQ qdisc for other traffic
            let output = Command::new("tc")
                .arg("qdisc")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:30")
                .arg("handle")
                .arg("30:")
                .arg("sfq")
                .arg("perturb")
                .arg("10")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up SFQ qdisc for other traffic: {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Prioritize VR traffic.
    fn prioritize_vr_traffic(&self) -> Result<()> {
        debug!("Prioritizing VR traffic");
        
        // Find network interfaces
        for interface in &self.info.interfaces {
            // Skip loopback interface
            if interface.interface_type == NetworkInterfaceType::Loopback {
                continue;
            }
            
            // Skip interfaces that don't support QoS
            if !interface.supports_qos {
                continue;
            }
            
            // Set up filter for VR traffic (UDP)
            let output = Command::new("tc")
                .arg("filter")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:")
                .arg("protocol")
                .arg("ip")
                .arg("prio")
                .arg("1")
                .arg("u32")
                .arg("match")
                .arg("ip")
                .arg("protocol")
                .arg("17")
                .arg("0xff")
                .arg("flowid")
                .arg("1:10")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up filter for VR traffic (UDP): {}", error);
            }
            
            // Set up filter for VR traffic (TCP)
            let output = Command::new("tc")
                .arg("filter")
                .arg("add")
                .arg("dev")
                .arg(&interface.name)
                .arg("parent")
                .arg("1:")
                .arg("protocol")
                .arg("ip")
                .arg("prio")
                .arg("2")
                .arg("u32")
                .arg("match")
                .arg("ip")
                .arg("protocol")
                .arg("6")
                .arg("0xff")
                .arg("match")
                .arg("ip")
                .arg("dport")
                .arg("443")
                .arg("0xffff")
                .arg("flowid")
                .arg("1:10")
                .output()
                .context("Failed to execute tc command")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to set up filter for VR traffic (TCP): {}", error);
            }
        }
        
        Ok(())
    }
    
    /// Set up DNS caching.
    fn setup_dns_caching(&self) -> Result<()> {
        debug!("Setting up DNS caching");
        
        // Check if systemd-resolved is running
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg("systemd-resolved")
            .output()
            .context("Failed to execute systemctl command")?;
        
        if output.status.success() {
            // Enable DNS cache in systemd-resolved
            let resolved_conf = Path::new("/etc/systemd/resolved.conf");
            if resolved_conf.exists() {
                let mut content = fs::read_to_string(&resolved_conf)
                    .context("Failed to read resolved.conf")?;
                
                if !content.contains("Cache=yes") {
                    content.push_str("\nCache=yes\n");
                }
                
                if !content.contains("DNSStubListener=yes") {
                    content.push_str("DNSStubListener=yes\n");
                }
                
                fs::write(&resolved_conf, content)
                    .context("Failed to write resolved.conf")?;
                
                // Restart systemd-resolved
                let output = Command::new("systemctl")
                    .arg("restart")
                    .arg("systemd-resolved")
                    .output()
                    .context("Failed to execute systemctl command")?;
                
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to restart systemd-resolved: {}", error);
                }
            }
        } else {
            // Check if dnsmasq is available
            let output = Command::new("which")
                .arg("dnsmasq")
                .output()
                .context("Failed to execute which command")?;
            
            if output.status.success() {
                // Install dnsmasq if not already running
                let output = Command::new("systemctl")
                    .arg("is-active")
                    .arg("dnsmasq")
                    .output()
                    .context("Failed to execute systemctl command")?;
                
                if !output.status.success() {
                    let output = Command::new("apt-get")
                        .arg("install")
                        .arg("-y")
                        .arg("dnsmasq")
                        .output()
                        .context("Failed to execute apt-get command")?;
                    
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to install dnsmasq: {}", error);
                    }
                }
                
                // Configure dnsmasq
                let dnsmasq_conf = Path::new("/etc/dnsmasq.conf");
                if dnsmasq_conf.exists() {
                    let mut content = fs::read_to_string(&dnsmasq_conf)
                        .context("Failed to read dnsmasq.conf")?;
                    
                    if !content.contains("cache-size=1000") {
                        content.push_str("\ncache-size=1000\n");
                    }
                    
                    fs::write(&dnsmasq_conf, content)
                        .context("Failed to write dnsmasq.conf")?;
                    
                    // Restart dnsmasq
                    let output = Command::new("systemctl")
                        .arg("restart")
                        .arg("dnsmasq")
                        .output()
                        .context("Failed to execute systemctl command")?;
                    
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to restart dnsmasq: {}", error);
                    }
                }
            } else {
                warn!("No DNS caching service available");
            }
        }
        
        Ok(())
    }
    
    /// Set up DNS prefetching.
    fn setup_dns_prefetching(&self) -> Result<()> {
        debug!("Setting up DNS prefetching");
        
        // DNS prefetching is not directly supported at the system level
        // It's typically implemented in applications
        
        Ok(())
    }
    
    /// Set interface MTU.
    fn set_interface_mtu(&self, interface: &str, mtu: u32) -> Result<()> {
        debug!("Setting MTU for {} to {}", interface, mtu);
        
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("mtu")
            .arg(mtu.to_string())
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set MTU for {}: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set interface MTU (static method).
    fn set_interface_mtu_static(interface: &str, mtu: u32) -> Result<()> {
        debug!("Setting MTU for {} to {}", interface, mtu);
        
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("mtu")
            .arg(mtu.to_string())
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set MTU for {}: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set interface TX queue length.
    fn set_interface_txqueuelen(&self, interface: &str, txqueuelen: u32) -> Result<()> {
        debug!("Setting TX queue length for {} to {}", interface, txqueuelen);
        
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("txqueuelen")
            .arg(txqueuelen.to_string())
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set TX queue length for {}: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set interface TX queue length (static method).
    fn set_interface_txqueuelen_static(interface: &str, txqueuelen: u32) -> Result<()> {
        debug!("Setting TX queue length for {} to {}", interface, txqueuelen);
        
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("txqueuelen")
            .arg(txqueuelen.to_string())
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set TX queue length for {}: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set interface parameters.
    fn set_interface_parameters(&self, interface: &str) -> Result<()> {
        debug!("Setting interface parameters for {}", interface);
        
        // Set interface up
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("up")
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set interface {} up: {}", interface, error);
        }
        
        // Set interface multicast on
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("multicast")
            .arg("on")
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set interface {} multicast on: {}", interface, error);
        }
        
        // Set interface arp on
        let output = Command::new("ip")
            .arg("link")
            .arg("set")
            .arg("dev")
            .arg(interface)
            .arg("arp")
            .arg("on")
            .output()
            .context("Failed to execute ip command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set interface {} arp on: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set Wi-Fi power management.
    fn set_wifi_power_management(&self, interface: &str, enabled: bool) -> Result<()> {
        debug!("Setting Wi-Fi power management for {} to {}", interface, enabled);
        
        let output = Command::new("iw")
            .arg("dev")
            .arg(interface)
            .arg("set")
            .arg("power_save")
            .arg(if enabled { "on" } else { "off" })
            .output()
            .context("Failed to execute iw command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set Wi-Fi power management for {}: {}", interface, error);
        }
        
        Ok(())
    }
    
    /// Set Wi-Fi parameters.
    fn set_wifi_parameters(&self, interface: &str) -> Result<()> {
        debug!("Setting Wi-Fi parameters for {}", interface);
        
        // Set Wi-Fi channel
        let output = Command::new("iw")
            .arg("dev")
            .arg(interface)
            .arg("set")
            .arg("channel")
            .arg("36")
            .output()
            .context("Failed to execute iw command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if !error.contains("Device or resource busy") {
                warn!("Failed to set Wi-Fi channel for {}: {}", interface, error);
            }
        }
        
        // Set Wi-Fi bitrate
        let output = Command::new("iw")
            .arg("dev")
            .arg(interface)
            .arg("set")
            .arg("bitrates")
            .arg("ht-mcs-2.4")
            .arg("0,1,2,3,4,5,6,7")
            .output()
            .context("Failed to execute iw command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if !error.contains("command failed") {
                warn!("Failed to set Wi-Fi bitrate for {}: {}", interface, error);
            }
        }
        
        Ok(())
    }
    
    /// Set Bluetooth parameters.
    fn set_bluetooth_parameters(&self) -> Result<()> {
        debug!("Setting Bluetooth parameters");
        
        // Check if Bluetooth is available
        if !self.info.has_bluetooth {
            return Ok(());
        }
        
        // Set Bluetooth power
        let output = Command::new("bluetoothctl")
            .arg("power")
            .arg("on")
            .output()
            .context("Failed to execute bluetoothctl command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set Bluetooth power: {}", error);
        }
        
        // Set Bluetooth discoverable
        let output = Command::new("bluetoothctl")
            .arg("discoverable")
            .arg("on")
            .output()
            .context("Failed to execute bluetoothctl command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set Bluetooth discoverable: {}", error);
        }
        
        // Set Bluetooth pairable
        let output = Command::new("bluetoothctl")
            .arg("pairable")
            .arg("on")
            .output()
            .context("Failed to execute bluetoothctl command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set Bluetooth pairable: {}", error);
        }
        
        Ok(())
    }
    
    /// Set socket buffer parameters.
    fn set_socket_buffer_parameters(&self) -> Result<()> {
        debug!("Setting socket buffer parameters");
        
        // Set socket buffer parameters
        let params = [
            ("/proc/sys/net/core/rmem_default", "262144"),
            ("/proc/sys/net/core/wmem_default", "262144"),
            ("/proc/sys/net/core/rmem_max", (self.settings.tcp_rmem_max * 1024).to_string().as_str()),
            ("/proc/sys/net/core/wmem_max", (self.settings.tcp_wmem_max * 1024).to_string().as_str()),
            ("/proc/sys/net/core/netdev_max_backlog", "5000"),
            ("/proc/sys/net/core/somaxconn", "4096"),
            ("/proc/sys/net/core/optmem_max", "65536"),
        ];
        
        for (path, value) in params {
            let path = Path::new(path);
            if path.exists() {
                fs::write(path, value)
                    .unwrap_or_else(|e| warn!("Failed to set {}: {}", path.display(), e));
            }
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
    
    /// Detect network information.
    fn detect_network_info() -> Result<NetworkInfo> {
        info!("Detecting network information");
        
        let mut interfaces = Vec::new();
        let mut available_congestion_controls = Vec::new();
        let mut supports_qos = false;
        let mut has_wifi = false;
        let mut has_bluetooth = false;
        
        // Detect network interfaces
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
                    let index = parts[0].trim_end_matches(':');
                    let name = parts[1].trim_end_matches(':');
                    
                    // Skip interfaces that are not up
                    if !line.contains("UP") {
                        continue;
                    }
                    
                    // Get interface type
                    let interface_type = if name == "lo" {
                        NetworkInterfaceType::Loopback
                    } else if name.starts_with("eth") || name.starts_with("en") {
                        NetworkInterfaceType::Ethernet
                    } else if name.starts_with("wl") {
                        has_wifi = true;
                        NetworkInterfaceType::Wifi
                    } else if name.starts_with("vir") || name.starts_with("tun") || name.starts_with("tap") {
                        NetworkInterfaceType::Virtual
                    } else {
                        NetworkInterfaceType::Unknown
                    };
                    
                    // Get MAC address
                    let mac_address = if let Some(mac_index) = parts.iter().position(|&p| p == "link/ether") {
                        if parts.len() > mac_index + 1 {
                            parts[mac_index + 1].to_string()
                        } else {
                            "00:00:00:00:00:00".to_string()
                        }
                    } else {
                        "00:00:00:00:00:00".to_string()
                    };
                    
                    // Get IP addresses
                    let ip_output = Command::new("ip")
                        .arg("-o")
                        .arg("addr")
                        .arg("show")
                        .arg("dev")
                        .arg(name)
                        .output()
                        .context("Failed to execute ip command")?;
                    
                    let mut ip_addresses = Vec::new();
                    
                    if ip_output.status.success() {
                        let ip_output_str = String::from_utf8_lossy(&ip_output.stdout);
                        
                        for ip_line in ip_output_str.lines() {
                            let ip_parts: Vec<&str> = ip_line.split_whitespace().collect();
                            if ip_parts.len() >= 4 {
                                if let Some(ip_index) = ip_parts.iter().position(|&p| p == "inet" || p == "inet6") {
                                    if ip_parts.len() > ip_index + 1 {
                                        ip_addresses.push(ip_parts[ip_index + 1].to_string());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Get link speed
                    let mut link_speed = 0;
                    
                    if interface_type == NetworkInterfaceType::Ethernet {
                        let ethtool_output = Command::new("ethtool")
                            .arg(name)
                            .output()
                            .context("Failed to execute ethtool command")?;
                        
                        if ethtool_output.status.success() {
                            let ethtool_output_str = String::from_utf8_lossy(&ethtool_output.stdout);
                            
                            for ethtool_line in ethtool_output_str.lines() {
                                if ethtool_line.contains("Speed:") {
                                    if let Some(speed_str) = ethtool_line.split(':').nth(1) {
                                        let speed_str = speed_str.trim();
                                        if speed_str.ends_with("Mb/s") {
                                            if let Ok(speed) = speed_str.trim_end_matches("Mb/s").trim().parse::<u32>() {
                                                link_speed = speed;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if interface_type == NetworkInterfaceType::Wifi {
                        let iwconfig_output = Command::new("iwconfig")
                            .arg(name)
                            .output()
                            .context("Failed to execute iwconfig command")?;
                        
                        if iwconfig_output.status.success() {
                            let iwconfig_output_str = String::from_utf8_lossy(&iwconfig_output.stdout);
                            
                            for iwconfig_line in iwconfig_output_str.lines() {
                                if iwconfig_line.contains("Bit Rate=") {
                                    if let Some(rate_str) = iwconfig_line.split("Bit Rate=").nth(1) {
                                        if let Some(rate_str) = rate_str.split_whitespace().next() {
                                            if let Ok(rate) = rate_str.parse::<u32>() {
                                                link_speed = rate;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Get MTU
                    let mut mtu = 1500;
                    
                    if let Some(mtu_index) = parts.iter().position(|&p| p == "mtu") {
                        if parts.len() > mtu_index + 1 {
                            if let Ok(m) = parts[mtu_index + 1].parse::<u32>() {
                                mtu = m;
                            }
                        }
                    }
                    
                    // Get TX queue length
                    let mut txqueuelen = 1000;
                    
                    if let Some(txq_index) = parts.iter().position(|&p| p == "qlen") {
                        if parts.len() > txq_index + 1 {
                            if let Ok(q) = parts[txq_index + 1].parse::<u32>() {
                                txqueuelen = q;
                            }
                        }
                    }
                    
                    // Check if interface supports QoS
                    let tc_output = Command::new("tc")
                        .arg("qdisc")
                        .arg("show")
                        .arg("dev")
                        .arg(name)
                        .output()
                        .context("Failed to execute tc command")?;
                    
                    let interface_supports_qos = tc_output.status.success();
                    
                    if interface_supports_qos {
                        supports_qos = true;
                    }
                    
                    interfaces.push(NetworkInterface {
                        name: name.to_string(),
                        interface_type,
                        mac_address,
                        ip_addresses,
                        link_speed,
                        mtu,
                        is_up: true,
                        supports_qos: interface_supports_qos,
                        txqueuelen,
                    });
                }
            }
        }
        
        // Detect available TCP congestion control algorithms
        let cc_path = Path::new("/proc/sys/net/ipv4/tcp_available_congestion_control");
        if cc_path.exists() {
            let cc_str = fs::read_to_string(&cc_path)
                .context("Failed to read available congestion control algorithms")?;
            
            for cc in cc_str.split_whitespace() {
                match cc {
                    "cubic" => available_congestion_controls.push(TcpCongestionControl::Cubic),
                    "bbr" => available_congestion_controls.push(TcpCongestionControl::Bbr),
                    "reno" => available_congestion_controls.push(TcpCongestionControl::Reno),
                    "vegas" => available_congestion_controls.push(TcpCongestionControl::Vegas),
                    "westwood" => available_congestion_controls.push(TcpCongestionControl::Westwood),
                    "illinois" => available_congestion_controls.push(TcpCongestionControl::Illinois),
                    _ => {}
                }
            }
        }
        
        // If no congestion control algorithms were found, add default ones
        if available_congestion_controls.is_empty() {
            available_congestion_controls.push(TcpCongestionControl::Cubic);
            available_congestion_controls.push(TcpCongestionControl::Reno);
        }
        
        // Check if Bluetooth is available
        let bt_output = Command::new("bluetoothctl")
            .arg("list")
            .output()
            .context("Failed to execute bluetoothctl command")?;
        
        has_bluetooth = bt_output.status.success() && !String::from_utf8_lossy(&bt_output.stdout).trim().is_empty();
        
        Ok(NetworkInfo {
            interfaces,
            available_congestion_controls,
            supports_qos,
            has_wifi,
            has_bluetooth,
        })
    }
    
    /// Get current network optimization state.
    fn get_current_state(info: &NetworkInfo) -> Result<NetworkOptimizationState> {
        debug!("Getting current network optimization state");
        
        // Get current TCP congestion control algorithm
        let cc_path = Path::new("/proc/sys/net/ipv4/tcp_congestion_control");
        let tcp_congestion_control = if cc_path.exists() {
            let cc_str = fs::read_to_string(&cc_path)
                .context("Failed to read current congestion control algorithm")?
                .trim()
                .to_string();
            
            match cc_str.as_str() {
                "cubic" => TcpCongestionControl::Cubic,
                "bbr" => TcpCongestionControl::Bbr,
                "reno" => TcpCongestionControl::Reno,
                "vegas" => TcpCongestionControl::Vegas,
                "westwood" => TcpCongestionControl::Westwood,
                "illinois" => TcpCongestionControl::Illinois,
                _ => TcpCongestionControl::Cubic,
            }
        } else {
            TcpCongestionControl::Cubic
        };
        
        // Get current TCP receive window size
        let rmem_path = Path::new("/proc/sys/net/core/rmem_max");
        let tcp_rmem_max = if rmem_path.exists() {
            let rmem_str = fs::read_to_string(&rmem_path)
                .context("Failed to read TCP receive window size")?
                .trim()
                .to_string();
            
            rmem_str.parse::<u32>().unwrap_or(4194304) / 1024 // Convert to KB
        } else {
            4096 // 4 MB in KB
        };
        
        // Get current TCP send window size
        let wmem_path = Path::new("/proc/sys/net/core/wmem_max");
        let tcp_wmem_max = if wmem_path.exists() {
            let wmem_str = fs::read_to_string(&wmem_path)
                .context("Failed to read TCP send window size")?
                .trim()
                .to_string();
            
            wmem_str.parse::<u32>().unwrap_or(4194304) / 1024 // Convert to KB
        } else {
            4096 // 4 MB in KB
        };
        
        // Get current UDP receive buffer size (same as TCP)
        let udp_rmem_max = tcp_rmem_max;
        
        // Get current UDP send buffer size (same as TCP)
        let udp_wmem_max = tcp_wmem_max;
        
        // Get current IP default TTL
        let ttl_path = Path::new("/proc/sys/net/ipv4/ip_default_ttl");
        let ip_default_ttl = if ttl_path.exists() {
            let ttl_str = fs::read_to_string(&ttl_path)
                .context("Failed to read IP default TTL")?
                .trim()
                .to_string();
            
            ttl_str.parse::<u8>().unwrap_or(64)
        } else {
            64
        };
        
        // Get current TCP fast open
        let fastopen_path = Path::new("/proc/sys/net/ipv4/tcp_fastopen");
        let tcp_fastopen = if fastopen_path.exists() {
            let fastopen_str = fs::read_to_string(&fastopen_path)
                .context("Failed to read TCP fast open")?
                .trim()
                .to_string();
            
            fastopen_str != "0"
        } else {
            false
        };
        
        // Get current TCP window scaling
        let window_scaling_path = Path::new("/proc/sys/net/ipv4/tcp_window_scaling");
        let tcp_window_scaling = if window_scaling_path.exists() {
            let window_scaling_str = fs::read_to_string(&window_scaling_path)
                .context("Failed to read TCP window scaling")?
                .trim()
                .to_string();
            
            window_scaling_str == "1"
        } else {
            true
        };
        
        // Get current TCP timestamps
        let timestamps_path = Path::new("/proc/sys/net/ipv4/tcp_timestamps");
        let tcp_timestamps = if timestamps_path.exists() {
            let timestamps_str = fs::read_to_string(&timestamps_path)
                .context("Failed to read TCP timestamps")?
                .trim()
                .to_string();
            
            timestamps_str == "1"
        } else {
            true
        };
        
        // Get current TCP SACK
        let sack_path = Path::new("/proc/sys/net/ipv4/tcp_sack");
        let tcp_sack = if sack_path.exists() {
            let sack_str = fs::read_to_string(&sack_path)
                .context("Failed to read TCP SACK")?
                .trim()
                .to_string();
            
            sack_str == "1"
        } else {
            true
        };
        
        // Get current TCP low latency
        let low_latency_path = Path::new("/proc/sys/net/ipv4/tcp_low_latency");
        let tcp_low_latency = if low_latency_path.exists() {
            let low_latency_str = fs::read_to_string(&low_latency_path)
                .context("Failed to read TCP low latency")?
                .trim()
                .to_string();
            
            low_latency_str == "1"
        } else {
            false
        };
        
        // Get current QoS for VR traffic
        let qos_for_vr = if info.supports_qos {
            let output = Command::new("tc")
                .arg("filter")
                .arg("show")
                .output()
                .context("Failed to execute tc command")?;
            
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("flowid 1:10")
            } else {
                false
            }
        } else {
            false
        };
        
        // Get network interface states
        let mut interface_states = Vec::new();
        
        for interface in &info.interfaces {
            // Get interface state
            let output = Command::new("ip")
                .arg("-s")
                .arg("link")
                .arg("show")
                .arg("dev")
                .arg(&interface.name)
                .output()
                .context("Failed to execute ip command")?;
            
            let mut is_up = interface.is_up;
            let mut mtu = interface.mtu;
            let mut txqueuelen = interface.txqueuelen;
            let mut rx_bytes = 0;
            let mut tx_bytes = 0;
            let mut rx_packets = 0;
            let mut tx_packets = 0;
            let mut rx_errors = 0;
            let mut tx_errors = 0;
            let mut rx_dropped = 0;
            let mut tx_dropped = 0;
            
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();
                
                // Parse interface state
                if lines.len() >= 1 {
                    is_up = lines[0].contains("UP");
                    
                    if let Some(mtu_index) = lines[0].find("mtu ") {
                        let mtu_str = &lines[0][mtu_index + 4..];
                        if let Some(end_index) = mtu_str.find(char::is_whitespace) {
                            if let Ok(m) = mtu_str[..end_index].parse::<u32>() {
                                mtu = m;
                            }
                        }
                    }
                    
                    if let Some(qlen_index) = lines[0].find("qlen ") {
                        let qlen_str = &lines[0][qlen_index + 5..];
                        if let Some(end_index) = qlen_str.find(char::is_whitespace) {
                            if let Ok(q) = qlen_str[..end_index].parse::<u32>() {
                                txqueuelen = q;
                            }
                        }
                    }
                }
                
                // Parse RX statistics
                if lines.len() >= 3 {
                    let rx_parts: Vec<&str> = lines[2].split_whitespace().collect();
                    if rx_parts.len() >= 5 {
                        rx_bytes = rx_parts[0].parse::<u64>().unwrap_or(0);
                        rx_packets = rx_parts[1].parse::<u64>().unwrap_or(0);
                        rx_errors = rx_parts[2].parse::<u64>().unwrap_or(0);
                        rx_dropped = rx_parts[3].parse::<u64>().unwrap_or(0);
                    }
                }
                
                // Parse TX statistics
                if lines.len() >= 5 {
                    let tx_parts: Vec<&str> = lines[4].split_whitespace().collect();
                    if tx_parts.len() >= 5 {
                        tx_bytes = tx_parts[0].parse::<u64>().unwrap_or(0);
                        tx_packets = tx_parts[1].parse::<u64>().unwrap_or(0);
                        tx_errors = tx_parts[2].parse::<u64>().unwrap_or(0);
                        tx_dropped = tx_parts[3].parse::<u64>().unwrap_or(0);
                    }
                }
            }
            
            // Get link speed
            let mut link_speed = interface.link_speed;
            
            if interface.interface_type == NetworkInterfaceType::Ethernet {
                let ethtool_output = Command::new("ethtool")
                    .arg(&interface.name)
                    .output()
                    .context("Failed to execute ethtool command")?;
                
                if ethtool_output.status.success() {
                    let ethtool_output_str = String::from_utf8_lossy(&ethtool_output.stdout);
                    
                    for ethtool_line in ethtool_output_str.lines() {
                        if ethtool_line.contains("Speed:") {
                            if let Some(speed_str) = ethtool_line.split(':').nth(1) {
                                let speed_str = speed_str.trim();
                                if speed_str.ends_with("Mb/s") {
                                    if let Ok(speed) = speed_str.trim_end_matches("Mb/s").trim().parse::<u32>() {
                                        link_speed = speed;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if interface.interface_type == NetworkInterfaceType::Wifi {
                let iwconfig_output = Command::new("iwconfig")
                    .arg(&interface.name)
                    .output()
                    .context("Failed to execute iwconfig command")?;
                
                if iwconfig_output.status.success() {
                    let iwconfig_output_str = String::from_utf8_lossy(&iwconfig_output.stdout);
                    
                    for iwconfig_line in iwconfig_output_str.lines() {
                        if iwconfig_line.contains("Bit Rate=") {
                            if let Some(rate_str) = iwconfig_line.split("Bit Rate=").nth(1) {
                                if let Some(rate_str) = rate_str.split_whitespace().next() {
                                    if let Ok(rate) = rate_str.parse::<u32>() {
                                        link_speed = rate;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            interface_states.push(NetworkInterfaceState {
                name: interface.name.clone(),
                is_up,
                mtu,
                txqueuelen,
                link_speed,
                rx_bytes,
                tx_bytes,
                rx_packets,
                tx_packets,
                rx_errors,
                tx_errors,
                rx_dropped,
                tx_dropped,
            });
        }
        
        Ok(NetworkOptimizationState {
            tcp_congestion_control,
            tcp_rmem_max,
            tcp_wmem_max,
            udp_rmem_max,
            udp_wmem_max,
            ip_default_ttl,
            tcp_fastopen,
            tcp_window_scaling,
            tcp_timestamps,
            tcp_sack,
            tcp_low_latency,
            qos_for_vr,
            interface_states,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tcp_congestion_control_conversion() {
        assert_eq!(TcpCongestionControl::Cubic.to_str(), "cubic");
        assert_eq!(TcpCongestionControl::Bbr.to_str(), "bbr");
        assert_eq!(TcpCongestionControl::Reno.to_str(), "reno");
        assert_eq!(TcpCongestionControl::Vegas.to_str(), "vegas");
        assert_eq!(TcpCongestionControl::Westwood.to_str(), "westwood");
        assert_eq!(TcpCongestionControl::Illinois.to_str(), "illinois");
        
        assert_eq!(TcpCongestionControl::from_str("cubic").unwrap(), TcpCongestionControl::Cubic);
        assert_eq!(TcpCongestionControl::from_str("bbr").unwrap(), TcpCongestionControl::Bbr);
        assert_eq!(TcpCongestionControl::from_str("reno").unwrap(), TcpCongestionControl::Reno);
        assert_eq!(TcpCongestionControl::from_str("vegas").unwrap(), TcpCongestionControl::Vegas);
        assert_eq!(TcpCongestionControl::from_str("westwood").unwrap(), TcpCongestionControl::Westwood);
        assert_eq!(TcpCongestionControl::from_str("illinois").unwrap(), TcpCongestionControl::Illinois);
        
        assert!(TcpCongestionControl::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_network_optimization_settings_default() {
        let settings = NetworkOptimizationSettings::default();
        
        assert!(settings.enabled);
        assert!(settings.optimize_tcp);
        assert!(settings.optimize_udp);
        assert!(settings.optimize_ip);
        assert!(settings.optimize_qos);
        assert!(settings.optimize_dns);
        assert!(settings.optimize_interfaces);
        assert!(settings.optimize_wifi);
        assert!(settings.optimize_bluetooth);
        assert!(settings.optimize_buffers);
        assert_eq!(settings.tcp_congestion_control, TcpCongestionControl::Bbr);
        assert_eq!(settings.tcp_rmem_max, 16777216 / 1024);
        assert_eq!(settings.tcp_wmem_max, 16777216 / 1024);
        assert_eq!(settings.udp_rmem_max, 8388608 / 1024);
        assert_eq!(settings.udp_wmem_max, 8388608 / 1024);
        assert_eq!(settings.ip_default_ttl, 64);
        assert!(settings.tcp_fastopen);
        assert!(settings.tcp_window_scaling);
        assert!(settings.tcp_timestamps);
        assert!(settings.tcp_sack);
        assert!(settings.tcp_low_latency);
        assert!(settings.qos_for_vr);
        assert!(settings.prioritize_vr_traffic);
        assert!(settings.adaptive);
        assert_eq!(settings.adaptive_interval_ms, 5000);
    }
}
