//! Network configuration module for the VR Core API.
//!
//! This module provides comprehensive configuration management for all network
//! components of the VR headset, including WiFi, Bluetooth, streaming, firewall,
//! VPN, and QoS settings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::{IpAddr, Ipv4Addr};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::hardware::{DeviceCapability, DeviceInfo, DeviceType, NetworkDevice};
use super::{ConfigError, ConfigResult, validation};

/// Network configuration manager.
#[derive(Debug)]
pub struct NetworkConfig {
    /// WiFi configuration
    wifi: RwLock<WiFiConfig>,
    
    /// Bluetooth configuration
    bluetooth: RwLock<BluetoothConfig>,
    
    /// Streaming configuration
    streaming: RwLock<StreamingConfig>,
    
    /// Firewall configuration
    firewall: RwLock<FirewallConfig>,
    
    /// VPN configuration
    vpn: RwLock<VPNConfig>,
    
    /// QoS configuration
    qos: RwLock<QoSConfig>,
}

impl NetworkConfig {
    /// Create a new network configuration manager.
    pub fn new() -> Self {
        Self {
            wifi: RwLock::new(WiFiConfig::default()),
            bluetooth: RwLock::new(BluetoothConfig::default()),
            streaming: RwLock::new(StreamingConfig::default()),
            firewall: RwLock::new(FirewallConfig::default()),
            vpn: RwLock::new(VPNConfig::default()),
            qos: RwLock::new(QoSConfig::default()),
        }
    }
    
