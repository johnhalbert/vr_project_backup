# CLI Interface Developer Guide

## Introduction

This guide provides detailed information for developers who want to work with the VR headset's Command Line Interface (CLI). The CLI provides a powerful text-based interface for managing and controlling the VR headset system, particularly useful for automation, scripting, and remote management scenarios.

This guide assumes you are already familiar with the general concepts covered in the main Developer Guide and focuses specifically on working with the CLI components.

## CLI Architecture

The CLI is structured as a modular Rust binary application:

```
/system_ui/vr_cli/
├── src/
│   ├── commands/       # Command implementations
│   │   ├── config.rs   # Configuration commands
│   │   ├── hardware.rs # Hardware control commands
│   │   ├── ipc.rs      # IPC management commands
│   │   ├── mod.rs      # Command registry
│   │   ├── monitoring.rs # System monitoring commands
│   │   ├── script.rs   # Script execution commands
│   │   ├── security.rs # Security commands
│   │   └── system.rs   # System management commands
│   ├── utils/          # Utility functions
│   │   ├── error.rs    # Error handling
│   │   ├── file.rs     # File operations
│   │   ├── formatting.rs # Output formatting
│   │   ├── mod.rs      # Utility registry
│   │   ├── script.rs   # Script handling
│   │   └── validation.rs # Input validation
│   └── main.rs         # Application entry point
└── Cargo.toml          # Project dependencies
```

The CLI uses the Core API to interact with the VR headset system, providing a text-based interface for all system functionality.

## Getting Started with CLI Development

### Setting Up Your Development Environment

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/vrheadset/vr_cli.git
   cd vr_cli
   ```

2. **Install Rust and Dependencies**:
   ```bash
   # Install Rust using rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install additional dependencies
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   ```

3. **Build the CLI**:
   ```bash
   cargo build
   ```

4. **Run the CLI**:
   ```bash
   cargo run -- --help
   ```

### Project Structure

The CLI follows a modular architecture with clear separation of concerns:

- `main.rs`: Application entry point and command-line argument parsing
- `commands/`: Command implementations organized by functionality
- `utils/`: Utility functions for common operations

### Command Structure

Each command in the CLI follows a consistent structure:

1. **Command Definition**: Defines the command name, arguments, and options
2. **Command Execution**: Implements the command logic
3. **Output Formatting**: Formats the command output for display
4. **Error Handling**: Handles and reports errors

## Implementing CLI Commands

### Command Registry

The command registry is responsible for registering and dispatching commands.

```rust
// src/commands/mod.rs
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;
use crate::utils::error::CliError;

// Import command modules
pub mod config;
pub mod hardware;
pub mod ipc;
pub mod monitoring;
pub mod script;
pub mod security;
pub mod system;

// Command trait
pub trait Command {
    fn name(&self) -> &'static str;
    fn app<'a, 'b>(&self) -> App<'a, 'b>;
    fn execute(&self, matches: &ArgMatches) -> Result<(), CliError>;
}

// Command registry
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = CommandRegistry {
            commands: HashMap::new(),
        };
        
        // Register commands
        registry.register(Box::new(config::ConfigCommand));
        registry.register(Box::new(hardware::HardwareCommand));
        registry.register(Box::new(ipc::IpcCommand));
        registry.register(Box::new(monitoring::MonitoringCommand));
        registry.register(Box::new(script::ScriptCommand));
        registry.register(Box::new(security::SecurityCommand));
        registry.register(Box::new(system::SystemCommand));
        
        registry
    }
    
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    pub fn app<'a, 'b>(&self) -> App<'a, 'b> {
        let mut app = App::new("vr-cli")
            .version("1.0.0")
            .author("VR Headset Team")
            .about("Command line interface for the VR headset system")
            .setting(AppSettings::SubcommandRequiredElseHelp);
        
        // Add subcommands
        for command in self.commands.values() {
            app = app.subcommand(command.app());
        }
        
        app
    }
    
    pub fn execute(&self, matches: ArgMatches) -> Result<(), CliError> {
        // Get the subcommand name and matches
        let (subcommand_name, subcommand_matches) = matches.subcommand();
        
        // Find the command
        if let Some(command) = self.commands.get(subcommand_name) {
            // Execute the command
            command.execute(subcommand_matches.unwrap())
        } else {
            Err(CliError::CommandNotFound(subcommand_name.to_string()))
        }
    }
}
```

### Example: Implementing a Configuration Command

```rust
// src/commands/config.rs
use clap::{App, Arg, ArgMatches, SubCommand};
use vr_core_api::config::{ConfigManager, ConfigError};
use crate::commands::Command;
use crate::utils::error::CliError;
use crate::utils::formatting::{format_table, format_json};

