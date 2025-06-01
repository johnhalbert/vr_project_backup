//! Integration tests for the Core API Layer.
//!
//! This module provides comprehensive integration tests for all subsystems
//! of the Core API Layer, ensuring they work together seamlessly.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, warn};
use tempfile::tempdir;

use crate::config::{ConfigManager, ConfigProfile};
use crate::hardware::{HardwareManager, DeviceType};
use crate::monitoring::{MetricsCollector, PerformanceMonitor};
use crate::ipc::{IpcServer, UnixSocketServer};
use crate::security::{SecurityManager, tls::TlsConfig, audit::EventStatus};

/// Test the integration of all Core API Layer subsystems.
#[test]
fn test_core_api_integration() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let test_dir = temp_dir.path().to_path_buf();
    
    // Initialize the configuration manager
    let config_dir = test_dir.join("config");
    let config_manager = ConfigManager::new(config_dir.clone())
        .expect("Failed to create configuration manager");
    
    // Initialize the hardware manager
    let hardware_manager = HardwareManager::new()
        .expect("Failed to create hardware manager");
    
    // Initialize the monitoring system
    let metrics_collector = MetricsCollector::new()
        .expect("Failed to create metrics collector");
    
    let performance_monitor = PerformanceMonitor::new(&metrics_collector)
        .expect("Failed to create performance monitor");
    
    // Initialize the IPC system
    let ipc_dir = test_dir.join("ipc");
    let ipc_server = UnixSocketServer::new(ipc_dir.join("core_api.sock"))
        .expect("Failed to create IPC server");
    
    // Initialize the security manager
    let security_dir = test_dir.join("security");
    let mut security_manager = SecurityManager::new(security_dir.clone())
        .expect("Failed to create security manager");
    
    // Initialize TLS
    security_manager.init_tls()
        .expect("Failed to initialize TLS");
    
    // Initialize audit logging
    security_manager.init_audit()
        .expect("Failed to initialize audit logging");
    
    // Initialize authentication
    security_manager.init_auth()
        .expect("Failed to initialize authentication");
    
    // Test configuration subsystem
    test_configuration_subsystem(&config_manager);
    
    // Test hardware subsystem
    test_hardware_subsystem(&hardware_manager);
    
    // Test monitoring subsystem
    test_monitoring_subsystem(&metrics_collector, &performance_monitor);
    
    // Test IPC subsystem
    test_ipc_subsystem(&ipc_server);
    
    // Test security subsystem
    test_security_subsystem(&security_manager);
    
    // Test cross-subsystem integration
    test_cross_subsystem_integration(
        &config_manager,
        &hardware_manager,
        &metrics_collector,
        &ipc_server,
        &security_manager,
    );
}

/// Test the configuration subsystem.
fn test_configuration_subsystem(config_manager: &ConfigManager) {
    // Create a test profile
    let profile = ConfigProfile::new("test_profile", "Test Profile");
    config_manager.create_profile(&profile)
        .expect("Failed to create profile");
    
    // Set a configuration value
    config_manager.set_value("test_profile", "test.key", "test_value")
        .expect("Failed to set configuration value");
    
    // Get the configuration value
    let value = config_manager.get_value("test_profile", "test.key")
        .expect("Failed to get configuration value");
    
    assert_eq!(value, "test_value");
    
    // Create a backup
    config_manager.create_backup("test_backup")
        .expect("Failed to create backup");
    
    // Delete the profile
    config_manager.delete_profile("test_profile")
        .expect("Failed to delete profile");
    
    // Restore from backup
    config_manager.restore_backup("test_backup")
        .expect("Failed to restore backup");
    
    // Verify the profile was restored
    let profiles = config_manager.get_profiles()
        .expect("Failed to get profiles");
    
    assert!(profiles.iter().any(|p| p.name == "test_profile"));
}

/// Test the hardware subsystem.
fn test_hardware_subsystem(hardware_manager: &HardwareManager) {
    // Get available devices
    let devices = hardware_manager.get_available_devices()
        .expect("Failed to get available devices");
    
    // Check if we can get device information
    for device in &devices {
        let info = hardware_manager.get_device_info(device.id())
            .expect("Failed to get device info");
        
        assert_eq!(info.id, device.id());
    }
    
    // Test device discovery
    hardware_manager.discover_devices()
        .expect("Failed to discover devices");
    
    // Test device type filtering
    let cameras = hardware_manager.get_devices_by_type(DeviceType::Camera)
        .expect("Failed to get cameras");
    
    let displays = hardware_manager.get_devices_by_type(DeviceType::Display)
        .expect("Failed to get displays");
    
    let imus = hardware_manager.get_devices_by_type(DeviceType::IMU)
        .expect("Failed to get IMUs");
    
    // Devices should be properly categorized
    for device in &cameras {
        assert_eq!(device.device_type(), DeviceType::Camera);
    }
    
    for device in &displays {
        assert_eq!(device.device_type(), DeviceType::Display);
    }
    
    for device in &imus {
        assert_eq!(device.device_type(), DeviceType::IMU);
    }
}

