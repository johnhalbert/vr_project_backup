//! Tracking system interface for the VR headset.
//!
//! This module provides the implementation of tracking devices (IMU, cameras)
//! and the tracking manager for sensor fusion and pose estimation.

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

/// Tracking device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackingCapability {
    /// 6DoF tracking (position + orientation)
    SixDof,
    
    /// 3DoF tracking (orientation only)
    ThreeDof,
    
    /// Inside-out tracking
    InsideOut,
    
    /// Outside-in tracking
    OutsideIn,
    
    /// Room-scale tracking
    RoomScale,
    
    /// Hand tracking
    HandTracking,
    
    /// Eye tracking
    EyeTracking,
    
    /// Face tracking
    FaceTracking,
    
    /// Body tracking
    BodyTracking,
    
    /// Sensor fusion
    SensorFusion,
    
    /// Predictive tracking
    PredictiveTracking,
    
    /// Drift correction
    DriftCorrection,
    
    /// Boundary system
    BoundarySystem,
    
    /// Custom capability
    Custom(u32),
}

/// 3D Vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Create a new Vector3.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    
    /// Zero vector.
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    /// Calculate the magnitude (length) of the vector.
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
    
    /// Normalize the vector.
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            Self::zero()
        } else {
            Self { x: self.x / mag, y: self.y / mag, z: self.z / mag }
        }
    }
}

/// Quaternion for representing orientation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    /// Create a new Quaternion.
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
    
    /// Identity quaternion.
    pub fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
    
    /// Normalize the quaternion.
    pub fn normalize(&self) -> Self {
        let mag = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt();
        if mag == 0.0 {
            Self::identity()
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
                w: self.w / mag,
            }
        }
    }
}

/// Pose representing position and orientation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    /// Position
    pub position: Vector3,
    
    /// Orientation
    pub orientation: Quaternion,
    
    /// Timestamp of the pose data
    pub timestamp: SystemTime,
    
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    
    /// Tracking state
    pub tracking_state: TrackingState,
}

impl Pose {
    /// Create a new Pose.
    pub fn new(
        position: Vector3,
        orientation: Quaternion,
        timestamp: SystemTime,
        confidence: f32,
        tracking_state: TrackingState,
    ) -> Self {
        Self {
            position,
            orientation,
            timestamp,
            confidence,
            tracking_state,
        }
    }
    
    /// Identity pose at the origin.
    pub fn identity() -> Self {
        Self {
            position: Vector3::zero(),
            orientation: Quaternion::identity(),
            timestamp: SystemTime::now(),
            confidence: 1.0,
            tracking_state: TrackingState::Tracking,
        }
    }
}

/// Tracking state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackingState {
    /// Tracking is active and accurate
    Tracking,
    
    /// Tracking is active but with limited accuracy
    Limited,
    
    /// Tracking is lost
    Lost,
    
    /// Tracking is initializing
    Initializing,
    
    /// Tracking is calibrating
    Calibrating,
    
    /// Tracking is inactive
    Inactive,
}

/// IMU data sample.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IMUData {
    /// Acceleration vector (m/s^2)
    pub acceleration: Vector3,
    
    /// Angular velocity vector (rad/s)
    pub angular_velocity: Vector3,
    
    /// Magnetic field vector (microtesla)
    pub magnetic_field: Option<Vector3>,
    
    /// Temperature (Celsius)
    pub temperature: Option<f32>,
    
    /// Timestamp of the data sample
    pub timestamp: SystemTime,
}

impl IMUData {
    /// Create a new IMUData sample.
    pub fn new(
        acceleration: Vector3,
        angular_velocity: Vector3,
        magnetic_field: Option<Vector3>,
        temperature: Option<f32>,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            acceleration,
            angular_velocity,
            magnetic_field,
            temperature,
            timestamp,
        }
    }
}

/// Camera frame data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraFrame {
    /// Frame width in pixels
    pub width: u32,
    
    /// Frame height in pixels
    pub height: u32,
    
    /// Frame format (e.g., "RGB", "Grayscale", "YUV")
    pub format: String,
    
    /// Frame data buffer
    pub data: Vec<u8>,
    
    /// Timestamp of the frame capture
    pub timestamp: SystemTime,
    
    /// Frame sequence number
    pub sequence_number: u64,
    
    /// Camera intrinsics (optional)
    pub intrinsics: Option<CameraIntrinsics>,
    
    /// Camera extrinsics (optional)
    pub extrinsics: Option<Pose>,
}