pub struct ConfigCommand;

impl Command for ConfigCommand {
    fn name(&self) -> &'static str {
        "config"
    }
    
    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name())
            .about("Manage system configuration")
            .subcommand(
                SubCommand::with_name("get")
                    .about("Get configuration value")
                    .arg(
                        Arg::with_name("key")
                            .help("Configuration key")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        Arg::with_name("format")
                            .help("Output format")
                            .long("format")
                            .short("f")
                            .possible_values(&["json", "table"])
                            .default_value("table")
                    )
            )
            .subcommand(
                SubCommand::with_name("set")
                    .about("Set configuration value")
                    .arg(
                        Arg::with_name("key")
                            .help("Configuration key")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        Arg::with_name("value")
                            .help("Configuration value (JSON format)")
                            .required(true)
                            .index(2)
                    )
            )
            .subcommand(
                SubCommand::with_name("list")
                    .about("List configuration keys")
                    .arg(
                        Arg::with_name("prefix")
                            .help("Key prefix filter")
                            .index(1)
                    )
                    .arg(
                        Arg::with_name("format")
                            .help("Output format")
                            .long("format")
                            .short("f")
                            .possible_values(&["json", "table"])
                            .default_value("table")
                    )
            )
    }
    
    fn execute(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Create a configuration manager
        let config_manager = ConfigManager::new()
            .map_err(|e| CliError::ConfigError(format!("Failed to create config manager: {}", e)))?;
        
        // Handle subcommands
        match matches.subcommand() {
            ("get", Some(sub_matches)) => {
                self.execute_get(sub_matches, &config_manager)
            },
            ("set", Some(sub_matches)) => {
                self.execute_set(sub_matches, &config_manager)
            },
            ("list", Some(sub_matches)) => {
                self.execute_list(sub_matches, &config_manager)
            },
            _ => {
                Err(CliError::InvalidSubcommand(self.name().to_string()))
            }
        }
    }
}

impl ConfigCommand {
    fn execute_get(&self, matches: &ArgMatches, config_manager: &ConfigManager) -> Result<(), CliError> {
        // Get the key
        let key = matches.value_of("key").unwrap();
        
        // Get the format
        let format = matches.value_of("format").unwrap();
        
        // Get the configuration value
        let value = config_manager.get(key)
            .map_err(|e| CliError::ConfigError(format!("Failed to get config value: {}", e)))?;
        
        // Format and print the value
        match format {
            "json" => {
                println!("{}", format_json(&value)?);
            },
            "table" => {
                let headers = vec!["Key", "Value"];
                let rows = vec![vec![key, &format_json(&value)?]];
                println!("{}", format_table(headers, rows)?);
            },
            _ => {
                return Err(CliError::InvalidFormat(format.to_string()));
            }
        }
        
        Ok(())
    }
    
    fn execute_set(&self, matches: &ArgMatches, config_manager: &ConfigManager) -> Result<(), CliError> {
        // Get the key and value
        let key = matches.value_of("key").unwrap();
        let value_str = matches.value_of("value").unwrap();
        
        // Parse the value as JSON
        let value = serde_json::from_str(value_str)
            .map_err(|e| CliError::InvalidJson(format!("Invalid JSON value: {}", e)))?;
        
        // Set the configuration value
        config_manager.set(key, &value)
            .map_err(|e| CliError::ConfigError(format!("Failed to set config value: {}", e)))?;
        
        // Save the configuration
        config_manager.save()
            .map_err(|e| CliError::ConfigError(format!("Failed to save config: {}", e)))?;
        
        println!("Configuration value set successfully");
        
        Ok(())
    }
    
