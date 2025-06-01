//! Telemetry and logging module for the VR headset.
//!
//! This module provides functionality for collecting system telemetry,
//! managing logs, and providing insights while respecting user privacy.

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, Instant};
use anyhow::{Result, Context, anyhow, bail};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time;
use chrono::{DateTime, Utc, Local};
use log::{info, warn, error, debug, trace, LevelFilter};
use uuid::Uuid;

pub mod collection;
pub mod privacy;
pub mod anonymization;
pub mod rotation;
pub mod forwarding;
pub mod analysis;

/// Telemetry system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry collection is enabled.
    pub telemetry_enabled: bool,
    
    /// Privacy settings for telemetry.
    pub privacy_settings: PrivacySettings,
    
    /// Log rotation settings.
    pub log_rotation: LogRotationSettings,
    
    /// Log forwarding settings.
    pub log_forwarding: LogForwardingSettings,
    
    /// Path to store telemetry data.
    pub telemetry_path: PathBuf,
    
    /// Path to store log files.
    pub log_path: PathBuf,
    
    /// Maximum size of telemetry database in MB.
    pub max_telemetry_size_mb: u32,
    
    /// How often to collect telemetry (in seconds).
    pub collection_interval_seconds: u32,
    
    /// Categories of telemetry to collect.
    pub enabled_categories: HashSet<TelemetryCategory>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        let mut enabled_categories = HashSet::new();
        enabled_categories.insert(TelemetryCategory::System);
        enabled_categories.insert(TelemetryCategory::Hardware);
        enabled_categories.insert(TelemetryCategory::Error);
        
        Self {
            telemetry_enabled: true,
            privacy_settings: PrivacySettings::default(),
            log_rotation: LogRotationSettings::default(),
            log_forwarding: LogForwardingSettings::default(),
            telemetry_path: PathBuf::from("/var/lib/vr-telemetry"),
            log_path: PathBuf::from("/var/log/vr-system"),
            max_telemetry_size_mb: 500,
            collection_interval_seconds: 300, // 5 minutes
            enabled_categories,
        }
    }
}

/// Privacy settings for telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Whether to collect usage statistics.
    pub collect_usage_statistics: bool,
    
    /// Whether to collect crash reports.
    pub collect_crash_reports: bool,
    
    /// Whether to collect hardware diagnostics.
    pub collect_hardware_diagnostics: bool,
    
    /// Whether to collect performance metrics.
    pub collect_performance_metrics: bool,
    
    /// Whether to collect network diagnostics.
    pub collect_network_diagnostics: bool,
    
    /// Whether to collect application usage.
    pub collect_application_usage: bool,
    
    /// Whether to include location data.
    pub include_location_data: bool,
    
    /// Whether to include user identifiers.
    pub include_user_identifiers: bool,
    
    /// Whether to include device identifiers.
    pub include_device_identifiers: bool,
    
    /// Whether to include IP addresses.
    pub include_ip_addresses: bool,
    
    /// Custom data fields to exclude.
    pub excluded_fields: HashSet<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        let mut excluded_fields = HashSet::new();
        excluded_fields.insert("username".to_string());
        excluded_fields.insert("password".to_string());
        excluded_fields.insert("email".to_string());
        excluded_fields.insert("ssid".to_string());
        excluded_fields.insert("wifi_password".to_string());
        
        Self {
            collect_usage_statistics: true,
            collect_crash_reports: true,
            collect_hardware_diagnostics: true,
            collect_performance_metrics: true,
            collect_network_diagnostics: true,
            collect_application_usage: false,
            include_location_data: false,
            include_user_identifiers: false,
            include_device_identifiers: true,
            include_ip_addresses: false,
            excluded_fields,
        }
    }
}

/// Log rotation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationSettings {
    /// Whether to rotate logs based on size.
    pub rotate_by_size: bool,
    
    /// Maximum size of a log file before rotation (in MB).
    pub max_log_size_mb: u32,
    
    /// Whether to rotate logs based on time.
    pub rotate_by_time: bool,
    
    /// Interval for time-based rotation (in hours).
    pub rotation_interval_hours: u32,
    
    /// Number of log files to keep.
    pub files_to_keep: u32,
    
    /// Whether to compress rotated logs.
    pub compress_rotated_logs: bool,
    
    /// Whether to include timestamps in log filenames.
    pub timestamp_filenames: bool,
}