    /// Load network configuration from TOML values.
    pub fn load_from_toml(&self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load WiFi configuration
        if let Some(TomlValue::Table(wifi_table)) = config.get("wifi") {
            let mut wifi = self.wifi.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for WiFi config".to_string())
            })?;
            wifi.load_from_toml(wifi_table)?;
        }
        
        // Load Bluetooth configuration
        if let Some(TomlValue::Table(bluetooth_table)) = config.get("bluetooth") {
            let mut bluetooth = self.bluetooth.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for Bluetooth config".to_string())
            })?;
            bluetooth.load_from_toml(bluetooth_table)?;
        }
        
        // Load streaming configuration
        if let Some(TomlValue::Table(streaming_table)) = config.get("streaming") {
            let mut streaming = self.streaming.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for streaming config".to_string())
            })?;
            streaming.load_from_toml(streaming_table)?;
        }
        
        // Load firewall configuration
        if let Some(TomlValue::Table(firewall_table)) = config.get("firewall") {
            let mut firewall = self.firewall.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for firewall config".to_string())
            })?;
            firewall.load_from_toml(firewall_table)?;
        }
        
        // Load VPN configuration
        if let Some(TomlValue::Table(vpn_table)) = config.get("vpn") {
            let mut vpn = self.vpn.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for VPN config".to_string())
            })?;
            vpn.load_from_toml(vpn_table)?;
        }
        
        // Load QoS configuration
        if let Some(TomlValue::Table(qos_table)) = config.get("qos") {
            let mut qos = self.qos.write().map_err(|_| {
                ConfigError::LockError("Failed to acquire write lock for QoS config".to_string())
            })?;
            qos.load_from_toml(qos_table)?;
        }
        
        Ok(())
    }
    
    /// Save network configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save WiFi configuration
        let wifi = self.wifi.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for WiFi config".to_string())
        })?;
        config.insert("wifi".to_string(), TomlValue::Table(wifi.save_to_toml()?));
        
        // Save Bluetooth configuration
        let bluetooth = self.bluetooth.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for Bluetooth config".to_string())
        })?;
        config.insert("bluetooth".to_string(), TomlValue::Table(bluetooth.save_to_toml()?));
        
        // Save streaming configuration
        let streaming = self.streaming.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for streaming config".to_string())
        })?;
        config.insert("streaming".to_string(), TomlValue::Table(streaming.save_to_toml()?));
        
        // Save firewall configuration
        let firewall = self.firewall.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for firewall config".to_string())
        })?;
        config.insert("firewall".to_string(), TomlValue::Table(firewall.save_to_toml()?));
        
        // Save VPN configuration
        let vpn = self.vpn.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for VPN config".to_string())
        })?;
        config.insert("vpn".to_string(), TomlValue::Table(vpn.save_to_toml()?));
        
        // Save QoS configuration
        let qos = self.qos.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for QoS config".to_string())
        })?;
        config.insert("qos".to_string(), TomlValue::Table(qos.save_to_toml()?));
        
        Ok(config)
    }
    
    /// Get WiFi configuration.
    pub fn wifi(&self) -> ConfigResult<WiFiConfig> {
        let wifi = self.wifi.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for WiFi config".to_string())
        })?;
        Ok(wifi.clone())
    }
    
    /// Update WiFi configuration.
    pub fn update_wifi(&self, config: WiFiConfig) -> ConfigResult<()> {
        let mut wifi = self.wifi.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for WiFi config".to_string())
        })?;
        *wifi = config;
        Ok(())
    }
    
    /// Get Bluetooth configuration.
    pub fn bluetooth(&self) -> ConfigResult<BluetoothConfig> {
        let bluetooth = self.bluetooth.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for Bluetooth config".to_string())
        })?;
        Ok(bluetooth.clone())
    }
    
    /// Update Bluetooth configuration.
    pub fn update_bluetooth(&self, config: BluetoothConfig) -> ConfigResult<()> {
        let mut bluetooth = self.bluetooth.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for Bluetooth config".to_string())
        })?;
        *bluetooth = config;
        Ok(())
    }
    
    /// Get streaming configuration.
    pub fn streaming(&self) -> ConfigResult<StreamingConfig> {
        let streaming = self.streaming.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for streaming config".to_string())
        })?;
        Ok(streaming.clone())
    }
    
    /// Update streaming configuration.
    pub fn update_streaming(&self, config: StreamingConfig) -> ConfigResult<()> {
        let mut streaming = self.streaming.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for streaming config".to_string())
        })?;
        *streaming = config;
        Ok(())
    }
    
    /// Get firewall configuration.
    pub fn firewall(&self) -> ConfigResult<FirewallConfig> {
        let firewall = self.firewall.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for firewall config".to_string())
        })?;
        Ok(firewall.clone())
    }
    
    /// Update firewall configuration.
    pub fn update_firewall(&self, config: FirewallConfig) -> ConfigResult<()> {
        let mut firewall = self.firewall.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for firewall config".to_string())
        })?;
        *firewall = config;
        Ok(())
    }
    
    /// Get VPN configuration.
    pub fn vpn(&self) -> ConfigResult<VPNConfig> {
        let vpn = self.vpn.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for VPN config".to_string())
        })?;
        Ok(vpn.clone())
    }
    
    /// Update VPN configuration.
    pub fn update_vpn(&self, config: VPNConfig) -> ConfigResult<()> {
        let mut vpn = self.vpn.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for VPN config".to_string())
        })?;
        *vpn = config;
        Ok(())
    }
    
    /// Get QoS configuration.
    pub fn qos(&self) -> ConfigResult<QoSConfig> {
        let qos = self.qos.read().map_err(|_| {
            ConfigError::LockError("Failed to acquire read lock for QoS config".to_string())
        })?;
        Ok(qos.clone())
    }
    
    /// Update QoS configuration.
    pub fn update_qos(&self, config: QoSConfig) -> ConfigResult<()> {
        let mut qos = self.qos.write().map_err(|_| {
            ConfigError::LockError("Failed to acquire write lock for QoS config".to_string())
        })?;
        *qos = config;
        Ok(())
    }
    
    /// Apply network configuration to devices.
    pub fn apply_to_devices(&self, network_devices: &[Arc<dyn NetworkDevice>]) -> ConfigResult<()> {
        // Apply WiFi configuration
        let wifi_config = self.wifi()?;
        for device in network_devices {
            if device.device_type() == DeviceType::WiFi {
                if let Err(e) = wifi_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply WiFi config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        // Apply Bluetooth configuration
        let bluetooth_config = self.bluetooth()?;
        for device in network_devices {
            if device.device_type() == DeviceType::Bluetooth {
                if let Err(e) = bluetooth_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply Bluetooth config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        // Apply streaming configuration
        let streaming_config = self.streaming()?;
        for device in network_devices {
            if device.capabilities().contains(&DeviceCapability::Streaming) {
                if let Err(e) = streaming_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply streaming config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        // Apply firewall configuration
        let firewall_config = self.firewall()?;
        for device in network_devices {
            if device.capabilities().contains(&DeviceCapability::Firewall) {
                if let Err(e) = firewall_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply firewall config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        // Apply VPN configuration
        let vpn_config = self.vpn()?;
        for device in network_devices {
            if device.capabilities().contains(&DeviceCapability::VPN) {
                if let Err(e) = vpn_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply VPN config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        // Apply QoS configuration
        let qos_config = self.qos()?;
        for device in network_devices {
            if device.capabilities().contains(&DeviceCapability::QoS) {
                if let Err(e) = qos_config.apply_to_device(device.as_ref()) {
                    warn!("Failed to apply QoS config to device {}: {}", device.device_info().name, e);
                }
            }
        }
        
        Ok(())
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// WiFi configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiConfig {
    /// Whether WiFi is enabled
    pub enabled: bool,
    
    /// Whether to automatically connect to known networks
    pub auto_connect: bool,
    
    /// Whether to prefer 5GHz networks
    pub prefer_5ghz: bool,
    
    /// Whether to enable power saving mode
    pub power_saving: bool,
    
    /// Whether to enable WiFi scanning
    pub scanning_enabled: bool,
    
    /// Scan interval in seconds
    pub scan_interval_sec: u32,
    
    /// Whether to enable WiFi Direct
    pub wifi_direct_enabled: bool,
    
    /// Known networks
    pub known_networks: Vec<WiFiNetwork>,
    
    /// Hidden networks
    pub hidden_networks: Vec<WiFiNetwork>,
    
    /// Whether to enable MAC address randomization
    pub mac_randomization: bool,
    
    /// Whether to enable WiFi calling
    pub wifi_calling_enabled: bool,
    
    /// Whether to enable WiFi hotspot
    pub hotspot_enabled: bool,
    
    /// Hotspot settings
    pub hotspot: HotspotSettings,
}

impl WiFiConfig {
    /// Load WiFi configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load auto connect
        if let Some(TomlValue::Boolean(auto_connect)) = config.get("auto_connect") {
            self.auto_connect = *auto_connect;
        }
        
        // Load prefer 5GHz
        if let Some(TomlValue::Boolean(prefer_5ghz)) = config.get("prefer_5ghz") {
            self.prefer_5ghz = *prefer_5ghz;
        }
        
        // Load power saving
        if let Some(TomlValue::Boolean(power_saving)) = config.get("power_saving") {
            self.power_saving = *power_saving;
        }
        
        // Load scanning enabled
        if let Some(TomlValue::Boolean(scanning_enabled)) = config.get("scanning_enabled") {
            self.scanning_enabled = *scanning_enabled;
        }
        
        // Load scan interval
        if let Some(TomlValue::Integer(scan_interval_sec)) = config.get("scan_interval_sec") {
            self.scan_interval_sec = *scan_interval_sec as u32;
        }
        
        // Load WiFi Direct enabled
        if let Some(TomlValue::Boolean(wifi_direct_enabled)) = config.get("wifi_direct_enabled") {
            self.wifi_direct_enabled = *wifi_direct_enabled;
        }
        
        // Load known networks
        if let Some(TomlValue::Array(networks)) = config.get("known_networks") {
            self.known_networks.clear();
            for network in networks {
                if let TomlValue::Table(network_table) = network {
                    let mut wifi_network = WiFiNetwork::default();
                    if let Some(TomlValue::String(ssid)) = network_table.get("ssid") {
                        wifi_network.ssid = ssid.clone();
                    }
                    if let Some(TomlValue::String(security)) = network_table.get("security") {
                        wifi_network.security = security.clone();
                    }
                    if let Some(TomlValue::Boolean(auto_connect)) = network_table.get("auto_connect") {
                        wifi_network.auto_connect = *auto_connect;
                    }
                    if let Some(TomlValue::Integer(priority)) = network_table.get("priority") {
                        wifi_network.priority = *priority as u32;
                    }
                    self.known_networks.push(wifi_network);
                }
            }
        }
        
        // Load hidden networks
        if let Some(TomlValue::Array(networks)) = config.get("hidden_networks") {
            self.hidden_networks.clear();
            for network in networks {
                if let TomlValue::Table(network_table) = network {
                    let mut wifi_network = WiFiNetwork::default();
                    if let Some(TomlValue::String(ssid)) = network_table.get("ssid") {
                        wifi_network.ssid = ssid.clone();
                    }
                    if let Some(TomlValue::String(security)) = network_table.get("security") {
                        wifi_network.security = security.clone();
                    }
                    if let Some(TomlValue::Boolean(auto_connect)) = network_table.get("auto_connect") {
                        wifi_network.auto_connect = *auto_connect;
                    }
                    if let Some(TomlValue::Integer(priority)) = network_table.get("priority") {
                        wifi_network.priority = *priority as u32;
                    }
                    self.hidden_networks.push(wifi_network);
                }
            }
        }
        
        // Load MAC randomization
        if let Some(TomlValue::Boolean(mac_randomization)) = config.get("mac_randomization") {
            self.mac_randomization = *mac_randomization;
        }
        
        // Load WiFi calling enabled
        if let Some(TomlValue::Boolean(wifi_calling_enabled)) = config.get("wifi_calling_enabled") {
            self.wifi_calling_enabled = *wifi_calling_enabled;
        }
        
        // Load hotspot enabled
        if let Some(TomlValue::Boolean(hotspot_enabled)) = config.get("hotspot_enabled") {
            self.hotspot_enabled = *hotspot_enabled;
        }
        
        // Load hotspot settings
        if let Some(TomlValue::Table(hotspot_table)) = config.get("hotspot") {
            if let Some(TomlValue::String(ssid)) = hotspot_table.get("ssid") {
                self.hotspot.ssid = ssid.clone();
            }
            if let Some(TomlValue::String(security)) = hotspot_table.get("security") {
                self.hotspot.security = security.clone();
            }
            if let Some(TomlValue::Integer(max_clients)) = hotspot_table.get("max_clients") {
                self.hotspot.max_clients = *max_clients as u32;
            }
            if let Some(TomlValue::Boolean(auto_shutdown)) = hotspot_table.get("auto_shutdown") {
                self.hotspot.auto_shutdown = *auto_shutdown;
            }
            if let Some(TomlValue::Integer(shutdown_timeout_min)) = hotspot_table.get("shutdown_timeout_min") {
                self.hotspot.shutdown_timeout_min = *shutdown_timeout_min as u32;
            }
        }
        
        Ok(())
    }
    
    /// Save WiFi configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save auto connect
        config.insert("auto_connect".to_string(), TomlValue::Boolean(self.auto_connect));
        
        // Save prefer 5GHz
        config.insert("prefer_5ghz".to_string(), TomlValue::Boolean(self.prefer_5ghz));
        
        // Save power saving
        config.insert("power_saving".to_string(), TomlValue::Boolean(self.power_saving));
        
        // Save scanning enabled
        config.insert("scanning_enabled".to_string(), TomlValue::Boolean(self.scanning_enabled));
        
        // Save scan interval
        config.insert("scan_interval_sec".to_string(), TomlValue::Integer(self.scan_interval_sec as i64));
        
        // Save WiFi Direct enabled
        config.insert("wifi_direct_enabled".to_string(), TomlValue::Boolean(self.wifi_direct_enabled));
        
        // Save known networks
        let known_networks: Vec<TomlValue> = self.known_networks.iter()
            .map(|network| {
                let mut network_table = HashMap::new();
                network_table.insert("ssid".to_string(), TomlValue::String(network.ssid.clone()));
                network_table.insert("security".to_string(), TomlValue::String(network.security.clone()));
                network_table.insert("auto_connect".to_string(), TomlValue::Boolean(network.auto_connect));
                network_table.insert("priority".to_string(), TomlValue::Integer(network.priority as i64));
                TomlValue::Table(network_table)
            })
            .collect();
        config.insert("known_networks".to_string(), TomlValue::Array(known_networks));
        
        // Save hidden networks
        let hidden_networks: Vec<TomlValue> = self.hidden_networks.iter()
            .map(|network| {
                let mut network_table = HashMap::new();
                network_table.insert("ssid".to_string(), TomlValue::String(network.ssid.clone()));
                network_table.insert("security".to_string(), TomlValue::String(network.security.clone()));
                network_table.insert("auto_connect".to_string(), TomlValue::Boolean(network.auto_connect));
                network_table.insert("priority".to_string(), TomlValue::Integer(network.priority as i64));
                TomlValue::Table(network_table)
            })
            .collect();
        config.insert("hidden_networks".to_string(), TomlValue::Array(hidden_networks));
        
        // Save MAC randomization
        config.insert("mac_randomization".to_string(), TomlValue::Boolean(self.mac_randomization));
        
        // Save WiFi calling enabled
        config.insert("wifi_calling_enabled".to_string(), TomlValue::Boolean(self.wifi_calling_enabled));
        
        // Save hotspot enabled
        config.insert("hotspot_enabled".to_string(), TomlValue::Boolean(self.hotspot_enabled));
        
        // Save hotspot settings
        let mut hotspot_table = HashMap::new();
        hotspot_table.insert("ssid".to_string(), TomlValue::String(self.hotspot.ssid.clone()));
        hotspot_table.insert("security".to_string(), TomlValue::String(self.hotspot.security.clone()));
        hotspot_table.insert("max_clients".to_string(), TomlValue::Integer(self.hotspot.max_clients as i64));
        hotspot_table.insert("auto_shutdown".to_string(), TomlValue::Boolean(self.hotspot.auto_shutdown));
        hotspot_table.insert("shutdown_timeout_min".to_string(), TomlValue::Integer(self.hotspot.shutdown_timeout_min as i64));
        config.insert("hotspot".to_string(), TomlValue::Table(hotspot_table));
        
        Ok(config)
    }
    
    /// Apply WiFi configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply enabled state
        if let Err(e) = device.set_enabled(self.enabled) {
            return Err(ConfigError::DeviceError(format!("Failed to set enabled state: {}", e)));
        }
        
        // Apply power saving mode
        if let Err(e) = device.set_power_saving(self.power_saving) {
            return Err(ConfigError::DeviceError(format!("Failed to set power saving mode: {}", e)));
        }
        
        // Apply MAC randomization if supported
        if device.capabilities().contains(&DeviceCapability::MACRandomization) {
            if let Err(e) = device.set_mac_randomization(self.mac_randomization) {
                return Err(ConfigError::DeviceError(format!("Failed to set MAC randomization: {}", e)));
            }
        }
        
        // Apply WiFi Direct if supported
        if device.capabilities().contains(&DeviceCapability::WiFiDirect) {
            if let Err(e) = device.set_wifi_direct(self.wifi_direct_enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set WiFi Direct: {}", e)));
            }
        }
        
        // Apply hotspot settings if supported
        if device.capabilities().contains(&DeviceCapability::WiFiHotspot) {
            if let Err(e) = device.set_hotspot(self.hotspot_enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set hotspot: {}", e)));
            }
            
            if self.hotspot_enabled {
                if let Err(e) = device.configure_hotspot(&self.hotspot.ssid, &self.hotspot.security, self.hotspot.max_clients) {
                    return Err(ConfigError::DeviceError(format!("Failed to configure hotspot: {}", e)));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for WiFiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_connect: true,
            prefer_5ghz: true,
            power_saving: true,
            scanning_enabled: true,
            scan_interval_sec: 60,
            wifi_direct_enabled: false,
            known_networks: Vec::new(),
            hidden_networks: Vec::new(),
            mac_randomization: true,
            wifi_calling_enabled: false,
            hotspot_enabled: false,
            hotspot: HotspotSettings {
                ssid: "VR-Hotspot".to_string(),
                security: "WPA2".to_string(),
                max_clients: 5,
                auto_shutdown: true,
                shutdown_timeout_min: 30,
            },
        }
    }
}

/// WiFi network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    /// Network SSID
    pub ssid: String,
    
    /// Security type (None, WEP, WPA, WPA2, WPA3)
    pub security: String,
    
    /// Whether to automatically connect to this network
    pub auto_connect: bool,
    
    /// Connection priority (higher values have higher priority)
    pub priority: u32,
}

impl Default for WiFiNetwork {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            security: "WPA2".to_string(),
            auto_connect: true,
            priority: 0,
        }
    }
}

/// Hotspot settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotSettings {
    /// Hotspot SSID
    pub ssid: String,
    
    /// Security type (None, WPA2, WPA3)
    pub security: String,
    
    /// Maximum number of clients
    pub max_clients: u32,
    
    /// Whether to automatically shut down when no clients are connected
    pub auto_shutdown: bool,
    
    /// Auto-shutdown timeout in minutes
    pub shutdown_timeout_min: u32,
}

/// Bluetooth configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothConfig {
    /// Whether Bluetooth is enabled
    pub enabled: bool,
    
    /// Whether device is discoverable
    pub discoverable: bool,
    
    /// Discoverable timeout in seconds
    pub discoverable_timeout_sec: u32,
    
    /// Whether to automatically connect to paired devices
    pub auto_connect: bool,
    
    /// Whether to enable Bluetooth Low Energy
    pub ble_enabled: bool,
    
    /// Whether to enable Bluetooth audio
    pub audio_enabled: bool,
    
    /// Whether to enable Bluetooth HID
    pub hid_enabled: bool,
    
    /// Whether to enable Bluetooth file transfer
    pub file_transfer_enabled: bool,
    
    /// Whether to enable Bluetooth tethering
    pub tethering_enabled: bool,
    
    /// Paired devices
    pub paired_devices: Vec<BluetoothDevice>,
    
    /// Blocked devices
    pub blocked_devices: Vec<String>,
    
    /// Whether to enable MAC address randomization
    pub mac_randomization: bool,
}

impl BluetoothConfig {
    /// Load Bluetooth configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load discoverable
        if let Some(TomlValue::Boolean(discoverable)) = config.get("discoverable") {
            self.discoverable = *discoverable;
        }
        
        // Load discoverable timeout
        if let Some(TomlValue::Integer(discoverable_timeout_sec)) = config.get("discoverable_timeout_sec") {
            self.discoverable_timeout_sec = *discoverable_timeout_sec as u32;
        }
        
        // Load auto connect
        if let Some(TomlValue::Boolean(auto_connect)) = config.get("auto_connect") {
            self.auto_connect = *auto_connect;
        }
        
        // Load BLE enabled
        if let Some(TomlValue::Boolean(ble_enabled)) = config.get("ble_enabled") {
            self.ble_enabled = *ble_enabled;
        }
        
        // Load audio enabled
        if let Some(TomlValue::Boolean(audio_enabled)) = config.get("audio_enabled") {
            self.audio_enabled = *audio_enabled;
        }
        
        // Load HID enabled
        if let Some(TomlValue::Boolean(hid_enabled)) = config.get("hid_enabled") {
            self.hid_enabled = *hid_enabled;
        }
        
        // Load file transfer enabled
        if let Some(TomlValue::Boolean(file_transfer_enabled)) = config.get("file_transfer_enabled") {
            self.file_transfer_enabled = *file_transfer_enabled;
        }
        
        // Load tethering enabled
        if let Some(TomlValue::Boolean(tethering_enabled)) = config.get("tethering_enabled") {
            self.tethering_enabled = *tethering_enabled;
        }
        
        // Load paired devices
        if let Some(TomlValue::Array(devices)) = config.get("paired_devices") {
            self.paired_devices.clear();
            for device in devices {
                if let TomlValue::Table(device_table) = device {
                    let mut bt_device = BluetoothDevice::default();
                    if let Some(TomlValue::String(address)) = device_table.get("address") {
                        bt_device.address = address.clone();
                    }
                    if let Some(TomlValue::String(name)) = device_table.get("name") {
                        bt_device.name = name.clone();
                    }
                    if let Some(TomlValue::String(device_type)) = device_table.get("device_type") {
                        bt_device.device_type = device_type.clone();
                    }
                    if let Some(TomlValue::Boolean(trusted)) = device_table.get("trusted") {
                        bt_device.trusted = *trusted;
                    }
                    if let Some(TomlValue::Boolean(auto_connect)) = device_table.get("auto_connect") {
                        bt_device.auto_connect = *auto_connect;
                    }
                    self.paired_devices.push(bt_device);
                }
            }
        }
        
        // Load blocked devices
        if let Some(TomlValue::Array(devices)) = config.get("blocked_devices") {
            self.blocked_devices.clear();
            for device in devices {
                if let TomlValue::String(address) = device {
                    self.blocked_devices.push(address.clone());
                }
            }
        }
        
        // Load MAC randomization
        if let Some(TomlValue::Boolean(mac_randomization)) = config.get("mac_randomization") {
            self.mac_randomization = *mac_randomization;
        }
        
        Ok(())
    }
    
    /// Save Bluetooth configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save discoverable
        config.insert("discoverable".to_string(), TomlValue::Boolean(self.discoverable));
        
        // Save discoverable timeout
        config.insert("discoverable_timeout_sec".to_string(), TomlValue::Integer(self.discoverable_timeout_sec as i64));
        
        // Save auto connect
        config.insert("auto_connect".to_string(), TomlValue::Boolean(self.auto_connect));
        
        // Save BLE enabled
        config.insert("ble_enabled".to_string(), TomlValue::Boolean(self.ble_enabled));
        
        // Save audio enabled
        config.insert("audio_enabled".to_string(), TomlValue::Boolean(self.audio_enabled));
        
        // Save HID enabled
        config.insert("hid_enabled".to_string(), TomlValue::Boolean(self.hid_enabled));
        
        // Save file transfer enabled
        config.insert("file_transfer_enabled".to_string(), TomlValue::Boolean(self.file_transfer_enabled));
        
        // Save tethering enabled
        config.insert("tethering_enabled".to_string(), TomlValue::Boolean(self.tethering_enabled));
        
        // Save paired devices
        let paired_devices: Vec<TomlValue> = self.paired_devices.iter()
            .map(|device| {
                let mut device_table = HashMap::new();
                device_table.insert("address".to_string(), TomlValue::String(device.address.clone()));
                device_table.insert("name".to_string(), TomlValue::String(device.name.clone()));
                device_table.insert("device_type".to_string(), TomlValue::String(device.device_type.clone()));
                device_table.insert("trusted".to_string(), TomlValue::Boolean(device.trusted));
                device_table.insert("auto_connect".to_string(), TomlValue::Boolean(device.auto_connect));
                TomlValue::Table(device_table)
            })
            .collect();
        config.insert("paired_devices".to_string(), TomlValue::Array(paired_devices));
        
        // Save blocked devices
        let blocked_devices: Vec<TomlValue> = self.blocked_devices.iter()
            .map(|address| TomlValue::String(address.clone()))
            .collect();
        config.insert("blocked_devices".to_string(), TomlValue::Array(blocked_devices));
        
        // Save MAC randomization
        config.insert("mac_randomization".to_string(), TomlValue::Boolean(self.mac_randomization));
        
        Ok(config)
    }
    
    /// Apply Bluetooth configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply enabled state
        if let Err(e) = device.set_enabled(self.enabled) {
            return Err(ConfigError::DeviceError(format!("Failed to set enabled state: {}", e)));
        }
        
        // Apply discoverable state
        if let Err(e) = device.set_discoverable(self.discoverable, self.discoverable_timeout_sec) {
            return Err(ConfigError::DeviceError(format!("Failed to set discoverable state: {}", e)));
        }
        
        // Apply BLE if supported
        if device.capabilities().contains(&DeviceCapability::BluetoothLE) {
            if let Err(e) = device.set_ble_enabled(self.ble_enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set BLE state: {}", e)));
            }
        }
        
        // Apply MAC randomization if supported
        if device.capabilities().contains(&DeviceCapability::MACRandomization) {
            if let Err(e) = device.set_mac_randomization(self.mac_randomization) {
                return Err(ConfigError::DeviceError(format!("Failed to set MAC randomization: {}", e)));
            }
        }
        
        // Apply blocked devices
        for address in &self.blocked_devices {
            if let Err(e) = device.block_device(address) {
                warn!("Failed to block Bluetooth device {}: {}", address, e);
            }
        }
        
        Ok(())
    }
}

impl Default for BluetoothConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discoverable: false,
            discoverable_timeout_sec: 120,
            auto_connect: true,
            ble_enabled: true,
            audio_enabled: true,
            hid_enabled: true,
            file_transfer_enabled: false,
            tethering_enabled: false,
            paired_devices: Vec::new(),
            blocked_devices: Vec::new(),
            mac_randomization: true,
        }
    }
}

