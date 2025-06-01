//! Error definitions for IPC mechanisms.
//!
//! This module provides error definitions for IPC mechanisms,
//! including error types and result type.

use std::fmt;
use std::io;

/// IPC error
#[derive(Debug, thiserror::Error)]
pub enum IPCError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Message error
    #[error("Message error: {0}")]
    MessageError(String),
    
    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

/// Result type for IPC operations
pub type Result<T> = std::result::Result<T, IPCError>;

impl From<&str> for IPCError {
    fn from(s: &str) -> Self {
        IPCError::InternalError(s.to_string())
    }
}

impl From<String> for IPCError {
    fn from(s: String) -> Self {
        IPCError::InternalError(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_from_str() {
        let error: IPCError = "test error".into();
        match error {
            IPCError::InternalError(s) => assert_eq!(s, "test error"),
            _ => panic!("Expected InternalError"),
        }
    }
    
    #[test]
    fn test_error_from_string() {
        let error: IPCError = "test error".to_string().into();
        match error {
            IPCError::InternalError(s) => assert_eq!(s, "test error"),
            _ => panic!("Expected InternalError"),
        }
    }
    
    #[test]
    fn test_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: IPCError = io_error.into();
        match error {
            IPCError::IoError(_) => {},
            _ => panic!("Expected IoError"),
        }
    }
}
