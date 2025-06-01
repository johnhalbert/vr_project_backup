use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use prettytable::{Table, Row, Cell};
use vr_core_api::{VRCoreAPI, config::{ConfigCategory, ConfigValue}};
use std::collections::HashMap;
use std::path::Path;

use crate::ConfigCommands;
use crate::utils::{error, formatting, file, validation};

pub fn handle_command(command: &ConfigCommands, api: &mut VRCoreAPI) -> Result<()> {
    match command {
        ConfigCommands::List { category, format } => {
            list_config(api, category.as_deref(), format)
        },
        ConfigCommands::Get { category, key, format } => {
            get_config(api, category, key, format)
        },
        ConfigCommands::Set { category, key, value, type_, no_save } => {
            set_config(api, category, key, value, type_, *no_save)
        },
        ConfigCommands::Reset { category, force } => {
            reset_config(api, category.as_deref(), *force)
        },
        ConfigCommands::Export { file, format, category } => {
            export_config(api, file, format, category.as_deref())
        },
        ConfigCommands::Import { file, format, force } => {
            import_config(api, file, format, *force)
        },
        ConfigCommands::Compare { file, category, format } => {
            compare_config(api, file.as_deref(), category.as_deref(), format)
        },
        ConfigCommands::Search { term, category, keys_only, values_only } => {
            search_config(api, term, category.as_deref(), *keys_only, *values_only)
        },
    }
}

fn list_config(api: &VRCoreAPI, category_filter: Option<&str>, format: &str) -> Result<()> {
    error::print_section("Configuration Values");
    
    // Get all categories
    let categories = [
        ConfigCategory::Hardware,
        ConfigCategory::Display,
        ConfigCategory::Audio,
        ConfigCategory::Tracking,
        ConfigCategory::Network,
        ConfigCategory::Power,
        ConfigCategory::SteamVR,
        ConfigCategory::Security,
        ConfigCategory::System,
    ];
    
    // Filter categories if requested
    let filtered_categories: Vec<&ConfigCategory> = if let Some(filter) = category_filter {
        categories.iter()
            .filter(|c| c.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        categories.iter().collect()
    };
    
    if filtered_categories.is_empty() {
        return Err(anyhow!("Invalid category filter: {}", category_filter.unwrap_or("")));
    }
    
    // Prepare data for output
    let mut table_data = Vec::new();
    let mut json_data = serde_json::Map::new();
    let mut toml_data = HashMap::new();
    
    for category in filtered_categories {
        let category_str = category.to_string();
        let keys = get_keys_for_category(*category);
        
        let mut category_json = serde_json::Map::new();
        let mut category_toml = HashMap::new();
        
        for key in keys {
            match api.config().get(*category, key) {
                Ok(value) => {
                    // Add to table data
                    table_data.push(vec![
                        category_str.clone(),
                        key.to_string(),
                        format_config_value(&value),
                        get_type_name(&value),
                    ]);
                    
                    // Add to JSON data
                    category_json.insert(key.to_string(), config_value_to_json(&value));
                    
                    // Add to TOML data
                    category_toml.insert(key.to_string(), config_value_to_toml(&value));
                },
                Err(_) => {
                    // Key not found, skip
                }
            }
        }
        
        // Add category data to JSON and TOML
        json_data.insert(category_str.clone(), serde_json::Value::Object(category_json));
        toml_data.insert(category_str.clone(), category_toml);
    }
    
    // Output based on format
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Category", "Key", "Value", "Type"];
            println!("{}", formatting::format_table(&headers, &table_data));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(json_data))
                .context("Failed to format JSON")?);
        },
        "toml" => {
            println!("{}", toml::to_string(&toml_data)
                .context("Failed to format TOML")?);
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn get_config(api: &VRCoreAPI, category: &str, key: &str, format: &str) -> Result<()> {
    let category = parse_category(category)?;
    
    match api.config().get(category, key) {
        Ok(value) => {
            match format.to_lowercase().as_str() {
                "text" => {
                    println!("{}: {} = {} ({})", 
                             category.to_string().green().bold(),
                             key.blue().bold(),
                             format_config_value(&value).yellow(),
                             get_type_name(&value));
                },
                "json" => {
                    let mut data = serde_json::Map::new();
                    data.insert("category".to_string(), serde_json::Value::String(category.to_string()));
                    data.insert("key".to_string(), serde_json::Value::String(key.to_string()));
                    data.insert("value".to_string(), config_value_to_json(&value));
                    data.insert("type".to_string(), serde_json::Value::String(get_type_name(&value)));
                    
                    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(data))
                        .context("Failed to format JSON")?);
                },
                _ => {
                    return Err(anyhow!("Unsupported output format: {}", format));
                }
            }
            Ok(())
        },
        Err(e) => {
            error::print_warning(&format!("Configuration value not found: {}.{}", category.to_string(), key));
            Err(e.into())
        }
    }
}