/// Test the monitoring subsystem.
fn test_monitoring_subsystem(
    metrics_collector: &MetricsCollector,
    performance_monitor: &PerformanceMonitor,
) {
    // Collect system metrics
    metrics_collector.collect_all()
        .expect("Failed to collect metrics");
    
    // Get CPU usage
    let cpu_usage = performance_monitor.get_cpu_usage()
        .expect("Failed to get CPU usage");
    
    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0);
    
    // Get memory usage
    let memory_usage = performance_monitor.get_memory_usage()
        .expect("Failed to get memory usage");
    
    assert!(memory_usage >= 0.0 && memory_usage <= 100.0);
    
    // Get storage usage
    let storage_usage = performance_monitor.get_storage_usage("/")
        .expect("Failed to get storage usage");
    
    assert!(storage_usage >= 0.0 && storage_usage <= 100.0);
    
    // Test metrics history
    metrics_collector.record_metric("test.metric", 42.0)
        .expect("Failed to record metric");
    
    std::thread::sleep(Duration::from_millis(100));
    
    metrics_collector.record_metric("test.metric", 43.0)
        .expect("Failed to record metric");
    
    let history = metrics_collector.get_metric_history("test.metric")
        .expect("Failed to get metric history");
    
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].value, 42.0);
    assert_eq!(history[1].value, 43.0);
}

