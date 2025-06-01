//! Power management interface for the VR headset.
//!
//! This module provides the implementation of power management devices and
//! the power manager for battery monitoring, charging control, and thermal management.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};

/// Power device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerCapability {
    /// Battery monitoring
    BatteryMonitoring,
    
    /// Charging control
    ChargingControl,
    
    /// Thermal monitoring
    ThermalMonitoring,
    
    /// Power profiles
    PowerProfiles,
    
    /// Power saving mode
    PowerSaving,
    
    /// Performance mode
    PerformanceMode,
    
    /// Voltage regulation
    VoltageRegulation,
    
    /// Current limiting
    CurrentLimiting,
    
    /// Overcurrent protection
    OvercurrentProtection,
    
    /// Overvoltage protection
    OvervoltageProtection,
    
    /// Undervoltage protection
    UndervoltageProtection,
    
    /// Overtemperature protection
    OvertemperatureProtection,
    
    /// Custom capability
    Custom(u32),
}

/// Power profile type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerProfileType {
    /// Balanced profile (default)
    Balanced,
    
    /// Performance profile
    Performance,
    
    /// Power saving profile
    PowerSaving,
    
    /// Ultra power saving profile
    UltraPowerSaving,
    
    /// Custom profile
    Custom(String),
}

/// Battery chemistry type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatteryChemistry {
    /// Lithium-ion
    LithiumIon,
    
    /// Lithium-polymer
    LithiumPolymer,
    
    /// Nickel-metal hydride
    NickelMetalHydride,
    
    /// Nickel-cadmium
    NickelCadmium,
    
    /// Lead-acid
    LeadAcid,
    
    /// Other chemistry
    Other(String),
}

/// Charging state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChargingState {
    /// Not charging
    NotCharging,
    
    /// Charging
    Charging,
    
    /// Fully charged
    FullyCharged,
    
    /// Discharging
    Discharging,
    
    /// Error state
    Error,
    
    /// Unknown state
    Unknown,
}

/// Thermal zone.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThermalZone {
    /// Battery
    Battery,
    
    /// CPU
    CPU,
    
    /// GPU
    GPU,
    
    /// Display
    Display,
    
    /// Charger
    Charger,
    
    /// Ambient
    Ambient,
    
    /// Other zone
    Other(String),
}

/// Thermal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThermalState {
    /// Normal
    Normal,
    
    /// Warning
    Warning,
    
    /// Critical
    Critical,
    
    /// Emergency
    Emergency,
    
    /// Unknown
    Unknown,
}

/// Battery information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatteryInfo {
    /// Battery capacity in mAh
    pub capacity: f32,
    
    /// Battery voltage in V
    pub voltage: f32,
    
    /// Battery chemistry
    pub chemistry: BatteryChemistry,
    
    /// Battery cycle count
    pub cycle_count: u32,
    
    /// Battery manufacture date
    pub manufacture_date: Option<SystemTime>,
    
    /// Battery serial number
    pub serial_number: Option<String>,
    
    /// Battery health percentage (0-100)
    pub health: f32,
    
    /// Battery temperature in Celsius
    pub temperature: f32,
    
    /// Battery design capacity in mAh
    pub design_capacity: f32,
    
    /// Battery full charge capacity in mAh
    pub full_charge_capacity: f32,
}

impl BatteryInfo {
    /// Create a new BatteryInfo.
    pub fn new(
        capacity: f32,
        voltage: f32,
        chemistry: BatteryChemistry,
        cycle_count: u32,
        manufacture_date: Option<SystemTime>,
        serial_number: Option<String>,
        health: f32,
        temperature: f32,
        design_capacity: f32,
        full_charge_capacity: f32,
    ) -> Self {
        Self {
            capacity,
            voltage,
            chemistry,
            cycle_count,
            manufacture_date,
            serial_number,
            health,
            temperature,
            design_capacity,
            full_charge_capacity,
        }
    }
}

/// Battery status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatteryStatus {
    /// Battery level percentage (0-100)
    pub level: f32,
    
    /// Charging state
    pub charging_state: ChargingState,
    
    /// Current in mA (positive for charging, negative for discharging)
    pub current: f32,
    
    /// Power in W (positive for charging, negative for discharging)
    pub power: f32,
    
    /// Time to full charge in seconds (if charging)
    pub time_to_full: Option<u32>,
    
    /// Time to empty in seconds (if discharging)
    pub time_to_empty: Option<u32>,
    
    /// Temperature in Celsius
    pub temperature: f32,
    
    /// Voltage in V
    pub voltage: f32,
    
    /// Timestamp of the status update
    pub timestamp: SystemTime,
}

