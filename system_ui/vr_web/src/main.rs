use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use log::{info, error};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use anyhow::Result;

// Import our Core API
use vr_core_api::VRCoreAPI;

// API modules
mod api;
mod error;
mod state;

use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Initialize VR Core API
    let core_api = match VRCoreAPI::new() {
        Ok(api) => api,
        Err(e) => {
            error!("Failed to initialize VR Core API: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };
    
    // Create shared application state
    let app_state = web::Data::new(AppState {
        core_api: Mutex::new(core_api),
    });
    
    // Get the port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);
    
    info!("Starting VR Web Server on {}", bind_address);
    
    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            // API routes
            .service(
                web::scope("/api")
                    .service(api::config::config_routes())
                    .service(api::hardware::hardware_routes())
                    .service(api::system::system_routes())
                    .service(api::monitoring::monitoring_routes())
                    .service(api::ipc::ipc_routes())
                    .service(api::security::security_routes())
            )
            // Health check endpoint
            .route("/health", web::get().to(health_check))
    })
    .bind(bind_address)?
    .run()
    .await
}

// Simple health check endpoint
async fn health_check() -> impl Responder {
    #[derive(Serialize)]
    struct HealthResponse {
        status: String,
        version: String,
    }
    
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
