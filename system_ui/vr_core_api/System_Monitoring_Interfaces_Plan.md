# System Monitoring Interfaces Plan

This document outlines the detailed implementation plan for the System Monitoring Interfaces in the VR Core API layer. The monitoring system will provide comprehensive metrics collection, analysis, and reporting for all aspects of the VR headset's operation.

## 1. Overall Architecture

### 1.1 Design Principles

- **Low overhead**: Minimal performance impact on the system
- **Comprehensive coverage**: Monitor all critical system components
- **Configurable sampling**: Adjustable collection frequency based on needs
- **Historical data**: Store and analyze trends over time
- **Alerting**: Threshold-based notifications for critical conditions
- **Extensibility**: Easy addition of new metrics and data sources
- **Visualization-ready**: Data formats suitable for dashboard display

### 1.2 Module Structure

```
monitoring/
├── mod.rs                 # Main module and MonitoringManager
├── metrics.rs             # Core metrics definitions and types
├── collectors/            # Metric collectors for different subsystems
│   ├── mod.rs             # Collector registry
│   ├── performance.rs     # CPU, GPU, memory metrics
│   ├── battery.rs         # Power and battery metrics
│   ├── network.rs         # Network connectivity metrics
│   ├── storage.rs         # Storage usage metrics
│   ├── thermal.rs         # Temperature metrics
│   ├── process.rs         # Process and thread metrics
│   └── custom.rs          # Custom metric collectors
├── storage/               # Metrics storage
│   ├── mod.rs             # Storage interface
│   ├── memory.rs          # In-memory storage
│   ├── file.rs            # File-based storage
│   └── database.rs        # Database storage (optional)
├── analysis/              # Metrics analysis
│   ├── mod.rs             # Analysis registry
│   ├── trends.rs          # Trend analysis
│   ├── anomaly.rs         # Anomaly detection
│   └── prediction.rs      # Predictive analysis
├── alerting/              # Alert system
│   ├── mod.rs             # Alert manager
│   ├── threshold.rs       # Threshold-based alerts
│   ├── rule.rs            # Rule-based alerts
│   └── notification.rs    # Alert notifications
└── tests/                 # Test modules
    ├── test_collectors.rs # Collector tests
    ├── test_storage.rs    # Storage tests
    └── ...
```

## 2. Core Metrics System

### 2.1 Metric Types and Definitions

```rust
/// Metric value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// String value
    String(String),
    /// Histogram of values
    Histogram(Vec<(f64, u64)>),
    /// Set of values
    Set(HashSet<String>),
    /// Map of values
    Map(HashMap<String, MetricValue>),
}

/// Metric data point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: MetricValue,
    /// Timestamp in milliseconds since epoch
    pub timestamp: u64,
    /// Labels/tags for the metric
    pub labels: HashMap<String, String>,
}

/// Metric metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricMetadata {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: String,
    /// Metric unit
    pub unit: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Default collection interval in milliseconds
    pub default_interval: u64,
    /// Whether the metric is enabled by default
    pub enabled_by_default: bool,
    /// Labels that this metric supports
    pub supported_labels: Vec<String>,
    /// Category of the metric
    pub category: MetricCategory,
}

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter (monotonically increasing)
    Counter,
    /// Gauge (can go up and down)
    Gauge,
    /// Histogram (distribution of values)
    Histogram,
    /// Summary (calculated statistics)
    Summary,
    /// Info (string or boolean information)
    Info,
}

/// Metric category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricCategory {
    /// CPU metrics
    CPU,
    /// GPU metrics
    GPU,
    /// Memory metrics
    Memory,
    /// Battery metrics
    Battery,
    /// Network metrics
    Network,
    /// Storage metrics
    Storage,
    /// Thermal metrics
    Thermal,
    /// Process metrics
    Process,
    /// Custom metrics
    Custom,
}
```

### 2.2 Metric Registry

```rust
/// Metric registry for managing metric definitions
pub struct MetricRegistry {
    metrics: HashMap<String, MetricMetadata>,
    categories: HashMap<MetricCategory, HashSet<String>>,
}

impl MetricRegistry {
    /// Create a new MetricRegistry
    pub fn new() -> Self;
    
    /// Register a metric
    pub fn register_metric(&mut self, metadata: MetricMetadata) -> Result<()>;
    
    /// Get metric metadata by name
    pub fn get_metric(&self, name: &str) -> Option<&MetricMetadata>;
    
    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<&MetricMetadata>;
    
    /// Get metrics by category
    pub fn get_metrics_by_category(&self, category: MetricCategory) -> Vec<&MetricMetadata>;
    
    /// Check if metric exists
    pub fn metric_exists(&self, name: &str) -> bool;
    
    /// Unregister a metric
    pub fn unregister_metric(&mut self, name: &str) -> Result<()>;
}
```

### 2.3 Metric Collector Interface

