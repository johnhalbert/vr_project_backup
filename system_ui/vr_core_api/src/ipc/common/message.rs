//! Message definitions for IPC mechanisms.
//!
//! This module provides message definitions for IPC mechanisms,
//! including message types, payloads, flags, and handler traits.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::{IPCError, Result};

/// IPC message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IPCMessage {
    /// Message ID
    pub id: String,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Source component
    pub source: String,
    
    /// Destination component
    pub destination: String,
    
    /// Message payload
    pub payload: MessagePayload,
    
    /// Message timestamp
    pub timestamp: u64,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Message flags
    pub flags: MessageFlags,
}

impl IPCMessage {
    /// Create a new IPCMessage.
    pub fn new(
        message_type: MessageType,
        source: &str,
        destination: &str,
        payload: MessagePayload,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id,
            message_type,
            source: source.to_string(),
            destination: destination.to_string(),
            payload,
            timestamp,
            auth_token: None,
            flags: MessageFlags::default(),
        }
    }
    
    /// Create a new request message.
    pub fn request(source: &str, destination: &str, payload: MessagePayload) -> Self {
        let mut message = Self::new(MessageType::Request, source, destination, payload);
        message.flags.requires_response = true;
        message
    }
    
    /// Create a new response message.
    pub fn response(request: &IPCMessage, payload: MessagePayload) -> Self {
        let mut message = Self::new(
            MessageType::Response,
            &request.destination,
            &request.source,
            payload,
        );
        message.flags.requires_response = false;
        message
    }
    
    /// Create a new notification message.
    pub fn notification(source: &str, destination: &str, payload: MessagePayload) -> Self {
        let mut message = Self::new(MessageType::Notification, source, destination, payload);
        message.flags.requires_response = false;
        message
    }
    
    /// Create a new error message.
    pub fn error(request: &IPCMessage, error: &str) -> Self {
        let mut message = Self::new(
            MessageType::Error,
            &request.destination,
            &request.source,
            MessagePayload::String(error.to_string()),
        );
        message.flags.requires_response = false;
        message
    }
    
    /// Set authentication token.
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }
    
    /// Set message flags.
    pub fn with_flags(mut self, flags: MessageFlags) -> Self {
        self.flags = flags;
        self
    }
    
    /// Set message priority.
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.flags.priority = priority;
        self
    }
    
    /// Set message timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.flags.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// Set message encryption.
    pub fn with_encryption(mut self, encrypted: bool) -> Self {
        self.flags.encrypted = encrypted;
        self
    }
    
    /// Set message compression.
    pub fn with_compression(mut self, compressed: bool) -> Self {
        self.flags.compressed = compressed;
        self
    }
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Request message
    Request,
    
    /// Response message
    Response,
    
    /// Notification message
    Notification,
    
    /// Error message
    Error,
    
    /// Heartbeat message
    Heartbeat,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Request => write!(f, "Request"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Notification => write!(f, "Notification"),
            MessageType::Error => write!(f, "Error"),
            MessageType::Heartbeat => write!(f, "Heartbeat"),
        }
    }
}

/// Message payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Empty payload
    Empty,
    
    /// String payload
    String(String),
    
    /// Binary payload
    Binary(Vec<u8>),
    
    /// JSON payload
    Json(serde_json::Value),
    
    /// TOML payload
    Toml(String),
}

impl MessagePayload {
    /// Create a new empty payload.
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Create a new string payload.
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }
    
    /// Create a new binary payload.
    pub fn binary<B: Into<Vec<u8>>>(b: B) -> Self {
        Self::Binary(b.into())
    }
    
    /// Create a new JSON payload.
    pub fn json<T: Serialize>(value: &T) -> Result<Self> {
        let json = serde_json::to_value(value)
            .map_err(|e| IPCError::SerializationError(e.to_string()))?;
        Ok(Self::Json(json))
    }
    
    /// Create a new TOML payload.
    pub fn toml<T: Serialize>(value: &T) -> Result<Self> {
        let toml = toml::to_string(value)
            .map_err(|e| IPCError::SerializationError(e.to_string()))?;
        Ok(Self::Toml(toml))
    }
    
    /// Get string payload.
    pub fn as_string(&self) -> Result<&str> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err(IPCError::MessageError("Payload is not a string".to_string())),
        }
    }
    
    /// Get binary payload.
    pub fn as_binary(&self) -> Result<&[u8]> {
        match self {
            Self::Binary(b) => Ok(b),
            _ => Err(IPCError::MessageError("Payload is not binary".to_string())),
        }
    }
    
    /// Get JSON payload.
    pub fn as_json(&self) -> Result<&serde_json::Value> {
        match self {
            Self::Json(j) => Ok(j),
            _ => Err(IPCError::MessageError("Payload is not JSON".to_string())),
        }
    }
    
    /// Get TOML payload.
    pub fn as_toml(&self) -> Result<&str> {
        match self {
            Self::Toml(t) => Ok(t),
            _ => Err(IPCError::MessageError("Payload is not TOML".to_string())),
        }
    }
    
    /// Parse JSON payload into a type.
    pub fn parse_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        match self {
            Self::Json(j) => {
                serde_json::from_value(j.clone())
                    .map_err(|e| IPCError::SerializationError(e.to_string()))
            }
            Self::String(s) => {
                serde_json::from_str(s)
                    .map_err(|e| IPCError::SerializationError(e.to_string()))
            }
            _ => Err(IPCError::MessageError("Payload cannot be parsed as JSON".to_string())),
        }
    }
    
    /// Parse TOML payload into a type.
    pub fn parse_toml<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        match self {
            Self::Toml(t) => {
                toml::from_str(t)
                    .map_err(|e| IPCError::SerializationError(e.to_string()))
            }
            Self::String(s) => {
                toml::from_str(s)
                    .map_err(|e| IPCError::SerializationError(e.to_string()))
            }
            _ => Err(IPCError::MessageError("Payload cannot be parsed as TOML".to_string())),
        }
    }
}

