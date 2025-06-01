//! WebSocket client implementation for IPC mechanisms.
//!
//! This module provides client implementation for WebSocket IPC,
//! including connection management and message handling.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt, future, pin_mut};
use log::{debug, error, info, trace, warn};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{Message as WsMessage, CloseFrame},
    WebSocketStream,
};
use url::Url;
use uuid::Uuid;

use crate::security::authentication::{AuthToken, Credentials};
use crate::ipc::common::{IPCError, Result};
use super::connection::WebSocketConnection;
use super::protocol::{WebSocketMessage, WebSocketProtocol, WebSocketMessageType};

/// WebSocket client
pub struct WebSocketClient {
    /// Server URL
    url: Url,
    
    /// Client ID
    client_id: String,
    
    /// Protocol
    protocol: WebSocketProtocol,
    
    /// Connection
    connection: Option<Arc<Mutex<WebSocketConnection>>>,
    
    /// Authentication token
    auth_token: Option<AuthToken>,
    
    /// Pending requests
    pending_requests: HashMap<String, oneshot::Sender<WebSocketMessage>>,
    
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
    
    /// Runtime
    runtime: Option<Runtime>,
    
    /// Sender
    sender: Option<Sender<WebSocketMessage>>,
    
    /// Receiver task
    receiver_task: Option<JoinHandle<()>>,
    
    /// Sender task
    sender_task: Option<JoinHandle<()>>,
}

impl WebSocketClient {
    /// Create a new WebSocketClient
    pub fn new(url: &str, client_id: &str, protocol: WebSocketProtocol) -> Result<Self> {
        // Parse URL
        let url = Url::parse(url)
            .map_err(|e| IPCError::ConnectionError(format!("Invalid URL: {}", e)))?;
        
        // Create client
        let client = Self {
            url,
            client_id: client_id.to_string(),
            protocol,
            connection: None,
            auth_token: None,
            pending_requests: HashMap::new(),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            runtime: None,
            sender: None,
            receiver_task: None,
            sender_task: None,
        };
        
        Ok(client)
    }
    
    /// Connect to server
    pub fn connect(&mut self) -> Result<()> {
        // Check if already connected
        if self.connection.is_some() {
            return Err(IPCError::ConnectionError("Already connected".to_string()));
        }
        
        // Reset shutdown signal
        self.shutdown_signal.store(false, Ordering::SeqCst);
        
        // Create runtime
        let runtime = Runtime::new()
            .map_err(|e| IPCError::InternalError(format!("Failed to create runtime: {}", e)))?;
        
        // Connect to server
        let url = self.url.clone();
        let client_id = self.client_id.clone();
        let protocol = self.protocol;
        let shutdown_signal = self.shutdown_signal.clone();
        let pending_requests = Arc::new(Mutex::new(self.pending_requests.clone()));
        
        let (connection, sender, receiver_task, sender_task) = runtime.block_on(async {
            Self::connect_async(url, &client_id, protocol, shutdown_signal.clone(), pending_requests).await
        })?;
        
        self.connection = Some(connection);
        self.sender = Some(sender);
        self.receiver_task = Some(receiver_task);
        self.sender_task = Some(sender_task);
        self.runtime = Some(runtime);
        
        info!("Connected to WebSocket server at {}", self.url);
        
        Ok(())
    }
    
    /// Disconnect from server
    pub fn disconnect(&mut self) -> Result<()> {
        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Close connection
        if let Some(connection) = &self.connection {
            if let Some(runtime) = &self.runtime {
                runtime.block_on(async {
                    let mut conn = connection.lock().unwrap();
                    if let Err(e) = conn.close().await {
                        warn!("Failed to close connection: {}", e);
                    }
                });
            }
        }
        
        // Clear tasks
        self.receiver_task = None;
        self.sender_task = None;
        
        // Clear connection
        self.connection = None;
        self.sender = None;
        
        // Clear runtime
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(Duration::from_secs(5));
        }
        
        // Clear pending requests
        for (_, sender) in self.pending_requests.drain() {
            let error_message = WebSocketMessage::error(
                "request_id",
                "Client disconnected",
            );
            let _ = sender.send(error_message);
        }
        
        info!("Disconnected from WebSocket server");
        
