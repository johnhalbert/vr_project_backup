//! Core API integration for the OpenVR driver

use std::sync::{Arc, Mutex};
use crate::tracking::{SLAMInterface, SLAMPose};
use crate::settings::ConfigInterface;
use crate::error::{Result, Error};
use vr_core_api::hardware::IMUData;
use vr_core_api::slam::SLAMData;
use vr_core_api::config::ConfigManager;

/// Core API integration for the OpenVR driver
pub struct CoreAPIIntegration {
    /// Core API instance
    core_api: Arc<vr_core_api::VRCoreAPI>,
}

impl CoreAPIIntegration {
    /// Create a new Core API integration
    pub fn new(core_api: Arc<vr_core_api::VRCoreAPI>) -> Self {
        Self {
            core_api,
        }
    }
    
    /// Get the SLAM interface
    pub fn get_slam_interface(&self) -> Arc<Mutex<dyn SLAMInterface>> {
        Arc::new(Mutex::new(CoreAPISLAMInterface::new(self.core_api.clone())))
    }
    
    /// Get the configuration interface
    pub fn get_config_interface(&self) -> Arc<Mutex<dyn ConfigInterface>> {
        Arc::new(Mutex::new(CoreAPIConfigInterface::new(self.core_api.clone())))
    }
}

/// Core API SLAM interface implementation
struct CoreAPISLAMInterface {
    /// Core API instance
    core_api: Arc<vr_core_api::VRCoreAPI>,
}

impl CoreAPISLAMInterface {
    /// Create a new Core API SLAM interface
    fn new(core_api: Arc<vr_core_api::VRCoreAPI>) -> Self {
        Self {
            core_api,
        }
    }
    
    /// Convert Core API SLAM data to OpenVR SLAM pose
    fn slam_data_to_pose(&self, slam_data: &SLAMData) -> SLAMPose {
        SLAMPose {
            position: slam_data.position,
            rotation: slam_data.rotation,
            velocity: slam_data.velocity,
            angular_velocity: slam_data.angular_velocity,
            confidence: slam_data.tracking_confidence,
        }
    }
}

impl SLAMInterface for CoreAPISLAMInterface {
    fn get_latest_pose(&self) -> Result<SLAMPose> {
        // Get latest IMU data
        let imu_data = self.core_api.hardware().get_imu_data()
            .map_err(|e| Error::CoreAPIError(format!("Failed to get IMU data: {}", e)))?;
        
        // Get latest camera frames
        let camera_frames = self.core_api.hardware().get_camera_frames()
            .map_err(|e| Error::CoreAPIError(format!("Failed to get camera frames: {}", e)))?;
        
        // Process through SLAM pipeline
        let slam_data = self.core_api.slam().process_frames_and_imu(&camera_frames, &imu_data)
            .map_err(|e| Error::CoreAPIError(format!("Failed to process SLAM data: {}", e)))?;
        
        Ok(self.slam_data_to_pose(&slam_data))
    }
    
    fn predict_pose(&self, time_offset_seconds: f32) -> Result<SLAMPose> {
        // Get latest pose
        let latest_pose = self.get_latest_pose()?;
        
        // Get latest IMU data for prediction
        let imu_data = self.core_api.hardware().get_imu_data()
            .map_err(|e| Error::CoreAPIError(format!("Failed to get IMU data: {}", e)))?;
        
        // Use IMU data to predict future pose
        let predicted_slam_data = self.core_api.slam().predict_pose(time_offset_seconds, &imu_data)
            .map_err(|e| Error::CoreAPIError(format!("Failed to predict pose: {}", e)))?;
        
        Ok(self.slam_data_to_pose(&predicted_slam_data))
    }
}

/// Core API configuration interface implementation
struct CoreAPIConfigInterface {
    /// Core API instance
    core_api: Arc<vr_core_api::VRCoreAPI>,
}

impl CoreAPIConfigInterface {
    /// Create a new Core API configuration interface
    fn new(core_api: Arc<vr_core_api::VRCoreAPI>) -> Self {
        Self {
            core_api,
        }
    }
}

impl ConfigInterface for CoreAPIConfigInterface {
    fn get_int(&self, key: &str) -> Result<i32> {
        self.core_api.config().get_int(key)
            .map_err(|e| Error::CoreAPIError(format!("Failed to get int config value: {}", e)))
    }
    
    fn get_float(&self, key: &str) -> Result<f32> {
        self.core_api.config().get_float(key)
            .map_err(|e| Error::CoreAPIError(format!("Failed to get float config value: {}", e)))
    }
    
    fn get_bool(&self, key: &str) -> Result<bool> {
        self.core_api.config().get_bool(key)
            .map_err(|e| Error::CoreAPIError(format!("Failed to get bool config value: {}", e)))
    }
    
    fn get_string(&self, key: &str) -> Result<String> {
        self.core_api.config().get_string(key)
            .map_err(|e| Error::CoreAPIError(format!("Failed to get string config value: {}", e)))
    }
    
    fn set_int(&mut self, key: &str, value: i32) -> Result<()> {
        self.core_api.config_mut().set_int(key, value)
            .map_err(|e| Error::CoreAPIError(format!("Failed to set int config value: {}", e)))
    }
    
    fn set_float(&mut self, key: &str, value: f32) -> Result<()> {
        self.core_api.config_mut().set_float(key, value)
            .map_err(|e| Error::CoreAPIError(format!("Failed to set float config value: {}", e)))
    }
    
    fn set_bool(&mut self, key: &str, value: bool) -> Result<()> {
        self.core_api.config_mut().set_bool(key, value)
            .map_err(|e| Error::CoreAPIError(format!("Failed to set bool config value: {}", e)))
    }
    
    fn set_string(&mut self, key: &str, value: &str) -> Result<()> {
        self.core_api.config_mut().set_string(key, value)
            .map_err(|e| Error::CoreAPIError(format!("Failed to set string config value: {}", e)))
    }
    
    fn save(&mut self) -> Result<()> {
        self.core_api.config_mut().save()
            .map_err(|e| Error::CoreAPIError(format!("Failed to save config: {}", e)))
    }
}