```rust
/// Metric collector trait
pub trait MetricCollector: Send + Sync {
    /// Get collector name
    fn name(&self) -> &str;
    
    /// Get collector description
    fn description(&self) -> &str;
    
    /// Get metrics collected by this collector
    fn metrics(&self) -> Vec<MetricMetadata>;
    
    /// Initialize the collector
    fn initialize(&mut self) -> Result<()>;
    
    /// Collect metrics
    fn collect(&self) -> Result<Vec<MetricDataPoint>>;
    
    /// Check if collector is enabled
    fn is_enabled(&self) -> bool;
    
    /// Enable or disable the collector
    fn set_enabled(&mut self, enabled: bool);
    
    /// Get collection interval in milliseconds
    fn collection_interval(&self) -> u64;
    
    /// Set collection interval in milliseconds
    fn set_collection_interval(&mut self, interval: u64);
    
    /// Shutdown the collector
    fn shutdown(&mut self) -> Result<()>;
}
```

### 2.4 Collector Registry

```rust
/// Collector registry for managing metric collectors
pub struct CollectorRegistry {
    collectors: HashMap<String, Box<dyn MetricCollector>>,
    category_collectors: HashMap<MetricCategory, HashSet<String>>,
}

impl CollectorRegistry {
    /// Create a new CollectorRegistry
    pub fn new() -> Self;
    
    /// Register a collector
    pub fn register_collector(&mut self, collector: Box<dyn MetricCollector>) -> Result<()>;
    
    /// Get collector by name
    pub fn get_collector(&self, name: &str) -> Option<&dyn MetricCollector>;
    
    /// Get mutable collector by name
    pub fn get_collector_mut(&mut self, name: &str) -> Option<&mut dyn MetricCollector>;
    
    /// Get all collectors
    pub fn get_all_collectors(&self) -> Vec<&dyn MetricCollector>;
    
    /// Get collectors by category
    pub fn get_collectors_by_category(&self, category: MetricCategory) -> Vec<&dyn MetricCollector>;
    
    /// Check if collector exists
    pub fn collector_exists(&self, name: &str) -> bool;
    
    /// Unregister a collector
    pub fn unregister_collector(&mut self, name: &str) -> Result<()>;
    
    /// Collect metrics from all enabled collectors
    pub fn collect_all(&self) -> Result<Vec<MetricDataPoint>>;
    
    /// Collect metrics from a specific collector
    pub fn collect_from(&self, name: &str) -> Result<Vec<MetricDataPoint>>;
    
    /// Collect metrics from collectors in a category
    pub fn collect_category(&self, category: MetricCategory) -> Result<Vec<MetricDataPoint>>;
}
```

## 3. Performance Metrics Collection

### 3.1 CPU Metrics Collector

```rust
/// CPU metrics collector
pub struct CPUCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
    last_cpu_times: Option<HashMap<u32, CPUTimes>>,
}

impl CPUCollector {
    /// Create a new CPUCollector
    pub fn new() -> Self;
}

impl MetricCollector for CPUCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "cpu_usage_percent".to_string(),
                description: "CPU usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["cpu".to_string(), "mode".to_string()],
                category: MetricCategory::CPU,
            },
            MetricMetadata {
                name: "cpu_frequency_mhz".to_string(),
                description: "CPU frequency in MHz".to_string(),
                unit: "MHz".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["cpu".to_string()],
                category: MetricCategory::CPU,
            },
            MetricMetadata {
                name: "cpu_temperature_celsius".to_string(),
                description: "CPU temperature in Celsius".to_string(),
                unit: "°C".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["cpu".to_string()],
                category: MetricCategory::CPU,
            },
            MetricMetadata {
                name: "cpu_load_average".to_string(),
                description: "System load average".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["period".to_string()],
                category: MetricCategory::CPU,
            },
            // Additional CPU metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of CPU metrics collection
        // This would use platform-specific APIs to gather CPU information
        // For Linux, this might involve reading from /proc/stat, /proc/cpuinfo, etc.
        
        let mut data_points = Vec::new();
        
        // Collect CPU usage
        let cpu_usage = self.collect_cpu_usage()?;
        for (cpu_id, usage) in cpu_usage {
            let mut labels = HashMap::new();
            labels.insert("cpu".to_string(), cpu_id.to_string());
            
            data_points.push(MetricDataPoint {
                name: "cpu_usage_percent".to_string(),
                value: MetricValue::Float(usage),
                timestamp: current_time_ms(),
                labels,
            });
        }
        
        // Collect CPU frequency
        let cpu_freq = self.collect_cpu_frequency()?;
        for (cpu_id, freq) in cpu_freq {
            let mut labels = HashMap::new();
            labels.insert("cpu".to_string(), cpu_id.to_string());
            
            data_points.push(MetricDataPoint {
                name: "cpu_frequency_mhz".to_string(),
                value: MetricValue::Float(freq),
                timestamp: current_time_ms(),
                labels,
            });
        }
        
        // Collect CPU temperature
        let cpu_temp = self.collect_cpu_temperature()?;
        for (cpu_id, temp) in cpu_temp {
            let mut labels = HashMap::new();
            labels.insert("cpu".to_string(), cpu_id.to_string());
            
            data_points.push(MetricDataPoint {
                name: "cpu_temperature_celsius".to_string(),
                value: MetricValue::Float(temp),
                timestamp: current_time_ms(),
                labels,
            });
        }
        
        // Collect load average
        let load_avg = self.collect_load_average()?;
        for (period, load) in load_avg {
            let mut labels = HashMap::new();
            labels.insert("period".to_string(), period.to_string());
            
            data_points.push(MetricDataPoint {
                name: "cpu_load_average".to_string(),
                value: MetricValue::Float(load),
                timestamp: current_time_ms(),
                labels,
            });
        }
        
        Ok(data_points)
    }
}
```