    fn execute_list(&self, matches: &ArgMatches, config_manager: &ConfigManager) -> Result<(), CliError> {
        // Get the prefix
        let prefix = matches.value_of("prefix").unwrap_or("");
        
        // Get the format
        let format = matches.value_of("format").unwrap();
        
        // Get the configuration keys
        let keys = config_manager.list_keys(prefix)
            .map_err(|e| CliError::ConfigError(format!("Failed to list config keys: {}", e)))?;
        
        // Format and print the keys
        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&keys)?);
            },
            "table" => {
                let headers = vec!["Key"];
                let rows: Vec<Vec<&str>> = keys.iter().map(|key| vec![key.as_str()]).collect();
                println!("{}", format_table(headers, rows)?);
            },
            _ => {
                return Err(CliError::InvalidFormat(format.to_string()));
            }
        }
        
        Ok(())
    }
}
```

### Example: Implementing a Hardware Command

```rust
// src/commands/hardware.rs
use clap::{App, Arg, ArgMatches, SubCommand};
use vr_core_api::hardware::{DeviceManager, DeviceType, DeviceError};
use crate::commands::Command;
use crate::utils::error::CliError;
use crate::utils::formatting::{format_table, format_json};

pub struct HardwareCommand;

impl Command for HardwareCommand {
    fn name(&self) -> &'static str {
        "hardware"
    }
    
    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name())
            .about("Manage hardware devices")
            .subcommand(
                SubCommand::with_name("list")
                    .about("List hardware devices")
                    .arg(
                        Arg::with_name("type")
                            .help("Device type filter")
                            .long("type")
                            .short("t")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("format")
                            .help("Output format")
                            .long("format")
                            .short("f")
                            .possible_values(&["json", "table"])
                            .default_value("table")
                    )
            )
            .subcommand(
                SubCommand::with_name("info")
                    .about("Get device information")
                    .arg(
                        Arg::with_name("id")
                            .help("Device ID")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        Arg::with_name("format")
                            .help("Output format")
                            .long("format")
                            .short("f")
                            .possible_values(&["json", "table"])
                            .default_value("table")
                    )
            )
            .subcommand(
                SubCommand::with_name("configure")
                    .about("Configure a device")
                    .arg(
                        Arg::with_name("id")
                            .help("Device ID")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        Arg::with_name("config")
                            .help("Configuration (JSON format)")
                            .required(true)
                            .index(2)
                    )
            )
    }
    
    fn execute(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Create a device manager
        let device_manager = DeviceManager::new()
            .map_err(|e| CliError::HardwareError(format!("Failed to create device manager: {}", e)))?;
        
        // Discover devices
        device_manager.discover_devices()
            .map_err(|e| CliError::HardwareError(format!("Failed to discover devices: {}", e)))?;
        
        // Handle subcommands
        match matches.subcommand() {
            ("list", Some(sub_matches)) => {
                self.execute_list(sub_matches, &device_manager)
            },
            ("info", Some(sub_matches)) => {
                self.execute_info(sub_matches, &device_manager)
            },
            ("configure", Some(sub_matches)) => {
                self.execute_configure(sub_matches, &device_manager)
            },
            _ => {
                Err(CliError::InvalidSubcommand(self.name().to_string()))
            }
        }
    }
}

