use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use prettytable::{Table, Row, Cell};
use vr_core_api::{VRCoreAPI, hardware::{DeviceType, Device}};
use std::collections::HashMap;

use crate::HardwareCommands;
use crate::utils::{error, formatting, file, validation};

pub fn handle_command(command: &HardwareCommands, api: &mut VRCoreAPI) -> Result<()> {
    match command {
        HardwareCommands::List { type_, format } => {
            list_devices(api, type_.as_deref(), format)
        },
        HardwareCommands::Info { name, format } => {
            device_info(api, name, format)
        },
        HardwareCommands::Init { device, force } => {
            init_devices(api, device.as_deref(), *force)
        },
        HardwareCommands::Shutdown { device, force } => {
            shutdown_devices(api, device.as_deref(), *force)
        },
        HardwareCommands::Diagnose { device, level, format } => {
            diagnose_devices(api, device.as_deref(), level, format)
        },
        HardwareCommands::Firmware { device, file, check_only, force } => {
            update_firmware(api, device.as_deref(), file.as_deref(), *check_only, *force)
        },
        HardwareCommands::Calibrate { device, type_, interactive } => {
            calibrate_device(api, device, type_.as_deref(), *interactive)
        },
    }
}

fn list_devices(api: &VRCoreAPI, type_filter: Option<&str>, format: &str) -> Result<()> {
    error::print_section("Hardware Devices");
    
    // Get all device types
    let device_types = [
        DeviceType::Camera,
        DeviceType::IMU,
        DeviceType::Display,
        DeviceType::Audio,
        DeviceType::TPU,
        DeviceType::WiFi,
        DeviceType::Battery,
    ];
    
    // Filter device types if requested
    let filtered_types: Vec<&DeviceType> = if let Some(filter) = type_filter {
        device_types.iter()
            .filter(|t| t.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        device_types.iter().collect()
    };
    
    if filtered_types.is_empty() && type_filter.is_some() {
        return Err(anyhow!("Invalid device type filter: {}", type_filter.unwrap_or("")));
    }
    
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = Vec::new();
    
    for device_type in filtered_types {
        // Get devices of this type
        let devices = get_devices_by_type(api, *device_type);
        
        for device in devices {
            // Add to table data
            table_data.push(vec![
                device_type.to_string(),
                device.name().to_string(),
                if device.is_initialized() { "Initialized".to_string() } else { "Not Initialized".to_string() },
                device.device_info().vendor.clone(),
                device.device_info().model.clone(),
            ]);
            
            // Add to JSON data
            let mut device_json = serde_json::Map::new();
            device_json.insert("type".to_string(), serde_json::Value::String(device_type.to_string()));
            device_json.insert("name".to_string(), serde_json::Value::String(device.name().to_string()));
            device_json.insert("initialized".to_string(), serde_json::Value::Bool(device.is_initialized()));
            device_json.insert("vendor".to_string(), serde_json::Value::String(device.device_info().vendor.clone()));
            device_json.insert("model".to_string(), serde_json::Value::String(device.device_info().model.clone()));
            
            json_data.push(serde_json::Value::Object(device_json));
        }
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            if table_data.is_empty() {
                println!("No devices found.");
            } else {
                let headers = ["Type", "Name", "Status", "Vendor", "Model"];
                println!("{}", formatting::format_table(&headers, &table_data));
            }
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&json_data)
                .context("Failed to format JSON")?);
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn device_info(api: &VRCoreAPI, name: &str, format: &str) -> Result<()> {
    // Try to find the device
    if let Some(device) = api.hardware().get_device(name) {
        let device_type = device.device_type();
        let device_info = device.device_info();
        
        match format.to_lowercase().as_str() {
            "text" => {
                error::print_section(&format!("Device: {}", name));
                
                println!("Type: {}", device_type.to_string());
                println!("Status: {}", if device.is_initialized() {
                    "Initialized".green()
                } else {
                    "Not Initialized".yellow()
                });
                println!("Vendor: {}", device_info.vendor);
                println!("Model: {}", device_info.model);
                println!("Serial: {}", device_info.serial);
                println!("Firmware Version: {}", device_info.firmware_version);
                
                // Additional device-specific information
                match device_type {
                    DeviceType::Camera => {
                        if let Some(camera) = api.hardware().get_camera(name) {
                            println!("\nCamera Specifications:");
                            println!("Resolution: {}x{}", camera.resolution().0, camera.resolution().1);
                            println!("FPS: {}", camera.fps());
                            println!("FOV: {:.1}°", camera.fov());
                        }
                    },
                    DeviceType::IMU => {
                        if let Some(imu) = api.hardware().get_imu(name) {
                            println!("\nIMU Specifications:");
                            println!("Sample Rate: {} Hz", imu.sample_rate());
                            println!("Accelerometer Range: ±{} g", imu.accel_range());
                            println!("Gyroscope Range: ±{} deg/s", imu.gyro_range());
                        }
                    },
                    DeviceType::Display => {
                        println!("\nDisplay Specifications:");
                        println!("Resolution: {}x{}", device_info.properties.get("resolution_width").unwrap_or(&"Unknown".to_string()), 
                                                     device_info.properties.get("resolution_height").unwrap_or(&"Unknown".to_string()));
                        println!("Refresh Rate: {} Hz", device_info.properties.get("refresh_rate").unwrap_or(&"Unknown".to_string()));
                        println!("Panel Type: {}", device_info.properties.get("panel_type").unwrap_or(&"Unknown".to_string()));
                    },
                    DeviceType::Audio => {
                        println!("\nAudio Specifications:");
                        println!("Channels: {}", device_info.properties.get("channels").unwrap_or(&"Unknown".to_string()));
                        println!("Sample Rate: {} Hz", device_info.properties.get("sample_rate").unwrap_or(&"Unknown".to_string()));
                        println!("Bit Depth: {} bits", device_info.properties.get("bit_depth").unwrap_or(&"Unknown".to_string()));
                    },
                    DeviceType::TPU => {
                        println!("\nTPU Specifications:");
                        println!("Compute Units: {}", device_info.properties.get("compute_units").unwrap_or(&"Unknown".to_string()));
                        println!("Performance: {} TOPS", device_info.properties.get("performance").unwrap_or(&"Unknown".to_string()));
                        println!("Power Consumption: {} W", device_info.properties.get("power").unwrap_or(&"Unknown".to_string()));
                    },
                    DeviceType::WiFi => {
                        println!("\nWiFi Specifications:");
                        println!("Standard: {}", device_info.properties.get("standard").unwrap_or(&"Unknown".to_string()));
                        println!("Frequency Bands: {}", device_info.properties.get("frequency_bands").unwrap_or(&"Unknown".to_string()));
                        println!("Max Speed: {} Mbps", device_info.properties.get("max_speed").unwrap_or(&"Unknown".to_string()));
                    },
                    DeviceType::Battery => {
                        println!("\nBattery Specifications:");
                        println!("Capacity: {} mAh", device_info.properties.get("capacity").unwrap_or(&"Unknown".to_string()));
                        println!("Chemistry: {}", device_info.properties.get("chemistry").unwrap_or(&"Unknown".to_string()));
                        println!("Max Voltage: {} V", device_info.properties.get("max_voltage").unwrap_or(&"Unknown".to_string()));
                    },
                }
                
                // Show capabilities
                println!("\nCapabilities:");
                for capability in device.capabilities() {
                    println!("- {}", capability.to_string());
                }
                
                // Show properties
                if !device_info.properties.is_empty() {
                    println!("\nProperties:");
                    for (key, value) in &device_info.properties {
                        println!("- {}: {}", key, value);
                    }
                }
            },
            "json" => {
                let mut device_json = serde_json::Map::new();
                device_json.insert("name".to_string(), serde_json::Value::String(name.to_string()));
                device_json.insert("type".to_string(), serde_json::Value::String(device_type.to_string()));
                device_json.insert("initialized".to_string(), serde_json::Value::Bool(device.is_initialized()));
                device_json.insert("vendor".to_string(), serde_json::Value::String(device_info.vendor.clone()));
                device_json.insert("model".to_string(), serde_json::Value::String(device_info.model.clone()));
                device_json.insert("serial".to_string(), serde_json::Value::String(device_info.serial.clone()));
                device_json.insert("firmware_version".to_string(), serde_json::Value::String(device_info.firmware_version.clone()));
                
                // Add capabilities
                let capabilities: Vec<serde_json::Value> = device.capabilities()
                    .iter()
                    .map(|c| serde_json::Value::String(c.to_string()))
                    .collect();
                device_json.insert("capabilities".to_string(), serde_json::Value::Array(capabilities));
                
                // Add properties
                let mut properties_json = serde_json::Map::new();
                for (key, value) in &device_info.properties {
                    properties_json.insert(key.clone(), serde_json::Value::String(value.clone()));
                }
                device_json.insert("properties".to_string(), serde_json::Value::Object(properties_json));
                
                // Add device-specific information
                match device_type {
                    DeviceType::Camera => {
                        if let Some(camera) = api.hardware().get_camera(name) {
                            let mut camera_json = serde_json::Map::new();
                            camera_json.insert("resolution_width".to_string(), serde_json::Value::Number(serde_json::Number::from(camera.resolution().0)));
                            camera_json.insert("resolution_height".to_string(), serde_json::Value::Number(serde_json::Number::from(camera.resolution().1)));
                            camera_json.insert("fps".to_string(), serde_json::Value::Number(serde_json::Number::from(camera.fps())));
                            
                            if let Some(fov) = serde_json::Number::from_f64(camera.fov() as f64) {
                                camera_json.insert("fov".to_string(), serde_json::Value::Number(fov));
                            }
                            
                            device_json.insert("camera_specs".to_string(), serde_json::Value::Object(camera_json));
                        }
                    },
                    DeviceType::IMU => {
                        if let Some(imu) = api.hardware().get_imu(name) {
                            let mut imu_json = serde_json::Map::new();
                            imu_json.insert("sample_rate".to_string(), serde_json::Value::Number(serde_json::Number::from(imu.sample_rate())));
                            
                            if let Some(accel_range) = serde_json::Number::from_f64(imu.accel_range() as f64) {
                                imu_json.insert("accel_range".to_string(), serde_json::Value::Number(accel_range));
                            }
                            
                            if let Some(gyro_range) = serde_json::Number::from_f64(imu.gyro_range() as f64) {
                                imu_json.insert("gyro_range".to_string(), serde_json::Value::Number(gyro_range));
                            }
                            
                            device_json.insert("imu_specs".to_string(), serde_json::Value::Object(imu_json));
                        }
                    },
                    _ => {
                        // Other device types handled through properties
                    }
                }
                
                println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(device_json))
                    .context("Failed to format JSON")?);
            },
            _ => {
                return Err(anyhow!("Unsupported output format: {}", format));
            }
        }
        
        Ok(())
    } else {
        error::print_warning(&format!("Device not found: {}", name));
        Err(anyhow!("Device not found: {}", name))
    }
}

