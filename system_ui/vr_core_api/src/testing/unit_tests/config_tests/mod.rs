//! Configuration unit tests module for the VR headset system.
//!
//! This module contains unit tests for the configuration components of the VR headset system,
//! including schema validation, versioning, profiles, and defaults.

use crate::testing::{Test, TestCategory, TestEnvironment, TestResult, TestStatus};
use crate::testing::fixtures::{TestFixture, SimulationTestFixture, HardwareTestFixture, CombinedTestFixture};
use crate::testing::mocks::{MockDevice, MockDisplayDevice, MockCameraDevice, MockImuDevice};
use crate::testing::utils::{assert_approx_eq, assert_vec3_approx_eq, measure_time, TestLogger};
use crate::testing::unit_tests::UnitTest;

use crate::config::schema::{ConfigSchema, SchemaField, SchemaType, SchemaValidationError};
use crate::config::validation::{ConfigValidator, ValidationRule, ValidationResult};
use crate::config::versioning::{ConfigVersion, VersionedConfig, MigrationResult};
use crate::config::profiles::{ConfigProfile, ProfileManager};
use crate::config::defaults::{DefaultConfig, DefaultsManager};
use crate::config::hardware::{HardwareConfig, DisplayConfig, CameraConfig, AudioConfig};
use crate::config::network::{NetworkConfig, WifiConfig, BluetoothConfig};
use crate::config::system::{SystemConfig, PerformanceConfig, AccessibilityConfig};
use crate::config::user::{UserConfig, UserProfile, NotificationSettings};

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};

/// Add configuration tests to the test suite
pub fn add_tests(suite: &mut crate::testing::TestSuite) {
    // Add schema tests
    add_schema_tests(suite);
    
    // Add validation tests
    add_validation_tests(suite);
    
    // Add versioning tests
    add_versioning_tests(suite);
    
    // Add profile tests
    add_profile_tests(suite);
    
    // Add defaults tests
    add_defaults_tests(suite);
    
    // Add hardware config tests
    add_hardware_config_tests(suite);
    
    // Add network config tests
    add_network_config_tests(suite);
    
    // Add system config tests
    add_system_config_tests(suite);
    
    // Add user config tests
    add_user_config_tests(suite);
}