fn set_config(api: &mut VRCoreAPI, category: &str, key: &str, value: &str, type_: &str, no_save: bool) -> Result<()> {
    let category = parse_category(category)?;
    
    // Parse value based on type
    let config_value = match type_.to_lowercase().as_str() {
        "string" => ConfigValue::String(value.to_string()),
        "integer" => {
            let parsed = value.parse::<i64>()
                .map_err(|_| anyhow!("Invalid integer value: {}", value))?;
            ConfigValue::Integer(parsed)
        },
        "float" => {
            let parsed = value.parse::<f64>()
                .map_err(|_| anyhow!("Invalid float value: {}", value))?;
            ConfigValue::Float(parsed)
        },
        "boolean" => {
            let parsed = match value.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => true,
                "false" | "no" | "0" | "off" => false,
                _ => return Err(anyhow!("Invalid boolean value: {}", value)),
            };
            ConfigValue::Boolean(parsed)
        },
        _ => return Err(anyhow!("Unsupported value type: {}", type_)),
    };
    
    // Set the value
    api.config_mut().set(category, key, config_value.clone())?;
    
    // Save configuration if requested
    if !no_save {
        api.config_mut().save()?;
        error::print_success(&format!("Set {}.{} = {} ({}) and saved configuration", 
                 category.to_string(),
                 key,
                 format_config_value(&config_value).yellow(),
                 get_type_name(&config_value)));
    } else {
        error::print_success(&format!("Set {}.{} = {} ({}) (not saved to disk)", 
                 category.to_string(),
                 key,
                 format_config_value(&config_value).yellow(),
                 get_type_name(&config_value)));
    }
    
    Ok(())
}

