//! Process monitoring for the VR headset.
//!
//! This module provides comprehensive process monitoring capabilities
//! for tracking system processes, resource usage, and application performance.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::metrics::{Metric, MetricsCollector, MetricType, MetricValue};

/// Process state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessState {
    /// Process is running
    Running,
    
    /// Process is sleeping
    Sleeping,
    
    /// Process is waiting for disk I/O
    Waiting,
    
    /// Process is stopped
    Stopped,
    
    /// Process is a zombie
    Zombie,
}

impl ProcessState {
    /// Get the process state as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::Running => "running",
            ProcessState::Sleeping => "sleeping",
            ProcessState::Waiting => "waiting",
            ProcessState::Stopped => "stopped",
            ProcessState::Zombie => "zombie",
        }
    }
    
    /// Parse a process state from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "running" => Some(ProcessState::Running),
            "sleeping" => Some(ProcessState::Sleeping),
            "waiting" => Some(ProcessState::Waiting),
            "stopped" => Some(ProcessState::Stopped),
            "zombie" => Some(ProcessState::Zombie),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process priority.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessPriority {
    /// Real-time priority
    RealTime,
    
    /// High priority
    High,
    
    /// Normal priority
    Normal,
    
    /// Low priority
    Low,
    
    /// Idle priority
    Idle,
}

impl ProcessPriority {
    /// Get the process priority as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessPriority::RealTime => "real_time",
            ProcessPriority::High => "high",
            ProcessPriority::Normal => "normal",
            ProcessPriority::Low => "low",
            ProcessPriority::Idle => "idle",
        }
    }
    
    /// Parse a process priority from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "real_time" => Some(ProcessPriority::RealTime),
            "high" => Some(ProcessPriority::High),
            "normal" => Some(ProcessPriority::Normal),
            "low" => Some(ProcessPriority::Low),
            "idle" => Some(ProcessPriority::Idle),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process statistics.
#[derive(Debug, Clone)]
pub struct ProcessStats {
    /// Process ID
    pub pid: u32,
    
    /// Parent process ID
    pub ppid: u32,
    
    /// Process name
    pub name: String,
    
    /// Command line
    pub cmdline: String,
    
    /// Process state
    pub state: ProcessState,
    
    /// Process priority
    pub priority: ProcessPriority,
    
    /// CPU usage percentage
    pub cpu_percent: f64,
    
    /// Memory usage in bytes
    pub memory_bytes: u64,
    
    /// Virtual memory size in bytes
    pub virtual_memory_bytes: u64,
    
    /// Resident set size in bytes
    pub resident_memory_bytes: u64,
    
    /// Number of threads
    pub thread_count: u32,
    
    /// Open file descriptors
    pub fd_count: u32,
    
    /// Process start time (seconds since epoch)
    pub start_time: u64,
    
    /// CPU time in seconds
    pub cpu_time: f64,
    
    /// I/O read bytes
    pub io_read_bytes: u64,
    
    /// I/O write bytes
    pub io_write_bytes: u64,
}

/// Process metrics collector.
#[derive(Debug)]
pub struct ProcessMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Last process statistics
    last_stats: Mutex<HashMap<u32, ProcessStats>>,
    
    /// Critical process list
    critical_processes: Vec<String>,
}

impl ProcessMetricsCollector {
    /// Create a new process metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        // Default critical processes
        let critical_processes = vec![
            "system_ui".to_string(),
            "slam_service".to_string(),
            "display_service".to_string(),
            "audio_service".to_string(),
            "tracking_service".to_string(),
        ];
        
