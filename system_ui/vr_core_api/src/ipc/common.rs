//! Common IPC components for the VR headset.
//!
//! This module provides shared functionality for all IPC mechanisms,
//! including message definitions, serialization, and authentication.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::security::auth::{AuthContext, AuthError, AuthResult};

/// IPC error type.
#[derive(Debug, Error)]
pub enum IpcError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Message format error
    #[error("Message format error: {0}")]
    Format(String),
    
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    /// Handler not found
    #[error("Handler not found for message type: {0}")]
    HandlerNotFound(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// IPC result type.
pub type IpcResult<T> = Result<T, IpcError>;

/// Message priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessagePriority {
    /// High priority
    High,
    
    /// Normal priority
    Normal,
    
    /// Low priority
    Low,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Message header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Message ID
    pub id: String,
    
    /// Message type
    pub message_type: String,
    
    /// Sender ID
    pub sender: String,
    
    /// Recipient ID
    pub recipient: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Priority
    pub priority: MessagePriority,
    
    /// Correlation ID for request-response
    pub correlation_id: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MessageHeader {
    /// Create a new message header.
    pub fn new(
        message_type: &str,
        sender: &str,
        recipient: &str,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        Self {
            id,
            message_type: message_type.to_string(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            timestamp,
            priority: MessagePriority::Normal,
            correlation_id: None,
            auth_token: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the message priority.
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the correlation ID.
    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }
    
    /// Set the authentication token.
    pub fn with_auth_token(mut self, auth_token: &str) -> Self {
        self.auth_token = Some(auth_token.to_string());
        self
    }
    
    /// Add metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessagePayload {
    /// String payload
    String(String),
    
    /// Binary payload
    Binary(Vec<u8>),
    
    /// JSON payload
    Json(serde_json::Value),
    
    /// Empty payload
    Empty,
}

impl MessagePayload {
    /// Create a new string payload.
    pub fn string(value: &str) -> Self {
        Self::String(value.to_string())
    }
    
    /// Create a new binary payload.
    pub fn binary(value: Vec<u8>) -> Self {
        Self::Binary(value)
    }
    
    /// Create a new JSON payload.
    pub fn json<T: Serialize>(value: &T) -> IpcResult<Self> {
        match serde_json::to_value(value) {
            Ok(json) => Ok(Self::Json(json)),
            Err(e) => Err(IpcError::Serialization(e.to_string())),
        }
    }
    
    /// Create a new empty payload.
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// Get the payload as a string.
    pub fn as_string(&self) -> IpcResult<&str> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err(IpcError::Format("Payload is not a string".to_string())),
        }
    }
    
    /// Get the payload as binary.
    pub fn as_binary(&self) -> IpcResult<&[u8]> {
        match self {
            Self::Binary(b) => Ok(b),
            _ => Err(IpcError::Format("Payload is not binary".to_string())),
        }
    }
    
    /// Get the payload as JSON.
    pub fn as_json(&self) -> IpcResult<&serde_json::Value> {
        match self {
            Self::Json(j) => Ok(j),
            _ => Err(IpcError::Format("Payload is not JSON".to_string())),
        }
    }
    
    /// Parse the payload as a specific type.
    pub fn parse<T: for<'de> Deserialize<'de>>(&self) -> IpcResult<T> {
        match self {
            Self::Json(j) => {
                serde_json::from_value(j.clone())
                    .map_err(|e| IpcError::Deserialization(e.to_string()))
            },
            Self::String(s) => {
                serde_json::from_str(s)
                    .map_err(|e| IpcError::Deserialization(e.to_string()))
            },
            _ => Err(IpcError::Format("Cannot parse payload".to_string())),
        }
    }
}

/// IPC message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message header
    pub header: MessageHeader,
    
    /// Message payload
    pub payload: MessagePayload,
}

impl Message {
    /// Create a new message.
    pub fn new(
        message_type: &str,
        sender: &str,
        recipient: &str,
        payload: MessagePayload,
    ) -> Self {
        Self {
            header: MessageHeader::new(message_type, sender, recipient),
            payload,
        }
    }
    
    /// Create a new request message.
    pub fn request(
        message_type: &str,
        sender: &str,
        recipient: &str,
        payload: MessagePayload,
    ) -> Self {
        Self {
            header: MessageHeader::new(message_type, sender, recipient)
                .with_priority(MessagePriority::Normal),
            payload,
        }
    }
    
    /// Create a new response message.
    pub fn response(
        request: &Message,
        payload: MessagePayload,
    ) -> Self {
        Self {
            header: MessageHeader::new(
                &format!("{}.response", request.header.message_type),
                &request.header.recipient,
                &request.header.sender,
            )
            .with_correlation_id(&request.header.id)
            .with_priority(request.header.priority),
            payload,
        }
    }
    
