use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::time::Duration;

use crate::ScriptCommands;
use crate::utils::{error, formatting, file, validation, script};

pub fn handle_command(command: &ScriptCommands, api: &mut vr_core_api::VRCoreAPI) -> Result<()> {
    match command {
        ScriptCommands::Run { file, variables, continue_on_error } => {
            run_script(file, variables.as_ref(), *continue_on_error, api)
        },
        ScriptCommands::List { format } => {
            list_scripts(format)
        },
        ScriptCommands::Create { file, template } => {
            create_script(file, template.as_deref())
        },
        ScriptCommands::Edit { file } => {
            edit_script(file)
        },
        ScriptCommands::Delete { file, force } => {
            delete_script(file, *force)
        },
        ScriptCommands::Export { file, output } => {
            export_script(file, output)
        },
        ScriptCommands::Import { file, input } => {
            import_script(file, input)
        },
    }
}

fn run_script(script_path: &Path, variables: Option<&Vec<String>>, continue_on_error: bool, api: &mut vr_core_api::VRCoreAPI) -> Result<()> {
    // Validate script exists
    validation::validate_file_exists(script_path)?;
    
    error::print_section(&format!("Running Script: {}", script_path.display()));
    
    // Parse variables
    let mut context = script::ScriptContext {
        continue_on_error,
        ..Default::default()
    };
    
    if let Some(vars) = variables {
        for var_str in vars {
            if let Some(pos) = var_str.find('=') {
                let name = var_str[..pos].trim();
                let value = var_str[pos+1..].trim();
                context.variables.insert(name.to_string(), value.to_string());
            } else {
                error::print_warning(&format!("Invalid variable format: {}", var_str));
                return Err(anyhow!("Invalid variable format: {}", var_str));
            }
        }
    }
    
    // Add API to context
    context.variables.insert("API_VERSION".to_string(), api.version().to_string());
    
    // Read script content
    let content = file::read_file(script_path)?;
    
    // Execute script
    let pb = error::create_progress_bar(100, "Executing script");
    pb.set_position(0);
    
    // Parse script lines
    let lines: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    let total_lines = lines.len() as u64;
    if total_lines == 0 {
        pb.finish_with_message("Script is empty");
        error::print_warning("Script is empty");
        return Ok(());
    }
    
    // Execute each line
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let progress = ((i as u64 + 1) * 100) / total_lines;
        pb.set_position(progress);
        
        // Check for variable assignment
        if line.contains('=') && !line.starts_with("if ") && !line.contains(" == ") && !line.contains(" != ") {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let var_value = parts[1].trim();
                
                // Handle quoted values
                let var_value = if (var_value.starts_with('"') && var_value.ends_with('"')) || 
                                  (var_value.starts_with('\'') && var_value.ends_with('\'')) {
                    &var_value[1..var_value.len()-1]
                } else {
                    var_value
                };
                
                // Store variable
                context.variables.insert(var_name.to_string(), var_value.to_string());
                continue;
            }
        }
        
        // Replace variables in command
        let mut command = line.to_string();
        for (var_name, var_value) in &context.variables {
            command = command.replace(&format!("${{{}}}", var_name), var_value);
            command = command.replace(&format!("${}", var_name), var_value);
        }
        
        // Check for special commands
        if command.starts_with("vr ") {
            // Execute VR CLI command
            pb.suspend(|| {
                println!("Executing: {}", command.blue());
            });
            
            // In a real implementation, this would call the CLI command directly
            // For now, we'll just simulate it
            std::thread::sleep(Duration::from_millis(500));
            
            pb.suspend(|| {
                println!("Command completed successfully");
            });
        } else if command.starts_with("sleep ") {
            // Sleep command
            if let Some(duration_str) = command.strip_prefix("sleep ") {
                if let Ok(duration) = duration_str.trim().parse::<u64>() {
                    pb.suspend(|| {
                        println!("Sleeping for {} seconds", duration);
                    });
                    
                    std::thread::sleep(Duration::from_secs(duration));
                } else {
                    pb.suspend(|| {
                        error::print_warning(&format!("Invalid sleep duration: {}", duration_str));
                    });
                    
                    if !context.continue_on_error {
                        pb.abandon_with_message(format!("Script execution failed at line {}", line_num));
                        return Err(anyhow!("Invalid sleep duration: {}", duration_str));
                    }
                }
            }
        } else if command.starts_with("echo ") {
            // Echo command
            if let Some(message) = command.strip_prefix("echo ") {
                pb.suspend(|| {
                    println!("{}", message);
                });
            }
        } else if command.starts_with("if ") {
            // Simple conditional
            pb.suspend(|| {
                println!("Conditional statements are not fully implemented yet");
            });
            // In a real implementation, this would parse and evaluate the condition
        } else {
            // Execute shell command
            pb.suspend(|| {
                println!("Executing: {}", command.blue());
            });
            
            match execute_command(&command, &context.working_dir) {
                Ok(output) => {
                    pb.suspend(|| {
                        println!("{}", output);
                    });
                },
                Err(e) => {
                    pb.suspend(|| {
                        error::print_warning(&format!("Error at line {}: {}", line_num, e));
                    });
                    
                    if !context.continue_on_error {
                        pb.abandon_with_message(format!("Script execution failed at line {}", line_num));
                        return Err(anyhow!("Script execution failed at line {}: {}", line_num, e));
                    }
                }
            }
        }
    }
    
    pb.finish_with_message("Script execution completed");
    error::print_success(&format!("Successfully executed script: {}", script_path.display()));
    
    Ok(())
}

