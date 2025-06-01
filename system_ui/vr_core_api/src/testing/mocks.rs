//! Mock objects module for the VR headset system.
//!
//! This module provides mock implementations of various system components
//! for testing purposes. These mocks simulate the behavior of real components
//! without requiring actual hardware.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Mock hardware device trait
pub trait MockDevice: Send + Sync {
    /// Get the device name
    fn name(&self) -> &str;
    
    /// Get the device type
    fn device_type(&self) -> &str;
    
    /// Initialize the device
    fn initialize(&mut self) -> Result<(), String>;
    
    /// Shutdown the device
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// Check if the device is initialized
    fn is_initialized(&self) -> bool;
    
    /// Reset the device to its initial state
    fn reset(&mut self) -> Result<(), String>;
}

/// Mock display device
pub struct MockDisplayDevice {
    /// Device name
    name: String,
    /// Whether the device is initialized
    initialized: bool,
    /// Display resolution
    resolution: (u32, u32),
    /// Refresh rate in Hz
    refresh_rate: u32,
    /// Current brightness (0-100)
    brightness: u32,
    /// Whether the display is on
    power_on: bool,
    /// Frame buffer
    frame_buffer: Vec<u8>,
    /// Error simulation
    simulate_error: bool,
}

impl MockDisplayDevice {
    /// Create a new mock display device
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            resolution: (1920, 1080),
            refresh_rate: 90,
            brightness: 80,
            power_on: false,
            frame_buffer: Vec::new(),
            simulate_error: false,
        }
    }
    
    /// Set the display resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.resolution = (width, height);
        // Resize frame buffer
        self.frame_buffer = vec![0; (width * height * 4) as usize];
        Ok(())
    }
    
    /// Get the display resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
    
    /// Set the refresh rate
    pub fn set_refresh_rate(&mut self, rate: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.refresh_rate = rate;
        Ok(())
    }
    
    /// Get the refresh rate
    pub fn refresh_rate(&self) -> u32 {
        self.refresh_rate
    }
    
    /// Set the brightness
    pub fn set_brightness(&mut self, brightness: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if brightness > 100 {
            return Err("Brightness must be between 0 and 100".to_string());
        }
        
        self.brightness = brightness;
        Ok(())
    }
    
    /// Get the brightness
    pub fn brightness(&self) -> u32 {
        self.brightness
    }
    
    /// Power on the display
    pub fn power_on(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if self.simulate_error {
            return Err("Simulated power on error".to_string());
        }
        
        self.power_on = true;
        Ok(())
    }
    
    /// Power off the display
    pub fn power_off(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.power_on = false;
        Ok(())
    }
    
    /// Check if the display is powered on
    pub fn is_powered_on(&self) -> bool {
        self.power_on
    }
    
    /// Update the frame buffer
    pub fn update_frame(&mut self, frame_data: &[u8]) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if !self.power_on {
            return Err("Display is powered off".to_string());
        }
        
        let expected_size = (self.resolution.0 * self.resolution.1 * 4) as usize;
        if frame_data.len() != expected_size {
            return Err(format!("Invalid frame size. Expected {} bytes, got {}", expected_size, frame_data.len()));
        }
        
        self.frame_buffer.clear();
        self.frame_buffer.extend_from_slice(frame_data);
        
        Ok(())
    }
    
    /// Get the current frame buffer
    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }
    
    /// Set whether to simulate errors
    pub fn set_simulate_error(&mut self, simulate: bool) {
        self.simulate_error = simulate;
    }
}

impl MockDevice for MockDisplayDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> &str {
        "display"
    }
    
    fn initialize(&mut self) -> Result<(), String> {
        if self.simulate_error {
            return Err("Simulated initialization error".to_string());
        }
        
        self.initialized = true;
        // Initialize frame buffer
        self.frame_buffer = vec![0; (self.resolution.0 * self.resolution.1 * 4) as usize];
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Power off first
        if self.power_on {
            self.power_off()?;
        }
        
        self.initialized = false;
        self.frame_buffer.clear();
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn reset(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Reset to default state
        self.power_on = false;
        self.brightness = 80;
        self.refresh_rate = 90;
        self.frame_buffer = vec![0; (self.resolution.0 * self.resolution.1 * 4) as usize];
        
        Ok(())
    }
}

