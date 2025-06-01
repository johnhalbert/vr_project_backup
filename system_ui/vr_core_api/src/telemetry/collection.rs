//! Telemetry collection module for the VR headset.
//!
//! This module provides functionality for collecting various types of telemetry
//! data from the system, hardware, applications, and other sources.

use std::collections::HashMap;
use std::process::Command;
use std::io::{self, Read};
use std::fs::{self, File};
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use tokio::sync::mpsc::Sender;
use sysinfo::{System, SystemExt, ProcessorExt, DiskExt, NetworkExt};
use uuid::Uuid;
use chrono::Utc;

use super::{TelemetryDataPoint, TelemetryCategory, TelemetryValue};

/// Collect system telemetry data.
///
/// This function collects telemetry data about the system, including CPU usage,
/// memory usage, disk usage, and other system metrics.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_system_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Initialize system information
    let mut system = System::new_all();
    system.refresh_all();
    
    // Collect CPU usage
    let global_cpu_usage = system.global_processor_info().cpu_usage();
    submit_telemetry_point(
        tx,
        TelemetryCategory::System,
        "cpu_usage_percent",
        TelemetryValue::Float(global_cpu_usage as f64),
        HashMap::new(),
    ).await?;
    
    // Collect per-core CPU usage
    for (i, cpu) in system.processors().iter().enumerate() {
        let cpu_usage = cpu.cpu_usage();
        let mut metadata = HashMap::new();
        metadata.insert("core".to_string(), i.to_string());
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "cpu_core_usage_percent",
            TelemetryValue::Float(cpu_usage as f64),
            metadata,
        ).await?;
    }
    
    // Collect memory usage
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
    
    submit_telemetry_point(
        tx,
        TelemetryCategory::System,
        "memory_total_kb",
        TelemetryValue::Integer(total_memory as i64),
        HashMap::new(),
    ).await?;
    
    submit_telemetry_point(
        tx,
        TelemetryCategory::System,
        "memory_used_kb",
        TelemetryValue::Integer(used_memory as i64),
        HashMap::new(),
    ).await?;
    
    submit_telemetry_point(
        tx,
        TelemetryCategory::System,
        "memory_usage_percent",
        TelemetryValue::Float(memory_usage_percent),
        HashMap::new(),
    ).await?;
    
    // Collect disk usage
    for disk in system.disks() {
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        let usage_percent = (used_space as f64 / total_space as f64) * 100.0;
        
        let mut metadata = HashMap::new();
        metadata.insert("mount_point".to_string(), disk.mount_point().to_string_lossy().to_string());
        metadata.insert("name".to_string(), disk.name().to_string_lossy().to_string());
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "disk_total_bytes",
            TelemetryValue::Integer(total_space as i64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "disk_available_bytes",
            TelemetryValue::Integer(available_space as i64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "disk_usage_percent",
            TelemetryValue::Float(usage_percent),
            metadata,
        ).await?;
    }
    
    // Collect system load average
    if let Ok(loadavg) = read_proc_loadavg() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "load_avg_1min",
            TelemetryValue::Float(loadavg.0),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "load_avg_5min",
            TelemetryValue::Float(loadavg.1),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "load_avg_15min",
            TelemetryValue::Float(loadavg.2),
            HashMap::new(),
        ).await?;
    }
    
    // Collect system uptime
    if let Ok(uptime) = read_proc_uptime() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::System,
            "uptime_seconds",
            TelemetryValue::Float(uptime),
            HashMap::new(),
        ).await?;
    }
    
    // Collect process count
    let process_count = system.processes().len();
    submit_telemetry_point(
        tx,
        TelemetryCategory::System,
        "process_count",
        TelemetryValue::Integer(process_count as i64),
        HashMap::new(),
    ).await?;
    
    Ok(())
}