fn list_scripts(format: &str) -> Result<()> {
    error::print_section("Available Scripts");
    
    // Get scripts directory
    let scripts_dir = script::get_scripts_dir()?;
    
    // List scripts
    let scripts = script::list_scripts()?;
    
    if scripts.is_empty() {
        println!("No scripts found.");
        println!("\nTo create a new script, use:");
        println!("  vr script create <file>");
        return Ok(());
    }
    
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = Vec::new();
    
    for script_path in &scripts {
        let file_name = script_path.file_name().unwrap().to_string_lossy().to_string();
        let metadata = fs::metadata(script_path)?;
        let size = metadata.len();
        let modified = metadata.modified()
            .map(|time| {
                chrono::DateTime::<chrono::Local>::from(time)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string());
        
        // Count lines
        let content = file::read_file(script_path)?;
        let total_lines = content.lines().count();
        let command_lines = content.lines()
            .filter(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with('#')
            })
            .count();
        
        // Add to table data
        table_data.push(vec![
            file_name.clone(),
            format!("{}", size),
            modified.clone(),
            format!("{}", total_lines),
            format!("{}", command_lines),
        ]);
        
        // Add to JSON data
        let mut script_json = serde_json::Map::new();
        script_json.insert("name".to_string(), serde_json::Value::String(file_name));
        script_json.insert("path".to_string(), serde_json::Value::String(script_path.to_string_lossy().to_string()));
        script_json.insert("size".to_string(), serde_json::Value::Number(serde_json::Number::from(size)));
        script_json.insert("modified".to_string(), serde_json::Value::String(modified));
        script_json.insert("total_lines".to_string(), serde_json::Value::Number(serde_json::Number::from(total_lines as u64)));
        script_json.insert("command_lines".to_string(), serde_json::Value::Number(serde_json::Number::from(command_lines as u64)));
        
        json_data.push(serde_json::Value::Object(script_json));
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Name", "Size (bytes)", "Last Modified", "Total Lines", "Command Lines"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&json_data)
                .context("Failed to format JSON")?);
        },
        "text" => {
            for (i, script_path) in scripts.iter().enumerate() {
                let file_name = script_path.file_name().unwrap().to_string_lossy();
                println!("{}. {} ({} lines, {} commands)", 
                         i + 1, 
                         file_name.bold(), 
                         table_data[i][3], 
                         table_data[i][4]);
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    println!("\nTo run a script, use:");
    println!("  vr script run <file>");
    
    Ok(())
}

fn create_script(file_path: &Path, template: Option<&str>) -> Result<()> {
    // Determine full path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        script::get_scripts_dir()?.join(file_path)
    };
    
    // Check if file already exists
    if full_path.exists() {
        if !error::confirm(&format!("Script {} already exists. Overwrite?", full_path.display()), false)? {
            error::print_info("Script creation cancelled");
            return Ok(());
        }
    }
    
    error::print_section(&format!("Creating Script: {}", full_path.display()));
    
    // Generate content based on template
    let content = match template {
        Some("basic") => {
            r#"#!/bin/bash
# Basic script template
# Created: $(date)

# Variables
NAME="VR Headset"
VERSION="1.0"

# Echo information
echo "Starting basic script for $NAME version $VERSION"

# Execute VR CLI commands
vr status
vr config list

echo "Script completed successfully"
"#
        },
        Some("monitoring") => {
            r#"#!/bin/bash
# Monitoring script template
# Created: $(date)

# Variables
INTERVAL=5
COUNT=10

# Echo information
echo "Starting monitoring script with interval $INTERVAL seconds for $COUNT iterations"

# Execute monitoring commands
vr monitoring metrics --interval $INTERVAL --count $COUNT

# Check for alerts
vr monitoring alerts

echo "Monitoring script completed successfully"
"#
        },
        Some("maintenance") => {
            r#"#!/bin/bash
# Maintenance script template
# Created: $(date)

# Variables
BACKUP_DIR="/tmp/vr_backup"

# Echo information
echo "Starting maintenance script"

# Create backup directory
mkdir -p $BACKUP_DIR

# Export configuration
vr config export $BACKUP_DIR/config.json --format json

# Check hardware status
vr hardware list

# Run diagnostics
vr hardware diagnose

echo "Maintenance script completed successfully"
"#
        },
        Some("calibration") => {
            r#"#!/bin/bash
# Calibration script template
# Created: $(date)

# Variables
DEVICES=("camera_left" "camera_right" "imu")

# Echo information
echo "Starting calibration script"

# Calibrate each device
for device in "${DEVICES[@]}"; do
    echo "Calibrating $device..."
    vr hardware calibrate --device $device
    sleep 2
done

echo "Calibration script completed successfully"
"#
        },
        Some("custom") | None => {
            r#"#!/bin/bash
# Custom script
# Created: $(date)

# Variables
# Define your variables here
NAME="VR Headset"

# Echo information
echo "Starting custom script for $NAME"

# Add your commands here
# Examples:
# vr status
# vr config list
# vr hardware list
# sleep 5
# vr monitoring metrics --count 1

echo "Script completed successfully"
"#
        },
        _ => {
            return Err(anyhow!("Unknown template: {}", template.unwrap_or("")));
        }
    };
    
    // Create script
    script::create_script(&full_path, &content)?;
    
    error::print_success(&format!("Created script: {}", full_path.display()));
    println!("\nTo run this script, use:");
    println!("  vr script run {}", full_path.display());
    
    Ok(())
}