impl BatteryStatus {
    /// Create a new BatteryStatus.
    pub fn new(
        level: f32,
        charging_state: ChargingState,
        current: f32,
        power: f32,
        time_to_full: Option<u32>,
        time_to_empty: Option<u32>,
        temperature: f32,
        voltage: f32,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            level,
            charging_state,
            current,
            power,
            time_to_full,
            time_to_empty,
            temperature,
            voltage,
            timestamp,
        }
    }
}

/// Thermal status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalStatus {
    /// Thermal zone
    pub zone: ThermalZone,
    
    /// Temperature in Celsius
    pub temperature: f32,
    
    /// Thermal state
    pub state: ThermalState,
    
    /// Throttling level (0.0 - 1.0, where 0.0 is no throttling)
    pub throttling: f32,
    
    /// Fan speed percentage (0-100, if applicable)
    pub fan_speed: Option<f32>,
    
    /// Timestamp of the status update
    pub timestamp: SystemTime,
}

impl ThermalStatus {
    /// Create a new ThermalStatus.
    pub fn new(
        zone: ThermalZone,
        temperature: f32,
        state: ThermalState,
        throttling: f32,
        fan_speed: Option<f32>,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            zone,
            temperature,
            state,
            throttling,
            fan_speed,
            timestamp,
        }
    }
}

/// Power profile configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerProfile {
    /// Profile type
    pub profile_type: PowerProfileType,
    
    /// CPU frequency scaling (0.0 - 1.0)
    pub cpu_scaling: f32,
    
    /// GPU frequency scaling (0.0 - 1.0)
    pub gpu_scaling: f32,
    
    /// Display brightness scaling (0.0 - 1.0)
    pub brightness_scaling: f32,
    
    /// Refresh rate scaling (0.0 - 1.0)
    pub refresh_rate_scaling: f32,
    
    /// Network power saving enabled
    pub network_power_saving: bool,
    
    /// Thermal throttling threshold in Celsius
    pub thermal_throttling_threshold: f32,
    
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl PowerProfile {
    /// Create a new PowerProfile.
    pub fn new(profile_type: PowerProfileType) -> Self {
        match profile_type {
            PowerProfileType::Balanced => Self {
                profile_type,
                cpu_scaling: 0.8,
                gpu_scaling: 0.8,
                brightness_scaling: 0.7,
                refresh_rate_scaling: 0.8,
                network_power_saving: false,
                thermal_throttling_threshold: 75.0,
                custom_settings: HashMap::new(),
            },
            PowerProfileType::Performance => Self {
                profile_type,
                cpu_scaling: 1.0,
                gpu_scaling: 1.0,
                brightness_scaling: 1.0,
                refresh_rate_scaling: 1.0,
                network_power_saving: false,
                thermal_throttling_threshold: 85.0,
                custom_settings: HashMap::new(),
            },
            PowerProfileType::PowerSaving => Self {
                profile_type,
                cpu_scaling: 0.6,
                gpu_scaling: 0.5,
                brightness_scaling: 0.5,
                refresh_rate_scaling: 0.6,
                network_power_saving: true,
                thermal_throttling_threshold: 70.0,
                custom_settings: HashMap::new(),
            },
            PowerProfileType::UltraPowerSaving => Self {
                profile_type,
                cpu_scaling: 0.4,
                gpu_scaling: 0.3,
                brightness_scaling: 0.3,
                refresh_rate_scaling: 0.5,
                network_power_saving: true,
                thermal_throttling_threshold: 65.0,
                custom_settings: HashMap::new(),
            },
            PowerProfileType::Custom(_) => Self {
                profile_type,
                cpu_scaling: 0.8,
                gpu_scaling: 0.8,
                brightness_scaling: 0.7,
                refresh_rate_scaling: 0.8,
                network_power_saving: false,
                thermal_throttling_threshold: 75.0,
                custom_settings: HashMap::new(),
            },
        }
    }
}

