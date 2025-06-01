//! Unix socket client implementation for IPC mechanisms.
//!
//! This module provides client implementation for Unix domain socket IPC,
//! including connection management and message handling.

use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use log::{debug, error, info, trace, warn};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::security::authentication::{AuthToken, Credentials};
use crate::ipc::common::{IPCError, Result, IPCMessage, MessageHandler, MessageType};
use super::connection::UnixSocketConnection;

/// Unix socket client
pub struct UnixSocketClient {
    /// Socket path
    socket_path: PathBuf,
    
    /// Client ID
    client_id: String,
    
    /// Connection
    connection: Option<UnixSocketConnection>,
    
    /// Authentication token
    auth_token: Option<AuthToken>,
    
    /// Message handlers
    message_handlers: HashMap<String, Box<dyn MessageHandler>>,
    
    /// Pending requests
    pending_requests: HashMap<String, oneshot::Sender<IPCMessage>>,
    
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
    
    /// Receiver thread
    receiver_thread: Option<JoinHandle<()>>,
}

impl UnixSocketClient {
    /// Create a new UnixSocketClient
    pub fn new(socket_path: &Path, client_id: &str) -> Self {
        Self {
            socket_path: socket_path.to_path_buf(),
            client_id: client_id.to_string(),
            connection: None,
            auth_token: None,
            message_handlers: HashMap::new(),
            pending_requests: HashMap::new(),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            receiver_thread: None,
        }
    }
    
    /// Connect to server
    pub fn connect(&mut self) -> Result<()> {
        // Check if already connected
        if self.connection.is_some() {
            return Err(IPCError::ConnectionError("Already connected".to_string()));
        }
        
        // Reset shutdown signal
        self.shutdown_signal.store(false, Ordering::SeqCst);
        
        // Connect to server
        let stream = UnixStream::connect(&self.socket_path)?;
        
        // Create connection
        let connection = UnixSocketConnection::new(stream, &self.client_id);
        self.connection = Some(connection);
        
        info!("Connected to Unix socket server at {:?}", self.socket_path);
        
        // Start receiver thread
        let client_id = self.client_id.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let mut connection = self.connection.take().unwrap();
        let pending_requests = Arc::new(Mutex::new(self.pending_requests.clone()));
        let message_handlers = Arc::new(Mutex::new(self.message_handlers.clone()));
        
        let receiver_thread = thread::spawn(move || {
            Self::receiver_loop(client_id, connection, pending_requests, message_handlers, shutdown_signal);
        });
        
        self.receiver_thread = Some(receiver_thread);
        
        Ok(())
    }
    
    /// Disconnect from server
    pub fn disconnect(&mut self) -> Result<()> {
        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Wait for receiver thread to finish
        if let Some(thread) = self.receiver_thread.take() {
            if thread.join().is_err() {
                warn!("Failed to join receiver thread");
            }
        }
        
        // Clear connection
        self.connection = None;
        
        // Clear pending requests
        for (_, sender) in self.pending_requests.drain() {
            let _ = sender.send(IPCMessage::error(
                &IPCMessage::new(
                    MessageType::Request,
                    &self.client_id,
                    "server",
                    crate::ipc::common::message::MessagePayload::Empty,
                ),
                "Client disconnected",
            ));
        }
        
        info!("Disconnected from Unix socket server");
        
        Ok(())
    }
    
    /// Authenticate with server
    pub fn authenticate(&mut self, credentials: &Credentials) -> Result<AuthToken> {
        // Create authentication request
        let request = IPCMessage::request(
            &self.client_id,
            "server",
            crate::ipc::common::message::MessagePayload::json(credentials)?,
        );
        
        // Send request and wait for response
        let response = self.send_request(request, Some(5000))?;
        
        // Parse response
        match response.message_type {
            MessageType::Response => {
                // Parse token
                let token: AuthToken = response.payload.parse_json()?;
                
                // Store token
                self.auth_token = Some(token.clone());
                
                Ok(token)
            }
            MessageType::Error => {
                Err(IPCError::AuthenticationError(response.payload.as_string()?.to_string()))
            }
            _ => {
                Err(IPCError::ProtocolError("Unexpected response type".to_string()))
            }
        }
    }
    
    /// Send message to server
    pub fn send_message(&self, message: IPCMessage) -> Result<()> {
        if let Some(ref connection) = self.connection {
            connection.send_message(message)?;
            Ok(())
        } else {
            Err(IPCError::ConnectionError("Not connected".to_string()))
        }
    }
    
