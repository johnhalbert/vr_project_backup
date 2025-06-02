use std::fmt;
use actix_web::{HttpResponse, ResponseError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "bad_request".to_string(),
                    message: msg.clone(),
                })
            }
            ApiError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "unauthorized".to_string(),
                    message: msg.clone(),
                })
            }
            ApiError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(ErrorResponse {
                    error: "forbidden".to_string(),
                    message: msg.clone(),
                })
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "not_found".to_string(),
                    message: msg.clone(),
                })
            }
            ApiError::Conflict(msg) => {
                HttpResponse::Conflict().json(ErrorResponse {
                    error: "conflict".to_string(),
                    message: msg.clone(),
                })
            }
            ApiError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "internal_server_error".to_string(),
                    message: msg.clone(),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}