/// Power device trait.
pub trait PowerDevice: Device {
    /// Get the battery information.
    fn get_battery_info(&self) -> DeviceResult<BatteryInfo>;
    
    /// Get the battery status.
    fn get_battery_status(&self) -> DeviceResult<BatteryStatus>;
    
    /// Get the thermal status for a specific zone.
    fn get_thermal_status(&self, zone: ThermalZone) -> DeviceResult<ThermalStatus>;
    
    /// Get all thermal statuses.
    fn get_all_thermal_statuses(&self) -> DeviceResult<HashMap<ThermalZone, ThermalStatus>>;
    
    /// Get the current power profile.
    fn get_power_profile(&self) -> DeviceResult<PowerProfile>;
    
    /// Set the power profile.
    fn set_power_profile(&mut self, profile: &PowerProfile) -> DeviceResult<()>;
    
    /// Get the available power profiles.
    fn get_available_power_profiles(&self) -> DeviceResult<Vec<PowerProfileType>>;
    
    /// Enable or disable charging.
    fn set_charging_enabled(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Check if charging is enabled.
    fn is_charging_enabled(&self) -> DeviceResult<bool>;
    
    /// Set the maximum charging current in mA.
    fn set_max_charging_current(&mut self, current_ma: u32) -> DeviceResult<()>;
    
    /// Get the maximum charging current in mA.
    fn get_max_charging_current(&self) -> DeviceResult<u32>;
    
    /// Set the maximum charging voltage in mV.
    fn set_max_charging_voltage(&mut self, voltage_mv: u32) -> DeviceResult<()>;
    
    /// Get the maximum charging voltage in mV.
    fn get_max_charging_voltage(&self) -> DeviceResult<u32>;
    
    /// Set the thermal throttling enabled state.
    fn set_thermal_throttling_enabled(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Check if thermal throttling is enabled.
    fn is_thermal_throttling_enabled(&self) -> DeviceResult<bool>;
    
    /// Set the thermal throttling threshold for a specific zone.
    fn set_thermal_throttling_threshold(&mut self, zone: ThermalZone, threshold_celsius: f32) -> DeviceResult<()>;
    
    /// Get the thermal throttling threshold for a specific zone.
    fn get_thermal_throttling_threshold(&self, zone: ThermalZone) -> DeviceResult<f32>;
    
    /// Get the power consumption in watts.
    fn get_power_consumption(&self) -> DeviceResult<f32>;
    
    /// Get the power consumption breakdown by component.
    fn get_power_consumption_breakdown(&self) -> DeviceResult<HashMap<String, f32>>;
    
    /// Run a battery calibration.
    fn calibrate_battery(&mut self) -> DeviceResult<bool>;
    
    /// Run a thermal sensor calibration.
    fn calibrate_thermal_sensors(&mut self) -> DeviceResult<bool>;
    
    /// Clone the power device.
    fn clone_power_box(&self) -> Box<dyn PowerDevice>;
}

/// Power manager for managing power devices and profiles.
#[derive(Debug)]
pub struct PowerManager {
    /// Power devices by ID
    devices: HashMap<String, Arc<Mutex<Box<dyn PowerDevice>>>>,
    
    /// Primary power device ID
    primary_device_id: Option<String>,
    
    /// Current power profile
    current_profile: PowerProfile,
    
    /// Available power profiles
    available_profiles: HashMap<PowerProfileType, PowerProfile>,
    
    /// Low battery threshold percentage
    low_battery_threshold: f32,
    
    /// Critical battery threshold percentage
    critical_battery_threshold: f32,
    
    /// Thermal throttling enabled
    thermal_throttling_enabled: bool,
    
    /// Thermal throttling thresholds by zone
    thermal_throttling_thresholds: HashMap<ThermalZone, f32>,
    
    /// Battery history (timestamp, level)
    battery_history: Vec<(SystemTime, f32)>,
    
    /// Maximum battery history entries
    max_battery_history: usize,
}

impl PowerManager {
    /// Create a new PowerManager.
    pub fn new() -> Self {
        let mut available_profiles = HashMap::new();
        available_profiles.insert(
            PowerProfileType::Balanced,
            PowerProfile::new(PowerProfileType::Balanced),
        );
        available_profiles.insert(
            PowerProfileType::Performance,
            PowerProfile::new(PowerProfileType::Performance),
        );
        available_profiles.insert(
            PowerProfileType::PowerSaving,
            PowerProfile::new(PowerProfileType::PowerSaving),
        );
        available_profiles.insert(
            PowerProfileType::UltraPowerSaving,
            PowerProfile::new(PowerProfileType::UltraPowerSaving),
        );
        
        let mut thermal_throttling_thresholds = HashMap::new();
        thermal_throttling_thresholds.insert(ThermalZone::CPU, 80.0);
        thermal_throttling_thresholds.insert(ThermalZone::GPU, 75.0);
        thermal_throttling_thresholds.insert(ThermalZone::Battery, 45.0);
        thermal_throttling_thresholds.insert(ThermalZone::Display, 60.0);
        
        Self {
            devices: HashMap::new(),
            primary_device_id: None,
            current_profile: PowerProfile::new(PowerProfileType::Balanced),
            available_profiles,
            low_battery_threshold: 20.0,
            critical_battery_threshold: 10.0,
            thermal_throttling_enabled: true,
            thermal_throttling_thresholds,
            battery_history: Vec::new(),
            max_battery_history: 100,
        }
    }
    
    /// Initialize the power manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing PowerManager");
        Ok(())
    }
    
    /// Shutdown the power manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down PowerManager");
        
        // Shutdown all power devices
        for (id, device) in &self.devices {
            info!("Shutting down power device {}", id);
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
            })?;
            
