//! Storage monitoring for the VR headset.
//!
//! This module provides comprehensive storage monitoring capabilities
//! for tracking storage usage, performance, and health.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::metrics::{Metric, MetricsCollector, MetricType, MetricValue};

/// Storage device type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    /// Internal storage
    Internal,
    
    /// External storage (e.g., SD card)
    External,
    
    /// USB storage
    Usb,
    
    /// Network storage
    Network,
}

impl StorageType {
    /// Get the storage type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageType::Internal => "internal",
            StorageType::External => "external",
            StorageType::Usb => "usb",
            StorageType::Network => "network",
        }
    }
    
    /// Parse a storage type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "internal" => Some(StorageType::Internal),
            "external" => Some(StorageType::External),
            "usb" => Some(StorageType::Usb),
            "network" => Some(StorageType::Network),
            _ => None,
        }
    }
}

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Storage device health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageHealth {
    /// Storage is healthy
    Good,
    
    /// Storage has some issues but is still usable
    Warning,
    
    /// Storage has critical issues
    Critical,
    
    /// Storage health is unknown
    Unknown,
}

impl StorageHealth {
    /// Get the storage health as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageHealth::Good => "good",
            StorageHealth::Warning => "warning",
            StorageHealth::Critical => "critical",
            StorageHealth::Unknown => "unknown",
        }
    }
    
    /// Parse a storage health from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "good" => Some(StorageHealth::Good),
            "warning" => Some(StorageHealth::Warning),
            "critical" => Some(StorageHealth::Critical),
            "unknown" => Some(StorageHealth::Unknown),
            _ => None,
        }
    }
}

impl std::fmt::Display for StorageHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Storage device statistics.
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Device name
    pub name: String,
    
    /// Mount point
    pub mount_point: String,
    
    /// Storage type
    pub storage_type: StorageType,
    
    /// File system type
    pub fs_type: String,
    
    /// Total capacity in bytes
    pub total_bytes: u64,
    
    /// Used space in bytes
    pub used_bytes: u64,
    
    /// Available space in bytes
    pub available_bytes: u64,
    
    /// Health status
    pub health: StorageHealth,
    
    /// Read operations
    pub read_ops: u64,
    
    /// Write operations
    pub write_ops: u64,
    
    /// Bytes read
    pub read_bytes: u64,
    
    /// Bytes written
    pub write_bytes: u64,
    
    /// Read time in milliseconds
    pub read_time_ms: u64,
    
    /// Write time in milliseconds
    pub write_time_ms: u64,
}

/// Storage metrics collector.
#[derive(Debug)]
pub struct StorageMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Last storage statistics
    last_stats: Mutex<HashMap<String, StorageStats>>,
}

