//! Network device interface for the VR headset.
//!
//! This module provides the implementation of network devices for the VR headset,
//! including WiFi, Bluetooth, and other network interfaces.

use std::collections::HashMap;
use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};

/// Network interface type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkInterfaceType {
    /// WiFi interface
    WiFi,
    
    /// Bluetooth interface
    Bluetooth,
    
    /// Ethernet interface
    Ethernet,
    
    /// USB network interface
    USB,
    
    /// Virtual interface
    Virtual,
    
    /// Loopback interface
    Loopback,
    
    /// Other interface type
    Other,
}

/// Network interface state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkInterfaceState {
    /// Interface is up
    Up,
    
    /// Interface is down
    Down,
    
    /// Interface is dormant
    Dormant,
    
    /// Interface is testing
    Testing,
    
    /// Interface state is unknown
    Unknown,
}

/// WiFi security type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WiFiSecurityType {
    /// No security (open network)
    None,
    
    /// WEP security
    WEP,
    
    /// WPA security
    WPA,
    
    /// WPA2 security
    WPA2,
    
    /// WPA3 security
    WPA3,
    
    /// Enterprise security
    Enterprise,
    
    /// Unknown security type
    Unknown,
}

/// WiFi band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WiFiBand {
    /// 2.4 GHz band
    Band2_4GHz,
    
    /// 5 GHz band
    Band5GHz,
    
    /// 6 GHz band
    Band6GHz,
    
    /// 60 GHz band
    Band60GHz,
    
    /// Unknown band
    Unknown,
}

/// WiFi standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WiFiStandard {
    /// 802.11a
    IEEE802_11a,
    
    /// 802.11b
    IEEE802_11b,
    
    /// 802.11g
    IEEE802_11g,
    
    /// 802.11n
    IEEE802_11n,
    
    /// 802.11ac
    IEEE802_11ac,
    
    /// 802.11ax (WiFi 6)
    IEEE802_11ax,
    
    /// 802.11be (WiFi 7)
    IEEE802_11be,
    
    /// Unknown standard
    Unknown,
}

/// WiFi network information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiFiNetworkInfo {
    /// Network SSID
    pub ssid: String,
    
    /// Network BSSID
    pub bssid: Option<String>,
    
    /// Signal strength in dBm
    pub signal_strength: i32,
    
    /// Security type
    pub security_type: WiFiSecurityType,
    
    /// WiFi band
    pub band: WiFiBand,
    
    /// WiFi standard
    pub standard: WiFiStandard,
    
    /// Channel number
    pub channel: u32,
    
    /// Frequency in MHz
    pub frequency: u32,
    
    /// Whether this network is currently connected
    pub connected: bool,
    
    /// Whether this network is saved
    pub saved: bool,
    
    /// Whether this network is hidden
    pub hidden: bool,
}

/// WiFi connection status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WiFiConnectionStatus {
    /// Connected network SSID
    pub ssid: Option<String>,
    
    /// Connected network BSSID
    pub bssid: Option<String>,
    
    /// Signal strength in dBm
    pub signal_strength: Option<i32>,
    
    /// Link speed in Mbps
    pub link_speed: Option<u32>,
    
    /// Frequency in MHz
    pub frequency: Option<u32>,
    
    /// IP address
    pub ip_address: Option<IpAddr>,
    
    /// Gateway address
    pub gateway: Option<IpAddr>,
    
    /// DNS servers
    pub dns_servers: Vec<IpAddr>,
    
    /// Connection state
    pub state: NetworkInterfaceState,
    
    /// Connection timestamp
    pub connected_since: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Connection details
    pub details: HashMap<String, String>,
}

