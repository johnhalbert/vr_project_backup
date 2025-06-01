//! Network monitoring for the VR headset.
//!
//! This module provides comprehensive network monitoring capabilities
//! for tracking network interfaces, WiFi connections, and network performance.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::metrics::{Metric, MetricsCollector, MetricType, MetricValue};

/// WiFi connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WiFiState {
    /// WiFi is connected
    Connected,
    
    /// WiFi is connecting
    Connecting,
    
    /// WiFi is disconnected
    Disconnected,
    
    /// WiFi is disabled
    Disabled,
}

impl WiFiState {
    /// Get the WiFi state as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            WiFiState::Connected => "connected",
            WiFiState::Connecting => "connecting",
            WiFiState::Disconnected => "disconnected",
            WiFiState::Disabled => "disabled",
        }
    }
    
    /// Parse a WiFi state from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "connected" => Some(WiFiState::Connected),
            "connecting" => Some(WiFiState::Connecting),
            "disconnected" => Some(WiFiState::Disconnected),
            "disabled" => Some(WiFiState::Disabled),
            _ => None,
        }
    }
}

impl std::fmt::Display for WiFiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Network interface statistics.
#[derive(Debug, Clone)]
pub struct NetworkInterfaceStats {
    /// Interface name
    pub name: String,
    
    /// Interface type (e.g., wifi, ethernet, loopback)
    pub interface_type: String,
    
    /// Whether the interface is up
    pub is_up: bool,
    
    /// MAC address
    pub mac_address: String,
    
    /// IP address
    pub ip_address: Option<String>,
    
    /// Bytes received
    pub rx_bytes: u64,
    
    /// Bytes transmitted
    pub tx_bytes: u64,
    
    /// Packets received
    pub rx_packets: u64,
    
    /// Packets transmitted
    pub tx_packets: u64,
    
    /// Receive errors
    pub rx_errors: u64,
    
    /// Transmit errors
    pub tx_errors: u64,
    
    /// Receive drops
    pub rx_drops: u64,
    
    /// Transmit drops
    pub tx_drops: u64,
}

/// WiFi connection statistics.
#[derive(Debug, Clone)]
pub struct WiFiStats {
    /// SSID
    pub ssid: String,
    
    /// BSSID
    pub bssid: String,
    
    /// Connection state
    pub state: WiFiState,
    
    /// Signal strength (dBm)
    pub signal_strength: i32,
    
    /// Link quality (percentage)
    pub link_quality: u8,
    
    /// Frequency (MHz)
    pub frequency: u32,
    
    /// Bitrate (Mbps)
    pub bitrate: u32,
    
    /// Security type (e.g., WPA2, WPA3)
    pub security_type: String,
}

/// Network metrics collector.
#[derive(Debug)]
pub struct NetworkMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Last network interface statistics
    last_stats: Mutex<HashMap<String, NetworkInterfaceStats>>,
    
    /// Current WiFi state
    wifi_state: Mutex<WiFiState>,
}

