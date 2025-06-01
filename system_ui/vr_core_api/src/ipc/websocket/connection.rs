//! WebSocket connection implementation for IPC mechanisms.
//!
//! This module provides connection handling for WebSocket IPC,
//! including reading, writing, and managing connections.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::protocol::{Message as WsMessage, CloseFrame},
    WebSocketStream,
};
use tokio_tungstenite::tungstenite::Error as WsError;

use crate::security::authentication::AuthToken;
use crate::ipc::common::{IPCError, Result};
use super::protocol::{WebSocketMessage, WebSocketProtocol};

/// WebSocket connection
pub struct WebSocketConnection {
    /// Client ID
    client_id: String,
    
    /// WebSocket stream
    stream: Option<WebSocketStream<TcpStream>>,
    
    /// Protocol
    protocol: WebSocketProtocol,
    
    /// Authentication token
    auth_token: Option<AuthToken>,
    
    /// Last activity timestamp
    last_activity: Instant,
    
    /// Message queue
    message_queue: VecDeque<WebSocketMessage>,
    
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
}

impl WebSocketConnection {
    /// Create a new WebSocketConnection
    pub fn new(stream: WebSocketStream<TcpStream>, client_id: &str, protocol: WebSocketProtocol) -> Self {
        Self {
            client_id: client_id.to_string(),
            stream: Some(stream),
            protocol,
            auth_token: None,
            last_activity: Instant::now(),
            message_queue: VecDeque::new(),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Set authentication token
    pub fn set_auth_token(&mut self, token: AuthToken) {
        self.auth_token = Some(token);
    }
    
    /// Get authentication token
    pub fn auth_token(&self) -> Option<&AuthToken> {
        self.auth_token.as_ref()
    }
    
    /// Send message
    pub async fn send_message(&mut self, message: WebSocketMessage) -> Result<()> {
        // Update last activity
        self.update_last_activity();
        
        // Get stream
        let stream = self.stream.as_mut().ok_or_else(|| {
            IPCError::ConnectionError("Connection closed".to_string())
        })?;
        
        // Serialize message
        let ws_message = match self.protocol {
            WebSocketProtocol::Json => {
                let json = message.to_json()?;
                WsMessage::Text(json)
            }
            WebSocketProtocol::Binary => {
                let binary = message.to_binary()?;
                WsMessage::Binary(binary)
            }
        };
        
        // Send message
        stream.send(ws_message).await
            .map_err(|e| IPCError::ConnectionError(format!("Failed to send message: {}", e)))?;
        
        debug!("Sent message {} to client {}", message.id, self.client_id);
        
        Ok(())
    }
    
    /// Receive message
    pub async fn receive_message(&mut self) -> Result<Option<WebSocketMessage>> {
        // Check if there are queued messages
        if let Some(message) = self.message_queue.pop_front() {
            return Ok(Some(message));
        }
        
        // Get stream
        let stream = self.stream.as_mut().ok_or_else(|| {
            IPCError::ConnectionError("Connection closed".to_string())
        })?;
        
        // Receive message
        match stream.next().await {
            Some(Ok(ws_message)) => {
                // Update last activity
                self.update_last_activity();
                
                // Parse message
                match ws_message {
                    WsMessage::Text(text) => {
                        let message = WebSocketMessage::from_json(&text)?;
                        debug!("Received text message {} from client {}", message.id, self.client_id);
                        Ok(Some(message))
                    }
                    WsMessage::Binary(binary) => {
                        let message = WebSocketMessage::from_binary(&binary)?;
                        debug!("Received binary message {} from client {}", message.id, self.client_id);
                        Ok(Some(message))
                    }
                    WsMessage::Ping(data) => {
                        // Respond with pong
                        stream.send(WsMessage::Pong(data)).await
                            .map_err(|e| IPCError::ConnectionError(format!("Failed to send pong: {}", e)))?;
                        Ok(None)
                    }
                    WsMessage::Pong(_) => {
                        // Ignore pong
                        Ok(None)
                    }
                    WsMessage::Close(frame) => {
                        // Close connection
                        self.close().await?;
                        Err(IPCError::ConnectionError("Connection closed by peer".to_string()))
                    }
                    WsMessage::Frame(_) => {
                        // Ignore raw frames
                        Ok(None)
                    }
                }
            }
            Some(Err(e)) => {
                Err(IPCError::ConnectionError(format!("WebSocket error: {}", e)))
            }
            None => {
                // Connection closed
                self.close().await?;
                Err(IPCError::ConnectionError("Connection closed".to_string()))
            }
        }
    }
    
    /// Close connection
    pub async fn close(&mut self) -> Result<()> {
        debug!("Closing connection for client {}", self.client_id);
        
        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Close stream
        if let Some(stream) = self.stream.as_mut() {
            // Send close frame
            let close_frame = CloseFrame {
                code: 1000, // Normal closure
                reason: "Connection closed".into(),
            };
            
            if let Err(e) = stream.send(WsMessage::Close(Some(close_frame))).await {
                warn!("Failed to send close frame: {}", e);
            }
        }
        
        // Remove stream
        self.stream = None;
        
        Ok(())
    }
    
    /// Is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }
    
    /// Get client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    
    /// Get protocol
    pub fn protocol(&self) -> WebSocketProtocol {
        self.protocol
    }
    
    /// Get last activity time
    pub fn last_activity(&self) -> Instant {
        self.last_activity
    }
    
    /// Update last activity time
    pub fn update_last_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Is shutdown signaled
    pub fn is_shutdown_signaled(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }
    
    /// Queue message
    pub fn queue_message(&mut self, message: WebSocketMessage) {
        self.message_queue.push_back(message);
    }
    
    /// Is connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && !self.is_shutdown_signaled()
    }
}

#[cfg(test)]
mod tests {
    // Tests would be added here
}