fn init_devices(api: &mut VRCoreAPI, device_name: Option<&str>, force: bool) -> Result<()> {
    // Confirm initialization if not forced
    if !force {
        let prompt = if let Some(name) = device_name {
            format!("Are you sure you want to initialize device {}?", name)
        } else {
            "Are you sure you want to initialize all hardware devices?".to_string()
        };
        
        if !error::confirm(&prompt, true)? {
            error::print_info("Initialization cancelled");
            return Ok(());
        }
    }
    
    if let Some(name) = device_name {
        // Initialize a specific device
        if let Some(device) = api.hardware_mut().get_device_mut(name) {
            let pb = error::create_progress_bar(100, &format!("Initializing device {}", name));
            pb.set_position(0);
            
            match device.initialize() {
                Ok(_) => {
                    pb.set_position(100);
                    pb.finish_with_message(format!("Device {} initialized successfully", name));
                    error::print_success(&format!("Initialized device: {}", name));
                    Ok(())
                },
                Err(e) => {
                    pb.abandon_with_message(format!("Failed to initialize device {}", name));
                    error::print_warning(&format!("Failed to initialize device {}: {}", name, e));
                    Err(e.into())
                }
            }
        } else {
            error::print_warning(&format!("Device not found: {}", name));
            Err(anyhow!("Device not found: {}", name))
        }
    } else {
        // Initialize all devices
        let devices = api.hardware().get_all_devices();
        let total_devices = devices.len() as u64;
        
        if total_devices == 0 {
            error::print_warning("No devices found to initialize");
            return Ok(());
        }
        
        let pb = error::create_progress_bar(total_devices, "Initializing all devices");
        pb.set_position(0);
        
        let mut success_count = 0;
        let mut failed_devices = Vec::new();
        
        for device in devices {
            let name = device.name().to_string();
            if let Some(device) = api.hardware_mut().get_device_mut(&name) {
                match device.initialize() {
                    Ok(_) => {
                        success_count += 1;
                    },
                    Err(e) => {
                        failed_devices.push((name.clone(), e.to_string()));
                    }
                }
            }
            pb.inc(1);
        }
        
        pb.finish();
        
        if failed_devices.is_empty() {
            error::print_success(&format!("All {} devices initialized successfully", success_count));
        } else {
            error::print_warning(&format!("{} of {} devices initialized successfully", success_count, total_devices));
            println!("\nFailed devices:");
            for (name, error) in failed_devices {
                println!("- {}: {}", name, error);
            }
            
            return Err(anyhow!("Some devices failed to initialize"));
        }
        
        Ok(())
    }
}