/// Test the IPC subsystem.
fn test_ipc_subsystem(ipc_server: &UnixSocketServer) {
    // Start the server
    ipc_server.start()
        .expect("Failed to start IPC server");
    
    // Register a handler
    ipc_server.register_handler("test", |message| {
        if message == "ping" {
            Ok("pong".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }).expect("Failed to register handler");
    
    // Create a client
    let client = ipc_server.create_client()
        .expect("Failed to create IPC client");
    
    // Send a message
    let response = client.send_message("test", "ping")
        .expect("Failed to send message");
    
    assert_eq!(response, "pong");
    
    // Stop the server
    ipc_server.stop()
        .expect("Failed to stop IPC server");
}

/// Test the security subsystem.
fn test_security_subsystem(security_manager: &SecurityManager) {
    // Test TLS
    if let Some(tls_manager) = security_manager.tls_manager() {
        // Generate a self-signed certificate
        let cert_info = tls_manager.certificate_manager().generate_self_signed(
            "test",
            "localhost",
            Some("Test Organization"),
            &["localhost", "127.0.0.1"],
            365,
        ).expect("Failed to generate certificate");
        
        assert_eq!(cert_info.name, "test");
        
        // Get the certificate
        let cert = tls_manager.certificate_manager().get_certificate("test")
            .expect("Failed to get certificate");
        
        assert_eq!(cert.info.name, "test");
        
        // Create a server configuration
        let server_config = tls_manager.create_server_config()
            .expect("Failed to create server config");
    }
    
    // Test audit logging
    if let Some(audit_logger) = security_manager.audit_logger() {
        // Log a security event
        audit_logger.log_security_event(
            audit::SecurityEventType::Authentication,
            Some("test_user"),
            Some("test_session"),
            EventStatus::Success,
            Some("Test authentication"),
            Some("127.0.0.1"),
        ).expect("Failed to log security event");
        
        // Query events
        let events = audit_logger.get_security_events()
            .expect("Failed to get security events");
        
        assert!(!events.is_empty());
        
        let event = &events[0];
        assert_eq!(event.user_id.as_deref(), Some("test_user"));
        assert_eq!(event.session_id.as_deref(), Some("test_session"));
        assert_eq!(event.status, EventStatus::Success);
    }
    
    // Test authentication
    if let Some(auth_manager) = security_manager.auth_manager() {
        // Create a test user
        let user = auth_manager.create_user(
            "test_user",
            "Password123!",
            Some("Test User"),
            vec!["user".to_string()],
            true,
        ).expect("Failed to create user");
        
        assert_eq!(user.username, "test_user");
        
        // Authenticate the user
        let session = auth_manager.authenticate("test_user", "Password123!")
            .expect("Failed to authenticate user");
        
        assert_eq!(session.user_id, user.id);
        
        // Validate the session
        let validated_session = auth_manager.validate_session(&session.id)
            .expect("Failed to validate session");
        
        assert_eq!(validated_session.id, session.id);
        
        // Check permissions
        let has_permission = auth_manager.has_permission(&user.id, "user.read")
            .expect("Failed to check permission");
        
        assert!(has_permission);
        
        // Invalidate the session
        auth_manager.invalidate_session(&session.id)
            .expect("Failed to invalidate session");
    }
}

/// Test cross-subsystem integration.
fn test_cross_subsystem_integration(
    config_manager: &ConfigManager,
    hardware_manager: &HardwareManager,
    metrics_collector: &MetricsCollector,
    ipc_server: &UnixSocketServer,
    security_manager: &SecurityManager,
) {
    // Test hardware configuration integration
    let profile = ConfigProfile::new("hardware_test", "Hardware Test");
    config_manager.create_profile(&profile)
        .expect("Failed to create profile");
    
    // Configure a device
    let devices = hardware_manager.get_available_devices()
        .expect("Failed to get available devices");
    
    if !devices.is_empty() {
        let device_id = devices[0].id();
        config_manager.set_value("hardware_test", &format!("device.{}.enabled", device_id), "true")
            .expect("Failed to set device configuration");
        
        // Verify the configuration
        let value = config_manager.get_value("hardware_test", &format!("device.{}.enabled", device_id))
            .expect("Failed to get device configuration");
        
        assert_eq!(value, "true");
    }
    
    // Test monitoring and security integration
    if let Some(audit_logger) = security_manager.audit_logger() {
        // Log a system event
        audit_logger.log_system_event(
            audit::SystemEventType::PerformanceIssue,
            EventStatus::Warning,
            Some("High CPU usage detected"),
        ).expect("Failed to log system event");
        
        // Record a performance metric
        metrics_collector.record_metric("system.cpu.usage", 90.0)
            .expect("Failed to record metric");
        
        // Query events
        let events = audit_logger.get_system_events()
            .expect("Failed to get system events");
        
        assert!(!events.is_empty());
        
        let event = &events[0];
        assert_eq!(event.status, EventStatus::Warning);
    }
    
    // Test IPC and security integration
    if let Some(auth_manager) = security_manager.auth_manager() {
        // Start the IPC server
        ipc_server.start()
            .expect("Failed to start IPC server");
        
        // Register a secure handler
        ipc_server.register_handler("secure", move |message| {
            // Parse the message (format: "session_id:command")
            let parts: Vec<&str> = message.split(':').collect();
            
            if parts.len() != 2 {
                return Ok("Invalid message format".to_string());
            }
            
            let session_id = parts[0];
            let command = parts[1];
            
            // Validate the session
            match auth_manager.validate_session(session_id) {
                Ok(session) => {
                    // Check if the user has permission
                    match auth_manager.has_permission(&session.user_id, "system.read") {
                        Ok(true) => {
                            // Process the command
                            match command {
                                "status" => Ok("System status: OK".to_string()),
                                _ => Ok("Unknown command".to_string()),
                            }
                        },
                        Ok(false) => Ok("Permission denied".to_string()),
                        Err(_) => Ok("Error checking permissions".to_string()),
                    }
                },
                Err(_) => Ok("Invalid session".to_string()),
            }
        }).expect("Failed to register secure handler");
        
        // Stop the IPC server
        ipc_server.stop()
            .expect("Failed to stop IPC server");
    }
}

/// Test the full Core API Layer integration.
#[test]
fn test_full_core_api_integration() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let test_dir = temp_dir.path().to_path_buf();
    
    // Initialize all subsystems
    let config_dir = test_dir.join("config");
    let config_manager = ConfigManager::new(config_dir.clone())
        .expect("Failed to create configuration manager");
    
    let hardware_manager = HardwareManager::new()
        .expect("Failed to create hardware manager");
    
    let metrics_collector = MetricsCollector::new()
        .expect("Failed to create metrics collector");
    
    let ipc_dir = test_dir.join("ipc");
    let ipc_server = UnixSocketServer::new(ipc_dir.join("core_api.sock"))
        .expect("Failed to create IPC server");
    
    let security_dir = test_dir.join("security");
    let mut security_manager = SecurityManager::new(security_dir.clone())
        .expect("Failed to create security manager");
    
    security_manager.init_tls().expect("Failed to initialize TLS");
    security_manager.init_audit().expect("Failed to initialize audit logging");
    security_manager.init_auth().expect("Failed to initialize authentication");
    
    // Start the IPC server
    ipc_server.start()
        .expect("Failed to start IPC server");
    
    // Create a test user
    let auth_manager = security_manager.auth_manager().unwrap();
    let user = auth_manager.create_user(
        "test_user",
        "Password123!",
        Some("Test User"),
        vec!["user".to_string()],
        true,
    ).expect("Failed to create user");
    
    // Authenticate the user
    let session = auth_manager.authenticate("test_user", "Password123!")
        .expect("Failed to authenticate user");
    
    // Register IPC handlers for all subsystems
    let config_manager_clone = Arc::new(config_manager);
    let hardware_manager_clone = Arc::new(hardware_manager);
    let metrics_collector_clone = Arc::new(metrics_collector);
    let security_manager_clone = Arc::new(security_manager);
    
    // Configuration handler
    let config_manager_ref = Arc::clone(&config_manager_clone);
    let auth_manager_ref = auth_manager;
    ipc_server.register_handler("config", move |message| {
        // Parse the message (format: "session_id:command:args")
        let parts: Vec<&str> = message.split(':').collect();
        
        if parts.len() < 2 {
            return Ok("Invalid message format".to_string());
        }
        
        let session_id = parts[0];
        let command = parts[1];
        
        // Validate the session
        match auth_manager_ref.validate_session(session_id) {
            Ok(session) => {
                // Process the command
                match command {
                    "get_profiles" => {
                        match auth_manager_ref.has_permission(&session.user_id, "config.read") {
                            Ok(true) => {
                                match config_manager_ref.get_profiles() {
                                    Ok(profiles) => {
                                        let names: Vec<String> = profiles.iter()
                                            .map(|p| p.name.clone())
                                            .collect();
                                        Ok(names.join(","))
                                    },
                                    Err(e) => Ok(format!("Error: {}", e)),
                                }
                            },
                            _ => Ok("Permission denied".to_string()),
                        }
                    },
                    "get_value" => {
                        if parts.len() < 4 {
                            return Ok("Invalid arguments".to_string());
                        }
                        
                        let profile = parts[2];
                        let key = parts[3];
                        
                        match auth_manager_ref.has_permission(&session.user_id, "config.read") {
                            Ok(true) => {
                                match config_manager_ref.get_value(profile, key) {
                                    Ok(value) => Ok(value),
                                    Err(e) => Ok(format!("Error: {}", e)),
                                }
                            },
                            _ => Ok("Permission denied".to_string()),
                        }
                    },
                    _ => Ok("Unknown command".to_string()),
                }
            },
            Err(_) => Ok("Invalid session".to_string()),
        }
    }).expect("Failed to register config handler");
    
    // Hardware handler
    let hardware_manager_ref = Arc::clone(&hardware_manager_clone);
    let auth_manager_ref = auth_manager;
    ipc_server.register_handler("hardware", move |message| {
        // Parse the message
        let parts: Vec<&str> = message.split(':').collect();
        
        if parts.len() < 2 {
            return Ok("Invalid message format".to_string());
        }
        
        let session_id = parts[0];
        let command = parts[1];
        
        // Validate the session
        match auth_manager_ref.validate_session(session_id) {
            Ok(session) => {
                // Process the command
                match command {
                    "get_devices" => {
                        match auth_manager_ref.has_permission(&session.user_id, "hardware.read") {
                            Ok(true) => {
                                match hardware_manager_ref.get_available_devices() {
                                    Ok(devices) => {
                                        let ids: Vec<String> = devices.iter()
                                            .map(|d| d.id().to_string())
                                            .collect();
                                        Ok(ids.join(","))
                                    },
                                    Err(e) => Ok(format!("Error: {}", e)),
                                }
                            },
                            _ => Ok("Permission denied".to_string()),
                        }
                    },
                    _ => Ok("Unknown command".to_string()),
                }
            },
            Err(_) => Ok("Invalid session".to_string()),
        }
    }).expect("Failed to register hardware handler");
    
    // Create a client
    let client = ipc_server.create_client()
        .expect("Failed to create IPC client");
    
    // Test the configuration handler
    let response = client.send_message("config", &format!("{}:get_profiles", session.id))
        .expect("Failed to send message");
    
    // The response should contain at least the default profile
    assert!(response.contains("default"));
    
    // Test the hardware handler
    let response = client.send_message("hardware", &format!("{}:get_devices", session.id))
        .expect("Failed to send message");
    
    // The response should be a comma-separated list of device IDs or an empty string
    assert!(response.is_empty() || response.contains(',') || !response.starts_with("Error"));
    
    // Stop the IPC server
    ipc_server.stop()
        .expect("Failed to stop IPC server");
    
    // Clean up
    auth_manager.invalidate_session(&session.id)
        .expect("Failed to invalidate session");
}