### 3.2 GPU Metrics Collector

```rust
/// GPU metrics collector
pub struct GPUCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl GPUCollector {
    /// Create a new GPUCollector
    pub fn new() -> Self;
}

impl MetricCollector for GPUCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "gpu_usage_percent".to_string(),
                description: "GPU usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["gpu".to_string()],
                category: MetricCategory::GPU,
            },
            MetricMetadata {
                name: "gpu_memory_usage_bytes".to_string(),
                description: "GPU memory usage in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["gpu".to_string()],
                category: MetricCategory::GPU,
            },
            MetricMetadata {
                name: "gpu_temperature_celsius".to_string(),
                description: "GPU temperature in Celsius".to_string(),
                unit: "°C".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["gpu".to_string()],
                category: MetricCategory::GPU,
            },
            MetricMetadata {
                name: "gpu_frequency_mhz".to_string(),
                description: "GPU frequency in MHz".to_string(),
                unit: "MHz".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec!["gpu".to_string()],
                category: MetricCategory::GPU,
            },
            // Additional GPU metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of GPU metrics collection
        // This would use platform-specific APIs to gather GPU information
        // For embedded GPUs, this might involve reading from sysfs or using vendor-specific libraries
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

### 3.3 Memory Metrics Collector

```rust
/// Memory metrics collector
pub struct MemoryCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl MemoryCollector {
    /// Create a new MemoryCollector
    pub fn new() -> Self;
}

impl MetricCollector for MemoryCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "memory_total_bytes".to_string(),
                description: "Total physical memory in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            MetricMetadata {
                name: "memory_used_bytes".to_string(),
                description: "Used physical memory in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            MetricMetadata {
                name: "memory_free_bytes".to_string(),
                description: "Free physical memory in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            MetricMetadata {
                name: "memory_usage_percent".to_string(),
                description: "Physical memory usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            MetricMetadata {
                name: "swap_total_bytes".to_string(),
                description: "Total swap memory in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            MetricMetadata {
                name: "swap_used_bytes".to_string(),
                description: "Used swap memory in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 1000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Memory,
            },
            // Additional memory metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of memory metrics collection
        // This would use platform-specific APIs to gather memory information
        // For Linux, this might involve reading from /proc/meminfo
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 4. Battery and Power Monitoring

### 4.1 Battery Metrics Collector

```rust
/// Battery metrics collector
pub struct BatteryCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl BatteryCollector {
    /// Create a new BatteryCollector
    pub fn new() -> Self;
}

impl MetricCollector for BatteryCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "battery_level_percent".to_string(),
                description: "Battery level percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_voltage_volts".to_string(),
                description: "Battery voltage in volts".to_string(),
                unit: "V".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_current_amps".to_string(),
                description: "Battery current in amps".to_string(),
                unit: "A".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_power_watts".to_string(),
                description: "Battery power in watts".to_string(),
                unit: "W".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_temperature_celsius".to_string(),
                description: "Battery temperature in Celsius".to_string(),
                unit: "°C".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_charging_state".to_string(),
                description: "Battery charging state".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Info,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_time_remaining_seconds".to_string(),
                description: "Estimated battery time remaining in seconds".to_string(),
                unit: "s".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 30000,
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "battery_cycle_count".to_string(),
                description: "Battery charge cycle count".to_string(),
                unit: "cycles".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 3600000, // 1 hour
                enabled_by_default: true,
                supported_labels: vec!["battery".to_string()],
                category: MetricCategory::Battery,
            },
            // Additional battery metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of battery metrics collection
        // This would use platform-specific APIs to gather battery information
        // For Linux, this might involve reading from /sys/class/power_supply/
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

### 4.2 Power Consumption Collector

```rust
/// Power consumption metrics collector
pub struct PowerConsumptionCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl PowerConsumptionCollector {
    /// Create a new PowerConsumptionCollector
    pub fn new() -> Self;
}