    /// Send request and wait for response
    pub fn send_request(&mut self, request: IPCMessage, timeout_ms: Option<u64>) -> Result<IPCMessage> {
        // Create response channel
        let (sender, receiver) = oneshot::channel();
        
        // Store sender
        self.pending_requests.insert(request.id.clone(), sender);
        
        // Send request
        self.send_message(request)?;
        
        // Wait for response with timeout
        let timeout = timeout_ms.unwrap_or(30000);
        match tokio::runtime::Runtime::new()?.block_on(async {
            tokio::time::timeout(Duration::from_millis(timeout), receiver).await
        }) {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(IPCError::ConnectionError("Response channel closed".to_string())),
            Err(_) => {
                // Remove pending request
                self.pending_requests.remove(&request.id);
                Err(IPCError::TimeoutError(format!("Request timed out after {}ms", timeout)))
            }
        }
    }
    
    /// Register message handler
    pub fn register_handler(&mut self, handler: Box<dyn MessageHandler>) -> Result<()> {
        let id = handler.id().to_string();
        
        if self.message_handlers.contains_key(&id) {
            return Err(IPCError::InternalError(format!("Handler {} already registered", id)));
        }
        
        self.message_handlers.insert(id, handler);
        
        Ok(())
    }
    
    /// Unregister message handler
    pub fn unregister_handler(&mut self, id: &str) -> Result<()> {
        if !self.message_handlers.contains_key(id) {
            return Err(IPCError::InternalError(format!("Handler {} not registered", id)));
        }
        
        self.message_handlers.remove(id);
        
        Ok(())
    }
    
    /// Is connected
    pub fn is_connected(&self) -> bool {
        self.connection.is_some() && !self.shutdown_signal.load(Ordering::SeqCst)
    }
    
    /// Receiver loop
    fn receiver_loop(
        client_id: String,
        mut connection: UnixSocketConnection,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<IPCMessage>>>>,
        message_handlers: Arc<Mutex<HashMap<String, Box<dyn MessageHandler>>>>,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        while !shutdown_signal.load(Ordering::SeqCst) {
            // Receive message
            match connection.receive_message() {
                Ok(Some(message)) => {
                    // Handle message
                    match message.message_type {
                        MessageType::Response | MessageType::Error => {
                            // Find pending request
                            let mut pending = pending_requests.lock().unwrap();
                            if let Some(sender) = pending.remove(&message.id) {
                                // Send response
                                if sender.send(message).is_err() {
                                    warn!("Failed to send response to requester");
                                }
                            } else {
                                warn!("Received response for unknown request: {}", message.id);
                            }
                        }
                        _ => {
                            // Find handler
                            let handlers = message_handlers.lock().unwrap();
                            let mut handled = false;
                            
                            for (_, handler) in handlers.iter() {
                                if handler.supported_message_types().contains(&message.message_type) {
                                    // Handle message
                                    match handler.handle_message(message.clone()) {
                                        Ok(Some(response)) => {
                                            // Send response
                                            if let Err(e) = connection.send_message(response) {
                                                error!("Failed to send response: {}", e);
                                            }
                                        }
                                        Ok(None) => {
                                            // No response
                                        }
                                        Err(e) => {
                                            error!("Error handling message: {}", e);
                                            
                                            // Send error response
                                            let error_message = IPCMessage::error(&message, &e.to_string());
                                            if let Err(e) = connection.send_message(error_message) {
                                                error!("Failed to send error response: {}", e);
                                            }
                                        }
                                    }
                                    
                                    handled = true;
                                    break;
                                }
                            }
                            
                            if !handled {
                                warn!("No handler found for message type {:?}", message.message_type);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No message, sleep a bit
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        
        debug!("Receiver loop exited");
    }
}

impl Drop for UnixSocketClient {
    fn drop(&mut self) {
        if self.is_connected() {
            if let Err(e) = self.disconnect() {
                error!("Error disconnecting: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;
    
    use crate::ipc::common::message::{MessagePayload, MessageType};
    
    #[test]
    fn test_client_connect_disconnect() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        // Create server
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        // Create client
        let mut client = UnixSocketClient::new(&socket_path, "test_client");
        
        // Connect
        client.connect().unwrap();
        assert!(client.is_connected());
        
        // Accept connection
        let (_, _) = listener.accept().unwrap();
        
        // Disconnect
        client.disconnect().unwrap();
        assert!(!client.is_connected());
    }
    
    // Additional tests would be added here
}