/// Bluetooth device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// Device MAC address
    pub address: String,
    
    /// Device name
    pub name: String,
    
    /// Device type (Audio, HID, Phone, Computer, etc.)
    pub device_type: String,
    
    /// Whether the device is trusted
    pub trusted: bool,
    
    /// Whether to automatically connect to this device
    pub auto_connect: bool,
}

impl Default for BluetoothDevice {
    fn default() -> Self {
        Self {
            address: String::new(),
            name: String::new(),
            device_type: "Unknown".to_string(),
            trusted: false,
            auto_connect: false,
        }
    }
}

/// Streaming configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Whether streaming is enabled
    pub enabled: bool,
    
    /// Streaming quality (low, medium, high, ultra)
    pub quality: String,
    
    /// Maximum bandwidth in Mbps
    pub max_bandwidth_mbps: u32,
    
    /// Target latency in milliseconds
    pub target_latency_ms: u32,
    
    /// Whether to use hardware acceleration
    pub hardware_acceleration: bool,
    
    /// Video settings
    pub video: VideoSettings,
    
    /// Audio settings
    pub audio: StreamingAudioSettings,
    
    /// Input settings
    pub input: InputSettings,
    
    /// Connection settings
    pub connection: ConnectionSettings,
}

impl StreamingConfig {
    /// Load streaming configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load quality
        if let Some(TomlValue::String(quality)) = config.get("quality") {
            self.quality = quality.clone();
            // Validate quality
            if self.quality != "low" && self.quality != "medium" && self.quality != "high" && self.quality != "ultra" {
                return Err(ConfigError::ValidationError(
                    "Streaming quality must be 'low', 'medium', 'high', or 'ultra'".to_string()
                ));
            }
        }
        
