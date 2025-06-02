use actix_web::{web, Scope, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use log::{info, error};

use crate::state::AppState;
use crate::error::ApiError;

// Request and response models
#[derive(Serialize, Deserialize)]
pub struct ConfigListRequest {
    category: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigItem {
    category: String,
    key: String,
    value: String,
    value_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigListResponse {
    items: Vec<ConfigItem>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigGetRequest {
    category: String,
    key: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigGetResponse {
    category: String,
    key: String,
    value: String,
    value_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigSetRequest {
    category: String,
    key: String,
    value: String,
    value_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigSetResponse {
    category: String,
    key: String,
    value: String,
    value_type: String,
    success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigResetRequest {
    category: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigResetResponse {
    success: bool,
    message: String,
}

// API handlers
async fn list_config(
    app_state: web::Data<AppState>,
    query: web::Query<ConfigListRequest>,
) -> Result<HttpResponse, ApiError> {
    let category_filter = query.category.clone();
    
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Get all categories
    let categories = [
        vr_core_api::config::ConfigCategory::Hardware,
        vr_core_api::config::ConfigCategory::Display,
        vr_core_api::config::ConfigCategory::Audio,
        vr_core_api::config::ConfigCategory::Tracking,
        vr_core_api::config::ConfigCategory::Network,
        vr_core_api::config::ConfigCategory::Power,
        vr_core_api::config::ConfigCategory::SteamVR,
        vr_core_api::config::ConfigCategory::Security,
        vr_core_api::config::ConfigCategory::System,
    ];
    
    let mut items = Vec::new();
    
    for category in categories.iter() {
        let category_str = category.to_string();
        
        // Skip if not matching filter
        if let Some(filter) = &category_filter {
            if !category_str.eq_ignore_ascii_case(filter) {
                continue;
            }
        }
        
        // For each category, we'd need to know the keys
        // Since the Core API doesn't provide a way to list all keys in a category,
        // we'll use a predefined list of keys for each category
        let keys = get_keys_for_category(*category);
        
        for key in keys {
            match core_api.config().get(*category, key) {
                Ok(value) => {
                    items.push(ConfigItem {
                        category: category_str.clone(),
                        key: key.to_string(),
                        value: format_config_value(&value),
                        value_type: get_type_name(&value),
                    });
                },
                Err(_) => {
                    // Key not found, skip
                }
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(ConfigListResponse { items }))
}

async fn get_config(
    app_state: web::Data<AppState>,
    params: web::Path<ConfigGetRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Parse category
    let category = parse_category(&params.category)?;
    
    // Get the value
    match core_api.config().get(category, &params.key) {
        Ok(value) => {
            Ok(HttpResponse::Ok().json(ConfigGetResponse {
                category: category.to_string(),
                key: params.key.clone(),
                value: format_config_value(&value),
                value_type: get_type_name(&value),
            }))
        },
        Err(e) => {
            Err(ApiError::NotFound(format!("Configuration value not found: {}.{}: {}", 
                                         category.to_string(), params.key, e)))
        }
    }
}

async fn set_config(
    app_state: web::Data<AppState>,
    params: web::Json<ConfigSetRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let mut core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Parse category
    let category = parse_category(&params.category)?;
    
    // Parse value based on type
    let config_value = match params.value_type.to_lowercase().as_str() {
        "string" => vr_core_api::config::ConfigValue::String(params.value.clone()),
        "integer" => {
            let parsed = params.value.parse::<i64>()
                .map_err(|_| ApiError::BadRequest(format!("Invalid integer value: {}", params.value)))?;
            vr_core_api::config::ConfigValue::Integer(parsed)
        },
        "float" => {
            let parsed = params.value.parse::<f64>()
                .map_err(|_| ApiError::BadRequest(format!("Invalid float value: {}", params.value)))?;
            vr_core_api::config::ConfigValue::Float(parsed)
        },
        "boolean" => {
            let parsed = match params.value.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => true,
                "false" | "no" | "0" | "off" => false,
                _ => return Err(ApiError::BadRequest(format!("Invalid boolean value: {}", params.value))),
            };
            vr_core_api::config::ConfigValue::Boolean(parsed)
        },
        _ => return Err(ApiError::BadRequest(format!("Unsupported value type: {}", params.value_type))),
    };
    
    // Set the value
    match core_api.config_mut().set(category, &params.key, config_value.clone()) {
        Ok(_) => {
            // Save configuration
            match core_api.config_mut().save() {
                Ok(_) => {
                    Ok(HttpResponse::Ok().json(ConfigSetResponse {
                        category: category.to_string(),
                        key: params.key.clone(),
                        value: params.value.clone(),
                        value_type: params.value_type.clone(),
                        success: true,
                    }))
                },
                Err(e) => {
                    Err(ApiError::InternalError(format!("Failed to save configuration: {}", e)))
                }
            }
        },
        Err(e) => {
            Err(ApiError::InternalError(format!("Failed to set configuration value: {}", e)))
        }
    }
}

async fn reset_config(
    app_state: web::Data<AppState>,
    params: web::Json<ConfigResetRequest>,
) -> Result<HttpResponse, ApiError> {
    // This would require creating a new configuration with defaults
    // and then copying over the values to the current configuration
    // For now, we'll just return a placeholder response
    
    if let Some(category) = &params.category {
        Ok(HttpResponse::Ok().json(ConfigResetResponse {
            success: false,
            message: format!("Reset configuration for category {} is not implemented yet", category),
        }))
    } else {
        Ok(HttpResponse::Ok().json(ConfigResetResponse {
            success: false,
            message: "Reset all configuration is not implemented yet".to_string(),
        }))
    }
}

// Helper functions
fn parse_category(category: &str) -> Result<vr_core_api::config::ConfigCategory, ApiError> {
    match category.to_lowercase().as_str() {
        "hardware" => Ok(vr_core_api::config::ConfigCategory::Hardware),
        "display" => Ok(vr_core_api::config::ConfigCategory::Display),
        "audio" => Ok(vr_core_api::config::ConfigCategory::Audio),
        "tracking" => Ok(vr_core_api::config::ConfigCategory::Tracking),
        "network" => Ok(vr_core_api::config::ConfigCategory::Network),
        "power" => Ok(vr_core_api::config::ConfigCategory::Power),
        "steamvr" => Ok(vr_core_api::config::ConfigCategory::SteamVR),
        "security" => Ok(vr_core_api::config::ConfigCategory::Security),
        "system" => Ok(vr_core_api::config::ConfigCategory::System),
        _ => Err(ApiError::BadRequest(format!("Invalid configuration category: {}", category))),
    }
}

fn format_config_value(value: &vr_core_api::config::ConfigValue) -> String {
    match value {
        vr_core_api::config::ConfigValue::String(s) => s.clone(),
        vr_core_api::config::ConfigValue::Integer(i) => i.to_string(),
        vr_core_api::config::ConfigValue::Float(f) => f.to_string(),
        vr_core_api::config::ConfigValue::Boolean(b) => b.to_string(),
        vr_core_api::config::ConfigValue::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .map(|v| format_config_value(v))
                .collect();
            format!("[{}]", items.join(", "))
        },
        vr_core_api::config::ConfigValue::Table(table) => {
            let items: Vec<String> = table.iter()
                .map(|(k, v)| format!("{}: {}", k, format_config_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        },
    }
}

fn get_type_name(value: &vr_core_api::config::ConfigValue) -> String {
    match value {
        vr_core_api::config::ConfigValue::String(_) => "string".to_string(),
        vr_core_api::config::ConfigValue::Integer(_) => "integer".to_string(),
        vr_core_api::config::ConfigValue::Float(_) => "float".to_string(),
        vr_core_api::config::ConfigValue::Boolean(_) => "boolean".to_string(),
        vr_core_api::config::ConfigValue::Array(_) => "array".to_string(),
        vr_core_api::config::ConfigValue::Table(_) => "table".to_string(),
    }
}

fn get_keys_for_category(category: vr_core_api::config::ConfigCategory) -> &'static [&'static str] {
    match category {
        vr_core_api::config::ConfigCategory::Hardware => &["board_type", "memory_size"],
        vr_core_api::config::ConfigCategory::Display => &["refresh_rate", "persistence", "brightness"],
        vr_core_api::config::ConfigCategory::Audio => &["volume", "mic_gain", "spatial_audio"],
        vr_core_api::config::ConfigCategory::Tracking => &["camera_fps", "imu_rate", "prediction_ms"],
        vr_core_api::config::ConfigCategory::Network => &["wifi_enabled", "latency_optimization"],
        vr_core_api::config::ConfigCategory::Power => &["profile", "cpu_governor"],
        vr_core_api::config::ConfigCategory::SteamVR => &["enabled", "driver_path"],
        vr_core_api::config::ConfigCategory::Security => &["auth_required", "encryption"],
        vr_core_api::config::ConfigCategory::System => &["log_level", "auto_update"],
    }
}

// Register routes
pub fn config_routes() -> Scope {
    web::scope("/config")
        .route("", web::get().to(list_config))
        .route("/{category}/{key}", web::get().to(get_config))
        .route("", web::post().to(set_config))
        .route("/reset", web::post().to(reset_config))
}