fn reset_config(api: &mut VRCoreAPI, category_filter: Option<&str>, force: bool) -> Result<()> {
    // Confirm reset if not forced
    if !force {
        let prompt = if let Some(category) = category_filter {
            format!("Are you sure you want to reset configuration for category {}?", category)
        } else {
            "Are you sure you want to reset all configuration to defaults?".to_string()
        };
        
        if !error::confirm(&prompt, false)? {
            error::print_info("Reset cancelled");
            return Ok(());
        }
    }
    
    // Create a new configuration with defaults
    let default_config = vr_core_api::config::Config::default();
    
    // Get all categories
    let categories = [
        ConfigCategory::Hardware,
        ConfigCategory::Display,
        ConfigCategory::Audio,
        ConfigCategory::Tracking,
        ConfigCategory::Network,
        ConfigCategory::Power,
        ConfigCategory::SteamVR,
        ConfigCategory::Security,
        ConfigCategory::System,
    ];
    
    // Filter categories if requested
    let filtered_categories: Vec<&ConfigCategory> = if let Some(filter) = category_filter {
        categories.iter()
            .filter(|c| c.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        categories.iter().collect()
    };
    
    if filtered_categories.is_empty() {
        return Err(anyhow!("Invalid category filter: {}", category_filter.unwrap_or("")));
    }
    
    // Reset each category
    for category in filtered_categories {
        let category_str = category.to_string();
        let keys = get_keys_for_category(*category);
        
        for key in keys {
            // Get default value
            if let Ok(default_value) = default_config.get(*category, key) {
                // Set value in current config
                api.config_mut().set(*category, key, default_value.clone())?;
                error::print_info(&format!("Reset {}.{} to default", category_str, key));
            }
        }
    }
    
    // Save configuration
    api.config_mut().save()?;
    
    if let Some(category) = category_filter {
        error::print_success(&format!("Reset configuration for category {} to defaults", category));
    } else {
        error::print_success("Reset all configuration to defaults");
    }
    
    Ok(())
}

fn export_config(api: &VRCoreAPI, file_path: &Path, format: &str, category_filter: Option<&str>) -> Result<()> {
    // Get all categories
    let categories = [
        ConfigCategory::Hardware,
        ConfigCategory::Display,
        ConfigCategory::Audio,
        ConfigCategory::Tracking,
        ConfigCategory::Network,
        ConfigCategory::Power,
        ConfigCategory::SteamVR,
        ConfigCategory::Security,
        ConfigCategory::System,
    ];
    
    // Filter categories if requested
    let filtered_categories: Vec<&ConfigCategory> = if let Some(filter) = category_filter {
        categories.iter()
            .filter(|c| c.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        categories.iter().collect()
    };
    
    if filtered_categories.is_empty() {
        return Err(anyhow!("Invalid category filter: {}", category_filter.unwrap_or("")));
    }
    
    // Prepare data for export
    let mut json_data = serde_json::Map::new();
    let mut toml_data = HashMap::new();
    
    for category in filtered_categories {
        let category_str = category.to_string();
        let keys = get_keys_for_category(*category);
        
        let mut category_json = serde_json::Map::new();
        let mut category_toml = HashMap::new();
        
        for key in keys {
            match api.config().get(*category, key) {
                Ok(value) => {
                    // Add to JSON data
                    category_json.insert(key.to_string(), config_value_to_json(&value));
                    
                    // Add to TOML data
                    category_toml.insert(key.to_string(), config_value_to_toml(&value));
                },
                Err(_) => {
                    // Key not found, skip
                }
            }
        }
        
        // Add category data to JSON and TOML
        json_data.insert(category_str.clone(), serde_json::Value::Object(category_json));
        toml_data.insert(category_str.clone(), category_toml);
    }
    
    // Export based on format
    match format.to_lowercase().as_str() {
        "json" => {
            let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(json_data))
                .context("Failed to format JSON")?;
            file::write_file(file_path, &json_str)?;
        },
        "toml" => {
            let toml_str = toml::to_string(&toml_data)
                .context("Failed to format TOML")?;
            file::write_file(file_path, &toml_str)?;
        },
        _ => {
            return Err(anyhow!("Unsupported export format: {}", format));
        }
    }
    
    if let Some(category) = category_filter {
        error::print_success(&format!("Exported configuration for category {} to {}", 
                                     category, file_path.display()));
    } else {
        error::print_success(&format!("Exported all configuration to {}", file_path.display()));
    }
    
    Ok(())
}

fn import_config(api: &mut VRCoreAPI, file_path: &Path, format: &str, force: bool) -> Result<()> {
    // Validate file exists
    validation::validate_file_exists(file_path)?;
    
    // Read file content
    let content = file::read_file(file_path)?;
    
    // Parse content based on format
    let mut changes = Vec::new();
    
    match format.to_lowercase().as_str() {
        "json" => {
            let json: serde_json::Value = serde_json::from_str(&content)
                .context("Failed to parse JSON")?;
            
            if let serde_json::Value::Object(categories) = json {
                for (category_str, category_data) in categories {
                    let category = parse_category(&category_str)?;
                    
                    if let serde_json::Value::Object(keys) = category_data {
                        for (key, value) in keys {
                            let config_value = json_to_config_value(&value)?;
                            changes.push((category, key.clone(), config_value));
                        }
                    }
                }
            }
        },
        "toml" => {
            let toml_value: toml::Value = toml::from_str(&content)
                .context("Failed to parse TOML")?;
            
            if let toml::Value::Table(categories) = toml_value {
                for (category_str, category_data) in categories {
                    let category = parse_category(&category_str)?;
                    
                    if let toml::Value::Table(keys) = category_data {
                        for (key, value) in keys {
                            let config_value = toml_to_config_value(&value)?;
                            changes.push((category, key.clone(), config_value));
                        }
                    }
                }
            }
        },
        _ => {
            return Err(anyhow!("Unsupported import format: {}", format));
        }
    }
    
    // Confirm import if not forced
    if !force && !changes.is_empty() {
        println!("The following changes will be made:");
        for (category, key, value) in &changes {
            println!("  {}.{} = {} ({})", 
                     category.to_string(),
                     key,
                     format_config_value(value),
                     get_type_name(value));
        }
        
        if !error::confirm("Are you sure you want to import these changes?", false)? {
            error::print_info("Import cancelled");
            return Ok(());
        }
    }
    
    // Apply changes
    for (category, key, value) in changes {
        api.config_mut().set(category, &key, value)?;
    }
    
    // Save configuration
    api.config_mut().save()?;
    
    error::print_success(&format!("Imported configuration from {}", file_path.display()));
    
    Ok(())
}

fn compare_config(api: &VRCoreAPI, file_path: Option<&Path>, category_filter: Option<&str>, format: &str) -> Result<()> {
    // Get all categories
    let categories = [
        ConfigCategory::Hardware,
        ConfigCategory::Display,
        ConfigCategory::Audio,
        ConfigCategory::Tracking,
        ConfigCategory::Network,
        ConfigCategory::Power,
        ConfigCategory::SteamVR,
        ConfigCategory::Security,
        ConfigCategory::System,
    ];
    
    // Filter categories if requested
    let filtered_categories: Vec<&ConfigCategory> = if let Some(filter) = category_filter {
        categories.iter()
            .filter(|c| c.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        categories.iter().collect()
    };
    
    if filtered_categories.is_empty() {
        return Err(anyhow!("Invalid category filter: {}", category_filter.unwrap_or("")));
    }
    
    // Get comparison config
    let comparison_config = if let Some(path) = file_path {
        // Validate file exists
        validation::validate_file_exists(path)?;
        
        // Read file content
        let content = file::read_file(path)?;
        
        // Determine format from file extension if not specified
        let file_format = if path.extension().map_or(false, |ext| ext == "json") {
            "json"
        } else if path.extension().map_or(false, |ext| ext == "toml") {
            "toml"
        } else {
            return Err(anyhow!("Unsupported file format: {}", path.display()));
        };
        
        // Parse content based on format
        match file_format {
            "json" => {
                let json: serde_json::Value = serde_json::from_str(&content)
                    .context("Failed to parse JSON")?;
                
                let mut config = vr_core_api::config::Config::default();
                
                if let serde_json::Value::Object(categories) = json {
                    for (category_str, category_data) in categories {
                        let category = parse_category(&category_str)?;
                        
                        if let serde_json::Value::Object(keys) = category_data {
                            for (key, value) in keys {
                                let config_value = json_to_config_value(&value)?;
                                config.set(category, &key, config_value)?;
                            }
                        }
                    }
                }
                
                config
            },
            "toml" => {
                let toml_value: toml::Value = toml::from_str(&content)
                    .context("Failed to parse TOML")?;
                
                let mut config = vr_core_api::config::Config::default();
                
                if let toml::Value::Table(categories) = toml_value {
                    for (category_str, category_data) in categories {
                        let category = parse_category(&category_str)?;
                        
                        if let toml::Value::Table(keys) = category_data {
                            for (key, value) in keys {
                                let config_value = toml_to_config_value(&value)?;
                                config.set(category, &key, config_value)?;
                            }
                        }
                    }
                }
                
                config
            },
            _ => {
                return Err(anyhow!("Unsupported file format: {}", file_format));
            }
        }
    } else {
        // Compare with defaults
        vr_core_api::config::Config::default()
    };
    
    // Compare configurations
    let mut differences = Vec::new();
    let mut json_data = serde_json::Map::new();
    
    for category in filtered_categories {
        let category_str = category.to_string();
        let keys = get_keys_for_category(*category);
        
        let mut category_json = serde_json::Map::new();
        
        for key in keys {
            let current_value = api.config().get(*category, key);
            let comparison_value = comparison_config.get(*category, key);
            
            match (current_value, comparison_value) {
                (Ok(current), Ok(comparison)) => {
                    if current != comparison {
                        // Add to differences
                        differences.push(vec![
                            category_str.clone(),
                            key.to_string(),
                            format_config_value(&current),
                            format_config_value(&comparison),
                        ]);
                        
                        // Add to JSON data
                        let mut diff_json = serde_json::Map::new();
                        diff_json.insert("current".to_string(), config_value_to_json(&current));
                        diff_json.insert("comparison".to_string(), config_value_to_json(&comparison));
                        category_json.insert(key.to_string(), serde_json::Value::Object(diff_json));
                    }
                },
                (Ok(current), Err(_)) => {
                    // Key exists in current but not in comparison
                    differences.push(vec![
                        category_str.clone(),
                        key.to_string(),
                        format_config_value(&current),
                        "Not set".to_string(),
                    ]);
                    
                    // Add to JSON data
                    let mut diff_json = serde_json::Map::new();
                    diff_json.insert("current".to_string(), config_value_to_json(&current));
                    diff_json.insert("comparison".to_string(), serde_json::Value::Null);
                    category_json.insert(key.to_string(), serde_json::Value::Object(diff_json));
                },
                (Err(_), Ok(comparison)) => {
                    // Key exists in comparison but not in current
                    differences.push(vec![
                        category_str.clone(),
                        key.to_string(),
                        "Not set".to_string(),
                        format_config_value(&comparison),
                    ]);
                    
                    // Add to JSON data
                    let mut diff_json = serde_json::Map::new();
                    diff_json.insert("current".to_string(), serde_json::Value::Null);
                    diff_json.insert("comparison".to_string(), config_value_to_json(&comparison));
                    category_json.insert(key.to_string(), serde_json::Value::Object(diff_json));
                },
                (Err(_), Err(_)) => {
                    // Key doesn't exist in either config, skip
                }
            }
        }
        
        // Add category data to JSON
        if !category_json.is_empty() {
            json_data.insert(category_str.clone(), serde_json::Value::Object(category_json));
        }
    }
    
    // Output based on format
    if differences.is_empty() {
        error::print_success("No differences found");
        return Ok(());
    }
    
    error::print_section("Configuration Differences");
    
    match format.to_lowercase().as_str() {
        "table" => {
            let headers = ["Category", "Key", "Current Value", "Comparison Value"];
            println!("{}", formatting::format_table(&headers, &differences));
        },
        "json" => {
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(json_data))
                .context("Failed to format JSON")?);
        },
        "text" => {
            for diff in &differences {
                println!("{}.{}: {} -> {}", 
                         diff[0].green().bold(),
                         diff[1].blue().bold(),
                         diff[2].yellow(),
                         diff[3].cyan());
            }
        },
        _ => {
            return Err(anyhow!("Unsupported output format: {}", format));
        }
    }
    
    Ok(())
}