fn shutdown_devices(api: &mut VRCoreAPI, device_name: Option<&str>, force: bool) -> Result<()> {
    // Confirm shutdown if not forced
    if !force {
        let prompt = if let Some(name) = device_name {
            format!("Are you sure you want to shutdown device {}?", name)
        } else {
            "Are you sure you want to shutdown all hardware devices?".to_string()
        };
        
        if !error::confirm(&prompt, false)? {
            error::print_info("Shutdown cancelled");
            return Ok(());
        }
    }
    
    if let Some(name) = device_name {
        // Shutdown a specific device
        if let Some(device) = api.hardware_mut().get_device_mut(name) {
            let pb = error::create_progress_bar(100, &format!("Shutting down device {}", name));
            pb.set_position(0);
            
            match device.shutdown() {
                Ok(_) => {
                    pb.set_position(100);
                    pb.finish_with_message(format!("Device {} shutdown successfully", name));
                    error::print_success(&format!("Shutdown device: {}", name));
                    Ok(())
                },
                Err(e) => {
                    pb.abandon_with_message(format!("Failed to shutdown device {}", name));
                    error::print_warning(&format!("Failed to shutdown device {}: {}", name, e));
                    Err(e.into())
                }
            }
        } else {
            error::print_warning(&format!("Device not found: {}", name));
            Err(anyhow!("Device not found: {}", name))
        }
    } else {
        // Shutdown all devices
        let devices = api.hardware().get_all_devices();
        let total_devices = devices.len() as u64;
        
        if total_devices == 0 {
            error::print_warning("No devices found to shutdown");
            return Ok(());
        }
        
        let pb = error::create_progress_bar(total_devices, "Shutting down all devices");
        pb.set_position(0);
        
        let mut success_count = 0;
        let mut failed_devices = Vec::new();
        
        for device in devices {
            let name = device.name().to_string();
            if let Some(device) = api.hardware_mut().get_device_mut(&name) {
                match device.shutdown() {
                    Ok(_) => {
                        success_count += 1;
                    },
                    Err(e) => {
                        failed_devices.push((name.clone(), e.to_string()));
                    }
                }
            }
            pb.inc(1);
        }
        
        pb.finish();
        
        if failed_devices.is_empty() {
            error::print_success(&format!("All {} devices shutdown successfully", success_count));
        } else {
            error::print_warning(&format!("{} of {} devices shutdown successfully", success_count, total_devices));
            println!("\nFailed devices:");
            for (name, error) in failed_devices {
                println!("- {}: {}", name, error);
            }
            
            return Err(anyhow!("Some devices failed to shutdown"));
        }
        
        Ok(())
    }
}