impl Default for LogRotationSettings {
    fn default() -> Self {
        Self {
            rotate_by_size: true,
            max_log_size_mb: 10,
            rotate_by_time: true,
            rotation_interval_hours: 24,
            files_to_keep: 7,
            compress_rotated_logs: true,
            timestamp_filenames: true,
        }
    }
}

/// Log forwarding settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogForwardingSettings {
    /// Whether to forward logs to a remote server.
    pub forward_logs: bool,
    
    /// URL of the log server.
    pub log_server_url: String,
    
    /// Authentication token for the log server.
    pub auth_token: String,
    
    /// Whether to use TLS for log forwarding.
    pub use_tls: bool,
    
    /// Whether to batch log messages.
    pub batch_logs: bool,
    
    /// Maximum batch size (in number of log entries).
    pub max_batch_size: u32,
    
    /// Maximum time to wait before sending a batch (in seconds).
    pub max_batch_wait_seconds: u32,
    
    /// Whether to retry failed transmissions.
    pub retry_on_failure: bool,
    
    /// Maximum number of retry attempts.
    pub max_retry_attempts: u32,
    
    /// Minimum log level to forward.
    pub min_level_to_forward: LogLevel,
}

impl Default for LogForwardingSettings {
    fn default() -> Self {
        Self {
            forward_logs: false,
            log_server_url: "https://logs.vrheadset.example.com".to_string(),
            auth_token: "".to_string(),
            use_tls: true,
            batch_logs: true,
            max_batch_size: 100,
            max_batch_wait_seconds: 60,
            retry_on_failure: true,
            max_retry_attempts: 3,
            min_level_to_forward: LogLevel::Warning,
        }
    }
}

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level.
    Trace,
    
    /// Debug level.
    Debug,
    
    /// Info level.
    Info,
    
    /// Warning level.
    Warning,
    
    /// Error level.
    Error,
    
    /// Critical level.
    Critical,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warning => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Critical => LevelFilter::Error, // No direct equivalent in log crate
        }
    }
}

/// Telemetry category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TelemetryCategory {
    /// System telemetry (CPU, memory, etc.).
    System,
    
    /// Hardware telemetry (temperature, fan speed, etc.).
    Hardware,
    
    /// Application telemetry (usage, performance, etc.).
    Application,
    
    /// Network telemetry (connectivity, bandwidth, etc.).
    Network,
    
    /// Error telemetry (crashes, exceptions, etc.).
    Error,
    
    /// User interaction telemetry (UI usage, etc.).
    UserInteraction,
    
    /// Custom telemetry category.
    Custom(String),
}

/// Telemetry data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDataPoint {
    /// Unique identifier for this data point.
    pub id: Uuid,
    
    /// Timestamp when this data point was collected.
    pub timestamp: DateTime<Utc>,
    
    /// Category of this data point.
    pub category: TelemetryCategory,
    
    /// Name of the metric.
    pub name: String,
    
    /// Value of the metric.
    pub value: TelemetryValue,
    
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Telemetry value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryValue {
    /// Boolean value.
    Boolean(bool),
    
    /// Integer value.
    Integer(i64),
    
    /// Float value.
    Float(f64),
    
    /// String value.
    String(String),
    
    /// Array of values.
    Array(Vec<TelemetryValue>),
    
    /// Map of values.
    Map(HashMap<String, TelemetryValue>),
}

/// Log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier for this log entry.
    pub id: Uuid,
    
    /// Timestamp when this log entry was created.
    pub timestamp: DateTime<Utc>,
    
    /// Log level.
    pub level: LogLevel,
    
    /// Source of the log (module, component, etc.).
    pub source: String,
    
    /// Log message.
    pub message: String,
    
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Telemetry manager.
pub struct TelemetryManager {
    /// Configuration for the telemetry system.
    config: RwLock<TelemetryConfig>,
    
    /// Channel for sending telemetry data points.
    telemetry_tx: mpsc::Sender<TelemetryDataPoint>,
    
    /// Channel for receiving telemetry data points.
    telemetry_rx: mpsc::Receiver<TelemetryDataPoint>,
    
    /// Channel for sending log entries.
    log_tx: mpsc::Sender<LogEntry>,
    
    /// Channel for receiving log entries.
    log_rx: mpsc::Receiver<LogEntry>,
    