/// Collect hardware telemetry data.
///
/// This function collects telemetry data about the hardware, including
/// temperatures, fan speeds, battery status, and other hardware metrics.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_hardware_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Collect CPU temperature
    if let Ok(temp) = read_cpu_temperature() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::Hardware,
            "cpu_temperature_celsius",
            TelemetryValue::Float(temp),
            HashMap::new(),
        ).await?;
    }
    
    // Collect GPU temperature
    if let Ok(temp) = read_gpu_temperature() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::Hardware,
            "gpu_temperature_celsius",
            TelemetryValue::Float(temp),
            HashMap::new(),
        ).await?;
    }
    
    // Collect fan speeds
    if let Ok(speeds) = read_fan_speeds() {
        for (i, speed) in speeds.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("fan_index".to_string(), i.to_string());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "fan_speed_rpm",
                TelemetryValue::Integer(*speed as i64),
                metadata,
            ).await?;
        }
    }
    
    // Collect battery status
    if let Ok(battery) = read_battery_status() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::Hardware,
            "battery_percent",
            TelemetryValue::Float(battery.percent),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Hardware,
            "battery_charging",
            TelemetryValue::Boolean(battery.charging),
            HashMap::new(),
        ).await?;
        
        if let Some(time_remaining) = battery.time_remaining {
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "battery_time_remaining_minutes",
                TelemetryValue::Float(time_remaining),
                HashMap::new(),
            ).await?;
        }
    }
    
    // Collect display information
    if let Ok(displays) = read_display_info() {
        for (i, display) in displays.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("display_index".to_string(), i.to_string());
            metadata.insert("display_name".to_string(), display.name.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "display_resolution_width",
                TelemetryValue::Integer(display.width as i64),
                metadata.clone(),
            ).await?;
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "display_resolution_height",
                TelemetryValue::Integer(display.height as i64),
                metadata.clone(),
            ).await?;
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "display_refresh_rate",
                TelemetryValue::Float(display.refresh_rate),
                metadata,
            ).await?;
        }
    }
    
    // Collect sensor information
    if let Ok(sensors) = read_sensor_status() {
        for sensor in &sensors {
            let mut metadata = HashMap::new();
            metadata.insert("sensor_name".to_string(), sensor.name.clone());
            metadata.insert("sensor_type".to_string(), sensor.sensor_type.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Hardware,
                "sensor_connected",
                TelemetryValue::Boolean(sensor.connected),
                metadata.clone(),
            ).await?;
            
            if sensor.connected {
                submit_telemetry_point(
                    tx,
                    TelemetryCategory::Hardware,
                    "sensor_error_rate",
                    TelemetryValue::Float(sensor.error_rate),
                    metadata,
                ).await?;
            }
        }
    }
    
    Ok(())
}

/// Collect application telemetry data.
///
/// This function collects telemetry data about applications, including
/// usage statistics, performance metrics, and other application-specific data.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_application_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Initialize system information
    let mut system = System::new_all();
    system.refresh_all();
    
    // Collect VR application processes
    let vr_processes = system.processes().iter()
        .filter(|(_, process)| {
            let name = process.name().to_lowercase();
            name.contains("vr") || name.contains("virtual") || name.contains("headset")
        })
        .collect::<Vec<_>>();
    
    // Collect process metrics
    for (pid, process) in vr_processes {
        let name = process.name();
        let cpu_usage = process.cpu_usage();
        let memory_usage = process.memory();
        
        let mut metadata = HashMap::new();
        metadata.insert("process_name".to_string(), name.to_string());
        metadata.insert("pid".to_string(), pid.to_string());
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "app_cpu_usage_percent",
            TelemetryValue::Float(cpu_usage as f64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "app_memory_usage_kb",
            TelemetryValue::Integer(memory_usage as i64),
            metadata,
        ).await?;
    }
    
    // Collect application usage statistics
    if let Ok(usage) = read_application_usage() {
        for (app_name, stats) in &usage {
            let mut metadata = HashMap::new();
            metadata.insert("app_name".to_string(), app_name.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Application,
                "app_usage_time_minutes",
                TelemetryValue::Float(stats.usage_time_minutes),
                metadata.clone(),
            ).await?;
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Application,
                "app_launch_count",
                TelemetryValue::Integer(stats.launch_count as i64),
                metadata.clone(),
            ).await?;
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Application,
                "app_crash_count",
                TelemetryValue::Integer(stats.crash_count as i64),
                metadata,
            ).await?;
        }
    }
    
    // Collect rendering performance metrics
    if let Ok(metrics) = read_rendering_metrics() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "render_fps",
            TelemetryValue::Float(metrics.fps),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "render_frame_time_ms",
            TelemetryValue::Float(metrics.frame_time_ms),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "render_gpu_time_ms",
            TelemetryValue::Float(metrics.gpu_time_ms),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Application,
            "render_cpu_time_ms",
            TelemetryValue::Float(metrics.cpu_time_ms),
            HashMap::new(),
        ).await?;
    }
    
    Ok(())
}

/// Collect network telemetry data.
///
/// This function collects telemetry data about the network, including
/// connectivity, bandwidth usage, latency, and other network metrics.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_network_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Initialize system information
    let mut system = System::new_all();
    system.refresh_all();
    
    // Collect network interface statistics
    for (interface_name, data) in system.networks() {
        let received_bytes = data.received();
        let transmitted_bytes = data.transmitted();
        let total_bytes = received_bytes + transmitted_bytes;
        
        let mut metadata = HashMap::new();
        metadata.insert("interface".to_string(), interface_name.to_string());
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "network_received_bytes",
            TelemetryValue::Integer(received_bytes as i64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "network_transmitted_bytes",
            TelemetryValue::Integer(transmitted_bytes as i64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "network_total_bytes",
            TelemetryValue::Integer(total_bytes as i64),
            metadata,
        ).await?;
    }
    
    // Collect WiFi signal strength
    if let Ok(signal) = read_wifi_signal_strength() {
        let mut metadata = HashMap::new();
        metadata.insert("ssid".to_string(), signal.ssid);
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "wifi_signal_strength_dbm",
            TelemetryValue::Integer(signal.strength_dbm as i64),
            metadata.clone(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "wifi_signal_quality_percent",
            TelemetryValue::Integer(signal.quality_percent as i64),
            metadata,
        ).await?;
    }
    
    // Collect network latency
    if let Ok(latency) = measure_network_latency() {
        for (host, ping_ms) in &latency {
            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), host.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Network,
                "network_latency_ms",
                TelemetryValue::Float(*ping_ms),
                metadata,
            ).await?;
        }
    }
    
    // Collect connection status
    if let Ok(status) = check_internet_connection() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::Network,
            "internet_connected",
            TelemetryValue::Boolean(status.connected),
            HashMap::new(),
        ).await?;
        
        if status.connected {
            submit_telemetry_point(
                tx,
                TelemetryCategory::Network,
                "internet_connection_type",
                TelemetryValue::String(status.connection_type),
                HashMap::new(),
            ).await?;
        }
    }
    
    Ok(())
}