fn search_config(api: &VRCoreAPI, term: &str, category_filter: Option<&str>, keys_only: bool, values_only: bool) -> Result<()> {
    // Get all categories
    let categories = [
        ConfigCategory::Hardware,
        ConfigCategory::Display,
        ConfigCategory::Audio,
        ConfigCategory::Tracking,
        ConfigCategory::Network,
        ConfigCategory::Power,
        ConfigCategory::SteamVR,
        ConfigCategory::Security,
        ConfigCategory::System,
    ];
    
    // Filter categories if requested
    let filtered_categories: Vec<&ConfigCategory> = if let Some(filter) = category_filter {
        categories.iter()
            .filter(|c| c.to_string().eq_ignore_ascii_case(filter))
            .collect()
    } else {
        categories.iter().collect()
    };
    
    if filtered_categories.is_empty() {
        return Err(anyhow!("Invalid category filter: {}", category_filter.unwrap_or("")));
    }
    
    // Search for term
    let mut results = Vec::new();
    
    for category in filtered_categories {
        let category_str = category.to_string();
        let keys = get_keys_for_category(*category);
        
        for key in keys {
            // Check key if not values_only
            if !values_only && key.to_lowercase().contains(&term.to_lowercase()) {
                if let Ok(value) = api.config().get(*category, key) {
                    results.push(vec![
                        category_str.clone(),
                        key.to_string(),
                        format_config_value(&value),
                        "Key match".to_string(),
                    ]);
                }
                continue;
            }
            
            // Check value if not keys_only
            if !keys_only {
                if let Ok(value) = api.config().get(*category, key) {
                    let value_str = format_config_value(&value);
                    if value_str.to_lowercase().contains(&term.to_lowercase()) {
                        results.push(vec![
                            category_str.clone(),
                            key.to_string(),
                            value_str,
                            "Value match".to_string(),
                        ]);
                    }
                }
            }
        }
    }
    
    // Output results
    if results.is_empty() {
        error::print_info(&format!("No matches found for search term: {}", term));
        return Ok(());
    }
    
    error::print_section(&format!("Search Results for '{}'", term));
    
    let headers = ["Category", "Key", "Value", "Match Type"];
    println!("{}", formatting::format_table(&headers, &results));
    
    Ok(())
}