        // Load max bandwidth
        if let Some(TomlValue::Integer(max_bandwidth_mbps)) = config.get("max_bandwidth_mbps") {
            self.max_bandwidth_mbps = *max_bandwidth_mbps as u32;
        }
        
        // Load target latency
        if let Some(TomlValue::Integer(target_latency_ms)) = config.get("target_latency_ms") {
            self.target_latency_ms = *target_latency_ms as u32;
        }
        
        // Load hardware acceleration
        if let Some(TomlValue::Boolean(hardware_acceleration)) = config.get("hardware_acceleration") {
            self.hardware_acceleration = *hardware_acceleration;
        }
        
        // Load video settings
        if let Some(TomlValue::Table(video_table)) = config.get("video") {
            if let Some(TomlValue::String(codec)) = video_table.get("codec") {
                self.video.codec = codec.clone();
            }
            if let Some(TomlValue::Integer(bitrate_mbps)) = video_table.get("bitrate_mbps") {
                self.video.bitrate_mbps = *bitrate_mbps as u32;
            }
            if let Some(TomlValue::Integer(fps)) = video_table.get("fps") {
                self.video.fps = *fps as u32;
            }
            if let Some(TomlValue::Boolean(dynamic_bitrate)) = video_table.get("dynamic_bitrate") {
                self.video.dynamic_bitrate = *dynamic_bitrate;
            }
            if let Some(TomlValue::Boolean(keyframe_interval)) = video_table.get("keyframe_interval") {
                self.video.keyframe_interval = *keyframe_interval as u32;
            }
            if let Some(TomlValue::Boolean(foveated_encoding)) = video_table.get("foveated_encoding") {
                self.video.foveated_encoding = *foveated_encoding;
            }
        }
        
        // Load audio settings
        if let Some(TomlValue::Table(audio_table)) = config.get("audio") {
            if let Some(TomlValue::String(codec)) = audio_table.get("codec") {
                self.audio.codec = codec.clone();
            }
            if let Some(TomlValue::Integer(bitrate_kbps)) = audio_table.get("bitrate_kbps") {
                self.audio.bitrate_kbps = *bitrate_kbps as u32;
            }
            if let Some(TomlValue::Integer(sample_rate)) = audio_table.get("sample_rate") {
                self.audio.sample_rate = *sample_rate as u32;
            }
            if let Some(TomlValue::Boolean(stereo)) = audio_table.get("stereo") {
                self.audio.stereo = *stereo;
            }
            if let Some(TomlValue::Boolean(echo_cancellation)) = audio_table.get("echo_cancellation") {
                self.audio.echo_cancellation = *echo_cancellation;
            }
            if let Some(TomlValue::Boolean(noise_suppression)) = audio_table.get("noise_suppression") {
                self.audio.noise_suppression = *noise_suppression;
            }
        }
        
        // Load input settings
        if let Some(TomlValue::Table(input_table)) = config.get("input") {
            if let Some(TomlValue::Boolean(haptics)) = input_table.get("haptics") {
                self.input.haptics = *haptics;
            }
            if let Some(TomlValue::Integer(controller_prediction_ms)) = input_table.get("controller_prediction_ms") {
                self.input.controller_prediction_ms = *controller_prediction_ms as u32;
            }
            if let Some(TomlValue::Boolean(optimize_for_latency)) = input_table.get("optimize_for_latency") {
                self.input.optimize_for_latency = *optimize_for_latency;
            }
        }
        
        // Load connection settings
        if let Some(TomlValue::Table(connection_table)) = config.get("connection") {
            if let Some(TomlValue::String(protocol)) = connection_table.get("protocol") {
                self.connection.protocol = protocol.clone();
            }
            if let Some(TomlValue::Boolean(use_ipv6)) = connection_table.get("use_ipv6") {
                self.connection.use_ipv6 = *use_ipv6;
            }
            if let Some(TomlValue::Integer(port)) = connection_table.get("port") {
                self.connection.port = *port as u16;
            }
            if let Some(TomlValue::Boolean(upnp)) = connection_table.get("upnp") {
                self.connection.upnp = *upnp;
            }
            if let Some(TomlValue::Boolean(relay_server)) = connection_table.get("relay_server") {
                self.connection.relay_server = *relay_server;
            }
        }
        