impl HardwareCommand {
    fn execute_list(&self, matches: &ArgMatches, device_manager: &DeviceManager) -> Result<(), CliError> {
        // Get the device type filter
        let device_type = matches.value_of("type").map(|t| {
            match t {
                "display" => DeviceType::Display,
                "audio" => DeviceType::Audio,
                "tracking" => DeviceType::Tracking,
                "power" => DeviceType::Power,
                "storage" => DeviceType::Storage,
                "network" => DeviceType::Network,
                _ => DeviceType::Unknown,
            }
        });
        
        // Get the format
        let format = matches.value_of("format").unwrap();
        
        // Get the devices
        let devices = if let Some(device_type) = device_type {
            device_manager.get_devices_by_type(device_type)
                .map_err(|e| CliError::HardwareError(format!("Failed to get devices: {}", e)))?
        } else {
            device_manager.get_all_devices()
                .map_err(|e| CliError::HardwareError(format!("Failed to get devices: {}", e)))?
        };
        
        // Format and print the devices
        match format {
            "json" => {
                let device_infos: Vec<_> = devices.iter().map(|device| {
                    let info = device.get_info();
                    serde_json::json!({
                        "id": info.id,
                        "name": info.name,
                        "type": format!("{:?}", info.device_type),
                        "vendor": info.vendor,
                        "model": info.model,
                        "available": device.is_available(),
                    })
                }).collect();
                
                println!("{}", serde_json::to_string_pretty(&device_infos)?);
            },
            "table" => {
                let headers = vec!["ID", "Name", "Type", "Vendor", "Model", "Available"];
                let rows: Vec<Vec<String>> = devices.iter().map(|device| {
                    let info = device.get_info();
                    vec![
                        info.id.clone(),
                        info.name.clone(),
                        format!("{:?}", info.device_type),
                        info.vendor.clone(),
                        info.model.clone(),
                        if device.is_available() { "Yes".to_string() } else { "No".to_string() },
                    ]
                }).collect();
                
                println!("{}", format_table(headers, rows.iter().map(|row| {
                    row.iter().map(|s| s.as_str()).collect()
                }).collect())?);
            },
            _ => {
                return Err(CliError::InvalidFormat(format.to_string()));
            }
        }
        
        Ok(())
    }
    
    fn execute_info(&self, matches: &ArgMatches, device_manager: &DeviceManager) -> Result<(), CliError> {
        // Get the device ID
        let id = matches.value_of("id").unwrap();
        
        // Get the format
        let format = matches.value_of("format").unwrap();
        
        // Get the device
        let device = device_manager.get_device(id)
            .map_err(|e| CliError::HardwareError(format!("Failed to get device: {}", e)))?;
        
        // Get the device info
        let info = device.get_info();
        
        // Format and print the device info
        match format {
            "json" => {
                let device_info = serde_json::json!({
                    "id": info.id,
                    "name": info.name,
                    "type": format!("{:?}", info.device_type),
                    "vendor": info.vendor,
                    "model": info.model,
                    "available": device.is_available(),
                });
                
                println!("{}", serde_json::to_string_pretty(&device_info)?);
            },
            "table" => {
                let headers = vec!["Property", "Value"];
                let rows = vec![
                    vec!["ID", &info.id],
                    vec!["Name", &info.name],
                    vec!["Type", &format!("{:?}", info.device_type)],
                    vec!["Vendor", &info.vendor],
                    vec!["Model", &info.model],
                    vec!["Available", if device.is_available() { "Yes" } else { "No" }],
                ];
                
                println!("{}", format_table(headers, rows)?);
            },
            _ => {
                return Err(CliError::InvalidFormat(format.to_string()));
            }
        }
        
        Ok(())
    }
    
    fn execute_configure(&self, matches: &ArgMatches, device_manager: &DeviceManager) -> Result<(), CliError> {
        // Get the device ID and configuration
        let id = matches.value_of("id").unwrap();
        let config_str = matches.value_of("config").unwrap();
        
        // Parse the configuration as JSON
        let config = serde_json::from_str(config_str)
            .map_err(|e| CliError::InvalidJson(format!("Invalid JSON configuration: {}", e)))?;
        
        // Get the device
        let device = device_manager.get_device(id)
            .map_err(|e| CliError::HardwareError(format!("Failed to get device: {}", e)))?;
        
        // Configure the device
        device.configure(&config)
            .map_err(|e| CliError::HardwareError(format!("Failed to configure device: {}", e)))?;
        
        println!("Device configured successfully");
        
        Ok(())
    }
}
```

### Main Application

The main application is responsible for parsing command-line arguments and dispatching commands.

```rust
// src/main.rs
use clap::App;
use std::process;
use vr_cli::commands::CommandRegistry;
use vr_cli::utils::error::CliError;

