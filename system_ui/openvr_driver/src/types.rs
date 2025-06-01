//! Common types used throughout the OpenVR driver

use serde::{Serialize, Deserialize};

/// OpenVR device class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    /// Head-mounted display
    HMD = 1,
    /// Controller
    Controller = 2,
    /// Generic tracker
    GenericTracker = 3,
    /// Tracking reference (base station)
    TrackingReference = 4,
}

/// Internal device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Head-mounted display
    HMD,
    /// Controller
    Controller,
    /// Generic tracker
    Tracker,
    /// Tracking reference (base station)
    TrackingReference,
}

/// Controller handedness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Handedness {
    /// Left hand
    Left,
    /// Right hand
    Right,
    /// Neither hand (e.g., tracker)
    None,
}

/// Pose data for a tracked device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    /// Device is connected
    pub device_is_connected: bool,
    /// Pose is valid
    pub pose_is_valid: bool,
    /// Device is tracking
    pub device_is_tracking: bool,
    /// Position [x, y, z] in meters
    pub position: [f32; 3],
    /// Rotation as quaternion [x, y, z, w]
    pub rotation: [f32; 4],
    /// Linear velocity [x, y, z] in meters per second
    pub velocity: [f32; 3],
    /// Angular velocity [x, y, z] in radians per second
    pub angular_velocity: [f32; 3],
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            device_is_connected: false,
            pose_is_valid: false,
            device_is_tracking: false,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            velocity: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
        }
    }
}

/// Button state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonState {
    /// Button is pressed
    pub pressed: bool,
    /// Button is touched
    pub touched: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            pressed: false,
            touched: false,
        }
    }
}

/// Button identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Button {
    /// Button ID
    pub id: u32,
    /// Component handle for OpenVR
    pub component_handle: u64,
}

/// Axis identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Axis {
    /// Axis ID
    pub id: u32,
    /// X component handle for OpenVR
    pub x_handle: u64,
    /// Y component handle for OpenVR (if applicable)
    pub y_handle: u64,
    /// Whether this axis has a Y component
    pub has_y: bool,
}

/// Driver settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverSettings {
    /// Render width in pixels
    pub render_width: i32,
    /// Render height in pixels
    pub render_height: i32,
    /// Refresh rate in Hz
    pub refresh_rate: f32,
    /// Interpupillary distance in meters
    pub ipd: f32,
    /// Prediction time in milliseconds
    pub prediction_time_ms: f32,
}

impl Default for DriverSettings {
    fn default() -> Self {
        Self {
            render_width: 1600,
            render_height: 1600,
            refresh_rate: 90.0,
            ipd: 0.063, // 63mm default
            prediction_time_ms: 30.0,
        }
    }
}