impl NetworkMetricsCollector {
    /// Create a new network metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "network".to_string(),
            interval: Duration::from_secs(interval_secs),
            last_stats: Mutex::new(HashMap::new()),
            wifi_state: Mutex::new(WiFiState::Disconnected),
        }
    }
    
    /// Set the WiFi state.
    pub fn set_wifi_state(&self, state: WiFiState) {
        let mut current_state = self.wifi_state.lock().unwrap();
        *current_state = state;
    }
    
    /// Get the WiFi state.
    pub fn get_wifi_state(&self) -> WiFiState {
        let state = self.wifi_state.lock().unwrap();
        *state
    }
    
    /// Get network interface statistics.
    fn get_network_interface_stats(&self) -> Vec<NetworkInterfaceStats> {
        // In a real implementation, this would read from /proc/net/dev or use a library
        // For now, we'll simulate some values
        
        // WiFi interface
        let wifi = NetworkInterfaceStats {
            name: "wlan0".to_string(),
            interface_type: "wifi".to_string(),
            is_up: self.get_wifi_state() == WiFiState::Connected,
            mac_address: "00:11:22:33:44:55".to_string(),
            ip_address: if self.get_wifi_state() == WiFiState::Connected {
                Some("192.168.1.100".to_string())
            } else {
                None
            },
            rx_bytes: 1_500_000,
            tx_bytes: 500_000,
            rx_packets: 10_000,
            tx_packets: 5_000,
            rx_errors: 10,
            tx_errors: 5,
            rx_drops: 20,
            tx_drops: 10,
        };
        
        // Loopback interface
        let loopback = NetworkInterfaceStats {
            name: "lo".to_string(),
            interface_type: "loopback".to_string(),
            is_up: true,
            mac_address: "00:00:00:00:00:00".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            rx_bytes: 100_000,
            tx_bytes: 100_000,
            rx_packets: 1_000,
            tx_packets: 1_000,
            rx_errors: 0,
            tx_errors: 0,
            rx_drops: 0,
            tx_drops: 0,
        };
        
        vec![wifi, loopback]
    }
    
    /// Get WiFi statistics.
    fn get_wifi_stats(&self) -> Option<WiFiStats> {
        // In a real implementation, this would use platform-specific APIs
        // For now, we'll simulate some values based on the WiFi state
        
        match self.get_wifi_state() {
            WiFiState::Connected => Some(WiFiStats {
                ssid: "HomeNetwork".to_string(),
                bssid: "00:11:22:33:44:55".to_string(),
                state: WiFiState::Connected,
                signal_strength: -65,
                link_quality: 70,
                frequency: 5240,
                bitrate: 300,
                security_type: "WPA2".to_string(),
            }),
            WiFiState::Connecting => Some(WiFiStats {
                ssid: "HomeNetwork".to_string(),
                bssid: "00:11:22:33:44:55".to_string(),
                state: WiFiState::Connecting,
                signal_strength: -65,
                link_quality: 70,
                frequency: 5240,
                bitrate: 0,
                security_type: "WPA2".to_string(),
            }),
            _ => None,
        }
    }
    
    /// Calculate network interface rates.
    fn calculate_interface_rates(&self, current: &NetworkInterfaceStats, previous: &NetworkInterfaceStats) -> (f64, f64) {
        // Calculate rates in bytes per second
        let rx_rate = if current.rx_bytes >= previous.rx_bytes {
            (current.rx_bytes - previous.rx_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        let tx_rate = if current.tx_bytes >= previous.tx_bytes {
            (current.tx_bytes - previous.tx_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        (rx_rate, tx_rate)
    }
}

impl MetricsCollector for NetworkMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get current network interface statistics
        let current_stats = self.get_network_interface_stats();
        
        // Get last statistics for rate calculation
        let mut last_stats = self.last_stats.lock().unwrap();
        
        // Process each interface
        for interface in &current_stats {
            // Interface labels
            let mut labels = HashMap::new();
            labels.insert("interface".to_string(), interface.name.clone());
            labels.insert("type".to_string(), interface.interface_type.clone());
            
            // Interface state metric
            metrics.push(Metric::new(
                &format!("network.interface.up"),
                MetricType::State,
                MetricValue::Boolean(interface.is_up),
                Some(labels.clone()),
                Some("Interface state (up/down)"),
                None,
            ));
            
            // IP address metric
            if let Some(ip) = &interface.ip_address {
                metrics.push(Metric::new(
                    &format!("network.interface.ip"),
                    MetricType::State,
                    MetricValue::String(ip.clone()),
                    Some(labels.clone()),
                    Some("Interface IP address"),
                    None,
                ));
            }
            
            // Bytes metrics
            metrics.push(Metric::new(
                &format!("network.interface.rx_bytes"),
                MetricType::Counter,
                MetricValue::Integer(interface.rx_bytes as i64),
                Some(labels.clone()),
                Some("Total bytes received"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                &format!("network.interface.tx_bytes"),
                MetricType::Counter,
                MetricValue::Integer(interface.tx_bytes as i64),
                Some(labels.clone()),
                Some("Total bytes transmitted"),
                Some("bytes"),
            ));
            
            // Packets metrics
            metrics.push(Metric::new(
                &format!("network.interface.rx_packets"),
                MetricType::Counter,
                MetricValue::Integer(interface.rx_packets as i64),
                Some(labels.clone()),
                Some("Total packets received"),
                Some("packets"),
            ));
            
            metrics.push(Metric::new(
                &format!("network.interface.tx_packets"),
                MetricType::Counter,
                MetricValue::Integer(interface.tx_packets as i64),
                Some(labels.clone()),
                Some("Total packets transmitted"),
                Some("packets"),
            ));
            
            // Error metrics
            metrics.push(Metric::new(
                &format!("network.interface.rx_errors"),
                MetricType::Counter,
                MetricValue::Integer(interface.rx_errors as i64),
                Some(labels.clone()),
                Some("Receive errors"),
                Some("errors"),
            ));
            
            metrics.push(Metric::new(
                &format!("network.interface.tx_errors"),
                MetricType::Counter,
                MetricValue::Integer(interface.tx_errors as i64),
                Some(labels.clone()),
                Some("Transmit errors"),
                Some("errors"),
            ));
            
            // Drop metrics
            metrics.push(Metric::new(
                &format!("network.interface.rx_drops"),
                MetricType::Counter,
                MetricValue::Integer(interface.rx_drops as i64),
                Some(labels.clone()),
                Some("Receive drops"),
                Some("drops"),
            ));
            
            metrics.push(Metric::new(
                &format!("network.interface.tx_drops"),
                MetricType::Counter,
                MetricValue::Integer(interface.tx_drops as i64),
                Some(labels.clone()),
                Some("Transmit drops"),
                Some("drops"),
            ));
            
            // Calculate rates if we have previous stats
            if let Some(previous) = last_stats.get(&interface.name) {
                let (rx_rate, tx_rate) = self.calculate_interface_rates(interface, previous);
                
                // Rate metrics (bytes per second)
                metrics.push(Metric::new(
                    &format!("network.interface.rx_rate"),
                    MetricType::Gauge,
                    MetricValue::Float(rx_rate),
                    Some(labels.clone()),
                    Some("Receive rate"),
                    Some("bytes/s"),
                ));
                
                metrics.push(Metric::new(
                    &format!("network.interface.tx_rate"),
                    MetricType::Gauge,
                    MetricValue::Float(tx_rate),
                    Some(labels.clone()),
                    Some("Transmit rate"),
                    Some("bytes/s"),
                ));
                
                // Rate metrics (bits per second for display)
                metrics.push(Metric::new(
                    &format!("network.interface.rx_rate_mbps"),
                    MetricType::Gauge,
                    MetricValue::Float(rx_rate * 8.0 / 1_000_000.0),
                    Some(labels.clone()),
                    Some("Receive rate"),
                    Some("Mbps"),
                ));
                
                metrics.push(Metric::new(
                    &format!("network.interface.tx_rate_mbps"),
                    MetricType::Gauge,
                    MetricValue::Float(tx_rate * 8.0 / 1_000_000.0),
                    Some(labels.clone()),
                    Some("Transmit rate"),
                    Some("Mbps"),
                ));
            }
            
            // Update last stats
            last_stats.insert(interface.name.clone(), interface.clone());
        }
        
        // WiFi-specific metrics
        if let Some(wifi) = self.get_wifi_stats() {
            let mut wifi_labels = HashMap::new();
            wifi_labels.insert("interface".to_string(), "wlan0".to_string());
            wifi_labels.insert("ssid".to_string(), wifi.ssid.clone());
            
            // WiFi state metric
            metrics.push(Metric::new(
                "network.wifi.state",
                MetricType::State,
                MetricValue::String(wifi.state.to_string()),
                Some(wifi_labels.clone()),
                Some("WiFi connection state"),
                None,
            ));
            
            // Signal strength metric
            metrics.push(Metric::new(
                "network.wifi.signal_strength",
                MetricType::Gauge,
                MetricValue::Integer(wifi.signal_strength as i64),
                Some(wifi_labels.clone()),
                Some("WiFi signal strength"),
                Some("dBm"),
            ));
            
            // Link quality metric
            metrics.push(Metric::new(
                "network.wifi.link_quality",
                MetricType::Gauge,
                MetricValue::Integer(wifi.link_quality as i64),
                Some(wifi_labels.clone()),
                Some("WiFi link quality"),
                Some("%"),
            ));
            
            // Frequency metric
            metrics.push(Metric::new(
                "network.wifi.frequency",
                MetricType::Gauge,
                MetricValue::Integer(wifi.frequency as i64),
                Some(wifi_labels.clone()),
                Some("WiFi frequency"),
                Some("MHz"),
            ));
            
            // Bitrate metric
            metrics.push(Metric::new(
                "network.wifi.bitrate",
                MetricType::Gauge,
                MetricValue::Integer(wifi.bitrate as i64),
                Some(wifi_labels.clone()),
                Some("WiFi bitrate"),
                Some("Mbps"),
            ));
            
            // Security type metric
            metrics.push(Metric::new(
                "network.wifi.security_type",
                MetricType::State,
                MetricValue::String(wifi.security_type.clone()),
                Some(wifi_labels),
                Some("WiFi security type"),
                None,
            ));
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Network latency metrics collector.
#[derive(Debug)]
pub struct NetworkLatencyMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
}

impl NetworkLatencyMetricsCollector {
    /// Create a new network latency metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "network_latency".to_string(),
            interval: Duration::from_secs(interval_secs),
        }
    }
    
    /// Simulate ping measurements.
    fn simulate_ping_measurements(&self) -> HashMap<String, f64> {
        // In a real implementation, this would perform actual ping measurements
        // For now, we'll simulate some values
        let mut results = HashMap::new();
        results.insert("local_gateway".to_string(), 2.5);
        results.insert("cloud_server".to_string(), 35.0);
        results.insert("content_cdn".to_string(), 25.0);
        results
    }
}

impl MetricsCollector for NetworkLatencyMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get ping measurements
        let ping_results = self.simulate_ping_measurements();
        
        // Create metrics for each target
        for (target, latency) in ping_results {
            let mut labels = HashMap::new();
            labels.insert("target".to_string(), target.clone());
            
            metrics.push(Metric::new(
                "network.latency.ping",
                MetricType::Gauge,
                MetricValue::Float(latency),
                Some(labels),
                Some("Network ping latency"),
                Some("ms"),
            ));
        }
        
        // Connection quality metric (simulated)
        let mut labels = HashMap::new();
        labels.insert("connection".to_string(), "overall".to_string());
        
        metrics.push(Metric::new(
            "network.connection.quality",
            MetricType::Gauge,
            MetricValue::Integer(85),
            Some(labels.clone()),
            Some("Overall connection quality"),
            Some("%"),
        ));
        
        // Jitter metric (simulated)
        metrics.push(Metric::new(
            "network.connection.jitter",
            MetricType::Gauge,
            MetricValue::Float(3.5),
            Some(labels.clone()),
            Some("Connection jitter"),
            Some("ms"),
        ));
        
        // Packet loss metric (simulated)
        metrics.push(Metric::new(
            "network.connection.packet_loss",
            MetricType::Gauge,
            MetricValue::Float(0.5),
            Some(labels),
            Some("Connection packet loss"),
            Some("%"),
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Network monitor.
#[derive(Debug)]
pub struct NetworkMonitor {
    /// Network metrics collector
    network_collector: Arc<NetworkMetricsCollector>,
    
    /// Network latency metrics collector
    latency_collector: Arc<NetworkLatencyMetricsCollector>,
}

impl NetworkMonitor {
    /// Create a new network monitor.
    pub fn new() -> Self {
        let network_collector = Arc::new(NetworkMetricsCollector::new(5));
        let latency_collector = Arc::new(NetworkLatencyMetricsCollector::new(10));
        
        Self {
            network_collector,
            latency_collector,
        }
    }
    
    /// Get the network metrics collector.
    pub fn network_collector(&self) -> Arc<NetworkMetricsCollector> {
        self.network_collector.clone()
    }
    
    /// Get the network latency metrics collector.
    pub fn latency_collector(&self) -> Arc<NetworkLatencyMetricsCollector> {
        self.latency_collector.clone()
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> Vec<Arc<dyn MetricsCollector>> {
        vec![
            self.network_collector.clone() as Arc<dyn MetricsCollector>,
            self.latency_collector.clone() as Arc<dyn MetricsCollector>,
        ]
    }
    
    /// Set the WiFi state.
    pub fn set_wifi_state(&self, state: WiFiState) {
        self.network_collector.set_wifi_state(state);
    }
    
    /// Get the WiFi state.
    pub fn get_wifi_state(&self) -> WiFiState {
        self.network_collector.get_wifi_state()
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wifi_state() {
        assert_eq!(WiFiState::Connected.as_str(), "connected");
        assert_eq!(WiFiState::Connecting.as_str(), "connecting");
        assert_eq!(WiFiState::Disconnected.as_str(), "disconnected");
        assert_eq!(WiFiState::Disabled.as_str(), "disabled");
        
        assert_eq!(WiFiState::from_str("connected"), Some(WiFiState::Connected));
        assert_eq!(WiFiState::from_str("connecting"), Some(WiFiState::Connecting));
        assert_eq!(WiFiState::from_str("disconnected"), Some(WiFiState::Disconnected));
        assert_eq!(WiFiState::from_str("disabled"), Some(WiFiState::Disabled));
        assert_eq!(WiFiState::from_str("invalid"), None);
        
        assert_eq!(WiFiState::Connected.to_string(), "connected");
        assert_eq!(WiFiState::Connecting.to_string(), "connecting");
        assert_eq!(WiFiState::Disconnected.to_string(), "disconnected");
        assert_eq!(WiFiState::Disabled.to_string(), "disabled");
    }
    
    #[test]
    fn test_network_metrics_collector() {
        let collector = NetworkMetricsCollector::new(5);
        
        // Check default state
        assert_eq!(collector.get_wifi_state(), WiFiState::Disconnected);
        
        // Set and get WiFi state
        collector.set_wifi_state(WiFiState::Connected);
        assert_eq!(collector.get_wifi_state(), WiFiState::Connected);
        
        // First collection
        let metrics = collector.collect();
        
        // Check interface metrics
        assert!(metrics.iter().any(|m| m.name == "network.interface.up"));
        assert!(metrics.iter().any(|m| m.name == "network.interface.rx_bytes"));
        assert!(metrics.iter().any(|m| m.name == "network.interface.tx_bytes"));
        
        // Check WiFi metrics when connected
        assert!(metrics.iter().any(|m| m.name == "network.wifi.state"));
        assert!(metrics.iter().any(|m| m.name == "network.wifi.signal_strength"));
        assert!(metrics.iter().any(|m| m.name == "network.wifi.link_quality"));
        
        // Second collection should include rate metrics
        let metrics = collector.collect();
        assert!(metrics.iter().any(|m| m.name == "network.interface.rx_rate"));
        assert!(metrics.iter().any(|m| m.name == "network.interface.tx_rate"));
        assert!(metrics.iter().any(|m| m.name == "network.interface.rx_rate_mbps"));
        assert!(metrics.iter().any(|m| m.name == "network.interface.tx_rate_mbps"));
        
        // Check WiFi state metric
        collector.set_wifi_state(WiFiState::Disconnected);
        let metrics = collector.collect();
        let wifi_metrics = metrics.iter().filter(|m| m.name == "network.wifi.state").count();
        assert_eq!(wifi_metrics, 0); // No WiFi metrics when disconnected
    }
    
    #[test]
    fn test_network_latency_metrics_collector() {
        let collector = NetworkLatencyMetricsCollector::new(10);
        let metrics = collector.collect();
        
        // Check latency metrics
        assert!(metrics.iter().any(|m| m.name == "network.latency.ping"));
        
        // Check connection quality metrics
        assert!(metrics.iter().any(|m| m.name == "network.connection.quality"));
        assert!(metrics.iter().any(|m| m.name == "network.connection.jitter"));
        assert!(metrics.iter().any(|m| m.name == "network.connection.packet_loss"));
    }
    
    #[test]
    fn test_network_monitor() {
        let monitor = NetworkMonitor::new();
        
        // Check collectors
        assert_eq!(monitor.collectors().len(), 2);
        
        // Set and get WiFi state
        monitor.set_wifi_state(WiFiState::Connected);
        assert_eq!(monitor.get_wifi_state(), WiFiState::Connected);
        
        // Check collector access
        assert_eq!(monitor.network_collector().get_wifi_state(), WiFiState::Connected);
    }
}