impl CameraFrame {
    /// Create a new CameraFrame.
    pub fn new(
        width: u32,
        height: u32,
        format: String,
        data: Vec<u8>,
        timestamp: SystemTime,
        sequence_number: u64,
        intrinsics: Option<CameraIntrinsics>,
        extrinsics: Option<Pose>,
    ) -> Self {
        Self {
            width,
            height,
            format,
            data,
            timestamp,
            sequence_number,
            intrinsics,
            extrinsics,
        }
    }
}

/// Camera intrinsics parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraIntrinsics {
    /// Focal length x
    pub fx: f64,
    
    /// Focal length y
    pub fy: f64,
    
    /// Principal point x
    pub cx: f64,
    
    /// Principal point y
    pub cy: f64,
    
    /// Distortion coefficients (k1, k2, p1, p2, k3)
    pub distortion: [f64; 5],
}

impl CameraIntrinsics {
    /// Create new CameraIntrinsics.
    pub fn new(fx: f64, fy: f64, cx: f64, cy: f64, distortion: [f64; 5]) -> Self {
        Self { fx, fy, cx, cy, distortion }
    }
}

/// Tracking device trait.
pub trait TrackingDevice: Device {
    /// Get the latest pose.
    fn get_pose(&self) -> DeviceResult<Pose>;
    
    /// Get the latest IMU data.
    fn get_imu_data(&self) -> DeviceResult<IMUData>;
    
    /// Get the latest camera frame.
    fn get_camera_frame(&self) -> DeviceResult<CameraFrame>;
    
    /// Get the tracking state.
    fn get_tracking_state(&self) -> DeviceResult<TrackingState>;
    
    /// Start tracking.
    fn start_tracking(&mut self) -> DeviceResult<()>;
    
    /// Stop tracking.
    fn stop_tracking(&mut self) -> DeviceResult<()>;
    
    /// Recenter the tracking origin.
    fn recenter(&mut self) -> DeviceResult<()>;
    
    /// Get the tracking frequency in Hz.
    fn get_tracking_frequency(&self) -> DeviceResult<f32>;
    
    /// Set the tracking frequency in Hz.
    fn set_tracking_frequency(&mut self, frequency: f32) -> DeviceResult<()>;
    
    /// Get the prediction time in milliseconds.
    fn get_prediction_time(&self) -> DeviceResult<f32>;
    
    /// Set the prediction time in milliseconds.
    fn set_prediction_time(&mut self, time_ms: f32) -> DeviceResult<()>;
    
    /// Get the tracking space origin.
    fn get_tracking_space_origin(&self) -> DeviceResult<Pose>;
    
    /// Set the tracking space origin.
    fn set_tracking_space_origin(&mut self, origin: Pose) -> DeviceResult<()>;
    
    /// Get the boundary points.
    fn get_boundary_points(&self) -> DeviceResult<Vec<Vector3>>;
    
    /// Set the boundary points.
    fn set_boundary_points(&mut self, points: Vec<Vector3>) -> DeviceResult<()>;
    
    /// Check if a point is within the boundary.
    fn is_point_in_boundary(&self, point: Vector3) -> DeviceResult<bool>;
    
    /// Get the camera intrinsics.
    fn get_camera_intrinsics(&self) -> DeviceResult<CameraIntrinsics>;
    
    /// Get the camera extrinsics relative to the tracking origin.
    fn get_camera_extrinsics(&self) -> DeviceResult<Pose>;
    
    /// Clone the tracking device.
    fn clone_tracking_box(&self) -> Box<dyn TrackingDevice>;
}

/// Tracking manager for managing multiple tracking devices and sensor fusion.
#[derive(Debug)]
pub struct TrackingManager {
    /// Tracking devices by ID
    devices: HashMap<String, Arc<Mutex<Box<dyn TrackingDevice>>>>,
    
    /// Primary tracking device ID (e.g., HMD IMU)
    primary_device_id: Option<String>,
    
    /// Sensor fusion algorithm
    fusion_algorithm: String,
    
    /// Current fused pose
    fused_pose: Arc<Mutex<Pose>>,
    
    /// Tracking space origin
    tracking_space_origin: Pose,
    
    /// Boundary points
    boundary_points: Vec<Vector3>,
}