/// Mock camera device
pub struct MockCameraDevice {
    /// Device name
    name: String,
    /// Whether the device is initialized
    initialized: bool,
    /// Camera resolution
    resolution: (u32, u32),
    /// Frame rate in fps
    frame_rate: u32,
    /// Whether the camera is streaming
    streaming: bool,
    /// Current frame data
    current_frame: Vec<u8>,
    /// Simulated frames
    simulated_frames: Vec<Vec<u8>>,
    /// Current frame index
    frame_index: usize,
    /// Error simulation
    simulate_error: bool,
}

impl MockCameraDevice {
    /// Create a new mock camera device
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            resolution: (1280, 800),
            frame_rate: 60,
            streaming: false,
            current_frame: Vec::new(),
            simulated_frames: Vec::new(),
            frame_index: 0,
            simulate_error: false,
        }
    }
    
    /// Set the camera resolution
    pub fn set_resolution(&mut self, width: u32, height: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.resolution = (width, height);
        Ok(())
    }
    
    /// Get the camera resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
    
    /// Set the frame rate
    pub fn set_frame_rate(&mut self, rate: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.frame_rate = rate;
        Ok(())
    }
    
    /// Get the frame rate
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate
    }
    
    /// Start streaming
    pub fn start_streaming(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if self.simulate_error {
            return Err("Simulated streaming error".to_string());
        }
        
        self.streaming = true;
        Ok(())
    }
    
    /// Stop streaming
    pub fn stop_streaming(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.streaming = false;
        Ok(())
    }
    
    /// Check if the camera is streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming
    }
    
    /// Add a simulated frame
    pub fn add_simulated_frame(&mut self, frame_data: &[u8]) {
        self.simulated_frames.push(frame_data.to_vec());
    }
    
    /// Get the next frame
    pub fn get_frame(&mut self) -> Result<&[u8], String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if !self.streaming {
            return Err("Camera is not streaming".to_string());
        }
        
        if self.simulated_frames.is_empty() {
            // Generate a test pattern if no simulated frames
            let frame_size = (self.resolution.0 * self.resolution.1 * 3) as usize;
            self.current_frame = vec![0; frame_size];
            
            // Create a simple test pattern
            for y in 0..self.resolution.1 {
                for x in 0..self.resolution.0 {
                    let index = ((y * self.resolution.0 + x) * 3) as usize;
                    self.current_frame[index] = (x % 256) as u8; // R
                    self.current_frame[index + 1] = (y % 256) as u8; // G
                    self.current_frame[index + 2] = ((x + y) % 256) as u8; // B
                }
            }
        } else {
            // Use the next simulated frame
            self.current_frame = self.simulated_frames[self.frame_index].clone();
            self.frame_index = (self.frame_index + 1) % self.simulated_frames.len();
        }
        
        Ok(&self.current_frame)
    }
    
    /// Set whether to simulate errors
    pub fn set_simulate_error(&mut self, simulate: bool) {
        self.simulate_error = simulate;
    }
}