fn main() {
    // Create the command registry
    let registry = CommandRegistry::new();
    
    // Create the command-line application
    let app = registry.app();
    
    // Parse command-line arguments
    let matches = app.get_matches();
    
    // Execute the command
    match registry.execute(matches) {
        Ok(_) => {
            // Command executed successfully
            process::exit(0);
        },
        Err(err) => {
            // Command execution failed
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}
```

## Error Handling

The CLI uses a centralized error handling system to ensure consistent error reporting.

```rust
// src/utils/error.rs
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum CliError {
    CommandNotFound(String),
    InvalidSubcommand(String),
    InvalidArgument(String),
    InvalidFormat(String),
    InvalidJson(String),
    ConfigError(String),
    HardwareError(String),
    IpcError(String),
    MonitoringError(String),
    ScriptError(String),
    SecurityError(String),
    SystemError(String),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            CliError::InvalidSubcommand(cmd) => write!(f, "Invalid subcommand for {}", cmd),
            CliError::InvalidArgument(arg) => write!(f, "Invalid argument: {}", arg),
            CliError::InvalidFormat(fmt) => write!(f, "Invalid format: {}", fmt),
            CliError::InvalidJson(err) => write!(f, "Invalid JSON: {}", err),
            CliError::ConfigError(err) => write!(f, "Configuration error: {}", err),
            CliError::HardwareError(err) => write!(f, "Hardware error: {}", err),
            CliError::IpcError(err) => write!(f, "IPC error: {}", err),
            CliError::MonitoringError(err) => write!(f, "Monitoring error: {}", err),
            CliError::ScriptError(err) => write!(f, "Script error: {}", err),
            CliError::SecurityError(err) => write!(f, "Security error: {}", err),
            CliError::SystemError(err) => write!(f, "System error: {}", err),
            CliError::IoError(err) => write!(f, "I/O error: {}", err),
            CliError::SerdeError(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::SerdeError(err)
    }
}
```

## Output Formatting

The CLI provides utilities for formatting command output in different formats.

```rust
// src/utils/formatting.rs
use prettytable::{Table, Row, Cell};
use serde_json::Value;
use crate::utils::error::CliError;

pub fn format_json(value: &Value) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|e| CliError::SerdeError(e))
}

pub fn format_table<'a>(headers: Vec<&'a str>, rows: Vec<Vec<&'a str>>) -> Result<String, CliError> {
    let mut table = Table::new();
    
    // Add headers
    table.add_row(Row::new(
        headers.into_iter().map(|h| Cell::new(h)).collect()
    ));
    
    // Add rows
    for row in rows {
        table.add_row(Row::new(
            row.into_iter().map(|c| Cell::new(c)).collect()
        ));
    }
    
    Ok(table.to_string())
}
```

## Script Support

The CLI supports executing scripts for automation and batch processing.

```rust
// src/commands/script.rs
use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::commands::Command;
use crate::utils::error::CliError;
use crate::utils::script::ScriptExecutor;

pub struct ScriptCommand;

impl Command for ScriptCommand {
    fn name(&self) -> &'static str {
        "script"
    }
    
    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name())
            .about("Execute scripts")
            .subcommand(
                SubCommand::with_name("run")
                    .about("Run a script file")
                    .arg(
                        Arg::with_name("file")
                            .help("Script file path")
                            .required(true)
                            .index(1)
                    )
            )
    }
    
    fn execute(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Handle subcommands
        match matches.subcommand() {
            ("run", Some(sub_matches)) => {
                self.execute_run(sub_matches)
            },
            _ => {
                Err(CliError::InvalidSubcommand(self.name().to_string()))
            }
        }
    }
}

impl ScriptCommand {
    fn execute_run(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Get the script file path
        let file_path = matches.value_of("file").unwrap();
        
        // Check if the file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(CliError::ScriptError(format!("Script file not found: {}", file_path)));
        }
        
        // Open the file
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // Create a script executor
        let mut executor = ScriptExecutor::new();
        
        // Execute each line in the script
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip empty lines and comments
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            // Execute the line
            if let Err(err) = executor.execute_line(&line) {
                return Err(CliError::ScriptError(
                    format!("Error at line {}: {}", line_number + 1, err)
                ));
            }
        }
        
        println!("Script executed successfully");
        
        Ok(())
    }
}
```

```rust
// src/utils/script.rs
use std::process::Command;
use crate::utils::error::CliError;

pub struct ScriptExecutor {
    // Add any state needed for script execution
}

impl ScriptExecutor {
    pub fn new() -> Self {
        ScriptExecutor {}
    }
    