            if let Err(e) = device.shutdown() {
                warn!("Failed to shutdown power device {}: {}", id, e);
            }
        }
        
        self.devices.clear();
        self.primary_device_id = None;
        
        Ok(())
    }
    
    /// Add a power device.
    pub fn add_device(&mut self, device: Box<dyn PowerDevice>) -> DeviceResult<()> {
        let info = device.info()?;
        let id = info.id.clone();
        
        // Check if this is the first device
        if self.devices.is_empty() {
            self.primary_device_id = Some(id.clone());
        }
        
        self.devices.insert(id, Arc::new(Mutex::new(device)));
        Ok(())
    }
    
    /// Remove a power device.
    pub fn remove_device(&mut self, device_id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!(
                "Power device not found: {}",
                device_id
            )));
        }
        
        // If this is the primary device, clear the primary device ID
        if let Some(primary_id) = &self.primary_device_id {
            if primary_id == device_id {
                self.primary_device_id = None;
                
                // Set a new primary device if there are other devices
                if !self.devices.is_empty() {
                    let next_id = self
                        .devices
                        .keys()
                        .find(|id| *id != device_id)
                        .unwrap()
                        .clone();
                    self.primary_device_id = Some(next_id);
                }
            }
        }
        
        self.devices.remove(device_id);
        Ok(())
    }
    
    /// Get a power device.
    pub fn get_device(&self, device_id: &str) -> DeviceResult<Arc<Mutex<Box<dyn PowerDevice>>>> {
        self.devices
            .get(device_id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Power device not found: {}", device_id)))
    }
    
    /// Get the primary power device.
    pub fn get_primary_device(&self) -> DeviceResult<Arc<Mutex<Box<dyn PowerDevice>>>> {
        if let Some(primary_id) = &self.primary_device_id {
            self.get_device(primary_id)
        } else {
            Err(DeviceError::NotFound(
                "No primary power device set".to_string(),
            ))
        }
    }
    
    /// Set the primary power device.
    pub fn set_primary_device(&mut self, device_id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(device_id) {
            return Err(DeviceError::NotFound(format!(
                "Power device not found: {}",
                device_id
            )));
        }
        
        self.primary_device_id = Some(device_id.to_string());
        Ok(())
    }
    
    /// List all power devices.
    pub fn list_devices(&self) -> DeviceResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        
        for device in self.devices.values() {
            let device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
            })?;
            devices.push(device.info()?);
        }
        
        Ok(devices)
    }
    
    /// Get the battery information from the primary power device.
    pub fn get_battery_info(&self) -> DeviceResult<BatteryInfo> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_battery_info()
    }
    
    /// Get the battery status from the primary power device.
    pub fn get_battery_status(&self) -> DeviceResult<BatteryStatus> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        let status = primary_device.get_battery_status()?;
        
        // Update battery history
        if self.battery_history.len() >= self.max_battery_history {
            self.battery_history.remove(0);
        }
        self.battery_history.push((status.timestamp, status.level));
        
        Ok(status)
    }
    
    /// Get the thermal status for a specific zone from the primary power device.
    pub fn get_thermal_status(&self, zone: ThermalZone) -> DeviceResult<ThermalStatus> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_thermal_status(zone)
    }
    
    /// Get all thermal statuses from the primary power device.
    pub fn get_all_thermal_statuses(&self) -> DeviceResult<HashMap<ThermalZone, ThermalStatus>> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_all_thermal_statuses()
    }
    
    /// Get the current power profile.
    pub fn get_power_profile(&self) -> &PowerProfile {
        &self.current_profile
    }
    
    /// Set the power profile.
    pub fn set_power_profile(&mut self, profile_type: PowerProfileType) -> DeviceResult<()> {
        // Get the profile from available profiles or create a new one
        let profile = match self.available_profiles.get(&profile_type) {
            Some(profile) => profile.clone(),
            None => PowerProfile::new(profile_type),
        };
        
        // Set the profile on all power devices
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
            })?;
            
            if let Err(e) = device.set_power_profile(&profile) {
                warn!("Failed to set power profile on device {}: {}", id, e);
            }
        }
        
        // Update the current profile
        self.current_profile = profile;
        
        Ok(())
    }
    
    /// Get the available power profiles.
    pub fn get_available_power_profiles(&self) -> Vec<PowerProfileType> {
        self.available_profiles.keys().cloned().collect()
    }
    
    /// Add a custom power profile.
    pub fn add_custom_profile(&mut self, profile: PowerProfile) -> DeviceResult<()> {
        self.available_profiles
            .insert(profile.profile_type, profile);
        Ok(())
    }
    
    /// Remove a custom power profile.
    pub fn remove_custom_profile(&mut self, profile_type: PowerProfileType) -> DeviceResult<()> {
        match profile_type {
            PowerProfileType::Balanced
            | PowerProfileType::Performance
            | PowerProfileType::PowerSaving
            | PowerProfileType::UltraPowerSaving => {
                return Err(DeviceError::InvalidOperation(
                    "Cannot remove built-in power profile".to_string(),
                ));
            }
            _ => {
                self.available_profiles.remove(&profile_type);
                Ok(())
            }
        }
    }
    
    /// Enable or disable charging on the primary power device.
    pub fn set_charging_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.set_charging_enabled(enabled)
    }
    
    /// Check if charging is enabled on the primary power device.
    pub fn is_charging_enabled(&self) -> DeviceResult<bool> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.is_charging_enabled()
    }
    
    /// Set the maximum charging current in mA on the primary power device.
    pub fn set_max_charging_current(&mut self, current_ma: u32) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.set_max_charging_current(current_ma)
    }
    
    /// Get the maximum charging current in mA from the primary power device.
    pub fn get_max_charging_current(&self) -> DeviceResult<u32> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_max_charging_current()
    }
    
    /// Set the maximum charging voltage in mV on the primary power device.
    pub fn set_max_charging_voltage(&mut self, voltage_mv: u32) -> DeviceResult<()> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.set_max_charging_voltage(voltage_mv)
    }
    
    /// Get the maximum charging voltage in mV from the primary power device.
    pub fn get_max_charging_voltage(&self) -> DeviceResult<u32> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_max_charging_voltage()
    }
    
    /// Set the thermal throttling enabled state.
    pub fn set_thermal_throttling_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        self.thermal_throttling_enabled = enabled;
        
        // Set on all power devices
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
            })?;
            
            if let Err(e) = device.set_thermal_throttling_enabled(enabled) {
                warn!(
                    "Failed to set thermal throttling enabled on device {}: {}",
                    id, e
                );
            }
        }
        
        Ok(())
    }
    
    /// Check if thermal throttling is enabled.
    pub fn is_thermal_throttling_enabled(&self) -> bool {
        self.thermal_throttling_enabled
    }
    
    /// Set the thermal throttling threshold for a specific zone.
    pub fn set_thermal_throttling_threshold(
        &mut self,
        zone: ThermalZone,
        threshold_celsius: f32,
    ) -> DeviceResult<()> {
        self.thermal_throttling_thresholds
            .insert(zone, threshold_celsius);
        
        // Set on all power devices
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
            })?;
            
            if let Err(e) = device.set_thermal_throttling_threshold(zone, threshold_celsius) {
                warn!(
                    "Failed to set thermal throttling threshold on device {}: {}",
                    id, e
                );
            }
        }
        
        Ok(())
    }
    
    /// Get the thermal throttling threshold for a specific zone.
    pub fn get_thermal_throttling_threshold(&self, zone: ThermalZone) -> Option<f32> {
        self.thermal_throttling_thresholds.get(&zone).copied()
    }
    
    /// Get the power consumption in watts from the primary power device.
    pub fn get_power_consumption(&self) -> DeviceResult<f32> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_power_consumption()
    }
    
    /// Get the power consumption breakdown by component from the primary power device.
    pub fn get_power_consumption_breakdown(&self) -> DeviceResult<HashMap<String, f32>> {
        let primary_device = self.get_primary_device()?;
        let primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.get_power_consumption_breakdown()
    }
    
    /// Run a battery calibration on the primary power device.
    pub fn calibrate_battery(&mut self) -> DeviceResult<bool> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.calibrate_battery()
    }
    
    /// Run a thermal sensor calibration on the primary power device.
    pub fn calibrate_thermal_sensors(&mut self) -> DeviceResult<bool> {
        let primary_device = self.get_primary_device()?;
        let mut primary_device = primary_device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on power device".to_string())
        })?;
        
        primary_device.calibrate_thermal_sensors()
    }
    
    /// Get the low battery threshold percentage.
    pub fn get_low_battery_threshold(&self) -> f32 {
        self.low_battery_threshold
    }
    
    /// Set the low battery threshold percentage.
    pub fn set_low_battery_threshold(&mut self, threshold: f32) {
        self.low_battery_threshold = threshold;
    }
    
    /// Get the critical battery threshold percentage.
    pub fn get_critical_battery_threshold(&self) -> f32 {
        self.critical_battery_threshold
    }
    
    /// Set the critical battery threshold percentage.
    pub fn set_critical_battery_threshold(&mut self, threshold: f32) {
        self.critical_battery_threshold = threshold;
    }
    
    /// Check if the battery is low.
    pub fn is_battery_low(&self) -> DeviceResult<bool> {
        let status = self.get_battery_status()?;
        Ok(status.level <= self.low_battery_threshold)
    }
    
    /// Check if the battery is critically low.
    pub fn is_battery_critical(&self) -> DeviceResult<bool> {
        let status = self.get_battery_status()?;
        Ok(status.level <= self.critical_battery_threshold)
    }
    
    /// Get the battery history.
    pub fn get_battery_history(&self) -> &[(SystemTime, f32)] {
        &self.battery_history
    }
    
    /// Clear the battery history.
    pub fn clear_battery_history(&mut self) {
        self.battery_history.clear();
    }
    
    /// Get the maximum battery history entries.
    pub fn get_max_battery_history(&self) -> usize {
        self.max_battery_history
    }
    
    /// Set the maximum battery history entries.
    pub fn set_max_battery_history(&mut self, max: usize) {
        self.max_battery_history = max;
        
        // Trim the history if needed
        if self.battery_history.len() > max {
            self.battery_history.drain(0..(self.battery_history.len() - max));
        }
    }
    
    /// Get the battery discharge rate in percent per hour.
    pub fn get_battery_discharge_rate(&self) -> Option<f32> {
        if self.battery_history.len() < 2 {
            return None;
        }
        
        let (last_time, last_level) = self.battery_history.last().unwrap();
        let (first_time, first_level) = self.battery_history.first().unwrap();
        
        let time_diff = match last_time.duration_since(*first_time) {
            Ok(duration) => duration,
            Err(_) => return None,
        };
        
        let hours = time_diff.as_secs_f32() / 3600.0;
        if hours < 0.1 {
            return None;
        }
        
        let level_diff = first_level - last_level;
        Some(level_diff / hours)
    }
    
    /// Estimate the remaining battery time in hours.
    pub fn estimate_remaining_battery_time(&self) -> DeviceResult<Option<f32>> {
        let status = self.get_battery_status()?;
        
        if status.charging_state != ChargingState::Discharging {
            return Ok(None);
        }
        
        if let Some(discharge_rate) = self.get_battery_discharge_rate() {
            if discharge_rate <= 0.0 {
                return Ok(None);
            }
            
            Ok(Some(status.level / discharge_rate))
        } else if let Some(time_to_empty) = status.time_to_empty {
            Ok(Some(time_to_empty as f32 / 3600.0))
        } else {
            Ok(None)
        }
    }
}