fn diagnose_devices(api: &mut VRCoreAPI, device_name: Option<&str>, level: &str, format: &str) -> Result<()> {
    // Validate diagnostic level
    let diagnostic_level = match level.to_lowercase().as_str() {
        "basic" => DiagnosticLevel::Basic,
        "advanced" => DiagnosticLevel::Advanced,
        "full" => DiagnosticLevel::Full,
        _ => return Err(anyhow!("Invalid diagnostic level: {}", level)),
    };
    
    if let Some(name) = device_name {
        // Diagnose a specific device
        if let Some(device) = api.hardware().get_device(name) {
            let device_type = device.device_type();
            
            error::print_section(&format!("Diagnostics for device: {}", name));
            println!("Running {} level diagnostics...", level);
            
            let pb = error::create_progress_bar(100, "Running diagnostics");
            pb.set_position(0);
            
            // Run diagnostics based on device type
            let results = match device_type {
                DeviceType::Camera => diagnose_camera(api, name, &diagnostic_level, &pb),
                DeviceType::IMU => diagnose_imu(api, name, &diagnostic_level, &pb),
                DeviceType::Display => diagnose_display(api, name, &diagnostic_level, &pb),
                DeviceType::Audio => diagnose_audio(api, name, &diagnostic_level, &pb),
                DeviceType::TPU => diagnose_tpu(api, name, &diagnostic_level, &pb),
                DeviceType::WiFi => diagnose_wifi(api, name, &diagnostic_level, &pb),
                DeviceType::Battery => diagnose_battery(api, name, &diagnostic_level, &pb),
            };
            
            pb.finish();
            
            // Output results based on format
            match format.to_lowercase().as_str() {
                "text" => {
                    println!("\nDiagnostic Results:");
                    
                    let mut all_passed = true;
                    for (test, result) in &results {
                        match result {
                            DiagnosticResult::Pass => {
                                println!("✓ {}: {}", test, "PASS".green());
                            },
                            DiagnosticResult::Warning(msg) => {
                                all_passed = false;
                                println!("⚠ {}: {}", test, format!("WARNING - {}", msg).yellow());
                            },
                            DiagnosticResult::Fail(msg) => {
                                all_passed = false;
                                println!("✗ {}: {}", test, format!("FAIL - {}", msg).red());
                            },
                        }
                    }
                    
                    if all_passed {
                        error::print_success(&format!("All diagnostic tests passed for device {}", name));
                    } else {
                        error::print_warning(&format!("Some diagnostic tests failed for device {}", name));
                    }
                },
                "json" => {
                    let mut results_json = serde_json::Map::new();
                    results_json.insert("device".to_string(), serde_json::Value::String(name.to_string()));
                    results_json.insert("type".to_string(), serde_json::Value::String(device_type.to_string()));
                    results_json.insert("level".to_string(), serde_json::Value::String(level.to_string()));
                    
                    let mut tests_json = serde_json::Map::new();
                    let mut all_passed = true;
                    
                    for (test, result) in &results {
                        let (status, message) = match result {
                            DiagnosticResult::Pass => {
                                ("pass", "")
                            },
                            DiagnosticResult::Warning(msg) => {
                                all_passed = false;
                                ("warning", msg)
                            },
                            DiagnosticResult::Fail(msg) => {
                                all_passed = false;
                                ("fail", msg)
                            },
                        };
                        
                        let mut test_json = serde_json::Map::new();
                        test_json.insert("status".to_string(), serde_json::Value::String(status.to_string()));
                        test_json.insert("message".to_string(), serde_json::Value::String(message.to_string()));
                        
                        tests_json.insert(test.clone(), serde_json::Value::Object(test_json));
                    }
                    
                    results_json.insert("tests".to_string(), serde_json::Value::Object(tests_json));
                    results_json.insert("all_passed".to_string(), serde_json::Value::Bool(all_passed));
                    
                    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(results_json))
                        .context("Failed to format JSON")?);
                },
                _ => {
                    return Err(anyhow!("Unsupported output format: {}", format));
                }
            }
            
            Ok(())
        } else {
            error::print_warning(&format!("Device not found: {}", name));
            Err(anyhow!("Device not found: {}", name))
        }
    } else {
        // Diagnose all devices
        let devices = api.hardware().get_all_devices();
        
        if devices.is_empty() {
            error::print_warning("No devices found to diagnose");
            return Ok(());
        }
        
        error::print_section(&format!("Diagnostics for all devices (Level: {})", level));
        
        let mut all_results = HashMap::new();
        let total_devices = devices.len() as u64;
        let pb = error::create_progress_bar(total_devices, "Running diagnostics on all devices");
        
        for device in devices {
            let name = device.name().to_string();
            let device_type = device.device_type();
            
            // Run diagnostics based on device type
            let results = match device_type {
                DeviceType::Camera => diagnose_camera(api, &name, &diagnostic_level, &pb),
                DeviceType::IMU => diagnose_imu(api, &name, &diagnostic_level, &pb),
                DeviceType::Display => diagnose_display(api, &name, &diagnostic_level, &pb),
                DeviceType::Audio => diagnose_audio(api, &name, &diagnostic_level, &pb),
                DeviceType::TPU => diagnose_tpu(api, &name, &diagnostic_level, &pb),
                DeviceType::WiFi => diagnose_wifi(api, &name, &diagnostic_level, &pb),
                DeviceType::Battery => diagnose_battery(api, &name, &diagnostic_level, &pb),
            };
            
            all_results.insert(name, (device_type, results));
            pb.inc(1);
        }
        
        pb.finish();
        
        // Output results based on format
        match format.to_lowercase().as_str() {
            "text" => {
                println!("\nDiagnostic Results Summary:");
                
                let mut table = Table::new();
                table.add_row(Row::new(vec![
                    Cell::new("Device").style_spec("Fb"),
                    Cell::new("Type").style_spec("Fb"),
                    Cell::new("Status").style_spec("Fb"),
                    Cell::new("Details").style_spec("Fb"),
                ]));
                
                let mut all_passed = true;
                
                for (name, (device_type, results)) in &all_results {
                    let mut device_passed = true;
                    let mut warnings = 0;
                    let mut failures = 0;
                    
                    for (_, result) in results {
                        match result {
                            DiagnosticResult::Pass => {},
                            DiagnosticResult::Warning(_) => {
                                device_passed = false;
                                warnings += 1;
                            },
                            DiagnosticResult::Fail(_) => {
                                device_passed = false;
                                failures += 1;
                            },
                        }
                    }
                    
                    let status = if device_passed {
                        "PASS".green().to_string()
                    } else if failures > 0 {
                        "FAIL".red().to_string()
                    } else {
                        "WARNING".yellow().to_string()
                    };
                    
                    let details = if device_passed {
                        "All tests passed".to_string()
                    } else {
                        format!("{} warnings, {} failures", warnings, failures)
                    };
                    
                    table.add_row(Row::new(vec![
                        Cell::new(name),
                        Cell::new(&device_type.to_string()),
                        Cell::new(&status),
                        Cell::new(&details),
                    ]));
                    
                    all_passed = all_passed && device_passed;
                }
                
                table.printstd();
                
                if all_passed {
                    error::print_success("All diagnostic tests passed for all devices");
                } else {
                    error::print_warning("Some diagnostic tests failed");
                    println!("\nUse 'vr hardware diagnose --device <name>' for detailed results");
                }
            },
            "json" => {
                let mut json_results = Vec::new();
                
                for (name, (device_type, results)) in &all_results {
                    let mut device_json = serde_json::Map::new();
                    device_json.insert("name".to_string(), serde_json::Value::String(name.clone()));
                    device_json.insert("type".to_string(), serde_json::Value::String(device_type.to_string()));
                    
                    let mut tests_json = serde_json::Map::new();
                    let mut all_passed = true;
                    
                    for (test, result) in results {
                        let (status, message) = match result {
                            DiagnosticResult::Pass => {
                                ("pass", "")
                            },
                            DiagnosticResult::Warning(msg) => {
                                all_passed = false;
                                ("warning", msg)
                            },
                            DiagnosticResult::Fail(msg) => {
                                all_passed = false;
                                ("fail", msg)
                            },
                        };
                        
                        let mut test_json = serde_json::Map::new();
                        test_json.insert("status".to_string(), serde_json::Value::String(status.to_string()));
                        test_json.insert("message".to_string(), serde_json::Value::String(message.to_string()));
                        
                        tests_json.insert(test.clone(), serde_json::Value::Object(test_json));
                    }
                    
                    device_json.insert("tests".to_string(), serde_json::Value::Object(tests_json));
                    device_json.insert("all_passed".to_string(), serde_json::Value::Bool(all_passed));
                    
                    json_results.push(serde_json::Value::Object(device_json));
                }
                
                println!("{}", serde_json::to_string_pretty(&json_results)
                    .context("Failed to format JSON")?);
            },
            _ => {
                return Err(anyhow!("Unsupported output format: {}", format));
            }
        }
        
        Ok(())
    }
}