    pub fn execute_line(&mut self, line: &str) -> Result<(), CliError> {
        // Parse the line into a command and arguments
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        let command = parts[0];
        let args = &parts[1..];
        
        // Execute the command
        match command {
            "echo" => {
                // Simple echo command
                println!("{}", args.join(" "));
                Ok(())
            },
            "sleep" => {
                // Sleep command
                if args.len() != 1 {
                    return Err(CliError::ScriptError(
                        "sleep command requires exactly one argument".to_string()
                    ));
                }
                
                let seconds = args[0].parse::<u64>()
                    .map_err(|_| CliError::ScriptError(
                        format!("Invalid sleep duration: {}", args[0])
                    ))?;
                
                std::thread::sleep(std::time::Duration::from_secs(seconds));
                Ok(())
            },
            "vr-cli" => {
                // Execute a vr-cli command
                let status = Command::new("vr-cli")
                    .args(args)
                    .status()
                    .map_err(|e| CliError::IoError(e))?;
                
                if !status.success() {
                    return Err(CliError::ScriptError(
                        format!("Command failed with exit code: {}", status)
                    ));
                }
                
                Ok(())
            },
            _ => {
                // Unknown command
                Err(CliError::ScriptError(format!("Unknown command: {}", command)))
            }
        }
    }
}
```

## Integration with Core API

The CLI integrates with the Core API to provide a text-based interface for all system functionality.

### Example: System Information Command

```rust
// src/commands/system.rs
use clap::{App, Arg, ArgMatches, SubCommand};
use vr_core_api::monitoring::{SystemMonitor, SystemInfo, MonitoringError};
use crate::commands::Command;
use crate::utils::error::CliError;
use crate::utils::formatting::{format_table, format_json};

pub struct SystemCommand;

impl Command for SystemCommand {
    fn name(&self) -> &'static str {
        "system"
    }
    
    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name())
            .about("Manage system")
            .subcommand(
                SubCommand::with_name("info")
                    .about("Get system information")
                    .arg(
                        Arg::with_name("format")
                            .help("Output format")
                            .long("format")
                            .short("f")
                            .possible_values(&["json", "table"])
                            .default_value("table")
                    )
            )
            .subcommand(
                SubCommand::with_name("reboot")
                    .about("Reboot the system")
                    .arg(
                        Arg::with_name("force")
                            .help("Force reboot")
                            .long("force")
                            .short("f")
                    )
            )
            .subcommand(
                SubCommand::with_name("shutdown")
                    .about("Shutdown the system")
                    .arg(
                        Arg::with_name("force")
                            .help("Force shutdown")
                            .long("force")
                            .short("f")
                    )
            )
    }
    
    fn execute(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Handle subcommands
        match matches.subcommand() {
            ("info", Some(sub_matches)) => {
                self.execute_info(sub_matches)
            },
            ("reboot", Some(sub_matches)) => {
                self.execute_reboot(sub_matches)
            },
            ("shutdown", Some(sub_matches)) => {
                self.execute_shutdown(sub_matches)
            },
            _ => {
                Err(CliError::InvalidSubcommand(self.name().to_string()))
            }
        }
    }
}

impl SystemCommand {
    fn execute_info(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Get the format
        let format = matches.value_of("format").unwrap();
        
        // Create a system monitor
        let monitor = SystemMonitor::new()
            .map_err(|e| CliError::MonitoringError(format!("Failed to create system monitor: {}", e)))?;
        
        // Get system information
        let info = monitor.get_system_info()
            .map_err(|e| CliError::MonitoringError(format!("Failed to get system info: {}", e)))?;
        
        // Format and print the system information
        match format {
            "json" => {
                let system_info = serde_json::json!({
                    "hostname": info.hostname,
                    "os_name": info.os_name,
                    "os_version": info.os_version,
                    "kernel_version": info.kernel_version,
                    "cpu_model": info.cpu_model,
                    "cpu_cores": info.cpu_cores,
                    "memory_total": info.memory_total,
                    "memory_free": info.memory_free,
                    "uptime": info.uptime,
                });
                
                println!("{}", serde_json::to_string_pretty(&system_info)?);
            },
            "table" => {
                let headers = vec!["Property", "Value"];
                let rows = vec![
                    vec!["Hostname", &info.hostname],
                    vec!["OS Name", &info.os_name],
                    vec!["OS Version", &info.os_version],
                    vec!["Kernel Version", &info.kernel_version],
                    vec!["CPU Model", &info.cpu_model],
                    vec!["CPU Cores", &info.cpu_cores.to_string()],
                    vec!["Memory Total", &format!("{} MB", info.memory_total / 1024 / 1024)],
                    vec!["Memory Free", &format!("{} MB", info.memory_free / 1024 / 1024)],
                    vec!["Uptime", &format!("{} seconds", info.uptime)],
                ];
                
                println!("{}", format_table(headers, rows)?);
            },
            _ => {
                return Err(CliError::InvalidFormat(format.to_string()));
            }
        }
        
        Ok(())
    }
    
