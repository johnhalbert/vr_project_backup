# Core API Developer Guide

## Introduction

This guide provides detailed information for developers who want to work with the VR headset's Core API Layer. The Core API Layer is the foundation of the VR headset system, providing interfaces for hardware access, configuration management, inter-process communication (IPC), security, and system services.

This guide assumes you are already familiar with the general concepts covered in the main Developer Guide and focuses specifically on working with the Core API components.

## Core API Architecture

The Core API is structured as a modular Rust library with several key components:

```
/system_ui/vr_core_api/
├── src/
│   ├── config/         # Configuration management
│   ├── hardware/       # Hardware access interfaces
│   ├── ipc/            # Inter-process communication
│   ├── monitoring/     # System monitoring
│   ├── optimization/   # Performance optimization
│   ├── security/       # Security and authentication
│   ├── telemetry/      # Telemetry and logging
│   ├── update/         # Update system
│   ├── factory_reset/  # Factory reset capability
│   ├── validation/     # Validation suite
│   └── lib.rs          # Library entry point
```

Each module provides a specific set of functionality and can be used independently or in combination with other modules.

## Getting Started with Core API Development

### Setting Up Your Development Environment

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/vrheadset/vr_core_api.git
   cd vr_core_api
   ```

2. **Install Rust and Dependencies**:
   ```bash
   # Install Rust using rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install additional dependencies
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev libusb-1.0-0-dev libudev-dev
   ```

3. **Build the Core API**:
   ```bash
   cargo build
   ```

4. **Run Tests**:
   ```bash
   cargo test
   ```

### Project Structure

The Core API follows a modular architecture with clear separation of concerns:

- Each module has a `mod.rs` file that defines the public interface
- Implementation details are contained in separate files
- Public types and functions are clearly marked with `pub`
- Error types are defined for each module
- Unit tests are included alongside the code

### Example: Using the Core API in Your Application

```rust
use vr_core_api::hardware::{DeviceManager, DeviceType};
use vr_core_api::config::ConfigManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the device manager
    let mut device_manager = DeviceManager::new()?;
    
    // Discover available devices
    device_manager.discover_devices()?;
    
    // Get a list of display devices
    let displays = device_manager.get_devices_by_type(DeviceType::Display)?;
    
    // Initialize the configuration manager
    let config_manager = ConfigManager::new()?;
    
    // Load the display configuration
    let display_config = config_manager.get_config("hardware.display")?;
    
    // Configure the first display
    if let Some(display) = displays.first() {
        display.configure(&display_config)?;
        println!("Configured display: {}", display.get_info().name);
    }
    
    Ok(())
}
```

## Hardware Access API

The Hardware Access API provides interfaces for interacting with the VR headset's hardware components.

### Key Concepts

- **Device Abstraction**: All hardware devices implement the `Device` trait
- **Device Manager**: Central registry for discovering and accessing devices
- **Device Events**: Event system for hardware state changes
- **Device Configuration**: Configuration interface for hardware devices

### Device Types

The Core API supports the following device types:

- **Display**: VR display panels and controllers
- **Audio**: Speakers, microphones, and spatial audio
- **Tracking**: IMU, cameras, and tracking systems
- **Power**: Battery management and power states
- **Storage**: Internal and external storage
- **Network**: WiFi, Bluetooth, and other connectivity

### Example: Implementing a Custom Device

```rust
use vr_core_api::hardware::{Device, DeviceInfo, DeviceType, DeviceError};
use std::sync::{Arc, Mutex};

struct MyCustomDevice {
    info: DeviceInfo,
    state: Arc<Mutex<DeviceState>>,
}

struct DeviceState {
    is_active: bool,
    temperature: f32,
}

impl Device for MyCustomDevice {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }
    
    fn initialize(&mut self) -> Result<(), DeviceError> {
        // Initialize the device hardware
        println!("Initializing device: {}", self.info.name);
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), DeviceError> {
        // Shutdown the device hardware
        println!("Shutting down device: {}", self.info.name);
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        // Check if the device is available
        true
    }
}

