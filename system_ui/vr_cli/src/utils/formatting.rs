// Formatting utilities for CLI output
use colored::Colorize;
use prettytable::{Table, Row, Cell, format};
use serde::Serialize;
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Format data as a table
pub fn format_table<T: AsRef<str>>(headers: &[T], rows: &[Vec<String>]) -> String {
    let mut table = Table::new();
    
    // Set table format
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    
    // Add headers
    let header_cells: Vec<Cell> = headers.iter()
        .map(|h| Cell::new(h.as_ref()).style_spec("Fb"))
        .collect();
    table.add_row(Row::new(header_cells));
    
    // Add rows
    for row in rows {
        let cells: Vec<Cell> = row.iter()
            .map(|c| Cell::new(c))
            .collect();
        table.add_row(Row::new(cells));
    }
    
    table.to_string()
}

/// Format data as JSON
pub fn format_json<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string_pretty(data).context("Failed to serialize to JSON")
}

/// Format data as TOML
pub fn format_toml<T: Serialize>(data: &T) -> Result<String> {
    toml::to_string(data).context("Failed to serialize to TOML")
}

/// Format key-value pairs
pub fn format_key_value_pairs(pairs: &HashMap<String, String>) -> String {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    
    for (key, value) in pairs {
        table.add_row(Row::new(vec![
            Cell::new(&key.blue().bold().to_string()),
            Cell::new(&value.to_string()),
        ]));
    }
    
    table.to_string()
}

/// Format a list of items
pub fn format_list<T: AsRef<str>>(items: &[T], title: &str) -> String {
    let mut result = String::new();
    result.push_str(&format!("{}\n", title.green().bold()));
    result.push_str(&format!("{}\n", "=".repeat(title.len()).green()));
    
    for (i, item) in items.iter().enumerate() {
        result.push_str(&format!("{}. {}\n", i + 1, item.as_ref()));
    }
    
    result
}

/// Format a value based on its type
pub fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".dimmed().to_string(),
        serde_json::Value::Bool(b) => if *b { "true".green() } else { "false".red() }.to_string(),
        serde_json::Value::Number(n) => n.to_string().yellow().to_string(),
        serde_json::Value::String(s) => s.blue().to_string(),
        serde_json::Value::Array(a) => {
            let items: Vec<String> = a.iter()
                .map(|v| format_value(v))
                .collect();
            format!("[{}]", items.join(", "))
        },
        serde_json::Value::Object(o) => {
            let items: Vec<String> = o.iter()
                .map(|(k, v)| format!("{}: {}", k.green(), format_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        },
    }
}

/// Format a duration in a human-readable format
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m {}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
    } else {
        format!("{}d {}h {}m {}s", 
                seconds / 86400, 
                (seconds % 86400) / 3600, 
                (seconds % 3600) / 60, 
                seconds % 60)
    }
}

/// Format a file size in a human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}

/// Format a percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.2}%", value)
}

/// Format a status with color
pub fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "ok" | "success" | "running" | "active" | "enabled" | "online" => 
            status.green().bold().to_string(),
        "warning" | "pending" | "partial" => 
            status.yellow().bold().to_string(),
        "error" | "failure" | "failed" | "stopped" | "disabled" | "offline" => 
            status.red().bold().to_string(),
        "unknown" | "unavailable" => 
            status.dimmed().to_string(),
        _ => status.to_string(),
    }
}