        Ok(())
    }
    
    /// Save streaming configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save quality
        config.insert("quality".to_string(), TomlValue::String(self.quality.clone()));
        
        // Save max bandwidth
        config.insert("max_bandwidth_mbps".to_string(), TomlValue::Integer(self.max_bandwidth_mbps as i64));
        
        // Save target latency
        config.insert("target_latency_ms".to_string(), TomlValue::Integer(self.target_latency_ms as i64));
        
        // Save hardware acceleration
        config.insert("hardware_acceleration".to_string(), TomlValue::Boolean(self.hardware_acceleration));
        
        // Save video settings
        let mut video_table = HashMap::new();
        video_table.insert("codec".to_string(), TomlValue::String(self.video.codec.clone()));
        video_table.insert("bitrate_mbps".to_string(), TomlValue::Integer(self.video.bitrate_mbps as i64));
        video_table.insert("fps".to_string(), TomlValue::Integer(self.video.fps as i64));
        video_table.insert("dynamic_bitrate".to_string(), TomlValue::Boolean(self.video.dynamic_bitrate));
        video_table.insert("keyframe_interval".to_string(), TomlValue::Integer(self.video.keyframe_interval as i64));
        video_table.insert("foveated_encoding".to_string(), TomlValue::Boolean(self.video.foveated_encoding));
        config.insert("video".to_string(), TomlValue::Table(video_table));
        
        // Save audio settings
        let mut audio_table = HashMap::new();
        audio_table.insert("codec".to_string(), TomlValue::String(self.audio.codec.clone()));
        audio_table.insert("bitrate_kbps".to_string(), TomlValue::Integer(self.audio.bitrate_kbps as i64));
        audio_table.insert("sample_rate".to_string(), TomlValue::Integer(self.audio.sample_rate as i64));
        audio_table.insert("stereo".to_string(), TomlValue::Boolean(self.audio.stereo));
        audio_table.insert("echo_cancellation".to_string(), TomlValue::Boolean(self.audio.echo_cancellation));
        audio_table.insert("noise_suppression".to_string(), TomlValue::Boolean(self.audio.noise_suppression));
        config.insert("audio".to_string(), TomlValue::Table(audio_table));
        
        // Save input settings
        let mut input_table = HashMap::new();
        input_table.insert("haptics".to_string(), TomlValue::Boolean(self.input.haptics));
        input_table.insert("controller_prediction_ms".to_string(), TomlValue::Integer(self.input.controller_prediction_ms as i64));
        input_table.insert("optimize_for_latency".to_string(), TomlValue::Boolean(self.input.optimize_for_latency));
        config.insert("input".to_string(), TomlValue::Table(input_table));
        
        // Save connection settings
        let mut connection_table = HashMap::new();
        connection_table.insert("protocol".to_string(), TomlValue::String(self.connection.protocol.clone()));
        connection_table.insert("use_ipv6".to_string(), TomlValue::Boolean(self.connection.use_ipv6));
        connection_table.insert("port".to_string(), TomlValue::Integer(self.connection.port as i64));
        connection_table.insert("upnp".to_string(), TomlValue::Boolean(self.connection.upnp));
        connection_table.insert("relay_server".to_string(), TomlValue::Boolean(self.connection.relay_server));
        config.insert("connection".to_string(), TomlValue::Table(connection_table));
        
        Ok(config)
    }
    
    /// Apply streaming configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply streaming settings if supported
        if device.capabilities().contains(&DeviceCapability::Streaming) {
            // Apply enabled state
            if let Err(e) = device.set_streaming_enabled(self.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set streaming enabled: {}", e)));
            }
            
            // Apply quality
            if let Err(e) = device.set_streaming_quality(&self.quality) {
                return Err(ConfigError::DeviceError(format!("Failed to set streaming quality: {}", e)));
            }
            
            // Apply bandwidth limit
            if let Err(e) = device.set_streaming_bandwidth(self.max_bandwidth_mbps) {
                return Err(ConfigError::DeviceError(format!("Failed to set streaming bandwidth: {}", e)));
            }
            
            // Apply latency target
            if let Err(e) = device.set_streaming_latency(self.target_latency_ms) {
                return Err(ConfigError::DeviceError(format!("Failed to set streaming latency: {}", e)));
            }
            
            // Apply hardware acceleration
            if let Err(e) = device.set_streaming_hardware_acceleration(self.hardware_acceleration) {
                return Err(ConfigError::DeviceError(format!("Failed to set hardware acceleration: {}", e)));
            }
        }
        
        Ok(())
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            quality: "high".to_string(),
            max_bandwidth_mbps: 50,
            target_latency_ms: 20,
            hardware_acceleration: true,
            video: VideoSettings {
                codec: "h264".to_string(),
                bitrate_mbps: 30,
                fps: 90,
                dynamic_bitrate: true,
                keyframe_interval: 60,
                foveated_encoding: true,
            },
            audio: StreamingAudioSettings {
                codec: "opus".to_string(),
                bitrate_kbps: 128,
                sample_rate: 48000,
                stereo: true,
                echo_cancellation: true,
                noise_suppression: true,
            },
            input: InputSettings {
                haptics: true,
                controller_prediction_ms: 10,
                optimize_for_latency: true,
            },
            connection: ConnectionSettings {
                protocol: "udp".to_string(),
                use_ipv6: false,
                port: 9000,
                upnp: true,
                relay_server: false,
            },
        }
    }
}

/// Video settings for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    /// Video codec (h264, h265, av1)
    pub codec: String,
    
    /// Video bitrate in Mbps
    pub bitrate_mbps: u32,
    
    /// Frames per second
    pub fps: u32,
    
    /// Whether to use dynamic bitrate
    pub dynamic_bitrate: bool,
    
    /// Keyframe interval in frames
    pub keyframe_interval: u32,
    
    /// Whether to use foveated encoding
    pub foveated_encoding: bool,
}

/// Audio settings for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingAudioSettings {
    /// Audio codec (opus, aac)
    pub codec: String,
    
    /// Audio bitrate in kbps
    pub bitrate_kbps: u32,
    
    /// Sample rate in Hz
    pub sample_rate: u32,
    
    /// Whether to use stereo audio
    pub stereo: bool,
    
    /// Whether to use echo cancellation
    pub echo_cancellation: bool,
    
    /// Whether to use noise suppression
    pub noise_suppression: bool,
}

/// Input settings for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// Whether to enable haptic feedback
    pub haptics: bool,
    
    /// Controller prediction time in milliseconds
    pub controller_prediction_ms: u32,
    
    /// Whether to optimize for input latency
    pub optimize_for_latency: bool,
}

/// Connection settings for streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    /// Protocol (tcp, udp, quic)
    pub protocol: String,
    
    /// Whether to use IPv6
    pub use_ipv6: bool,
    
    /// Port number
    pub port: u16,
    
    /// Whether to use UPnP for port forwarding
    pub upnp: bool,
    
    /// Whether to use relay server
    pub relay_server: bool,
}

/// Firewall configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    /// Whether firewall is enabled
    pub enabled: bool,
    
    /// Default policy (allow, deny)
    pub default_policy: String,
    
    /// Whether to enable logging
    pub logging: bool,
    
    /// Whether to enable stealth mode
    pub stealth_mode: bool,
    
    /// Whether to allow local network access
    pub allow_local_network: bool,
    
    /// Whether to allow VPN traffic
    pub allow_vpn: bool,
    
    /// Whether to allow streaming traffic
    pub allow_streaming: bool,
    
    /// Whether to allow system services
    pub allow_system_services: bool,
    
    /// Allowed applications
    pub allowed_apps: Vec<String>,
    
    /// Blocked applications
    pub blocked_apps: Vec<String>,
    
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    
    /// Blocked IP addresses
    pub blocked_ips: Vec<String>,
    
    /// Allowed ports
    pub allowed_ports: Vec<u16>,
    
    /// Blocked ports
    pub blocked_ports: Vec<u16>,
}

impl FirewallConfig {
    /// Load firewall configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load default policy
        if let Some(TomlValue::String(default_policy)) = config.get("default_policy") {
            self.default_policy = default_policy.clone();
            // Validate default policy
            if self.default_policy != "allow" && self.default_policy != "deny" {
                return Err(ConfigError::ValidationError(
                    "Default policy must be 'allow' or 'deny'".to_string()
                ));
            }
        }
        
        // Load logging
        if let Some(TomlValue::Boolean(logging)) = config.get("logging") {
            self.logging = *logging;
        }
        
        // Load stealth mode
        if let Some(TomlValue::Boolean(stealth_mode)) = config.get("stealth_mode") {
            self.stealth_mode = *stealth_mode;
        }
        
        // Load allow local network
        if let Some(TomlValue::Boolean(allow_local_network)) = config.get("allow_local_network") {
            self.allow_local_network = *allow_local_network;
        }
        
        // Load allow VPN
        if let Some(TomlValue::Boolean(allow_vpn)) = config.get("allow_vpn") {
            self.allow_vpn = *allow_vpn;
        }
        
        // Load allow streaming
        if let Some(TomlValue::Boolean(allow_streaming)) = config.get("allow_streaming") {
            self.allow_streaming = *allow_streaming;
        }
        
        // Load allow system services
        if let Some(TomlValue::Boolean(allow_system_services)) = config.get("allow_system_services") {
            self.allow_system_services = *allow_system_services;
        }
        
        // Load allowed applications
        if let Some(TomlValue::Array(apps)) = config.get("allowed_apps") {
            self.allowed_apps.clear();
            for app in apps {
                if let TomlValue::String(app_str) = app {
                    self.allowed_apps.push(app_str.clone());
                }
            }
        }
        
        // Load blocked applications
        if let Some(TomlValue::Array(apps)) = config.get("blocked_apps") {
            self.blocked_apps.clear();
            for app in apps {
                if let TomlValue::String(app_str) = app {
                    self.blocked_apps.push(app_str.clone());
                }
            }
        }
        
        // Load allowed IP addresses
        if let Some(TomlValue::Array(ips)) = config.get("allowed_ips") {
            self.allowed_ips.clear();
            for ip in ips {
                if let TomlValue::String(ip_str) = ip {
                    self.allowed_ips.push(ip_str.clone());
                }
            }
        }
        