/// Collect error telemetry data.
///
/// This function collects telemetry data about errors, including
/// crashes, exceptions, and other error conditions.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_error_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Collect crash reports
    if let Ok(crashes) = read_crash_reports() {
        for crash in &crashes {
            let mut metadata = HashMap::new();
            metadata.insert("app_name".to_string(), crash.app_name.clone());
            metadata.insert("crash_type".to_string(), crash.crash_type.clone());
            metadata.insert("timestamp".to_string(), crash.timestamp.to_rfc3339());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Error,
                "app_crash",
                TelemetryValue::String(crash.error_message.clone()),
                metadata,
            ).await?;
        }
    }
    
    // Collect system errors
    if let Ok(errors) = read_system_errors() {
        for error in &errors {
            let mut metadata = HashMap::new();
            metadata.insert("component".to_string(), error.component.clone());
            metadata.insert("error_code".to_string(), error.error_code.to_string());
            metadata.insert("timestamp".to_string(), error.timestamp.to_rfc3339());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Error,
                "system_error",
                TelemetryValue::String(error.error_message.clone()),
                metadata,
            ).await?;
        }
    }
    
    // Collect hardware errors
    if let Ok(errors) = read_hardware_errors() {
        for error in &errors {
            let mut metadata = HashMap::new();
            metadata.insert("device".to_string(), error.device.clone());
            metadata.insert("error_code".to_string(), error.error_code.to_string());
            metadata.insert("timestamp".to_string(), error.timestamp.to_rfc3339());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::Error,
                "hardware_error",
                TelemetryValue::String(error.error_message.clone()),
                metadata,
            ).await?;
        }
    }
    
    Ok(())
}

/// Collect user interaction telemetry data.
///
/// This function collects telemetry data about user interactions, including
/// UI usage, feature usage, and other user behavior metrics.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_user_interaction_telemetry(tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Collect UI interaction statistics
    if let Ok(stats) = read_ui_interaction_stats() {
        for (ui_element, count) in &stats.interaction_counts {
            let mut metadata = HashMap::new();
            metadata.insert("ui_element".to_string(), ui_element.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::UserInteraction,
                "ui_interaction_count",
                TelemetryValue::Integer(*count as i64),
                metadata,
            ).await?;
        }
        
        for (screen, time) in &stats.screen_times {
            let mut metadata = HashMap::new();
            metadata.insert("screen".to_string(), screen.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::UserInteraction,
                "screen_time_seconds",
                TelemetryValue::Float(*time),
                metadata,
            ).await?;
        }
    }
    
    // Collect feature usage statistics
    if let Ok(stats) = read_feature_usage_stats() {
        for (feature, count) in &stats.usage_counts {
            let mut metadata = HashMap::new();
            metadata.insert("feature".to_string(), feature.clone());
            
            submit_telemetry_point(
                tx,
                TelemetryCategory::UserInteraction,
                "feature_usage_count",
                TelemetryValue::Integer(*count as i64),
                metadata,
            ).await?;
        }
    }
    
    // Collect session statistics
    if let Ok(stats) = read_session_stats() {
        submit_telemetry_point(
            tx,
            TelemetryCategory::UserInteraction,
            "session_count",
            TelemetryValue::Integer(stats.session_count as i64),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::UserInteraction,
            "average_session_duration_minutes",
            TelemetryValue::Float(stats.average_session_duration_minutes),
            HashMap::new(),
        ).await?;
        
        submit_telemetry_point(
            tx,
            TelemetryCategory::UserInteraction,
            "total_usage_time_minutes",
            TelemetryValue::Float(stats.total_usage_time_minutes),
            HashMap::new(),
        ).await?;
    }
    
    Ok(())
}

/// Collect custom telemetry data.
///
/// This function collects telemetry data for a custom category.
///
/// # Arguments
///
/// * `category_name` - Name of the custom category
/// * `tx` - Channel for sending telemetry data points
///
/// # Returns
///
/// `Ok(())` if telemetry was collected successfully.
pub async fn collect_custom_telemetry(category_name: &str, tx: &Sender<TelemetryDataPoint>) -> Result<()> {
    // Custom telemetry collection would be implemented by the application
    // This is just a placeholder
    
    let mut metadata = HashMap::new();
    metadata.insert("custom_category".to_string(), category_name.to_string());
    
    submit_telemetry_point(
        tx,
        TelemetryCategory::Custom(category_name.to_string()),
        "custom_metric",
        TelemetryValue::String("Custom telemetry data".to_string()),
        metadata,
    ).await?;
    
    Ok(())
}