impl TrackingManager {
    /// Create a new TrackingManager.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            primary_device_id: None,
            fusion_algorithm: "EKF".to_string(), // Default to Extended Kalman Filter
            fused_pose: Arc::new(Mutex::new(Pose::identity())),
            tracking_space_origin: Pose::identity(),
            boundary_points: Vec::new(),
        }
    }
    
    /// Initialize the tracking manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing TrackingManager");
        // TODO: Initialize sensor fusion algorithm
        Ok(())
    }
    
    /// Shutdown the tracking manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down TrackingManager");
        
        // Shutdown all tracking devices
        for (id, device) in &self.devices {
            info!("Shutting down tracking device {}", id);
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on tracking device".to_string())
            })?;
            
            if let Err(e) = device.shutdown() {
                warn!("Failed to shutdown tracking device {}: {}", id, e);
            }
        }
        
        self.devices.clear();
        self.primary_device_id = None;
        
        Ok(())
    }
    
    /// Register a tracking device.
    pub fn register_device(
        &mut self,
        id: &str,
        device: Box<dyn TrackingDevice>,
    ) -> DeviceResult<()> {
        info!("Registering tracking device {}", id);
        
        let device = Arc::new(Mutex::new(device));
        self.devices.insert(id.to_string(), device);
        
        // If this is the first device, set it as primary
        if self.primary_device_id.is_none() {
            self.set_primary_device(id)?;
        }
        
        Ok(())
    }
    
    /// Unregister a tracking device.
    pub fn unregister_device(&mut self, id: &str) -> DeviceResult<()> {
        info!("Unregistering tracking device {}", id);
        
        if self.devices.remove(id).is_none() {
            return Err(DeviceError::NotFound(format!("Tracking device {} not found", id)));
        }
        
        // Update primary device ID if necessary
        if Some(id.to_string()) == self.primary_device_id {
            self.primary_device_id = None;
            
            // Find a new primary device (e.g., the first remaining device)
            if let Some(first_key) = self.devices.keys().next() {
                self.primary_device_id = Some(first_key.clone());
            }
        }
        
        Ok(())
    }
    
    /// Get a tracking device.
    pub fn get_device(&self, id: &str) -> DeviceResult<Arc<Mutex<Box<dyn TrackingDevice>>>> {
        self.devices
            .get(id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Tracking device {} not found", id)))
    }
    
    /// Get all tracking devices.
    pub fn get_all_devices(&self) -> HashMap<String, Arc<Mutex<Box<dyn TrackingDevice>>>> {
        self.devices.clone()
    }
    
    /// Get the primary tracking device.
    pub fn get_primary_device(&self) -> DeviceResult<Arc<Mutex<Box<dyn TrackingDevice>>>> {
        if let Some(id) = &self.primary_device_id {
            self.get_device(id)
        } else {
            Err(DeviceError::NotFound("No primary tracking device set".to_string()))
        }
    }
    
    /// Set the primary tracking device.
    pub fn set_primary_device(&mut self, id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(id) {
            return Err(DeviceError::NotFound(format!("Tracking device {} not found", id)));
        }
        
        info!("Setting {} as primary tracking device", id);
        self.primary_device_id = Some(id.to_string());
        Ok(())
    }
    
    /// Get the current fused pose.
    pub fn get_fused_pose(&self) -> DeviceResult<Pose> {
        let pose = self.fused_pose.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on fused pose".to_string())
        })?;
        Ok(*pose)
    }
    
    /// Update the fused pose based on sensor data.
    pub fn update_fusion(&mut self) -> DeviceResult<()> {
        // TODO: Implement sensor fusion logic
        // This would involve:
        // 1. Getting data from all registered tracking devices
        // 2. Applying the chosen fusion algorithm (e.g., EKF)
        // 3. Updating the self.fused_pose
        
        // Placeholder: Just get pose from primary device for now
        if let Ok(primary_device) = self.get_primary_device() {
            let device_guard = primary_device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on primary device".to_string())
            })?;
            
            if let Ok(pose) = device_guard.get_pose() {
                let mut fused_pose = self.fused_pose.lock().map_err(|_| {
                    DeviceError::CommunicationError("Failed to acquire lock on fused pose".to_string())
                })?;
                *fused_pose = pose;
            }
        }
        
        Ok(())
    }
    
    /// Set the sensor fusion algorithm.
    pub fn set_fusion_algorithm(&mut self, algorithm: &str) -> DeviceResult<()> {
        info!("Setting sensor fusion algorithm to {}", algorithm);
        self.fusion_algorithm = algorithm.to_string();
        // TODO: Reinitialize fusion algorithm if necessary
        Ok(())
    }
    
    /// Get the sensor fusion algorithm.
    pub fn get_fusion_algorithm(&self) -> String {
        self.fusion_algorithm.clone()
    }
    
    /// Recenter the tracking origin for all devices.
    pub fn recenter_all(&mut self) -> DeviceResult<()> {
        info!("Recentering tracking origin for all devices");
        
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on tracking device".to_string())
            })?;
            
            if let Err(e) = device.recenter() {
                warn!("Failed to recenter tracking device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get the tracking space origin.
    pub fn get_tracking_space_origin(&self) -> Pose {
        self.tracking_space_origin
    }
    
    /// Set the tracking space origin.
    pub fn set_tracking_space_origin(&mut self, origin: Pose) -> DeviceResult<()> {
        info!("Setting tracking space origin");
        self.tracking_space_origin = origin;
        
        // Set the origin on all tracking devices
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on tracking device".to_string())
            })?;
            
            if let Err(e) = device.set_tracking_space_origin(origin) {
                warn!("Failed to set tracking space origin on device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get the boundary points.
    pub fn get_boundary_points(&self) -> Vec<Vector3> {
        self.boundary_points.clone()
    }
    
    /// Set the boundary points.
    pub fn set_boundary_points(&mut self, points: Vec<Vector3>) -> DeviceResult<()> {
        info!("Setting boundary points");
        self.boundary_points = points.clone();
        
        // Set the boundary points on all tracking devices
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on tracking device".to_string())
            })?;
            
            if let Err(e) = device.set_boundary_points(points.clone()) {
                warn!("Failed to set boundary points on device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Check if a point is within the boundary.
    pub fn is_point_in_boundary(&self, point: Vector3) -> bool {
        // Simple implementation for now
        // TODO: Implement proper boundary checking algorithm
        if self.boundary_points.is_empty() {
            return true; // No boundary defined, all points are "in bounds"
        }
        
        // For now, just check if the point is within a certain distance of any boundary point
        // This is not a proper boundary check, just a placeholder
        for boundary_point in &self.boundary_points {
            let dx = boundary_point.x - point.x;
            let dy = boundary_point.y - point.y;
            let dz = boundary_point.z - point.z;
            let distance_squared = dx * dx + dy * dy + dz * dz;
            
            if distance_squared < 1.0 {
                return true;
            }
        }
        
        false
    }
}

/// Mock tracking device for testing.
#[derive(Debug, Clone)]
pub struct MockTrackingDevice {
    /// Device info
    pub info: DeviceInfo,
    
    /// Device state
    pub state: DeviceState,
    
    /// Device properties
    pub properties: HashMap<String, String>,
    
    /// Event handlers
    pub event_handlers: Vec<DeviceEventHandler>,
    
    /// Current pose
    pub pose: Pose,
    
    /// Current IMU data
    pub imu_data: IMUData,
    
    /// Current camera frame
    pub camera_frame: CameraFrame,
    
    /// Tracking state
    pub tracking_state: TrackingState,
    
    /// Tracking frequency in Hz
    pub tracking_frequency: f32,
    
    /// Prediction time in milliseconds
    pub prediction_time: f32,
    
    /// Tracking space origin
    pub tracking_space_origin: Pose,
    
    /// Boundary points
    pub boundary_points: Vec<Vector3>,
    
    /// Camera intrinsics
    pub camera_intrinsics: CameraIntrinsics,
    
    /// Camera extrinsics
    pub camera_extrinsics: Pose,
}

impl MockTrackingDevice {
    /// Create a new MockTrackingDevice.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let mut info = DeviceInfo::new(
            id,
            name,
            DeviceType::Tracking,
            manufacturer,
            model,
            DeviceBus::Virtual,
        );
        
        info.add_capability(DeviceCapability::Tracking(TrackingCapability::SixDof));
        info.add_capability(DeviceCapability::Tracking(TrackingCapability::InsideOut));
        info.state = DeviceState::Connected;
        
        let now = SystemTime::now();
        
        let pose = Pose::identity();
        
        let imu_data = IMUData::new(
            Vector3::zero(),
            Vector3::zero(),
            Some(Vector3::new(0.0, 0.0, 0.0)),
            Some(25.0),
            now,
        );
        
        let camera_intrinsics = CameraIntrinsics::new(
            500.0, 500.0, 320.0, 240.0, [0.0, 0.0, 0.0, 0.0, 0.0],
        );
        
        let camera_extrinsics = Pose::identity();
        
        // Create a dummy camera frame
        let width = 640;
        let height = 480;
        let format = "RGB".to_string();
        let data = vec![0; width as usize * height as usize * 3]; // RGB data
        
        let camera_frame = CameraFrame::new(
            width,
            height,
            format,
            data,
            now,
            0,
            Some(camera_intrinsics),
            Some(camera_extrinsics),
        );
        
        Self {
            info,
            state: DeviceState::Connected,
            properties: HashMap::new(),
            event_handlers: Vec::new(),
            pose,
            imu_data,
            camera_frame,
            tracking_state: TrackingState::Tracking,
            tracking_frequency: 60.0,
            prediction_time: 10.0,
            tracking_space_origin: Pose::identity(),
            boundary_points: Vec::new(),
            camera_intrinsics,
            camera_extrinsics,
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

impl Device for MockTrackingDevice {
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

impl TrackingDevice for MockTrackingDevice {
    fn get_pose(&self) -> DeviceResult<Pose> {
        Ok(self.pose)
    }
    
    fn get_imu_data(&self) -> DeviceResult<IMUData> {
        Ok(self.imu_data)
    }
    
    fn get_camera_frame(&self) -> DeviceResult<CameraFrame> {
        Ok(self.camera_frame.clone())
    }
    
    fn get_tracking_state(&self) -> DeviceResult<TrackingState> {
        Ok(self.tracking_state)
    }
    
    fn start_tracking(&mut self) -> DeviceResult<()> {
        self.tracking_state = TrackingState::Tracking;
        Ok(())
    }
    
    fn stop_tracking(&mut self) -> DeviceResult<()> {
        self.tracking_state = TrackingState::Inactive;
        Ok(())
    }
    
    fn recenter(&mut self) -> DeviceResult<()> {
        self.pose = Pose::identity();
        self.tracking_space_origin = Pose::identity();
        Ok(())
    }
    
    fn get_tracking_frequency(&self) -> DeviceResult<f32> {
        Ok(self.tracking_frequency)
    }
    
    fn set_tracking_frequency(&mut self, frequency: f32) -> DeviceResult<()> {
        self.tracking_frequency = frequency;
        Ok(())
    }
    
    fn get_prediction_time(&self) -> DeviceResult<f32> {
        Ok(self.prediction_time)
    }
    
    fn set_prediction_time(&mut self, time_ms: f32) -> DeviceResult<()> {
        self.prediction_time = time_ms;
        Ok(())
    }
    
    fn get_tracking_space_origin(&self) -> DeviceResult<Pose> {
        Ok(self.tracking_space_origin)
    }
    
    fn set_tracking_space_origin(&mut self, origin: Pose) -> DeviceResult<()> {
        self.tracking_space_origin = origin;
        Ok(())
    }
    
    fn get_boundary_points(&self) -> DeviceResult<Vec<Vector3>> {
        Ok(self.boundary_points.clone())
    }
    
    fn set_boundary_points(&mut self, points: Vec<Vector3>) -> DeviceResult<()> {
        self.boundary_points = points;
        Ok(())
    }
    
    fn is_point_in_boundary(&self, point: Vector3) -> DeviceResult<bool> {
        // Simple implementation for now
        if self.boundary_points.is_empty() {
            return Ok(true); // No boundary defined, all points are "in bounds"
        }
        
        // For now, just check if the point is within a certain distance of any boundary point
        for boundary_point in &self.boundary_points {
            let dx = boundary_point.x - point.x;
            let dy = boundary_point.y - point.y;
            let dz = boundary_point.z - point.z;
            let distance_squared = dx * dx + dy * dy + dz * dz;
            
            if distance_squared < 1.0 {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn get_camera_intrinsics(&self) -> DeviceResult<CameraIntrinsics> {
        Ok(self.camera_intrinsics)
    }
    
    fn get_camera_extrinsics(&self) -> DeviceResult<Pose> {
        Ok(self.camera_extrinsics)
    }
    
    fn clone_tracking_box(&self) -> Box<dyn TrackingDevice> {
        Box::new(self.clone())
    }
}
