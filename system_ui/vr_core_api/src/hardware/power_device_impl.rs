//! Power device implementation for the Hardware Access API.
//!
//! This module provides concrete implementations of power devices for the VR headset,
//! including battery, power management, and charging control.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};
use super::power::{
    BatteryDevice, BatteryInfo, BatteryState, ChargingState, PowerCapability, PowerConfig,
    PowerDevice, PowerMode, PowerProfile, PowerState, ThermalZone,
};

/// VR Battery device implementation.
#[derive(Debug)]
pub struct VRBatteryDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Power configuration
    config: PowerConfig,
    
    /// Battery information
    battery_info: BatteryInfo,
    
    /// Battery state
    battery_state: BatteryState,
    
    /// Charging state
    charging_state: ChargingState,
    
    /// Power state
    power_state: PowerState,
    
    /// Thermal zones
    thermal_zones: HashMap<String, ThermalZone>,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRBatteryDevice {
    /// Create a new VRBatteryDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Power,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::I2C,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::ThermalManagement,
            ],
            state: DeviceState::Connected,
            description: Some("VR Battery".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add power-specific properties
        info.properties.insert("power_type".to_string(), "battery".to_string());
        
        // Create battery information
        let battery_info = BatteryInfo {
            capacity_mah: 5000,
            design_capacity_mah: 5000,
            voltage_mv: 3700,
            chemistry: "Li-ion".to_string(),
            cycle_count: 0,
            manufacture_date: chrono::Utc::now() - chrono::Duration::days(30), // 30 days old
            serial_number: "BAT12345".to_string(),
            temperature_support: true,
            health_support: true,
        };
        
        // Create battery state
        let battery_state = BatteryState {
            level: 1.0, // 100%
            voltage_mv: 3700,
            current_ma: 0,
            temperature_c: 25.0,
            health: 1.0, // 100%
            time_to_empty_mins: None,
            time_to_full_mins: None,
            is_present: true,
        };
        
        // Create charging state
        let charging_state = ChargingState {
            is_charging: false,
            charger_type: None,
            max_charging_current_ma: 2000,
            max_charging_voltage_mv: 5000,
            charging_enabled: true,
            fast_charging_enabled: true,
        };
        
        // Create power state
        let power_state = PowerState {
            mode: PowerMode::Normal,
            profile: PowerProfile::Balanced,
            system_power_mw: 2500,
            cpu_power_mw: 1000,
            gpu_power_mw: 1000,
            display_power_mw: 500,
            other_power_mw: 0,
        };
        
        // Create thermal zones
        let mut thermal_zones = HashMap::new();
        thermal_zones.insert("cpu".to_string(), ThermalZone {
            name: "CPU".to_string(),
            temperature_c: 35.0,
            critical_temperature_c: 80.0,
            throttling_temperature_c: 70.0,
            fan_speed_rpm: None,
        });
        thermal_zones.insert("battery".to_string(), ThermalZone {
            name: "Battery".to_string(),
            temperature_c: 25.0,
            critical_temperature_c: 60.0,
            throttling_temperature_c: 45.0,
            fan_speed_rpm: None,
        });
        thermal_zones.insert("gpu".to_string(), ThermalZone {
            name: "GPU".to_string(),
            temperature_c: 40.0,
            critical_temperature_c: 85.0,
            throttling_temperature_c: 75.0,
            fan_speed_rpm: None,
        });
        
        // Create power configuration
        let config = PowerConfig::vr_optimized();
        
        Self {
            info,
            config,
            battery_info,
            battery_state,
            charging_state,
            power_state,
            thermal_zones,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate battery drain and temperature changes based on time and power state
        let elapsed = self.last_update.elapsed().as_secs_f32();
        
        // Calculate power consumption based on power mode and profile
        let power_factor = match self.power_state.mode {
            PowerMode::Normal => 1.0,
            PowerMode::LowPower => 0.6,
            PowerMode::UltraLowPower => 0.3,
            PowerMode::Performance => 1.5,
            PowerMode::Custom => 1.0,
        };
        
        let profile_factor = match self.power_state.profile {
            PowerProfile::PowerSaver => 0.7,
            PowerProfile::Balanced => 1.0,
            PowerProfile::Performance => 1.3,
            PowerProfile::Gaming => 1.5,
            PowerProfile::Custom => 1.0,
        };
        
        // Calculate battery drain rate (percent per second)
        let drain_rate = if self.charging_state.is_charging {
            // Charging
            -0.0003 * power_factor // Gain about 0.03% per second when charging
        } else {
            // Discharging
            0.0001 * power_factor * profile_factor // Lose about 0.01% per second when idle
        };
        
        // Update battery level
        self.battery_state.level -= drain_rate * elapsed;
        self.battery_state.level = self.battery_state.level.clamp(0.0, 1.0);
        
        // Update time estimates
        if self.charging_state.is_charging {
            // Time to full
            let remaining_capacity = (1.0 - self.battery_state.level) * self.battery_info.capacity_mah as f32;
            let charging_rate = 2000.0; // mA
            self.battery_state.time_to_full_mins = Some((remaining_capacity / charging_rate * 60.0) as u32);
            self.battery_state.time_to_empty_mins = None;
        } else {
            // Time to empty
            let remaining_capacity = self.battery_state.level * self.battery_info.capacity_mah as f32;
            let discharge_rate = self.power_state.system_power_mw as f32 / self.battery_state.voltage_mv as f32;
            self.battery_state.time_to_empty_mins = Some((remaining_capacity / discharge_rate * 60.0) as u32);
            self.battery_state.time_to_full_mins = None;
        }
        
        // Update current
        if self.charging_state.is_charging {
            self.battery_state.current_ma = 2000; // Charging at 2A
        } else {
            // Calculate discharge current based on power consumption
            self.battery_state.current_ma = -(self.power_state.system_power_mw as i32) / (self.battery_state.voltage_mv / 1000);
        }
        
        // Update temperatures
        for (_, zone) in self.thermal_zones.iter_mut() {
            // Temperature increases with power consumption and time
            let temp_increase = elapsed * power_factor * profile_factor * 0.05;
            // Temperature decreases over time (cooling)
            let temp_decrease = elapsed * 0.01;
            
            zone.temperature_c += temp_increase - temp_decrease;
            zone.temperature_c = zone.temperature_c.clamp(20.0, zone.critical_temperature_c);
        }
        
        // Update battery temperature based on charging state and power consumption
        let battery_zone = self.thermal_zones.get_mut("battery").unwrap();
        if self.charging_state.is_charging {
            battery_zone.temperature_c += elapsed * 0.02; // Additional heating when charging
        }
        self.battery_state.temperature_c = battery_zone.temperature_c;
        
        // Update system power consumption
        self.power_state.system_power_mw = (2500.0 * power_factor * profile_factor) as u32;
        self.power_state.cpu_power_mw = (1000.0 * power_factor * profile_factor) as u32;
        self.power_state.gpu_power_mw = (1000.0 * power_factor * profile_factor) as u32;
        self.power_state.display_power_mw = (500.0 * power_factor) as u32;
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRBatteryDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Battery: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Battery: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Battery: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = PowerConfig::vr_optimized();
        
        // Reset power state
        self.power_state.mode = PowerMode::Normal;
        self.power_state.profile = PowerProfile::Balanced;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
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
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            battery_info: self.battery_info.clone(),
            battery_state: self.battery_state.clone(),
            charging_state: self.charging_state.clone(),
            power_state: self.power_state.clone(),
            thermal_zones: self.thermal_zones.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl PowerDevice for VRBatteryDevice {
    fn get_power_config(&self) -> DeviceResult<PowerConfig> {
        Ok(self.config.clone())
    }
    
    fn set_power_config(&mut self, config: &PowerConfig) -> DeviceResult<()> {
        // Apply configuration
        self.config = config.clone();
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "PowerConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("auto_power_saving".to_string(), config.auto_power_saving.to_string());
                data.insert("low_battery_threshold".to_string(), config.low_battery_threshold.to_string());
                data.insert("critical_battery_threshold".to_string(), config.critical_battery_threshold.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_power_state(&mut self) -> DeviceResult<PowerState> {
        // Update status
        self.update_status();
        
        Ok(self.power_state.clone())
    }
    
    fn set_power_mode(&mut self, mode: PowerMode) -> DeviceResult<()> {
        // Update power state
        self.power_state.mode = mode;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "PowerModeChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("mode".to_string(), format!("{:?}", mode));
                data
            },
        });
        
        Ok(())
    }
    
    fn set_power_profile(&mut self, profile: PowerProfile) -> DeviceResult<()> {
        // Update power state
        self.power_state.profile = profile;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "PowerProfileChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("profile".to_string(), format!("{:?}", profile));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_thermal_zones(&mut self) -> DeviceResult<HashMap<String, ThermalZone>> {
        // Update status
        self.update_status();
        
        Ok(self.thermal_zones.clone())
    }
    
    fn get_thermal_zone(&mut self, zone_id: &str) -> DeviceResult<ThermalZone> {
        // Update status
        self.update_status();
        
        match self.thermal_zones.get(zone_id) {
            Some(zone) => Ok(zone.clone()),
            None => Err(DeviceError::NotFound(format!("Thermal zone {} not found", zone_id))),
        }
    }
    
    fn has_power_capability(&self, capability: PowerCapability) -> DeviceResult<bool> {
        match capability {
            PowerCapability::Battery => Ok(true),
            PowerCapability::Charging => Ok(true),
            PowerCapability::PowerModes => Ok(true),
            PowerCapability::PowerProfiles => Ok(true),
            PowerCapability::ThermalManagement => Ok(true),
            PowerCapability::PowerStatistics => Ok(true),
        }
    }
    
    fn clone_power_box(&self) -> Box<dyn PowerDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            battery_info: self.battery_info.clone(),
            battery_state: self.battery_state.clone(),
            charging_state: self.charging_state.clone(),
            power_state: self.power_state.clone(),
            thermal_zones: self.thermal_zones.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl BatteryDevice for VRBatteryDevice {
    fn get_battery_info(&self) -> DeviceResult<BatteryInfo> {
        Ok(self.battery_info.clone())
    }
    
    fn get_battery_state(&mut self) -> DeviceResult<BatteryState> {
        // Update status
        self.update_status();
        
        Ok(self.battery_state.clone())
    }
    
    fn get_charging_state(&mut self) -> DeviceResult<ChargingState> {
        // Update status
        self.update_status();
        
        Ok(self.charging_state.clone())
    }
    
    fn set_charging_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update charging state
        self.charging_state.charging_enabled = enabled;
        
        // If charging is disabled, also stop charging
        if !enabled && self.charging_state.is_charging {
            self.charging_state.is_charging = false;
        }
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ChargingEnabledChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_fast_charging_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update charging state
        self.charging_state.fast_charging_enabled = enabled;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "FastChargingEnabledChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn start_charging(&mut self) -> DeviceResult<()> {
        // Check if charging is enabled
        if !self.charging_state.charging_enabled {
            return Err(DeviceError::InvalidState("Charging is disabled".to_string()));
        }
        
        // Check if already charging
        if self.charging_state.is_charging {
            return Ok(());
        }
        
        // Update charging state
        self.charging_state.is_charging = true;
        self.charging_state.charger_type = Some("USB-C".to_string());
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ChargingStarted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("charger_type".to_string(), "USB-C".to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn stop_charging(&mut self) -> DeviceResult<()> {
        // Check if already not charging
        if !self.charging_state.is_charging {
            return Ok(());
        }
        
        // Update charging state
        self.charging_state.is_charging = false;
        self.charging_state.charger_type = None;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ChargingStopped".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn is_charging(&self) -> DeviceResult<bool> {
        Ok(self.charging_state.is_charging)
    }
    
    fn get_battery_level(&self) -> DeviceResult<f32> {
        Ok(self.battery_state.level)
    }
    
    fn get_battery_temperature(&self) -> DeviceResult<f32> {
        Ok(self.battery_state.temperature_c)
    }
    
    fn get_time_to_empty(&self) -> DeviceResult<Option<u32>> {
        Ok(self.battery_state.time_to_empty_mins)
    }
    
    fn get_time_to_full(&self) -> DeviceResult<Option<u32>> {
        Ok(self.battery_state.time_to_full_mins)
    }
    
    fn clone_battery_box(&self) -> Box<dyn BatteryDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            battery_info: self.battery_info.clone(),
            battery_state: self.battery_state.clone(),
            charging_state: self.charging_state.clone(),
            power_state: self.power_state.clone(),
            thermal_zones: self.thermal_zones.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_battery_creation() {
        let battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        let info = battery.info().unwrap();
        assert_eq!(info.id, "bat1");
        assert_eq!(info.name, "VR Battery");
        assert_eq!(info.device_type, DeviceType::Power);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "BAT-VR-5000");
        assert_eq!(info.bus_type, DeviceBus::I2C);
        assert_eq!(info.state, DeviceState::Connected);
        
        let battery_info = battery.get_battery_info().unwrap();
        assert_eq!(battery_info.capacity_mah, 5000);
        assert_eq!(battery_info.voltage_mv, 3700);
        assert_eq!(battery_info.chemistry, "Li-ion");
    }
    
    #[test]
    fn test_battery_state() {
        let mut battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        let state = battery.get_battery_state().unwrap();
        assert_eq!(state.level, 1.0); // 100%
        assert_eq!(state.voltage_mv, 3700);
        assert!(state.is_present);
        
        let charging_state = battery.get_charging_state().unwrap();
        assert_eq!(charging_state.is_charging, false);
        assert_eq!(charging_state.charging_enabled, true);
    }
    
    #[test]
    fn test_power_modes() {
        let mut battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        // Test power mode
        battery.set_power_mode(PowerMode::LowPower).unwrap();
        let power_state = battery.get_power_state().unwrap();
        assert_eq!(power_state.mode, PowerMode::LowPower);
        
        // Test power profile
        battery.set_power_profile(PowerProfile::PowerSaver).unwrap();
        let power_state = battery.get_power_state().unwrap();
        assert_eq!(power_state.profile, PowerProfile::PowerSaver);
    }
    
    #[test]
    fn test_charging() {
        let mut battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        // Test starting charging
        assert_eq!(battery.is_charging().unwrap(), false);
        battery.start_charging().unwrap();
        assert_eq!(battery.is_charging().unwrap(), true);
        
        // Test stopping charging
        battery.stop_charging().unwrap();
        assert_eq!(battery.is_charging().unwrap(), false);
        
        // Test disabling charging
        battery.set_charging_enabled(false).unwrap();
        assert!(battery.start_charging().is_err());
    }
    
    #[test]
    fn test_thermal_zones() {
        let mut battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        let zones = battery.get_thermal_zones().unwrap();
        assert!(zones.contains_key("cpu"));
        assert!(zones.contains_key("battery"));
        assert!(zones.contains_key("gpu"));
        
        let cpu_zone = battery.get_thermal_zone("cpu").unwrap();
        assert_eq!(cpu_zone.name, "CPU");
        assert!(cpu_zone.temperature_c > 0.0);
        
        // Test non-existent zone
        assert!(battery.get_thermal_zone("nonexistent").is_err());
    }
    
    #[test]
    fn test_battery_drain() {
        let mut battery = VRBatteryDevice::new(
            "bat1".to_string(),
            "VR Battery".to_string(),
            "Test Manufacturer".to_string(),
            "BAT-VR-5000".to_string(),
        );
        
        // Get initial battery level
        let initial_level = battery.get_battery_level().unwrap();
        
        // Set to performance mode to drain faster
        battery.set_power_mode(PowerMode::Performance).unwrap();
        battery.set_power_profile(PowerProfile::Gaming).unwrap();
        
        // Wait a bit
        std::thread::sleep(Duration::from_secs(1));
        
        // Get new battery level
        let new_level = battery.get_battery_level().unwrap();
        
        // Battery should have drained
        assert!(new_level < initial_level);
    }
}