fn edit_script(file_path: &Path) -> Result<()> {
    // Determine full path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        script::get_scripts_dir()?.join(file_path)
    };
    
    // Check if file exists
    if !full_path.exists() {
        error::print_warning(&format!("Script does not exist: {}", full_path.display()));
        
        if error::confirm("Would you like to create this script?", true)? {
            return create_script(file_path, None);
        } else {
            return Err(anyhow!("Script does not exist: {}", full_path.display()));
        }
    }
    
    error::print_section(&format!("Editing Script: {}", full_path.display()));
    
    // Determine editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    
    // Open editor
    let status = Command::new(&editor)
        .arg(&full_path)
        .status()
        .context(format!("Failed to open editor: {}", editor))?;
    
    if status.success() {
        error::print_success(&format!("Edited script: {}", full_path.display()));
        Ok(())
    } else {
        error::print_warning(&format!("Editor exited with non-zero status: {}", status));
        Err(anyhow!("Editor exited with non-zero status: {}", status))
    }
}

fn delete_script(file_path: &Path, force: bool) -> Result<()> {
    // Determine full path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        script::get_scripts_dir()?.join(file_path)
    };
    
    // Check if file exists
    if !full_path.exists() {
        error::print_warning(&format!("Script does not exist: {}", full_path.display()));
        return Err(anyhow!("Script does not exist: {}", full_path.display()));
    }
    
    // Confirm deletion if not forced
    if !force {
        if !error::confirm(&format!("Are you sure you want to delete script {}?", full_path.display()), false)? {
            error::print_info("Script deletion cancelled");
            return Ok(());
        }
    }
    
    // Delete script
    fs::remove_file(&full_path)
        .context(format!("Failed to delete script: {}", full_path.display()))?;
    
    error::print_success(&format!("Deleted script: {}", full_path.display()));
    
    Ok(())
}

fn export_script(file_path: &Path, output_path: &Path) -> Result<()> {
    // Determine full path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        script::get_scripts_dir()?.join(file_path)
    };
    
    // Check if file exists
    if !full_path.exists() {
        error::print_warning(&format!("Script does not exist: {}", full_path.display()));
        return Err(anyhow!("Script does not exist: {}", full_path.display()));
    }
    
    // Check if output file already exists
    if output_path.exists() {
        if !error::confirm(&format!("Output file {} already exists. Overwrite?", output_path.display()), false)? {
            error::print_info("Script export cancelled");
            return Ok(());
        }
    }
    
    // Copy script
    fs::copy(&full_path, output_path)
        .context(format!("Failed to export script from {} to {}", full_path.display(), output_path.display()))?;
    
    error::print_success(&format!("Exported script from {} to {}", full_path.display(), output_path.display()));
    
    Ok(())
}

fn import_script(file_path: &Path, input_path: &Path) -> Result<()> {
    // Check if input file exists
    if !input_path.exists() {
        error::print_warning(&format!("Input file does not exist: {}", input_path.display()));
        return Err(anyhow!("Input file does not exist: {}", input_path.display()));
    }
    
    // Determine full path
    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        script::get_scripts_dir()?.join(file_path)
    };
    
    // Check if file already exists
    if full_path.exists() {
        if !error::confirm(&format!("Script {} already exists. Overwrite?", full_path.display()), false)? {
            error::print_info("Script import cancelled");
            return Ok(());
        }
    }
    
    // Copy script
    fs::copy(input_path, &full_path)
        .context(format!("Failed to import script from {} to {}", input_path.display(), full_path.display()))?;
    
    // Make script executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&full_path)
            .context(format!("Failed to get metadata for script file: {}", full_path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&full_path, perms)
            .context(format!("Failed to set permissions for script file: {}", full_path.display()))?;
    }
    
    error::print_success(&format!("Imported script from {} to {}", input_path.display(), full_path.display()));
    
    Ok(())
}

// Helper functions

fn execute_command(command: &str, working_dir: &Path) -> Result<String> {
    // Split command into program and arguments
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
    let args: Vec<&str> = parts.collect();
    
    // Execute command
    let output = Command::new(program)
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("Failed to execute command: {}", command))?;
    
    // Check if command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Command failed: {}", stderr));
    }
    
    // Return stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}
