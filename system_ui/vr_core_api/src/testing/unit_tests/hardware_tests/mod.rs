//! Hardware unit tests module for the VR headset system.
//!
//! This module contains unit tests for the hardware components of the VR headset system,
//! including display, camera, IMU, audio, storage, and network devices.

use crate::testing::{Test, TestCategory, TestEnvironment, TestResult, TestStatus};
use crate::testing::fixtures::{TestFixture, SimulationTestFixture, HardwareTestFixture, CombinedTestFixture};
use crate::testing::mocks::{MockDevice, MockDisplayDevice, MockCameraDevice, MockImuDevice};
use crate::testing::utils::{assert_approx_eq, assert_vec3_approx_eq, measure_time, TestLogger};
use crate::testing::hardware::{HardwareDeviceType, HardwareTestEnvironment};
use crate::testing::simulation::{SimulatedDeviceType, SimulationTestEnvironment};
use crate::testing::unit_tests::UnitTest;

use crate::hardware::device::{Device, DeviceType, DeviceState, DeviceInfo, DeviceError};
use crate::hardware::display::{DisplayDevice, DisplayMode, DisplayState};
use crate::hardware::audio::{AudioDevice, AudioMode, AudioState};
use crate::hardware::tracking::{TrackingDevice, TrackingMode, TrackingState};
use crate::hardware::power::{PowerDevice, PowerMode, PowerState};
use crate::hardware::storage::{StorageDevice, StorageMode, StorageState};
use crate::hardware::network::{NetworkDevice, NetworkMode, NetworkState};

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Add hardware tests to the test suite
pub fn add_tests(suite: &mut crate::testing::TestSuite) {
    // Add display device tests
    add_display_tests(suite);
    
    // Add camera device tests
    add_camera_tests(suite);
    
    // Add IMU device tests
    add_imu_tests(suite);
    
    // Add audio device tests
    add_audio_tests(suite);
    
    // Add storage device tests
    add_storage_tests(suite);
    
    // Add network device tests
    add_network_tests(suite);
    
    // Add device manager tests
    add_device_manager_tests(suite);
}