impl MetricCollector for PowerConsumptionCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "power_consumption_total_watts".to_string(),
                description: "Total system power consumption in watts".to_string(),
                unit: "W".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "power_consumption_component_watts".to_string(),
                description: "Component power consumption in watts".to_string(),
                unit: "W".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["component".to_string()],
                category: MetricCategory::Battery,
            },
            MetricMetadata {
                name: "power_profile".to_string(),
                description: "Current power profile".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Info,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec![],
                category: MetricCategory::Battery,
            },
            // Additional power metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of power consumption metrics collection
        // This would use platform-specific APIs to gather power information
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 5. Network Status Monitoring

### 5.1 Network Metrics Collector

```rust
/// Network metrics collector
pub struct NetworkCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
    last_io_stats: Option<HashMap<String, NetworkIOStats>>,
}

impl NetworkCollector {
    /// Create a new NetworkCollector
    pub fn new() -> Self;
}

impl MetricCollector for NetworkCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "network_bytes_sent".to_string(),
                description: "Network bytes sent".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "network_bytes_received".to_string(),
                description: "Network bytes received".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "network_packets_sent".to_string(),
                description: "Network packets sent".to_string(),
                unit: "packets".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "network_packets_received".to_string(),
                description: "Network packets received".to_string(),
                unit: "packets".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "network_errors".to_string(),
                description: "Network errors".to_string(),
                unit: "errors".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string(), "direction".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "network_bandwidth_bytes_per_second".to_string(),
                description: "Network bandwidth in bytes per second".to_string(),
                unit: "bytes/s".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string(), "direction".to_string()],
                category: MetricCategory::Network,
            },
            // Additional network metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of network metrics collection
        // This would use platform-specific APIs to gather network information
        // For Linux, this might involve reading from /proc/net/dev
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

### 5.2 WiFi Metrics Collector

```rust
/// WiFi metrics collector
pub struct WiFiCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl WiFiCollector {
    /// Create a new WiFiCollector
    pub fn new() -> Self;
}

impl MetricCollector for WiFiCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "wifi_signal_strength_dbm".to_string(),
                description: "WiFi signal strength in dBm".to_string(),
                unit: "dBm".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "wifi_signal_quality_percent".to_string(),
                description: "WiFi signal quality percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "wifi_connection_state".to_string(),
                description: "WiFi connection state".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Info,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "wifi_frequency_mhz".to_string(),
                description: "WiFi frequency in MHz".to_string(),
                unit: "MHz".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            MetricMetadata {
                name: "wifi_latency_ms".to_string(),
                description: "WiFi latency in milliseconds".to_string(),
                unit: "ms".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["interface".to_string()],
                category: MetricCategory::Network,
            },
            // Additional WiFi metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of WiFi metrics collection
        // This would use platform-specific APIs to gather WiFi information
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 6. Storage Usage Monitoring

### 6.1 Storage Metrics Collector

```rust
/// Storage metrics collector
pub struct StorageCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
    last_io_stats: Option<HashMap<String, DiskIOStats>>,
}

impl StorageCollector {
    /// Create a new StorageCollector
    pub fn new() -> Self;
}

impl MetricCollector for StorageCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "storage_total_bytes".to_string(),
                description: "Total storage capacity in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 60000, // 1 minute
                enabled_by_default: true,
                supported_labels: vec!["device".to_string(), "mountpoint".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_used_bytes".to_string(),
                description: "Used storage in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 60000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string(), "mountpoint".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_free_bytes".to_string(),
                description: "Free storage in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 60000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string(), "mountpoint".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_usage_percent".to_string(),
                description: "Storage usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 60000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string(), "mountpoint".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_read_bytes".to_string(),
                description: "Storage bytes read".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_written_bytes".to_string(),
                description: "Storage bytes written".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_io_time_ms".to_string(),
                description: "Storage I/O time in milliseconds".to_string(),
                unit: "ms".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string()],
                category: MetricCategory::Storage,
            },
            MetricMetadata {
                name: "storage_io_operations".to_string(),
                description: "Storage I/O operations".to_string(),
                unit: "operations".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string(), "operation".to_string()],
                category: MetricCategory::Storage,
            },
            // Additional storage metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of storage metrics collection
        // This would use platform-specific APIs to gather storage information
        // For Linux, this might involve reading from /proc/diskstats and using statvfs
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 7. Process Monitoring

### 7.1 Process Metrics Collector

```rust
/// Process metrics collector
pub struct ProcessCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
    processes_to_monitor: HashSet<String>,
}

impl ProcessCollector {
    /// Create a new ProcessCollector
    pub fn new() -> Self;
    
    /// Add process to monitor
    pub fn add_process(&mut self, process_name: &str);
    
    /// Remove process from monitoring
    pub fn remove_process(&mut self, process_name: &str);
}

