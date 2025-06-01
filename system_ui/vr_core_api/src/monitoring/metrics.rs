//! Core metrics system for the VR headset.
//!
//! This module provides a comprehensive metrics collection and monitoring
//! system for tracking system performance, resource usage, and health.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

/// Metric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Counter metric (monotonically increasing)
    Counter,
    
    /// Gauge metric (can increase or decrease)
    Gauge,
    
    /// Histogram metric (distribution of values)
    Histogram,
    
    /// State metric (enumerated state)
    State,
}

impl MetricType {
    /// Get the metric type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::State => "state",
        }
    }
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metric value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    
    /// Float value
    Float(f64),
    
    /// String value
    String(String),
    
    /// Boolean value
    Boolean(bool),
    
    /// Histogram values
    Histogram(Vec<f64>),
}

impl MetricValue {
    /// Get the value as an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            MetricValue::Integer(v) => Some(*v),
            MetricValue::Float(v) => Some(*v as i64),
            MetricValue::Boolean(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }
    
    /// Get the value as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            MetricValue::Integer(v) => Some(*v as f64),
            MetricValue::Float(v) => Some(*v),
            MetricValue::Boolean(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
    
    /// Get the value as a string.
    pub fn as_string(&self) -> String {
        match self {
            MetricValue::Integer(v) => v.to_string(),
            MetricValue::Float(v) => v.to_string(),
            MetricValue::String(v) => v.clone(),
            MetricValue::Boolean(v) => v.to_string(),
            MetricValue::Histogram(v) => format!("{:?}", v),
        }
    }
    
    /// Get the value as a boolean.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            MetricValue::Integer(v) => Some(*v != 0),
            MetricValue::Float(v) => Some(*v != 0.0),
            MetricValue::Boolean(v) => Some(*v),
            MetricValue::String(v) => {
                match v.to_lowercase().as_str() {
                    "true" | "yes" | "1" | "on" => Some(true),
                    "false" | "no" | "0" | "off" => Some(false),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    /// Get the value as a histogram.
    pub fn as_histogram(&self) -> Option<&[f64]> {
        match self {
            MetricValue::Histogram(v) => Some(v),
            _ => None,
        }
    }
}

/// Metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    
    /// Metric type
    pub metric_type: MetricType,
    
    /// Metric value
    pub value: MetricValue,
    
    /// Metric timestamp
    pub timestamp: u64,
    
    /// Metric labels
    pub labels: HashMap<String, String>,
    
    /// Metric description
    pub description: Option<String>,
    
    /// Metric unit
    pub unit: Option<String>,
}

impl Metric {
    /// Create a new metric.
    pub fn new(
        name: &str,
        metric_type: MetricType,
        value: MetricValue,
        labels: Option<HashMap<String, String>>,
        description: Option<&str>,
        unit: Option<&str>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            name: name.to_string(),
            metric_type,
            value,
            timestamp,
            labels: labels.unwrap_or_default(),
            description: description.map(|s| s.to_string()),
            unit: unit.map(|s| s.to_string()),
        }
    }
    
    /// Add a label to the metric.
    pub fn add_label(&mut self, key: &str, value: &str) {
        self.labels.insert(key.to_string(), value.to_string());
    }
    
    /// Remove a label from the metric.
    pub fn remove_label(&mut self, key: &str) -> Option<String> {
        self.labels.remove(key)
    }
    
    /// Update the metric value.
    pub fn update_value(&mut self, value: MetricValue) {
        self.value = value;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Get the metric age in seconds.
    pub fn age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now.saturating_sub(self.timestamp)
    }
    
    /// Check if the metric is stale.
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        self.age() > max_age_secs
    }
}
/// Metric collector trait.
pub trait MetricsCollector: Send + Sync + std::fmt::Debug {
    /// Get the collector name.
    fn name(&self) -> &str;
    
    /// Collect metrics.
    fn collect(&self) -> Vec<Metric>;
    
    /// Get the collection interval.
    fn interval(&self) -> Duration;
}// Metric registry.
#[derive(Debug)]
pub struct MetricRegistry {
    /// Metrics
    metrics: RwLock<HashMap<String, Metric>>,
    
    /// Collectors
    collectors: RwLock<Vec<Arc<dyn MetricsCollector>>>,
    
    /// Last collection time
    last_collection: Mutex<HashMap<String, Instant>>,
}

