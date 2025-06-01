//! Battery and power monitoring for the VR headset.
//!
//! This module provides comprehensive battery and power monitoring capabilities
//! for tracking battery status, power consumption, and charging state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::metrics::{Metric, MetricsCollector, MetricType, MetricValue};

/// Battery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatteryState {
    /// Battery is charging
    Charging,
    
    /// Battery is discharging
    Discharging,
    
    /// Battery is fully charged
    Full,
    
    /// Battery state is unknown
    Unknown,
}

impl BatteryState {
    /// Get the battery state as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            BatteryState::Charging => "charging",
            BatteryState::Discharging => "discharging",
            BatteryState::Full => "full",
            BatteryState::Unknown => "unknown",
        }
    }
    
    /// Parse a battery state from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "charging" => Some(BatteryState::Charging),
            "discharging" => Some(BatteryState::Discharging),
            "full" => Some(BatteryState::Full),
            "unknown" => Some(BatteryState::Unknown),
            _ => None,
        }
    }
}

impl std::fmt::Display for BatteryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Power profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerProfile {
    /// Power saving mode
    PowerSave,
    
    /// Balanced mode
    Balanced,
    
    /// Performance mode
    Performance,
    
    /// Custom mode
    Custom,
}

impl PowerProfile {
    /// Get the power profile as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            PowerProfile::PowerSave => "power_save",
            PowerProfile::Balanced => "balanced",
            PowerProfile::Performance => "performance",
            PowerProfile::Custom => "custom",
        }
    }
    
    /// Parse a power profile from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "power_save" => Some(PowerProfile::PowerSave),
            "balanced" => Some(PowerProfile::Balanced),
            "performance" => Some(PowerProfile::Performance),
            "custom" => Some(PowerProfile::Custom),
            _ => None,
        }
    }
}

impl std::fmt::Display for PowerProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Battery metrics collector.
#[derive(Debug)]
pub struct BatteryMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Current battery state
    battery_state: Mutex<BatteryState>,
    
    /// Current power profile
    power_profile: Mutex<PowerProfile>,
}

impl BatteryMetricsCollector {
    /// Create a new battery metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "battery".to_string(),
            interval: Duration::from_secs(interval_secs),
            battery_state: Mutex::new(BatteryState::Discharging),
            power_profile: Mutex::new(PowerProfile::Balanced),
        }
    }
    
    /// Set the battery state.
    pub fn set_battery_state(&self, state: BatteryState) {
        let mut current_state = self.battery_state.lock().unwrap();
        *current_state = state;
    }
    
    /// Get the battery state.
    pub fn get_battery_state(&self) -> BatteryState {
        let state = self.battery_state.lock().unwrap();
        *state
    }
    
    /// Set the power profile.
    pub fn set_power_profile(&self, profile: PowerProfile) {
        let mut current_profile = self.power_profile.lock().unwrap();
        *current_profile = profile;
    }
    
    /// Get the power profile.
    pub fn get_power_profile(&self) -> PowerProfile {
        let profile = self.power_profile.lock().unwrap();
        *profile
    }
}