    /// Database of collected telemetry.
    telemetry_db: Arc<Mutex<Vec<TelemetryDataPoint>>>,
    
    /// Last time telemetry was collected.
    last_collection_time: Arc<Mutex<Option<Instant>>>,
    
    /// Whether the telemetry system is running.
    running: Arc<RwLock<bool>>,
}

impl TelemetryManager {
    /// Create a new telemetry manager with the given configuration.
    pub async fn new(config: TelemetryConfig) -> Result<Self> {
        // Create necessary directories
        fs::create_dir_all(&config.telemetry_path)
            .context("Failed to create telemetry directory")?;
        fs::create_dir_all(&config.log_path)
            .context("Failed to create log directory")?;
            
        // Create channels for telemetry and logs
        let (telemetry_tx, telemetry_rx) = mpsc::channel(1000);
        let (log_tx, log_rx) = mpsc::channel(1000);
        
        // Load existing telemetry if available
        let telemetry_db_path = config.telemetry_path.join("telemetry.json");
        let telemetry_db = if telemetry_db_path.exists() {
            let file = File::open(&telemetry_db_path)
                .context("Failed to open telemetry database file")?;
            serde_json::from_reader(file)
                .context("Failed to parse telemetry database")?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            config: RwLock::new(config),
            telemetry_tx,
            telemetry_rx,
            log_tx,
            log_rx,
            telemetry_db: Arc::new(Mutex::new(telemetry_db)),
            last_collection_time: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the telemetry manager.
    pub async fn start(&self) -> Result<()> {
        // Set running flag
        {
            let mut running = self.running.write().unwrap();
            *running = true;
        }
        
        // Clone necessary values for background tasks
        let config = self.config.read().unwrap().clone();
        let telemetry_db = Arc::clone(&self.telemetry_db);
        let last_collection_time = Arc::clone(&self.last_collection_time);
        let running = Arc::clone(&self.running);
        let telemetry_tx = self.telemetry_tx.clone();
        
        // Start telemetry collection task
        tokio::spawn(async move {
            while *running.read().unwrap() {
                // Sleep for a while
                time::sleep(Duration::from_secs(1)).await;
                
                // Check if it's time to collect telemetry
                let should_collect = {
                    let last_time = last_collection_time.lock().unwrap();
                    match *last_time {
                        Some(time) => {
                            time.elapsed() > Duration::from_secs(config.collection_interval_seconds as u64)
                        },
                        None => true, // First run, should collect
                    }
                };
                
                if should_collect && config.telemetry_enabled {
                    debug!("Collecting telemetry");
                    
                    // Update last collection time
                    {
                        let mut last_time = last_collection_time.lock().unwrap();
                        *last_time = Some(Instant::now());
                    }
                    
                    // Collect telemetry for each enabled category
                    for category in &config.enabled_categories {
                        match category {
                            TelemetryCategory::System => {
                                if let Err(e) = collection::collect_system_telemetry(&telemetry_tx).await {
                                    error!("Failed to collect system telemetry: {}", e);
                                }
                            },
                            TelemetryCategory::Hardware => {
                                if let Err(e) = collection::collect_hardware_telemetry(&telemetry_tx).await {
                                    error!("Failed to collect hardware telemetry: {}", e);
                                }
                            },
                            TelemetryCategory::Application => {
                                if config.privacy_settings.collect_application_usage {
                                    if let Err(e) = collection::collect_application_telemetry(&telemetry_tx).await {
                                        error!("Failed to collect application telemetry: {}", e);
                                    }
                                }
                            },
                            TelemetryCategory::Network => {
                                if config.privacy_settings.collect_network_diagnostics {
                                    if let Err(e) = collection::collect_network_telemetry(&telemetry_tx).await {
                                        error!("Failed to collect network telemetry: {}", e);
                                    }
                                }
                            },
                            TelemetryCategory::Error => {
                                if config.privacy_settings.collect_crash_reports {
                                    if let Err(e) = collection::collect_error_telemetry(&telemetry_tx).await {
                                        error!("Failed to collect error telemetry: {}", e);
                                    }
                                }
                            },
                            TelemetryCategory::UserInteraction => {
                                if config.privacy_settings.collect_usage_statistics {
                                    if let Err(e) = collection::collect_user_interaction_telemetry(&telemetry_tx).await {
                                        error!("Failed to collect user interaction telemetry: {}", e);
                                    }
                                }
                            },
                            TelemetryCategory::Custom(name) => {
                                if let Err(e) = collection::collect_custom_telemetry(name, &telemetry_tx).await {
                                    error!("Failed to collect custom telemetry '{}': {}", name, e);
                                }
                            },
                        }
                    }
                    
                    // Save telemetry database periodically
                    let telemetry_db_path = config.telemetry_path.join("telemetry.json");
                    let telemetry_data = {
                        let db = telemetry_db.lock().unwrap();
                        db.clone()
                    };
                    
                    if let Err(e) = fs::create_dir_all(&config.telemetry_path) {
                        error!("Failed to create telemetry directory: {}", e);
                    } else {
                        let file = File::create(&telemetry_db_path);
                        match file {
                            Ok(file) => {
                                if let Err(e) = serde_json::to_writer(file, &telemetry_data) {
                                    error!("Failed to write telemetry database: {}", e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to create telemetry database file: {}", e);
                            }
                        }
                    }
                    
                    // Check if telemetry database is too large
                    let db_size = {
                        let db = telemetry_db.lock().unwrap();
                        db.len()
                    };
                    
                    if db_size > (config.max_telemetry_size_mb as usize * 1024 * 1024 / 100) {
                        // Database is too large, remove oldest entries
                        let mut db = telemetry_db.lock().unwrap();
                        db.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                        let to_remove = db.len() - (config.max_telemetry_size_mb as usize * 1024 * 1024 / 200);
                        db.drain(0..to_remove);
                    }
                }
            }
        });
        
        // Clone necessary values for telemetry processing task
        let config = self.config.read().unwrap().clone();
        let telemetry_db = Arc::clone(&self.telemetry_db);
        let running = Arc::clone(&self.running);
        let mut telemetry_rx = self.telemetry_rx.clone();
        
        // Start telemetry processing task
        tokio::spawn(async move {
            while *running.read().unwrap() {
                // Wait for telemetry data point
                if let Some(mut data_point) = telemetry_rx.recv().await {
                    // Apply privacy settings
                    data_point = privacy::apply_privacy_settings(data_point, &config.privacy_settings);
                    
                    // Anonymize data if needed
                    data_point = anonymization::anonymize_data_point(data_point, &config.privacy_settings);
                    
                    // Store telemetry data point
                    let mut db = telemetry_db.lock().unwrap();
                    db.push(data_point);
                }
            }
        });
        
        // Clone necessary values for log processing task
        let config = self.config.read().unwrap().clone();
        let running = Arc::clone(&self.running);
        let mut log_rx = self.log_rx.clone();
        
        // Start log processing task
        tokio::spawn(async move {
            // Open log file
            let log_file_path = config.log_path.join("vr-system.log");
            let mut log_file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open log file: {}", e);
                    return;
                }
            };
            
            // Create log batch for forwarding
            let mut log_batch = Vec::new();
            let mut last_batch_time = Instant::now();
            
            while *running.read().unwrap() {
                // Wait for log entry
                if let Some(log_entry) = log_rx.recv().await {
                    // Write log entry to file
                    let log_line = format!("[{}] [{}] [{}] {}\n",
                        log_entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                        log_entry.level,
                        log_entry.source,
                        log_entry.message);
                    
                    if let Err(e) = log_file.write_all(log_line.as_bytes()) {
                        error!("Failed to write to log file: {}", e);
                    }
                    
                    // Check if log rotation is needed
                    if config.log_rotation.rotate_by_size {
                        if let Ok(metadata) = fs::metadata(&log_file_path) {
                            let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                            if file_size_mb >= config.log_rotation.max_log_size_mb as f64 {
                                // Close current log file
                                drop(log_file);
                                
                                // Rotate logs
                                if let Err(e) = rotation::rotate_logs(&config.log_path, &config.log_rotation) {
                                    error!("Failed to rotate logs: {}", e);
                                }
                                
                                // Reopen log file
                                log_file = match OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(&log_file_path) {
                                    Ok(file) => file,
                                    Err(e) => {
                                        error!("Failed to reopen log file: {}", e);
                                        return;
                                    }
                                };
                            }
                        }
                    }
                    
                    // Add to batch for forwarding if enabled
                    if config.log_forwarding.forward_logs && 
                       log_entry.level >= config.log_forwarding.min_level_to_forward {
                        log_batch.push(log_entry);
                        
                        // Check if batch should be sent
                        let batch_full = log_batch.len() >= config.log_forwarding.max_batch_size as usize;
                        let batch_timeout = last_batch_time.elapsed() >= 
                            Duration::from_secs(config.log_forwarding.max_batch_wait_seconds as u64);
                        
                        if batch_full || batch_timeout {
                            // Send batch
                            let batch_to_send = log_batch.clone();
                            log_batch.clear();
                            last_batch_time = Instant::now();
                            
                            // Clone config for async task
                            let forwarding_config = config.log_forwarding.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = forwarding::forward_logs(&batch_to_send, &forwarding_config).await {
                                    error!("Failed to forward logs: {}", e);
                                }
                            });
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the telemetry manager.
    pub async fn stop(&self) -> Result<()> {
        // Set running flag to false
        {
            let mut running = self.running.write().unwrap();
            *running = false;
        }
        
        // Save telemetry database
        let config = self.config.read().unwrap();
        let telemetry_db_path = config.telemetry_path.join("telemetry.json");
        let telemetry_data = {
            let db = self.telemetry_db.lock().unwrap();
            db.clone()
        };
        
        let file = File::create(&telemetry_db_path)
            .context("Failed to create telemetry database file")?;
        serde_json::to_writer(file, &telemetry_data)
            .context("Failed to write telemetry database")?;
        
        Ok(())
    }
    
    /// Submit a telemetry data point.
    pub async fn submit_telemetry(&self, category: TelemetryCategory, name: &str, value: TelemetryValue) -> Result<()> {
        // Check if telemetry is enabled
        let config = self.config.read().unwrap();
        if !config.telemetry_enabled {
            return Ok(());
        }
        
        // Check if category is enabled
        if !config.enabled_categories.contains(&category) {
            return Ok(());
        }
        
        // Create data point
        let data_point = TelemetryDataPoint {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            name: name.to_string(),
            value,
            metadata: HashMap::new(),
        };
        
        // Send data point
        self.telemetry_tx.send(data_point).await
            .context("Failed to send telemetry data point")?;
        
        Ok(())
    }
    
    /// Submit a log entry.
    pub async fn submit_log(&self, level: LogLevel, source: &str, message: &str) -> Result<()> {
        // Create log entry
        let log_entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            source: source.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        };
        
        // Send log entry
        self.log_tx.send(log_entry).await
            .context("Failed to send log entry")?;
        
        Ok(())
    }
    
    /// Get telemetry data points.
    pub fn get_telemetry(&self, category: Option<TelemetryCategory>, limit: Option<usize>) -> Vec<TelemetryDataPoint> {
        let db = self.telemetry_db.lock().unwrap();
        
        let mut result = if let Some(cat) = category {
            db.iter().filter(|dp| dp.category == cat).cloned().collect::<Vec<_>>()
        } else {
            db.clone()
        };
        
        // Sort by timestamp (newest first)
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit
        if let Some(limit) = limit {
            if result.len() > limit {
                result.truncate(limit);
            }
        }
        
        result
    }
    
    /// Analyze telemetry data.
    pub async fn analyze_telemetry(&self, category: Option<TelemetryCategory>) -> Result<analysis::TelemetryAnalysis> {
        // Get telemetry data
        let data = self.get_telemetry(category, None);
        
        // Perform analysis
        analysis::analyze_telemetry(&data)
    }
    
    /// Update telemetry configuration.
    pub async fn update_config(&self, config: TelemetryConfig) -> Result<()> {
        // Update configuration
        {
            let mut current_config = self.config.write().unwrap();
            *current_config = config;
        }
        
        Ok(())
    }
    
    /// Get current telemetry configuration.
    pub fn get_config(&self) -> TelemetryConfig {
        let config = self.config.read().unwrap();
        config.clone()
    }
    
    /// Update privacy settings.
    pub async fn update_privacy_settings(&self, settings: PrivacySettings) -> Result<()> {
        // Update privacy settings
        {
            let mut config = self.config.write().unwrap();
            config.privacy_settings = settings;
        }
        
        Ok(())
    }
    
    /// Get current privacy settings.
    pub fn get_privacy_settings(&self) -> PrivacySettings {
        let config = self.config.read().unwrap();
        config.privacy_settings.clone()
    }
}