        Self {
            name: "process".to_string(),
            interval: Duration::from_secs(interval_secs),
            last_stats: Mutex::new(HashMap::new()),
            critical_processes,
        }
    }
    
    /// Add a critical process to monitor.
    pub fn add_critical_process(&mut self, process_name: String) {
        if !self.critical_processes.contains(&process_name) {
            self.critical_processes.push(process_name);
        }
    }
    
    /// Remove a critical process from monitoring.
    pub fn remove_critical_process(&mut self, process_name: &str) {
        self.critical_processes.retain(|name| name != process_name);
    }
    
    /// Get the list of critical processes.
    pub fn get_critical_processes(&self) -> Vec<String> {
        self.critical_processes.clone()
    }
    
    /// Get process statistics.
    fn get_process_stats(&self) -> Vec<ProcessStats> {
        // In a real implementation, this would read from /proc/<pid>/...
        // For now, we'll simulate some values
        
        let mut processes = Vec::new();
        
        // System UI process
        processes.push(ProcessStats {
            pid: 1000,
            ppid: 1,
            name: "system_ui".to_string(),
            cmdline: "/usr/bin/system_ui".to_string(),
            state: ProcessState::Running,
            priority: ProcessPriority::Normal,
            cpu_percent: 5.0,
            memory_bytes: 50 * 1024 * 1024, // 50 MB
            virtual_memory_bytes: 200 * 1024 * 1024, // 200 MB
            resident_memory_bytes: 50 * 1024 * 1024, // 50 MB
            thread_count: 4,
            fd_count: 30,
            start_time: 1621500000,
            cpu_time: 120.0,
            io_read_bytes: 10 * 1024 * 1024, // 10 MB
            io_write_bytes: 5 * 1024 * 1024, // 5 MB
        });
        
        // SLAM service process
        processes.push(ProcessStats {
            pid: 1001,
            ppid: 1,
            name: "slam_service".to_string(),
            cmdline: "/usr/bin/slam_service".to_string(),
            state: ProcessState::Running,
            priority: ProcessPriority::RealTime,
            cpu_percent: 15.0,
            memory_bytes: 100 * 1024 * 1024, // 100 MB
            virtual_memory_bytes: 300 * 1024 * 1024, // 300 MB
            resident_memory_bytes: 100 * 1024 * 1024, // 100 MB
            thread_count: 6,
            fd_count: 25,
            start_time: 1621500010,
            cpu_time: 300.0,
            io_read_bytes: 20 * 1024 * 1024, // 20 MB
            io_write_bytes: 10 * 1024 * 1024, // 10 MB
        });
        
        // Display service process
        processes.push(ProcessStats {
            pid: 1002,
            ppid: 1,
            name: "display_service".to_string(),
            cmdline: "/usr/bin/display_service".to_string(),
            state: ProcessState::Running,
            priority: ProcessPriority::High,
            cpu_percent: 10.0,
            memory_bytes: 30 * 1024 * 1024, // 30 MB
            virtual_memory_bytes: 150 * 1024 * 1024, // 150 MB
            resident_memory_bytes: 30 * 1024 * 1024, // 30 MB
            thread_count: 3,
            fd_count: 20,
            start_time: 1621500020,
            cpu_time: 200.0,
            io_read_bytes: 5 * 1024 * 1024, // 5 MB
            io_write_bytes: 2 * 1024 * 1024, // 2 MB
        });
        
        // Audio service process
        processes.push(ProcessStats {
            pid: 1003,
            ppid: 1,
            name: "audio_service".to_string(),
            cmdline: "/usr/bin/audio_service".to_string(),
            state: ProcessState::Running,
            priority: ProcessPriority::High,
            cpu_percent: 8.0,
            memory_bytes: 25 * 1024 * 1024, // 25 MB
            virtual_memory_bytes: 120 * 1024 * 1024, // 120 MB
            resident_memory_bytes: 25 * 1024 * 1024, // 25 MB
            thread_count: 3,
            fd_count: 15,
            start_time: 1621500030,
            cpu_time: 150.0,
            io_read_bytes: 3 * 1024 * 1024, // 3 MB
            io_write_bytes: 1 * 1024 * 1024, // 1 MB
        });
        
        // Tracking service process
        processes.push(ProcessStats {
            pid: 1004,
            ppid: 1,
            name: "tracking_service".to_string(),
            cmdline: "/usr/bin/tracking_service".to_string(),
            state: ProcessState::Running,
            priority: ProcessPriority::RealTime,
            cpu_percent: 12.0,
            memory_bytes: 40 * 1024 * 1024, // 40 MB
            virtual_memory_bytes: 180 * 1024 * 1024, // 180 MB
            resident_memory_bytes: 40 * 1024 * 1024, // 40 MB
            thread_count: 5,
            fd_count: 22,
            start_time: 1621500040,
            cpu_time: 250.0,
            io_read_bytes: 8 * 1024 * 1024, // 8 MB
            io_write_bytes: 4 * 1024 * 1024, // 4 MB
        });
        
        // Background service process
        processes.push(ProcessStats {
            pid: 1005,
            ppid: 1,
            name: "background_service".to_string(),
            cmdline: "/usr/bin/background_service".to_string(),
            state: ProcessState::Sleeping,
            priority: ProcessPriority::Low,
            cpu_percent: 1.0,
            memory_bytes: 10 * 1024 * 1024, // 10 MB
            virtual_memory_bytes: 50 * 1024 * 1024, // 50 MB
            resident_memory_bytes: 10 * 1024 * 1024, // 10 MB
            thread_count: 2,
            fd_count: 10,
            start_time: 1621500050,
            cpu_time: 50.0,
            io_read_bytes: 1 * 1024 * 1024, // 1 MB
            io_write_bytes: 500 * 1024, // 500 KB
        });
        
        processes
    }
    
    /// Calculate I/O rates for a process.
    fn calculate_io_rates(&self, current: &ProcessStats, previous: &ProcessStats) -> (f64, f64) {
        // Calculate rates in bytes per second
        let read_rate = if current.io_read_bytes >= previous.io_read_bytes {
            (current.io_read_bytes - previous.io_read_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        let write_rate = if current.io_write_bytes >= previous.io_write_bytes {
            (current.io_write_bytes - previous.io_write_bytes) as f64 / self.interval.as_secs() as f64
        } else {
            0.0
        };
        
        (read_rate, write_rate)
    }
    
    /// Check if a process is critical.
    fn is_critical_process(&self, process_name: &str) -> bool {
        self.critical_processes.contains(&process_name.to_string())
    }
}

impl MetricsCollector for ProcessMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get current process statistics
        let current_stats = self.get_process_stats();
        
        // Get last statistics for rate calculation
        let mut last_stats = self.last_stats.lock().unwrap();
        
        // Process each process
        for process in &current_stats {
            // Process labels
            let mut labels = HashMap::new();
            labels.insert("pid".to_string(), process.pid.to_string());
            labels.insert("name".to_string(), process.name.clone());
            
            // Add critical label if this is a critical process
            if self.is_critical_process(&process.name) {
                labels.insert("critical".to_string(), "true".to_string());
            }
            
            // Process state metric
            metrics.push(Metric::new(
                "process.state",
                MetricType::State,
                MetricValue::String(process.state.to_string()),
                Some(labels.clone()),
                Some("Process state"),
                None,
            ));
            
            // Process priority metric
            metrics.push(Metric::new(
                "process.priority",
                MetricType::State,
                MetricValue::String(process.priority.to_string()),
                Some(labels.clone()),
                Some("Process priority"),
                None,
            ));
            
            // CPU usage metric
            metrics.push(Metric::new(
                "process.cpu.usage",
                MetricType::Gauge,
                MetricValue::Float(process.cpu_percent),
                Some(labels.clone()),
                Some("Process CPU usage"),
                Some("%"),
            ));
            
            // Memory usage metrics
            metrics.push(Metric::new(
                "process.memory.resident",
                MetricType::Gauge,
                MetricValue::Integer(process.resident_memory_bytes as i64),
                Some(labels.clone()),
                Some("Process resident memory usage"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                "process.memory.virtual",
                MetricType::Gauge,
                MetricValue::Integer(process.virtual_memory_bytes as i64),
                Some(labels.clone()),
                Some("Process virtual memory usage"),
                Some("bytes"),
            ));
            
            // Thread count metric
            metrics.push(Metric::new(
                "process.threads",
                MetricType::Gauge,
                MetricValue::Integer(process.thread_count as i64),
                Some(labels.clone()),
                Some("Process thread count"),
                None,
            ));
            
            // File descriptor count metric
            metrics.push(Metric::new(
                "process.fds",
                MetricType::Gauge,
                MetricValue::Integer(process.fd_count as i64),
                Some(labels.clone()),
                Some("Process file descriptor count"),
                None,
            ));
            
            // I/O metrics
            metrics.push(Metric::new(
                "process.io.read_bytes",
                MetricType::Counter,
                MetricValue::Integer(process.io_read_bytes as i64),
                Some(labels.clone()),
                Some("Process total bytes read"),
                Some("bytes"),
            ));
            
            metrics.push(Metric::new(
                "process.io.write_bytes",
                MetricType::Counter,
                MetricValue::Integer(process.io_write_bytes as i64),
                Some(labels.clone()),
                Some("Process total bytes written"),
                Some("bytes"),
            ));
            
            // Calculate I/O rates if we have previous stats
            if let Some(previous) = last_stats.get(&process.pid) {
                let (read_rate, write_rate) = self.calculate_io_rates(process, previous);
                
                // I/O rate metrics
                metrics.push(Metric::new(
                    "process.io.read_rate",
                    MetricType::Gauge,
                    MetricValue::Float(read_rate),
                    Some(labels.clone()),
                    Some("Process read rate"),
                    Some("bytes/s"),
                ));
                
                metrics.push(Metric::new(
                    "process.io.write_rate",
                    MetricType::Gauge,
                    MetricValue::Float(write_rate),
                    Some(labels.clone()),
                    Some("Process write rate"),
                    Some("bytes/s"),
                ));
            }
            
            // Update last stats
            last_stats.insert(process.pid, process.clone());
        }
        
        // Add system-wide process metrics
        let mut system_labels = HashMap::new();
        system_labels.insert("scope".to_string(), "system".to_string());
        
        // Total process count
        metrics.push(Metric::new(
            "process.count.total",
            MetricType::Gauge,
            MetricValue::Integer(current_stats.len() as i64),
            Some(system_labels.clone()),
            Some("Total process count"),
            None,
        ));
        
        // Process count by state
        let mut state_counts = HashMap::new();
        for process in &current_stats {
            *state_counts.entry(process.state).or_insert(0) += 1;
        }
        
        for (state, count) in state_counts {
            let mut state_labels = system_labels.clone();
            state_labels.insert("state".to_string(), state.to_string());
            
            metrics.push(Metric::new(
                "process.count.by_state",
                MetricType::Gauge,
                MetricValue::Integer(count),
                Some(state_labels),
                Some(&format!("Process count in {} state", state)),
                None,
            ));
        }
        
        // Critical process status
        for process_name in &self.critical_processes {
            let process = current_stats.iter().find(|p| &p.name == process_name);
            
            let mut critical_labels = HashMap::new();
            critical_labels.insert("name".to_string(), process_name.clone());
            
            if let Some(process) = process {
                // Process is running
                metrics.push(Metric::new(
                    "process.critical.running",
                    MetricType::State,
                    MetricValue::Boolean(true),
                    Some(critical_labels.clone()),
                    Some("Critical process running status"),
                    None,
                ));
                
                // Add PID
                critical_labels.insert("pid".to_string(), process.pid.to_string());
                
                // CPU usage
                metrics.push(Metric::new(
                    "process.critical.cpu",
                    MetricType::Gauge,
                    MetricValue::Float(process.cpu_percent),
                    Some(critical_labels.clone()),
                    Some("Critical process CPU usage"),
                    Some("%"),
                ));
                
                // Memory usage
                metrics.push(Metric::new(
                    "process.critical.memory",
                    MetricType::Gauge,
                    MetricValue::Integer(process.resident_memory_bytes as i64),
                    Some(critical_labels),
                    Some("Critical process memory usage"),
                    Some("bytes"),
                ));
            } else {
                // Process is not running
                metrics.push(Metric::new(
                    "process.critical.running",
                    MetricType::State,
                    MetricValue::Boolean(false),
                    Some(critical_labels),
                    Some("Critical process running status"),
                    None,
                ));
            }
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Thread statistics.
#[derive(Debug, Clone)]
pub struct ThreadStats {
    /// Process ID
    pub pid: u32,
    
    /// Thread ID
    pub tid: u32,
    
    /// Thread name
    pub name: String,
    
    /// Thread state
    pub state: ProcessState,
    
    /// CPU usage percentage
    pub cpu_percent: f64,
    
    /// CPU time in seconds
    pub cpu_time: f64,
}

/// Thread metrics collector.
#[derive(Debug)]
pub struct ThreadMetricsCollector {
    /// Collector name
    name: String,
    
    /// Collection interval
    interval: Duration,
    
    /// Processes to monitor threads for
    monitored_processes: Vec<String>,
}

impl ThreadMetricsCollector {
    /// Create a new thread metrics collector.
    pub fn new(interval_secs: u64) -> Self {
        // Default processes to monitor threads for
        let monitored_processes = vec![
            "slam_service".to_string(),
            "tracking_service".to_string(),
        ];
        
        Self {
            name: "thread".to_string(),
            interval: Duration::from_secs(interval_secs),
            monitored_processes,
        }
    }
    
    /// Add a process to monitor threads for.
    pub fn add_monitored_process(&mut self, process_name: String) {
        if !self.monitored_processes.contains(&process_name) {
            self.monitored_processes.push(process_name);
        }
    }
    
    /// Remove a process from thread monitoring.
    pub fn remove_monitored_process(&mut self, process_name: &str) {
        self.monitored_processes.retain(|name| name != process_name);
    }
    
    /// Get the list of monitored processes.
    pub fn get_monitored_processes(&self) -> Vec<String> {
        self.monitored_processes.clone()
    }
    
    /// Get thread statistics.
    fn get_thread_stats(&self) -> Vec<ThreadStats> {
        // In a real implementation, this would read from /proc/<pid>/task/<tid>/...
        // For now, we'll simulate some values
        
        let mut threads = Vec::new();
        
        // SLAM service threads
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1001,
            name: "slam_main".to_string(),
            state: ProcessState::Running,
            cpu_percent: 5.0,
            cpu_time: 100.0,
        });
        
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1006,
            name: "slam_tracking".to_string(),
            state: ProcessState::Running,
            cpu_percent: 8.0,
            cpu_time: 150.0,
        });
        
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1007,
            name: "slam_mapping".to_string(),
            state: ProcessState::Running,
            cpu_percent: 7.0,
            cpu_time: 130.0,
        });
        
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1008,
            name: "slam_loop_closure".to_string(),
            state: ProcessState::Sleeping,
            cpu_percent: 0.5,
            cpu_time: 50.0,
        });
        
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1009,
            name: "slam_visualization".to_string(),
            state: ProcessState::Sleeping,
            cpu_percent: 0.2,
            cpu_time: 30.0,
        });
        
        threads.push(ThreadStats {
            pid: 1001,
            tid: 1010,
            name: "slam_io".to_string(),
            state: ProcessState::Waiting,
            cpu_percent: 0.1,
            cpu_time: 20.0,
        });
        
        // Tracking service threads
        threads.push(ThreadStats {
            pid: 1004,
            tid: 1004,
            name: "tracking_main".to_string(),
            state: ProcessState::Running,
            cpu_percent: 3.0,
            cpu_time: 80.0,
        });
        
        threads.push(ThreadStats {
            pid: 1004,
            tid: 1011,
            name: "tracking_imu".to_string(),
            state: ProcessState::Running,
            cpu_percent: 4.0,
            cpu_time: 90.0,
        });
        
        threads.push(ThreadStats {
            pid: 1004,
            tid: 1012,
            name: "tracking_camera".to_string(),
            state: ProcessState::Running,
            cpu_percent: 5.0,
            cpu_time: 100.0,
        });
        
        threads.push(ThreadStats {
            pid: 1004,
            tid: 1013,
            name: "tracking_fusion".to_string(),
            state: ProcessState::Running,
            cpu_percent: 3.5,
            cpu_time: 85.0,
        });
        
        threads.push(ThreadStats {
            pid: 1004,
            tid: 1014,
            name: "tracking_io".to_string(),
            state: ProcessState::Waiting,
            cpu_percent: 0.2,
            cpu_time: 25.0,
        });
        
        threads
    }
    
    /// Check if a process should be monitored for threads.
    fn should_monitor_process(&self, process_name: &str) -> bool {
        self.monitored_processes.contains(&process_name.to_string())
    }
}