impl StorageMetricsCollector {
    /// Create a new storage metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "storage".to_string(),
            interval: Duration::from_secs(interval_secs),
            last_stats: Mutex::new(HashMap::new()),
        }
    }
    
    /// Get storage statistics.
    fn get_storage_stats(&self) -> Vec<StorageStats> {
        // In a real implementation, this would read from /proc/mounts, /proc/diskstats, etc.
        // For now, we'll simulate some values
        
        // Internal storage
        let internal = StorageStats {
            name: "mmcblk0".to_string(),
            mount_point: "/".to_string(),
            storage_type: StorageType::Internal,
            fs_type: "ext4".to_string(),
            total_bytes: 64 * 1024 * 1024 * 1024, // 64 GB
            used_bytes: 32 * 1024 * 1024 * 1024,  // 32 GB
            available_bytes: 32 * 1024 * 1024 * 1024, // 32 GB
            health: StorageHealth::Good,
            read_ops: 100000,
            write_ops: 50000,
            read_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            write_bytes: 1 * 1024 * 1024 * 1024, // 1 GB
            read_time_ms: 5000,
            write_time_ms: 3000,
        };
        
        // External storage (SD card)
        let external = StorageStats {
            name: "mmcblk1".to_string(),
            mount_point: "/mnt/sdcard".to_string(),
            storage_type: StorageType::External,
            fs_type: "vfat".to_string(),
            total_bytes: 128 * 1024 * 1024 * 1024, // 128 GB
            used_bytes: 64 * 1024 * 1024 * 1024,   // 64 GB
            available_bytes: 64 * 1024 * 1024 * 1024, // 64 GB
            health: StorageHealth::Good,
            read_ops: 50000,
            write_ops: 25000,
            read_bytes: 1 * 1024 * 1024 * 1024, // 1 GB
            write_bytes: 500 * 1024 * 1024,     // 500 MB
            read_time_ms: 3000,
            write_time_ms: 2000,
        };
        
        vec![internal, external]
    }
    
    /// Calculate storage I/O rates.
    fn calculate_io_rates(&self, current: &StorageStats, previous: &StorageStats) -> (f64, f64, f64, f64) {
        // Calculate rates in operations per second and bytes per second
        let read_ops_rate = if current.read_ops >= previous.read_ops {
            (current.read_ops - previous.read_ops) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        let write_ops_rate = if current.write_ops >= previous.write_ops {
            (current.write_ops - previous.write_ops) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        let read_bytes_rate = if current.read_bytes >= previous.read_bytes {
            (current.read_bytes - previous.read_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        let write_bytes_rate = if current.write_bytes >= previous.write_bytes {
            (current.write_bytes - previous.write_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        (read_ops_rate, write_ops_rate, read_bytes_rate, write_bytes_rate)
    }
}

impl MetricsCollector for StorageMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get current storage statistics
        let current_stats = self.get_storage_stats();
        
        // Get last statistics for rate calculation
        let mut last_stats = self.last_stats.lock().unwrap();
        
        // Process each storage device
        for device in &current_stats {
            // Device labels
            let mut labels = HashMap::new();
            labels.insert("device".to_string(), device.name.clone());
            labels.insert("mount_point".to_string(), device.mount_point.clone());
            labels.insert("type".to_string(), device.storage_type.to_string());
            labels.insert("fs_type".to_string(), device.fs_type.clone());
            
            // Capacity metrics
            metrics.push(Metric::new(
                "storage.capacity.total",
                MetricType::Gauge,
                MetricValue::Integer(device.total_bytes as i64),
                Some(labels.clone()),
                Some("Total storage capacity"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                "storage.capacity.used",
                MetricType::Gauge,
                MetricValue::Integer(device.used_bytes as i64),
                Some(labels.clone()),
                Some("Used storage space"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                "storage.capacity.available",
                MetricType::Gauge,
                MetricValue::Integer(device.available_bytes as i64),
                Some(labels.clone()),
                Some("Available storage space"),
                Some("bytes"),
            ));
            
            // Usage percentage
            let usage_percent = if device.total_bytes > 0 {
                100.0 * device.used_bytes as f64 / device.total_bytes as f64
            } else {
                0.0
            };
            
            metrics.push(Metric::new(
                "storage.capacity.usage",
                MetricType::Gauge,
                MetricValue::Float(usage_percent),
                Some(labels.clone()),
                Some("Storage usage percentage"),
                Some("%"),
            ));
            
            // Health metric
            metrics.push(Metric::new(
                "storage.health",
                MetricType::State,
                MetricValue::String(device.health.to_string()),
                Some(labels.clone()),
                Some("Storage health status"),
                None,
            ));
            
            // I/O operation metrics
            metrics.push(Metric::new(
                "storage.io.read_ops",
                MetricType::Counter,
                MetricValue::Integer(device.read_ops as i64),
                Some(labels.clone()),
                Some("Total read operations"),
                Some("ops"),
            ));
            
            metrics.push(Metric::new(
                "storage.io.write_ops",
                MetricType::Counter,
                MetricValue::Integer(device.write_ops as i64),
                Some(labels.clone()),
                Some("Total write operations"),
                Some("ops"),
            ));
            
            // I/O bytes metrics
            metrics.push(Metric::new(
                "storage.io.read_bytes",
                MetricType::Counter,
                MetricValue::Integer(device.read_bytes as i64),
                Some(labels.clone()),
                Some("Total bytes read"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                "storage.io.write_bytes",
                MetricType::Counter,
                MetricValue::Integer(device.write_bytes as i64),
                Some(labels.clone()),
                Some("Total bytes written"),
                Some("bytes"),
            ));
            
            // I/O time metrics
            metrics.push(Metric::new(
                "storage.io.read_time",
                MetricType::Counter,
                MetricValue::Integer(device.read_time_ms as i64),
                Some(labels.clone()),
                Some("Total read time"),
                Some("ms"),
            ));
            
            metrics.push(Metric::new(
                "storage.io.write_time",
                MetricType::Counter,
                MetricValue::Integer(device.write_time_ms as i64),
                Some(labels.clone()),
                Some("Total write time"),
                Some("ms"),
            ));
            
            // Calculate I/O rates if we have previous stats
            if let Some(previous) = last_stats.get(&device.name) {
                let (read_ops_rate, write_ops_rate, read_bytes_rate, write_bytes_rate) = 
                    self.calculate_io_rates(device, previous);
                
                // I/O operations rate metrics
                metrics.push(Metric::new(
                    "storage.io.read_ops_rate",
                    MetricType::Gauge,
                    MetricValue::Float(read_ops_rate),
                    Some(labels.clone()),
                    Some("Read operations rate"),
                    Some("ops/s"),
                ));
                
                metrics.push(Metric::new(
                    "storage.io.write_ops_rate",
                    MetricType::Gauge,
                    MetricValue::Float(write_ops_rate),
                    Some(labels.clone()),
                    Some("Write operations rate"),
                    Some("ops/s"),
                ));
                
                // I/O bytes rate metrics
                metrics.push(Metric::new(
                    "storage.io.read_bytes_rate",
                    MetricType::Gauge,
                    MetricValue::Float(read_bytes_rate),
                    Some(labels.clone()),
                    Some("Read bytes rate"),
                    Some("bytes/s"),
                ));
                
                metrics.push(Metric::new(
                    "storage.io.write_bytes_rate",
                    MetricType::Gauge,
                    MetricValue::Float(write_bytes_rate),
                    Some(labels.clone()),
                    Some("Write bytes rate"),
                    Some("bytes/s"),
                ));
                
                // I/O bytes rate metrics (MB/s for display)
                metrics.push(Metric::new(
                    "storage.io.read_mbytes_rate",
                    MetricType::Gauge,
                    MetricValue::Float(read_bytes_rate / (1024.0 * 1024.0)),
                    Some(labels.clone()),
                    Some("Read rate"),
                    Some("MB/s"),
                ));
                
                metrics.push(Metric::new(
                    "storage.io.write_mbytes_rate",
                    MetricType::Gauge,
                    MetricValue::Float(write_bytes_rate / (1024.0 * 1024.0)),
                    Some(labels.clone()),
                    Some("Write rate"),
                    Some("MB/s"),
                ));
            }
            
            // Update last stats
            last_stats.insert(device.name.clone(), device.clone());
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Directory usage statistics.
#[derive(Debug, Clone)]
pub struct DirectoryStats {
    /// Directory path
    pub path: String,
    
    /// Total size in bytes
    pub size_bytes: u64,
    
    /// Number of files
    pub file_count: u64,
    
    /// Number of directories
    pub dir_count: u64,
}

/// Directory usage metrics collector.
#[derive(Debug)]
pub struct DirectoryUsageMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Directories to monitor
    directories: Vec<String>,
}

impl DirectoryUsageMetricsCollector {
    /// Create a new directory usage metrics collector.
    pub fn new(interval_secs: u64, directories: Vec<String>) -> Self {
        Self {
            name: "directory_usage".to_string(),
            interval: Duration::from_secs(interval_secs),
            directories,
        }
    }
    
    /// Get directory statistics.
    fn get_directory_stats(&self) -> Vec<DirectoryStats> {
        // In a real implementation, this would use file system APIs
        // For now, we'll simulate some values
        
        let mut stats = Vec::new();
        
        for dir in &self.directories {
            // Simulate different sizes based on directory path
            let (size, files, dirs) = match dir.as_str() {
                "/mnt/sdcard/Downloads" => (
                    5 * 1024 * 1024 * 1024, // 5 GB
                    100,
                    10,
                ),
                "/mnt/sdcard/Pictures" => (
                    2 * 1024 * 1024 * 1024, // 2 GB
                    500,
                    20,
                ),
                "/mnt/sdcard/Videos" => (
                    10 * 1024 * 1024 * 1024, // 10 GB
                    50,
                    5,
                ),
                "/mnt/sdcard/Music" => (
                    3 * 1024 * 1024 * 1024, // 3 GB
                    300,
                    15,
                ),
                "/mnt/sdcard/Documents" => (
                    1 * 1024 * 1024 * 1024, // 1 GB
                    200,
                    30,
                ),
                _ => (
                    100 * 1024 * 1024, // 100 MB
                    50,
                    5,
                ),
            };
            
            stats.push(DirectoryStats {
                path: dir.clone(),
                size_bytes: size,
                file_count: files,
                dir_count: dirs,
            });
        }
        
        stats
    }
}

impl MetricsCollector for DirectoryUsageMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get directory statistics
        let dir_stats = self.get_directory_stats();
        
        // Process each directory
        for dir in &dir_stats {
            // Directory labels
            let mut labels = HashMap::new();
            labels.insert("path".to_string(), dir.path.clone());
            
            // Size metric
            metrics.push(Metric::new(
                "storage.directory.size",
                MetricType::Gauge,
                MetricValue::Integer(dir.size_bytes as i64),
                Some(labels.clone()),
                Some("Directory size"),
                Some("bytes"),
            ));
            
            // File count metric
            metrics.push(Metric::new(
                "storage.directory.file_count",
                MetricType::Gauge,
                MetricValue::Integer(dir.file_count as i64),
                Some(labels.clone()),
                Some("Number of files in directory"),
                None,
            ));
            
            // Directory count metric
            metrics.push(Metric::new(
                "storage.directory.dir_count",
                MetricType::Gauge,
                MetricValue::Integer(dir.dir_count as i64),
                Some(labels.clone()),
                Some("Number of subdirectories in directory"),
                None,
            ));
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Storage monitor.
#[derive(Debug)]
pub struct StorageMonitor {
    /// Storage metrics collector
    storage_collector: Arc<StorageMetricsCollector>,
    
    /// Directory usage metrics collector
    directory_collector: Arc<DirectoryUsageMetricsCollector>,
}

impl StorageMonitor {
    /// Create a new storage monitor.
    pub fn new() -> Self {
        // Default directories to monitor
        let directories = vec![
            "/mnt/sdcard/Downloads".to_string(),
            "/mnt/sdcard/Pictures".to_string(),
            "/mnt/sdcard/Videos".to_string(),
            "/mnt/sdcard/Music".to_string(),
            "/mnt/sdcard/Documents".to_string(),
        ];
        
        let storage_collector = Arc::new(StorageMetricsCollector::new(10));
        let directory_collector = Arc::new(DirectoryUsageMetricsCollector::new(60, directories));
        
        Self {
            storage_collector,
            directory_collector,
        }
    }
    
    /// Get the storage metrics collector.
    pub fn storage_collector(&self) -> Arc<StorageMetricsCollector> {
        self.storage_collector.clone()
    }
    
    /// Get the directory usage metrics collector.
    pub fn directory_collector(&self) -> Arc<DirectoryUsageMetricsCollector> {
        self.directory_collector.clone()
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> Vec<Arc<dyn MetricsCollector>> {
        vec![
            self.storage_collector.clone() as Arc<dyn MetricsCollector>,
            self.directory_collector.clone() as Arc<dyn MetricsCollector>,
        ]
    }
}

impl Default for StorageMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_type() {
        assert_eq!(StorageType::Internal.as_str(), "internal");
        assert_eq!(StorageType::External.as_str(), "external");
        assert_eq!(StorageType::Usb.as_str(), "usb");
        assert_eq!(StorageType::Network.as_str(), "network");
        
        assert_eq!(StorageType::from_str("internal"), Some(StorageType::Internal));
        assert_eq!(StorageType::from_str("external"), Some(StorageType::External));
        assert_eq!(StorageType::from_str("usb"), Some(StorageType::Usb));
        assert_eq!(StorageType::from_str("network"), Some(StorageType::Network));
        assert_eq!(StorageType::from_str("invalid"), None);
        
        assert_eq!(StorageType::Internal.to_string(), "internal");
        assert_eq!(StorageType::External.to_string(), "external");
        assert_eq!(StorageType::Usb.to_string(), "usb");
        assert_eq!(StorageType::Network.to_string(), "network");
    }
    
    #[test]
    fn test_storage_health() {
        assert_eq!(StorageHealth::Good.as_str(), "good");
        assert_eq!(StorageHealth::Warning.as_str(), "warning");
        assert_eq!(StorageHealth::Critical.as_str(), "critical");
        assert_eq!(StorageHealth::Unknown.as_str(), "unknown");
        
        assert_eq!(StorageHealth::from_str("good"), Some(StorageHealth::Good));
        assert_eq!(StorageHealth::from_str("warning"), Some(StorageHealth::Warning));
        assert_eq!(StorageHealth::from_str("critical"), Some(StorageHealth::Critical));
        assert_eq!(StorageHealth::from_str("unknown"), Some(StorageHealth::Unknown));
        assert_eq!(StorageHealth::from_str("invalid"), None);
        
        assert_eq!(StorageHealth::Good.to_string(), "good");
        assert_eq!(StorageHealth::Warning.to_string(), "warning");
        assert_eq!(StorageHealth::Critical.to_string(), "critical");
        assert_eq!(StorageHealth::Unknown.to_string(), "unknown");
    }
    
    #[test]
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new(10);
        
        // First collection
        let metrics = collector.collect();
        
        // Check capacity metrics
        assert!(metrics.iter().any(|m| m.name == "storage.capacity.total"));
        assert!(metrics.iter().any(|m| m.name == "storage.capacity.used"));
        assert!(metrics.iter().any(|m| m.name == "storage.capacity.available"));
        assert!(metrics.iter().any(|m| m.name == "storage.capacity.usage"));
        
        // Check health metric
        assert!(metrics.iter().any(|m| m.name == "storage.health"));
        
        // Check I/O metrics
        assert!(metrics.iter().any(|m| m.name == "storage.io.read_ops"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.write_ops"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.read_bytes"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.write_bytes"));
        
        // Second collection should include rate metrics
        let metrics = collector.collect();
        assert!(metrics.iter().any(|m| m.name == "storage.io.read_ops_rate"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.write_ops_rate"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.read_bytes_rate"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.write_bytes_rate"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.read_mbytes_rate"));
        assert!(metrics.iter().any(|m| m.name == "storage.io.write_mbytes_rate"));
    }
    
    #[test]
    fn test_directory_usage_metrics_collector() {
        let directories = vec![
            "/mnt/sdcard/Downloads".to_string(),
            "/mnt/sdcard/Pictures".to_string(),
        ];
        
        let collector = DirectoryUsageMetricsCollector::new(60, directories);
        let metrics = collector.collect();
        
        // Check directory metrics
        assert!(metrics.iter().any(|m| m.name == "storage.directory.size"));
        assert!(metrics.iter().any(|m| m.name == "storage.directory.file_count"));
        assert!(metrics.iter().any(|m| m.name == "storage.directory.dir_count"));
        
        // Check that we have metrics for both directories
        let download_metrics = metrics.iter().filter(|m| {
            if let Some(labels) = &m.labels {
                if let Some(path) = labels.get("path") {
                    return path == "/mnt/sdcard/Downloads";
                }
            }
            false
        }).count();
        
        let pictures_metrics = metrics.iter().filter(|m| {
            if let Some(labels) = &m.labels {
                if let Some(path) = labels.get("path") {
                    return path == "/mnt/sdcard/Pictures";
                }
            }
            false
        }).count();
        
        assert_eq!(download_metrics, 3); // size, file_count, dir_count
        assert_eq!(pictures_metrics, 3); // size, file_count, dir_count
    }
    
    #[test]
    fn test_storage_monitor() {
        let monitor = StorageMonitor::new();
        
        // Check collectors
        assert_eq!(monitor.collectors().len(), 2);
        
        // Check collector access
        assert_eq!(monitor.storage_collector().name(), "storage");
        assert_eq!(monitor.directory_collector().name(), "directory_usage");
    }
}