/// Submit a telemetry data point.
///
/// # Arguments
///
/// * `tx` - Channel for sending telemetry data points
/// * `category` - Category of the telemetry data
/// * `name` - Name of the metric
/// * `value` - Value of the metric
/// * `metadata` - Additional metadata
///
/// # Returns
///
/// `Ok(())` if the data point was submitted successfully.
async fn submit_telemetry_point(
    tx: &Sender<TelemetryDataPoint>,
    category: TelemetryCategory,
    name: &str,
    value: TelemetryValue,
    metadata: HashMap<String, String>,
) -> Result<()> {
    let data_point = TelemetryDataPoint {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        category,
        name: name.to_string(),
        value,
        metadata,
    };
    
    tx.send(data_point).await
        .context("Failed to send telemetry data point")?;
    
    Ok(())
}

/// Read system load average from /proc/loadavg.
///
/// # Returns
///
/// A tuple of (1min, 5min, 15min) load averages.
fn read_proc_loadavg() -> Result<(f64, f64, f64)> {
    let mut file = File::open("/proc/loadavg")
        .context("Failed to open /proc/loadavg")?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read /proc/loadavg")?;
    
    let parts: Vec<&str> = contents.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(anyhow!("Invalid format in /proc/loadavg"));
    }
    
    let load1 = parts[0].parse::<f64>()
        .context("Failed to parse 1min load average")?;
    let load5 = parts[1].parse::<f64>()
        .context("Failed to parse 5min load average")?;
    let load15 = parts[2].parse::<f64>()
        .context("Failed to parse 15min load average")?;
    
    Ok((load1, load5, load15))
}

/// Read system uptime from /proc/uptime.
///
/// # Returns
///
/// System uptime in seconds.
fn read_proc_uptime() -> Result<f64> {
    let mut file = File::open("/proc/uptime")
        .context("Failed to open /proc/uptime")?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read /proc/uptime")?;
    
    let parts: Vec<&str> = contents.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Invalid format in /proc/uptime"));
    }
    
    let uptime = parts[0].parse::<f64>()
        .context("Failed to parse uptime")?;
    
    Ok(uptime)
}

/// Read CPU temperature.
///
/// # Returns
///
/// CPU temperature in Celsius.
fn read_cpu_temperature() -> Result<f64> {
    // This is a simplified implementation that may not work on all systems
    // A more robust implementation would check multiple possible locations
    
    let temp_paths = [
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/devices/platform/coretemp.0/temp1_input",
        "/sys/bus/acpi/devices/LNXTHERM:00/thermal_zone/temp",
    ];
    
    for path in &temp_paths {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(temp) = contents.trim().parse::<i32>() {
                    // Most systems report temperature in millidegrees Celsius
                    return Ok(temp as f64 / 1000.0);
                }
            }
        }
    }
    
    // If we couldn't read from any of the standard paths, try using lm-sensors
    let output = Command::new("sensors")
        .output()
        .context("Failed to execute sensors command")?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for lines like "Core 0: +45.0°C"
        for line in output_str.lines() {
            if line.contains("Core") && line.contains("°C") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let temp_part = parts[1].trim();
                    let temp_str: String = temp_part.chars()
                        .take_while(|c| c.is_digit(10) || *c == '.' || *c == '+' || *c == '-')
                        .collect();
                    
                    if let Ok(temp) = temp_str.parse::<f64>() {
                        return Ok(temp);
                    }
                }
            }
        }
    }
    
    Err(anyhow!("Could not read CPU temperature"))
}

/// Read GPU temperature.
///
/// # Returns
///
/// GPU temperature in Celsius.
fn read_gpu_temperature() -> Result<f64> {
    // Try using nvidia-smi for NVIDIA GPUs
    let nvidia_output = Command::new("nvidia-smi")
        .args(&["--query-gpu=temperature.gpu", "--format=csv,noheader"])
        .output();
    
    if let Ok(output) = nvidia_output {
        if output.status.success() {
            let temp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(temp) = temp_str.parse::<f64>() {
                return Ok(temp);
            }
        }
    }
    
    // Try using rocm-smi for AMD GPUs
    let amd_output = Command::new("rocm-smi")
        .args(&["--showtemp"])
        .output();
    
    if let Ok(output) = amd_output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Temperature") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(temp) = parts[parts.len() - 2].parse::<f64>() {
                            return Ok(temp);
                        }
                    }
                }
            }
        }
    }
    
    // Try using lm-sensors as a fallback
    let sensors_output = Command::new("sensors")
        .output();
    
    if let Ok(output) = sensors_output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Look for lines containing "GPU" and "°C"
            for line in output_str.lines() {
                if (line.contains("GPU") || line.contains("gpu")) && line.contains("°C") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let temp_part = parts[1].trim();
                        let temp_str: String = temp_part.chars()
                            .take_while(|c| c.is_digit(10) || *c == '.' || *c == '+' || *c == '-')
                            .collect();
                        
                        if let Ok(temp) = temp_str.parse::<f64>() {
                            return Ok(temp);
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow!("Could not read GPU temperature"))
}