impl MetricCollector for ProcessCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "process_cpu_percent".to_string(),
                description: "Process CPU usage percentage".to_string(),
                unit: "%".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_memory_rss_bytes".to_string(),
                description: "Process resident set size in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_memory_virtual_bytes".to_string(),
                description: "Process virtual memory size in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_threads".to_string(),
                description: "Number of threads in process".to_string(),
                unit: "threads".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_open_files".to_string(),
                description: "Number of open files by process".to_string(),
                unit: "files".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_io_read_bytes".to_string(),
                description: "Process I/O bytes read".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_io_write_bytes".to_string(),
                description: "Process I/O bytes written".to_string(),
                unit: "bytes".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            MetricMetadata {
                name: "process_status".to_string(),
                description: "Process status".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Info,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["process".to_string(), "pid".to_string()],
                category: MetricCategory::Process,
            },
            // Additional process metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of process metrics collection
        // This would use platform-specific APIs to gather process information
        // For Linux, this might involve reading from /proc/<pid>/
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 8. Thermal Monitoring

### 8.1 Thermal Metrics Collector

```rust
/// Thermal metrics collector
pub struct ThermalCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
}

impl ThermalCollector {
    /// Create a new ThermalCollector
    pub fn new() -> Self;
}

impl MetricCollector for ThermalCollector {
    // Implementation of MetricCollector trait methods
    
    fn metrics(&self) -> Vec<MetricMetadata> {
        vec![
            MetricMetadata {
                name: "thermal_zone_temperature_celsius".to_string(),
                description: "Thermal zone temperature in Celsius".to_string(),
                unit: "°C".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["zone".to_string()],
                category: MetricCategory::Thermal,
            },
            MetricMetadata {
                name: "thermal_zone_trip_point_celsius".to_string(),
                description: "Thermal zone trip point in Celsius".to_string(),
                unit: "°C".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 60000, // 1 minute
                enabled_by_default: true,
                supported_labels: vec!["zone".to_string(), "type".to_string()],
                category: MetricCategory::Thermal,
            },
            MetricMetadata {
                name: "thermal_cooling_device_state".to_string(),
                description: "Thermal cooling device state".to_string(),
                unit: "".to_string(),
                metric_type: MetricType::Gauge,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["device".to_string()],
                category: MetricCategory::Thermal,
            },
            MetricMetadata {
                name: "thermal_throttling_events".to_string(),
                description: "Thermal throttling events".to_string(),
                unit: "events".to_string(),
                metric_type: MetricType::Counter,
                default_interval: 5000,
                enabled_by_default: true,
                supported_labels: vec!["component".to_string()],
                category: MetricCategory::Thermal,
            },
            // Additional thermal metrics...
        ]
    }
    
    fn collect(&self) -> Result<Vec<MetricDataPoint>> {
        // Implementation of thermal metrics collection
        // This would use platform-specific APIs to gather thermal information
        // For Linux, this might involve reading from /sys/class/thermal/
        
        // Similar implementation to CPUCollector
        Ok(Vec::new())
    }
}
```

## 9. Metrics Storage

### 9.1 Metrics Storage Interface

```rust
/// Metrics storage trait
pub trait MetricsStorage: Send + Sync {
    /// Store metrics
    fn store(&mut self, metrics: &[MetricDataPoint]) -> Result<()>;
    
    /// Query metrics
    fn query(&self, query: &MetricsQuery) -> Result<Vec<MetricDataPoint>>;
    
    /// Get metric names
    fn get_metric_names(&self) -> Result<HashSet<String>>;
    
    /// Get metric labels
    fn get_metric_labels(&self, metric_name: &str) -> Result<HashSet<String>>;
    
    /// Get label values
    fn get_label_values(&self, metric_name: &str, label_name: &str) -> Result<HashSet<String>>;
    
    /// Clear metrics
    fn clear(&mut self) -> Result<()>;
    
    /// Clear metrics older than timestamp
    fn clear_older_than(&mut self, timestamp: u64) -> Result<()>;
}

/// Metrics query
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsQuery {
    /// Metric names to query
    pub metric_names: Option<Vec<String>>,
    /// Label filters
    pub label_filters: HashMap<String, String>,
    /// Start timestamp
    pub start_time: Option<u64>,
    /// End timestamp
    pub end_time: Option<u64>,
    /// Maximum number of data points
    pub limit: Option<usize>,
    /// Aggregation function
    pub aggregation: Option<AggregationFunction>,
    /// Aggregation interval in milliseconds
    pub aggregation_interval: Option<u64>,
}

/// Aggregation function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationFunction {
    /// Average
    Average,
    /// Sum
    Sum,
    /// Minimum
    Min,
    /// Maximum
    Max,
    /// Count
    Count,
}
```

### 9.2 In-Memory Storage

```rust
/// In-memory metrics storage
pub struct InMemoryStorage {
    metrics: Vec<MetricDataPoint>,
    max_size: usize,
    retention_period: Option<u64>,
}

impl InMemoryStorage {
    /// Create a new InMemoryStorage
    pub fn new(max_size: usize, retention_period: Option<u64>) -> Self;
}

impl MetricsStorage for InMemoryStorage {
    // Implementation of MetricsStorage trait methods
}
```