    /// Create a new error response message.
    pub fn error_response(
        request: &Message,
        error: &str,
    ) -> Self {
        Self {
            header: MessageHeader::new(
                &format!("{}.error", request.header.message_type),
                &request.header.recipient,
                &request.header.sender,
            )
            .with_correlation_id(&request.header.id)
            .with_priority(MessagePriority::High),
            payload: MessagePayload::string(error),
        }
    }
    
    /// Create a new broadcast message.
    pub fn broadcast(
        message_type: &str,
        sender: &str,
        payload: MessagePayload,
    ) -> Self {
        Self {
            header: MessageHeader::new(message_type, sender, "*"),
            payload,
        }
    }
    
    /// Serialize the message to JSON.
    pub fn to_json(&self) -> IpcResult<String> {
        serde_json::to_string(self)
            .map_err(|e| IpcError::Serialization(e.to_string()))
    }
    
    /// Deserialize a message from JSON.
    pub fn from_json(json: &str) -> IpcResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| IpcError::Deserialization(e.to_string()))
    }
    
    /// Serialize the message to binary.
    pub fn to_binary(&self) -> IpcResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| IpcError::Serialization(e.to_string()))
    }
    
    /// Deserialize a message from binary.
    pub fn from_binary(binary: &[u8]) -> IpcResult<Self> {
        bincode::deserialize(binary)
            .map_err(|e| IpcError::Deserialization(e.to_string()))
    }
    
    /// Check if this is a response message.
    pub fn is_response(&self) -> bool {
        self.header.message_type.ends_with(".response")
    }
    
    /// Check if this is an error response message.
    pub fn is_error(&self) -> bool {
        self.header.message_type.ends_with(".error")
    }
    
    /// Check if this is a broadcast message.
    pub fn is_broadcast(&self) -> bool {
        self.header.recipient == "*"
    }
}

/// Message handler function.
pub type MessageHandlerFn = Box<dyn Fn(&Message, &AuthContext) -> IpcResult<Message> + Send + Sync>;

/// Message handler.
#[derive(Clone)]
pub struct MessageHandler {
    /// Message type
    message_type: String,
    
    /// Handler function
    handler: Arc<MessageHandlerFn>,
    
    /// Authentication required
    auth_required: bool,
}

impl MessageHandler {
    /// Create a new message handler.
    pub fn new<F>(message_type: &str, handler: F, auth_required: bool) -> Self
    where
        F: Fn(&Message, &AuthContext) -> IpcResult<Message> + Send + Sync + 'static,
    {
        Self {
            message_type: message_type.to_string(),
            handler: Arc::new(Box::new(handler)),
            auth_required,
        }
    }
    
    /// Get the message type.
    pub fn message_type(&self) -> &str {
        &self.message_type
    }
    
    /// Check if authentication is required.
    pub fn auth_required(&self) -> bool {
        self.auth_required
    }
    
    /// Handle a message.
    pub fn handle(&self, message: &Message, auth_context: &AuthContext) -> IpcResult<Message> {
        // Check if authentication is required
        if self.auth_required && !auth_context.is_authenticated() {
            return Err(IpcError::Auth(AuthError::NotAuthenticated));
        }
        
        // Call the handler function
        (self.handler)(message)
    }
}

impl fmt::Debug for MessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageHandler")
            .field("message_type", &self.message_type)
            .field("auth_required", &self.auth_required)
            .finish()
    }
}

/// Message router.
#[derive(Debug, Default, Clone)]
pub struct MessageRouter {
    /// Message handlers
    handlers: HashMap<String, MessageHandler>,
}

impl MessageRouter {
    /// Create a new message router.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register a message handler.
    pub fn register_handler(&mut self, handler: MessageHandler) {
        self.handlers.insert(handler.message_type().to_string(), handler);
    }
    
    /// Unregister a message handler.
    pub fn unregister_handler(&mut self, message_type: &str) -> Option<MessageHandler> {
        self.handlers.remove(message_type)
    }
    
    /// Get a message handler.
    pub fn get_handler(&self, message_type: &str) -> Option<&MessageHandler> {
        self.handlers.get(message_type)
    }
    
    /// Route a message to the appropriate handler.
    pub fn route(&self, message: &Message, auth_context: &AuthContext) -> IpcResult<Message> {
        // Get the handler for this message type
        let handler = self.get_handler(&message.header.message_type)
            .ok_or_else(|| IpcError::HandlerNotFound(message.header.message_type.clone()))?;
        
        // Handle the message
        handler.handle(message, auth_context)
    }
    