    fn execute_reboot(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Get the force flag
        let force = matches.is_present("force");
        
        // Create a system monitor
        let monitor = SystemMonitor::new()
            .map_err(|e| CliError::MonitoringError(format!("Failed to create system monitor: {}", e)))?;
        
        // Reboot the system
        monitor.reboot(force)
            .map_err(|e| CliError::SystemError(format!("Failed to reboot system: {}", e)))?;
        
        println!("System is rebooting...");
        
        Ok(())
    }
    
    fn execute_shutdown(&self, matches: &ArgMatches) -> Result<(), CliError> {
        // Get the force flag
        let force = matches.is_present("force");
        
        // Create a system monitor
        let monitor = SystemMonitor::new()
            .map_err(|e| CliError::MonitoringError(format!("Failed to create system monitor: {}", e)))?;
        
        // Shutdown the system
        monitor.shutdown(force)
            .map_err(|e| CliError::SystemError(format!("Failed to shutdown system: {}", e)))?;
        
        println!("System is shutting down...");
        
        Ok(())
    }
}
```

## Best Practices for CLI Development

### Command Design

1. **Consistent Interface**:
   - Use consistent command and subcommand naming
   - Use consistent option naming and formatting
   - Provide help text for all commands and options
   - Use appropriate default values

2. **Error Handling**:
   - Use a centralized error handling system
   - Provide clear error messages
   - Include context in error messages
   - Return appropriate exit codes

3. **Output Formatting**:
   - Support multiple output formats (table, JSON, etc.)
   - Use consistent formatting across commands
   - Make output machine-readable when needed
   - Provide appropriate verbosity levels

4. **Performance**:
   - Minimize startup time
   - Use lazy loading for expensive operations
   - Implement caching where appropriate
   - Optimize for common use cases

### Script Support

1. **Script Syntax**:
   - Use a simple, line-based syntax
   - Support comments and empty lines
   - Provide error handling and reporting
   - Support variables and control flow

2. **Script Execution**:
   - Execute scripts line by line
   - Provide context in error messages
   - Support both interactive and batch modes
   - Implement proper error handling

3. **Script Examples**:
   - Provide example scripts for common tasks
   - Document script syntax and commands
   - Include comments in example scripts
   - Test scripts on different platforms

## Troubleshooting

### Common Issues

1. **Command Not Found**:
   - Check the command name and spelling
   - Verify that the command is registered
   - Check for typos in the command name

2. **Invalid Arguments**:
   - Check the argument names and values
   - Verify that required arguments are provided
   - Check for typos in argument names

3. **Permission Issues**:
   - Check file and directory permissions
   - Run with appropriate privileges when needed
   - Check for permission issues in the Core API

4. **Configuration Issues**:
   - Check configuration file syntax
   - Verify that configuration files exist
   - Check for permission issues with configuration files

### Debugging Techniques

1. **Verbose Output**:
   - Use verbose output flags for more information
   - Check error messages and context
   - Look for specific error codes

2. **Logging**:
   - Enable debug logging for detailed information
   - Check log files for error messages
   - Use log filtering to focus on relevant components

3. **Manual Testing**:
   - Test commands individually
   - Use simple test cases
   - Verify expected behavior

## Conclusion

The CLI provides a powerful and flexible way to manage the VR headset system from the command line. By following the guidelines in this document, you can create robust, user-friendly CLI commands that integrate seamlessly with the Core API.

For more information, refer to the API documentation and example code provided with the CLI.
