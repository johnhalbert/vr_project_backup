//! Unix socket connection handling for IPC mechanisms.
//!
//! This module provides connection handling for Unix domain socket IPC,
//! including reading, writing, and managing connections.

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::collections::VecDeque;

use log::{debug, error, info, trace, warn};

use crate::security::authentication::AuthToken;
use crate::ipc::common::{IPCError, Result, IPCMessage, serialize_message, deserialize_message};

/// Unix socket connection
pub struct UnixSocketConnection {
    /// Unix stream
    stream: UnixStream,
    
    /// Client ID
    client_id: String,
    
    /// Authentication token
    auth_token: Option<AuthToken>,
    
    /// Last activity timestamp
    last_activity: Instant,
    
    /// Message queue
    message_queue: VecDeque<IPCMessage>,
    
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
}

impl UnixSocketConnection {
    /// Create a new UnixSocketConnection
    pub fn new(stream: UnixStream, client_id: &str) -> Self {
        // Set non-blocking mode
        if let Err(e) = stream.set_nonblocking(true) {
            warn!("Failed to set non-blocking mode for connection {}: {}", client_id, e);
        }
        
        Self {
            stream,
            client_id: client_id.to_string(),
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
    pub fn send_message(&mut self, message: IPCMessage) -> Result<()> {
        // Update last activity
        self.update_last_activity();
        
        // Serialize message
        let bytes = serialize_message(&message)?;
        
        // Write message length
        let length = bytes.len() as u32;
        let length_bytes = length.to_be_bytes();
        self.stream.write_all(&length_bytes)?;
        
        // Write message
        self.stream.write_all(&bytes)?;
        self.stream.flush()?;
        
        debug!("Sent message {} to client {}", message.id, self.client_id);
        
        Ok(())
    }
    
    /// Receive message
    pub fn receive_message(&mut self) -> Result<Option<IPCMessage>> {
        // Check if there are queued messages
        if let Some(message) = self.message_queue.pop_front() {
            return Ok(Some(message));
        }
        
        // Read message length
        let mut length_bytes = [0u8; 4];
        match self.stream.read_exact(&mut length_bytes) {
            Ok(_) => {
                // Update last activity
                self.update_last_activity();
                
                // Parse message length
                let length = u32::from_be_bytes(length_bytes) as usize;
                
                // Read message
                let mut bytes = vec![0u8; length];
                self.stream.read_exact(&mut bytes)?;
                
                // Deserialize message
                let message = deserialize_message(&bytes)?;
                
                debug!("Received message {} from client {}", message.id, self.client_id);
                
                Ok(Some(message))
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available
                Ok(None)
            }
            Err(e) => {
                Err(IPCError::IoError(e))
            }
        }
    }
    
    /// Close connection
    pub fn close(&mut self) -> Result<()> {
        debug!("Closing connection for client {}", self.client_id);
        self.shutdown_signal.store(true, Ordering::SeqCst);
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
    pub fn queue_message(&mut self, message: IPCMessage) {
        self.message_queue.push_back(message);
    }
    
    /// Get stream
    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }
    
    /// Get stream mut
    pub fn stream_mut(&mut self) -> &mut UnixStream {
        &mut self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::{UnixStream, UnixListener};
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;
    
    use crate::ipc::common::message::{MessagePayload, MessageType};
    
    #[test]
    fn test_connection_creation() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        let client_thread = thread::spawn(move || {
            let stream = UnixStream::connect(&socket_path).unwrap();
            stream
        });
        
        let (server_stream, _) = listener.accept().unwrap();
        let connection = UnixSocketConnection::new(server_stream, "test_client");
        
        assert_eq!(connection.client_id(), "test_client");
        assert!(!connection.is_authenticated());
        assert!(!connection.is_shutdown_signaled());
        
        let _client_stream = client_thread.join().unwrap();
    }
    
    #[test]
    fn test_send_receive_message() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        let client_thread = thread::spawn(move || {
            let stream = UnixStream::connect(&socket_path).unwrap();
            let mut connection = UnixSocketConnection::new(stream, "test_client");
            
            let message = IPCMessage::new(
                MessageType::Request,
                "client",
                "server",
                MessagePayload::String("test".to_string()),
            );
            
            connection.send_message(message).unwrap();
            
            // Wait for response
            thread::sleep(Duration::from_millis(100));
            
            let response = connection.receive_message().unwrap();
            response
        });
        
        let (server_stream, _) = listener.accept().unwrap();
        let mut connection = UnixSocketConnection::new(server_stream, "test_server");
        
        // Wait for message
        thread::sleep(Duration::from_millis(50));
        
        let message = connection.receive_message().unwrap().unwrap();
        assert_eq!(message.message_type, MessageType::Request);
        assert_eq!(message.source, "client");
        assert_eq!(message.destination, "server");
        assert_eq!(message.payload, MessagePayload::String("test".to_string()));
        
        // Send response
        let response = IPCMessage::new(
            MessageType::Response,
            "server",
            "client",
            MessagePayload::String("response".to_string()),
        );
        
        connection.send_message(response).unwrap();
        
        let client_response = client_thread.join().unwrap();
        assert!(client_response.is_some());
        
        let client_response = client_response.unwrap();
        assert_eq!(client_response.message_type, MessageType::Response);
        assert_eq!(client_response.source, "server");
        assert_eq!(client_response.destination, "client");
        assert_eq!(client_response.payload, MessagePayload::String("response".to_string()));
    }
    
    #[test]
    fn test_connection_close() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        let client_thread = thread::spawn(move || {
            let stream = UnixStream::connect(&socket_path).unwrap();
            let mut connection = UnixSocketConnection::new(stream, "test_client");
            
            assert!(!connection.is_shutdown_signaled());
            connection.close().unwrap();
            assert!(connection.is_shutdown_signaled());
        });
        
        let (server_stream, _) = listener.accept().unwrap();
        let _connection = UnixSocketConnection::new(server_stream, "test_server");
        
        client_thread.join().unwrap();
    }
}
