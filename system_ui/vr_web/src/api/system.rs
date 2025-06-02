use actix_web::{web, Scope, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use log::{info, error};

use crate::state::AppState;
use crate::error::ApiError;

// Request and response models
#[derive(Serialize, Deserialize)]
pub struct SystemStatusResponse {
    version: String,
    uptime: u64,
    components: Vec<SystemComponent>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemComponent {
    name: String,
    status: String,
    details: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfoResponse {
    version: String,
    board_type: String,
    memory_size: String,
    os_version: String,
    kernel_version: String,
    config_path: String,
    log_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SystemUpdateRequest {
    check_only: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SystemUpdate {
    component: String,
    current_version: String,
    available_version: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
pub struct SystemUpdateResponse {
    success: bool,
    message: String,
    available_updates: Option<Vec<SystemUpdate>>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemRestartRequest {
    force: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SystemRestartResponse {
    success: bool,
    message: String,
}

// API handlers
async fn get_system_status(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Get system status from Core API
    // For now, we'll return a placeholder response
    
    // Get camera and IMU counts
    let camera_count = core_api.hardware().get_cameras().len();
    let imu_count = core_api.hardware().get_imus().len();
    
    // Create components list
    let mut components = Vec::new();
    
    // Add hardware components
    components.push(SystemComponent {
        name: "Cameras".to_string(),
        status: if camera_count > 0 { "OK".to_string() } else { "Not Found".to_string() },
        details: Some(format!("{} cameras detected", camera_count)),
    });
    
    components.push(SystemComponent {
        name: "IMUs".to_string(),
        status: if imu_count > 0 { "OK".to_string() } else { "Not Found".to_string() },
        details: Some(format!("{} IMUs detected", imu_count)),
    });
    
    // Add software components
    components.push(SystemComponent {
        name: "SLAM Pipeline".to_string(),
        status: "OK".to_string(),
        details: None,
    });
    
    components.push(SystemComponent {
        name: "SteamVR Integration".to_string(),
        status: "Connected".to_string(),
        details: None,
    });
    
    // Create response
    let response = SystemStatusResponse {
        version: "1.0.0".to_string(),
        uptime: 3600, // 1 hour in seconds
        components,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

async fn get_system_info(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Get system info from Core API
    // For now, we'll return a placeholder response
    
    let response = SystemInfoResponse {
        version: "1.0.0".to_string(),
        board_type: "Orange Pi CM5".to_string(),
        memory_size: "16GB".to_string(),
        os_version: "Ubuntu 22.04 LTS".to_string(),
        kernel_version: "5.15.0-rt with PREEMPT_RT".to_string(),
        config_path: "/etc/vr_headset/config.toml".to_string(),
        log_path: "/var/log/vr_headset".to_string(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

async fn update_system(
    _app_state: web::Data<AppState>,
    params: web::Json<SystemUpdateRequest>,
) -> Result<HttpResponse, ApiError> {
    // This would require additional functionality in the Core API
    // For now, we'll just return a placeholder response
    
    let check_only = params.check_only;
    
    if check_only {
        // Just check for updates
        let available_updates = vec![
            SystemUpdate {
                component: "Core API".to_string(),
                current_version: "1.0.0".to_string(),
                available_version: "1.1.0".to_string(),
                description: "Performance improvements and bug fixes".to_string(),
            },
            SystemUpdate {
                component: "SLAM Pipeline".to_string(),
                current_version: "0.9.5".to_string(),
                available_version: "1.0.0".to_string(),
                description: "Stability improvements and new feature tracking algorithm".to_string(),
            },
        ];
        
        Ok(HttpResponse::Ok().json(SystemUpdateResponse {
            success: true,
            message: "Updates available".to_string(),
            available_updates: Some(available_updates),
        }))
    } else {
        // Install updates
        Ok(HttpResponse::Ok().json(SystemUpdateResponse {
            success: true,
            message: "Updates installed successfully. System will restart in 5 minutes.".to_string(),
            available_updates: None,
        }))
    }
}

async fn restart_system(
    _app_state: web::Data<AppState>,
    params: web::Json<SystemRestartRequest>,
) -> Result<HttpResponse, ApiError> {
    // This would require additional functionality in the Core API
    // For now, we'll just return a placeholder response
    
    let force = params.force;
    
    Ok(HttpResponse::Ok().json(SystemRestartResponse {
        success: true,
        message: if force {
            "System is restarting immediately".to_string()
        } else {
            "System will restart in 5 minutes".to_string()
        },
    }))
}

// Register routes
pub fn system_routes() -> Scope {
    web::scope("/system")
        .route("/status", web::get().to(get_system_status))
        .route("/info", web::get().to(get_system_info))
        .route("/update", web::post().to(update_system))
        .route("/restart", web::post().to(restart_system))
}