/// Mock power device for testing.
#[derive(Debug, Clone)]
pub struct MockPowerDevice {
    /// Device info
    pub info: DeviceInfo,
    
    /// Device state
    pub state: DeviceState,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Event handlers
    pub event_handlers: Vec<DeviceEventHandler>,
    
    /// Battery info
    pub battery_info: BatteryInfo,
    
    /// Battery status
    pub battery_status: BatteryStatus,
    
    /// Thermal statuses
    pub thermal_statuses: HashMap<ThermalZone, ThermalStatus>,
    
    /// Current power profile
    pub power_profile: PowerProfile,
    
    /// Available power profiles
    pub available_profiles: Vec<PowerProfileType>,
    
    /// Charging enabled
    pub charging_enabled: bool,
    
    /// Maximum charging current in mA
    pub max_charging_current: u32,
    
    /// Maximum charging voltage in mV
    pub max_charging_voltage: u32,
    
    /// Thermal throttling enabled
    pub thermal_throttling_enabled: bool,
    
    /// Thermal throttling thresholds
    pub thermal_throttling_thresholds: HashMap<ThermalZone, f32>,
    
    /// Power consumption in watts
    pub power_consumption: f32,
    
    /// Power consumption breakdown
    pub power_consumption_breakdown: HashMap<String, f32>,
}