impl MetricRegistry {
    /// Create a new metric registry.
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
            collectors: RwLock::new(Vec::new()),
            last_collection: Mutex::new(HashMap::new()),
        }
    }
    
    /// Register a collector.
    pub fn register_collector(&self, collector: Arc<dyn MetricsCollector>) {
        let mut collectors = self.collectors.write().unwrap();
        collectors.push(collector);
    }
    
    /// Unregister a collector.
    pub fn unregister_collector(&self, name: &str) -> bool {
        let mut collectors = self.collectors.write().unwrap();
        let len = collectors.len();
        collectors.retain(|c| c.name() != name);
        collectors.len() < len
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> Vec<Arc<dyn MetricsCollector>> {
        let collectors = self.collectors.read().unwrap();
        collectors.clone()
    }
    
    /// Register a metric.
    pub fn register_metric(&self, metric: Metric) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.insert(metric.name.clone(), metric);
    }
    
    /// Unregister a metric.
    pub fn unregister_metric(&self, name: &str) -> bool {
        let mut metrics = self.metrics.write().unwrap();
        metrics.remove(name).is_some()
    }
    
    /// Get a metric.
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(name).cloned()
    }
    
    /// Get all metrics.
    pub fn metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.values().cloned().collect()
    }
    
    /// Get metrics by type.
    pub fn metrics_by_type(&self, metric_type: MetricType) -> Vec<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.values()
            .filter(|m| m.metric_type == metric_type)
            .cloned()
            .collect()
    }
    
    /// Get metrics by label.
    pub fn metrics_by_label(&self, key: &str, value: &str) -> Vec<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.values()
            .filter(|m| m.labels.get(key).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }
    
    /// Update a metric.
    pub fn update_metric(&self, name: &str, value: MetricValue) -> bool {
        let mut metrics = self.metrics.write().unwrap();
        
        if let Some(metric) = metrics.get_mut(name) {
            metric.update_value(value);
            true
        } else {
            false
        }
    }
    
    /// Collect metrics from all collectors.
    pub fn collect_metrics(&self) {
        let collectors = self.collectors.read().unwrap();
        let mut last_collection = self.last_collection.lock().unwrap();
        let now = Instant::now();
        
        for collector in collectors.iter() {
            let collector_name = collector.name().to_string();
            let interval = collector.interval();
            
            // Check if it's time to collect
            if let Some(last) = last_collection.get(&collector_name) {
                if now.duration_since(*last) < interval {
                    continue;
                }
            }
            
            // Collect metrics
            let metrics = collector.collect();
            
            // Update metrics
            let mut registry_metrics = self.metrics.write().unwrap();
            for metric in metrics {
                registry_metrics.insert(metric.name.clone(), metric);
            }
            
            // Update last collection time
            last_collection.insert(collector_name, now);
        }
    }
    
    /// Remove stale metrics.
    pub fn remove_stale_metrics(&self, max_age_secs: u64) -> usize {
        let mut metrics = self.metrics.write().unwrap();
        let len = metrics.len();
        
        metrics.retain(|_, m| !m.is_stale(max_age_secs));
        
        len - metrics.len()
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metric_type() {
        assert_eq!(MetricType::Counter.as_str(), "counter");
        assert_eq!(MetricType::Gauge.as_str(), "gauge");
        assert_eq!(MetricType::Histogram.as_str(), "histogram");
        assert_eq!(MetricType::State.as_str(), "state");
        
        assert_eq!(MetricType::Counter.to_string(), "counter");
        assert_eq!(MetricType::Gauge.to_string(), "gauge");
        assert_eq!(MetricType::Histogram.to_string(), "histogram");
        assert_eq!(MetricType::State.to_string(), "state");
    }
    
    #[test]
    fn test_metric_value() {
        let int_value = MetricValue::Integer(42);
        let float_value = MetricValue::Float(3.14);
        let string_value = MetricValue::String("test".to_string());
        let bool_value = MetricValue::Boolean(true);
        let hist_value = MetricValue::Histogram(vec![1.0, 2.0, 3.0]);
        
        // Integer conversions
        assert_eq!(int_value.as_integer(), Some(42));
        assert_eq!(float_value.as_integer(), Some(3));
        assert_eq!(bool_value.as_integer(), Some(1));
        assert_eq!(string_value.as_integer(), None);
        assert_eq!(hist_value.as_integer(), None);
        
        // Float conversions
        assert_eq!(int_value.as_float(), Some(42.0));
        assert_eq!(float_value.as_float(), Some(3.14));
        assert_eq!(bool_value.as_float(), Some(1.0));
        assert_eq!(string_value.as_float(), None);
        assert_eq!(hist_value.as_float(), None);
        
        // String conversions
        assert_eq!(int_value.as_string(), "42");
        assert_eq!(float_value.as_string(), "3.14");
        assert_eq!(string_value.as_string(), "test");
        assert_eq!(bool_value.as_string(), "true");
        assert_eq!(hist_value.as_string(), "[1.0, 2.0, 3.0]");
        
        // Boolean conversions
        assert_eq!(int_value.as_boolean(), Some(true));
        assert_eq!(float_value.as_boolean(), Some(true));
        assert_eq!(bool_value.as_boolean(), Some(true));
        assert_eq!(MetricValue::String("true".to_string()).as_boolean(), Some(true));
        assert_eq!(MetricValue::String("false".to_string()).as_boolean(), Some(false));
        assert_eq!(MetricValue::String("invalid".to_string()).as_boolean(), None);
        assert_eq!(hist_value.as_boolean(), None);
        
        // Histogram conversions
        assert_eq!(int_value.as_histogram(), None);
        assert_eq!(float_value.as_histogram(), None);
        assert_eq!(string_value.as_histogram(), None);
        assert_eq!(bool_value.as_histogram(), None);
        assert_eq!(hist_value.as_histogram(), Some(&[1.0, 2.0, 3.0][..]));
    }
    
    #[test]
    fn test_metric() {
        let mut labels = HashMap::new();
        labels.insert("device".to_string(), "gpu".to_string());
        
        let mut metric = Metric::new(
            "gpu.temperature",
            MetricType::Gauge,
            MetricValue::Float(65.5),
            Some(labels),
            Some("GPU temperature"),
            Some("°C"),
        );
        
        assert_eq!(metric.name, "gpu.temperature");
        assert_eq!(metric.metric_type, MetricType::Gauge);
        assert_eq!(metric.value, MetricValue::Float(65.5));
        assert_eq!(metric.labels.get("device"), Some(&"gpu".to_string()));
        assert_eq!(metric.description, Some("GPU temperature".to_string()));
        assert_eq!(metric.unit, Some("°C".to_string()));
        
        // Add label
        metric.add_label("vendor", "nvidia");
        assert_eq!(metric.labels.get("vendor"), Some(&"nvidia".to_string()));
        
        // Remove label
        let removed = metric.remove_label("device");
        assert_eq!(removed, Some("gpu".to_string()));
        assert_eq!(metric.labels.get("device"), None);
        
        // Update value
        let original_timestamp = metric.timestamp;
        std::thread::sleep(std::time::Duration::from_secs(1));
        metric.update_value(MetricValue::Float(70.0));
        assert_eq!(metric.value, MetricValue::Float(70.0));
        assert!(metric.timestamp > original_timestamp);
        
        // Age and staleness
        assert!(metric.age() >= 0);
        assert!(!metric.is_stale(3600));
        assert!(metric.is_stale(0));
    }
    
    struct TestCollector {
        name: String,
        interval: Duration,
    }
    
    impl TestCollector {
        fn new(name: &str, interval_secs: u64) -> Self {
            Self {
                name: name.to_string(),
                interval: Duration::from_secs(interval_secs),
            }
        }
    }
    
    impl MetricsCollector for TestCollector {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn collect(&self) -> Vec<Metric> {
            vec![
                Metric::new(
                    &format!("{}.metric1", self.name),
                    MetricType::Counter,
                    MetricValue::Integer(42),
                    None,
                    None,
                    None,
                ),
                Metric::new(
                    &format!("{}.metric2", self.name),
                    MetricType::Gauge,
                    MetricValue::Float(3.14),
                    None,
                    None,
                    None,
                ),
            ]
        }
        
        fn interval(&self) -> Duration {
            self.interval
        }
    }
    
    #[test]
    fn test_metric_registry() {
        let registry = MetricRegistry::new();
        
        // Register collectors
        let collector1 = Arc::new(TestCollector::new("test1", 60));
        let collector2 = Arc::new(TestCollector::new("test2", 300));
        
        registry.register_collector(collector1.clone());
        registry.register_collector(collector2.clone());
        
        assert_eq!(registry.collectors().len(), 2);
        
        // Register metrics
        let metric1 = Metric::new(
            "cpu.usage",
            MetricType::Gauge,
            MetricValue::Float(25.0),
            None,
            Some("CPU usage"),
            Some("%"),
        );
        
        let metric2 = Metric::new(
            "memory.used",
            MetricType::Gauge,
            MetricValue::Integer(1024),
            None,
            Some("Memory used"),
            Some("MB"),
        );
        
        registry.register_metric(metric1.clone());
        registry.register_metric(metric2.clone());
        
        assert_eq!(registry.metrics().len(), 2);
        
        // Get metrics
        let cpu_metric = registry.get_metric("cpu.usage").unwrap();
        assert_eq!(cpu_metric.name, "cpu.usage");
        assert_eq!(cpu_metric.value, MetricValue::Float(25.0));
        
        // Update metric
        registry.update_metric("cpu.usage", MetricValue::Float(30.0));
        let cpu_metric = registry.get_metric("cpu.usage").unwrap();
        assert_eq!(cpu_metric.value, MetricValue::Float(30.0));
        
        // Metrics by type
        let gauge_metrics = registry.metrics_by_type(MetricType::Gauge);
        assert_eq!(gauge_metrics.len(), 2);
        
        // Collect metrics
        registry.collect_metrics();
        
        // Should have 4 metrics now (2 original + 2 from collector1)
        // collector2 won't be collected yet due to interval
        assert_eq!(registry.metrics().len(), 4);
        
        // Unregister collector
        assert!(registry.unregister_collector("test1"));
        assert_eq!(registry.collectors().len(), 1);
        
        // Unregister metric
        assert!(registry.unregister_metric("cpu.usage"));
        assert_eq!(registry.metrics().len(), 3);
        
        // Remove stale metrics (none should be stale)
        assert_eq!(registry.remove_stale_metrics(3600), 0);
    }
}