    /// Route a message with connection information.
    pub fn route_message(&self, message: &Message, connection: Option<Arc<dyn IpcConnection>>) -> IpcResult<Message> {
        // Create a default auth context if no connection is provided
        let auth_context = match &connection {
            Some(conn) => {
                let guard = conn.auth_context().read().unwrap();
                AuthContext {
                    session_id: guard.session_id.clone(),
                    user_id: guard.user_id.clone(),
                    username: guard.username.clone(),
                    roles: guard.roles.clone(),
                }
            },
            None => AuthContext::new(),
        };
        
        // Route the message
        self.route(message, &auth_context)
    }
}

/// IPC connection interface.
pub trait IpcConnection: Send + Sync {
    /// Get the connection ID.
    fn id(&self) -> &str;
    
    /// Send a message.
    fn send(&self, message: &Message) -> IpcResult<()>;
    
    /// Send a message and wait for a response.
    fn send_and_receive(&self, message: &Message, timeout: Duration) -> IpcResult<Message>;
    
    /// Close the connection.
    fn close(&self) -> IpcResult<()>;
    
    /// Check if the connection is open.
    fn is_open(&self) -> bool;
    
    /// Get the authentication context.
    fn auth_context(&self) -> Arc<RwLock<AuthContext>>;
    
    /// Set the authentication context.
    fn set_auth_context(&self, auth_context: AuthContext) -> IpcResult<()>;
}

/// IPC server interface.
pub trait IpcServer: Send + Sync {
    /// Start the server.
    fn start(&self) -> IpcResult<()>;
    
    /// Stop the server.
    fn stop(&self) -> IpcResult<()>;
    
    /// Check if the server is running.
    fn is_running(&self) -> bool;
    
    /// Get the server address.
    fn address(&self) -> &str;
    
    /// Get the message router.
    fn router(&self) -> Arc<RwLock<MessageRouter>>;
    
    /// Set the message router.
    fn set_router(&mut self, router: MessageRouter) -> IpcResult<()>;
    
    /// Broadcast a message to all connected clients.
    fn broadcast(&self, message: &Message) -> IpcResult<()>;
    
    /// Get the number of connected clients.
    fn client_count(&self) -> usize;
}

/// IPC client interface.
pub trait IpcClient: Send + Sync {
    /// Connect to the server.
    fn connect(&self) -> IpcResult<Box<dyn IpcConnection>>;
    
    /// Check if the client is connected.
    fn is_connected(&self) -> bool;
    
    /// Get the server address.
    fn server_address(&self) -> &str;
    
    /// Get the client ID.
    fn id(&self) -> &str;
    
