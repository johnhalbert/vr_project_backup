//! Test utilities module for the VR headset system.
//!
//! This module provides utility functions and helpers for testing,
//! including assertion helpers, test data generators, and timing utilities.

use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// Assert that two floating point values are approximately equal
pub fn assert_approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Assert that two 3D vectors are approximately equal
pub fn assert_vec3_approx_eq(a: (f32, f32, f32), b: (f32, f32, f32), epsilon: f32) -> bool {
    assert_approx_eq(a.0, b.0, epsilon) &&
    assert_approx_eq(a.1, b.1, epsilon) &&
    assert_approx_eq(a.2, b.2, epsilon)
}

/// Measure the execution time of a function
pub fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Run a function with a timeout
pub fn run_with_timeout<F, T>(f: F, timeout: Duration) -> Result<T, &'static str>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let result = Arc::new(Mutex::new(None));
    let result_clone = Arc::clone(&result);
    
    let handle = thread::spawn(move || {
        let value = f();
        let mut result = result_clone.lock().unwrap();
        *result = Some(value);
    });
    
    // Wait for the thread to complete or timeout
    let sleep_interval = Duration::from_millis(10);
    let mut elapsed = Duration::from_millis(0);
    
    while elapsed < timeout {
        {
            let result_guard = result.lock().unwrap();
            if result_guard.is_some() {
                // Function completed
                return Ok(result.lock().unwrap().take().unwrap());
            }
        }
        
        thread::sleep(sleep_interval);
        elapsed += sleep_interval;
    }
    
    // Timeout occurred
    Err("Function timed out")
}

/// Create a temporary test directory
pub fn create_temp_test_dir(prefix: &str) -> io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("{}_{}", prefix, rand::random::<u64>()));
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

/// Clean up a temporary test directory
pub fn cleanup_temp_test_dir(dir: &PathBuf) -> io::Result<()> {
    fs::remove_dir_all(dir)
}

/// Create a test file with the given content
pub fn create_test_file(dir: &PathBuf, filename: &str, content: &[u8]) -> io::Result<PathBuf> {
    let file_path = dir.join(filename);
    let mut file = File::create(&file_path)?;
    file.write_all(content)?;
    Ok(file_path)
}

/// Read a test file
pub fn read_test_file(path: &PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

/// Generate a random test image
pub fn generate_test_image(width: u32, height: u32) -> Vec<u8> {
    let mut image = Vec::with_capacity((width * height * 3) as usize);
    
    for y in 0..height {
        for x in 0..width {
            // Generate a simple pattern
            image.push(((x * 255) / width) as u8);
            image.push(((y * 255) / height) as u8);
            image.push((((x + y) * 255) / (width + height)) as u8);
        }
    }
    
    image
}

/// Generate a random test IMU data sample
pub fn generate_test_imu_sample() -> (f32, f32, f32, f32, f32, f32, f32, f32, f32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Generate random acceleration (with gravity)
    let accel_x = rng.gen_range(-1.0..1.0);
    let accel_y = rng.gen_range(-1.0..1.0);
    let accel_z = rng.gen_range(8.8..9.8); // Approximate gravity
    
    // Generate random gyroscope data
    let gyro_x = rng.gen_range(-0.1..0.1);
    let gyro_y = rng.gen_range(-0.1..0.1);
    let gyro_z = rng.gen_range(-0.1..0.1);
    
    // Generate random magnetometer data
    let mag_x = rng.gen_range(-50.0..50.0);
    let mag_y = rng.gen_range(-50.0..50.0);
    let mag_z = rng.gen_range(-50.0..50.0);
    
    (accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z)
}

/// Generate a sequence of test IMU data samples
pub fn generate_test_imu_sequence(count: usize) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
    let mut samples = Vec::with_capacity(count);
    
    for _ in 0..count {
        samples.push(generate_test_imu_sample());
    }
    
    samples
}