/// WiFi device configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WiFiConfig {
    /// Whether WiFi is enabled
    pub enabled: bool,
    
    /// Whether to automatically connect to known networks
    pub auto_connect: bool,
    
    /// Whether to scan for networks periodically
    pub scan_always: bool,
    
    /// Scan interval in seconds
    pub scan_interval: u32,
    
    /// Power save mode enabled
    pub power_save: bool,
    
    /// Regulatory domain
    pub regulatory_domain: String,
    
    /// Preferred bands
    pub preferred_bands: Vec<WiFiBand>,
    
    /// Preferred networks (SSIDs)
    pub preferred_networks: Vec<String>,
    
    /// Blocked networks (SSIDs)
    pub blocked_networks: Vec<String>,
    
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl WiFiConfig {
    /// Create a new WiFiConfig with default values.
    pub fn new() -> Self {
        Self {
            enabled: true,
            auto_connect: true,
            scan_always: true,
            scan_interval: 60,
            power_save: true,
            regulatory_domain: "US".to_string(),
            preferred_bands: vec![WiFiBand::Band5GHz, WiFiBand::Band2_4GHz],
            preferred_networks: Vec::new(),
            blocked_networks: Vec::new(),
            custom_settings: HashMap::new(),
        }
    }
    
    /// Create a new WiFiConfig optimized for VR.
    pub fn vr_optimized() -> Self {
        Self {
            enabled: true,
            auto_connect: true,
            scan_always: false,
            scan_interval: 300,
            power_save: false,
            regulatory_domain: "US".to_string(),
            preferred_bands: vec![WiFiBand::Band5GHz, WiFiBand::Band6GHz],
            preferred_networks: Vec::new(),
            blocked_networks: Vec::new(),
            custom_settings: HashMap::new(),
        }
    }
}

/// Bluetooth device class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BluetoothDeviceClass {
    /// Computer
    Computer,
    
    /// Phone
    Phone,
    
    /// Headset
    Headset,
    
    /// Headphones
    Headphones,
    
    /// Speaker
    Speaker,
    
    /// Microphone
    Microphone,
    
    /// Keyboard
    Keyboard,
    
    /// Mouse
    Mouse,
    
    /// Gamepad
    Gamepad,
    
    /// Wearable
    Wearable,
    
    /// Health device
    Health,
    
    /// Toy
    Toy,
    
    /// Uncategorized
    Uncategorized,
    
    /// Unknown device class
    Unknown,
}

/// Bluetooth device information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BluetoothDeviceInfo {
    /// Device address
    pub address: String,
    
    /// Device name
    pub name: Option<String>,
    
    /// Device class
    pub device_class: BluetoothDeviceClass,
    
    /// Signal strength in dBm
    pub signal_strength: Option<i32>,
    
    /// Whether this device is paired
    pub paired: bool,
    
    /// Whether this device is connected
    pub connected: bool,
    
    /// Whether this device is trusted
    pub trusted: bool,
    
    /// Whether this device is blocked
    pub blocked: bool,
    
    /// Services provided by this device
    pub services: Vec<String>,
    
    /// Device manufacturer
    pub manufacturer: Option<String>,
    
    /// Device model
    pub model: Option<String>,
    
    /// Device details
    pub details: HashMap<String, String>,
}

/// Bluetooth configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BluetoothConfig {
    /// Whether Bluetooth is enabled
    pub enabled: bool,
    
    /// Whether the device is discoverable
    pub discoverable: bool,
    
    /// Discoverable timeout in seconds (0 = no timeout)
    pub discoverable_timeout: u32,
    
    /// Whether to automatically connect to paired devices
    pub auto_connect: bool,
    
    /// Power save mode enabled
    pub power_save: bool,
    
    /// Trusted devices (addresses)
    pub trusted_devices: Vec<String>,
    
    /// Blocked devices (addresses)
    pub blocked_devices: Vec<String>,
    
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl BluetoothConfig {
    /// Create a new BluetoothConfig with default values.
    pub fn new() -> Self {
        Self {
            enabled: true,
            discoverable: false,
            discoverable_timeout: 120,
            auto_connect: true,
            power_save: true,
            trusted_devices: Vec::new(),
            blocked_devices: Vec::new(),
            custom_settings: HashMap::new(),
        }
    }
    
    /// Create a new BluetoothConfig optimized for VR.
    pub fn vr_optimized() -> Self {
        Self {
            enabled: true,
            discoverable: false,
            discoverable_timeout: 0,
            auto_connect: true,
            power_save: false,
            trusted_devices: Vec::new(),
            blocked_devices: Vec::new(),
            custom_settings: HashMap::new(),
        }
    }
}