impl MetricsCollector for BatteryMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Battery labels
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "battery".to_string());
        
        // Battery level metric (simulated)
        metrics.push(Metric::new(
            "battery.level",
            MetricType::Gauge,
            MetricValue::Integer(75),
            Some(labels.clone()),
            Some("Battery level percentage"),
            Some("%"),
        ));
        
        // Battery voltage metric (simulated)
        metrics.push(Metric::new(
            "battery.voltage",
            MetricType::Gauge,
            MetricValue::Float(3.85),
            Some(labels.clone()),
            Some("Battery voltage"),
            Some("V"),
        ));
        
        // Battery current metric (simulated)
        let current = match self.get_battery_state() {
            BatteryState::Charging => 1.2,
            BatteryState::Discharging => -0.8,
            _ => 0.0,
        };
        
        metrics.push(Metric::new(
            "battery.current",
            MetricType::Gauge,
            MetricValue::Float(current),
            Some(labels.clone()),
            Some("Battery current (positive when charging, negative when discharging)"),
            Some("A"),
        ));
        
        // Battery temperature metric (simulated)
        metrics.push(Metric::new(
            "battery.temperature",
            MetricType::Gauge,
            MetricValue::Float(35.0),
            Some(labels.clone()),
            Some("Battery temperature"),
            Some("°C"),
        ));
        
        // Battery state metric
        metrics.push(Metric::new(
            "battery.state",
            MetricType::State,
            MetricValue::String(self.get_battery_state().to_string()),
            Some(labels.clone()),
            Some("Battery state"),
            None,
        ));
        
        // Battery health metric (simulated)
        metrics.push(Metric::new(
            "battery.health",
            MetricType::Gauge,
            MetricValue::Integer(95),
            Some(labels.clone()),
            Some("Battery health percentage"),
            Some("%"),
        ));
        
        // Battery cycle count metric (simulated)
        metrics.push(Metric::new(
            "battery.cycles",
            MetricType::Counter,
            MetricValue::Integer(120),
            Some(labels.clone()),
            Some("Battery charge cycles"),
            None,
        ));
        
        // Power consumption metrics
        let mut power_labels = HashMap::new();
        power_labels.insert("component".to_string(), "power".to_string());
        
        // Total power consumption (simulated)
        metrics.push(Metric::new(
            "power.consumption.total",
            MetricType::Gauge,
            MetricValue::Float(4.5),
            Some(power_labels.clone()),
            Some("Total power consumption"),
            Some("W"),
        ));
        
        // Component power consumption (simulated)
        let components = vec![
            ("cpu", 1.2),
            ("gpu", 2.0),
            ("display", 0.8),
            ("sensors", 0.3),
            ("audio", 0.2),
        ];
        
        for (component, power) in components {
            let mut comp_labels = HashMap::new();
            comp_labels.insert("component".to_string(), component.to_string());
            
            metrics.push(Metric::new(
                &format!("power.consumption.{}", component),
                MetricType::Gauge,
                MetricValue::Float(power),
                Some(comp_labels),
                Some(&format!("{} power consumption", component)),
                Some("W"),
            ));
        }
        
        // Power profile metric
        metrics.push(Metric::new(
            "power.profile",
            MetricType::State,
            MetricValue::String(self.get_power_profile().to_string()),
            Some(power_labels.clone()),
            Some("Current power profile"),
            None,
        ));
        
        // Estimated runtime metric (simulated)
        let runtime = match self.get_battery_state() {
            BatteryState::Discharging => 180, // 3 hours in minutes
            _ => 0,
        };
        
        metrics.push(Metric::new(
            "battery.runtime",
            MetricType::Gauge,
            MetricValue::Integer(runtime),
            Some(labels),
            Some("Estimated runtime remaining"),
            Some("min"),
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Power management metrics collector.
#[derive(Debug)]
pub struct PowerManagementMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
}

impl PowerManagementMetricsCollector {
    /// Create a new power management metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "power_management".to_string(),
            interval: Duration::from_secs(interval_secs),
        }
    }
}