impl MetricsCollector for ThreadMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        
        // Get thread statistics
        let threads = self.get_thread_stats();
        
        // Process each thread
        for thread in &threads {
            // Get process name (in a real implementation, this would be looked up)
            let process_name = match thread.pid {
                1001 => "slam_service",
                1004 => "tracking_service",
                _ => "unknown",
            };
            
            // Skip threads for processes we're not monitoring
            if !self.should_monitor_process(process_name) {
                continue;
            }
            
            // Thread labels
            let mut labels = HashMap::new();
            labels.insert("pid".to_string(), thread.pid.to_string());
            labels.insert("tid".to_string(), thread.tid.to_string());
            labels.insert("name".to_string(), thread.name.clone());
            labels.insert("process".to_string(), process_name.to_string());
            
            // Thread state metric
            metrics.push(Metric::new(
                "thread.state",
                MetricType::State,
                MetricValue::String(thread.state.to_string()),
                Some(labels.clone()),
                Some("Thread state"),
                None,
            ));
            
            // CPU usage metric
            metrics.push(Metric::new(
                "thread.cpu.usage",
                MetricType::Gauge,
                MetricValue::Float(thread.cpu_percent),
                Some(labels.clone()),
                Some("Thread CPU usage"),
                Some("%"),
            ));
            
            // CPU time metric
            metrics.push(Metric::new(
                "thread.cpu.time",
                MetricType::Counter,
                MetricValue::Float(thread.cpu_time),
                Some(labels),
                Some("Thread CPU time"),
                Some("s"),
            ));
        }
        
        // Add process-level thread metrics
        let process_thread_counts = threads.iter()
            .map(|t| t.pid)
            .fold(HashMap::new(), |mut acc, pid| {
                *acc.entry(pid).or_insert(0) += 1;
                acc
            });
        
        for (pid, count) in process_thread_counts {
            // Get process name (in a real implementation, this would be looked up)
            let process_name = match pid {
                1001 => "slam_service",
                1004 => "tracking_service",
                _ => "unknown",
            };
            
            // Skip processes we're not monitoring
            if !self.should_monitor_process(process_name) {
                continue;
            }
            
            let mut process_labels = HashMap::new();
            process_labels.insert("pid".to_string(), pid.to_string());
            process_labels.insert("process".to_string(), process_name.to_string());
            
            metrics.push(Metric::new(
                "thread.count",
                MetricType::Gauge,
                MetricValue::Integer(count),
                Some(process_labels),
                Some("Thread count for process"),
                None,
            ));
        }
        
        // Add thread state counts per process
        let process_thread_states = threads.iter()
            .fold(HashMap::new(), |mut acc, thread| {
                let key = (thread.pid, thread.state);
                *acc.entry(key).or_insert(0) += 1;
                acc
            });
        
        for ((pid, state), count) in process_thread_states {
            // Get process name (in a real implementation, this would be looked up)
            let process_name = match pid {
                1001 => "slam_service",
                1004 => "tracking_service",
                _ => "unknown",
            };
            
            // Skip processes we're not monitoring
            if !self.should_monitor_process(process_name) {
                continue;
            }
            
            let mut state_labels = HashMap::new();
            state_labels.insert("pid".to_string(), pid.to_string());
            state_labels.insert("process".to_string(), process_name.to_string());
            state_labels.insert("state".to_string(), state.to_string());
            
            metrics.push(Metric::new(
                "thread.count.by_state",
                MetricType::Gauge,
                MetricValue::Integer(count),
                Some(state_labels),
                Some(&format!("Thread count in {} state for process", state)),
                None,
            ));
        }
        
        metrics
    }
    
    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Process monitor.