impl MockDevice for MockCameraDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> &str {
        "camera"
    }
    
    fn initialize(&mut self) -> Result<(), String> {
        if self.simulate_error {
            return Err("Simulated initialization error".to_string());
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Stop streaming first
        if self.streaming {
            self.stop_streaming()?;
        }
        
        self.initialized = false;
        self.current_frame.clear();
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn reset(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Reset to default state
        self.streaming = false;
        self.frame_rate = 60;
        self.current_frame.clear();
        self.frame_index = 0;
        
        Ok(())
    }
}

/// Mock IMU device
pub struct MockImuDevice {
    /// Device name
    name: String,
    /// Whether the device is initialized
    initialized: bool,
    /// Sample rate in Hz
    sample_rate: u32,
    /// Whether the IMU is streaming
    streaming: bool,
    /// Current acceleration (x, y, z) in m/s^2
    acceleration: (f32, f32, f32),
    /// Current gyroscope (x, y, z) in rad/s
    gyroscope: (f32, f32, f32),
    /// Current magnetometer (x, y, z) in uT
    magnetometer: (f32, f32, f32),
    /// Simulated IMU data
    simulated_data: Vec<(f32, f32, f32, f32, f32, f32, f32, f32, f32)>,
    /// Current data index
    data_index: usize,
    /// Error simulation
    simulate_error: bool,
}

impl MockImuDevice {
    /// Create a new mock IMU device
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            sample_rate: 1000,
            streaming: false,
            acceleration: (0.0, 0.0, 9.81), // Default to gravity
            gyroscope: (0.0, 0.0, 0.0),
            magnetometer: (0.0, 0.0, 0.0),
            simulated_data: Vec::new(),
            data_index: 0,
            simulate_error: false,
        }
    }
    
    /// Set the sample rate
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.sample_rate = rate;
        Ok(())
    }
    
    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    /// Start streaming
    pub fn start_streaming(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if self.simulate_error {
            return Err("Simulated streaming error".to_string());
        }
        
        self.streaming = true;
        Ok(())
    }
    
    /// Stop streaming
    pub fn stop_streaming(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        self.streaming = false;
        Ok(())
    }
    
    /// Check if the IMU is streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming
    }
    
    /// Add simulated IMU data (accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z)
    pub fn add_simulated_data(&mut self, data: (f32, f32, f32, f32, f32, f32, f32, f32, f32)) {
        self.simulated_data.push(data);
    }
    
    /// Get the next IMU sample
    pub fn get_sample(&mut self) -> Result<(f32, f32, f32, f32, f32, f32, f32, f32, f32), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        if !self.streaming {
            return Err("IMU is not streaming".to_string());
        }
        
        if !self.simulated_data.is_empty() {
            // Use the next simulated data
            let data = self.simulated_data[self.data_index];
            self.data_index = (self.data_index + 1) % self.simulated_data.len();
            
            // Update current values
            self.acceleration = (data.0, data.1, data.2);
            self.gyroscope = (data.3, data.4, data.5);
            self.magnetometer = (data.6, data.7, data.8);
        }
        
        Ok((
            self.acceleration.0, self.acceleration.1, self.acceleration.2,
            self.gyroscope.0, self.gyroscope.1, self.gyroscope.2,
            self.magnetometer.0, self.magnetometer.1, self.magnetometer.2,
        ))
    }
    
    /// Get the current acceleration
    pub fn acceleration(&self) -> (f32, f32, f32) {
        self.acceleration
    }
    
    /// Get the current gyroscope
    pub fn gyroscope(&self) -> (f32, f32, f32) {
        self.gyroscope
    }
    
    /// Get the current magnetometer
    pub fn magnetometer(&self) -> (f32, f32, f32) {
        self.magnetometer
    }
    
    /// Set whether to simulate errors
    pub fn set_simulate_error(&mut self, simulate: bool) {
        self.simulate_error = simulate;
    }
}

impl MockDevice for MockImuDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn device_type(&self) -> &str {
        "imu"
    }
    
    fn initialize(&mut self) -> Result<(), String> {
        if self.simulate_error {
            return Err("Simulated initialization error".to_string());
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Stop streaming first
        if self.streaming {
            self.stop_streaming()?;
        }
        
        self.initialized = false;
        Ok(())
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    fn reset(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Device not initialized".to_string());
        }
        
        // Reset to default state
        self.streaming = false;
        self.sample_rate = 1000;
        self.acceleration = (0.0, 0.0, 9.81);
        self.gyroscope = (0.0, 0.0, 0.0);
        self.magnetometer = (0.0, 0.0, 0.0);
        self.data_index = 0;
        
        Ok(())
    }
}

/// Mock device manager
pub struct MockDeviceManager {
    /// Devices
    devices: HashMap<String, Box<dyn MockDevice>>,
}

impl MockDeviceManager {
    /// Create a new mock device manager
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
    