// Helper functions

fn parse_category(category: &str) -> Result<ConfigCategory> {
    match category.to_lowercase().as_str() {
        "hardware" => Ok(ConfigCategory::Hardware),
        "display" => Ok(ConfigCategory::Display),
        "audio" => Ok(ConfigCategory::Audio),
        "tracking" => Ok(ConfigCategory::Tracking),
        "network" => Ok(ConfigCategory::Network),
        "power" => Ok(ConfigCategory::Power),
        "steamvr" => Ok(ConfigCategory::SteamVR),
        "security" => Ok(ConfigCategory::Security),
        "system" => Ok(ConfigCategory::System),
        _ => Err(anyhow!("Invalid configuration category: {}", category)),
    }
}

fn format_config_value(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(s) => s.clone(),
        ConfigValue::Integer(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Boolean(b) => b.to_string(),
        ConfigValue::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .map(|v| format_config_value(v))
                .collect();
            format!("[{}]", items.join(", "))
        },
        ConfigValue::Table(table) => {
            let items: Vec<String> = table.iter()
                .map(|(k, v)| format!("{}: {}", k, format_config_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        },
    }
}

fn get_type_name(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(_) => "string".to_string(),
        ConfigValue::Integer(_) => "integer".to_string(),
        ConfigValue::Float(_) => "float".to_string(),
        ConfigValue::Boolean(_) => "boolean".to_string(),
        ConfigValue::Array(_) => "array".to_string(),
        ConfigValue::Table(_) => "table".to_string(),
    }
}