impl MetricsCollector for PowerManagementMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Power management labels
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "power_management".to_string());
        
        // CPU frequency scaling governor (simulated)
        metrics.push(Metric::new(
            "power.cpu.governor",
            MetricType::State,
            MetricValue::String("ondemand".to_string()),
            Some(labels.clone()),
            Some("CPU frequency scaling governor"),
            None,
        ));
        
        // GPU power state (simulated)
        metrics.push(Metric::new(
            "power.gpu.state",
            MetricType::State,
            MetricValue::String("auto".to_string()),
            Some(labels.clone()),
            Some("GPU power state"),
            None,
        ));
        
        // Display brightness (simulated)
        metrics.push(Metric::new(
            "power.display.brightness",
            MetricType::Gauge,
            MetricValue::Integer(80),
            Some(labels.clone()),
            Some("Display brightness level"),
            Some("%"),
        ));
        
        // Thermal throttling (simulated)
        metrics.push(Metric::new(
            "power.thermal.throttling",
            MetricType::State,
            MetricValue::Boolean(false),
            Some(labels.clone()),
            Some("Thermal throttling active"),
            None,
        ));
        
        // Power saving features (simulated)
        let features = vec![
            ("wifi", true),
            ("bluetooth", true),
            ("cpu_cores", false),
            ("gpu_cores", false),
        ];
        
        for (feature, enabled) in features {
            let mut feature_labels = HashMap::new();
            feature_labels.insert("feature".to_string(), feature.to_string());
            
            metrics.push(Metric::new(
                &format!("power.saving.{}", feature),
                MetricType::State,
                MetricValue::Boolean(!enabled), // Inverted: true means power saving is active (feature disabled)
                Some(feature_labels),
                Some(&format!("{} power saving active", feature)),
                None,
            ));
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Power monitor.
#[derive(Debug)]
pub struct PowerMonitor {
    /// Battery metrics collector
    battery_collector: Arc<BatteryMetricsCollector>,
    
    /// Power management metrics collector
    power_management_collector: Arc<PowerManagementMetricsCollector>,
}

impl PowerMonitor {
    /// Create a new power monitor.
    pub fn new() -> Self {
        let battery_collector = Arc::new(BatteryMetricsCollector::new(5));
        let power_management_collector = Arc::new(PowerManagementMetricsCollector::new(10));
        
        Self {
            battery_collector,
            power_management_collector,
        }
    }
    
    /// Get the battery metrics collector.
    pub fn battery_collector(&self) -> Arc<BatteryMetricsCollector> {
        self.battery_collector.clone()
    }
    
    /// Get the power management metrics collector.
    pub fn power_management_collector(&self) -> Arc<PowerManagementMetricsCollector> {
        self.power_management_collector.clone()
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> Vec<Arc<dyn MetricsCollector>> {
        vec![
            self.battery_collector.clone() as Arc<dyn MetricsCollector>,
            self.power_management_collector.clone() as Arc<dyn MetricsCollector>,
        ]
    }
    
    /// Set the battery state.
    pub fn set_battery_state(&self, state: BatteryState) {
        self.battery_collector.set_battery_state(state);
    }
    
    /// Get the battery state.
    pub fn get_battery_state(&self) -> BatteryState {
        self.battery_collector.get_battery_state()
    }
    
    /// Set the power profile.
    pub fn set_power_profile(&self, profile: PowerProfile) {
        self.battery_collector.set_power_profile(profile);
    }
    
    /// Get the power profile.
    pub fn get_power_profile(&self) -> PowerProfile {
        self.battery_collector.get_power_profile()
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_battery_state() {
        assert_eq!(BatteryState::Charging.as_str(), "charging");
        assert_eq!(BatteryState::Discharging.as_str(), "discharging");
        assert_eq!(BatteryState::Full.as_str(), "full");
        assert_eq!(BatteryState::Unknown.as_str(), "unknown");
        
        assert_eq!(BatteryState::from_str("charging"), Some(BatteryState::Charging));
        assert_eq!(BatteryState::from_str("discharging"), Some(BatteryState::Discharging));
        assert_eq!(BatteryState::from_str("full"), Some(BatteryState::Full));
        assert_eq!(BatteryState::from_str("unknown"), Some(BatteryState::Unknown));
        assert_eq!(BatteryState::from_str("invalid"), None);
        
        assert_eq!(BatteryState::Charging.to_string(), "charging");
        assert_eq!(BatteryState::Discharging.to_string(), "discharging");
        assert_eq!(BatteryState::Full.to_string(), "full");
        assert_eq!(BatteryState::Unknown.to_string(), "unknown");
    }
    
    #[test]
    fn test_power_profile() {
        assert_eq!(PowerProfile::PowerSave.as_str(), "power_save");
        assert_eq!(PowerProfile::Balanced.as_str(), "balanced");
        assert_eq!(PowerProfile::Performance.as_str(), "performance");
        assert_eq!(PowerProfile::Custom.as_str(), "custom");
        
        assert_eq!(PowerProfile::from_str("power_save"), Some(PowerProfile::PowerSave));
        assert_eq!(PowerProfile::from_str("balanced"), Some(PowerProfile::Balanced));
        assert_eq!(PowerProfile::from_str("performance"), Some(PowerProfile::Performance));
        assert_eq!(PowerProfile::from_str("custom"), Some(PowerProfile::Custom));
        assert_eq!(PowerProfile::from_str("invalid"), None);
        
        assert_eq!(PowerProfile::PowerSave.to_string(), "power_save");
        assert_eq!(PowerProfile::Balanced.to_string(), "balanced");
        assert_eq!(PowerProfile::Performance.to_string(), "performance");
        assert_eq!(PowerProfile::Custom.to_string(), "custom");
    }
    
    #[test]
    fn test_battery_metrics_collector() {
        let collector = BatteryMetricsCollector::new(5);
        
        // Check default state
        assert_eq!(collector.get_battery_state(), BatteryState::Discharging);
        assert_eq!(collector.get_power_profile(), PowerProfile::Balanced);
        
        // Set and get battery state
        collector.set_battery_state(BatteryState::Charging);
        assert_eq!(collector.get_battery_state(), BatteryState::Charging);
        
        // Set and get power profile
        collector.set_power_profile(PowerProfile::Performance);
        assert_eq!(collector.get_power_profile(), PowerProfile::Performance);
        
        // Collect metrics
        let metrics = collector.collect();
        
        // Check basic metrics
        assert!(metrics.iter().any(|m| m.name == "battery.level"));
        assert!(metrics.iter().any(|m| m.name == "battery.voltage"));
        assert!(metrics.iter().any(|m| m.name == "battery.current"));
        assert!(metrics.iter().any(|m| m.name == "battery.temperature"));
        assert!(metrics.iter().any(|m| m.name == "battery.state"));
        
        // Check power consumption metrics
        assert!(metrics.iter().any(|m| m.name == "power.consumption.total"));
        assert!(metrics.iter().any(|m| m.name == "power.consumption.cpu"));
        assert!(metrics.iter().any(|m| m.name == "power.consumption.gpu"));
        
        // Check power profile metric
        let profile_metric = metrics.iter().find(|m| m.name == "power.profile").unwrap();
        assert_eq!(profile_metric.value, MetricValue::String("performance".to_string()));
        
        // Check battery state metric
        let state_metric = metrics.iter().find(|m| m.name == "battery.state").unwrap();
        assert_eq!(state_metric.value, MetricValue::String("charging".to_string()));
        
        // Check current direction based on state
        let current_metric = metrics.iter().find(|m| m.name == "battery.current").unwrap();
        if let MetricValue::Float(current) = current_metric.value {
            assert!(current > 0.0); // Positive when charging
        } else {
            panic!("Expected float value for battery.current");
        }
        
        // Change to discharging and check current direction
        collector.set_battery_state(BatteryState::Discharging);
        let metrics = collector.collect();
        let current_metric = metrics.iter().find(|m| m.name == "battery.current").unwrap();
        if let MetricValue::Float(current) = current_metric.value {
            assert!(current < 0.0); // Negative when discharging
        } else {
            panic!("Expected float value for battery.current");
        }
    }
    
    #[test]
    fn test_power_management_metrics_collector() {
        let collector = PowerManagementMetricsCollector::new(10);
        let metrics = collector.collect();
        
        // Check power management metrics
        assert!(metrics.iter().any(|m| m.name == "power.cpu.governor"));
        assert!(metrics.iter().any(|m| m.name == "power.gpu.state"));
        assert!(metrics.iter().any(|m| m.name == "power.display.brightness"));
        assert!(metrics.iter().any(|m| m.name == "power.thermal.throttling"));
        
        // Check power saving features
        assert!(metrics.iter().any(|m| m.name == "power.saving.wifi"));
        assert!(metrics.iter().any(|m| m.name == "power.saving.bluetooth"));
        assert!(metrics.iter().any(|m| m.name == "power.saving.cpu_cores"));
        assert!(metrics.iter().any(|m| m.name == "power.saving.gpu_cores"));
    }
    
    #[test]
    fn test_power_monitor() {
        let monitor = PowerMonitor::new();
        
        // Check collectors
        assert_eq!(monitor.collectors().len(), 2);
        
        // Set and get battery state
        monitor.set_battery_state(BatteryState::Full);
        assert_eq!(monitor.get_battery_state(), BatteryState::Full);
        
        // Set and get power profile
        monitor.set_power_profile(PowerProfile::PowerSave);
        assert_eq!(monitor.get_power_profile(), PowerProfile::PowerSave);
        
        // Check collector access
        assert_eq!(monitor.battery_collector().get_battery_state(), BatteryState::Full);
        assert_eq!(monitor.battery_collector().get_power_profile(), PowerProfile::PowerSave);
    }
}