/// Network statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkStatistics {
    /// Bytes received
    pub bytes_received: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Packets received
    pub packets_received: u64,
    
    /// Packets sent
    pub packets_sent: u64,
    
    /// Receive errors
    pub receive_errors: u64,
    
    /// Send errors
    pub send_errors: u64,
    
    /// Dropped packets
    pub dropped_packets: u64,
    
    /// Current receive rate in bytes per second
    pub receive_rate: f64,
    
    /// Current send rate in bytes per second
    pub send_rate: f64,
    
    /// Current latency in milliseconds
    pub latency: f64,
    
    /// Current packet loss percentage
    pub packet_loss: f64,
    
    /// Statistics timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Network device trait.
pub trait NetworkDevice: Device {
    /// Get the network interface type.
    fn get_interface_type(&self) -> DeviceResult<NetworkInterfaceType>;
    
    /// Get the network interface state.
    fn get_interface_state(&self) -> DeviceResult<NetworkInterfaceState>;
    
    /// Set the network interface state.
    fn set_interface_state(&mut self, state: NetworkInterfaceState) -> DeviceResult<()>;
    
    /// Get the MAC address.
    fn get_mac_address(&self) -> DeviceResult<String>;
    
    /// Get the IP addresses.
    fn get_ip_addresses(&self) -> DeviceResult<Vec<IpAddr>>;
    
    /// Get the network statistics.
    fn get_statistics(&self) -> DeviceResult<NetworkStatistics>;
    
    /// Reset the network statistics.
    fn reset_statistics(&mut self) -> DeviceResult<()>;
    
    /// Test the network connection.
    fn test_connection(&self, target: &str, timeout: Duration) -> DeviceResult<bool>;
    
    /// Measure the network latency.
    fn measure_latency(&self, target: &str, count: u32) -> DeviceResult<f64>;
    
    /// Measure the network bandwidth.
    fn measure_bandwidth(&self, target: &str, duration: Duration) -> DeviceResult<f64>;
    
    /// Clone as NetworkDevice
    fn clone_network_box(&self) -> Box<dyn NetworkDevice>;
}

/// WiFi device trait.
pub trait WiFiDevice: NetworkDevice {
    /// Get the WiFi configuration.
    fn get_wifi_config(&self) -> DeviceResult<WiFiConfig>;
    
    /// Set the WiFi configuration.
    fn set_wifi_config(&mut self, config: &WiFiConfig) -> DeviceResult<()>;
    
    /// Scan for WiFi networks.
    fn scan_networks(&self) -> DeviceResult<Vec<WiFiNetworkInfo>>;
    
    /// Get the current WiFi connection status.
    fn get_connection_status(&self) -> DeviceResult<WiFiConnectionStatus>;
    
    /// Connect to a WiFi network.
    fn connect(&mut self, ssid: &str, password: Option<&str>) -> DeviceResult<bool>;
    
    /// Disconnect from the current WiFi network.
    fn disconnect(&mut self) -> DeviceResult<()>;
    
    /// Save a WiFi network configuration.
    fn save_network(&mut self, ssid: &str, password: Option<&str>) -> DeviceResult<()>;
    
    /// Forget a saved WiFi network.
    fn forget_network(&mut self, ssid: &str) -> DeviceResult<()>;
    
    /// Get the list of saved WiFi networks.
    fn get_saved_networks(&self) -> DeviceResult<Vec<String>>;
    
    /// Get the WiFi signal strength.
    fn get_signal_strength(&self) -> DeviceResult<i32>;
    
    /// Get the WiFi link speed.
    fn get_link_speed(&self) -> DeviceResult<u32>;
    
    /// Get the WiFi frequency.
    fn get_frequency(&self) -> DeviceResult<u32>;
    
    /// Get the WiFi channel.
    fn get_channel(&self) -> DeviceResult<u32>;
    
    /// Clone as WiFiDevice
    fn clone_wifi_box(&self) -> Box<dyn WiFiDevice>;
}

/// Bluetooth device trait.
pub trait BluetoothDevice: NetworkDevice {
    /// Get the Bluetooth configuration.
    fn get_bluetooth_config(&self) -> DeviceResult<BluetoothConfig>;
    
    /// Set the Bluetooth configuration.
    fn set_bluetooth_config(&mut self, config: &BluetoothConfig) -> DeviceResult<()>;
    
