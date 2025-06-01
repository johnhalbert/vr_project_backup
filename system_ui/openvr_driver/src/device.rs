//! Device interfaces and implementations for OpenVR

use std::sync::atomic::{AtomicU32, Ordering};
use crate::types::{DeviceClass, DeviceType, Pose};
use crate::error::Result;

/// Interface for VR devices
pub trait VRDevice: Send + Sync {
    /// Get the device serial number
    fn get_serial(&self) -> &str;
    
    /// Get the device class for OpenVR
    fn get_device_class(&self) -> DeviceClass;
    
    /// Get the internal device type
    fn get_device_type(&self) -> DeviceType;
    
    /// Update the device state
    fn update(&mut self) -> Result<()>;
    
    /// Get the current device pose
    fn get_pose(&self) -> Pose;
    
    /// Get the OpenVR device index
    fn get_device_index(&self) -> u32 {
        self.get_device_index_atomic().load(Ordering::Relaxed)
    }
    
    /// Set the OpenVR device index
    fn set_device_index(&self, index: u32) {
        self.get_device_index_atomic().store(index, Ordering::Relaxed);
    }
    
    /// Get atomic reference to device index
    fn get_device_index_atomic(&self) -> &AtomicU32;
    
    /// Activate the device
    fn activate(&mut self, device_index: u32) -> Result<()>;
    
    /// Deactivate the device
    fn deactivate(&mut self) -> Result<()>;
    
    /// Enter standby mode
    fn enter_standby(&mut self) -> Result<()>;
    
    /// Process a property read request
    fn get_property(&self, prop: u32) -> Result<serde_json::Value>;
    
    /// Process a property write request
    fn set_property(&mut self, prop: u32, value: serde_json::Value) -> Result<()>;
}

/// Base implementation for common device functionality
pub struct BaseDevice {
    /// Device serial number
    serial: String,
    
    /// Device type
    device_type: DeviceType,
    
    /// OpenVR device index
    device_index: AtomicU32,
    
    /// Current pose
    pose: Pose,
}

impl BaseDevice {
    /// Create a new base device
    pub fn new(serial: &str, device_type: DeviceType) -> Self {
        Self {
            serial: serial.to_string(),
            device_type,
            device_index: AtomicU32::new(u32::MAX), // Invalid index
            pose: Pose::default(),
        }
    }
    
    /// Get the device serial number
    pub fn serial(&self) -> &str {
        &self.serial
    }
    
    /// Get the device type
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
    
    /// Get the device index atomic reference
    pub fn device_index_atomic(&self) -> &AtomicU32 {
        &self.device_index
    }
    
    /// Get the current pose
    pub fn pose(&self) -> &Pose {
        &self.pose
    }
    
    /// Set the current pose
    pub fn set_pose(&mut self, pose: Pose) {
        self.pose = pose;
    }
    
    /// Map device type to OpenVR device class
    pub fn map_device_type_to_class(device_type: DeviceType) -> DeviceClass {
        match device_type {
            DeviceType::HMD => DeviceClass::HMD,
            DeviceType::Controller => DeviceClass::Controller,
            DeviceType::Tracker => DeviceClass::GenericTracker,
            DeviceType::TrackingReference => DeviceClass::TrackingReference,
        }
    }
}