    /// Add a device
    pub fn add_device<D: MockDevice + 'static>(&mut self, device: D) {
        self.devices.insert(device.name().to_string(), Box::new(device));
    }
    
    /// Get a device
    pub fn get_device(&self, name: &str) -> Option<&Box<dyn MockDevice>> {
        self.devices.get(name)
    }
    
    /// Get a device mutably
    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut Box<dyn MockDevice>> {
        self.devices.get_mut(name)
    }
    
    /// Get a device as a specific type
    pub fn get_device_as<T: 'static>(&self, name: &str) -> Option<&T> {
        if let Some(device) = self.get_device(name) {
            device.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }
    
    /// Get a device as a specific type mutably
    pub fn get_device_as_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        if let Some(device) = self.get_device_mut(name) {
            device.as_any_mut().downcast_mut::<T>()
        } else {
            None
        }
    }
    
    /// Initialize all devices
    pub fn initialize_all(&mut self) -> Result<(), String> {
        for (_, device) in &mut self.devices {
            device.initialize()?;
        }
        Ok(())
    }
    
    /// Shutdown all devices
    pub fn shutdown_all(&mut self) -> Result<(), String> {
        for (_, device) in &mut self.devices {
            device.shutdown()?;
        }
        Ok(())
    }
    
    /// Reset all devices
    pub fn reset_all(&mut self) -> Result<(), String> {
        for (_, device) in &mut self.devices {
            if device.is_initialized() {
                device.reset()?;
            }
        }
        Ok(())
    }
}

/// Extension trait for MockDevice to support downcasting
pub trait MockDeviceExt: MockDevice {
    /// Convert to Any
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Convert to Any mutably
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: MockDevice + 'static> MockDeviceExt for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Extend MockDevice to include MockDeviceExt
pub trait MockDevice: MockDeviceExt + Send + Sync {
    /// Get the device name
    fn name(&self) -> &str;
    
    /// Get the device type
    fn device_type(&self) -> &str;
    
    /// Initialize the device
    fn initialize(&mut self) -> Result<(), String>;
    
    /// Shutdown the device
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// Check if the device is initialized
    fn is_initialized(&self) -> bool;
    
    /// Reset the device to its initial state
    fn reset(&mut self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_display_device() {
        let mut display = MockDisplayDevice::new("test_display");
        
        // Test initialization
        assert!(!display.is_initialized());
        assert!(display.initialize().is_ok());
        assert!(display.is_initialized());
        
        // Test resolution
        assert_eq!(display.resolution(), (1920, 1080));
        assert!(display.set_resolution(2560, 1440).is_ok());
        assert_eq!(display.resolution(), (2560, 1440));
        
        // Test power
        assert!(!display.is_powered_on());
        assert!(display.power_on().is_ok());
        assert!(display.is_powered_on());
        assert!(display.power_off().is_ok());
        assert!(!display.is_powered_on());
        
        // Test shutdown
        assert!(display.shutdown().is_ok());
        assert!(!display.is_initialized());
    }

    #[test]
    fn test_mock_camera_device() {
        let mut camera = MockCameraDevice::new("test_camera");
        
        // Test initialization
        assert!(!camera.is_initialized());
        assert!(camera.initialize().is_ok());
        assert!(camera.is_initialized());
        
        // Test resolution
        assert_eq!(camera.resolution(), (1280, 800));
        assert!(camera.set_resolution(1920, 1080).is_ok());
        assert_eq!(camera.resolution(), (1920, 1080));
        
        // Test streaming
        assert!(!camera.is_streaming());
        assert!(camera.start_streaming().is_ok());
        assert!(camera.is_streaming());
        
        // Test frame
        let frame = camera.get_frame();
        assert!(frame.is_ok());
        assert!(!frame.unwrap().is_empty());
        
        // Test shutdown
        assert!(camera.shutdown().is_ok());
        assert!(!camera.is_initialized());
    }

    #[test]
    fn test_mock_device_manager() {
        let mut manager = MockDeviceManager::new();
        
        // Add devices
        manager.add_device(MockDisplayDevice::new("display1"));
        manager.add_device(MockCameraDevice::new("camera1"));
        
        // Initialize all
        assert!(manager.initialize_all().is_ok());
        
        // Get devices
        let display = manager.get_device("display1");
        assert!(display.is_some());
        assert_eq!(display.unwrap().device_type(), "display");
        
        let camera = manager.get_device("camera1");
        assert!(camera.is_some());
        assert_eq!(camera.unwrap().device_type(), "camera");
        
        // Get as specific type
        let display = manager.get_device_as_mut::<MockDisplayDevice>("display1");
        assert!(display.is_some());
        if let Some(display) = display {
            assert!(display.power_on().is_ok());
            assert!(display.is_powered_on());
        }
        
        // Shutdown all
        assert!(manager.shutdown_all().is_ok());
    }
}
