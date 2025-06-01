//! WebSocket protocol definitions for IPC mechanisms.
//!
//! This module provides protocol definitions for WebSocket IPC,
//! including message formats and serialization.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ipc::common::{IPCError, Result, IPCMessage};

/// WebSocket protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketProtocol {
    /// JSON protocol
    Json,
    
    /// Binary protocol
    Binary,
}

impl fmt::Display for WebSocketProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketProtocol::Json => write!(f, "json"),
            WebSocketProtocol::Binary => write!(f, "binary"),
        }
    }
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Message ID
    pub id: String,
    
    /// Message type
    pub message_type: WebSocketMessageType,
    
    /// Message payload
    pub payload: String,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Timestamp
    pub timestamp: u64,
}

impl WebSocketMessage {
    /// Create a new WebSocketMessage
    pub fn new(message_type: WebSocketMessageType, payload: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            id,
            message_type,
            payload: payload.to_string(),
            auth_token: None,
            timestamp,
        }
    }
    
    /// Create a new request message
    pub fn request(payload: &str) -> Self {
        Self::new(WebSocketMessageType::Request, payload)
    }
    
    /// Create a new response message
    pub fn response(request_id: &str, payload: &str) -> Self {
        let mut message = Self::new(WebSocketMessageType::Response, payload);
        message.id = request_id.to_string();
        message
    }
    
    /// Create a new notification message
    pub fn notification(payload: &str) -> Self {
        Self::new(WebSocketMessageType::Notification, payload)
    }
    
    /// Create a new error message
    pub fn error(request_id: &str, error: &str) -> Self {
        let mut message = Self::new(WebSocketMessageType::Error, error);
        message.id = request_id.to_string();
        message
    }
    
    /// Create a new authentication message
    pub fn authentication(token: &str) -> Self {
        let mut message = Self::new(WebSocketMessageType::Authentication, "");
        message.auth_token = Some(token.to_string());
        message
    }
    
    /// Set authentication token
    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }
    
    /// Convert to IPC message
    pub fn to_ipc_message(&self, source: &str, destination: &str) -> Result<IPCMessage> {
        let message_type = match self.message_type {
            WebSocketMessageType::Request => crate::ipc::common::message::MessageType::Request,
            WebSocketMessageType::Response => crate::ipc::common::message::MessageType::Response,
            WebSocketMessageType::Notification => crate::ipc::common::message::MessageType::Notification,
            WebSocketMessageType::Error => crate::ipc::common::message::MessageType::Error,
            WebSocketMessageType::Authentication => crate::ipc::common::message::MessageType::Request,
            WebSocketMessageType::Heartbeat => crate::ipc::common::message::MessageType::Heartbeat,
        };
        
        let payload = crate::ipc::common::message::MessagePayload::String(self.payload.clone());
        
        let mut message = IPCMessage::new(
            message_type,
            source,
            destination,
            payload,
        );
        
        message.id = self.id.clone();
        message.timestamp = self.timestamp;
        message.auth_token = self.auth_token.clone();
        
        Ok(message)
    }
    
    /// Create from IPC message
    pub fn from_ipc_message(message: &IPCMessage) -> Result<Self> {
        let message_type = match message.message_type {
            crate::ipc::common::message::MessageType::Request => WebSocketMessageType::Request,
            crate::ipc::common::message::MessageType::Response => WebSocketMessageType::Response,
            crate::ipc::common::message::MessageType::Notification => WebSocketMessageType::Notification,
            crate::ipc::common::message::MessageType::Error => WebSocketMessageType::Error,
            crate::ipc::common::message::MessageType::Heartbeat => WebSocketMessageType::Heartbeat,
        };
        
        let payload = match &message.payload {
            crate::ipc::common::message::MessagePayload::String(s) => s.clone(),
            crate::ipc::common::message::MessagePayload::Json(j) => j.to_string(),
            crate::ipc::common::message::MessagePayload::Toml(t) => t.clone(),
            crate::ipc::common::message::MessagePayload::Binary(b) => String::from_utf8_lossy(b).to_string(),
            crate::ipc::common::message::MessagePayload::Empty => String::new(),
        };
        
        let mut ws_message = Self {
            id: message.id.clone(),
            message_type,
            payload,
            auth_token: message.auth_token.clone(),
            timestamp: message.timestamp,
        };
        
        Ok(ws_message)
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| IPCError::SerializationError(format!("Failed to serialize WebSocket message: {}", e)))
    }
    
    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| IPCError::SerializationError(format!("Failed to deserialize WebSocket message: {}", e)))
    }
    
    /// Serialize to binary
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| IPCError::SerializationError(format!("Failed to serialize WebSocket message: {}", e)))
    }
    
    /// Deserialize from binary
    pub fn from_binary(binary: &[u8]) -> Result<Self> {
        bincode::deserialize(binary)
            .map_err(|e| IPCError::SerializationError(format!("Failed to deserialize WebSocket message: {}", e)))
    }
}

