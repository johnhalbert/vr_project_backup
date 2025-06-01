use clap::{Parser, Subcommand, Command, ArgAction, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use colored::Colorize;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::io;

mod commands;
mod utils;

/// VR Headset Command Line Interface
#[derive(Parser)]
#[command(name = "vr")]
#[command(author = "VR Headset Team")]
#[command(version = "0.1.0")]
#[command(about = "Command line interface for VR headset management")]
#[command(long_about = "A comprehensive command line interface for managing and configuring the VR headset system. This tool provides access to hardware control, configuration management, system monitoring, and more.")]
#[command(propagate_version = true)]
struct Cli {
    /// Optional config file path
    #[arg(short, long, value_name = "FILE", env = "VR_CONFIG_FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on (repeat for more verbosity)
    #[arg(short, long, action = ArgAction::Count)]
    debug: u8,

    /// Generate shell completions
    #[arg(long, value_enum)]
    generate_completion: Option<Shell>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration management commands
    #[command(alias = "cfg", visible_alias = "conf")]
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    
    /// Hardware management commands
    #[command(alias = "hw", visible_alias = "device")]
    Hardware {
        #[command(subcommand)]
        action: HardwareCommands,
    },
    
    /// System management commands
    #[command(alias = "sys", visible_alias = "system-control")]
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },
    
    /// Monitoring and diagnostics commands
    #[command(alias = "mon", visible_alias = "metrics")]
    Monitoring {
        #[command(subcommand)]
        action: MonitoringCommands,
    },
    
    /// Inter-process communication commands
    #[command(alias = "ipc", visible_alias = "comm")]
    Ipc {
        #[command(subcommand)]
        action: IpcCommands,
    },
    
    /// Security and authentication commands
    #[command(alias = "sec", visible_alias = "auth")]
    Security {
        #[command(subcommand)]
        action: SecurityCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// List all configuration values
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Output format (table, json, toml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Get a specific configuration value
    #[command(alias = "g", visible_alias = "read")]
    Get {
        /// Configuration category
        #[arg(required = true)]
        category: String,
        
        /// Configuration key
        #[arg(required = true)]
        key: String,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Set a configuration value
    #[command(alias = "s", visible_alias = "write")]
    Set {
        /// Configuration category
        #[arg(required = true)]
        category: String,
        
        /// Configuration key
        #[arg(required = true)]
        key: String,
        
        /// Configuration value
        #[arg(required = true)]
        value: String,
        
        /// Value type (string, integer, float, boolean)
        #[arg(short, long, default_value = "string")]
        type_: String,
        
        /// Don't save changes to disk
        #[arg(long)]
        no_save: bool,
    },
    
    /// Reset configuration to defaults
    #[command(alias = "rst", visible_alias = "default")]
    Reset {
        /// Reset only a specific category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Export configuration to a file
    #[command(alias = "exp", visible_alias = "save")]
    Export {
        /// Output file path
        #[arg(required = true)]
        file: PathBuf,
        
        /// Export format (toml, json)
        #[arg(short, long, default_value = "toml")]
        format: String,
        
        /// Export only a specific category
        #[arg(short, long)]
        category: Option<String>,
    },
    
    /// Import configuration from a file
    #[command(alias = "imp", visible_alias = "load")]
    Import {
        /// Input file path
        #[arg(required = true)]
        file: PathBuf,
        
        /// Import format (toml, json)
        #[arg(short, long, default_value = "toml")]
        format: String,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Compare configuration with another file or defaults
    #[command(alias = "diff", visible_alias = "compare")]
    Compare {
        /// File to compare with (if not specified, compares with defaults)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Compare only a specific category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Output format (text, table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Search for configuration keys or values
    #[command(alias = "find", visible_alias = "grep")]
    Search {
        /// Search term
        #[arg(required = true)]
        term: String,
        
        /// Search in specific category
        #[arg(short, long)]
        category: Option<String>,
        
        /// Search in keys only
        #[arg(short, long)]
        keys_only: bool,
        
        /// Search in values only
        #[arg(short, long)]
        values_only: bool,
    },
}

#[derive(Subcommand)]
enum HardwareCommands {
    /// List all hardware devices
    #[command(alias = "ls", visible_alias = "devices")]
    List {
        /// Filter by device type
        #[arg(short, long)]
        type_: Option<String>,
        
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Get details for a specific device
    #[command(alias = "i", visible_alias = "details")]
    Info {
        /// Device name
        #[arg(required = true)]
        name: String,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Initialize hardware devices
    #[command(alias = "init", visible_alias = "start")]
    Init {
        /// Initialize only a specific device
        #[arg(short, long)]
        device: Option<String>,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Shutdown hardware devices
    #[command(alias = "shut", visible_alias = "stop")]
    Shutdown {
        /// Shutdown only a specific device
        #[arg(short, long)]
        device: Option<String>,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Run diagnostics on hardware devices
    #[command(alias = "diag", visible_alias = "test")]
    Diagnose {
        /// Run diagnostics only on a specific device
        #[arg(short, long)]
        device: Option<String>,
        
        /// Diagnostic level (basic, advanced, full)
        #[arg(short, long, default_value = "basic")]
        level: String,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Update firmware for hardware devices
    #[command(alias = "fw", visible_alias = "upgrade")]
    Firmware {
        /// Update firmware for a specific device
        #[arg(short, long)]
        device: Option<String>,
        
        /// Path to firmware file
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Check for updates only, don't install
        #[arg(short, long)]
        check_only: bool,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Calibrate hardware devices
    #[command(alias = "cal", visible_alias = "adjust")]
    Calibrate {
        /// Calibrate a specific device
        #[arg(required = true)]
        device: String,
        
        /// Calibration type (depends on device)
        #[arg(short, long)]
        type_: Option<String>,
        
        /// Interactive calibration
        #[arg(short, long)]
        interactive: bool,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Show system status
    #[command(alias = "st", visible_alias = "health")]
    Status {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Show system information
    #[command(alias = "i", visible_alias = "about")]
    Info {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Restart the VR system
    #[command(alias = "rst", visible_alias = "reboot")]
    Restart {
        /// Force restart even if devices are in use
        #[arg(short, long)]
        force: bool,
        
        /// Restart after a delay (in seconds)
        #[arg(short, long)]
        delay: Option<u32>,
    },
    
    /// Update the VR system
    #[command(alias = "upd", visible_alias = "upgrade")]
    Update {
        /// Check for updates only, don't install
        #[arg(short, long)]
        check_only: bool,
        
        /// Update to a specific version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Manage system logs
    #[command(alias = "log", visible_alias = "journal")]
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Filter by log level (debug, info, warn, error)
        #[arg(short, long)]
        level: Option<String>,
        
        /// Filter by component
        #[arg(short, long)]
        component: Option<String>,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Backup system configuration and data
    #[command(alias = "bak", visible_alias = "save")]
    Backup {
        /// Output file path
        #[arg(required = true)]
        file: PathBuf,
        
        /// Backup type (config, data, full)
        #[arg(short, long, default_value = "config")]
        type_: String,
    },
    
    /// Restore system configuration and data
    #[command(alias = "res", visible_alias = "load")]
    Restore {
        /// Input file path
        #[arg(required = true)]
        file: PathBuf,
        
        /// Restore type (config, data, full)
        #[arg(short, long, default_value = "config")]
        type_: String,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum MonitoringCommands {
    /// Show real-time system metrics
    #[command(alias = "rt", visible_alias = "dashboard")]
    Realtime {
        /// Update interval in seconds
        #[arg(short, long, default_value = "1")]
        interval: u32,
        
        /// Display mode (basic, detailed, full)
        #[arg(short, long, default_value = "basic")]
        mode: String,
    },
    
    /// Show resource usage statistics
    #[command(alias = "res", visible_alias = "usage")]
    Resources {
        /// Resource type (cpu, memory, disk, network, all)
        #[arg(short, long, default_value = "all")]
        type_: String,
        
        /// Output format (text, table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Manage system alerts
    #[command(alias = "alt", visible_alias = "warnings")]
    Alerts {
        #[command(subcommand)]
        action: AlertCommands,
    },
    
    /// Show performance statistics
    #[command(alias = "perf", visible_alias = "stats")]
    Performance {
        /// Performance metric (latency, throughput, fps, all)
        #[arg(short, long, default_value = "all")]
        metric: String,
        
        /// Time period (1m, 5m, 1h, 24h)
        #[arg(short, long, default_value = "5m")]
        period: String,
        
        /// Output format (text, table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Export monitoring data
    #[command(alias = "exp", visible_alias = "save")]
    Export {
        /// Output file path
        #[arg(required = true)]
        file: PathBuf,
        
        /// Data type (metrics, alerts, performance, all)
        #[arg(short, long, default_value = "all")]
        type_: String,
        
        /// Time period (1h, 24h, 7d, 30d)
        #[arg(short, long, default_value = "24h")]
        period: String,
        
        /// Output format (csv, json)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },
}

#[derive(Subcommand)]
enum AlertCommands {
    /// List all alerts
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Filter by severity (info, warning, error, critical)
        #[arg(short, long)]
        severity: Option<String>,
        
        /// Filter by component
        #[arg(short, long)]
        component: Option<String>,
        
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Configure alert thresholds
    #[command(alias = "cfg", visible_alias = "thresholds")]
    Configure {
        /// Alert type
        #[arg(required = true)]
        type_: String,
        
        /// Warning threshold
        #[arg(short, long)]
        warning: Option<String>,
        
        /// Error threshold
        #[arg(short, long)]
        error: Option<String>,
        
        /// Critical threshold
        #[arg(short, long)]
        critical: Option<String>,
    },
    
    /// Acknowledge alerts
    #[command(alias = "ack", visible_alias = "clear")]
    Acknowledge {
        /// Alert ID (or "all" for all alerts)
        #[arg(required = true)]
        id: String,
    },
}

#[derive(Subcommand)]
enum IpcCommands {
    /// Manage Unix domain sockets
    #[command(alias = "uds", visible_alias = "socket")]
    Unix {
        #[command(subcommand)]
        action: UnixSocketCommands,
    },
    
    /// Manage D-Bus services
    #[command(alias = "db", visible_alias = "dbus")]
    Dbus {
        #[command(subcommand)]
        action: DbusCommands,
    },
    
    /// Manage WebSocket server
    #[command(alias = "ws", visible_alias = "websocket")]
    Websocket {
        #[command(subcommand)]
        action: WebsocketCommands,
    },
    
    /// Test IPC performance
    #[command(alias = "perf", visible_alias = "benchmark")]
    Performance {
        /// IPC type (unix, dbus, websocket, all)
        #[arg(short, long, default_value = "all")]
        type_: String,
        
        /// Test duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u32,
        
        /// Message size in bytes
        #[arg(short, long, default_value = "1024")]
        size: usize,
    },
}

#[derive(Subcommand)]
enum UnixSocketCommands {
    /// List all Unix domain sockets
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Get details for a specific socket
    #[command(alias = "i", visible_alias = "details")]
    Info {
        /// Socket name
        #[arg(required = true)]
        name: String,
    },
    
    /// Send a message to a socket
    #[command(alias = "s", visible_alias = "write")]
    Send {
        /// Socket name
        #[arg(required = true)]
        name: String,
        
        /// Message content
        #[arg(required = true)]
        message: String,
        
        /// Message format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum DbusCommands {
    /// List all D-Bus services
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Get details for a specific service
    #[command(alias = "i", visible_alias = "details")]
    Info {
        /// Service name
        #[arg(required = true)]
        name: String,
    },
    
    /// Call a D-Bus method
    #[command(alias = "c", visible_alias = "invoke")]
    Call {
        /// Service name
        #[arg(required = true)]
        service: String,
        
        /// Object path
        #[arg(required = true)]
        path: String,
        
        /// Interface name
        #[arg(required = true)]
        interface: String,
        
        /// Method name
        #[arg(required = true)]
        method: String,
        
        /// Method arguments (in JSON format)
        #[arg(short, long)]
        args: Option<String>,
    },
}

#[derive(Subcommand)]
enum WebsocketCommands {
    /// Show WebSocket server status
    #[command(alias = "st", visible_alias = "info")]
    Status {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// List all WebSocket connections
    #[command(alias = "ls", visible_alias = "connections")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Send a message to all clients
    #[command(alias = "s", visible_alias = "broadcast")]
    Send {
        /// Message content
        #[arg(required = true)]
        message: String,
        
        /// Message format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum SecurityCommands {
    /// Manage user authentication
    #[command(alias = "auth", visible_alias = "users")]
    Authentication {
        #[command(subcommand)]
        action: AuthCommands,
    },
    
    /// Manage access control
    #[command(alias = "acl", visible_alias = "permissions")]
    Authorization {
        #[command(subcommand)]
        action: AuthzCommands,
    },
    
    /// Manage encryption
    #[command(alias = "enc", visible_alias = "crypto")]
    Encryption {
        #[command(subcommand)]
        action: EncryptionCommands,
    },
    
    /// View security audit logs
    #[command(alias = "log", visible_alias = "audit")]
    Logs {
        /// Number of entries to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Filter by event type
        #[arg(short, long)]
        type_: Option<String>,
        
        /// Filter by user
        #[arg(short, long)]
        user: Option<String>,
        
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// List all users
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Add a new user
    #[command(alias = "a", visible_alias = "create")]
    Add {
        /// Username
        #[arg(required = true)]
        username: String,
        
        /// Password (if not provided, will prompt)
        #[arg(short, long)]
        password: Option<String>,
        
        /// User role (admin, user, guest)
        #[arg(short, long, default_value = "user")]
        role: String,
    },
    
    /// Remove a user
    #[command(alias = "rm", visible_alias = "delete")]
    Remove {
        /// Username
        #[arg(required = true)]
        username: String,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Change user password
    #[command(alias = "pw", visible_alias = "passwd")]
    Password {
        /// Username
        #[arg(required = true)]
        username: String,
        
        /// New password (if not provided, will prompt)
        #[arg(short, long)]
        password: Option<String>,
    },
}

#[derive(Subcommand)]
enum AuthzCommands {
    /// List all roles
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Show permissions for a role
    #[command(alias = "p", visible_alias = "perms")]
    Permissions {
        /// Role name
        #[arg(required = true)]
        role: String,
        
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Grant permission to a role
    #[command(alias = "g", visible_alias = "allow")]
    Grant {
        /// Role name
        #[arg(required = true)]
        role: String,
        
        /// Permission name
        #[arg(required = true)]
        permission: String,
    },
    
    /// Revoke permission from a role
    #[command(alias = "r", visible_alias = "deny")]
    Revoke {
        /// Role name
        #[arg(required = true)]
        role: String,
        
        /// Permission name
        #[arg(required = true)]
        permission: String,
    },
}

#[derive(Subcommand)]
enum EncryptionCommands {
    /// Show encryption status
    #[command(alias = "st", visible_alias = "info")]
    Status {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Manage encryption keys
    #[command(alias = "k", visible_alias = "keys")]
    Keys {
        #[command(subcommand)]
        action: KeyCommands,
    },
    
    /// Encrypt a file
    #[command(alias = "e", visible_alias = "protect")]
    Encrypt {
        /// Input file path
        #[arg(required = true)]
        input: PathBuf,
        
        /// Output file path
        #[arg(required = true)]
        output: PathBuf,
        
        /// Key ID to use
        #[arg(short, long)]
        key: Option<String>,
    },
    
    /// Decrypt a file
    #[command(alias = "d", visible_alias = "unprotect")]
    Decrypt {
        /// Input file path
        #[arg(required = true)]
        input: PathBuf,
        
        /// Output file path
        #[arg(required = true)]
        output: PathBuf,
        
        /// Key ID to use
        #[arg(short, long)]
        key: Option<String>,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// List all encryption keys
    #[command(alias = "ls", visible_alias = "show")]
    List {
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Generate a new encryption key
    #[command(alias = "g", visible_alias = "create")]
    Generate {
        /// Key name
        #[arg(required = true)]
        name: String,
        
        /// Key type (aes, rsa)
        #[arg(short, long, default_value = "aes")]
        type_: String,
        
        /// Key size in bits
        #[arg(short, long)]
        size: Option<usize>,
    },
    
    /// Import an encryption key
    #[command(alias = "i", visible_alias = "load")]
    Import {
        /// Key name
        #[arg(required = true)]
        name: String,
        
        /// Key file path
        #[arg(required = true)]
        file: PathBuf,
    },
    
    /// Export an encryption key
    #[command(alias = "e", visible_alias = "save")]
    Export {
        /// Key name
        #[arg(required = true)]
        name: String,
        
        /// Output file path
        #[arg(required = true)]
        file: PathBuf,
    },
    
    /// Remove an encryption key
    #[command(alias = "rm", visible_alias = "delete")]
    Remove {
        /// Key name
        #[arg(required = true)]
        name: String,
        
        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle shell completion generation
    if let Some(shell) = cli.generate_completion {
        let mut cmd = Cli::command();
        print_completions(shell, &mut cmd);
        return Ok(());
    }
    
    // Initialize logging based on verbosity
    match cli.debug {
        0 => env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init(),
        1 => env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init(),
        _ => env_logger::Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .init(),
    }
    
    // If no command is provided, print help and exit
    if cli.command.is_none() {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!("\n");
        return Ok(());
    }
    
    // Initialize VR Core API
    let mut api = match vr_core_api::VRCoreAPI::with_config_path(
        cli.config.as_ref().map(|p| p.to_str().unwrap())
    ) {
        Ok(api) => api,
        Err(e) => {
            eprintln!("{}: Failed to initialize VR Core API: {}", "Error".red().bold(), e);
            eprintln!("This could be due to a missing or invalid configuration file, or the VR system not being properly installed.");
            eprintln!("Try specifying a configuration file with --config or check the system installation.");
            return Err(anyhow::anyhow!("Failed to initialize VR Core API: {}", e));
        }
    };
    
    // Process commands
    let result = match &cli.command.unwrap() {
        Commands::Config { action } => {
            commands::config::handle_command(action, &mut api)
        },
        Commands::Hardware { action } => {
            commands::hardware::handle_command(action, &mut api)
        },
        Commands::System { action } => {
            commands::system::handle_command(action, &mut api)
        },
        Commands::Monitoring { action } => {
            commands::monitoring::handle_command(action, &mut api)
        },
        Commands::Ipc { action } => {
            commands::ipc::handle_command(action, &mut api)
        },
        Commands::Security { action } => {
            commands::security::handle_command(action, &mut api)
        },
    };
    
    // Ensure proper shutdown of the API
    match api.shutdown() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}: Failed to cleanly shutdown VR Core API: {}", 
                      "Warning".yellow().bold(), 
                      e);
        }
    }
    
    // Handle command result
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            
            // Provide helpful error context and suggestions
            if let Some(ctx) = e.downcast_ref::<anyhow::Error>() {
                if let Some(source) = ctx.source() {
                    eprintln!("Caused by: {}", source);
                }
            }
            
            // Suggest solutions based on error type
            if e.to_string().contains("not found") {
                eprintln!("Suggestion: Check if the specified resource exists and try again.");
            } else if e.to_string().contains("permission") {
                eprintln!("Suggestion: You may need elevated permissions. Try running with sudo or check your user permissions.");
            } else if e.to_string().contains("connection") {
                eprintln!("Suggestion: Check if the VR system is running and properly connected.");
            }
            
            Err(e)
        }
    }
}