        // Load blocked IP addresses
        if let Some(TomlValue::Array(ips)) = config.get("blocked_ips") {
            self.blocked_ips.clear();
            for ip in ips {
                if let TomlValue::String(ip_str) = ip {
                    self.blocked_ips.push(ip_str.clone());
                }
            }
        }
        
        // Load allowed ports
        if let Some(TomlValue::Array(ports)) = config.get("allowed_ports") {
            self.allowed_ports.clear();
            for port in ports {
                if let TomlValue::Integer(port_int) = port {
                    if *port_int >= 0 && *port_int <= 65535 {
                        self.allowed_ports.push(*port_int as u16);
                    }
                }
            }
        }
        
        // Load blocked ports
        if let Some(TomlValue::Array(ports)) = config.get("blocked_ports") {
            self.blocked_ports.clear();
            for port in ports {
                if let TomlValue::Integer(port_int) = port {
                    if *port_int >= 0 && *port_int <= 65535 {
                        self.blocked_ports.push(*port_int as u16);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Save firewall configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save default policy
        config.insert("default_policy".to_string(), TomlValue::String(self.default_policy.clone()));
        
        // Save logging
        config.insert("logging".to_string(), TomlValue::Boolean(self.logging));
        
        // Save stealth mode
        config.insert("stealth_mode".to_string(), TomlValue::Boolean(self.stealth_mode));
        
        // Save allow local network
        config.insert("allow_local_network".to_string(), TomlValue::Boolean(self.allow_local_network));
        
        // Save allow VPN
        config.insert("allow_vpn".to_string(), TomlValue::Boolean(self.allow_vpn));
        
        // Save allow streaming
        config.insert("allow_streaming".to_string(), TomlValue::Boolean(self.allow_streaming));
        
        // Save allow system services
        config.insert("allow_system_services".to_string(), TomlValue::Boolean(self.allow_system_services));
        
        // Save allowed applications
        let allowed_apps: Vec<TomlValue> = self.allowed_apps.iter()
            .map(|app| TomlValue::String(app.clone()))
            .collect();
        config.insert("allowed_apps".to_string(), TomlValue::Array(allowed_apps));
        
        // Save blocked applications
        let blocked_apps: Vec<TomlValue> = self.blocked_apps.iter()
            .map(|app| TomlValue::String(app.clone()))
            .collect();
        config.insert("blocked_apps".to_string(), TomlValue::Array(blocked_apps));
        
        // Save allowed IP addresses
        let allowed_ips: Vec<TomlValue> = self.allowed_ips.iter()
            .map(|ip| TomlValue::String(ip.clone()))
            .collect();
        config.insert("allowed_ips".to_string(), TomlValue::Array(allowed_ips));
        
        // Save blocked IP addresses
        let blocked_ips: Vec<TomlValue> = self.blocked_ips.iter()
            .map(|ip| TomlValue::String(ip.clone()))
            .collect();
        config.insert("blocked_ips".to_string(), TomlValue::Array(blocked_ips));
        
        // Save allowed ports
        let allowed_ports: Vec<TomlValue> = self.allowed_ports.iter()
            .map(|port| TomlValue::Integer(*port as i64))
            .collect();
        config.insert("allowed_ports".to_string(), TomlValue::Array(allowed_ports));
        
        // Save blocked ports
        let blocked_ports: Vec<TomlValue> = self.blocked_ports.iter()
            .map(|port| TomlValue::Integer(*port as i64))
            .collect();
        config.insert("blocked_ports".to_string(), TomlValue::Array(blocked_ports));
        
        Ok(config)
    }
    
    /// Apply firewall configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply firewall settings if supported
        if device.capabilities().contains(&DeviceCapability::Firewall) {
            // Apply enabled state
            if let Err(e) = device.set_firewall_enabled(self.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set firewall enabled: {}", e)));
            }
            
            // Apply default policy
            if let Err(e) = device.set_firewall_default_policy(&self.default_policy) {
                return Err(ConfigError::DeviceError(format!("Failed to set firewall default policy: {}", e)));
            }
            
            // Apply logging
            if let Err(e) = device.set_firewall_logging(self.logging) {
                return Err(ConfigError::DeviceError(format!("Failed to set firewall logging: {}", e)));
            }
            
            // Apply stealth mode
            if let Err(e) = device.set_firewall_stealth_mode(self.stealth_mode) {
                return Err(ConfigError::DeviceError(format!("Failed to set firewall stealth mode: {}", e)));
            }
            
            // Apply allowed applications
            for app in &self.allowed_apps {
                if let Err(e) = device.add_firewall_app_rule(app, true) {
                    warn!("Failed to add firewall allow rule for app {}: {}", app, e);
                }
            }
            
            // Apply blocked applications
            for app in &self.blocked_apps {
                if let Err(e) = device.add_firewall_app_rule(app, false) {
                    warn!("Failed to add firewall block rule for app {}: {}", app, e);
                }
            }
            
            // Apply allowed IP addresses
            for ip in &self.allowed_ips {
                if let Err(e) = device.add_firewall_ip_rule(ip, true) {
                    warn!("Failed to add firewall allow rule for IP {}: {}", ip, e);
                }
            }
            
            // Apply blocked IP addresses
            for ip in &self.blocked_ips {
                if let Err(e) = device.add_firewall_ip_rule(ip, false) {
                    warn!("Failed to add firewall block rule for IP {}: {}", ip, e);
                }
            }
            
            // Apply allowed ports
            for port in &self.allowed_ports {
                if let Err(e) = device.add_firewall_port_rule(*port, true) {
                    warn!("Failed to add firewall allow rule for port {}: {}", port, e);
                }
            }
            
            // Apply blocked ports
            for port in &self.blocked_ports {
                if let Err(e) = device.add_firewall_port_rule(*port, false) {
                    warn!("Failed to add firewall block rule for port {}: {}", port, e);
                }
            }
        }
        
        Ok(())
    }
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_policy: "deny".to_string(),
            logging: true,
            stealth_mode: false,
            allow_local_network: true,
            allow_vpn: true,
            allow_streaming: true,
            allow_system_services: true,
            allowed_apps: Vec::new(),
            blocked_apps: Vec::new(),
            allowed_ips: Vec::new(),
            blocked_ips: Vec::new(),
            allowed_ports: vec![80, 443, 53],
            blocked_ports: Vec::new(),
        }
    }
}

/// VPN configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPNConfig {
    /// Whether VPN is enabled
    pub enabled: bool,
    
    /// VPN protocol (openvpn, wireguard, ipsec)
    pub protocol: String,
    
    /// Whether to auto-connect on startup
    pub auto_connect: bool,
    
    /// Whether to block non-VPN traffic
    pub kill_switch: bool,
    
    /// Whether to use split tunneling
    pub split_tunneling: bool,
    
    /// Applications to exclude from VPN
    pub excluded_apps: Vec<String>,
    
    /// Whether to use DNS over VPN
    pub use_vpn_dns: bool,
    
    /// Custom DNS servers
    pub custom_dns: Vec<String>,
    
    /// VPN server configurations
    pub servers: Vec<VPNServer>,
}

impl VPNConfig {
    /// Load VPN configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load protocol
        if let Some(TomlValue::String(protocol)) = config.get("protocol") {
            self.protocol = protocol.clone();
            // Validate protocol
            if self.protocol != "openvpn" && self.protocol != "wireguard" && self.protocol != "ipsec" {
                return Err(ConfigError::ValidationError(
                    "VPN protocol must be 'openvpn', 'wireguard', or 'ipsec'".to_string()
                ));
            }
        }
        
        // Load auto connect
        if let Some(TomlValue::Boolean(auto_connect)) = config.get("auto_connect") {
            self.auto_connect = *auto_connect;
        }
        
        // Load kill switch
        if let Some(TomlValue::Boolean(kill_switch)) = config.get("kill_switch") {
            self.kill_switch = *kill_switch;
        }
        
        // Load split tunneling
        if let Some(TomlValue::Boolean(split_tunneling)) = config.get("split_tunneling") {
            self.split_tunneling = *split_tunneling;
        }
        
        // Load excluded applications
        if let Some(TomlValue::Array(apps)) = config.get("excluded_apps") {
            self.excluded_apps.clear();
            for app in apps {
                if let TomlValue::String(app_str) = app {
                    self.excluded_apps.push(app_str.clone());
                }
            }
        }
        
        // Load use VPN DNS
        if let Some(TomlValue::Boolean(use_vpn_dns)) = config.get("use_vpn_dns") {
            self.use_vpn_dns = *use_vpn_dns;
        }
        
        // Load custom DNS servers
        if let Some(TomlValue::Array(dns_servers)) = config.get("custom_dns") {
            self.custom_dns.clear();
            for dns in dns_servers {
                if let TomlValue::String(dns_str) = dns {
                    self.custom_dns.push(dns_str.clone());
                }
            }
        }
        
