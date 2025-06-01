use anyhow::Result;
use colored::Colorize;
use prettytable::{Table, Row, Cell};
use vr_core_api::VRCoreAPI;

use crate::SystemCommands;
use crate::commands::config::format_config_value;

pub fn handle_command(command: &SystemCommands, api: &mut VRCoreAPI) -> Result<()> {
    match command {
        SystemCommands::Status => {
            show_status(api)
        },
        SystemCommands::Info => {
            show_info(api)
        },
        SystemCommands::Restart { force } => {
            restart_system(api, *force)
        },
        SystemCommands::Update { check_only } => {
            update_system(api, *check_only)
        },
    }
}

fn show_status(api: &VRCoreAPI) -> Result<()> {
    println!("{}", "System Status".green().bold());
    println!("{}", "=============".green());
    
    // Display system status information
    println!("VR Core API Version: {}", vr_core_api::VERSION);
    
    // Hardware status
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Component").style_spec("Fb"),
        Cell::new("Status").style_spec("Fb"),
    ]));
    
    // Configuration
    table.add_row(Row::new(vec![
        Cell::new("Configuration"),
        Cell::new(&format!("{}", if api.config().is_dirty() {
            "Modified (unsaved changes)".yellow()
        } else {
            "OK".green()
        })),
    ]));
    
    // Hardware
    let camera_count = api.hardware().get_cameras().len();
    let imu_count = api.hardware().get_imus().len();
    
    table.add_row(Row::new(vec![
        Cell::new("Hardware"),
        Cell::new(&format!("{} cameras, {} IMUs", camera_count, imu_count)),
    ]));
    
    // Display the table
    table.printstd();
    
    Ok(())
}

fn show_info(api: &VRCoreAPI) -> Result<()> {
    println!("{}", "System Information".green().bold());
    println!("{}", "==================".green());
    
    // Display detailed system information
    println!("VR Core API Version: {}", vr_core_api::VERSION);
    
    // Get hardware information from config
    match api.config().get(vr_core_api::config::ConfigCategory::Hardware, "board_type") {
        Ok(board_type) => {
            println!("Board Type: {}", format_config_value(&board_type).blue());
        },
        Err(_) => {
            println!("Board Type: {}", "Unknown".yellow());
        }
    }
    
    match api.config().get(vr_core_api::config::ConfigCategory::Hardware, "memory_size") {
        Ok(memory_size) => {
            println!("Memory Size: {}", format_config_value(&memory_size).blue());
        },
        Err(_) => {
            println!("Memory Size: {}", "Unknown".yellow());
        }
    }
    
    // System paths
    println!("\n{}", "System Paths:".bold());
    println!("Config Directory: {}", dirs::config_dir()
        .unwrap_or_default()
        .join("vr")
        .to_string_lossy()
        .blue());
    
    // Runtime information
    println!("\n{}", "Runtime Information:".bold());
    println!("Rust Version: {}", env!("CARGO_PKG_RUST_VERSION").blue());
    
    Ok(())
}

fn restart_system(_api: &mut VRCoreAPI, force: bool) -> Result<()> {
    // This would require additional functionality in the Core API
    // For now, we'll just show a message
    
    if force {
        println!("{}: Force restart of the VR system is not implemented yet", 
                 "Warning".yellow().bold());
    } else {
        println!("{}: Restart of the VR system is not implemented yet", 
                 "Warning".yellow().bold());
    }
    
    Ok(())
}

fn update_system(_api: &mut VRCoreAPI, check_only: bool) -> Result<()> {
    // This would require additional functionality in the Core API
    // For now, we'll just show a message
    
    if check_only {
        println!("{}: Checking for updates is not implemented yet", 
                 "Warning".yellow().bold());
    } else {
        println!("{}: Updating the VR system is not implemented yet", 
                 "Warning".yellow().bold());
    }
    
    Ok(())
}