    /// Scan for Bluetooth devices.
    fn scan_devices(&self) -> DeviceResult<Vec<BluetoothDeviceInfo>>;
    
    /// Get the list of paired Bluetooth devices.
    fn get_paired_devices(&self) -> DeviceResult<Vec<BluetoothDeviceInfo>>;
    
    /// Get the list of connected Bluetooth devices.
    fn get_connected_devices(&self) -> DeviceResult<Vec<BluetoothDeviceInfo>>;
    
    /// Pair with a Bluetooth device.
    fn pair_device(&mut self, address: &str) -> DeviceResult<bool>;
    
    /// Connect to a paired Bluetooth device.
    fn connect_device(&mut self, address: &str) -> DeviceResult<bool>;
    
    /// Disconnect from a Bluetooth device.
    fn disconnect_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Remove a paired Bluetooth device.
    fn remove_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Trust a Bluetooth device.
    fn trust_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Untrust a Bluetooth device.
    fn untrust_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Block a Bluetooth device.
    fn block_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Unblock a Bluetooth device.
    fn unblock_device(&mut self, address: &str) -> DeviceResult<()>;
    
    /// Get information about a specific Bluetooth device.
    fn get_device_info(&self, address: &str) -> DeviceResult<BluetoothDeviceInfo>;
    
    /// Clone as BluetoothDevice
    fn clone_bluetooth_box(&self) -> Box<dyn BluetoothDevice>;
}

/// Network manager for managing network interfaces.
#[derive(Debug)]
pub struct NetworkManager {
    /// Network devices by ID
    devices: HashMap<String, Box<dyn NetworkDevice>>,
    
    /// Primary WiFi device ID
    primary_wifi_id: Option<String>,
    
    /// Primary Bluetooth device ID
    primary_bluetooth_id: Option<String>,
    
    /// Network monitoring enabled
    monitoring_enabled: bool,
    
    /// Network statistics
    statistics: HashMap<String, NetworkStatistics>,
}