/// Add schema tests to the test suite
fn add_schema_tests(suite: &mut crate::testing::TestSuite) {
    // Test schema validation with valid data
    let sim_fixture = SimulationTestFixture::new("schema_valid_sim");
    let schema_valid_test = UnitTest::new(
        "schema_validation_valid",
        "Test schema validation with valid data",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a schema
            let schema = ConfigSchema::new(
                "test_schema",
                vec![
                    SchemaField::new("string_field", SchemaType::String, true),
                    SchemaField::new("int_field", SchemaType::Integer, true),
                    SchemaField::new("float_field", SchemaType::Float, false),
                    SchemaField::new("bool_field", SchemaType::Boolean, true),
                ],
            );
            
            // Create valid data
            let mut data = HashMap::new();
            data.insert("string_field".to_string(), "test".to_string());
            data.insert("int_field".to_string(), "42".to_string());
            data.insert("bool_field".to_string(), "true".to_string());
            
            // Validate the data
            let result = schema.validate(&data);
            assert!(result.is_ok(), "Schema validation failed: {:?}", result.err());
            
            // Create test result
            TestResult::new(
                "schema_validation_valid",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Schema validation with valid data successful",
                0,
            )
        },
        100,
    );
    suite.add_test(schema_valid_test);
    
    // Test schema validation with invalid data
    let sim_fixture = SimulationTestFixture::new("schema_invalid_sim");
    let schema_invalid_test = UnitTest::new(
        "schema_validation_invalid",
        "Test schema validation with invalid data",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a schema
            let schema = ConfigSchema::new(
                "test_schema",
                vec![
                    SchemaField::new("string_field", SchemaType::String, true),
                    SchemaField::new("int_field", SchemaType::Integer, true),
                    SchemaField::new("float_field", SchemaType::Float, false),
                    SchemaField::new("bool_field", SchemaType::Boolean, true),
                ],
            );
            
            // Create invalid data (missing required field)
            let mut data = HashMap::new();
            data.insert("string_field".to_string(), "test".to_string());
            data.insert("int_field".to_string(), "42".to_string());
            // Missing bool_field
            
            // Validate the data
            let result = schema.validate(&data);
            assert!(result.is_err(), "Schema validation should fail");
            
            // Check error type
            match result.err().unwrap() {
                SchemaValidationError::MissingRequiredField(field) => {
                    assert_eq!(field, "bool_field", "Unexpected missing field");
                }
                err => {
                    panic!("Unexpected error type: {:?}", err);
                }
            }
            
            // Create test result
            TestResult::new(
                "schema_validation_invalid",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Schema validation with invalid data correctly failed",
                0,
            )
        },
        100,
    );
    suite.add_test(schema_invalid_test);
    
    // Test schema validation with type mismatch
    let sim_fixture = SimulationTestFixture::new("schema_type_mismatch_sim");
    let schema_type_mismatch_test = UnitTest::new(
        "schema_validation_type_mismatch",
        "Test schema validation with type mismatch",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a schema
            let schema = ConfigSchema::new(
                "test_schema",
                vec![
                    SchemaField::new("string_field", SchemaType::String, true),
                    SchemaField::new("int_field", SchemaType::Integer, true),
                    SchemaField::new("float_field", SchemaType::Float, false),
                    SchemaField::new("bool_field", SchemaType::Boolean, true),
                ],
            );
            
            // Create invalid data (type mismatch)
            let mut data = HashMap::new();
            data.insert("string_field".to_string(), "test".to_string());
            data.insert("int_field".to_string(), "not_an_integer".to_string());
            data.insert("bool_field".to_string(), "true".to_string());
            
            // Validate the data
            let result = schema.validate(&data);
            assert!(result.is_err(), "Schema validation should fail");
            
            // Check error type
            match result.err().unwrap() {
                SchemaValidationError::TypeMismatch(field, expected, _) => {
                    assert_eq!(field, "int_field", "Unexpected field with type mismatch");
                    assert_eq!(expected, SchemaType::Integer, "Unexpected expected type");
                }
                err => {
                    panic!("Unexpected error type: {:?}", err);
                }
            }
            
            // Create test result
            TestResult::new(
                "schema_validation_type_mismatch",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Schema validation with type mismatch correctly failed",
                0,
            )
        },
        100,
    );
    suite.add_test(schema_type_mismatch_test);
}

/// Add validation tests to the test suite
fn add_validation_tests(suite: &mut crate::testing::TestSuite) {
    // Test config validator with valid data
    let sim_fixture = SimulationTestFixture::new("validator_valid_sim");
    let validator_valid_test = UnitTest::new(
        "config_validator_valid",
        "Test config validator with valid data",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a validator with rules
            let mut validator = ConfigValidator::new();
            
            // Add a range rule for an integer field
            validator.add_rule(ValidationRule::IntRange("int_field".to_string(), 0, 100));
            
            // Add a string length rule
            validator.add_rule(ValidationRule::StringLength("string_field".to_string(), 1, 10));
            
            // Add a regex pattern rule
            validator.add_rule(ValidationRule::Pattern("pattern_field".to_string(), r"^[a-z]+$".to_string()));
            
            // Create valid data
            let mut data = HashMap::new();
            data.insert("int_field".to_string(), "42".to_string());
            data.insert("string_field".to_string(), "test".to_string());
            data.insert("pattern_field".to_string(), "abcdef".to_string());
            
            // Validate the data
            let result = validator.validate(&data);
            assert!(result.is_ok(), "Validation failed: {:?}", result.err());
            
            // Create test result
            TestResult::new(
                "config_validator_valid",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Config validator with valid data successful",
                0,
            )
        },
        100,
    );
    suite.add_test(validator_valid_test);
    
    // Test config validator with invalid data
    let sim_fixture = SimulationTestFixture::new("validator_invalid_sim");
    let validator_invalid_test = UnitTest::new(
        "config_validator_invalid",
        "Test config validator with invalid data",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a validator with rules
            let mut validator = ConfigValidator::new();
            
            // Add a range rule for an integer field
            validator.add_rule(ValidationRule::IntRange("int_field".to_string(), 0, 100));
            
            // Add a string length rule
            validator.add_rule(ValidationRule::StringLength("string_field".to_string(), 1, 10));
            
            // Add a regex pattern rule
            validator.add_rule(ValidationRule::Pattern("pattern_field".to_string(), r"^[a-z]+$".to_string()));
            
            // Create invalid data (out of range)
            let mut data = HashMap::new();
            data.insert("int_field".to_string(), "200".to_string()); // Out of range
            data.insert("string_field".to_string(), "test".to_string());
            data.insert("pattern_field".to_string(), "abcdef".to_string());
            
            // Validate the data
            let result = validator.validate(&data);
            assert!(result.is_err(), "Validation should fail");
            
            // Check error type
            match result.err().unwrap() {
                ValidationResult::IntOutOfRange(field, min, max, value) => {
                    assert_eq!(field, "int_field", "Unexpected field with range error");
                    assert_eq!(min, 0, "Unexpected minimum value");
                    assert_eq!(max, 100, "Unexpected maximum value");
                    assert_eq!(value, 200, "Unexpected actual value");
                }
                err => {
                    panic!("Unexpected error type: {:?}", err);
                }
            }
            
            // Create test result
            TestResult::new(
                "config_validator_invalid",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Config validator with invalid data correctly failed",
                0,
            )
        },
        100,
    );
    suite.add_test(validator_invalid_test);
}

