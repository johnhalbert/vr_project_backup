//! Performance monitoring for the VR headset.
//!
//! This module provides comprehensive performance monitoring capabilities
//! for CPU, GPU, memory, and thermal metrics.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::metrics::{Metric, MetricsCollector, MetricType, MetricValue};

/// CPU metrics collector.
#[derive(Debug)]
pub struct CpuMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Last CPU stats
    last_stats: Mutex<Option<CpuStats>>,
}

impl CpuMetricsCollector {
    /// Create a new CPU metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "cpu".to_string(),
            interval: Duration::from_secs(interval_secs),
            last_stats: Mutex::new(None),
        }
    }
    
    /// Get CPU stats.
    fn get_cpu_stats(&self) -> CpuStats {
        // In a real implementation, this would read from /proc/stat
        // For now, we'll simulate some values
        CpuStats {
            user: 10000,
            nice: 500,
            system: 5000,
            idle: 80000,
            iowait: 1000,
            irq: 500,
            softirq: 200,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        }
    }
    
    /// Calculate CPU usage.
    fn calculate_cpu_usage(&self, current: &CpuStats, previous: &CpuStats) -> f64 {
        let prev_idle = previous.idle + previous.iowait;
        let idle = current.idle + current.iowait;
        
        let prev_non_idle = previous.user + previous.nice + previous.system + 
                           previous.irq + previous.softirq + previous.steal;
        let non_idle = current.user + current.nice + current.system + 
                      current.irq + current.softirq + current.steal;
        
        let prev_total = prev_idle + prev_non_idle;
        let total = idle + non_idle;
        
        let total_delta = total - prev_total;
        let idle_delta = idle - prev_idle;
        
        if total_delta == 0 {
            return 0.0;
        }
        
        let usage = 100.0 * (total_delta - idle_delta) as f64 / total_delta as f64;
        usage.max(0.0).min(100.0)
    }
}

impl MetricsCollector for CpuMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get current CPU stats
        let current_stats = self.get_cpu_stats();
        
        // Calculate CPU usage if we have previous stats
        let mut last_stats = self.last_stats.lock().unwrap();
        
        if let Some(previous_stats) = last_stats.as_ref() {
            let usage = self.calculate_cpu_usage(&current_stats, previous_stats);
            
            // CPU usage metric
            let mut labels = HashMap::new();
            labels.insert("component".to_string(), "cpu".to_string());
            
            metrics.push(Metric::new(
                "cpu.usage",
                MetricType::Gauge,
                MetricValue::Float(usage),
                Some(labels),
                Some("CPU usage percentage"),
                Some("%"),
            ));
        }
        
        // Update last stats
        *last_stats = Some(current_stats);
        
        // Add CPU frequency metric (simulated)
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "cpu".to_string());
        
        metrics.push(Metric::new(
            "cpu.frequency",
            MetricType::Gauge,
            MetricValue::Float(2400.0),
            Some(labels),
            Some("CPU frequency"),
            Some("MHz"),
        ));
        
        // Add CPU temperature metric (simulated)
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "cpu".to_string());
        
        metrics.push(Metric::new(
            "cpu.temperature",
            MetricType::Gauge,
            MetricValue::Float(45.5),
            Some(labels),
            Some("CPU temperature"),
            Some("°C"),
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// CPU statistics.
#[derive(Debug, Clone, Copy)]
struct CpuStats {
    /// User mode
    user: u64,
    
    /// Nice user mode
    nice: u64,
    
    /// System mode
    system: u64,
    
    /// Idle task
    idle: u64,
    
    /// I/O wait
    iowait: u64,
    
    /// Servicing interrupts
    irq: u64,
    
    /// Servicing soft IRQs
    softirq: u64,
    
    /// Involuntary wait
    steal: u64,
    
    /// Running a virtual CPU
    guest: u64,
    
    /// Running a niced guest
    guest_nice: u64,
}

/// GPU metrics collector.
#[derive(Debug)]
pub struct GpuMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
}

impl GpuMetricsCollector {
    /// Create a new GPU metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "gpu".to_string(),
            interval: Duration::from_secs(interval_secs),
        }
    }
}