impl MockPowerDevice {
    /// Create a new MockPowerDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let mut info = DeviceInfo::new(
            id,
            name,
            DeviceType::Power,
            manufacturer,
            model,
            DeviceBus::Virtual,
        );
        
        info.state = DeviceState::Connected;
        
        let now = SystemTime::now();
        
        let battery_info = BatteryInfo::new(
            5000.0,
            3.7,
            BatteryChemistry::LithiumPolymer,
            0,
            None,
            None,
            100.0,
            25.0,
            5000.0,
            5000.0,
        );
        
        let battery_status = BatteryStatus::new(
            100.0,
            ChargingState::FullyCharged,
            0.0,
            0.0,
            None,
            Some(18000),
            25.0,
            4.2,
            now,
        );
        
        let mut thermal_statuses = HashMap::new();
        thermal_statuses.insert(
            ThermalZone::Battery,
            ThermalStatus::new(ThermalZone::Battery, 25.0, ThermalState::Normal, 0.0, None, now),
        );
        thermal_statuses.insert(
            ThermalZone::CPU,
            ThermalStatus::new(ThermalZone::CPU, 40.0, ThermalState::Normal, 0.0, None, now),
        );
        thermal_statuses.insert(
            ThermalZone::GPU,
            ThermalStatus::new(ThermalZone::GPU, 45.0, ThermalState::Normal, 0.0, None, now),
        );
        
        let power_profile = PowerProfile::new(PowerProfileType::Balanced);
        
        let available_profiles = vec![
            PowerProfileType::Balanced,
            PowerProfileType::Performance,
            PowerProfileType::PowerSaving,
            PowerProfileType::UltraPowerSaving,
        ];
        
        let mut thermal_throttling_thresholds = HashMap::new();
        thermal_throttling_thresholds.insert(ThermalZone::CPU, 80.0);
        thermal_throttling_thresholds.insert(ThermalZone::GPU, 75.0);
        thermal_throttling_thresholds.insert(ThermalZone::Battery, 45.0);
        
        let mut power_consumption_breakdown = HashMap::new();
        power_consumption_breakdown.insert("CPU".to_string(), 1.0);
        power_consumption_breakdown.insert("GPU".to_string(), 1.5);
        power_consumption_breakdown.insert("Display".to_string(), 0.8);
        power_consumption_breakdown.insert("Sensors".to_string(), 0.2);
        power_consumption_breakdown.insert("Other".to_string(), 0.5);
        
        Self {
            info,
            state: DeviceState::Connected,
            properties: HashMap::new(),
            event_handlers: Vec::new(),
            battery_info,
            battery_status,
            thermal_statuses,
            power_profile,
            available_profiles,
            charging_enabled: true,
            max_charging_current: 2000,
            max_charging_voltage: 5000,
            thermal_throttling_enabled: true,
            thermal_throttling_thresholds,
            power_consumption: 4.0,
            power_consumption_breakdown,
        }
    }
    
    /// Fire an event.
    pub fn fire_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for MockPowerDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        self.fire_event(DeviceEventType::Initialized);
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::ShuttingDown;
        self.info.state = DeviceState::ShuttingDown;
        self.fire_event(DeviceEventType::Shutdown);
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        self.state = DeviceState::Initializing;
        self.info.state = DeviceState::Initializing;
        self.fire_event(DeviceEventType::Reset);
        self.state = DeviceState::Ready;
        self.info.state = DeviceState::Ready;
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous = self.state;
        self.state = state;
        self.info.state = state;
        self.fire_event(DeviceEventType::StateChanged {
            previous,
            current: state,
        });
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.has_capability(capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.properties.get(key).cloned();
        self.properties.insert(key.to_string(), value.to_string());
        self.fire_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl PowerDevice for MockPowerDevice {
    fn get_battery_info(&self) -> DeviceResult<BatteryInfo> {
        Ok(self.battery_info.clone())
    }
    
    fn get_battery_status(&self) -> DeviceResult<BatteryStatus> {
        Ok(self.battery_status.clone())
    }
    
    fn get_thermal_status(&self, zone: ThermalZone) -> DeviceResult<ThermalStatus> {
        self.thermal_statuses
            .get(&zone)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Thermal zone not found: {:?}", zone)))
    }
    
    fn get_all_thermal_statuses(&self) -> DeviceResult<HashMap<ThermalZone, ThermalStatus>> {
        Ok(self.thermal_statuses.clone())
    }
    
    fn get_power_profile(&self) -> DeviceResult<PowerProfile> {
        Ok(self.power_profile.clone())
    }
    
    fn set_power_profile(&mut self, profile: &PowerProfile) -> DeviceResult<()> {
        self.power_profile = profile.clone();
        Ok(())
    }
    
    fn get_available_power_profiles(&self) -> DeviceResult<Vec<PowerProfileType>> {
        Ok(self.available_profiles.clone())
    }
    
    fn set_charging_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        self.charging_enabled = enabled;
        Ok(())
    }
    
    fn is_charging_enabled(&self) -> DeviceResult<bool> {
        Ok(self.charging_enabled)
    }
    
    fn set_max_charging_current(&mut self, current_ma: u32) -> DeviceResult<()> {
        self.max_charging_current = current_ma;
        Ok(())
    }
    
    fn get_max_charging_current(&self) -> DeviceResult<u32> {
        Ok(self.max_charging_current)
    }
    
    fn set_max_charging_voltage(&mut self, voltage_mv: u32) -> DeviceResult<()> {
        self.max_charging_voltage = voltage_mv;
        Ok(())
    }
    
    fn get_max_charging_voltage(&self) -> DeviceResult<u32> {
        Ok(self.max_charging_voltage)
    }
    
    fn set_thermal_throttling_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        self.thermal_throttling_enabled = enabled;
        Ok(())
    }
    
    fn is_thermal_throttling_enabled(&self) -> DeviceResult<bool> {
        Ok(self.thermal_throttling_enabled)
    }
    
    fn set_thermal_throttling_threshold(&mut self, zone: ThermalZone, threshold_celsius: f32) -> DeviceResult<()> {
        self.thermal_throttling_thresholds.insert(zone, threshold_celsius);
        Ok(())
    }
    
    fn get_thermal_throttling_threshold(&self, zone: ThermalZone) -> DeviceResult<f32> {
        self.thermal_throttling_thresholds
            .get(&zone)
            .copied()
            .ok_or_else(|| DeviceError::NotFound(format!("Thermal zone not found: {:?}", zone)))
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn get_power_consumption_breakdown(&self) -> DeviceResult<HashMap<String, f32>> {
        Ok(self.power_consumption_breakdown.clone())
    }
    
    fn calibrate_battery(&mut self) -> DeviceResult<bool> {
        Ok(true)
    }
    
    fn calibrate_thermal_sensors(&mut self) -> DeviceResult<bool> {
        Ok(true)
    }
    
    fn clone_power_box(&self) -> Box<dyn PowerDevice> {
        Box::new(self.clone())
    }
}