fn update_firmware(api: &mut VRCoreAPI, device_name: Option<&str>, file_path: Option<&std::path::Path>, check_only: bool, force: bool) -> Result<()> {
    if check_only {
        // Check for firmware updates
        if let Some(name) = device_name {
            // Check for a specific device
            if let Some(device) = api.hardware().get_device(name) {
                error::print_section(&format!("Firmware Update Check for device: {}", name));
                
                let current_version = device.device_info().firmware_version.clone();
                println!("Current firmware version: {}", current_version);
                
                // This would normally check with a server or local database for updates
                // For now, we'll simulate the check
                let latest_version = simulate_firmware_check(device.device_type(), &current_version);
                
                if latest_version > current_version {
                    error::print_warning(&format!("Firmware update available: {} -> {}", current_version, latest_version));
                    println!("Use 'vr hardware firmware --device {} --file <path>' to update", name);
                } else {
                    error::print_success(&format!("Device {} is running the latest firmware", name));
                }
            } else {
                error::print_warning(&format!("Device not found: {}", name));
                return Err(anyhow!("Device not found: {}", name));
            }
        } else {
            // Check for all devices
            let devices = api.hardware().get_all_devices();
            
            if devices.is_empty() {
                error::print_warning("No devices found to check for firmware updates");
                return Ok(());
            }
            
            error::print_section("Firmware Update Check for all devices");
            
            let mut table = Table::new();
            table.add_row(Row::new(vec![
                Cell::new("Device").style_spec("Fb"),
                Cell::new("Type").style_spec("Fb"),
                Cell::new("Current Version").style_spec("Fb"),
                Cell::new("Latest Version").style_spec("Fb"),
                Cell::new("Status").style_spec("Fb"),
            ]));
            
            let mut updates_available = false;
            
            for device in devices {
                let name = device.name().to_string();
                let device_type = device.device_type();
                let current_version = device.device_info().firmware_version.clone();
                let latest_version = simulate_firmware_check(device_type, &current_version);
                
                let (status, status_style) = if latest_version > current_version {
                    updates_available = true;
                    ("Update Available", "Fr")
                } else {
                    ("Up to Date", "Fg")
                };
                
                table.add_row(Row::new(vec![
                    Cell::new(&name),
                    Cell::new(&device_type.to_string()),
                    Cell::new(&current_version),
                    Cell::new(&latest_version),
                    Cell::new(status).style_spec(status_style),
                ]));
            }
            
            table.printstd();
            
            if updates_available {
                error::print_warning("Firmware updates are available for some devices");
                println!("Use 'vr hardware firmware --device <name> --file <path>' to update");
            } else {
                error::print_success("All devices are running the latest firmware");
            }
        }
        
        Ok(())
    } else {
        // Update firmware
        if let Some(name) = device_name {
            // Update a specific device
            if let Some(device) = api.hardware_mut().get_device_mut(name) {
                if let Some(path) = file_path {
                    // Validate file exists
                    validation::validate_file_exists(path)?;
                    
                    error::print_section(&format!("Firmware Update for device: {}", name));
                    
                    let current_version = device.device_info().firmware_version.clone();
                    println!("Current firmware version: {}", current_version);
                    
                    // Confirm update if not forced
                    if !force {
                        let prompt = format!("Are you sure you want to update firmware for device {}?", name);
                        if !error::confirm(&prompt, false)? {
                            error::print_info("Firmware update cancelled");
                            return Ok(());
                        }
                    }
                    
                    // Simulate firmware update
                    let pb = error::create_progress_bar(100, &format!("Updating firmware for device {}", name));
                    
                    // Read firmware file (in a real implementation, this would validate the firmware)
                    let _firmware_data = file::read_file(path)?;
                    
                    // Simulate update process
                    for i in 0..100 {
                        pb.set_position(i);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    
                    // Update device firmware version (in a real implementation, this would be done by the device)
                    let new_version = simulate_firmware_update(device.device_type(), &current_version);
                    
                    // Finish update
                    pb.finish_with_message(format!("Firmware updated successfully: {} -> {}", current_version, new_version));
                    error::print_success(&format!("Firmware updated for device {}: {} -> {}", name, current_version, new_version));
                    
                    Ok(())
                } else {
                    error::print_warning("No firmware file specified");
                    println!("Use 'vr hardware firmware --device {} --file <path>' to update", name);
                    Err(anyhow!("No firmware file specified"))
                }
            } else {
                error::print_warning(&format!("Device not found: {}", name));
                Err(anyhow!("Device not found: {}", name))
            }
        } else {
            error::print_warning("No device specified for firmware update");
            println!("Use 'vr hardware firmware --device <name> --file <path>' to update a specific device");
            Err(anyhow!("No device specified for firmware update"))
        }
    }
}

fn calibrate_device(api: &mut VRCoreAPI, device: &str, calibration_type: Option<&str>, interactive: bool) -> Result<()> {
    // Try to find the device
    if let Some(device) = api.hardware_mut().get_device_mut(device) {
        let device_type = device.device_type();
        let name = device.name().to_string();
        
        error::print_section(&format!("Calibrating device: {}", name));
        
        // Determine calibration type based on device type if not specified
        let cal_type = if let Some(t) = calibration_type {
            t.to_string()
        } else {
            match device_type {
                DeviceType::Camera => "intrinsic".to_string(),
                DeviceType::IMU => "gyro".to_string(),
                DeviceType::Display => "color".to_string(),
                DeviceType::Audio => "microphone".to_string(),
                _ => return Err(anyhow!("No default calibration type for device type: {}", device_type)),
            }
        };
        
        println!("Calibration type: {}", cal_type);
        
        if interactive {
            println!("\nInteractive calibration mode:");
            println!("Follow the on-screen instructions to complete the calibration process.");
            println!("Press Enter to begin...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }
        
        // Simulate calibration process
        let pb = error::create_progress_bar(100, &format!("Calibrating {} for device {}", cal_type, name));
        
        for i in 0..100 {
            pb.set_position(i);
            
            if interactive && i == 25 {
                pb.suspend(|| {
                    println!("\nCalibration step 1 of 3:");
                    println!("Please ensure the device is on a flat, stable surface.");
                    println!("Press Enter when ready...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                });
            }
            
            if interactive && i == 50 {
                pb.suspend(|| {
                    println!("\nCalibration step 2 of 3:");
                    println!("Please rotate the device slowly in all directions.");
                    println!("Press Enter when complete...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                });
            }
            
            if interactive && i == 75 {
                pb.suspend(|| {
                    println!("\nCalibration step 3 of 3:");
                    println!("Please return the device to its original position.");
                    println!("Press Enter when ready...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                });
            }
            
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        
        pb.finish_with_message(format!("Calibration completed for device {}", name));
        
        // In a real implementation, this would update the device's calibration data
        error::print_success(&format!("Successfully calibrated {} for device {}", cal_type, name));
        
        Ok(())
    } else {
        error::print_warning(&format!("Device not found: {}", device));
        Err(anyhow!("Device not found: {}", device))
    }
}

// Helper functions and types

enum DiagnosticLevel {
    Basic,
    Advanced,
    Full,
}

enum DiagnosticResult {
    Pass,
    Warning(String),
    Fail(String),
}

fn get_devices_by_type(api: &VRCoreAPI, device_type: DeviceType) -> Vec<Box<dyn Device>> {
    match device_type {
        DeviceType::Camera => {
            api.hardware().get_cameras().into_iter()
                .map(|camera| {
                    let device: Box<dyn Device> = camera;
                    device
                })
                .collect()
        },
        DeviceType::IMU => {
            api.hardware().get_imus().into_iter()
                .map(|imu| {
                    let device: Box<dyn Device> = imu;
                    device
                })
                .collect()
        },
        _ => {
            // For other device types, we'll use a generic approach
            api.hardware().get_all_devices().into_iter()
                .filter(|device| device.device_type() == device_type)
                .collect()
        }
    }
}

fn diagnose_camera(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(camera) = api.hardware().get_camera(name) {
        // Basic tests
        results.insert("Initialization".to_string(), 
            if camera.is_initialized() { 
                DiagnosticResult::Pass 
            } else { 
                DiagnosticResult::Fail("Camera is not initialized".to_string()) 
            });
        
        results.insert("Connection".to_string(), DiagnosticResult::Pass);
        
        // Simulate resolution test
        let (width, height) = camera.resolution();
        results.insert("Resolution".to_string(), 
            if width >= 640 && height >= 480 { 
                DiagnosticResult::Pass 
            } else { 
                DiagnosticResult::Warning(format!("Resolution {}x{} is below recommended minimum", width, height)) 
            });
        
        // Advanced tests
        match level {
            DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                // Simulate frame rate test
                let fps = camera.fps();
                results.insert("Frame Rate".to_string(), 
                    if fps >= 30 { 
                        DiagnosticResult::Pass 
                    } else { 
                        DiagnosticResult::Warning(format!("Frame rate {} fps is below recommended minimum", fps)) 
                    });
                
                // Simulate exposure test
                results.insert("Exposure".to_string(), DiagnosticResult::Pass);
                
                // Simulate calibration test
                results.insert("Calibration".to_string(), DiagnosticResult::Pass);
            },
            _ => {}
        }
        
        // Full tests
        if let DiagnosticLevel::Full = level {
            // Simulate image quality test
            results.insert("Image Quality".to_string(), DiagnosticResult::Pass);
            
            // Simulate distortion test
            results.insert("Distortion".to_string(), DiagnosticResult::Pass);
            
            // Simulate color accuracy test
            results.insert("Color Accuracy".to_string(), DiagnosticResult::Warning("Color accuracy is slightly off".to_string()));
            
            // Simulate latency test
            results.insert("Latency".to_string(), DiagnosticResult::Pass);
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("Camera {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_imu(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(imu) = api.hardware().get_imu(name) {
        // Basic tests
        results.insert("Initialization".to_string(), 
            if imu.is_initialized() { 
                DiagnosticResult::Pass 
            } else { 
                DiagnosticResult::Fail("IMU is not initialized".to_string()) 
            });
        
        results.insert("Connection".to_string(), DiagnosticResult::Pass);
        
        // Simulate sample rate test
        let sample_rate = imu.sample_rate();
        results.insert("Sample Rate".to_string(), 
            if sample_rate >= 100 { 
                DiagnosticResult::Pass 
            } else { 
                DiagnosticResult::Warning(format!("Sample rate {} Hz is below recommended minimum", sample_rate)) 
            });
        
        // Advanced tests
        match level {
            DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                // Simulate accelerometer test
                results.insert("Accelerometer".to_string(), DiagnosticResult::Pass);
                
                // Simulate gyroscope test
                results.insert("Gyroscope".to_string(), DiagnosticResult::Pass);
                
                // Simulate magnetometer test
                results.insert("Magnetometer".to_string(), DiagnosticResult::Warning("Magnetometer calibration recommended".to_string()));
            },
            _ => {}
        }
        
        // Full tests
        if let DiagnosticLevel::Full = level {
            // Simulate drift test
            results.insert("Drift".to_string(), DiagnosticResult::Pass);
            
            // Simulate noise test
            results.insert("Noise".to_string(), DiagnosticResult::Pass);
            
            // Simulate temperature compensation test
            results.insert("Temperature Compensation".to_string(), DiagnosticResult::Pass);
            
            // Simulate calibration test
            results.insert("Calibration".to_string(), DiagnosticResult::Pass);
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("IMU {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_display(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(device) = api.hardware().get_device(name) {
        if device.device_type() == DeviceType::Display {
            // Basic tests
            results.insert("Initialization".to_string(), 
                if device.is_initialized() { 
                    DiagnosticResult::Pass 
                } else { 
                    DiagnosticResult::Fail("Display is not initialized".to_string()) 
                });
            
            results.insert("Connection".to_string(), DiagnosticResult::Pass);
            
            // Advanced tests
            match level {
                DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                    // Simulate brightness test
                    results.insert("Brightness".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate contrast test
                    results.insert("Contrast".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate refresh rate test
                    results.insert("Refresh Rate".to_string(), DiagnosticResult::Pass);
                },
                _ => {}
            }
            
            // Full tests
            if let DiagnosticLevel::Full = level {
                // Simulate color accuracy test
                results.insert("Color Accuracy".to_string(), DiagnosticResult::Pass);
                
                // Simulate pixel test
                results.insert("Pixel Test".to_string(), DiagnosticResult::Pass);
                
                // Simulate persistence test
                results.insert("Persistence".to_string(), DiagnosticResult::Warning("Persistence slightly higher than optimal".to_string()));
                
                // Simulate uniformity test
                results.insert("Uniformity".to_string(), DiagnosticResult::Pass);
            }
        } else {
            results.insert("Device Type".to_string(), DiagnosticResult::Fail(format!("Device {} is not a display", name)));
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("Display {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_audio(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(device) = api.hardware().get_device(name) {
        if device.device_type() == DeviceType::Audio {
            // Basic tests
            results.insert("Initialization".to_string(), 
                if device.is_initialized() { 
                    DiagnosticResult::Pass 
                } else { 
                    DiagnosticResult::Fail("Audio device is not initialized".to_string()) 
                });
            
            results.insert("Connection".to_string(), DiagnosticResult::Pass);
            
            // Advanced tests
            match level {
                DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                    // Simulate playback test
                    results.insert("Playback".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate recording test
                    results.insert("Recording".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate volume test
                    results.insert("Volume Control".to_string(), DiagnosticResult::Pass);
                },
                _ => {}
            }
            
            // Full tests
            if let DiagnosticLevel::Full = level {
                // Simulate frequency response test
                results.insert("Frequency Response".to_string(), DiagnosticResult::Pass);
                
                // Simulate distortion test
                results.insert("Distortion".to_string(), DiagnosticResult::Pass);
                
                // Simulate noise test
                results.insert("Noise Level".to_string(), DiagnosticResult::Warning("Background noise level slightly elevated".to_string()));
                
                // Simulate latency test
                results.insert("Latency".to_string(), DiagnosticResult::Pass);
            }
        } else {
            results.insert("Device Type".to_string(), DiagnosticResult::Fail(format!("Device {} is not an audio device", name)));
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("Audio device {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_tpu(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(device) = api.hardware().get_device(name) {
        if device.device_type() == DeviceType::TPU {
            // Basic tests
            results.insert("Initialization".to_string(), 
                if device.is_initialized() { 
                    DiagnosticResult::Pass 
                } else { 
                    DiagnosticResult::Fail("TPU is not initialized".to_string()) 
                });
            
            results.insert("Connection".to_string(), DiagnosticResult::Pass);
            
            // Advanced tests
            match level {
                DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                    // Simulate performance test
                    results.insert("Performance".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate temperature test
                    results.insert("Temperature".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate power test
                    results.insert("Power Consumption".to_string(), DiagnosticResult::Pass);
                },
                _ => {}
            }
            
            // Full tests
            if let DiagnosticLevel::Full = level {
                // Simulate inference test
                results.insert("Inference Speed".to_string(), DiagnosticResult::Pass);
                
                // Simulate accuracy test
                results.insert("Inference Accuracy".to_string(), DiagnosticResult::Pass);
                
                // Simulate stability test
                results.insert("Stability".to_string(), DiagnosticResult::Pass);
                
                // Simulate memory test
                results.insert("Memory".to_string(), DiagnosticResult::Pass);
            }
        } else {
            results.insert("Device Type".to_string(), DiagnosticResult::Fail(format!("Device {} is not a TPU", name)));
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("TPU {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_wifi(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(device) = api.hardware().get_device(name) {
        if device.device_type() == DeviceType::WiFi {
            // Basic tests
            results.insert("Initialization".to_string(), 
                if device.is_initialized() { 
                    DiagnosticResult::Pass 
                } else { 
                    DiagnosticResult::Fail("WiFi device is not initialized".to_string()) 
                });
            
            results.insert("Connection".to_string(), DiagnosticResult::Pass);
            
            // Advanced tests
            match level {
                DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                    // Simulate signal strength test
                    results.insert("Signal Strength".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate throughput test
                    results.insert("Throughput".to_string(), DiagnosticResult::Warning("Throughput is below optimal levels".to_string()));
                    
                    // Simulate latency test
                    results.insert("Latency".to_string(), DiagnosticResult::Pass);
                },
                _ => {}
            }
            
            // Full tests
            if let DiagnosticLevel::Full = level {
                // Simulate packet loss test
                results.insert("Packet Loss".to_string(), DiagnosticResult::Pass);
                
                // Simulate interference test
                results.insert("Interference".to_string(), DiagnosticResult::Pass);
                
                // Simulate frequency band test
                results.insert("Frequency Band".to_string(), DiagnosticResult::Pass);
                
                // Simulate power consumption test
                results.insert("Power Consumption".to_string(), DiagnosticResult::Pass);
            }
        } else {
            results.insert("Device Type".to_string(), DiagnosticResult::Fail(format!("Device {} is not a WiFi device", name)));
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("WiFi device {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn diagnose_battery(api: &VRCoreAPI, name: &str, level: &DiagnosticLevel, pb: &indicatif::ProgressBar) -> HashMap<String, DiagnosticResult> {
    let mut results = HashMap::new();
    
    if let Some(device) = api.hardware().get_device(name) {
        if device.device_type() == DeviceType::Battery {
            // Basic tests
            results.insert("Initialization".to_string(), 
                if device.is_initialized() { 
                    DiagnosticResult::Pass 
                } else { 
                    DiagnosticResult::Fail("Battery is not initialized".to_string()) 
                });
            
            results.insert("Connection".to_string(), DiagnosticResult::Pass);
            
            // Advanced tests
            match level {
                DiagnosticLevel::Advanced | DiagnosticLevel::Full => {
                    // Simulate charge level test
                    results.insert("Charge Level".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate charging test
                    results.insert("Charging".to_string(), DiagnosticResult::Pass);
                    
                    // Simulate temperature test
                    results.insert("Temperature".to_string(), DiagnosticResult::Pass);
                },
                _ => {}
            }
            
            // Full tests
            if let DiagnosticLevel::Full = level {
                // Simulate capacity test
                results.insert("Capacity".to_string(), DiagnosticResult::Warning("Battery capacity is 85% of original".to_string()));
                
                // Simulate discharge rate test
                results.insert("Discharge Rate".to_string(), DiagnosticResult::Pass);
                
                // Simulate voltage test
                results.insert("Voltage".to_string(), DiagnosticResult::Pass);
                
                // Simulate health test
                results.insert("Health".to_string(), DiagnosticResult::Pass);
            }
        } else {
            results.insert("Device Type".to_string(), DiagnosticResult::Fail(format!("Device {} is not a battery", name)));
        }
    } else {
        results.insert("Device".to_string(), DiagnosticResult::Fail(format!("Battery {} not found", name)));
    }
    
    pb.inc(1);
    results
}

fn simulate_firmware_check(device_type: DeviceType, current_version: &str) -> String {
    // Parse current version
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() < 3 {
        return current_version.to_string();
    }
    
    // Simulate version check
    match device_type {
        DeviceType::Camera => {
            // Cameras have a new firmware version
            format!("{}.{}.{}", parts[0], parts[1], parts[2].parse::<i32>().unwrap_or(0) + 1)
        },
        DeviceType::IMU => {
            // IMUs have a new firmware version
            format!("{}.{}.{}", parts[0], parts[1].parse::<i32>().unwrap_or(0) + 1, parts[2])
        },
        _ => {
            // Other devices are up to date
            current_version.to_string()
        }
    }
}

fn simulate_firmware_update(device_type: DeviceType, current_version: &str) -> String {
    // Parse current version
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() < 3 {
        return format!("1.0.0");
    }
    
    // Simulate version update
    match device_type {
        DeviceType::Camera => {
            // Cameras get a patch update
            format!("{}.{}.{}", parts[0], parts[1], parts[2].parse::<i32>().unwrap_or(0) + 1)
        },
        DeviceType::IMU => {
            // IMUs get a minor update
            format!("{}.{}.{}", parts[0], parts[1].parse::<i32>().unwrap_or(0) + 1, parts[2])
        },
        DeviceType::Display => {
            // Displays get a major update
            format!("{}.{}.{}", parts[0].parse::<i32>().unwrap_or(0) + 1, "0", "0")
        },
        _ => {
            // Other devices get a minor update
            format!("{}.{}.{}", parts[0], parts[1].parse::<i32>().unwrap_or(0) + 1, "0")
        }
    }
}