### 9.3 File-Based Storage

```rust
/// File-based metrics storage
pub struct FileStorage {
    base_path: PathBuf,
    max_file_size: usize,
    retention_period: Option<u64>,
    current_file: Option<File>,
    current_file_size: usize,
}

impl FileStorage {
    /// Create a new FileStorage
    pub fn new(base_path: &Path, max_file_size: usize, retention_period: Option<u64>) -> Result<Self>;
}

impl MetricsStorage for FileStorage {
    // Implementation of MetricsStorage trait methods
}
```

## 10. Metrics Analysis

### 10.1 Trend Analysis

```rust
/// Trend analyzer
pub struct TrendAnalyzer {
    storage: Arc<dyn MetricsStorage>,
}

impl TrendAnalyzer {
    /// Create a new TrendAnalyzer
    pub fn new(storage: Arc<dyn MetricsStorage>) -> Self;
    
    /// Calculate trend for metric
    pub fn calculate_trend(&self, metric_name: &str, labels: Option<HashMap<String, String>>, 
                          window: u64) -> Result<TrendResult>;
    
    /// Detect anomalies in trend
    pub fn detect_anomalies(&self, metric_name: &str, labels: Option<HashMap<String, String>>, 
                           window: u64, threshold: f64) -> Result<Vec<AnomalyPoint>>;
}

/// Trend result
#[derive(Debug, Clone, PartialEq)]
pub struct TrendResult {
    /// Metric name
    pub metric_name: String,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Slope (change per unit time)
    pub slope: f64,
    /// Intercept
    pub intercept: f64,
    /// Correlation coefficient
    pub correlation: f64,
    /// Prediction for next value
    pub next_prediction: f64,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

/// Anomaly point
#[derive(Debug, Clone, PartialEq)]
pub struct AnomalyPoint {
    /// Metric data point
    pub data_point: MetricDataPoint,
    /// Expected value
    pub expected_value: f64,
    /// Deviation
    pub deviation: f64,
    /// Severity
    pub severity: AnomalySeverity,
}

/// Anomaly severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnomalySeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}
```

## 11. Alerting System

### 11.1 Alert Definition

```rust
/// Alert definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertDefinition {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: String,
    /// Metric name
    pub metric_name: String,
    /// Label filters
    pub label_filters: HashMap<String, String>,
    /// Threshold type
    pub threshold_type: ThresholdType,
    /// Threshold value
    pub threshold_value: f64,
    /// Duration in milliseconds (how long condition must be true)
    pub duration: Option<u64>,
    /// Severity
    pub severity: AlertSeverity,
    /// Enabled
    pub enabled: bool,
}

/// Threshold type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Equal to
    EqualTo,
    /// Not equal to
    NotEqualTo,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}
```

### 11.2 Alert Manager

```rust
/// Alert manager
pub struct AlertManager {
    storage: Arc<dyn MetricsStorage>,
    alert_definitions: HashMap<String, AlertDefinition>,
    active_alerts: HashMap<String, Alert>,
    alert_history: Vec<Alert>,
    notifiers: Vec<Box<dyn AlertNotifier>>,
}

impl AlertManager {
    /// Create a new AlertManager
    pub fn new(storage: Arc<dyn MetricsStorage>) -> Self;
    
    /// Add alert definition
    pub fn add_alert_definition(&mut self, definition: AlertDefinition) -> Result<()>;
    
    /// Remove alert definition
    pub fn remove_alert_definition(&mut self, id: &str) -> Result<()>;
    
    /// Get alert definition
    pub fn get_alert_definition(&self, id: &str) -> Option<&AlertDefinition>;
    
    /// Get all alert definitions
    pub fn get_all_alert_definitions(&self) -> Vec<&AlertDefinition>;
    
    /// Enable alert definition
    pub fn enable_alert_definition(&mut self, id: &str) -> Result<()>;
    
    /// Disable alert definition
    pub fn disable_alert_definition(&mut self, id: &str) -> Result<()>;
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert>;
    
    /// Get alert history
    pub fn get_alert_history(&self) -> Vec<&Alert>;
    
    /// Add alert notifier
    pub fn add_notifier(&mut self, notifier: Box<dyn AlertNotifier>) -> Result<()>;
    
    /// Check alerts
    pub fn check_alerts(&mut self) -> Result<Vec<Alert>>;
}

/// Alert
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert definition ID
    pub definition_id: String,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp (None if still active)
    pub end_time: Option<u64>,
    /// Metric value that triggered the alert
    pub value: f64,
    /// Severity
    pub severity: AlertSeverity,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Alert notifier trait
pub trait AlertNotifier: Send + Sync {
    /// Notify about new alert
    fn notify_new_alert(&self, alert: &Alert) -> Result<()>;
    
    /// Notify about resolved alert
    fn notify_resolved_alert(&self, alert: &Alert) -> Result<()>;
}
```

## 12. MonitoringManager Integration