impl NetworkManager {
    /// Create a new NetworkManager.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            primary_wifi_id: None,
            primary_bluetooth_id: None,
            monitoring_enabled: false,
            statistics: HashMap::new(),
        }
    }
    
    /// Initialize the network manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing NetworkManager");
        self.monitoring_enabled = true;
        Ok(())
    }
    
    /// Shutdown the network manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down NetworkManager");
        
        // Shutdown all network devices
        for (id, device) in &mut self.devices {
            info!("Shutting down network device {}", id);
            
            if let Err(e) = device.shutdown() {
                warn!("Failed to shutdown network device {}: {}", id, e);
            }
        }
        
        self.devices.clear();
        self.primary_wifi_id = None;
        self.primary_bluetooth_id = None;
        self.monitoring_enabled = false;
        self.statistics.clear();
        
        Ok(())
    }
    
    /// Register a network device.
    pub fn register_device(
        &mut self,
        id: &str,
        device: Box<dyn NetworkDevice>,
    ) -> DeviceResult<()> {
        info!("Registering network device {}", id);
        
        // Get the interface type
        let interface_type = device.get_interface_type()?;
        
        self.devices.insert(id.to_string(), device);
        
        // Set as primary WiFi or Bluetooth device if applicable
        match interface_type {
            NetworkInterfaceType::WiFi => {
                if self.primary_wifi_id.is_none() {
                    self.set_primary_wifi_device(id)?;
                }
            }
            NetworkInterfaceType::Bluetooth => {
                if self.primary_bluetooth_id.is_none() {
                    self.set_primary_bluetooth_device(id)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Unregister a network device.
    pub fn unregister_device(&mut self, id: &str) -> DeviceResult<()> {
        info!("Unregistering network device {}", id);
        
        if self.devices.remove(id).is_none() {
            return Err(DeviceError::NotFound(format!("Network device {} not found", id)));
        }
        
        // Update primary device IDs if necessary
        if Some(id.to_string()) == self.primary_wifi_id {
            self.primary_wifi_id = None;
            
            // Find another WiFi device to set as primary
            for (device_id, device) in &self.devices {
                if let Ok(interface_type) = device.get_interface_type() {
                    if interface_type == NetworkInterfaceType::WiFi {
                        self.primary_wifi_id = Some(device_id.clone());
                        break;
                    }
                }
            }
        } else if Some(id.to_string()) == self.primary_bluetooth_id {
            self.primary_bluetooth_id = None;
            
            // Find another Bluetooth device to set as primary
            for (device_id, device) in &self.devices {
                if let Ok(interface_type) = device.get_interface_type() {
                    if interface_type == NetworkInterfaceType::Bluetooth {
                        self.primary_bluetooth_id = Some(device_id.clone());
                        break;
                    }
                }
            }
        }
        
        // Remove statistics
        self.statistics.remove(id);
        
        Ok(())
    }
    
    /// Get a network device.
    pub fn get_device(&self, id: &str) -> DeviceResult<Box<dyn NetworkDevice>> {
        match self.devices.get(id) {
            Some(device) => Ok(device.clone_network_box()),
            None => Err(DeviceError::NotFound(format!("Network device {} not found", id)))
        }
    }
    
    /// Get all network devices.
    pub fn get_all_devices(&self) -> DeviceResult<Vec<DeviceInfo>> {
        let mut result = Vec::new();
        for device in self.devices.values() {
            result.push(device.info()?);
        }
        Ok(result)
    }
    
    /// Get network devices by type.
    pub fn get_devices_by_type(&self, interface_type: NetworkInterfaceType) -> DeviceResult<Vec<DeviceInfo>> {
        let mut result = Vec::new();
        for device in self.devices.values() {
            if let Ok(device_type) = device.get_interface_type() {
                if device_type == interface_type {
                    result.push(device.info()?);
                }
            }
        }
        Ok(result)
    }
    
    /// Get the primary WiFi device.
    pub fn get_primary_wifi_device(&self) -> DeviceResult<Box<dyn WiFiDevice>> {
        if let Some(id) = &self.primary_wifi_id {
            let device = self.get_device(id)?;
            
            // Try to cast to WiFiDevice
            if let Some(wifi_device) = self.cast_to_wifi_device(device) {
                return Ok(wifi_device);
            }
            
            return Err(DeviceError::InvalidState(format!(
                "Device {} is not a WiFi device", id
            )));
        }
        
        Err(DeviceError::NotFound("No primary WiFi device set".to_string()))
    }
    
    /// Get the primary Bluetooth device.
    pub fn get_primary_bluetooth_device(&self) -> DeviceResult<Box<dyn BluetoothDevice>> {
        if let Some(id) = &self.primary_bluetooth_id {
            let device = self.get_device(id)?;
            
            // Try to cast to BluetoothDevice
            if let Some(bluetooth_device) = self.cast_to_bluetooth_device(device) {
                return Ok(bluetooth_device);
            }
            
            return Err(DeviceError::InvalidState(format!(
                "Device {} is not a Bluetooth device", id
            )));
        }
        
        Err(DeviceError::NotFound("No primary Bluetooth device set".to_string()))
    }
    
    /// Set the primary WiFi device.
    pub fn set_primary_wifi_device(&mut self, id: &str) -> DeviceResult<()> {
        let device = self.get_device(id)?;
        
        // Verify that this is a WiFi device
        if let Ok(interface_type) = device.get_interface_type() {
            if interface_type != NetworkInterfaceType::WiFi {
                return Err(DeviceError::InvalidParameter(format!(
                    "Device {} is not a WiFi device", id
                )));
            }
        } else {
            return Err(DeviceError::InvalidState(format!(
                "Could not determine interface type for device {}", id
            )));
        }
        
        info!("Setting {} as primary WiFi device", id);
        self.primary_wifi_id = Some(id.to_string());
        Ok(())
    }
    
    /// Set the primary Bluetooth device.
    pub fn set_primary_bluetooth_device(&mut self, id: &str) -> DeviceResult<()> {
        let device = self.get_device(id)?;
        
        // Verify that this is a Bluetooth device
        if let Ok(interface_type) = device.get_interface_type() {
            if interface_type != NetworkInterfaceType::Bluetooth {
                return Err(DeviceError::InvalidParameter(format!(
                    "Device {} is not a Bluetooth device", id
                )));
            }
        } else {
            return Err(DeviceError::InvalidState(format!(
                "Could not determine interface type for device {}", id
            )));
        }
        
        info!("Setting {} as primary Bluetooth device", id);
        self.primary_bluetooth_id = Some(id.to_string());
        Ok(())
    }
    
    /// Enable or disable network monitoring.
    pub fn set_monitoring_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        info!("Setting network monitoring to {}", enabled);
        self.monitoring_enabled = enabled;
        
        if !enabled {
            // Clear statistics
            self.statistics.clear();
        }
        
        Ok(())
    }
    
    /// Check if network monitoring is enabled.
    pub fn is_monitoring_enabled(&self) -> bool {
        self.monitoring_enabled
    }
    
    /// Get network statistics for a device.
    pub fn get_device_statistics(&self, id: &str) -> DeviceResult<NetworkStatistics> {
        // First check if we have cached statistics
        if let Some(stats) = self.statistics.get(id) {
            return Ok(stats.clone());
        }
        
        // Otherwise, get fresh statistics from the device
        let device = self.get_device(id)?;
        device.get_statistics()
    }
    
    /// Get network statistics for all devices.
    pub fn get_all_statistics(&self) -> DeviceResult<HashMap<String, NetworkStatistics>> {
        let mut result = HashMap::new();
        
        for (id, device) in &self.devices {
            if let Ok(stats) = device.get_statistics() {
                result.insert(id.clone(), stats);
            }
        }
        
        Ok(result)
    }
    
    /// Update network statistics.
    pub fn update_statistics(&mut self) -> DeviceResult<()> {
        if !self.monitoring_enabled {
            return Ok(());
        }
        
        for (id, device) in &self.devices {
            if let Ok(stats) = device.get_statistics() {
                self.statistics.insert(id.clone(), stats);
            }
        }
        
        Ok(())
    }
    
    /// Test the network connection.
    pub fn test_connection(&self, target: &str, timeout: Duration) -> DeviceResult<bool> {
        // Try with primary WiFi device first
        if let Some(id) = &self.primary_wifi_id {
            if let Ok(device) = self.get_device(id) {
                return device.test_connection(target, timeout);
            }
        }
        
        // Try with any other device
        for device in self.devices.values() {
            if let Ok(result) = device.test_connection(target, timeout) {
                return Ok(result);
            }
        }
        
        Err(DeviceError::NotFound("No network device available".to_string()))
    }
    
    /// Measure the network latency.
    pub fn measure_latency(&self, target: &str, count: u32) -> DeviceResult<f64> {
        // Try with primary WiFi device first
        if let Some(id) = &self.primary_wifi_id {
            if let Ok(device) = self.get_device(id) {
                return device.measure_latency(target, count);
            }
        }
        
        // Try with any other device
        for device in self.devices.values() {
            if let Ok(result) = device.measure_latency(target, count) {
                return Ok(result);
            }
        }
        
        Err(DeviceError::NotFound("No network device available".to_string()))
    }
    
    /// Measure the network bandwidth.
    pub fn measure_bandwidth(&self, target: &str, duration: Duration) -> DeviceResult<f64> {
        // Try with primary WiFi device first
        if let Some(id) = &self.primary_wifi_id {
            if let Ok(device) = self.get_device(id) {
                return device.measure_bandwidth(target, duration);
            }
        }
        
        // Try with any other device
        for device in self.devices.values() {
            if let Ok(result) = device.measure_bandwidth(target, duration) {
                return Ok(result);
            }
        }
        
        Err(DeviceError::NotFound("No network device available".to_string()))
    }
    
    /// Cast a NetworkDevice to a WiFiDevice.
    fn cast_to_wifi_device(&self, device: Box<dyn NetworkDevice>) -> Option<Box<dyn WiFiDevice>> {
        // This would be implemented with proper dynamic casting in a real implementation
        // For now, we'll return None as this is just a placeholder
        None
    }
    
    /// Cast a NetworkDevice to a BluetoothDevice.
    fn cast_to_bluetooth_device(&self, device: Box<dyn NetworkDevice>) -> Option<Box<dyn BluetoothDevice>> {
        // This would be implemented with proper dynamic casting in a real implementation
        // For now, we'll return None as this is just a placeholder
        None
    }
}