/// Add versioning tests to the test suite
fn add_versioning_tests(suite: &mut crate::testing::TestSuite) {
    // Test config versioning with migration
    let sim_fixture = SimulationTestFixture::new("versioning_migration_sim");
    let versioning_migration_test = UnitTest::new(
        "config_versioning_migration",
        "Test config versioning with migration",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a versioned config
            let mut versioned_config = VersionedConfig::new("test_config", ConfigVersion::new(1, 0, 0));
            
            // Add a migration from v1.0.0 to v2.0.0
            versioned_config.add_migration(
                ConfigVersion::new(1, 0, 0),
                ConfigVersion::new(2, 0, 0),
                Box::new(|config: &mut HashMap<String, String>| {
                    // Rename a field
                    if let Some(value) = config.remove("old_field") {
                        config.insert("new_field".to_string(), value);
                    }
                    
                    // Add a new field
                    config.insert("added_field".to_string(), "default_value".to_string());
                    
                    Ok(())
                }),
            );
            
            // Create v1.0.0 config data
            let mut data = HashMap::new();
            data.insert("old_field".to_string(), "test_value".to_string());
            data.insert("unchanged_field".to_string(), "unchanged_value".to_string());
            
            // Migrate to v2.0.0
            let result = versioned_config.migrate(&mut data, ConfigVersion::new(2, 0, 0));
            assert!(result.is_ok(), "Migration failed: {:?}", result.err());
            
            // Check migrated data
            assert!(!data.contains_key("old_field"), "Old field should be removed");
            assert!(data.contains_key("new_field"), "New field should be added");
            assert_eq!(data.get("new_field").unwrap(), "test_value", "New field should have the old field's value");
            assert!(data.contains_key("added_field"), "Added field should be present");
            assert_eq!(data.get("added_field").unwrap(), "default_value", "Added field should have the default value");
            assert!(data.contains_key("unchanged_field"), "Unchanged field should still be present");
            assert_eq!(data.get("unchanged_field").unwrap(), "unchanged_value", "Unchanged field should have the same value");
            
            // Create test result
            TestResult::new(
                "config_versioning_migration",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Config versioning with migration successful",
                0,
            )
        },
        100,
    );
    suite.add_test(versioning_migration_test);
}