impl MetricsCollector for GpuMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // GPU usage metric (simulated)
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "gpu".to_string());
        
        metrics.push(Metric::new(
            "gpu.usage",
            MetricType::Gauge,
            MetricValue::Float(30.0),
            Some(labels.clone()),
            Some("GPU usage percentage"),
            Some("%"),
        ));
        
        // GPU memory usage metric (simulated)
        metrics.push(Metric::new(
            "gpu.memory.used",
            MetricType::Gauge,
            MetricValue::Integer(512),
            Some(labels.clone()),
            Some("GPU memory used"),
            Some("MB"),
        ));
        
        // GPU memory total metric (simulated)
        metrics.push(Metric::new(
            "gpu.memory.total",
            MetricType::Gauge,
            MetricValue::Integer(2048),
            Some(labels.clone()),
            Some("GPU memory total"),
            Some("MB"),
        ));
        
        // GPU temperature metric (simulated)
        metrics.push(Metric::new(
            "gpu.temperature",
            MetricType::Gauge,
            MetricValue::Float(55.0),
            Some(labels),
            Some("GPU temperature"),
            Some("°C"),
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Memory metrics collector.
#[derive(Debug)]
pub struct MemoryMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
}

impl MemoryMetricsCollector {
    /// Create a new memory metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "memory".to_string(),
            interval: Duration::from_secs(interval_secs),
        }
    }
    
    /// Get memory stats.
    fn get_memory_stats(&self) -> MemoryStats {
        // In a real implementation, this would read from /proc/meminfo
        // For now, we'll simulate some values
        MemoryStats {
            total: 4096,
            free: 1024,
            available: 2048,
            buffers: 256,
            cached: 768,
            swap_total: 2048,
            swap_free: 1536,
        }
    }
}

impl MetricsCollector for MemoryMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get memory stats
        let stats = self.get_memory_stats();
        
        // Memory labels
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "memory".to_string());
        
        // Memory total metric
        metrics.push(Metric::new(
            "memory.total",
            MetricType::Gauge,
            MetricValue::Integer(stats.total),
            Some(labels.clone()),
            Some("Total memory"),
            Some("MB"),
        ));
        
        // Memory free metric
        metrics.push(Metric::new(
            "memory.free",
            MetricType::Gauge,
            MetricValue::Integer(stats.free),
            Some(labels.clone()),
            Some("Free memory"),
            Some("MB"),
        ));
        
        // Memory available metric
        metrics.push(Metric::new(
            "memory.available",
            MetricType::Gauge,
            MetricValue::Integer(stats.available),
            Some(labels.clone()),
            Some("Available memory"),
            Some("MB"),
        ));
        
        // Memory used metric
        let used = stats.total - stats.free;
        metrics.push(Metric::new(
            "memory.used",
            MetricType::Gauge,
            MetricValue::Integer(used),
            Some(labels.clone()),
            Some("Used memory"),
            Some("MB"),
        ));
        
        // Memory usage percentage
        let usage_percent = if stats.total > 0 {
            100.0 * used as f64 / stats.total as f64
        } else {
            0.0
        };
        
        metrics.push(Metric::new(
            "memory.usage",
            MetricType::Gauge,
            MetricValue::Float(usage_percent),
            Some(labels.clone()),
            Some("Memory usage percentage"),
            Some("%"),
        ));
        
        // Swap metrics
        let mut swap_labels = HashMap::new();
        swap_labels.insert("component".to_string(), "swap".to_string());
        
        metrics.push(Metric::new(
            "memory.swap.total",
            MetricType::Gauge,
            MetricValue::Integer(stats.swap_total),
            Some(swap_labels.clone()),
            Some("Total swap"),
            Some("MB"),
        ));
        
        metrics.push(Metric::new(
            "memory.swap.free",
            MetricType::Gauge,
            MetricValue::Integer(stats.swap_free),
            Some(swap_labels.clone()),
            Some("Free swap"),
            Some("MB"),
        ));
        
        let swap_used = stats.swap_total - stats.swap_free;
        metrics.push(Metric::new(
            "memory.swap.used",
            MetricType::Gauge,
            MetricValue::Integer(swap_used),
            Some(swap_labels.clone()),
            Some("Used swap"),
            Some("MB"),
        ));
        
        let swap_usage_percent = if stats.swap_total > 0 {
            100.0 * swap_used as f64 / stats.swap_total as f64
        } else {
            0.0
        };
        
        metrics.push(Metric::new(
            "memory.swap.usage",
            MetricType::Gauge,
            MetricValue::Float(swap_usage_percent),
            Some(swap_labels),
            Some("Swap usage percentage"),
            Some("%"),
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Memory statistics.
#[derive(Debug, Clone, Copy)]
struct MemoryStats {
    /// Total memory
    total: i64,
    
    /// Free memory
    free: i64,
    
    /// Available memory
    available: i64,
    
    /// Buffer memory
    buffers: i64,
    
    /// Cached memory
    cached: i64,
    
    /// Total swap
    swap_total: i64,
    
    /// Free swap
    swap_free: i64,
}

/// Thermal metrics collector.
#[derive(Debug)]
pub struct ThermalMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
}

impl ThermalMetricsCollector {
    /// Create a new thermal metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            name: "thermal".to_string(),
            interval: Duration::from_secs(interval_secs),
        }
    }
}