        Ok(())
    }
    
    /// Authenticate with server
    pub fn authenticate(&mut self, credentials: &Credentials) -> Result<AuthToken> {
        // Create authentication message
        let token = credentials.to_token()?;
        let message = WebSocketMessage::authentication(&token);
        
        // Send message and wait for response
        let response = self.send_request(message, Some(5000))?;
        
        // Parse response
        match response.message_type {
            WebSocketMessageType::Response => {
                // Store token
                self.auth_token = Some(token.clone());
                
                Ok(token)
            }
            WebSocketMessageType::Error => {
                Err(IPCError::AuthenticationError(response.payload))
            }
            _ => {
                Err(IPCError::ProtocolError("Unexpected response type".to_string()))
            }
        }
    }
    
    /// Send message to server
    pub fn send_message(&self, message: WebSocketMessage) -> Result<()> {
        if let Some(sender) = &self.sender {
            if let Some(runtime) = &self.runtime {
                runtime.block_on(async {
                    sender.send(message).await
                        .map_err(|e| IPCError::ConnectionError(format!("Failed to send message: {}", e)))
                })
            } else {
                Err(IPCError::InternalError("Runtime not available".to_string()))
            }
        } else {
            Err(IPCError::ConnectionError("Not connected".to_string()))
        }
    }
    
    /// Send request and wait for response
    pub fn send_request(&mut self, request: WebSocketMessage, timeout_ms: Option<u64>) -> Result<WebSocketMessage> {
        // Create response channel
        let (sender, receiver) = oneshot::channel();
        
        // Store sender
        self.pending_requests.insert(request.id.clone(), sender);
        
        // Send request
        self.send_message(request)?;
        
        // Wait for response with timeout
        if let Some(runtime) = &self.runtime {
            let timeout = timeout_ms.unwrap_or(30000);
            match runtime.block_on(async {
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
        } else {
            Err(IPCError::InternalError("Runtime not available".to_string()))
        }
    }
    
    /// Is connected
    pub fn is_connected(&self) -> bool {
        if let Some(connection) = &self.connection {
            let conn = connection.lock().unwrap();
            conn.is_connected() && !self.shutdown_signal.load(Ordering::SeqCst)
        } else {
            false
        }
    }
    
    /// Connect async
    async fn connect_async(
        url: Url,
        client_id: &str,
        protocol: WebSocketProtocol,
        shutdown_signal: Arc<AtomicBool>,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<WebSocketMessage>>>>,
    ) -> Result<(
        Arc<Mutex<WebSocketConnection>>,
        Sender<WebSocketMessage>,
        JoinHandle<()>,
        JoinHandle<()>,
    )> {
        // Connect to server
        let (ws_stream, _) = connect_async(url.clone()).await
            .map_err(|e| IPCError::ConnectionError(format!("Failed to connect to WebSocket server: {}", e)))?;
        
        // Create connection
        let connection = WebSocketConnection::new(ws_stream, client_id, protocol);
        let connection = Arc::new(Mutex::new(connection));
        
        // Create message channels
        let (tx, rx) = mpsc::channel::<WebSocketMessage>(100);
        
        // Start sender task
        let connection_clone = connection.clone();
        let client_id_clone = client_id.to_string();
        let sender_task = tokio::spawn(async move {
            Self::sender_loop(connection_clone, rx, &client_id_clone).await;
        });
        
        // Start receiver task
        let connection_clone = connection.clone();
        let client_id_clone = client_id.to_string();
        let shutdown_signal_clone = shutdown_signal.clone();
        let pending_requests_clone = pending_requests.clone();
        let receiver_task = tokio::spawn(async move {
            Self::receiver_loop(
                connection_clone,
                &client_id_clone,
                shutdown_signal_clone,
                pending_requests_clone,
            ).await;
        });
        
        Ok((connection, tx, receiver_task, sender_task))
    }
    
    /// Sender loop
    async fn sender_loop(
        connection: Arc<Mutex<WebSocketConnection>>,
        mut rx: Receiver<WebSocketMessage>,
        client_id: &str,
    ) {
        while let Some(message) = rx.recv().await {
            // Send message
            let mut conn = connection.lock().unwrap();
            
            if let Err(e) = conn.send_message(message).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
        
        debug!("Sender loop exited for client {}", client_id);
    }
    
    /// Receiver loop
    async fn receiver_loop(
        connection: Arc<Mutex<WebSocketConnection>>,
        client_id: &str,
        shutdown_signal: Arc<AtomicBool>,
        pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<WebSocketMessage>>>>,
    ) {
        while !shutdown_signal.load(Ordering::SeqCst) {
            // Receive message
            let message = {
                let mut conn = connection.lock().unwrap();
                
                match conn.receive_message().await {
                    Ok(Some(message)) => message,
                    Ok(None) => {
                        // No message, continue
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        break;
                    }
                }
            };
            
            // Handle message
            match message.message_type {
                WebSocketMessageType::Response | WebSocketMessageType::Error => {
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
                WebSocketMessageType::Heartbeat => {
                    // Send heartbeat response
                    let mut conn = connection.lock().unwrap();
                    let response = WebSocketMessage::response(&message.id, "");
                    if let Err(e) = conn.send_message(response).await {
                        error!("Failed to send heartbeat response: {}", e);
                    }
                }
                _ => {
                    // Ignore other message types
                    debug!("Received message of type {:?}: {}", message.message_type, message.id);
                }
            }
        }
        
        debug!("Receiver loop exited for client {}", client_id);
    }
}

impl Drop for WebSocketClient {
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
    // Tests would be added here
}