        // Load VPN servers
        if let Some(TomlValue::Array(servers)) = config.get("servers") {
            self.servers.clear();
            for server in servers {
                if let TomlValue::Table(server_table) = server {
                    let mut vpn_server = VPNServer::default();
                    if let Some(TomlValue::String(name)) = server_table.get("name") {
                        vpn_server.name = name.clone();
                    }
                    if let Some(TomlValue::String(address)) = server_table.get("address") {
                        vpn_server.address = address.clone();
                    }
                    if let Some(TomlValue::Integer(port)) = server_table.get("port") {
                        vpn_server.port = *port as u16;
                    }
                    if let Some(TomlValue::String(country)) = server_table.get("country") {
                        vpn_server.country = country.clone();
                    }
                    if let Some(TomlValue::Boolean(favorite)) = server_table.get("favorite") {
                        vpn_server.favorite = *favorite;
                    }
                    self.servers.push(vpn_server);
                }
            }
        }
        
        Ok(())
    }
    
    /// Save VPN configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save protocol
        config.insert("protocol".to_string(), TomlValue::String(self.protocol.clone()));
        
        // Save auto connect
        config.insert("auto_connect".to_string(), TomlValue::Boolean(self.auto_connect));
        
        // Save kill switch
        config.insert("kill_switch".to_string(), TomlValue::Boolean(self.kill_switch));
        
        // Save split tunneling
        config.insert("split_tunneling".to_string(), TomlValue::Boolean(self.split_tunneling));
        
        // Save excluded applications
        let excluded_apps: Vec<TomlValue> = self.excluded_apps.iter()
            .map(|app| TomlValue::String(app.clone()))
            .collect();
        config.insert("excluded_apps".to_string(), TomlValue::Array(excluded_apps));
        
        // Save use VPN DNS
        config.insert("use_vpn_dns".to_string(), TomlValue::Boolean(self.use_vpn_dns));
        
        // Save custom DNS servers
        let custom_dns: Vec<TomlValue> = self.custom_dns.iter()
            .map(|dns| TomlValue::String(dns.clone()))
            .collect();
        config.insert("custom_dns".to_string(), TomlValue::Array(custom_dns));
        
        // Save VPN servers
        let servers: Vec<TomlValue> = self.servers.iter()
            .map(|server| {
                let mut server_table = HashMap::new();
                server_table.insert("name".to_string(), TomlValue::String(server.name.clone()));
                server_table.insert("address".to_string(), TomlValue::String(server.address.clone()));
                server_table.insert("port".to_string(), TomlValue::Integer(server.port as i64));
                server_table.insert("country".to_string(), TomlValue::String(server.country.clone()));
                server_table.insert("favorite".to_string(), TomlValue::Boolean(server.favorite));
                TomlValue::Table(server_table)
            })
            .collect();
        config.insert("servers".to_string(), TomlValue::Array(servers));
        
        Ok(config)
    }
    
    /// Apply VPN configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply VPN settings if supported
        if device.capabilities().contains(&DeviceCapability::VPN) {
            // Apply enabled state
            if let Err(e) = device.set_vpn_enabled(self.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set VPN enabled: {}", e)));
            }
            
            // Apply protocol
            if let Err(e) = device.set_vpn_protocol(&self.protocol) {
                return Err(ConfigError::DeviceError(format!("Failed to set VPN protocol: {}", e)));
            }
            
            // Apply kill switch
            if let Err(e) = device.set_vpn_kill_switch(self.kill_switch) {
                return Err(ConfigError::DeviceError(format!("Failed to set VPN kill switch: {}", e)));
            }
            
            // Apply split tunneling
            if let Err(e) = device.set_vpn_split_tunneling(self.split_tunneling) {
                return Err(ConfigError::DeviceError(format!("Failed to set VPN split tunneling: {}", e)));
            }
            
            // Apply DNS settings
            if let Err(e) = device.set_vpn_dns(self.use_vpn_dns, &self.custom_dns) {
                return Err(ConfigError::DeviceError(format!("Failed to set VPN DNS: {}", e)));
            }
            
            // Apply server configuration if there's a favorite server
            if let Some(server) = self.servers.iter().find(|s| s.favorite) {
                if let Err(e) = device.set_vpn_server(&server.address, server.port) {
                    return Err(ConfigError::DeviceError(format!("Failed to set VPN server: {}", e)));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for VPNConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: "wireguard".to_string(),
            auto_connect: false,
            kill_switch: false,
            split_tunneling: false,
            excluded_apps: Vec::new(),
            use_vpn_dns: true,
            custom_dns: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            servers: Vec::new(),
        }
    }
}

/// VPN server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPNServer {
    /// Server name
    pub name: String,
    
    /// Server address
    pub address: String,
    
    /// Server port
    pub port: u16,
    
    /// Server country
    pub country: String,
    
    /// Whether this is a favorite server
    pub favorite: bool,
}

impl Default for VPNServer {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: String::new(),
            port: 1194,
            country: String::new(),
            favorite: false,
        }
    }
}

/// QoS (Quality of Service) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSConfig {
    /// Whether QoS is enabled
    pub enabled: bool,
    
    /// Whether to prioritize VR applications
    pub prioritize_vr: bool,
    
    /// Whether to prioritize streaming
    pub prioritize_streaming: bool,
    
    /// Whether to prioritize voice communication
    pub prioritize_voice: bool,
    
    /// Whether to prioritize downloads
    pub prioritize_downloads: bool,
    
    /// Whether to limit background traffic
    pub limit_background: bool,
    
    /// Background traffic limit in Mbps
    pub background_limit_mbps: u32,
    
    /// Traffic classes
    pub traffic_classes: Vec<TrafficClass>,
}