/// Read fan speeds.
///
/// # Returns
///
/// A vector of fan speeds in RPM.
fn read_fan_speeds() -> Result<Vec<u32>> {
    let mut speeds = Vec::new();
    
    // Try using lm-sensors
    let output = Command::new("sensors")
        .output()
        .context("Failed to execute sensors command")?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for lines like "fan1: 1500 RPM"
        for line in output_str.lines() {
            if line.contains("fan") && line.contains("RPM") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let speed_part = parts[1].trim();
                    let speed_str: String = speed_part.chars()
                        .take_while(|c| c.is_digit(10) || *c == '.')
                        .collect();
                    
                    if let Ok(speed) = speed_str.parse::<u32>() {
                        speeds.push(speed);
                    }
                }
            }
        }
    }
    
    if speeds.is_empty() {
        // Try reading directly from sysfs
        let hwmon_dir = Path::new("/sys/class/hwmon");
        if hwmon_dir.exists() {
            for entry in fs::read_dir(hwmon_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Check for fan speed files
                for i in 1..10 {
                    let fan_file = path.join(format!("fan{}_input", i));
                    if fan_file.exists() {
                        let mut file = File::open(&fan_file)?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        
                        if let Ok(speed) = contents.trim().parse::<u32>() {
                            speeds.push(speed);
                        }
                    }
                }
            }
        }
    }
    
    if speeds.is_empty() {
        return Err(anyhow!("Could not read fan speeds"));
    }
    
    Ok(speeds)
}

/// Battery status information.
#[derive(Debug)]
struct BatteryStatus {
    /// Battery charge percentage.
    percent: f64,
    
    /// Whether the battery is charging.
    charging: bool,
    
    /// Estimated time remaining in minutes, if available.
    time_remaining: Option<f64>,
}