/// Add display device tests to the test suite
fn add_display_tests(suite: &mut crate::testing::TestSuite) {
    // Simulation test for display initialization
    let sim_fixture = SimulationTestFixture::new("display_init_sim");
    let display_init_sim_test = UnitTest::new(
        "display_init_simulation",
        "Test display device initialization in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the display device
            let display_device = sim_env.device_manager().get_device("display");
            assert!(display_device.is_some(), "Display device not found");
            
            // Check that the display is initialized
            assert!(display_device.unwrap().is_initialized(), "Display device not initialized");
            
            // Create test result
            TestResult::new(
                "display_init_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Display device initialized successfully in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(display_init_sim_test);
    
    // Simulation test for display power on/off
    let sim_fixture = SimulationTestFixture::new("display_power_sim");
    let display_power_sim_test = UnitTest::new(
        "display_power_simulation",
        "Test display device power on/off in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the display device
            let device_manager = sim_env.device_manager_mut();
            let display_device = device_manager.get_device_mut("display");
            assert!(display_device.is_some(), "Display device not found");
            
            // Get the display device as SimulatedDisplayDevice
            let display = display_device.unwrap().as_any_mut().downcast_mut::<crate::testing::simulation::SimulatedDisplayDevice>();
            assert!(display.is_some(), "Failed to cast to SimulatedDisplayDevice");
            let display = display.unwrap();
            
            // Power on the display
            assert!(display.power_on().is_ok(), "Failed to power on display");
            assert!(display.is_powered_on(), "Display should be powered on");
            
            // Power off the display
            assert!(display.power_off().is_ok(), "Failed to power off display");
            assert!(!display.is_powered_on(), "Display should be powered off");
            
            // Create test result
            TestResult::new(
                "display_power_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Display device power on/off successful in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(display_power_sim_test);
    
    // Simulation test for display resolution change
    let sim_fixture = SimulationTestFixture::new("display_resolution_sim");
    let display_resolution_sim_test = UnitTest::new(
        "display_resolution_simulation",
        "Test display device resolution change in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the display device
            let device_manager = sim_env.device_manager_mut();
            let display_device = device_manager.get_device_mut("display");
            assert!(display_device.is_some(), "Display device not found");
            
            // Get the display device as SimulatedDisplayDevice
            let display = display_device.unwrap().as_any_mut().downcast_mut::<crate::testing::simulation::SimulatedDisplayDevice>();
            assert!(display.is_some(), "Failed to cast to SimulatedDisplayDevice");
            let display = display.unwrap();
            
            // Check initial resolution
            let initial_resolution = display.resolution();
            assert_eq!(initial_resolution, (1920, 1080), "Unexpected initial resolution");
            
            // Change resolution
            assert!(display.set_resolution(2560, 1440).is_ok(), "Failed to change resolution");
            
            // Check new resolution
            let new_resolution = display.resolution();
            assert_eq!(new_resolution, (2560, 1440), "Resolution change failed");
            
            // Create test result
            TestResult::new(
                "display_resolution_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Display device resolution change successful in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(display_resolution_sim_test);
    
    // Hardware test for display detection (if available)
    let hw_fixture = HardwareTestFixture::new("display_detection_hw");
    let display_detection_hw_test = UnitTest::new(
        "display_detection_hardware",
        "Test display device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "display_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if display device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Display) {
                return TestResult::new(
                    "display_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Display device not available on hardware",
                    0,
                );
            }
            
            // Get display devices
            let display_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Display);
            assert!(!display_devices.is_empty(), "No display devices found");
            
            // Create test result
            TestResult::new(
                "display_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("Display device detected on hardware: {}", display_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(display_detection_hw_test);
}

/// Add camera device tests to the test suite
fn add_camera_tests(suite: &mut crate::testing::TestSuite) {
    // Simulation test for camera initialization
    let sim_fixture = SimulationTestFixture::new("camera_init_sim");
    let camera_init_sim_test = UnitTest::new(
        "camera_init_simulation",
        "Test camera device initialization in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the camera device
            let camera_device = sim_env.device_manager().get_device("camera");
            assert!(camera_device.is_some(), "Camera device not found");
            
            // Check that the camera is initialized
            assert!(camera_device.unwrap().is_initialized(), "Camera device not initialized");
            
            // Create test result
            TestResult::new(
                "camera_init_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Camera device initialized successfully in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(camera_init_sim_test);
    
    // Simulation test for camera streaming
    let sim_fixture = SimulationTestFixture::new("camera_streaming_sim");
    let camera_streaming_sim_test = UnitTest::new(
        "camera_streaming_simulation",
        "Test camera device streaming in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the camera device
            let device_manager = sim_env.device_manager_mut();
            let camera_device = device_manager.get_device_mut("camera");
            assert!(camera_device.is_some(), "Camera device not found");
            
            // Get the camera device as SimulatedCameraDevice
            let camera = camera_device.unwrap().as_any_mut().downcast_mut::<crate::testing::simulation::SimulatedCameraDevice>();
            assert!(camera.is_some(), "Failed to cast to SimulatedCameraDevice");
            let camera = camera.unwrap();
            
            // Start streaming
            assert!(camera.start_streaming().is_ok(), "Failed to start streaming");
            assert!(camera.is_streaming(), "Camera should be streaming");
            
            // Get a frame
            let frame = camera.get_frame();
            assert!(frame.is_ok(), "Failed to get frame");
            assert!(!frame.unwrap().is_empty(), "Frame should not be empty");
            
            // Stop streaming
            assert!(camera.stop_streaming().is_ok(), "Failed to stop streaming");
            assert!(!camera.is_streaming(), "Camera should not be streaming");
            
            // Create test result
            TestResult::new(
                "camera_streaming_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Camera device streaming successful in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(camera_streaming_sim_test);
    
    // Hardware test for camera detection (if available)
    let hw_fixture = HardwareTestFixture::new("camera_detection_hw");
    let camera_detection_hw_test = UnitTest::new(
        "camera_detection_hardware",
        "Test camera device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "camera_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if camera device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Camera) {
                return TestResult::new(
                    "camera_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Camera device not available on hardware",
                    0,
                );
            }
            
            // Get camera devices
            let camera_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Camera);
            assert!(!camera_devices.is_empty(), "No camera devices found");
            
            // Create test result
            TestResult::new(
                "camera_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("Camera device detected on hardware: {}", camera_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(camera_detection_hw_test);
}

/// Add IMU device tests to the test suite
fn add_imu_tests(suite: &mut crate::testing::TestSuite) {
    // Simulation test for IMU initialization
    let sim_fixture = SimulationTestFixture::new("imu_init_sim");
    let imu_init_sim_test = UnitTest::new(
        "imu_init_simulation",
        "Test IMU device initialization in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the IMU device
            let imu_device = sim_env.device_manager().get_device("imu");
            assert!(imu_device.is_some(), "IMU device not found");
            
            // Check that the IMU is initialized
            assert!(imu_device.unwrap().is_initialized(), "IMU device not initialized");
            
            // Create test result
            TestResult::new(
                "imu_init_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "IMU device initialized successfully in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(imu_init_sim_test);
    
    // Simulation test for IMU data acquisition
    let sim_fixture = SimulationTestFixture::new("imu_data_sim");
    let imu_data_sim_test = UnitTest::new(
        "imu_data_simulation",
        "Test IMU device data acquisition in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the IMU device
            let device_manager = sim_env.device_manager_mut();
            let imu_device = device_manager.get_device_mut("imu");
            assert!(imu_device.is_some(), "IMU device not found");
            
            // Get the IMU device as SimulatedImuDevice
            let imu = imu_device.unwrap().as_any_mut().downcast_mut::<crate::testing::simulation::SimulatedImuDevice>();
            assert!(imu.is_some(), "Failed to cast to SimulatedImuDevice");
            let imu = imu.unwrap();
            
            // Start streaming
            assert!(imu.start_streaming().is_ok(), "Failed to start streaming");
            assert!(imu.is_streaming(), "IMU should be streaming");
            
            // Get a sample
            let sample = imu.get_sample();
            assert!(sample.is_ok(), "Failed to get sample");
            
            // Check that the sample has reasonable values
            let (accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z) = sample.unwrap();
            
            // Acceleration should be around 9.81 m/s^2 in the z direction (gravity)
            assert_approx_eq(accel_z, 9.81, 1.0), "Unexpected acceleration in z direction";
            
            // Stop streaming
            assert!(imu.stop_streaming().is_ok(), "Failed to stop streaming");
            assert!(!imu.is_streaming(), "IMU should not be streaming");
            
            // Create test result
            TestResult::new(
                "imu_data_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "IMU device data acquisition successful in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(imu_data_sim_test);
    
    // Hardware test for IMU detection (if available)
    let hw_fixture = HardwareTestFixture::new("imu_detection_hw");
    let imu_detection_hw_test = UnitTest::new(
        "imu_detection_hardware",
        "Test IMU device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "imu_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if IMU device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Imu) {
                return TestResult::new(
                    "imu_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "IMU device not available on hardware",
                    0,
                );
            }
            
            // Get IMU devices
            let imu_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Imu);
            assert!(!imu_devices.is_empty(), "No IMU devices found");
            
            // Create test result
            TestResult::new(
                "imu_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("IMU device detected on hardware: {}", imu_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(imu_detection_hw_test);
}

/// Add audio device tests to the test suite
fn add_audio_tests(suite: &mut crate::testing::TestSuite) {
    // Hardware test for audio detection (if available)
    let hw_fixture = HardwareTestFixture::new("audio_detection_hw");
    let audio_detection_hw_test = UnitTest::new(
        "audio_detection_hardware",
        "Test audio device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "audio_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if audio device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Audio) {
                return TestResult::new(
                    "audio_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Audio device not available on hardware",
                    0,
                );
            }
            
            // Get audio devices
            let audio_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Audio);
            assert!(!audio_devices.is_empty(), "No audio devices found");
            
            // Create test result
            TestResult::new(
                "audio_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("Audio device detected on hardware: {}", audio_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(audio_detection_hw_test);
}

/// Add storage device tests to the test suite
fn add_storage_tests(suite: &mut crate::testing::TestSuite) {
    // Hardware test for storage detection (if available)
    let hw_fixture = HardwareTestFixture::new("storage_detection_hw");
    let storage_detection_hw_test = UnitTest::new(
        "storage_detection_hardware",
        "Test storage device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "storage_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if storage device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Storage) {
                return TestResult::new(
                    "storage_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Storage device not available on hardware",
                    0,
                );
            }
            
            // Get storage devices
            let storage_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Storage);
            assert!(!storage_devices.is_empty(), "No storage devices found");
            
            // Create test result
            TestResult::new(
                "storage_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("Storage device detected on hardware: {}", storage_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(storage_detection_hw_test);
}

/// Add network device tests to the test suite
fn add_network_tests(suite: &mut crate::testing::TestSuite) {
    // Hardware test for network detection (if available)
    let hw_fixture = HardwareTestFixture::new("network_detection_hw");
    let network_detection_hw_test = UnitTest::new(
        "network_detection_hardware",
        "Test network device detection on hardware",
        TestEnvironment::Hardware,
        hw_fixture,
        |fixture| {
            // Set up hardware environment
            let mut hw_env = HardwareTestEnvironment::new();
            if hw_env.initialize().is_err() {
                return TestResult::new(
                    "network_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Hardware environment initialization failed",
                    0,
                );
            }
            
            // Check if network device is available
            if !hw_env.is_device_type_available(HardwareDeviceType::Network) {
                return TestResult::new(
                    "network_detection_hardware",
                    TestCategory::Unit,
                    TestEnvironment::Hardware,
                    TestStatus::Skipped,
                    "Network device not available on hardware",
                    0,
                );
            }
            
            // Get network devices
            let network_devices = hw_env.detector().devices_of_type(HardwareDeviceType::Network);
            assert!(!network_devices.is_empty(), "No network devices found");
            
            // Create test result
            TestResult::new(
                "network_detection_hardware",
                TestCategory::Unit,
                TestEnvironment::Hardware,
                TestStatus::Passed,
                format!("Network device detected on hardware: {}", network_devices[0].name),
                0,
            )
        },
        100,
    );
    suite.add_test(network_detection_hw_test);
}

/// Add device manager tests to the test suite
fn add_device_manager_tests(suite: &mut crate::testing::TestSuite) {
    // Simulation test for device manager
    let sim_fixture = SimulationTestFixture::new("device_manager_sim");
    let device_manager_sim_test = UnitTest::new(
        "device_manager_simulation",
        "Test device manager in simulation",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Set up simulation environment
            let mut sim_env = SimulationTestEnvironment::new();
            assert!(sim_env.initialize().is_ok(), "Failed to initialize simulation environment");
            
            // Get the device manager
            let device_manager = sim_env.device_manager();
            
            // Check that all expected devices are present
            assert!(device_manager.get_device("display").is_some(), "Display device not found");
            assert!(device_manager.get_device("camera").is_some(), "Camera device not found");
            assert!(device_manager.get_device("imu").is_some(), "IMU device not found");
            
            // Create test result
            TestResult::new(
                "device_manager_simulation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Device manager test successful in simulation",
                0,
            )
        },
        100,
    );
    suite.add_test(device_manager_sim_test);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_device_simulation() {
        // Set up simulation environment
        let mut sim_env = SimulationTestEnvironment::new();
        assert!(sim_env.initialize().is_ok());
        
        // Get the display device
        let device_manager = sim_env.device_manager_mut();
        let display_device = device_manager.get_device_mut("display");
        assert!(display_device.is_some());
        
        // Get the display device as SimulatedDisplayDevice
        let display = display_device.unwrap().as_any_mut().downcast_mut::<crate::testing::simulation::SimulatedDisplayDevice>();
        assert!(display.is_some());
        let display = display.unwrap();
        
        // Test power on/off
        assert!(display.power_on().is_ok());
        assert!(display.is_powered_on());
        assert!(display.power_off().is_ok());
        assert!(!display.is_powered_on());
    }
}