// Implementation of device-specific functionality
impl MyCustomDevice {
    pub fn new(id: String, name: String) -> Self {
        let info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Custom("MyDevice".to_string()),
            vendor: "My Company".to_string(),
            model: "Custom Device 1.0".to_string(),
        };
        
        let state = DeviceState {
            is_active: false,
            temperature: 25.0,
        };
        
        MyCustomDevice {
            info,
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    pub fn set_active(&mut self, active: bool) -> Result<(), DeviceError> {
        let mut state = self.state.lock().unwrap();
        state.is_active = active;
        // Actual hardware control would happen here
        Ok(())
    }
    
    pub fn get_temperature(&self) -> Result<f32, DeviceError> {
        let state = self.state.lock().unwrap();
        // Actual hardware reading would happen here
        Ok(state.temperature)
    }
}
```

### Registering a Custom Device

```rust
use vr_core_api::hardware::DeviceManager;

fn register_custom_device() -> Result<(), Box<dyn std::error::Error>> {
    let mut device_manager = DeviceManager::new()?;
    
    let mut custom_device = MyCustomDevice::new(
        "custom1".to_string(),
        "My Custom Device".to_string()
    );
    
    // Initialize the device
    custom_device.initialize()?;
    
    // Register with the device manager
    device_manager.register_device(Box::new(custom_device))?;
    
    println!("Custom device registered successfully");
    Ok(())
}
```

## Configuration Management

The Configuration Management module provides a centralized system for storing and retrieving configuration data.

### Key Concepts

- **Configuration Schema**: Defines the structure and validation rules for configuration data
- **Configuration Storage**: Persists configuration data to disk
- **Configuration Versioning**: Handles migration between different configuration versions
- **Profile Management**: Supports multiple user profiles with different configurations

### Configuration Format

The Core API uses TOML as the configuration format:

```toml
# Example configuration file: hardware.toml

[display]
brightness = 0.8
refresh_rate = 90
resolution = { width = 2880, height = 1600 }

[audio]
volume = 0.5
spatial_audio = true
microphone_gain = 0.7

[tracking]
prediction_ms = 15
smoothing = 0.2
```

### Example: Working with Configuration

```rust
use vr_core_api::config::{ConfigManager, ConfigValue, ConfigError};
use serde::{Serialize, Deserialize};

// Define a configuration structure
#[derive(Serialize, Deserialize)]
struct DisplayConfig {
    brightness: f32,
    refresh_rate: u32,
    resolution: Resolution,
}

#[derive(Serialize, Deserialize)]
struct Resolution {
    width: u32,
    height: u32,
}

fn configure_display() -> Result<(), ConfigError> {
    // Create or get the configuration manager
    let mut config_manager = ConfigManager::new()?;
    
    // Load the display configuration
    let display_config: DisplayConfig = config_manager.get_typed("hardware.display")?;
    
    println!("Current display configuration:");
    println!("  Brightness: {}", display_config.brightness);
    println!("  Refresh Rate: {}", display_config.refresh_rate);
    println!("  Resolution: {}x{}", 
             display_config.resolution.width, 
             display_config.resolution.height);
    
    // Modify the configuration
    let mut updated_config = display_config;
    updated_config.brightness = 0.7;
    updated_config.refresh_rate = 120;
    
    // Save the updated configuration
    config_manager.set_typed("hardware.display", &updated_config)?;
    
    // Commit changes to disk
    config_manager.save()?;
    
    println!("Display configuration updated successfully");
    Ok(())
}
```

### Creating a Configuration Schema

```rust
use vr_core_api::config::{SchemaBuilder, ConfigType, ConfigError};

fn create_display_schema() -> Result<(), ConfigError> {
    let mut schema_builder = SchemaBuilder::new();
    
    // Define the schema for display configuration
    schema_builder.add_field("brightness", ConfigType::Float)
        .set_description("Display brightness level")
        .set_range(0.0, 1.0)
        .set_default(0.8);
    
    schema_builder.add_field("refresh_rate", ConfigType::Integer)
        .set_description("Display refresh rate in Hz")
        .set_allowed_values(&[60, 72, 90, 120, 144])
        .set_default(90);
    
    schema_builder.add_object("resolution")
        .add_field("width", ConfigType::Integer)
            .set_description("Display width in pixels")
            .set_range(1024, 8192)
            .set_default(2880)
        .add_field("height", ConfigType::Integer)
            .set_description("Display height in pixels")
            .set_range(1024, 8192)
            .set_default(1600);
    
    // Register the schema
    let schema = schema_builder.build();
    let mut config_manager = ConfigManager::new()?;
    config_manager.register_schema("hardware.display", schema)?;
    
    println!("Display configuration schema registered successfully");
    Ok(())
}
```

## Inter-Process Communication (IPC)

The IPC module provides mechanisms for communication between different processes in the VR headset system.

### Key Concepts

- **Message Format**: Standardized format for all IPC messages
- **Transport Mechanisms**: Multiple transport options (Unix sockets, D-Bus, WebSockets)
- **Service Discovery**: Mechanism for finding available services
- **Error Handling**: Robust error handling for IPC failures

### IPC Transports

The Core API supports the following IPC transports:

- **Unix Sockets**: High-performance local communication
- **D-Bus**: Integration with system services
- **WebSockets**: Remote communication and web interfaces

### Example: Creating an IPC Server

```rust
use vr_core_api::ipc::unix_socket::{UnixSocketServer, ConnectionHandler};
use vr_core_api::ipc::common::{Message, MessageType, IpcError};
use std::sync::{Arc, Mutex};

struct MyHandler {
    connection_count: Arc<Mutex<usize>>,
}

impl ConnectionHandler for MyHandler {
    fn handle_message(&mut self, message: Message) -> Result<Message, IpcError> {
        match message.message_type {
            MessageType::Request => {
                println!("Received request: {}", message.payload);
                
                // Process the request
                let response_payload = format!("Processed: {}", message.payload);
                
                // Create a response message
                Ok(Message {
                    message_type: MessageType::Response,
                    id: message.id,
                    payload: response_payload,
                })
            },
            _ => Err(IpcError::UnsupportedMessageType),
        }
    }
    