Update the existing `MonitoringManager` to integrate all the new functionality:

```rust
pub struct MonitoringManager {
    config: Arc<ConfigManager>,
    metric_registry: MetricRegistry,
    collector_registry: CollectorRegistry,
    storage: Arc<dyn MetricsStorage>,
    trend_analyzer: TrendAnalyzer,
    alert_manager: AlertManager,
    collection_thread: Option<JoinHandle<()>>,
    shutdown_signal: Arc<AtomicBool>,
    initialized: bool,
}

impl MonitoringManager {
    /// Create a new MonitoringManager
    pub fn new(config: &ConfigManager) -> Result<Self> {
        let storage: Arc<dyn MetricsStorage> = Arc::new(InMemoryStorage::new(10000, Some(3600000))); // 1 hour retention
        
        let trend_analyzer = TrendAnalyzer::new(Arc::clone(&storage));
        
        let alert_manager = AlertManager::new(Arc::clone(&storage));
        
        let manager = Self {
            config: Arc::new(config.clone()),
            metric_registry: MetricRegistry::new(),
            collector_registry: CollectorRegistry::new(),
            storage,
            trend_analyzer,
            alert_manager,
            collection_thread: None,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            initialized: false,
        };
        
        Ok(manager)
    }
    
    /// Initialize the monitoring manager
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Register default collectors
        self.register_default_collectors()?;
        
        // Start collection thread
        self.start_collection_thread()?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// Register default collectors
    fn register_default_collectors(&mut self) -> Result<()> {
        // CPU collector
        let cpu_collector = Box::new(CPUCollector::new());
        self.collector_registry.register_collector(cpu_collector)?;
        
        // GPU collector
        let gpu_collector = Box::new(GPUCollector::new());
        self.collector_registry.register_collector(gpu_collector)?;
        
        // Memory collector
        let memory_collector = Box::new(MemoryCollector::new());
        self.collector_registry.register_collector(memory_collector)?;
        
        // Battery collector
        let battery_collector = Box::new(BatteryCollector::new());
        self.collector_registry.register_collector(battery_collector)?;
        
        // Power consumption collector
        let power_collector = Box::new(PowerConsumptionCollector::new());
        self.collector_registry.register_collector(power_collector)?;
        
        // Network collector
        let network_collector = Box::new(NetworkCollector::new());
        self.collector_registry.register_collector(network_collector)?;
        
        // WiFi collector
        let wifi_collector = Box::new(WiFiCollector::new());
        self.collector_registry.register_collector(wifi_collector)?;
        
        // Storage collector
        let storage_collector = Box::new(StorageCollector::new());
        self.collector_registry.register_collector(storage_collector)?;
        
        // Process collector
        let mut process_collector = ProcessCollector::new();
        
        // Add critical processes to monitor
        process_collector.add_process("vr_core_api");
        process_collector.add_process("vr_web");
        process_collector.add_process("vr_cli");
        process_collector.add_process("steamvr");
        
        self.collector_registry.register_collector(Box::new(process_collector))?;
        
        // Thermal collector
        let thermal_collector = Box::new(ThermalCollector::new());
        self.collector_registry.register_collector(thermal_collector)?;
        
        Ok(())
    }
    
    /// Start collection thread
    fn start_collection_thread(&mut self) -> Result<()> {
        let collector_registry = self.collector_registry.clone();
        let storage = Arc::clone(&self.storage);
        let alert_manager = self.alert_manager.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        
        let handle = thread::spawn(move || {
            let mut collector_intervals: HashMap<String, (Instant, u64)> = HashMap::new();
            
            while !shutdown_signal.load(Ordering::SeqCst) {
                let now = Instant::now();
                
                // Check which collectors need to run
                for collector in collector_registry.get_all_collectors() {
                    if !collector.is_enabled() {
                        continue;
                    }
                    
                    let collector_name = collector.name().to_string();
                    let interval = collector.collection_interval();
                    
                    let should_collect = if let Some((last_run, _)) = collector_intervals.get(&collector_name) {
                        now.duration_since(*last_run).as_millis() as u64 >= interval
                    } else {
                        true // First run
                    };
                    
                    if should_collect {
                        match collector.collect() {
                            Ok(metrics) => {
                                if let Err(e) = storage.store(&metrics) {
                                    eprintln!("Failed to store metrics from {}: {}", collector_name, e);
                                }
                                collector_intervals.insert(collector_name, (now, interval));
                            }
                            Err(e) => {
                                eprintln!("Failed to collect metrics from {}: {}", collector_name, e);
                            }
                        }
                    }
                }
                
                // Check alerts
                if let Err(e) = alert_manager.check_alerts() {
                    eprintln!("Failed to check alerts: {}", e);
                }
                
                // Sleep for a short time
                thread::sleep(Duration::from_millis(100));
            }
        });
        
        self.collection_thread = Some(handle);
        Ok(())
    }
    
    /// Get metric registry
    pub fn metric_registry(&self) -> &MetricRegistry {
        &self.metric_registry
    }
    
    /// Get collector registry
    pub fn collector_registry(&self) -> &CollectorRegistry {
        &self.collector_registry
    }
    
    /// Get mutable collector registry
    pub fn collector_registry_mut(&mut self) -> &mut CollectorRegistry {
        &mut self.collector_registry
    }
    
    /// Get metrics storage
    pub fn storage(&self) -> Arc<dyn MetricsStorage> {
        Arc::clone(&self.storage)
    }
    
    /// Get trend analyzer
    pub fn trend_analyzer(&self) -> &TrendAnalyzer {
        &self.trend_analyzer
    }
    
    /// Get alert manager
    pub fn alert_manager(&self) -> &AlertManager {
        &self.alert_manager
    }
    
    /// Get mutable alert manager
    pub fn alert_manager_mut(&mut self) -> &mut AlertManager {
        &mut self.alert_manager
    }
    
    /// Query metrics
    pub fn query_metrics(&self, query: &MetricsQuery) -> Result<Vec<MetricDataPoint>> {
        self.storage.query(query)
    }
    
    /// Add custom collector
    pub fn add_custom_collector(&mut self, collector: Box<dyn MetricCollector>) -> Result<()> {
        self.collector_registry.register_collector(collector)
    }
    
    /// Add alert definition
    pub fn add_alert_definition(&mut self, definition: AlertDefinition) -> Result<()> {
        self.alert_manager.add_alert_definition(definition)
    }
    
    /// Shutdown the monitoring manager
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        // Signal collection thread to stop
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Wait for collection thread to finish
        if let Some(handle) = self.collection_thread.take() {
            if let Err(e) = handle.join() {
                eprintln!("Failed to join collection thread: {:?}", e);
            }
        }
        
        self.initialized = false;
        Ok(())
    }
}
```