fn get_keys_for_category(category: ConfigCategory) -> &'static [&'static str] {
    match category {
        ConfigCategory::Hardware => &["board_type", "memory_size", "cpu_cores", "gpu_type"],
        ConfigCategory::Display => &["refresh_rate", "persistence", "brightness", "contrast", "resolution"],
        ConfigCategory::Audio => &["volume", "mic_gain", "spatial_audio", "noise_cancellation", "sample_rate"],
        ConfigCategory::Tracking => &["camera_fps", "imu_rate", "prediction_ms", "tracking_quality", "boundary_visible"],
        ConfigCategory::Network => &["wifi_enabled", "latency_optimization", "bandwidth_limit", "connection_type"],
        ConfigCategory::Power => &["profile", "cpu_governor", "battery_saver", "auto_sleep_timeout"],
        ConfigCategory::SteamVR => &["enabled", "driver_path", "render_resolution", "supersampling"],
        ConfigCategory::Security => &["auth_required", "encryption", "pin_code", "auto_lock"],
        ConfigCategory::System => &["log_level", "auto_update", "telemetry_enabled", "developer_mode"],
    }
}

fn config_value_to_json(value: &ConfigValue) -> serde_json::Value {
    match value {
        ConfigValue::String(s) => serde_json::Value::String(s.clone()),
        ConfigValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        ConfigValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        },
        ConfigValue::Boolean(b) => serde_json::Value::Bool(*b),
        ConfigValue::Array(arr) => {
            let items: Vec<serde_json::Value> = arr.iter()
                .map(|v| config_value_to_json(v))
                .collect();
            serde_json::Value::Array(items)
        },
        ConfigValue::Table(table) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in table {
                obj.insert(k.clone(), config_value_to_json(v));
            }
            serde_json::Value::Object(obj)
        },
    }
}