#[derive(Debug)]
pub struct ProcessMonitor {
    /// Process metrics collector
    process_collector: Arc<ProcessMetricsCollector>,
    
    /// Thread metrics collector
    thread_collector: Arc<ThreadMetricsCollector>,
}

impl ProcessMonitor {
    /// Create a new process monitor.
    pub fn new() -> Self {
        let process_collector = Arc::new(ProcessMetricsCollector::new(5));
        let thread_collector = Arc::new(ThreadMetricsCollector::new(10));
        
        Self {
            process_collector,
            thread_collector,
        }
    }
    
    /// Get the process metrics collector.
    pub fn process_collector(&self) -> Arc<ProcessMetricsCollector> {
        self.process_collector.clone()
    }
    
    /// Get the thread metrics collector.
    pub fn thread_collector(&self) -> Arc<ThreadMetricsCollector> {
        self.thread_collector.clone()
    }
    
    /// Get all collectors.
    pub fn collectors(&self) -> Vec<Arc<dyn MetricsCollector>> {
        vec![
            self.process_collector.clone() as Arc<dyn MetricsCollector>,
            self.thread_collector.clone() as Arc<dyn MetricsCollector>,
        ]
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_state() {
        assert_eq!(ProcessState::Running.as_str(), "running");
        assert_eq!(ProcessState::Sleeping.as_str(), "sleeping");
        assert_eq!(ProcessState::Waiting.as_str(), "waiting");
        assert_eq!(ProcessState::Stopped.as_str(), "stopped");
        assert_eq!(ProcessState::Zombie.as_str(), "zombie");
        
        assert_eq!(ProcessState::from_str("running"), Some(ProcessState::Running));
        assert_eq!(ProcessState::from_str("sleeping"), Some(ProcessState::Sleeping));
        assert_eq!(ProcessState::from_str("waiting"), Some(ProcessState::Waiting));
        assert_eq!(ProcessState::from_str("stopped"), Some(ProcessState::Stopped));
        assert_eq!(ProcessState::from_str("zombie"), Some(ProcessState::Zombie));
        assert_eq!(ProcessState::from_str("invalid"), None);
        
        assert_eq!(ProcessState::Running.to_string(), "running");
        assert_eq!(ProcessState::Sleeping.to_string(), "sleeping");
        assert_eq!(ProcessState::Waiting.to_string(), "waiting");
        assert_eq!(ProcessState::Stopped.to_string(), "stopped");
        assert_eq!(ProcessState::Zombie.to_string(), "zombie");
    }
    
    #[test]
    fn test_process_priority() {
        assert_eq!(ProcessPriority::RealTime.as_str(), "real_time");
        assert_eq!(ProcessPriority::High.as_str(), "high");
        assert_eq!(ProcessPriority::Normal.as_str(), "normal");
        assert_eq!(ProcessPriority::Low.as_str(), "low");
        assert_eq!(ProcessPriority::Idle.as_str(), "idle");
        
        assert_eq!(ProcessPriority::from_str("real_time"), Some(ProcessPriority::RealTime));
        assert_eq!(ProcessPriority::from_str("high"), Some(ProcessPriority::High));
        assert_eq!(ProcessPriority::from_str("normal"), Some(ProcessPriority::Normal));
        assert_eq!(ProcessPriority::from_str("low"), Some(ProcessPriority::Low));
        assert_eq!(ProcessPriority::from_str("idle"), Some(ProcessPriority::Idle));
        assert_eq!(ProcessPriority::from_str("invalid"), None);
        
        assert_eq!(ProcessPriority::RealTime.to_string(), "real_time");
        assert_eq!(ProcessPriority::High.to_string(), "high");
        assert_eq!(ProcessPriority::Normal.to_string(), "normal");
        assert_eq!(ProcessPriority::Low.to_string(), "low");
        assert_eq!(ProcessPriority::Idle.to_string(), "idle");
    }
    
    #[test]
    fn test_process_metrics_collector() {
        let collector = ProcessMetricsCollector::new(5);
        
        // Check critical processes
        assert!(collector.is_critical_process("system_ui"));
        assert!(collector.is_critical_process("slam_service"));
        assert!(!collector.is_critical_process("unknown_process"));
        
        // First collection
        let metrics = collector.collect();
        
        // Check process metrics
        assert!(metrics.iter().any(|m| m.name == "process.state"));
        assert!(metrics.iter().any(|m| m.name == "process.priority"));
        assert!(metrics.iter().any(|m| m.name == "process.cpu.usage"));
        assert!(metrics.iter().any(|m| m.name == "process.memory.resident"));
        assert!(metrics.iter().any(|m| m.name == "process.memory.virtual"));
        
        // Check system-wide metrics
        assert!(metrics.iter().any(|m| m.name == "process.count.total"));
        assert!(metrics.iter().any(|m| m.name == "process.count.by_state"));
        
        // Check critical process metrics
        assert!(metrics.iter().any(|m| m.name == "process.critical.running"));
        assert!(metrics.iter().any(|m| m.name == "process.critical.cpu"));
        assert!(metrics.iter().any(|m| m.name == "process.critical.memory"));
        
        // Second collection should include I/O rate metrics
        let metrics = collector.collect();
        assert!(metrics.iter().any(|m| m.name == "process.io.read_rate"));
        assert!(metrics.iter().any(|m| m.name == "process.io.write_rate"));
    }
    
    #[test]
    fn test_thread_metrics_collector() {
        let collector = ThreadMetricsCollector::new(10);
        
        // Check monitored processes
        assert!(collector.should_monitor_process("slam_service"));
        assert!(collector.should_monitor_process("tracking_service"));
        assert!(!collector.should_monitor_process("unknown_process"));
        
        // Collect metrics
        let metrics = collector.collect();
        
        // Check thread metrics
        assert!(metrics.iter().any(|m| m.name == "thread.state"));
        assert!(metrics.iter().any(|m| m.name == "thread.cpu.usage"));
        assert!(metrics.iter().any(|m| m.name == "thread.cpu.time"));
        
        // Check process-level thread metrics
        assert!(metrics.iter().any(|m| m.name == "thread.count"));
        assert!(metrics.iter().any(|m| m.name == "thread.count.by_state"));
        
        // Check that we have metrics for both monitored processes
        let slam_metrics = metrics.iter().filter(|m| {
            if let Some(labels) = &m.labels {
                if let Some(process) = labels.get("process") {
                    return process == "slam_service";
                }
            }
            false
        }).count();
        
        let tracking_metrics = metrics.iter().filter(|m| {
            if let Some(labels) = &m.labels {
                if let Some(process) = labels.get("process") {
                    return process == "tracking_service";
                }
            }
            false
        }).count();
        
        assert!(slam_metrics > 0);
        assert!(tracking_metrics > 0);
    }
    
    #[test]
    fn test_process_monitor() {
        let monitor = ProcessMonitor::new();
        
        // Check collectors
        assert_eq!(monitor.collectors().len(), 2);
        
        // Check collector access
        assert_eq!(monitor.process_collector().name(), "process");
        assert_eq!(monitor.thread_collector().name(), "thread");
    }
}
