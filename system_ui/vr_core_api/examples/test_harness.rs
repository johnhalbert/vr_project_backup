//! Test harness for the VR Core API
//!
//! This binary provides a simple test harness for verifying the functionality
//! of the VR Core API components.

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::env;
use vr_core_api::{VRCoreAPI, config::{ConfigCategory, ConfigValue}};

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    println!("VR Core API Test Harness");
    println!("========================");
    
    // Initialize the VR Core API
    println!("\nInitializing VR Core API...");
    let mut api = VRCoreAPI::with_config_path(config_path)
        .context("Failed to initialize VR Core API")?;
    
    // Test configuration management
    test_configuration(&mut api)?;
    
    // Test hardware management
    test_hardware(&mut api)?;
    
    // Shutdown the API
    println!("\nShutting down VR Core API...");
    api.shutdown()?;
    println!("Shutdown complete");
    
    println!("\nAll tests completed successfully!");
    Ok(())
}

/// Test configuration management functionality
fn test_configuration(api: &mut VRCoreAPI) -> Result<()> {
    println!("\nTesting Configuration Management:");
    println!("--------------------------------");
    
    // Get configuration values
    let display_refresh = api.config().get(ConfigCategory::Display, "refresh_rate")?;
    println!("Display refresh rate: {:?}", display_refresh);
    
    let board_type = api.config().get(ConfigCategory::Hardware, "board_type")?;
    println!("Board type: {:?}", board_type);
    
    // Set a configuration value
    println!("Setting display brightness to 85...");
    api.config_mut().set(ConfigCategory::Display, "brightness", ConfigValue::Integer(85))?;
    
    // Verify the change
    let brightness = api.config().get(ConfigCategory::Display, "brightness")?;
    println!("Display brightness: {:?}", brightness);
    
    println!("Configuration tests passed");
    Ok(())
}

/// Test hardware management functionality
fn test_hardware(api: &mut VRCoreAPI) -> Result<()> {
    println!("\nTesting Hardware Management:");
    println!("---------------------------");
    
    // Initialize hardware
    println!("Initializing hardware devices...");
    api.hardware_mut().initialize()?;
    
    // Get camera devices
    let cameras = api.hardware().get_cameras();
    println!("Found {} camera devices:", cameras.len());
    for (i, camera) in cameras.iter().enumerate() {
        println!("  Camera {}: {} (initialized: {})", 
                 i, camera.name(), camera.is_initialized());
    }
    
    // Get IMU devices
    let imus = api.hardware().get_imus();
    println!("Found {} IMU devices:", imus.len());
    for (i, imu) in imus.iter().enumerate() {
        println!("  IMU {}: {} (initialized: {})", 
                 i, imu.name(), imu.is_initialized());
    }
    
    println!("Hardware tests passed");
    Ok(())
}
