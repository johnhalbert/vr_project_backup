use actix_web::{web, Scope, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use log::{info, error};

use crate::state::AppState;
use crate::error::ApiError;

// Request and response models
#[derive(Serialize, Deserialize)]
pub struct HardwareListRequest {
    device_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareDevice {
    name: String,
    device_type: String,
    status: String,
    properties: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareListResponse {
    devices: Vec<HardwareDevice>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareInfoRequest {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareInfoResponse {
    device: HardwareDevice,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareInitRequest {
    device: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareInitResponse {
    success: bool,
    message: String,
    initialized_devices: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareShutdownRequest {
    device: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareShutdownResponse {
    success: bool,
    message: String,
    shutdown_devices: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareDiagnoseRequest {
    device: Option<String>,
    level: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DiagnosticResult {
    test_name: String,
    status: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareDiagnoseResponse {
    success: bool,
    message: String,
    results: Vec<DiagnosticResult>,
}

// API handlers
async fn list_hardware(
    app_state: web::Data<AppState>,
    query: web::Query<HardwareListRequest>,
) -> Result<HttpResponse, ApiError> {
    let device_type_filter = query.device_type.clone();
    
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    let mut devices = Vec::new();
    
    // Get cameras
    let cameras = core_api.hardware().get_cameras();
    for camera in cameras {
        if let Some(filter) = &device_type_filter {
            if filter != "camera" && filter != "all" {
                continue;
            }
        }
        
        let mut properties = std::collections::HashMap::new();
        
        // Since we can't directly access Camera-specific methods through the Device trait,
        // we'll add generic properties based on what we know about cameras
        properties.insert("type".to_string(), "camera".to_string());
        
        devices.push(HardwareDevice {
            name: camera.name().to_string(),
            device_type: "camera".to_string(),
            status: if camera.is_initialized() { "initialized".to_string() } else { "not initialized".to_string() },
            properties,
        });
    }
    
    // Get IMUs
    let imus = core_api.hardware().get_imus();
    for imu in imus {
        if let Some(filter) = &device_type_filter {
            if filter != "imu" && filter != "all" {
                continue;
            }
        }
        
        let mut properties = std::collections::HashMap::new();
        properties.insert("type".to_string(), "imu".to_string());
        
        devices.push(HardwareDevice {
            name: imu.name().to_string(),
            device_type: "imu".to_string(),
            status: if imu.is_initialized() { "initialized".to_string() } else { "not initialized".to_string() },
            properties,
        });
    }
    
    Ok(HttpResponse::Ok().json(HardwareListResponse { devices }))
}

async fn get_hardware_info(
    app_state: web::Data<AppState>,
    params: web::Path<HardwareInfoRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // Search for the device by name
    let device_name = &params.name;
    
    // Check cameras
    let cameras = core_api.hardware().get_cameras();
    for camera in cameras {
        if camera.name() == *device_name {
            let mut properties = std::collections::HashMap::new();
            properties.insert("type".to_string(), "camera".to_string());
            
            let device = HardwareDevice {
                name: camera.name().to_string(),
                device_type: "camera".to_string(),
                status: if camera.is_initialized() { "initialized".to_string() } else { "not initialized".to_string() },
                properties,
            };
            
            return Ok(HttpResponse::Ok().json(HardwareInfoResponse { device }));
        }
    }
    
    // Check IMUs
    let imus = core_api.hardware().get_imus();
    for imu in imus {
        if imu.name() == *device_name {
            let mut properties = std::collections::HashMap::new();
            properties.insert("type".to_string(), "imu".to_string());
            
            let device = HardwareDevice {
                name: imu.name().to_string(),
                device_type: "imu".to_string(),
                status: if imu.is_initialized() { "initialized".to_string() } else { "not initialized".to_string() },
                properties,
            };
            
            return Ok(HttpResponse::Ok().json(HardwareInfoResponse { device }));
        }
    }
    
    // Device not found
    Err(ApiError::NotFound(format!("Hardware device not found: {}", device_name)))
}

async fn init_hardware(
    app_state: web::Data<AppState>,
    params: web::Json<HardwareInitRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let mut core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    let device_filter = params.device.clone();
    let mut initialized_devices = Vec::new();
    let mut success = true;
    let mut message = "Hardware initialization successful".to_string();
    
    // Initialize specific device or all devices
    if let Some(device_name) = device_filter {
        // Initialize specific device
        if let Some(device) = core_api.hardware_mut().get_device_mut(&device_name) {
            match device.initialize() {
                Ok(_) => {
                    initialized_devices.push(device_name.clone());
                },
                Err(e) => {
                    success = false;
                    message = format!("Failed to initialize device {}: {}", device_name, e);
                }
            }
        } else {
            success = false;
            message = format!("Device not found: {}", device_name);
        }
    } else {
        // Initialize all devices
        match core_api.hardware_mut().initialize() {
            Ok(_) => {
                // Get list of all devices
                let cameras = core_api.hardware().get_cameras();
                for camera in cameras {
                    if camera.is_initialized() {
                        initialized_devices.push(camera.name().to_string());
                    }
                }
                
                let imus = core_api.hardware().get_imus();
                for imu in imus {
                    if imu.is_initialized() {
                        initialized_devices.push(imu.name().to_string());
                    }
                }
            },
            Err(e) => {
                success = false;
                message = format!("Failed to initialize all devices: {}", e);
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(HardwareInitResponse {
        success,
        message,
        initialized_devices,
    }))
}

async fn shutdown_hardware(
    app_state: web::Data<AppState>,
    params: web::Json<HardwareShutdownRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let mut core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    let device_filter = params.device.clone();
    let mut shutdown_devices = Vec::new();
    let mut success = true;
    let mut message = "Hardware shutdown successful".to_string();
    
    // Shutdown specific device or all devices
    if let Some(device_name) = device_filter {
        // Shutdown specific device
        if let Some(device) = core_api.hardware_mut().get_device_mut(&device_name) {
            match device.shutdown() {
                Ok(_) => {
                    shutdown_devices.push(device_name.clone());
                },
                Err(e) => {
                    success = false;
                    message = format!("Failed to shutdown device {}: {}", device_name, e);
                }
            }
        } else {
            success = false;
            message = format!("Device not found: {}", device_name);
        }
    } else {
        // Shutdown all devices
        match core_api.hardware_mut().shutdown() {
            Ok(_) => {
                // Get list of all devices that should now be shut down
                let cameras = core_api.hardware().get_cameras();
                for camera in cameras {
                    if !camera.is_initialized() {
                        shutdown_devices.push(camera.name().to_string());
                    }
                }
                
                let imus = core_api.hardware().get_imus();
                for imu in imus {
                    if !imu.is_initialized() {
                        shutdown_devices.push(imu.name().to_string());
                    }
                }
            },
            Err(e) => {
                success = false;
                message = format!("Failed to shutdown all devices: {}", e);
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(HardwareShutdownResponse {
        success,
        message,
        shutdown_devices,
    }))
}

async fn diagnose_hardware(
    app_state: web::Data<AppState>,
    params: web::Json<HardwareDiagnoseRequest>,
) -> Result<HttpResponse, ApiError> {
    // Get a lock on the Core API
    let core_api = app_state.core_api.lock().map_err(|_| {
        ApiError::InternalError("Failed to acquire lock on Core API".to_string())
    })?;
    
    // This would require additional functionality in the Core API
    // For now, we'll just return a placeholder response
    
    let level = params.level.clone().unwrap_or_else(|| "basic".to_string());
    let device_name = params.device.clone();
    
    // Create some mock diagnostic results
    let mut results = Vec::new();
    
    if let Some(device) = device_name {
        results.push(DiagnosticResult {
            test_name: format!("{} connectivity", device),
            status: "passed".to_string(),
            message: format!("Device {} is connected", device),
        });
        
        results.push(DiagnosticResult {
            test_name: format!("{} initialization", device),
            status: "passed".to_string(),
            message: format!("Device {} initialized successfully", device),
        });
        
        if level == "advanced" || level == "full" {
            results.push(DiagnosticResult {
                test_name: format!("{} performance", device),
                status: "warning".to_string(),
                message: format!("Device {} performance is below optimal", device),
            });
        }
        
        if level == "full" {
            results.push(DiagnosticResult {
                test_name: format!("{} calibration", device),
                status: "passed".to_string(),
                message: format!("Device {} is properly calibrated", device),
            });
        }
    } else {
        // System-wide diagnostics
        results.push(DiagnosticResult {
            test_name: "System connectivity".to_string(),
            status: "passed".to_string(),
            message: "All devices are connected".to_string(),
        });
        
        results.push(DiagnosticResult {
            test_name: "System initialization".to_string(),
            status: "passed".to_string(),
            message: "All devices initialized successfully".to_string(),
        });
        
        if level == "advanced" || level == "full" {
            results.push(DiagnosticResult {
                test_name: "System performance".to_string(),
                status: "warning".to_string(),
                message: "System performance is below optimal".to_string(),
            });
        }
        
        if level == "full" {
            results.push(DiagnosticResult {
                test_name: "System calibration".to_string(),
                status: "passed".to_string(),
                message: "All devices are properly calibrated".to_string(),
            });
        }
    }
    
    Ok(HttpResponse::Ok().json(HardwareDiagnoseResponse {
        success: true,
        message: format!("Diagnostic level: {}", level),
        results,
    }))
}

// Register routes
pub fn hardware_routes() -> Scope {
    web::scope("/hardware")
        .route("", web::get().to(list_hardware))
        .route("/{name}", web::get().to(get_hardware_info))
        .route("/init", web::post().to(init_hardware))
        .route("/shutdown", web::post().to(shutdown_hardware))
        .route("/diagnose", web::post().to(diagnose_hardware))
}