impl QoSConfig {
    /// Load QoS configuration from TOML values.
    pub fn load_from_toml(&mut self, config: &HashMap<String, TomlValue>) -> ConfigResult<()> {
        // Load enabled
        if let Some(TomlValue::Boolean(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        // Load prioritize VR
        if let Some(TomlValue::Boolean(prioritize_vr)) = config.get("prioritize_vr") {
            self.prioritize_vr = *prioritize_vr;
        }
        
        // Load prioritize streaming
        if let Some(TomlValue::Boolean(prioritize_streaming)) = config.get("prioritize_streaming") {
            self.prioritize_streaming = *prioritize_streaming;
        }
        
        // Load prioritize voice
        if let Some(TomlValue::Boolean(prioritize_voice)) = config.get("prioritize_voice") {
            self.prioritize_voice = *prioritize_voice;
        }
        
        // Load prioritize downloads
        if let Some(TomlValue::Boolean(prioritize_downloads)) = config.get("prioritize_downloads") {
            self.prioritize_downloads = *prioritize_downloads;
        }
        
        // Load limit background
        if let Some(TomlValue::Boolean(limit_background)) = config.get("limit_background") {
            self.limit_background = *limit_background;
        }
        
        // Load background limit
        if let Some(TomlValue::Integer(background_limit_mbps)) = config.get("background_limit_mbps") {
            self.background_limit_mbps = *background_limit_mbps as u32;
        }
        
        // Load traffic classes
        if let Some(TomlValue::Array(classes)) = config.get("traffic_classes") {
            self.traffic_classes.clear();
            for class in classes {
                if let TomlValue::Table(class_table) = class {
                    let mut traffic_class = TrafficClass::default();
                    if let Some(TomlValue::String(name)) = class_table.get("name") {
                        traffic_class.name = name.clone();
                    }
                    if let Some(TomlValue::Integer(priority)) = class_table.get("priority") {
                        traffic_class.priority = *priority as u32;
                    }
                    if let Some(TomlValue::Integer(min_bandwidth_mbps)) = class_table.get("min_bandwidth_mbps") {
                        traffic_class.min_bandwidth_mbps = *min_bandwidth_mbps as u32;
                    }
                    if let Some(TomlValue::Integer(max_bandwidth_mbps)) = class_table.get("max_bandwidth_mbps") {
                        traffic_class.max_bandwidth_mbps = *max_bandwidth_mbps as u32;
                    }
                    if let Some(TomlValue::Array(ports)) = class_table.get("ports") {
                        traffic_class.ports.clear();
                        for port in ports {
                            if let TomlValue::Integer(port_int) = port {
                                if *port_int >= 0 && *port_int <= 65535 {
                                    traffic_class.ports.push(*port_int as u16);
                                }
                            }
                        }
                    }
                    if let Some(TomlValue::Array(apps)) = class_table.get("applications") {
                        traffic_class.applications.clear();
                        for app in apps {
                            if let TomlValue::String(app_str) = app {
                                traffic_class.applications.push(app_str.clone());
                            }
                        }
                    }
                    self.traffic_classes.push(traffic_class);
                }
            }
        }
        
        Ok(())
    }
    
    /// Save QoS configuration to TOML values.
    pub fn save_to_toml(&self) -> ConfigResult<HashMap<String, TomlValue>> {
        let mut config = HashMap::new();
        
        // Save enabled
        config.insert("enabled".to_string(), TomlValue::Boolean(self.enabled));
        
        // Save prioritize VR
        config.insert("prioritize_vr".to_string(), TomlValue::Boolean(self.prioritize_vr));
        
        // Save prioritize streaming
        config.insert("prioritize_streaming".to_string(), TomlValue::Boolean(self.prioritize_streaming));
        
        // Save prioritize voice
        config.insert("prioritize_voice".to_string(), TomlValue::Boolean(self.prioritize_voice));
        
        // Save prioritize downloads
        config.insert("prioritize_downloads".to_string(), TomlValue::Boolean(self.prioritize_downloads));
        
        // Save limit background
        config.insert("limit_background".to_string(), TomlValue::Boolean(self.limit_background));
        
        // Save background limit
        config.insert("background_limit_mbps".to_string(), TomlValue::Integer(self.background_limit_mbps as i64));
        
        // Save traffic classes
        let classes: Vec<TomlValue> = self.traffic_classes.iter()
            .map(|class| {
                let mut class_table = HashMap::new();
                class_table.insert("name".to_string(), TomlValue::String(class.name.clone()));
                class_table.insert("priority".to_string(), TomlValue::Integer(class.priority as i64));
                class_table.insert("min_bandwidth_mbps".to_string(), TomlValue::Integer(class.min_bandwidth_mbps as i64));
                class_table.insert("max_bandwidth_mbps".to_string(), TomlValue::Integer(class.max_bandwidth_mbps as i64));
                
                let ports: Vec<TomlValue> = class.ports.iter()
                    .map(|port| TomlValue::Integer(*port as i64))
                    .collect();
                class_table.insert("ports".to_string(), TomlValue::Array(ports));
                
                let applications: Vec<TomlValue> = class.applications.iter()
                    .map(|app| TomlValue::String(app.clone()))
                    .collect();
                class_table.insert("applications".to_string(), TomlValue::Array(applications));
                
                TomlValue::Table(class_table)
            })
            .collect();
        config.insert("traffic_classes".to_string(), TomlValue::Array(classes));
        
        Ok(config)
    }
    
    /// Apply QoS configuration to a network device.
    pub fn apply_to_device(&self, device: &dyn NetworkDevice) -> ConfigResult<()> {
        // Apply QoS settings if supported
        if device.capabilities().contains(&DeviceCapability::QoS) {
            // Apply enabled state
            if let Err(e) = device.set_qos_enabled(self.enabled) {
                return Err(ConfigError::DeviceError(format!("Failed to set QoS enabled: {}", e)));
            }
            
            // Apply VR prioritization
            if let Err(e) = device.set_qos_prioritize_vr(self.prioritize_vr) {
                return Err(ConfigError::DeviceError(format!("Failed to set QoS VR prioritization: {}", e)));
            }
            
            // Apply streaming prioritization
            if let Err(e) = device.set_qos_prioritize_streaming(self.prioritize_streaming) {
                return Err(ConfigError::DeviceError(format!("Failed to set QoS streaming prioritization: {}", e)));
            }
            
            // Apply voice prioritization
            if let Err(e) = device.set_qos_prioritize_voice(self.prioritize_voice) {
                return Err(ConfigError::DeviceError(format!("Failed to set QoS voice prioritization: {}", e)));
            }
            
            // Apply background traffic limit
            if let Err(e) = device.set_qos_background_limit(self.limit_background, self.background_limit_mbps) {
                return Err(ConfigError::DeviceError(format!("Failed to set QoS background limit: {}", e)));
            }
            
            // Apply traffic classes
            for class in &self.traffic_classes {
                if let Err(e) = device.add_qos_traffic_class(&class.name, class.priority, class.min_bandwidth_mbps, class.max_bandwidth_mbps) {
                    warn!("Failed to add QoS traffic class {}: {}", class.name, e);
                    continue;
                }
                
                // Add ports to traffic class
                for port in &class.ports {
                    if let Err(e) = device.add_qos_port_to_class(&class.name, *port) {
                        warn!("Failed to add port {} to QoS traffic class {}: {}", port, class.name, e);
                    }
                }
                
                // Add applications to traffic class
                for app in &class.applications {
                    if let Err(e) = device.add_qos_app_to_class(&class.name, app) {
                        warn!("Failed to add application {} to QoS traffic class {}: {}", app, class.name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for QoSConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            prioritize_vr: true,
            prioritize_streaming: true,
            prioritize_voice: true,
            prioritize_downloads: false,
            limit_background: true,
            background_limit_mbps: 5,
            traffic_classes: vec![
                TrafficClass {
                    name: "VR".to_string(),
                    priority: 1,
                    min_bandwidth_mbps: 50,
                    max_bandwidth_mbps: 0,
                    ports: vec![9000, 9001, 9002],
                    applications: vec!["vr_app".to_string()],
                },
                TrafficClass {
                    name: "Voice".to_string(),
                    priority: 2,
                    min_bandwidth_mbps: 1,
                    max_bandwidth_mbps: 5,
                    ports: vec![5060, 5061],
                    applications: vec!["voice_app".to_string()],
                },
                TrafficClass {
                    name: "Background".to_string(),
                    priority: 5,
                    min_bandwidth_mbps: 0,
                    max_bandwidth_mbps: 5,
                    ports: Vec::new(),
                    applications: vec!["update_service".to_string()],
                },
            ],
        }
    }
}

/// Traffic class for QoS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficClass {
    /// Class name
    pub name: String,
    
    /// Priority (lower values have higher priority)
    pub priority: u32,
    
    /// Minimum guaranteed bandwidth in Mbps
    pub min_bandwidth_mbps: u32,
    
    /// Maximum bandwidth in Mbps (0 for unlimited)
    pub max_bandwidth_mbps: u32,
    
    /// Ports to include in this class
    pub ports: Vec<u16>,
    
    /// Applications to include in this class
    pub applications: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wifi_config_load_save() {
        let mut config = WiFiConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("auto_connect".to_string(), TomlValue::Boolean(false));
        toml_values.insert("prefer_5ghz".to_string(), TomlValue::Boolean(false));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, false);
        assert_eq!(config.auto_connect, false);
        assert_eq!(config.prefer_5ghz, false);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("auto_connect"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("prefer_5ghz"), Some(&TomlValue::Boolean(false)));
    }
    
    #[test]
    fn test_bluetooth_config_load_save() {
        let mut config = BluetoothConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("discoverable".to_string(), TomlValue::Boolean(true));
        toml_values.insert("discoverable_timeout_sec".to_string(), TomlValue::Integer(60));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, false);
        assert_eq!(config.discoverable, true);
        assert_eq!(config.discoverable_timeout_sec, 60);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("discoverable"), Some(&TomlValue::Boolean(true)));
        assert_eq!(saved.get("discoverable_timeout_sec"), Some(&TomlValue::Integer(60)));
    }
    
    #[test]
    fn test_streaming_config_load_save() {
        let mut config = StreamingConfig::default();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        toml_values.insert("enabled".to_string(), TomlValue::Boolean(false));
        toml_values.insert("quality".to_string(), TomlValue::String("medium".to_string()));
        toml_values.insert("max_bandwidth_mbps".to_string(), TomlValue::Integer(30));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Verify values were loaded correctly
        assert_eq!(config.enabled, false);
        assert_eq!(config.quality, "medium");
        assert_eq!(config.max_bandwidth_mbps, 30);
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify values were saved correctly
        assert_eq!(saved.get("enabled"), Some(&TomlValue::Boolean(false)));
        assert_eq!(saved.get("quality"), Some(&TomlValue::String("medium".to_string())));
        assert_eq!(saved.get("max_bandwidth_mbps"), Some(&TomlValue::Integer(30)));
    }
    
    #[test]
    fn test_network_config_load_save() {
        let config = NetworkConfig::new();
        
        // Create test TOML values
        let mut toml_values = HashMap::new();
        
        let mut wifi = HashMap::new();
        wifi.insert("enabled".to_string(), TomlValue::Boolean(false));
        wifi.insert("auto_connect".to_string(), TomlValue::Boolean(false));
        toml_values.insert("wifi".to_string(), TomlValue::Table(wifi));
        
        let mut bluetooth = HashMap::new();
        bluetooth.insert("enabled".to_string(), TomlValue::Boolean(false));
        bluetooth.insert("discoverable".to_string(), TomlValue::Boolean(true));
        toml_values.insert("bluetooth".to_string(), TomlValue::Table(bluetooth));
        
        // Load config from TOML
        config.load_from_toml(&toml_values).unwrap();
        
        // Save config to TOML
        let saved = config.save_to_toml().unwrap();
        
        // Verify WiFi values were saved correctly
        if let Some(TomlValue::Table(wifi)) = saved.get("wifi") {
            assert_eq!(wifi.get("enabled"), Some(&TomlValue::Boolean(false)));
            assert_eq!(wifi.get("auto_connect"), Some(&TomlValue::Boolean(false)));
        } else {
            panic!("Expected WiFi table");
        }
        
        // Verify Bluetooth values were saved correctly
        if let Some(TomlValue::Table(bluetooth)) = saved.get("bluetooth") {
            assert_eq!(bluetooth.get("enabled"), Some(&TomlValue::Boolean(false)));
            assert_eq!(bluetooth.get("discoverable"), Some(&TomlValue::Boolean(true)));
        } else {
            panic!("Expected Bluetooth table");
        }
    }
}