    /// Get the current connection.
    fn connection(&self) -> Option<Box<dyn IpcConnection>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_header() {
        let header = MessageHeader::new("test", "sender", "recipient");
        
        assert_eq!(header.message_type, "test");
        assert_eq!(header.sender, "sender");
        assert_eq!(header.recipient, "recipient");
        assert_eq!(header.priority, MessagePriority::Normal);
        assert!(header.correlation_id.is_none());
        assert!(header.auth_token.is_none());
        assert!(header.metadata.is_empty());
        
        let header = header
            .with_priority(MessagePriority::High)
            .with_correlation_id("corr-id")
            .with_auth_token("auth-token")
            .with_metadata("key", "value");
        
        assert_eq!(header.priority, MessagePriority::High);
        assert_eq!(header.correlation_id, Some("corr-id".to_string()));
        assert_eq!(header.auth_token, Some("auth-token".to_string()));
        assert_eq!(header.metadata.get("key"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_message_payload() {
        // String payload
        let payload = MessagePayload::string("test");
        assert_eq!(payload.as_string().unwrap(), "test");
        assert!(payload.as_binary().is_err());
        assert!(payload.as_json().is_err());
        
        // Binary payload
        let payload = MessagePayload::binary(vec![1, 2, 3]);
        assert_eq!(payload.as_binary().unwrap(), &[1, 2, 3]);
        assert!(payload.as_string().is_err());
        assert!(payload.as_json().is_err());
        
        // JSON payload
        let json = serde_json::json!({"key": "value"});
        let payload = MessagePayload::Json(json.clone());
        assert_eq!(payload.as_json().unwrap(), &json);
        assert!(payload.as_string().is_err());
        assert!(payload.as_binary().is_err());
        
        // Empty payload
        let payload = MessagePayload::empty();
        assert!(payload.as_string().is_err());
        assert!(payload.as_binary().is_err());
        assert!(payload.as_json().is_err());
        
        // Parse payload
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestStruct {
            key: String,
        }
        
        let test_struct = TestStruct { key: "value".to_string() };
        let payload = MessagePayload::json(&test_struct).unwrap();
        let parsed: TestStruct = payload.parse().unwrap();
        assert_eq!(parsed, test_struct);
    }
    
    #[test]
    fn test_message() {
        // Create a request message
        let request = Message::request(
            "test.request",
            "client",
            "server",
            MessagePayload::string("request data"),
        );
        
        assert_eq!(request.header.message_type, "test.request");
        assert_eq!(request.header.sender, "client");
        assert_eq!(request.header.recipient, "server");
        assert_eq!(request.payload.as_string().unwrap(), "request data");
        assert!(!request.is_response());
        assert!(!request.is_error());
        assert!(!request.is_broadcast());
        
        // Create a response message
        let response = Message::response(
            &request,
            MessagePayload::string("response data"),
        );
        
        assert_eq!(response.header.message_type, "test.request.response");
        assert_eq!(response.header.sender, "server");
        assert_eq!(response.header.recipient, "client");
        assert_eq!(response.header.correlation_id, Some(request.header.id.clone()));
        assert_eq!(response.payload.as_string().unwrap(), "response data");
        assert!(response.is_response());
        assert!(!response.is_error());
        assert!(!response.is_broadcast());
        
        // Create an error response message
        let error = Message::error_response(
            &request,
            "error message",
        );
        
        assert_eq!(error.header.message_type, "test.request.error");
        assert_eq!(error.header.sender, "server");
        assert_eq!(error.header.recipient, "client");
        assert_eq!(error.header.correlation_id, Some(request.header.id.clone()));
        assert_eq!(error.payload.as_string().unwrap(), "error message");
        assert!(!error.is_response());
        assert!(error.is_error());
        assert!(!error.is_broadcast());
        
        // Create a broadcast message
        let broadcast = Message::broadcast(
            "test.broadcast",
            "server",
            MessagePayload::string("broadcast data"),
        );
        
        assert_eq!(broadcast.header.message_type, "test.broadcast");
        assert_eq!(broadcast.header.sender, "server");
        assert_eq!(broadcast.header.recipient, "*");
        assert_eq!(broadcast.payload.as_string().unwrap(), "broadcast data");
        assert!(!broadcast.is_response());
        assert!(!broadcast.is_error());
        assert!(broadcast.is_broadcast());
        
        // Serialize and deserialize
        let json = request.to_json().unwrap();
        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(deserialized.header.id, request.header.id);
        assert_eq!(deserialized.header.message_type, request.header.message_type);
        assert_eq!(deserialized.payload.as_string().unwrap(), request.payload.as_string().unwrap());
        
        let binary = request.to_binary().unwrap();
        let deserialized = Message::from_binary(&binary).unwrap();
        assert_eq!(deserialized.header.id, request.header.id);
        assert_eq!(deserialized.header.message_type, request.header.message_type);
        assert_eq!(deserialized.payload.as_string().unwrap(), request.payload.as_string().unwrap());
    }
    
    #[test]
    fn test_message_router() {
        let mut router = MessageRouter::new();
        
        // Create a test handler
        let handler = MessageHandler::new(
            "test.request",
            |message, _| {
                Ok(Message::response(
                    message,
                    MessagePayload::string("response data"),
                ))
            },
            false,
        );
        
        // Register the handler
        router.register_handler(handler);
        
        // Create a test message
        let request = Message::request(
            "test.request",
            "client",
            "server",
            MessagePayload::string("request data"),
        );
        
        // Create an auth context
        let auth_context = AuthContext::new();
        
        // Route the message
        let response = router.route(&request, &auth_context).unwrap();
        
        assert_eq!(response.header.message_type, "test.request.response");
        assert_eq!(response.header.sender, "server");
        assert_eq!(response.header.recipient, "client");
        assert_eq!(response.header.correlation_id, Some(request.header.id.clone()));
        assert_eq!(response.payload.as_string().unwrap(), "response data");
        
        // Try to route a message with no handler
        let request = Message::request(
            "unknown.request",
            "client",
            "server",
            MessagePayload::string("request data"),
        );
        
        let result = router.route(&request, &auth_context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IpcError::HandlerNotFound(_)));
        
        // Unregister the handler
        let handler = router.unregister_handler("test.request").unwrap();
        assert_eq!(handler.message_type(), "test.request");
        
        // Try to route a message after unregistering the handler
        let request = Message::request(
            "test.request",
            "client",
            "server",
            MessagePayload::string("request data"),
        );
        
        let result = router.route(&request, &auth_context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IpcError::HandlerNotFound(_)));
    }
}