    fn on_connect(&mut self) {
        let mut count = self.connection_count.lock().unwrap();
        *count += 1;
        println!("New connection. Total connections: {}", *count);
    }
    
    fn on_disconnect(&mut self) {
        let mut count = self.connection_count.lock().unwrap();
        *count -= 1;
        println!("Connection closed. Total connections: {}", *count);
    }
}

fn create_ipc_server() -> Result<(), IpcError> {
    // Create a connection handler
    let handler = MyHandler {
        connection_count: Arc::new(Mutex::new(0)),
    };
    
    // Create a Unix socket server
    let mut server = UnixSocketServer::new("/tmp/my_service.sock", handler)?;
    
    // Start the server (this will block and handle connections)
    server.start()?;
    
    Ok(())
}
```

### Example: Creating an IPC Client

```rust
use vr_core_api::ipc::unix_socket::UnixSocketClient;
use vr_core_api::ipc::common::{Message, MessageType, IpcError};

fn create_ipc_client() -> Result<(), IpcError> {
    // Connect to the server
    let mut client = UnixSocketClient::connect("/tmp/my_service.sock")?;
    
    // Create a request message
    let request = Message {
        message_type: MessageType::Request,
        id: "req1".to_string(),
        payload: "Hello, server!".to_string(),
    };
    
    // Send the request and wait for response
    let response = client.send_and_receive(request)?;
    
    println!("Received response: {}", response.payload);
    
    // Close the connection
    client.close()?;
    
    Ok(())
}
```

### Creating a D-Bus Service

```rust
use vr_core_api::ipc::dbus::{DbusService, DbusInterface, DbusObject, DbusError};

fn create_dbus_service() -> Result<(), DbusError> {
    // Create a D-Bus interface
    let mut interface = DbusInterface::new("com.vrheadset.MyService")?;
    
    // Add methods to the interface
    interface.add_method("GetStatus", |args| {
        // Process the method call
        println!("GetStatus called with args: {:?}", args);
        
        // Return the status
        Ok(vec!["Running".to_string()])
    })?;
    
    interface.add_method("SetConfiguration", |args| {
        // Process the method call
        println!("SetConfiguration called with args: {:?}", args);
        
        // Apply the configuration
        // ...
        
        // Return success
        Ok(vec!["Success".to_string()])
    })?;
    
    // Create a D-Bus object
    let mut object = DbusObject::new("/com/vrheadset/MyService")?;
    object.add_interface(interface)?;
    
    // Create a D-Bus service
    let mut service = DbusService::new("com.vrheadset.MyService")?;
    service.add_object(object)?;
    
    // Start the service (this will block and handle method calls)
    service.start()?;
    
    Ok(())
}
```

## Security System

The Security System module provides authentication, authorization, and encryption services for the VR headset system.

### Key Concepts

- **Authentication**: Verifying the identity of users and applications
- **Authorization**: Controlling access to resources based on permissions
- **Encryption**: Protecting sensitive data at rest and in transit
- **Secure Storage**: Safely storing sensitive information
- **Audit Logging**: Recording security-relevant events

### Example: Implementing Authentication

```rust
use vr_core_api::security::authentication::{AuthManager, Credentials, AuthError};
use vr_core_api::security::authorization::{Permission, Resource};

fn authenticate_user() -> Result<(), AuthError> {
    // Create an authentication manager
    let auth_manager = AuthManager::new()?;
    
    // Authenticate a user
    let credentials = Credentials::new("username", "password");
    let token = auth_manager.authenticate(credentials)?;
    
    println!("User authenticated successfully");
    println!("Token: {}", token.to_string());
    println!("Expires: {}", token.expiration());
    
    // Check if the token has permission to access a resource
    let resource = Resource::new("display", "settings");
    let permission = Permission::Write;
    
    if auth_manager.check_permission(&token, &resource, permission)? {
        println!("User has permission to write display settings");
    } else {
        println!("User does not have permission to write display settings");
    }
    
    Ok(())
}
```

### Example: Encrypting Sensitive Data

```rust
use vr_core_api::security::encryption::{EncryptionManager, EncryptionError};

fn encrypt_sensitive_data() -> Result<(), EncryptionError> {
    // Create an encryption manager
    let encryption_manager = EncryptionManager::new()?;
    
    // Encrypt sensitive data
    let sensitive_data = "This is sensitive information";
    let encrypted_data = encryption_manager.encrypt(sensitive_data.as_bytes())?;
    
    println!("Data encrypted successfully");
    
    // Decrypt the data
    let decrypted_data = encryption_manager.decrypt(&encrypted_data)?;
    let decrypted_string = String::from_utf8(decrypted_data)
        .map_err(|_| EncryptionError::InvalidData)?;
    
    println!("Decrypted data: {}", decrypted_string);
    
    Ok(())
}
```

### Example: Using Secure Storage

```rust
use vr_core_api::security::secure_storage::{SecureStorage, StorageError};

fn store_sensitive_data() -> Result<(), StorageError> {
    // Create a secure storage instance
    let secure_storage = SecureStorage::new()?;
    
    // Store sensitive data
    let key = "api_key";
    let value = "1234567890abcdef";
    
    secure_storage.store(key, value.as_bytes())?;
    println!("Data stored securely");
    
    // Retrieve the data
    let retrieved_data = secure_storage.retrieve(key)?;
    let retrieved_string = String::from_utf8(retrieved_data)
        .map_err(|_| StorageError::InvalidData)?;
    
    println!("Retrieved data: {}", retrieved_string);
    
    // Delete the data when no longer needed
    secure_storage.delete(key)?;
    println!("Data deleted from secure storage");
    
    Ok(())
}
```

## Telemetry and Logging

The Telemetry and Logging module provides mechanisms for collecting, processing, and analyzing system telemetry and logs.

### Key Concepts

- **Telemetry Collection**: Gathering system metrics and events
- **Privacy Controls**: Managing user opt-in/opt-out preferences
- **Data Anonymization**: Removing personally identifiable information
- **Log Rotation**: Managing log file size and retention
- **Log Forwarding**: Sending logs to remote systems
- **Log Analysis**: Analyzing logs for patterns and anomalies

### Example: Collecting Telemetry

```rust
use vr_core_api::telemetry::collection::{TelemetryCollector, TelemetryEvent, TelemetryError};
use vr_core_api::telemetry::privacy::PrivacyManager;

fn collect_telemetry() -> Result<(), TelemetryError> {
    // Check privacy settings first
    let privacy_manager = PrivacyManager::new()?;
    if !privacy_manager.is_telemetry_enabled()? {
        println!("Telemetry is disabled by user preference");
        return Ok(());
    }
    
    // Create a telemetry collector
    let mut collector = TelemetryCollector::new()?;
    
    // Collect system performance metrics
    collector.collect_system_metrics()?;
    
    // Record a specific event
    let event = TelemetryEvent::new("app_launched")
        .add_property("app_id", "com.example.myapp")
        .add_property("app_version", "1.0.0")
        .add_property("launch_time_ms", 1250);
    
    collector.record_event(event)?;
    
    // Submit telemetry data
    collector.submit()?;
    
    println!("Telemetry collected and submitted successfully");
    Ok(())
}
```

### Example: Configuring Log Rotation

```rust
use vr_core_api::telemetry::rotation::{LogRotationConfig, RotationTrigger, RotationError};

fn configure_log_rotation() -> Result<(), RotationError> {
    // Create a log rotation configuration
    let mut config = LogRotationConfig::new("/var/log/vrheadset.log")?;
    
    // Configure size-based rotation
    config.set_size_trigger(RotationTrigger::Size(10 * 1024 * 1024))?; // 10 MB
    
    // Configure time-based rotation
    config.set_time_trigger(RotationTrigger::Daily)?;
    
    // Configure compression
    config.set_compression(true)?;
    
    // Configure retention
    config.set_max_files(7)?; // Keep logs for a week
    
    // Apply the configuration
    config.apply()?;
    
    println!("Log rotation configured successfully");
    Ok(())
}
```

## Update System

The Update System module provides mechanisms for checking, downloading, verifying, and installing system updates.

### Key Concepts

- **Update Package**: Standard format for update packages
- **Update Checking**: Checking for available updates
- **Download Management**: Downloading update packages
- **Verification**: Verifying package integrity and authenticity
- **Installation**: Installing updates safely
- **Rollback**: Rolling back to previous versions if needed

### Example: Checking for Updates

```rust
use vr_core_api::update::checker::{UpdateChecker, UpdateInfo, CheckerError};

fn check_for_updates() -> Result<(), CheckerError> {
    // Create an update checker
    let checker = UpdateChecker::new()?;
    
    // Check for updates
    let updates = checker.check_for_updates()?;
    
    if updates.is_empty() {
        println!("No updates available");
        return Ok(());
    }
    
    // Display available updates
    println!("Available updates:");
    for update in &updates {
        println!("  - {} (v{})", update.name, update.version);
        println!("    Size: {} bytes", update.size);
        println!("    Description: {}", update.description);
        println!("    Priority: {}", update.priority);
    }
    
    Ok(())
}
```

### Example: Downloading and Installing Updates

```rust
use vr_core_api::update::downloader::{UpdateDownloader, DownloadProgress, DownloaderError};
use vr_core_api::update::installer::{UpdateInstaller, InstallationError};
use std::sync::mpsc;

fn download_and_install_update() -> Result<(), Box<dyn std::error::Error>> {
    // Create an update checker
    let checker = UpdateChecker::new()?;
    
    // Check for updates
    let updates = checker.check_for_updates()?;
    
    if updates.is_empty() {
        println!("No updates available");
        return Ok(());
    }
    
    // Get the first update
    let update = &updates[0];
    
    // Create a downloader
    let mut downloader = UpdateDownloader::new()?;
    
    // Set up progress reporting
    let (tx, rx) = mpsc::channel();
    downloader.set_progress_callback(move |progress: DownloadProgress| {
        tx.send(progress).unwrap();
    });
    
    // Start the download in a separate thread
    let update_id = update.id.clone();
    std::thread::spawn(move || {
        match downloader.download_update(&update_id) {
            Ok(_) => println!("Download completed successfully"),
            Err(e) => eprintln!("Download failed: {}", e),
        }
    });
    
    // Monitor download progress
    for progress in rx {
        println!("Download progress: {}%", progress.percentage);
        if progress.completed {
            break;
        }
    }
    
    // Create an installer
    let installer = UpdateInstaller::new()?;
    
    // Verify the update package
    installer.verify_package(&update.id)?;
    
    // Install the update
    installer.install_update(&update.id)?;
    
    println!("Update installed successfully");
    
    Ok(())
}
```

## Best Practices for Core API Development

### Code Organization

- **Modular Design**: Keep modules focused on a single responsibility
- **Clear Interfaces**: Define clear public interfaces for each module
- **Error Handling**: Use custom error types for each module
- **Documentation**: Document all public APIs with examples
- **Testing**: Write unit tests for all functionality

### Performance Considerations

- **Asynchronous APIs**: Use async/await for I/O-bound operations
- **Resource Management**: Properly manage resources with RAII patterns
- **Caching**: Cache expensive operations where appropriate
- **Batching**: Batch operations when possible
- **Profiling**: Profile your code to identify bottlenecks

### Security Considerations

- **Input Validation**: Validate all inputs, especially from external sources
- **Least Privilege**: Follow the principle of least privilege
- **Secure Defaults**: Provide secure defaults for all settings
- **Audit Logging**: Log security-relevant events
- **Regular Updates**: Keep dependencies updated

## Troubleshooting

### Common Issues

1. **Device Not Found**:
   - Check if the device is properly connected
   - Verify that the device driver is installed
   - Check permissions for device access

2. **Configuration Errors**:
   - Validate configuration against the schema
   - Check for syntax errors in TOML files
   - Verify file permissions for configuration files

3. **IPC Communication Failures**:
   - Check if the server is running
   - Verify socket permissions
   - Check for network connectivity issues

4. **Authentication Failures**:
   - Verify credentials
   - Check token expiration
   - Verify that the user has the required permissions

### Debugging Techniques

1. **Logging**:
   - Enable debug logging for detailed information
   - Check log files for error messages
   - Use log filtering to focus on relevant components

2. **Tracing**:
   - Use the `tracing` crate for detailed execution tracing
   - Set up span hierarchies for complex operations
   - Analyze trace output to identify issues

3. **Unit Testing**:
   - Write targeted tests for problematic functionality
   - Use mocks to isolate components
   - Test edge cases and error conditions

## Conclusion

The Core API Layer provides a solid foundation for building VR headset applications and services. By following the guidelines in this document, you can create robust, secure, and performant software that integrates seamlessly with the VR headset system.

For more information, refer to the API documentation and example code provided with the Core API.
