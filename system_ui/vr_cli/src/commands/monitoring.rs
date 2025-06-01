use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use prettytable::{Table, Row, Cell};
use vr_core_api::{VRCoreAPI, monitoring::{MetricType, MetricValue, AlertLevel}};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

use crate::MonitoringCommands;
use crate::utils::{error, formatting, file, validation};

pub fn handle_command(command: &MonitoringCommands, api: &mut VRCoreAPI) -> Result<()> {
    match command {
        MonitoringCommands::Status { component, format } => {
            system_status(api, component.as_deref(), format)
        },
        MonitoringCommands::Metrics { component, metric, interval, count, format } => {
            show_metrics(api, component.as_deref(), metric.as_deref(), *interval, *count, format)
        },
        MonitoringCommands::Alerts { level, component, format } => {
            show_alerts(api, level.as_deref(), component.as_deref(), format)
        },
        MonitoringCommands::Log { component, level, count, follow, format } => {
            show_logs(api, component.as_deref(), level.as_deref(), *count, *follow, format)
        },
        MonitoringCommands::Performance { component, duration, format } => {
            performance_report(api, component.as_deref(), *duration, format)
        },
    }
}

fn system_status(api: &VRCoreAPI, component_filter: Option<&str>, format: &str) -> Result<()> {
    error::print_section("System Status");
    
    // Get all components
    let components = [
        "cpu", "memory", "gpu", "storage", "network", "battery", 
        "display", "audio", "tracking", "thermal"
    ];
    
    // Filter components if requested
    let filtered_components: Vec<&str> = if let Some(filter) = component_filter {
        components.iter()
            .filter(|c| c.contains(filter))
            .copied()
            .collect()
    } else {
        components.iter().copied().collect()
    };
    
    if filtered_components.is_empty() && component_filter.is_some() {
        return Err(anyhow!("Invalid component filter: {}", component_filter.unwrap_or("")));
    }
    
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = serde_json::Map::new();
    
    for component in filtered_components {
        // Get status for this component
        let status = api.monitoring().get_component_status(component);
        
        // Get health percentage
        let health = api.monitoring().get_component_health(component);
        
        // Get alert level
        let alert_level = api.monitoring().get_component_alert_level(component);
        
        // Get key metrics
        let metrics = api.monitoring().get_component_metrics(component);
        
        // Format status for display
        let status_str = match status {
            Ok(true) => "Online".green().to_string(),
            Ok(false) => "Offline".red().to_string(),
            Err(_) => "Unknown".yellow().to_string(),
        };
        
        // Format health for display
        let health_str = match health {
            Ok(h) if h >= 90.0 => format!("{}%", h).green().to_string(),
            Ok(h) if h >= 70.0 => format!("{}%", h).yellow().to_string(),
            Ok(h) => format!("{}%", h).red().to_string(),
            Err(_) => "Unknown".yellow().to_string(),
        };
        
        // Format alert level for display
        let alert_str = match alert_level {
            Ok(AlertLevel::None) => "None".green().to_string(),
            Ok(AlertLevel::Info) => "Info".blue().to_string(),
            Ok(AlertLevel::Warning) => "Warning".yellow().to_string(),
            Ok(AlertLevel::Critical) => "Critical".red().to_string(),
            Err(_) => "Unknown".yellow().to_string(),
        };
        
        // Format metrics for display
        let metrics_str = match metrics {
            Ok(m) if !m.is_empty() => {
                let mut parts = Vec::new();
                for (name, value) in m.iter().take(3) {
                    parts.push(format!("{}: {}", name, format_metric_value(value)));
                }
                parts.join(", ")
            },
            _ => "No metrics available".to_string(),
        };
        
        // Add to table data
        table_data.push(vec![
            component.to_string(),
            status_str.to_string(),
            health_str.to_string(),
            alert_str.to_string(),
            metrics_str,
        ]);
        
        // Add to JSON data
        let mut component_json = serde_json::Map::new();
        component_json.insert("status".to_string(), serde_json::Value::String(
            match status {
                Ok(true) => "online",
                Ok(false) => "offline",
                Err(_) => "unknown",
            }.to_string()
        ));
        
        component_json.insert("health".to_string(), 
            match health {
                Ok(h) => serde_json::Value::Number(serde_json::Number::from_f64(h).unwrap_or_default()),
                Err(_) => serde_json::Value::Null,
            }
        );
        
        component_json.insert("alert_level".to_string(), serde_json::Value::String(
            match alert_level {
                Ok(AlertLevel::None) => "none",
                Ok(AlertLevel::Info) => "info",
                Ok(AlertLevel::Warning) => "warning",
                Ok(AlertLevel::Critical) => "critical",
                Err(_) => "unknown",
            }.to_string()
        ));
        
        if let Ok(m) = metrics {
            let mut metrics_json = serde_json::Map::new();
            for (name, value) in m {
                metrics_json.insert(name, metric_value_to_json(&value));
            }
            component_json.insert("metrics".to_string(), serde_json::Value::Object(metrics_json));
        }
        
        json_data.insert(component.to_string(), serde_json::Value::Object(component_json));
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Component", "Status", "Health", "Alert Level", "Key Metrics"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(json_data))
                .context("Failed to format JSON")?);
        },
        "text" => {
            for row in &table_data {
                println!("{}: {} | Health: {} | Alerts: {} | {}", 
                         row[0].bold(), row[1], row[2], row[3], row[4]);
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn show_metrics(api: &VRCoreAPI, component_filter: Option<&str>, metric_filter: Option<&str>, 
                interval: u64, count: u64, format: &str) -> Result<()> {
    // Get all components
    let components = [
        "cpu", "memory", "gpu", "storage", "network", "battery", 
        "display", "audio", "tracking", "thermal"
    ];
    
    // Filter components if requested
    let filtered_components: Vec<&str> = if let Some(filter) = component_filter {
        components.iter()
            .filter(|c| c.contains(filter))
            .copied()
            .collect()
    } else {
        components.iter().copied().collect()
    };
    
    if filtered_components.is_empty() && component_filter.is_some() {
        return Err(anyhow!("Invalid component filter: {}", component_filter.unwrap_or("")));
    }
    
    // Prepare for metrics collection
    let interval_duration = Duration::from_secs(interval);
    let iterations = if count == 0 { u64::MAX } else { count };
    
    error::print_section(&format!("System Metrics (Interval: {}s, Count: {})", 
                                 interval, if count == 0 { "infinite" } else { &count.to_string() }));
    
    // For JSON output, we'll collect all data and output at the end
    let mut all_json_data = Vec::new();
    
    // For table output, print headers first
    if format == "table" {
        let mut headers = vec!["Timestamp".to_string()];
        
        for component in &filtered_components {
            // Get metrics for this component to determine column headers
            if let Ok(metrics) = api.monitoring().get_component_metrics(component) {
                for (name, _) in metrics {
                    if metric_filter.is_none() || metric_filter.as_ref().map_or(false, |f| name.contains(f)) {
                        headers.push(format!("{}.{}", component, name));
                    }
                }
            }
        }
        
        let header_cells: Vec<Cell> = headers.iter()
            .map(|h| Cell::new(h).style_spec("Fb"))
            .collect();
        
        let mut table = Table::new();
        table.add_row(Row::new(header_cells));
        table.printstd();
    }
    
    // Collect metrics for the specified number of iterations
    for i in 0..iterations {
        let start_time = Instant::now();
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        
        // Collect metrics for this iteration
        let mut row_data = vec![timestamp.clone()];
        let mut json_data = serde_json::Map::new();
        json_data.insert("timestamp".to_string(), serde_json::Value::String(timestamp));
        
        for component in &filtered_components {
            // Get metrics for this component
            if let Ok(metrics) = api.monitoring().get_component_metrics(component) {
                let mut component_json = serde_json::Map::new();
                
                for (name, value) in metrics {
                    if metric_filter.is_none() || metric_filter.as_ref().map_or(false, |f| name.contains(f)) {
                        // Add to row data for table output
                        row_data.push(format_metric_value(&value));
                        
                        // Add to JSON data
                        component_json.insert(name, metric_value_to_json(&value));
                    }
                }
                
                if !component_json.is_empty() {
                    json_data.insert(component.to_string(), serde_json::Value::Object(component_json));
                }
            }
        }
        
        // Output based on format
        match format.to_lowercase().as_str() {
            "table" => {
                let cells: Vec<Cell> = row_data.iter()
                    .map(|d| Cell::new(d))
                    .collect();
                
                let mut table = Table::new();
                table.add_row(Row::new(cells));
                table.printstd();
            },
            "json" => {
                all_json_data.push(serde_json::Value::Object(json_data));
                
                // If this is the last iteration or we're doing infinite iterations,
                // print the current data point
                if count == 0 || i == iterations - 1 {
                    println!("{}", serde_json::to_string_pretty(&all_json_data.last().unwrap())
                        .context("Failed to format JSON")?);
                }
            },
            "text" => {
                println!("[{}]", timestamp);
                
                let mut metric_index = 1;
                for component in &filtered_components {
                    if let Ok(metrics) = api.monitoring().get_component_metrics(component) {
                        for (name, value) in metrics {
                            if metric_filter.is_none() || metric_filter.as_ref().map_or(false, |f| name.contains(f)) {
                                println!("  {}.{}: {}", component, name, row_data[metric_index]);
                                metric_index += 1;
                            }
                        }
                    }
                }
                println!();
            },
            _ => {
                return Err(anyhow!("Unsupported output format: {}", format));
            }
        }
        
        // Wait for the next interval
        let elapsed = start_time.elapsed();
        if elapsed < interval_duration && i < iterations - 1 {
            thread::sleep(interval_duration - elapsed);
        }
    }
    
    // Output all JSON data at the end if we're not doing infinite iterations
    if format == "json" && count > 0 {
        println!("{}", serde_json::to_string_pretty(&all_json_data)
            .context("Failed to format JSON")?);
    }
    
    Ok(())
}

fn show_alerts(api: &VRCoreAPI, level_filter: Option<&str>, component_filter: Option<&str>, format: &str) -> Result<()> {
    // Parse alert level filter
    let alert_level = if let Some(level) = level_filter {
        match level.to_lowercase().as_str() {
            "none" => Some(AlertLevel::None),
            "info" => Some(AlertLevel::Info),
            "warning" => Some(AlertLevel::Warning),
            "critical" => Some(AlertLevel::Critical),
            _ => return Err(anyhow!("Invalid alert level: {}", level)),
        }
    } else {
        None
    };
    
    error::print_section("System Alerts");
    
    // Get all alerts
    let alerts = api.monitoring().get_alerts(alert_level, component_filter);
    
    if alerts.is_empty() {
        println!("No alerts found matching the specified criteria.");
        return Ok(());
    }
    
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = Vec::new();
    
    for alert in &alerts {
        // Format timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&alert.timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|_| alert.timestamp.clone());
        
        // Format alert level for display
        let level_str = match alert.level {
            AlertLevel::None => "None".green().to_string(),
            AlertLevel::Info => "Info".blue().to_string(),
            AlertLevel::Warning => "Warning".yellow().to_string(),
            AlertLevel::Critical => "Critical".red().to_string(),
        };
        
        // Add to table data
        table_data.push(vec![
            timestamp.clone(),
            alert.component.clone(),
            level_str.to_string(),
            alert.message.clone(),
        ]);
        
        // Add to JSON data
        let mut alert_json = serde_json::Map::new();
        alert_json.insert("timestamp".to_string(), serde_json::Value::String(timestamp));
        alert_json.insert("component".to_string(), serde_json::Value::String(alert.component.clone()));
        alert_json.insert("level".to_string(), serde_json::Value::String(
            match alert.level {
                AlertLevel::None => "none",
                AlertLevel::Info => "info",
                AlertLevel::Warning => "warning",
                AlertLevel::Critical => "critical",
            }.to_string()
        ));
        alert_json.insert("message".to_string(), serde_json::Value::String(alert.message.clone()));
        
        if let Some(ref details) = alert.details {
            alert_json.insert("details".to_string(), serde_json::Value::String(details.clone()));
        }
        
        json_data.push(serde_json::Value::Object(alert_json));
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Timestamp", "Component", "Level", "Message"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&json_data)
                .context("Failed to format JSON")?);
        },
        "text" => {
            for (i, alert) in alerts.iter().enumerate() {
                let level_str = match alert.level {
                    AlertLevel::None => "None".green(),
                    AlertLevel::Info => "Info".blue(),
                    AlertLevel::Warning => "Warning".yellow(),
                    AlertLevel::Critical => "Critical".red(),
                };
                
                println!("Alert #{}", i + 1);
                println!("Timestamp: {}", table_data[i][0]);
                println!("Component: {}", alert.component);
                println!("Level: {}", level_str);
                println!("Message: {}", alert.message);
                
                if let Some(ref details) = alert.details {
                    println!("Details: {}", details);
                }
                
                println!();
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn show_logs(api: &VRCoreAPI, component_filter: Option<&str>, level_filter: Option<&str>, 
             count: u64, follow: bool, format: &str) -> Result<()> {
    // Parse log level filter
    let log_level = if let Some(level) = level_filter {
        match level.to_lowercase().as_str() {
            "trace" => Some("TRACE"),
            "debug" => Some("DEBUG"),
            "info" => Some("INFO"),
            "warn" | "warning" => Some("WARN"),
            "error" => Some("ERROR"),
            "fatal" => Some("FATAL"),
            _ => return Err(anyhow!("Invalid log level: {}", level)),
        }
    } else {
        None
    };
    
    error::print_section("System Logs");
    
    // Get initial logs
    let mut logs = api.monitoring().get_logs(component_filter, log_level, count);
    
    if logs.is_empty() && !follow {
        println!("No logs found matching the specified criteria.");
        return Ok(());
    }
    
    // Output initial logs
    output_logs(&logs, format)?;
    
    // If follow mode is enabled, continue to get new logs
    if follow {
        println!("\nWaiting for new logs... (Press Ctrl+C to exit)");
        
        // Get the timestamp of the last log
        let mut last_timestamp = logs.last().map(|log| log.timestamp.clone()).unwrap_or_default();
        
        loop {
            // Wait a bit before checking for new logs
            thread::sleep(Duration::from_millis(500));
            
            // Get new logs since the last timestamp
            let new_logs = api.monitoring().get_logs_since(component_filter, log_level, &last_timestamp);
            
            if !new_logs.is_empty() {
                // Output new logs
                output_logs(&new_logs, format)?;
                
                // Update the last timestamp
                last_timestamp = new_logs.last().map(|log| log.timestamp.clone()).unwrap_or(last_timestamp);
            }
        }
    }
    
    Ok(())
}

fn output_logs(logs: &[vr_core_api::monitoring::LogEntry], format: &str) -> Result<()> {
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = Vec::new();
    
    for log in logs {
        // Format timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&log.timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|_| log.timestamp.clone());
        
        // Format log level for display
        let level_str = match log.level.as_str() {
            "TRACE" => "TRACE".magenta().to_string(),
            "DEBUG" => "DEBUG".blue().to_string(),
            "INFO" => "INFO".green().to_string(),
            "WARN" => "WARN".yellow().to_string(),
            "ERROR" => "ERROR".red().to_string(),
            "FATAL" => "FATAL".red().bold().to_string(),
            _ => log.level.clone(),
        };
        
        // Add to table data
        table_data.push(vec![
            timestamp.clone(),
            log.component.clone(),
            level_str.to_string(),
            log.message.clone(),
        ]);
        
        // Add to JSON data
        let mut log_json = serde_json::Map::new();
        log_json.insert("timestamp".to_string(), serde_json::Value::String(timestamp));
        log_json.insert("component".to_string(), serde_json::Value::String(log.component.clone()));
        log_json.insert("level".to_string(), serde_json::Value::String(log.level.clone()));
        log_json.insert("message".to_string(), serde_json::Value::String(log.message.clone()));
        
        json_data.push(serde_json::Value::Object(log_json));
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Timestamp", "Component", "Level", "Message"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&json_data)
                .context("Failed to format JSON")?);
        },
        "text" => {
            for log in logs {
                let level_str = match log.level.as_str() {
                    "TRACE" => "TRACE".magenta(),
                    "DEBUG" => "DEBUG".blue(),
                    "INFO" => "INFO".green(),
                    "WARN" => "WARN".yellow(),
                    "ERROR" => "ERROR".red(),
                    "FATAL" => "FATAL".red().bold(),
                    _ => log.level.as_str().normal(),
                };
                
                let timestamp = chrono::DateTime::parse_from_rfc3339(&log.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|_| log.timestamp.clone());
                
                println!("[{}] [{}] [{}] {}", 
                         timestamp, log.component, level_str, log.message);
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn performance_report(api: &VRCoreAPI, component_filter: Option<&str>, duration: u64, format: &str) -> Result<()> {
    // Get all components
    let components = [
        "cpu", "memory", "gpu", "storage", "network", "battery", 
        "display", "audio", "tracking", "thermal"
    ];
    
    // Filter components if requested
    let filtered_components: Vec<&str> = if let Some(filter) = component_filter {
        components.iter()
            .filter(|c| c.contains(filter))
            .copied()
            .collect()
    } else {
        components.iter().copied().collect()
    };
    
    if filtered_components.is_empty() && component_filter.is_some() {
        return Err(anyhow!("Invalid component filter: {}", component_filter.unwrap_or("")));
    }
    
    error::print_section(&format!("Performance Report (Duration: {}s)", duration));
    
    // Create progress bar
    let pb = error::create_progress_bar(duration, "Collecting performance data");
    
    // Collect performance data
    let mut performance_data = HashMap::new();
    let start_time = Instant::now();
    
    for i in 0..duration {
        // Collect metrics for each component
        for component in &filtered_components {
            if let Ok(metrics) = api.monitoring().get_component_metrics(component) {
                let component_data = performance_data
                    .entry(component.to_string())
                    .or_insert_with(HashMap::new);
                
                for (name, value) in metrics {
                    let metric_data = component_data
                        .entry(name)
                        .or_insert_with(Vec::new);
                    
                    metric_data.push(value);
                }
            }
        }
        
        // Update progress bar
        pb.set_position(i + 1);
        
        // Wait for the next second
        let elapsed = start_time.elapsed().as_secs();
        if elapsed <= i {
            thread::sleep(Duration::from_secs(1));
        }
    }
    
    pb.finish_with_message("Performance data collection complete");
    
    // Analyze performance data
    let mut table_data = Vec::new();
    let mut json_data = serde_json::Map::new();
    
    for (component, metrics) in &performance_data {
        let mut component_json = serde_json::Map::new();
        
        for (metric_name, values) in metrics {
            // Calculate statistics
            let (min, max, avg) = calculate_statistics(values);
            
            // Add to table data
            table_data.push(vec![
                component.clone(),
                metric_name.clone(),
                format_metric_value(&min),
                format_metric_value(&max),
                format_metric_value(&avg),
            ]);
            
            // Add to JSON data
            let mut metric_json = serde_json::Map::new();
            metric_json.insert("min".to_string(), metric_value_to_json(&min));
            metric_json.insert("max".to_string(), metric_value_to_json(&max));
            metric_json.insert("avg".to_string(), metric_value_to_json(&avg));
            
            component_json.insert(metric_name.clone(), serde_json::Value::Object(metric_json));
        }
        
        json_data.insert(component.clone(), serde_json::Value::Object(component_json));
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Component", "Metric", "Min", "Max", "Avg"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(json_data))
                .context("Failed to format JSON")?);
        },
        "text" => {
            for component in &filtered_components {
                if let Some(metrics) = performance_data.get(*component) {
                    println!("\n{}:", component.bold());
                    
                    for (metric_name, values) in metrics {
                        let (min, max, avg) = calculate_statistics(values);
                        
                        println!("  {}: Min: {}, Max: {}, Avg: {}", 
                                 metric_name,
                                 format_metric_value(&min),
                                 format_metric_value(&max),
                                 format_metric_value(&avg));
                    }
                }
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

// Helper functions

fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::Integer(i) => i.to_string(),
        MetricValue::Float(f) => format!("{:.2}", f),
        MetricValue::Percentage(p) => format!("{}%", p),
        MetricValue::Temperature(t) => format!("{}°C", t),
        MetricValue::Bytes(b) => format_bytes(*b),
        MetricValue::BytesPerSecond(bps) => format!("{}/s", format_bytes(*bps)),
        MetricValue::Milliseconds(ms) => format!("{}ms", ms),
        MetricValue::Count(c) => c.to_string(),
        MetricValue::Boolean(b) => b.to_string(),
        MetricValue::String(s) => s.clone(),
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes < KB {
        format!("{}B", bytes)
    } else if bytes < MB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2}TB", bytes as f64 / TB as f64)
    }
}

fn metric_value_to_json(value: &MetricValue) -> serde_json::Value {
    match value {
        MetricValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        MetricValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        MetricValue::Percentage(p) => {
            if let Some(n) = serde_json::Number::from_f64(*p) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        MetricValue::Temperature(t) => {
            if let Some(n) = serde_json::Number::from_f64(*t) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        MetricValue::Bytes(b) => serde_json::Value::Number(serde_json::Number::from(*b)),
        MetricValue::BytesPerSecond(bps) => serde_json::Value::Number(serde_json::Number::from(*bps)),
        MetricValue::Milliseconds(ms) => serde_json::Value::Number(serde_json::Number::from(*ms)),
        MetricValue::Count(c) => serde_json::Value::Number(serde_json::Number::from(*c)),
        MetricValue::Boolean(b) => serde_json::Value::Bool(*b),
        MetricValue::String(s) => serde_json::Value::String(s.clone()),
    }
}

fn calculate_statistics(values: &[MetricValue]) -> (MetricValue, MetricValue, MetricValue) {
    if values.is_empty() {
        return (
            MetricValue::String("N/A".to_string()),
            MetricValue::String("N/A".to_string()),
            MetricValue::String("N/A".to_string()),
        );
    }
    
    // Determine the type of the first value
    match &values[0] {
        MetricValue::Integer(_) => {
            let mut min = i64::MAX;
            let mut max = i64::MIN;
            let mut sum = 0i64;
            
            for value in values {
                if let MetricValue::Integer(i) = value {
                    min = min.min(*i);
                    max = max.max(*i);
                    sum += i;
                }
            }
            
            let avg = sum as f64 / values.len() as f64;
            
            (
                MetricValue::Integer(min),
                MetricValue::Integer(max),
                MetricValue::Float(avg),
            )
        },
        MetricValue::Float(_) => {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            let mut sum = 0.0;
            
            for value in values {
                if let MetricValue::Float(f) = value {
                    min = min.min(*f);
                    max = max.max(*f);
                    sum += f;
                }
            }
            
            let avg = sum / values.len() as f64;
            
            (
                MetricValue::Float(min),
                MetricValue::Float(max),
                MetricValue::Float(avg),
            )
        },
        MetricValue::Percentage(_) => {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            let mut sum = 0.0;
            
            for value in values {
                if let MetricValue::Percentage(p) = value {
                    min = min.min(*p);
                    max = max.max(*p);
                    sum += p;
                }
            }
            
            let avg = sum / values.len() as f64;
            
            (
                MetricValue::Percentage(min),
                MetricValue::Percentage(max),
                MetricValue::Percentage(avg),
            )
        },
        MetricValue::Temperature(_) => {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            let mut sum = 0.0;
            
            for value in values {
                if let MetricValue::Temperature(t) = value {
                    min = min.min(*t);
                    max = max.max(*t);
                    sum += t;
                }
            }
            
            let avg = sum / values.len() as f64;
            
            (
                MetricValue::Temperature(min),
                MetricValue::Temperature(max),
                MetricValue::Temperature(avg),
            )
        },
        MetricValue::Bytes(_) => {
            let mut min = u64::MAX;
            let mut max = 0u64;
            let mut sum = 0u64;
            
            for value in values {
                if let MetricValue::Bytes(b) = value {
                    min = min.min(*b);
                    max = max.max(*b);
                    sum += b;
                }
            }
            
            let avg = sum as f64 / values.len() as f64;
            
            (
                MetricValue::Bytes(min),
                MetricValue::Bytes(max),
                MetricValue::Bytes(avg as u64),
            )
        },
        MetricValue::BytesPerSecond(_) => {
            let mut min = u64::MAX;
            let mut max = 0u64;
            let mut sum = 0u64;
            
            for value in values {
                if let MetricValue::BytesPerSecond(bps) = value {
                    min = min.min(*bps);
                    max = max.max(*bps);
                    sum += bps;
                }
            }
            
            let avg = sum as f64 / values.len() as f64;
            
            (
                MetricValue::BytesPerSecond(min),
                MetricValue::BytesPerSecond(max),
                MetricValue::BytesPerSecond(avg as u64),
            )
        },
        MetricValue::Milliseconds(_) => {
            let mut min = u64::MAX;
            let mut max = 0u64;
            let mut sum = 0u64;
            
            for value in values {
                if let MetricValue::Milliseconds(ms) = value {
                    min = min.min(*ms);
                    max = max.max(*ms);
                    sum += ms;
                }
            }
            
            let avg = sum as f64 / values.len() as f64;
            
            (
                MetricValue::Milliseconds(min),
                MetricValue::Milliseconds(max),
                MetricValue::Milliseconds(avg as u64),
            )
        },
        MetricValue::Count(_) => {
            let mut min = u64::MAX;
            let mut max = 0u64;
            let mut sum = 0u64;
            
            for value in values {
                if let MetricValue::Count(c) = value {
                    min = min.min(*c);
                    max = max.max(*c);
                    sum += c;
                }
            }
            
            let avg = sum as f64 / values.len() as f64;
            
            (
                MetricValue::Count(min),
                MetricValue::Count(max),
                MetricValue::Count(avg as u64),
            )
        },
        MetricValue::Boolean(_) => {
            let mut true_count = 0;
            
            for value in values {
                if let MetricValue::Boolean(b) = value {
                    if *b {
                        true_count += 1;
                    }
                }
            }
            
            let true_percentage = (true_count as f64 / values.len() as f64) * 100.0;
            
            (
                MetricValue::Boolean(true_count == 0),
                MetricValue::Boolean(true_count == values.len()),
                MetricValue::Percentage(true_percentage),
            )
        },
        MetricValue::String(_) => {
            // For string values, we can't really calculate min/max/avg
            (
                values[0].clone(),
                values[values.len() - 1].clone(),
                MetricValue::String(format!("{} values", values.len())),
            )
        },
    }
}