impl MetricsCollector for ThermalMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Thermal zones (simulated)
        let zones = vec![
            ("cpu", 45.5),
            ("gpu", 55.0),
            ("battery", 35.0),
            ("ambient", 30.0),
        ];
        
        for (zone, temp) in zones {
            let mut labels = HashMap::new();
            labels.insert("zone".to_string(), zone.to_string());
            
            metrics.push(Metric::new(
                &format!("thermal.{}", zone),
                MetricType::Gauge,
                MetricValue::Float(temp),
                Some(labels),
                Some(&format!("{} temperature", zone)),
                Some("°C"),
            ));
        }
        
        // Thermal throttling state (simulated)
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "system".to_string());
        
        metrics.push(Metric::new(
            "thermal.throttling",
            MetricType::State,
            MetricValue::Boolean(false),
            Some(labels),
            Some("Thermal throttling active"),
            None,
        ));
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Performance monitoring manager.
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Metric collectors
    collectors: Vec<Arc<dyn MetricsCollector>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor.
    pub fn new() -> Self {
        Self {
            collectors: Vec::new(),
        }
    }
    
    /// Initialize with default collectors.
    pub fn init_default(&mut self) {
        // CPU metrics collector (collect every 2 seconds)
        self.collectors.push(Arc::new(CpuMetricsCollector::new(2)));
        
        // GPU metrics collector (collect every 2 seconds)
        self.collectors.push(Arc::new(GpuMetricsCollector::new(2)));
        
        // Memory metrics collector (collect every 5 seconds)
        self.collectors.push(Arc::new(MemoryMetricsCollector::new(5)));
        
        // Thermal metrics collector (collect every 5 seconds)
        self.collectors.push(Arc::new(ThermalMetricsCollector::new(5)));
    }
    
    /// Add a collector.
    pub fn add_collector(&mut self, collector: Arc<dyn MetricsCollector>) {
        self.collectors.push(collector);
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> &[Arc<dyn MetricsCollector>] {
        &self.collectors
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        let mut monitor = Self::new();
        monitor.init_default();
        monitor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_metrics_collector() {
        let collector = CpuMetricsCollector::new(2);
        
        // First collection should not produce usage metrics
        let metrics = collector.collect();
        assert_eq!(metrics.len(), 2); // frequency and temperature
        
        // Second collection should produce usage metrics
        let metrics = collector.collect();
        assert_eq!(metrics.len(), 3); // usage, frequency, and temperature
        
        // Check metric types
        let usage_metric = metrics.iter().find(|m| m.name == "cpu.usage").unwrap();
        assert_eq!(usage_metric.metric_type, MetricType::Gauge);
        assert!(usage_metric.value.as_float().is_some());
        
        let freq_metric = metrics.iter().find(|m| m.name == "cpu.frequency").unwrap();
        assert_eq!(freq_metric.metric_type, MetricType::Gauge);
        assert!(freq_metric.value.as_float().is_some());
        
        let temp_metric = metrics.iter().find(|m| m.name == "cpu.temperature").unwrap();
        assert_eq!(temp_metric.metric_type, MetricType::Gauge);
        assert!(temp_metric.value.as_float().is_some());
    }
    
    #[test]
    fn test_gpu_metrics_collector() {
        let collector = GpuMetricsCollector::new(2);
        let metrics = collector.collect();
        
        assert_eq!(metrics.len(), 4); // usage, memory used, memory total, temperature
        
        // Check metric types
        let usage_metric = metrics.iter().find(|m| m.name == "gpu.usage").unwrap();
        assert_eq!(usage_metric.metric_type, MetricType::Gauge);
        assert!(usage_metric.value.as_float().is_some());
        
        let mem_used_metric = metrics.iter().find(|m| m.name == "gpu.memory.used").unwrap();
        assert_eq!(mem_used_metric.metric_type, MetricType::Gauge);
        assert!(mem_used_metric.value.as_integer().is_some());
        
        let mem_total_metric = metrics.iter().find(|m| m.name == "gpu.memory.total").unwrap();
        assert_eq!(mem_total_metric.metric_type, MetricType::Gauge);
        assert!(mem_total_metric.value.as_integer().is_some());
        
        let temp_metric = metrics.iter().find(|m| m.name == "gpu.temperature").unwrap();
        assert_eq!(temp_metric.metric_type, MetricType::Gauge);
        assert!(temp_metric.value.as_float().is_some());
    }
    
    #[test]
    fn test_memory_metrics_collector() {
        let collector = MemoryMetricsCollector::new(5);
        let metrics = collector.collect();
        
        assert_eq!(metrics.len(), 9); // total, free, available, used, usage, swap total, swap free, swap used, swap usage
        
        // Check memory metrics
        let total_metric = metrics.iter().find(|m| m.name == "memory.total").unwrap();
        assert_eq!(total_metric.metric_type, MetricType::Gauge);
        assert!(total_metric.value.as_integer().is_some());
        
        let usage_metric = metrics.iter().find(|m| m.name == "memory.usage").unwrap();
        assert_eq!(usage_metric.metric_type, MetricType::Gauge);
        assert!(usage_metric.value.as_float().is_some());
        
        // Check swap metrics
        let swap_total_metric = metrics.iter().find(|m| m.name == "memory.swap.total").unwrap();
        assert_eq!(swap_total_metric.metric_type, MetricType::Gauge);
        assert!(swap_total_metric.value.as_integer().is_some());
        
        let swap_usage_metric = metrics.iter().find(|m| m.name == "memory.swap.usage").unwrap();
        assert_eq!(swap_usage_metric.metric_type, MetricType::Gauge);
        assert!(swap_usage_metric.value.as_float().is_some());
    }
    
    #[test]
    fn test_thermal_metrics_collector() {
        let collector = ThermalMetricsCollector::new(5);
        let metrics = collector.collect();
        
        assert_eq!(metrics.len(), 5); // cpu, gpu, battery, ambient, throttling
        
        // Check thermal zone metrics
        let cpu_temp_metric = metrics.iter().find(|m| m.name == "thermal.cpu").unwrap();
        assert_eq!(cpu_temp_metric.metric_type, MetricType::Gauge);
        assert!(cpu_temp_metric.value.as_float().is_some());
        
        let gpu_temp_metric = metrics.iter().find(|m| m.name == "thermal.gpu").unwrap();
        assert_eq!(gpu_temp_metric.metric_type, MetricType::Gauge);
        assert!(gpu_temp_metric.value.as_float().is_some());
        
        // Check throttling metric
        let throttling_metric = metrics.iter().find(|m| m.name == "thermal.throttling").unwrap();
        assert_eq!(throttling_metric.metric_type, MetricType::State);
        assert!(throttling_metric.value.as_boolean().is_some());
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        assert_eq!(monitor.collectors().len(), 0);
        
        monitor.init_default();
        assert_eq!(monitor.collectors().len(), 4);
        
        // Check collector types
        assert!(monitor.collectors().iter().any(|c| c.name() == "cpu"));
        assert!(monitor.collectors().iter().any(|c| c.name() == "gpu"));
        assert!(monitor.collectors().iter().any(|c| c.name() == "memory"));
        assert!(monitor.collectors().iter().any(|c| c.name() == "thermal"));
        
        // Add custom collector
        struct TestCollector;
        impl MetricCollector for TestCollector {
            fn name(&self) -> &str { "test" }
            fn collect(&self) -> Vec<Metric> { Vec::new() }
            fn interval(&self) -> Duration { Duration::from_secs(10) }
        }
        
        monitor.add_collector(Arc::new(TestCollector));
        assert_eq!(monitor.collectors().len(), 5);
    }
}
