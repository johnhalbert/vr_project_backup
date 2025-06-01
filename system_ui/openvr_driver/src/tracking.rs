//! Tracking functionality for the OpenVR driver

use std::sync::{Arc, Mutex};
use crate::types::{Pose, DeviceType};
use crate::error::{Result, Error};

/// Interface for tracking data providers
pub trait TrackingDataProvider: Send + Sync {
    /// Get the latest HMD pose
    fn get_hmd_pose(&self) -> Result<Pose>;
    
    /// Get the latest controller poses (left, right)
    fn get_controller_poses(&self) -> Result<(Pose, Pose)>;
    
    /// Get the latest tracker poses
    fn get_tracker_poses(&self) -> Result<Vec<Pose>>;
    
    /// Get the latest tracking reference poses
    fn get_tracking_reference_poses(&self) -> Result<Vec<Pose>>;
}

/// Tracking provider that integrates with the SLAM system
pub struct TrackingProvider {
    /// Core API interface for SLAM data
    slam_interface: Arc<Mutex<dyn SLAMInterface>>,
    
    /// Prediction time in seconds
    prediction_time: f32,
}

/// Interface to the SLAM system
pub trait SLAMInterface: Send + Sync {
    /// Get the latest SLAM pose data
    fn get_latest_pose(&self) -> Result<SLAMPose>;
    
    /// Predict pose at a future time
    fn predict_pose(&self, time_offset_seconds: f32) -> Result<SLAMPose>;
}

/// SLAM pose data
#[derive(Debug, Clone)]
pub struct SLAMPose {
    /// Position [x, y, z] in meters
    pub position: [f32; 3],
    
    /// Rotation as quaternion [x, y, z, w]
    pub rotation: [f32; 4],
    
    /// Linear velocity [x, y, z] in meters per second
    pub velocity: [f32; 3],
    
    /// Angular velocity [x, y, z] in radians per second
    pub angular_velocity: [f32; 3],
    
    /// Tracking confidence (0.0 - 1.0)
    pub confidence: f32,
}

impl TrackingProvider {
    /// Create a new tracking provider
    pub fn new(slam_interface: Arc<Mutex<dyn SLAMInterface>>, prediction_time_ms: f32) -> Self {
        Self {
            slam_interface,
            prediction_time: prediction_time_ms / 1000.0, // Convert to seconds
        }
    }
    
    /// Set the prediction time in milliseconds
    pub fn set_prediction_time_ms(&mut self, prediction_time_ms: f32) {
        self.prediction_time = prediction_time_ms / 1000.0;
    }
    
    /// Convert SLAM pose to OpenVR pose
    fn slam_pose_to_openvr_pose(&self, slam_pose: SLAMPose, device_type: DeviceType) -> Pose {
        // Apply device-specific transformations based on type
        let (position, rotation) = match device_type {
            DeviceType::HMD => (slam_pose.position, slam_pose.rotation),
            DeviceType::Controller => {
                // Controllers would have offsets from the HMD
                // This is a simplified example
                (slam_pose.position, slam_pose.rotation)
            },
            DeviceType::Tracker => (slam_pose.position, slam_pose.rotation),
            DeviceType::TrackingReference => (slam_pose.position, slam_pose.rotation),
        };
        
        Pose {
            device_is_connected: true,
            pose_is_valid: slam_pose.confidence > 0.5,
            device_is_tracking: slam_pose.confidence > 0.5,
            position,
            rotation,
            velocity: slam_pose.velocity,
            angular_velocity: slam_pose.angular_velocity,
        }
    }
    
    /// Derive left controller pose from HMD pose
    fn derive_left_controller_pose(&self, hmd_pose: &Pose) -> Pose {
        // This is a simplified implementation
        // In a real system, this would use actual controller tracking data
        let mut pose = hmd_pose.clone();
        
        // Offset to the left of the HMD
        pose.position[0] -= 0.2; // 20cm to the left
        pose.position[1] -= 0.3; // 30cm down
        
        pose
    }
    
    /// Derive right controller pose from HMD pose
    fn derive_right_controller_pose(&self, hmd_pose: &Pose) -> Pose {
        // This is a simplified implementation
        // In a real system, this would use actual controller tracking data
        let mut pose = hmd_pose.clone();
        
        // Offset to the right of the HMD
        pose.position[0] += 0.2; // 20cm to the right
        pose.position[1] -= 0.3; // 30cm down
        
        pose
    }
}

impl TrackingDataProvider for TrackingProvider {
    fn get_hmd_pose(&self) -> Result<Pose> {
        let slam = self.slam_interface.lock().map_err(|_| Error::CoreAPIError("Failed to lock SLAM interface".to_string()))?;
        
        // Get predicted pose
        let slam_pose = slam.predict_pose(self.prediction_time)?;
        
        Ok(self.slam_pose_to_openvr_pose(slam_pose, DeviceType::HMD))
    }
    
    fn get_controller_poses(&self) -> Result<(Pose, Pose)> {
        // Get HMD pose first
        let hmd_pose = self.get_hmd_pose()?;
        
        // Derive controller poses
        // In a real implementation, these would come from actual tracking data
        let left_pose = self.derive_left_controller_pose(&hmd_pose);
        let right_pose = self.derive_right_controller_pose(&hmd_pose);
        
        Ok((left_pose, right_pose))
    }
    
    fn get_tracker_poses(&self) -> Result<Vec<Pose>> {
        // In a real implementation, this would return actual tracker poses
        // For now, return an empty vector
        Ok(Vec::new())
    }
    
    fn get_tracking_reference_poses(&self) -> Result<Vec<Pose>> {
        // In a real implementation, this would return actual tracking reference poses
        // For now, return an empty vector
        Ok(Vec::new())
    }
}