/// Add profile tests to the test suite
fn add_profile_tests(suite: &mut crate::testing::TestSuite) {
    // Test profile manager
    let sim_fixture = SimulationTestFixture::new("profile_manager_sim");
    let profile_manager_test = UnitTest::new(
        "profile_manager",
        "Test profile manager",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a profile manager
            let mut profile_manager = ProfileManager::new();
            
            // Create profiles
            let mut default_profile = ConfigProfile::new("default", "Default Profile");
            default_profile.set("display.resolution", "1920x1080");
            default_profile.set("display.refresh_rate", "90");
            
            let mut performance_profile = ConfigProfile::new("performance", "Performance Profile");
            performance_profile.set("display.resolution", "1280x720");
            performance_profile.set("display.refresh_rate", "120");
            
            let mut quality_profile = ConfigProfile::new("quality", "Quality Profile");
            quality_profile.set("display.resolution", "2560x1440");
            quality_profile.set("display.refresh_rate", "60");
            
            // Add profiles
            profile_manager.add_profile(default_profile);
            profile_manager.add_profile(performance_profile);
            profile_manager.add_profile(quality_profile);
            
            // Set active profile
            profile_manager.set_active_profile("performance");
            
            // Check active profile
            let active_profile = profile_manager.active_profile();
            assert!(active_profile.is_some(), "Active profile should be set");
            assert_eq!(active_profile.unwrap().name(), "performance", "Unexpected active profile");
            
            // Get a setting from the active profile
            let resolution = profile_manager.get("display.resolution");
            assert!(resolution.is_some(), "Setting should be present");
            assert_eq!(resolution.unwrap(), "1280x720", "Unexpected resolution value");
            
            // Switch to another profile
            profile_manager.set_active_profile("quality");
            
            // Check new active profile
            let active_profile = profile_manager.active_profile();
            assert!(active_profile.is_some(), "Active profile should be set");
            assert_eq!(active_profile.unwrap().name(), "quality", "Unexpected active profile");
            
            // Get a setting from the new active profile
            let resolution = profile_manager.get("display.resolution");
            assert!(resolution.is_some(), "Setting should be present");
            assert_eq!(resolution.unwrap(), "2560x1440", "Unexpected resolution value");
            
            // Create test result
            TestResult::new(
                "profile_manager",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Profile manager test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(profile_manager_test);
}

/// Add defaults tests to the test suite
fn add_defaults_tests(suite: &mut crate::testing::TestSuite) {
    // Test defaults manager
    let sim_fixture = SimulationTestFixture::new("defaults_manager_sim");
    let defaults_manager_test = UnitTest::new(
        "defaults_manager",
        "Test defaults manager",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a defaults manager
            let mut defaults_manager = DefaultsManager::new();
            
            // Set default values
            defaults_manager.set_default("display.resolution", "1920x1080");
            defaults_manager.set_default("display.refresh_rate", "90");
            defaults_manager.set_default("audio.volume", "80");
            
            // Get default values
            let resolution = defaults_manager.get_default("display.resolution");
            assert!(resolution.is_some(), "Default value should be present");
            assert_eq!(resolution.unwrap(), "1920x1080", "Unexpected default resolution");
            
            let refresh_rate = defaults_manager.get_default("display.refresh_rate");
            assert!(resolution.is_some(), "Default value should be present");
            assert_eq!(refresh_rate.unwrap(), "90", "Unexpected default refresh rate");
            
            let volume = defaults_manager.get_default("audio.volume");
            assert!(volume.is_some(), "Default value should be present");
            assert_eq!(volume.unwrap(), "80", "Unexpected default volume");
            
            // Get non-existent default
            let non_existent = defaults_manager.get_default("non.existent");
            assert!(non_existent.is_none(), "Non-existent default should not be present");
            
            // Create a config with missing values
            let mut config = HashMap::new();
            config.insert("display.resolution".to_string(), "2560x1440".to_string());
            // Missing refresh_rate and volume
            
            // Apply defaults
            defaults_manager.apply_defaults(&mut config);
            
            // Check that defaults were applied
            assert!(config.contains_key("display.resolution"), "Config should contain resolution");
            assert_eq!(config.get("display.resolution").unwrap(), "2560x1440", "Resolution should not be changed");
            
            assert!(config.contains_key("display.refresh_rate"), "Config should contain refresh rate");
            assert_eq!(config.get("display.refresh_rate").unwrap(), "90", "Refresh rate should be set to default");
            
            assert!(config.contains_key("audio.volume"), "Config should contain volume");
            assert_eq!(config.get("audio.volume").unwrap(), "80", "Volume should be set to default");
            
            // Create test result
            TestResult::new(
                "defaults_manager",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Defaults manager test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(defaults_manager_test);
}

/// Add hardware config tests to the test suite
fn add_hardware_config_tests(suite: &mut crate::testing::TestSuite) {
    // Test hardware config
    let sim_fixture = SimulationTestFixture::new("hardware_config_sim");
    let hardware_config_test = UnitTest::new(
        "hardware_config",
        "Test hardware configuration",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a hardware config
            let mut hardware_config = HardwareConfig::new();
            
            // Set display config
            let mut display_config = DisplayConfig::new();
            display_config.set_resolution(1920, 1080);
            display_config.set_refresh_rate(90);
            display_config.set_brightness(80);
            hardware_config.set_display_config(display_config);
            
            // Set camera config
            let mut camera_config = CameraConfig::new();
            camera_config.set_resolution(1280, 720);
            camera_config.set_fps(60);
            camera_config.set_exposure(50);
            hardware_config.set_camera_config(camera_config);
            
            // Set audio config
            let mut audio_config = AudioConfig::new();
            audio_config.set_volume(70);
            audio_config.set_sample_rate(48000);
            audio_config.set_channels(2);
            hardware_config.set_audio_config(audio_config);
            
            // Check display config
            let display = hardware_config.display_config();
            assert_eq!(display.resolution(), (1920, 1080), "Unexpected display resolution");
            assert_eq!(display.refresh_rate(), 90, "Unexpected display refresh rate");
            assert_eq!(display.brightness(), 80, "Unexpected display brightness");
            
            // Check camera config
            let camera = hardware_config.camera_config();
            assert_eq!(camera.resolution(), (1280, 720), "Unexpected camera resolution");
            assert_eq!(camera.fps(), 60, "Unexpected camera fps");
            assert_eq!(camera.exposure(), 50, "Unexpected camera exposure");
            
            // Check audio config
            let audio = hardware_config.audio_config();
            assert_eq!(audio.volume(), 70, "Unexpected audio volume");
            assert_eq!(audio.sample_rate(), 48000, "Unexpected audio sample rate");
            assert_eq!(audio.channels(), 2, "Unexpected audio channels");
            
            // Create test result
            TestResult::new(
                "hardware_config",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Hardware configuration test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(hardware_config_test);
}

/// Add network config tests to the test suite
fn add_network_config_tests(suite: &mut crate::testing::TestSuite) {
    // Test network config
    let sim_fixture = SimulationTestFixture::new("network_config_sim");
    let network_config_test = UnitTest::new(
        "network_config",
        "Test network configuration",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a network config
            let mut network_config = NetworkConfig::new();
            
            // Set WiFi config
            let mut wifi_config = WifiConfig::new();
            wifi_config.set_enabled(true);
            wifi_config.set_ssid("Test_Network");
            wifi_config.set_security_type("WPA2");
            wifi_config.set_password("password123");
            network_config.set_wifi_config(wifi_config);
            
            // Set Bluetooth config
            let mut bluetooth_config = BluetoothConfig::new();
            bluetooth_config.set_enabled(true);
            bluetooth_config.set_discoverable(true);
            bluetooth_config.set_name("VR_Headset");
            network_config.set_bluetooth_config(bluetooth_config);
            
            // Check WiFi config
            let wifi = network_config.wifi_config();
            assert!(wifi.enabled(), "WiFi should be enabled");
            assert_eq!(wifi.ssid(), "Test_Network", "Unexpected WiFi SSID");
            assert_eq!(wifi.security_type(), "WPA2", "Unexpected WiFi security type");
            assert_eq!(wifi.password(), "password123", "Unexpected WiFi password");
            
            // Check Bluetooth config
            let bluetooth = network_config.bluetooth_config();
            assert!(bluetooth.enabled(), "Bluetooth should be enabled");
            assert!(bluetooth.discoverable(), "Bluetooth should be discoverable");
            assert_eq!(bluetooth.name(), "VR_Headset", "Unexpected Bluetooth name");
            
            // Create test result
            TestResult::new(
                "network_config",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Network configuration test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(network_config_test);
}

/// Add system config tests to the test suite
fn add_system_config_tests(suite: &mut crate::testing::TestSuite) {
    // Test system config
    let sim_fixture = SimulationTestFixture::new("system_config_sim");
    let system_config_test = UnitTest::new(
        "system_config",
        "Test system configuration",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a system config
            let mut system_config = SystemConfig::new();
            
            // Set performance config
            let mut performance_config = PerformanceConfig::new();
            performance_config.set_power_mode("balanced");
            performance_config.set_cpu_boost_enabled(true);
            performance_config.set_gpu_performance_level(2);
            system_config.set_performance_config(performance_config);
            
            // Set accessibility config
            let mut accessibility_config = AccessibilityConfig::new();
            accessibility_config.set_high_contrast(false);
            accessibility_config.set_text_size(1.0);
            accessibility_config.set_color_correction("none");
            system_config.set_accessibility_config(accessibility_config);
            
            // Check performance config
            let performance = system_config.performance_config();
            assert_eq!(performance.power_mode(), "balanced", "Unexpected power mode");
            assert!(performance.cpu_boost_enabled(), "CPU boost should be enabled");
            assert_eq!(performance.gpu_performance_level(), 2, "Unexpected GPU performance level");
            
            // Check accessibility config
            let accessibility = system_config.accessibility_config();
            assert!(!accessibility.high_contrast(), "High contrast should be disabled");
            assert_eq!(accessibility.text_size(), 1.0, "Unexpected text size");
            assert_eq!(accessibility.color_correction(), "none", "Unexpected color correction");
            
            // Create test result
            TestResult::new(
                "system_config",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "System configuration test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(system_config_test);
}

/// Add user config tests to the test suite
fn add_user_config_tests(suite: &mut crate::testing::TestSuite) {
    // Test user config
    let sim_fixture = SimulationTestFixture::new("user_config_sim");
    let user_config_test = UnitTest::new(
        "user_config",
        "Test user configuration",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a user config
            let mut user_config = UserConfig::new();
            
            // Set user profile
            let mut user_profile = UserProfile::new();
            user_profile.set_username("testuser");
            user_profile.set_display_name("Test User");
            user_profile.set_avatar("default_avatar.png");
            user_config.set_user_profile(user_profile);
            
            // Set notification settings
            let mut notification_settings = NotificationSettings::new();
            notification_settings.set_enabled(true);
            notification_settings.set_sound_enabled(true);
            notification_settings.set_vibration_enabled(false);
            notification_settings.set_priority_only(false);
            user_config.set_notification_settings(notification_settings);
            
            // Check user profile
            let profile = user_config.user_profile();
            assert_eq!(profile.username(), "testuser", "Unexpected username");
            assert_eq!(profile.display_name(), "Test User", "Unexpected display name");
            assert_eq!(profile.avatar(), "default_avatar.png", "Unexpected avatar");
            
            // Check notification settings
            let notifications = user_config.notification_settings();
            assert!(notifications.enabled(), "Notifications should be enabled");
            assert!(notifications.sound_enabled(), "Notification sound should be enabled");
            assert!(!notifications.vibration_enabled(), "Notification vibration should be disabled");
            assert!(!notifications.priority_only(), "Priority only mode should be disabled");
            
            // Create test result
            TestResult::new(
                "user_config",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "User configuration test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(user_config_test);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_validation() {
        // Create a schema
        let schema = ConfigSchema::new(
            "test_schema",
            vec![
                SchemaField::new("string_field", SchemaType::String, true),
                SchemaField::new("int_field", SchemaType::Integer, true),
                SchemaField::new("float_field", SchemaType::Float, false),
                SchemaField::new("bool_field", SchemaType::Boolean, true),
            ],
        );
        
        // Create valid data
        let mut data = HashMap::new();
        data.insert("string_field".to_string(), "test".to_string());
        data.insert("int_field".to_string(), "42".to_string());
        data.insert("bool_field".to_string(), "true".to_string());
        
        // Validate the data
        let result = schema.validate(&data);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_config_validator() {
        // Create a validator with rules
        let mut validator = ConfigValidator::new();
        
        // Add a range rule for an integer field
        validator.add_rule(ValidationRule::IntRange("int_field".to_string(), 0, 100));
        
        // Create valid data
        let mut data = HashMap::new();
        data.insert("int_field".to_string(), "42".to_string());
        
        // Validate the data
        let result = validator.validate(&data);
        assert!(result.is_ok());
    }
}