fn json_to_config_value(value: &serde_json::Value) -> Result<ConfigValue> {
    match value {
        serde_json::Value::String(s) => Ok(ConfigValue::String(s.clone())),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(ConfigValue::Integer(n.as_i64().unwrap()))
            } else if n.is_f64() {
                Ok(ConfigValue::Float(n.as_f64().unwrap()))
            } else {
                Err(anyhow!("Unsupported number type"))
            }
        },
        serde_json::Value::Bool(b) => Ok(ConfigValue::Boolean(*b)),
        serde_json::Value::Array(arr) => {
            let mut items = Vec::new();
            for v in arr {
                items.push(json_to_config_value(v)?);
            }
            Ok(ConfigValue::Array(items))
        },
        serde_json::Value::Object(obj) => {
            let mut table = HashMap::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_config_value(v)?);
            }
            Ok(ConfigValue::Table(table))
        },
        serde_json::Value::Null => Err(anyhow!("Cannot convert null to ConfigValue")),
    }
}

fn config_value_to_toml(value: &ConfigValue) -> toml::Value {
    match value {
        ConfigValue::String(s) => toml::Value::String(s.clone()),
        ConfigValue::Integer(i) => toml::Value::Integer(*i),
        ConfigValue::Float(f) => toml::Value::Float(*f),
        ConfigValue::Boolean(b) => toml::Value::Boolean(*b),
        ConfigValue::Array(arr) => {
            let items: Vec<toml::Value> = arr.iter()
                .map(|v| config_value_to_toml(v))
                .collect();
            toml::Value::Array(items)
        },
        ConfigValue::Table(table) => {
            let mut toml_table = toml::value::Table::new();
            for (k, v) in table {
                toml_table.insert(k.clone(), config_value_to_toml(v));
            }
            toml::Value::Table(toml_table)
        },
    }
}

fn toml_to_config_value(value: &toml::Value) -> Result<ConfigValue> {
    match value {
        toml::Value::String(s) => Ok(ConfigValue::String(s.clone())),
        toml::Value::Integer(i) => Ok(ConfigValue::Integer(*i)),
        toml::Value::Float(f) => Ok(ConfigValue::Float(*f)),
        toml::Value::Boolean(b) => Ok(ConfigValue::Boolean(*b)),
        toml::Value::Array(arr) => {
            let mut items = Vec::new();
            for v in arr {
                items.push(toml_to_config_value(v)?);
            }
            Ok(ConfigValue::Array(items))
        },
        toml::Value::Table(table) => {
            let mut config_table = HashMap::new();
            for (k, v) in table {
                config_table.insert(k.clone(), toml_to_config_value(v)?);
            }
            Ok(ConfigValue::Table(config_table))
        },
        toml::Value::Datetime(dt) => Ok(ConfigValue::String(dt.to_string())),
    }
}