/// Message flags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageFlags {
    /// Whether the message requires a response
    pub requires_response: bool,
    
    /// Whether the message is encrypted
    pub encrypted: bool,
    
    /// Whether the message is compressed
    pub compressed: bool,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// Message timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl Default for MessageFlags {
    fn default() -> Self {
        Self {
            requires_response: false,
            encrypted: false,
            compressed: false,
            priority: MessagePriority::Normal,
            timeout_ms: None,
        }
    }
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority
    Low,
    
    /// Normal priority
    Normal,
    
    /// High priority
    High,
    
    /// Critical priority
    Critical,
}

impl fmt::Display for MessagePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessagePriority::Low => write!(f, "Low"),
            MessagePriority::Normal => write!(f, "Normal"),
            MessagePriority::High => write!(f, "High"),
            MessagePriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Message handler trait
pub trait MessageHandler: Send + Sync {
    /// Handle message
    fn handle_message(&self, message: IPCMessage) -> Result<Option<IPCMessage>>;
    
    /// Get handler ID
    fn id(&self) -> &str;
    
    /// Get supported message types
    fn supported_message_types(&self) -> Vec<MessageType>;
    
    /// Clone handler
    fn clone_box(&self) -> Box<dyn MessageHandler>;
}

impl Clone for Box<dyn MessageHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let message = IPCMessage::new(
            MessageType::Request,
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        assert_eq!(message.message_type, MessageType::Request);
        assert_eq!(message.source, "source");
        assert_eq!(message.destination, "destination");
        assert_eq!(message.payload, MessagePayload::String("test".to_string()));
        assert_eq!(message.flags.requires_response, false);
    }
    
    #[test]
    fn test_request_message() {
        let message = IPCMessage::request(
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        assert_eq!(message.message_type, MessageType::Request);
        assert_eq!(message.flags.requires_response, true);
    }
    
    #[test]
    fn test_response_message() {
        let request = IPCMessage::request(
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        let response = IPCMessage::response(
            &request,
            MessagePayload::String("response".to_string()),
        );
        
        assert_eq!(response.message_type, MessageType::Response);
        assert_eq!(response.source, "destination");
        assert_eq!(response.destination, "source");
        assert_eq!(response.payload, MessagePayload::String("response".to_string()));
        assert_eq!(response.flags.requires_response, false);
    }
    
    #[test]
    fn test_notification_message() {
        let message = IPCMessage::notification(
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        assert_eq!(message.message_type, MessageType::Notification);
        assert_eq!(message.flags.requires_response, false);
    }
    
    #[test]
    fn test_error_message() {
        let request = IPCMessage::request(
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        let error = IPCMessage::error(&request, "error message");
        
        assert_eq!(error.message_type, MessageType::Error);
        assert_eq!(error.source, "destination");
        assert_eq!(error.destination, "source");
        assert_eq!(error.payload, MessagePayload::String("error message".to_string()));
        assert_eq!(error.flags.requires_response, false);
    }
    
    #[test]
    fn test_message_with_auth_token() {
        let message = IPCMessage::new(
            MessageType::Request,
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        ).with_auth_token("token123");
        
        assert_eq!(message.auth_token, Some("token123".to_string()));
    }
    
    #[test]
    fn test_message_with_flags() {
        let flags = MessageFlags {
            requires_response: true,
            encrypted: true,
            compressed: true,
            priority: MessagePriority::High,
            timeout_ms: Some(1000),
        };
        
        let message = IPCMessage::new(
            MessageType::Request,
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        ).with_flags(flags.clone());
        
        assert_eq!(message.flags, flags);
    }
    
    #[test]
    fn test_message_payload_json() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            field1: String,
            field2: i32,
        }
        
        let test_struct = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };
        
        let payload = MessagePayload::json(&test_struct).unwrap();
        let parsed: TestStruct = payload.parse_json().unwrap();
        
        assert_eq!(parsed, test_struct);
    }
    
    #[test]
    fn test_message_payload_toml() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            field1: String,
            field2: i32,
        }
        
        let test_struct = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };
        
        let payload = MessagePayload::toml(&test_struct).unwrap();
        let parsed: TestStruct = payload.parse_toml().unwrap();
        
        assert_eq!(parsed, test_struct);
    }
}