/// Generate a test configuration
pub fn generate_test_config() -> String {
    r#"
    # Test Configuration
    
    [display]
    resolution = "1920x1080"
    refresh_rate = 90
    brightness = 80
    
    [camera]
    resolution = "1280x800"
    fps = 60
    
    [imu]
    sample_rate = 1000
    
    [audio]
    channels = 2
    sample_rate = 48000
    
    [network]
    wifi_enabled = true
    bluetooth_enabled = true
    
    [power]
    power_save_mode = "balanced"
    "#.to_string()
}

/// Parse a resolution string (e.g., "1920x1080") into a tuple
pub fn parse_resolution(resolution: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let width = parts[0].trim().parse::<u32>().ok()?;
    let height = parts[1].trim().parse::<u32>().ok()?;
    
    Some((width, height))
}

/// Format a resolution tuple as a string
pub fn format_resolution(resolution: (u32, u32)) -> String {
    format!("{}x{}", resolution.0, resolution.1)
}

/// Test logger that captures log messages
pub struct TestLogger {
    /// Log messages
    messages: Arc<Mutex<Vec<String>>>,
}

impl TestLogger {
    /// Create a new test logger
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Log a message
    pub fn log(&self, message: &str) {
        let mut messages = self.messages.lock().unwrap();
        messages.push(message.to_string());
    }
    
    /// Get all log messages
    pub fn messages(&self) -> Vec<String> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }
    
    /// Clear all log messages
    pub fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
    
    /// Check if a message was logged
    pub fn contains(&self, substring: &str) -> bool {
        let messages = self.messages.lock().unwrap();
        messages.iter().any(|m| m.contains(substring))
    }
    
    /// Count messages containing a substring
    pub fn count_containing(&self, substring: &str) -> usize {
        let messages = self.messages.lock().unwrap();
        messages.iter().filter(|m| m.contains(substring)).count()
    }
}

impl Clone for TestLogger {
    fn clone(&self) -> Self {
        Self {
            messages: Arc::clone(&self.messages),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_approx_eq() {
        assert!(assert_approx_eq(1.0, 1.0, 0.001));
        assert!(assert_approx_eq(1.0, 1.0001, 0.001));
        assert!(!assert_approx_eq(1.0, 1.01, 0.001));
    }

    #[test]
    fn test_assert_vec3_approx_eq() {
        assert!(assert_vec3_approx_eq((1.0, 2.0, 3.0), (1.0, 2.0, 3.0), 0.001));
        assert!(assert_vec3_approx_eq((1.0, 2.0, 3.0), (1.0001, 2.0001, 3.0001), 0.001));
        assert!(!assert_vec3_approx_eq((1.0, 2.0, 3.0), (1.01, 2.0, 3.0), 0.001));
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_run_with_timeout_success() {
        let result = run_with_timeout(|| {
            thread::sleep(Duration::from_millis(10));
            42
        }, Duration::from_millis(100));
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_run_with_timeout_failure() {
        let result = run_with_timeout(|| {
            thread::sleep(Duration::from_millis(100));
            42
        }, Duration::from_millis(10));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_resolution() {
        assert_eq!(parse_resolution("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_resolution("640x480"), Some((640, 480)));
        assert_eq!(parse_resolution("invalid"), None);
    }

    #[test]
    fn test_format_resolution() {
        assert_eq!(format_resolution((1920, 1080)), "1920x1080");
        assert_eq!(format_resolution((640, 480)), "640x480");
    }

    #[test]
    fn test_test_logger() {
        let logger = TestLogger::new();
        
        logger.log("Test message 1");
        logger.log("Test message 2");
        logger.log("Another message");
        
        assert_eq!(logger.messages().len(), 3);
        assert!(logger.contains("Test message"));
        assert_eq!(logger.count_containing("Test"), 2);
        
        logger.clear();
        assert_eq!(logger.messages().len(), 0);
    }
}
