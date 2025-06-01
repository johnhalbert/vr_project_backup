//! Error handling for the OpenVR driver

use thiserror::Error;
use std::ffi::NulError;

/// Result type for OpenVR driver operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the OpenVR driver
#[derive(Error, Debug)]
pub enum Error {
    #[error("OpenVR initialization failed: {0}")]
    OpenVRInitFailed(i32),
    
    #[error("Device registration failed: {0}")]
    DeviceRegistrationFailed(String),
    
    #[error("Invalid device index: {0}")]
    InvalidDeviceIndex(u32),
    
    #[error("Core API error: {0}")]
    CoreAPIError(String),
    
    #[error("FFI error: {0}")]
    FFIError(String),
    
    #[error("String contains null byte")]
    NulError(#[from] NulError),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JSONError(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}