## 13. Implementation Strategy

### 13.1 Phase 1: Core Metrics System

1. Define metric types and data structures
2. Implement MetricRegistry
3. Define MetricCollector trait
4. Implement CollectorRegistry

### 13.2 Phase 2: Metrics Storage

1. Define MetricsStorage trait
2. Implement InMemoryStorage
3. Implement FileStorage
4. Create query system

### 13.3 Phase 3: Collectors Implementation

1. Implement CPUCollector and GPUCollector
2. Implement MemoryCollector
3. Implement BatteryCollector and PowerConsumptionCollector
4. Implement NetworkCollector and WiFiCollector
5. Implement StorageCollector
6. Implement ProcessCollector
7. Implement ThermalCollector

### 13.4 Phase 4: Analysis and Alerting

1. Implement TrendAnalyzer
2. Define AlertDefinition and Alert structures
3. Implement AlertManager
4. Create alert notifiers

### 13.5 Phase 5: Integration

1. Update MonitoringManager to use all new components
2. Implement collection thread
3. Add configuration options
4. Create API for querying metrics and managing alerts

## 14. Testing Plan

### 14.1 Unit Tests

- Test each collector individually
- Test metrics storage implementations
- Test trend analysis functions
- Test alert detection logic

### 14.2 Integration Tests

- Test MonitoringManager with all components
- Test collection and storage of metrics
- Test alert generation and notification
- Test performance impact of monitoring

### 14.3 Mock Implementations

Create mock implementations for testing:

```rust
pub struct MockCollector {
    name: String,
    description: String,
    enabled: bool,
    interval: u64,
    metrics: Vec<MetricMetadata>,
    data_points: Vec<MetricDataPoint>,
}

impl MockCollector {
    pub fn new(name: &str, metrics: Vec<MetricMetadata>, data_points: Vec<MetricDataPoint>) -> Self;
}

impl MetricCollector for MockCollector {
    // Mock implementation
}
```

## 15. Documentation Plan

### 15.1 API Documentation

- Document all public traits, structs, and methods
- Include examples for common monitoring operations
- Document metric types and categories
- Document alert system

### 15.2 User Guide

- Create guide for monitoring configuration
- Document available metrics and their meaning
- Provide alert configuration examples
- Include troubleshooting information

## 16. Timeline and Milestones

1. **Week 1**: Implement core metrics system and storage
2. **Week 2**: Implement performance and battery collectors
3. **Week 3**: Implement network, storage, and process collectors
4. **Week 4**: Implement analysis and alerting system
5. **Week 5**: Integration, testing, and documentation

## 17. Dependencies and Requirements

- Rust crates:
  - `serde` and `serde_derive` for serialization
  - `thiserror` for error handling
  - `log` for logging
  - `chrono` for time handling
  - `sysinfo` or similar for system information
  - `tokio` for async operations (optional)

This implementation plan provides a comprehensive approach to expanding the system monitoring interfaces in the VR Core API layer, covering performance metrics, battery monitoring, network status, storage usage, and process monitoring while ensuring proper integration with the existing codebase.