/// Read battery status.
///
/// # Returns
///
/// Battery status information.
fn read_battery_status() -> Result<BatteryStatus> {
    // Try using upower
    let output = Command::new("upower")
        .args(&["-i", "/org/freedesktop/UPower/devices/battery_BAT0"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            let mut percent = 0.0;
            let mut charging = false;
            let mut time_remaining = None;
            
            for line in output_str.lines() {
                if line.contains("percentage:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let percent_str: String = parts[1].trim().chars()
                            .take_while(|c| c.is_digit(10) || *c == '.')
                            .collect();
                        
                        if let Ok(p) = percent_str.parse::<f64>() {
                            percent = p;
                        }
                    }
                } else if line.contains("state:") {
                    charging = line.contains("charging");
                } else if line.contains("time to empty:") || line.contains("time to full:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let time_part = parts[1].trim();
                        
                        // Parse time in format like "1.5 hours" or "45.3 minutes"
                        let time_parts: Vec<&str> = time_part.split_whitespace().collect();
                        if time_parts.len() >= 2 {
                            if let Ok(time) = time_parts[0].parse::<f64>() {
                                if time_parts[1].starts_with("hour") {
                                    time_remaining = Some(time * 60.0);
                                } else if time_parts[1].starts_with("minute") {
                                    time_remaining = Some(time);
                                }
                            }
                        }
                    }
                }
            }
            
            return Ok(BatteryStatus {
                percent,
                charging,
                time_remaining,
            });
        }
    }
    
    // Try reading directly from sysfs
    let battery_dir = Path::new("/sys/class/power_supply/BAT0");
    if battery_dir.exists() {
        let mut percent = 0.0;
        let mut charging = false;
        let mut time_remaining = None;
        
        // Read capacity
        let capacity_file = battery_dir.join("capacity");
        if capacity_file.exists() {
            let mut file = File::open(&capacity_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            if let Ok(p) = contents.trim().parse::<f64>() {
                percent = p;
            }
        }
        
        // Read status
        let status_file = battery_dir.join("status");
        if status_file.exists() {
            let mut file = File::open(&status_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            charging = contents.trim() == "Charging";
        }
        
        // Read energy and power to calculate time remaining
        let energy_file = battery_dir.join("energy_now");
        let power_file = battery_dir.join("power_now");
        
        if energy_file.exists() && power_file.exists() {
            let mut energy_file = File::open(&energy_file)?;
            let mut power_file = File::open(&power_file)?;
            
            let mut energy_str = String::new();
            let mut power_str = String::new();
            
            energy_file.read_to_string(&mut energy_str)?;
            power_file.read_to_string(&mut power_str)?;
            
            if let (Ok(energy), Ok(power)) = (energy_str.trim().parse::<f64>(), power_str.trim().parse::<f64>()) {
                if power > 0.0 {
                    let hours = energy / power;
                    time_remaining = Some(hours * 60.0);
                }
            }
        }
        
        return Ok(BatteryStatus {
            percent,
            charging,
            time_remaining,
        });
    }
    
    Err(anyhow!("Could not read battery status"))
}

/// Display information.
#[derive(Debug)]
struct DisplayInfo {
    /// Display name.
    name: String,
    
    /// Display width in pixels.
    width: u32,
    
    /// Display height in pixels.
    height: u32,
    
    /// Display refresh rate in Hz.
    refresh_rate: f64,
}

/// Read display information.
///
/// # Returns
///
/// A vector of display information.
fn read_display_info() -> Result<Vec<DisplayInfo>> {
    let mut displays = Vec::new();
    
    // Try using xrandr
    let output = Command::new("xrandr")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            let mut current_display = None;
            
            for line in output_str.lines() {
                if line.contains(" connected ") {
                    // New display found
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        current_display = Some(parts[0].to_string());
                    }
                } else if line.contains("*") && current_display.is_some() {
                    // This line contains the current mode
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let resolution_parts: Vec<&str> = parts[0].split('x').collect();
                        if resolution_parts.len() >= 2 {
                            if let (Ok(width), Ok(height)) = (
                                resolution_parts[0].parse::<u32>(),
                                resolution_parts[1].parse::<u32>(),
                            ) {
                                let mut refresh_rate = 60.0; // Default
                                
                                // Look for refresh rate
                                for part in &parts[1..] {
                                    if part.contains("*") {
                                        let rate_str: String = part.chars()
                                            .take_while(|c| c.is_digit(10) || *c == '.')
                                            .collect();
                                        
                                        if let Ok(rate) = rate_str.parse::<f64>() {
                                            refresh_rate = rate;
                                            break;
                                        }
                                    }
                                }
                                
                                displays.push(DisplayInfo {
                                    name: current_display.clone().unwrap(),
                                    width,
                                    height,
                                    refresh_rate,
                                });
                                
                                current_display = None;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if displays.is_empty() {
        // Fallback to a default display for VR headset
        displays.push(DisplayInfo {
            name: "VR Headset Display".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 90.0,
        });
    }
    
    Ok(displays)
}

/// Sensor status information.
#[derive(Debug)]
struct SensorStatus {
    /// Sensor name.
    name: String,
    
    /// Sensor type.
    sensor_type: String,
    
    /// Whether the sensor is connected.
    connected: bool,
    
    /// Sensor error rate (0.0 to 1.0).
    error_rate: f64,
}

/// Read sensor status.
///
/// # Returns
///
/// A vector of sensor status information.
fn read_sensor_status() -> Result<Vec<SensorStatus>> {
    // This is a placeholder implementation for VR headset sensors
    // In a real implementation, this would query the actual sensors
    
    let mut sensors = Vec::new();
    
    // Add some example sensors
    sensors.push(SensorStatus {
        name: "IMU".to_string(),
        sensor_type: "Inertial".to_string(),
        connected: true,
        error_rate: 0.01,
    });
    
    sensors.push(SensorStatus {
        name: "Camera Left".to_string(),
        sensor_type: "Optical".to_string(),
        connected: true,
        error_rate: 0.02,
    });
    
    sensors.push(SensorStatus {
        name: "Camera Right".to_string(),
        sensor_type: "Optical".to_string(),
        connected: true,
        error_rate: 0.02,
    });
    
    sensors.push(SensorStatus {
        name: "Proximity".to_string(),
        sensor_type: "Infrared".to_string(),
        connected: true,
        error_rate: 0.0,
    });
    
    Ok(sensors)
}

/// Application usage statistics.
#[derive(Debug)]
struct AppUsageStats {
    /// Usage time in minutes.
    usage_time_minutes: f64,
    
    /// Number of times the application was launched.
    launch_count: u32,
    
    /// Number of times the application crashed.
    crash_count: u32,
}

/// Read application usage statistics.
///
/// # Returns
///
/// A map of application names to usage statistics.
fn read_application_usage() -> Result<HashMap<String, AppUsageStats>> {
    // This is a placeholder implementation
    // In a real implementation, this would read from a usage database
    
    let mut usage = HashMap::new();
    
    usage.insert("VR Home".to_string(), AppUsageStats {
        usage_time_minutes: 120.5,
        launch_count: 15,
        crash_count: 0,
    });
    
    usage.insert("VR Browser".to_string(), AppUsageStats {
        usage_time_minutes: 45.2,
        launch_count: 8,
        crash_count: 1,
    });
    
    usage.insert("VR Media Player".to_string(), AppUsageStats {
        usage_time_minutes: 78.3,
        launch_count: 12,
        crash_count: 0,
    });
    
    Ok(usage)
}

/// Rendering performance metrics.
#[derive(Debug)]
struct RenderingMetrics {
    /// Frames per second.
    fps: f64,
    
    /// Frame time in milliseconds.
    frame_time_ms: f64,
    
    /// GPU time in milliseconds.
    gpu_time_ms: f64,
    
    /// CPU time in milliseconds.
    cpu_time_ms: f64,
}

/// Read rendering performance metrics.
///
/// # Returns
///
/// Rendering performance metrics.
fn read_rendering_metrics() -> Result<RenderingMetrics> {
    // This is a placeholder implementation
    // In a real implementation, this would read from the rendering engine
    
    Ok(RenderingMetrics {
        fps: 90.0,
        frame_time_ms: 11.1,
        gpu_time_ms: 8.5,
        cpu_time_ms: 2.6,
    })
}

/// WiFi signal information.
#[derive(Debug)]
struct WiFiSignal {
    /// SSID of the connected network.
    ssid: String,
    
    /// Signal strength in dBm.
    strength_dbm: i32,
    
    /// Signal quality as a percentage.
    quality_percent: u32,
}

/// Read WiFi signal strength.
///
/// # Returns
///
/// WiFi signal information.
fn read_wifi_signal_strength() -> Result<WiFiSignal> {
    // Try using iwconfig
    let output = Command::new("iwconfig")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            let mut ssid = String::new();
            let mut strength_dbm = 0;
            let mut quality_percent = 0;
            
            for line in output_str.lines() {
                if line.contains("ESSID:") {
                    let start = line.find("ESSID:\"");
                    let end = line.rfind('"');
                    
                    if let (Some(start), Some(end)) = (start, end) {
                        ssid = line[start + 7..end].to_string();
                    }
                } else if line.contains("Signal level=") {
                    let start = line.find("Signal level=");
                    
                    if let Some(start) = start {
                        let level_part = &line[start + 13..];
                        let end = level_part.find(' ').unwrap_or(level_part.len());
                        let level_str = &level_part[..end];
                        
                        if level_str.ends_with("dBm") {
                            if let Ok(level) = level_str[..level_str.len() - 3].parse::<i32>() {
                                strength_dbm = level;
                                
                                // Convert dBm to percentage (approximate)
                                // -50 dBm or higher is excellent (100%)
                                // -100 dBm or lower is very poor (0%)
                                if strength_dbm >= -50 {
                                    quality_percent = 100;
                                } else if strength_dbm <= -100 {
                                    quality_percent = 0;
                                } else {
                                    quality_percent = ((strength_dbm + 100) * 2) as u32;
                                }
                            }
                        }
                    }
                } else if line.contains("Quality=") {
                    let start = line.find("Quality=");
                    
                    if let Some(start) = start {
                        let quality_part = &line[start + 8..];
                        let end = quality_part.find(' ').unwrap_or(quality_part.len());
                        let quality_str = &quality_part[..end];
                        
                        if quality_str.contains('/') {
                            let parts: Vec<&str> = quality_str.split('/').collect();
                            if parts.len() >= 2 {
                                if let (Ok(current), Ok(max)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                                    if max > 0 {
                                        quality_percent = (current * 100) / max;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if !ssid.is_empty() {
                return Ok(WiFiSignal {
                    ssid,
                    strength_dbm,
                    quality_percent,
                });
            }
        }
    }
    
    // Fallback to a placeholder
    Ok(WiFiSignal {
        ssid: "VR_Network".to_string(),
        strength_dbm: -65,
        quality_percent: 70,
    })
}

/// Measure network latency to various hosts.
///
/// # Returns
///
/// A map of host names to ping times in milliseconds.
fn measure_network_latency() -> Result<HashMap<String, f64>> {
    let mut latency = HashMap::new();
    
    // List of hosts to ping
    let hosts = [
        "8.8.8.8",           // Google DNS
        "1.1.1.1",           // Cloudflare DNS
        "www.example.com",   // Example website
    ];
    
    for host in &hosts {
        let output = Command::new("ping")
            .args(&["-c", "1", "-W", "2", host])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Look for "time=X ms"
                for line in output_str.lines() {
                    if line.contains("time=") {
                        let start = line.find("time=");
                        
                        if let Some(start) = start {
                            let time_part = &line[start + 5..];
                            let end = time_part.find(' ').unwrap_or(time_part.len());
                            let time_str = &time_part[..end];
                            
                            if let Ok(time) = time_str.parse::<f64>() {
                                latency.insert(host.to_string(), time);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(latency)
}

/// Internet connection status.
#[derive(Debug)]
struct ConnectionStatus {
    /// Whether the device is connected to the internet.
    connected: bool,
    
    /// Type of connection (WiFi, Ethernet, etc.).
    connection_type: String,
}

/// Check internet connection status.
///
/// # Returns
///
/// Internet connection status.
fn check_internet_connection() -> Result<ConnectionStatus> {
    // Check if we can ping a reliable host
    let output = Command::new("ping")
        .args(&["-c", "1", "-W", "2", "8.8.8.8"])
        .output();
    
    let connected = if let Ok(output) = output {
        output.status.success()
    } else {
        false
    };
    
    // Determine connection type
    let mut connection_type = "Unknown".to_string();
    
    if connected {
        // Check for WiFi connection
        let iwconfig_output = Command::new("iwconfig")
            .output();
        
        if let Ok(output) = iwconfig_output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("ESSID:") && !output_str.contains("ESSID:\"\"") {
                    connection_type = "WiFi".to_string();
                }
            }
        }
        
        // If not WiFi, check for Ethernet
        if connection_type == "Unknown" {
            let ifconfig_output = Command::new("ifconfig")
                .output();
            
            if let Ok(output) = ifconfig_output {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains("eth") || output_str.contains("enp") {
                        connection_type = "Ethernet".to_string();
                    }
                }
            }
        }
    }
    
    Ok(ConnectionStatus {
        connected,
        connection_type,
    })
}

/// Crash report information.
#[derive(Debug)]
struct CrashReport {
    /// Name of the application that crashed.
    app_name: String,
    
    /// Type of crash.
    crash_type: String,
    
    /// Error message.
    error_message: String,
    
    /// Timestamp of the crash.
    timestamp: chrono::DateTime<Utc>,
}

/// Read crash reports.
///
/// # Returns
///
/// A vector of crash reports.
fn read_crash_reports() -> Result<Vec<CrashReport>> {
    // This is a placeholder implementation
    // In a real implementation, this would read from crash report files
    
    let mut reports = Vec::new();
    
    // Add some example crash reports
    reports.push(CrashReport {
        app_name: "VR Browser".to_string(),
        crash_type: "Segmentation Fault".to_string(),
        error_message: "Null pointer dereference in render thread".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(2),
    });
    
    Ok(reports)
}

/// System error information.
#[derive(Debug)]
struct SystemError {
    /// Component that generated the error.
    component: String,
    
    /// Error code.
    error_code: i32,
    
    /// Error message.
    error_message: String,
    
    /// Timestamp of the error.
    timestamp: chrono::DateTime<Utc>,
}

/// Read system errors.
///
/// # Returns
///
/// A vector of system errors.
fn read_system_errors() -> Result<Vec<SystemError>> {
    // This is a placeholder implementation
    // In a real implementation, this would read from system logs
    
    let mut errors = Vec::new();
    
    // Add some example system errors
    errors.push(SystemError {
        component: "Graphics Driver".to_string(),
        error_code: 1234,
        error_message: "Failed to initialize shader compiler".to_string(),
        timestamp: Utc::now() - chrono::Duration::days(1),
    });
    
    Ok(errors)
}

/// Hardware error information.
#[derive(Debug)]
struct HardwareError {
    /// Device that generated the error.
    device: String,
    
    /// Error code.
    error_code: i32,
    
    /// Error message.
    error_message: String,
    
    /// Timestamp of the error.
    timestamp: chrono::DateTime<Utc>,
}

/// Read hardware errors.
///
/// # Returns
///
/// A vector of hardware errors.
fn read_hardware_errors() -> Result<Vec<HardwareError>> {
    // This is a placeholder implementation
    // In a real implementation, this would read from hardware diagnostics
    
    let mut errors = Vec::new();
    
    // Add some example hardware errors
    errors.push(HardwareError {
        device: "Left Controller".to_string(),
        error_code: 5678,
        error_message: "Tracking lost - occlusion detected".to_string(),
        timestamp: Utc::now() - chrono::Duration::minutes(30),
    });
    
    Ok(errors)
}

/// UI interaction statistics.
#[derive(Debug)]
struct UIInteractionStats {
    /// Count of interactions with UI elements.
    interaction_counts: HashMap<String, u32>,
    
    /// Time spent on different screens (in seconds).
    screen_times: HashMap<String, f64>,
}

/// Read UI interaction statistics.
///
/// # Returns
///
/// UI interaction statistics.
fn read_ui_interaction_stats() -> Result<UIInteractionStats> {
    // This is a placeholder implementation
    // In a real implementation, this would read from a usage database
    
    let mut interaction_counts = HashMap::new();
    interaction_counts.insert("Settings Button".to_string(), 12);
    interaction_counts.insert("Home Button".to_string(), 25);
    interaction_counts.insert("App Launcher".to_string(), 18);
    
    let mut screen_times = HashMap::new();
    screen_times.insert("Home Screen".to_string(), 450.0);
    screen_times.insert("Settings Screen".to_string(), 120.0);
    screen_times.insert("App Library".to_string(), 180.0);
    
    Ok(UIInteractionStats {
        interaction_counts,
        screen_times,
    })
}

/// Feature usage statistics.
#[derive(Debug)]
struct FeatureUsageStats {
    /// Count of feature usage.
    usage_counts: HashMap<String, u32>,
}

/// Read feature usage statistics.
///
/// # Returns
///
/// Feature usage statistics.
fn read_feature_usage_stats() -> Result<FeatureUsageStats> {
    // This is a placeholder implementation
    // In a real implementation, this would read from a usage database
    
    let mut usage_counts = HashMap::new();
    usage_counts.insert("Voice Commands".to_string(), 8);
    usage_counts.insert("Hand Tracking".to_string(), 15);
    usage_counts.insert("Room Setup".to_string(), 2);
    
    Ok(FeatureUsageStats {
        usage_counts,
    })
}

/// Session statistics.
#[derive(Debug)]
struct SessionStats {
    /// Number of sessions.
    session_count: u32,
    
    /// Average session duration in minutes.
    average_session_duration_minutes: f64,
    
    /// Total usage time in minutes.
    total_usage_time_minutes: f64,
}

/// Read session statistics.
///
/// # Returns
///
/// Session statistics.
fn read_session_stats() -> Result<SessionStats> {
    // This is a placeholder implementation
    // In a real implementation, this would read from a usage database
    
    Ok(SessionStats {
        session_count: 42,
        average_session_duration_minutes: 35.5,
        total_usage_time_minutes: 1491.0,
    })
}