/// WebSocket message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    /// Request message
    Request,
    
    /// Response message
    Response,
    
    /// Notification message
    Notification,
    
    /// Error message
    Error,
    
    /// Authentication message
    Authentication,
    
    /// Heartbeat message
    Heartbeat,
}

impl fmt::Display for WebSocketMessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketMessageType::Request => write!(f, "Request"),
            WebSocketMessageType::Response => write!(f, "Response"),
            WebSocketMessageType::Notification => write!(f, "Notification"),
            WebSocketMessageType::Error => write!(f, "Error"),
            WebSocketMessageType::Authentication => write!(f, "Authentication"),
            WebSocketMessageType::Heartbeat => write!(f, "Heartbeat"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let message = WebSocketMessage::new(
            WebSocketMessageType::Request,
            "test payload",
        );
        
        assert_eq!(message.message_type, WebSocketMessageType::Request);
        assert_eq!(message.payload, "test payload");
        assert_eq!(message.auth_token, None);
    }
    
    #[test]
    fn test_request_message() {
        let message = WebSocketMessage::request("test payload");
        
        assert_eq!(message.message_type, WebSocketMessageType::Request);
        assert_eq!(message.payload, "test payload");
    }
    
    #[test]
    fn test_response_message() {
        let request = WebSocketMessage::request("test payload");
        let response = WebSocketMessage::response(&request.id, "response payload");
        
        assert_eq!(response.message_type, WebSocketMessageType::Response);
        assert_eq!(response.payload, "response payload");
        assert_eq!(response.id, request.id);
    }
    
    #[test]
    fn test_notification_message() {
        let message = WebSocketMessage::notification("test payload");
        
        assert_eq!(message.message_type, WebSocketMessageType::Notification);
        assert_eq!(message.payload, "test payload");
    }
    
    #[test]
    fn test_error_message() {
        let request = WebSocketMessage::request("test payload");
        let error = WebSocketMessage::error(&request.id, "error message");
        
        assert_eq!(error.message_type, WebSocketMessageType::Error);
        assert_eq!(error.payload, "error message");
        assert_eq!(error.id, request.id);
    }
    
    #[test]
    fn test_authentication_message() {
        let message = WebSocketMessage::authentication("token123");
        
        assert_eq!(message.message_type, WebSocketMessageType::Authentication);
        assert_eq!(message.auth_token, Some("token123".to_string()));
    }
    
    #[test]
    fn test_message_with_auth_token() {
        let message = WebSocketMessage::new(
            WebSocketMessageType::Request,
            "test payload",
        ).with_auth_token("token123");
        
        assert_eq!(message.auth_token, Some("token123".to_string()));
    }
    
    #[test]
    fn test_json_serialization() {
        let message = WebSocketMessage::request("test payload")
            .with_auth_token("token123");
        
        let json = message.to_json().unwrap();
        let deserialized = WebSocketMessage::from_json(&json).unwrap();
        
        assert_eq!(deserialized.id, message.id);
        assert_eq!(deserialized.message_type, message.message_type);
        assert_eq!(deserialized.payload, message.payload);
        assert_eq!(deserialized.auth_token, message.auth_token);
        assert_eq!(deserialized.timestamp, message.timestamp);
    }
    
    #[test]
    fn test_binary_serialization() {
        let message = WebSocketMessage::request("test payload")
            .with_auth_token("token123");
        
        let binary = message.to_binary().unwrap();
        let deserialized = WebSocketMessage::from_binary(&binary).unwrap();
        
        assert_eq!(deserialized.id, message.id);
        assert_eq!(deserialized.message_type, message.message_type);
        assert_eq!(deserialized.payload, message.payload);
        assert_eq!(deserialized.auth_token, message.auth_token);
        assert_eq!(deserialized.timestamp, message.timestamp);
    }
    
    #[test]
    fn test_ipc_message_conversion() {
        let ws_message = WebSocketMessage::request("test payload")
            .with_auth_token("token123");
        
        let ipc_message = ws_message.to_ipc_message("client", "server").unwrap();
        let ws_message2 = WebSocketMessage::from_ipc_message(&ipc_message).unwrap();
        
        assert_eq!(ws_message2.id, ws_message.id);
        assert_eq!(ws_message2.message_type, ws_message.message_type);
        assert_eq!(ws_message2.payload, ws_message.payload);
        assert_eq!(ws_message2.auth_token, ws_message.auth_token);
        assert_eq!(ws_message2.timestamp, ws_message.timestamp);
    }
}
