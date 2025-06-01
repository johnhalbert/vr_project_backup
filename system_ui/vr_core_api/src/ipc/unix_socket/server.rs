//! Unix socket server implementation for IPC mechanisms.
//!
//! This module provides server implementation for Unix domain socket IPC,
//! including connection management and message handling.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use log::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::security::authentication::AuthenticationProvider;
use crate::ipc::common::{IPCError, Result, IPCMessage, MessageHandler, MessageType};
use super::connection::UnixSocketConnection;

/// Unix socket server
pub struct UnixSocketServer {
    /// Socket path
    socket_path: PathBuf,
    
    /// Authentication provider
    auth_provider: Arc<dyn AuthenticationProvider>,
    
    /// Message handlers
    message_handlers: HashMap<String, Box<dyn MessageHandler>>,
    
    /// Active connections
    active_connections: Arc<RwLock<HashMap<String, UnixSocketConnection>>>,
    
    /// Shutdown signal
    shutdown_signal: Arc<AtomicBool>,
    
    /// Listener thread
    listener_thread: Option<JoinHandle<()>>,
    
    /// Connection handler thread
    connection_handler_thread: Option<JoinHandle<()>>,
}

impl UnixSocketServer {
    /// Create a new UnixSocketServer
    pub fn new(socket_path: &Path, auth_provider: Arc<dyn AuthenticationProvider>) -> Self {
        Self {
            socket_path: socket_path.to_path_buf(),
            auth_provider,
            message_handlers: HashMap::new(),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            listener_thread: None,
            connection_handler_thread: None,
        }
    }
    
    /// Start the server
    pub fn start(&mut self) -> Result<()> {
        // Check if already started
        if self.listener_thread.is_some() {
            return Err(IPCError::InternalError("Server already started".to_string()));
        }
        
        // Reset shutdown signal
        self.shutdown_signal.store(false, Ordering::SeqCst);
        
        // Create socket directory if it doesn't exist
        if let Some(parent) = self.socket_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Remove socket file if it exists
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
        }
        
        // Create listener
        let listener = UnixListener::bind(&self.socket_path)?;
        
        // Set non-blocking mode
        listener.set_nonblocking(true)?;
        
        info!("Unix socket server started at {:?}", self.socket_path);
        
        // Clone shared data for threads
        let socket_path = self.socket_path.clone();
        let active_connections = self.active_connections.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let auth_provider = self.auth_provider.clone();
        
        // Start listener thread
        let listener_thread = thread::spawn(move || {
            Self::listener_loop(listener, active_connections, auth_provider, shutdown_signal);
        });
        
        self.listener_thread = Some(listener_thread);
        
        // Start connection handler thread
        let active_connections = self.active_connections.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let connection_handler_thread = thread::spawn(move || {
            Self::connection_handler_loop(active_connections, shutdown_signal);
        });
        
        self.connection_handler_thread = Some(connection_handler_thread);
        
        Ok(())
    }
    
    /// Stop the server
    pub fn stop(&mut self) -> Result<()> {
        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Wait for threads to finish
        if let Some(thread) = self.listener_thread.take() {
            if thread.join().is_err() {
                warn!("Failed to join listener thread");
            }
        }
        
        if let Some(thread) = self.connection_handler_thread.take() {
            if thread.join().is_err() {
                warn!("Failed to join connection handler thread");
            }
        }
        
        // Close all connections
        let mut connections = self.active_connections.write().unwrap();
        for (client_id, connection) in connections.drain() {
            debug!("Closing connection for client {}", client_id);
        }
        
        // Remove socket file
        if self.socket_path.exists() {
            if let Err(e) = fs::remove_file(&self.socket_path) {
                warn!("Failed to remove socket file: {}", e);
            }
        }
        
        info!("Unix socket server stopped");
        
        Ok(())
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
    
    /// Get active connections
    pub fn get_active_connections(&self) -> Vec<String> {
        let connections = self.active_connections.read().unwrap();
        connections.keys().cloned().collect()
    }
    
    /// Send message to client
    pub fn send_message(&self, client_id: &str, message: IPCMessage) -> Result<()> {
        let mut connections = self.active_connections.write().unwrap();
        
        if let Some(connection) = connections.get_mut(client_id) {
            connection.send_message(message)?;
            Ok(())
        } else {
            Err(IPCError::ConnectionError(format!("Client {} not found", client_id)))
        }
    }
    
    /// Broadcast message to all clients
    pub fn broadcast_message(&self, message: IPCMessage) -> Result<()> {
        let mut connections = self.active_connections.write().unwrap();
        
        for (client_id, connection) in connections.iter_mut() {
            if let Err(e) = connection.send_message(message.clone()) {
                warn!("Failed to send message to client {}: {}", client_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Listener loop
    fn listener_loop(
        listener: UnixListener,
        active_connections: Arc<RwLock<HashMap<String, UnixSocketConnection>>>,
        auth_provider: Arc<dyn AuthenticationProvider>,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        while !shutdown_signal.load(Ordering::SeqCst) {
            // Accept new connections
            match listener.accept() {
                Ok((stream, _)) => {
                    // Generate client ID
                    let client_id = Uuid::new_v4().to_string();
                    
                    // Create connection
                    let connection = UnixSocketConnection::new(stream, &client_id);
                    
                    // Add to active connections
                    let mut connections = active_connections.write().unwrap();
                    connections.insert(client_id.clone(), connection);
                    
                    info!("New client connected: {}", client_id);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No new connections, sleep a bit
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    break;
                }
            }
        }
        
        debug!("Listener loop exited");
    }
    
    /// Connection handler loop
    fn connection_handler_loop(
        active_connections: Arc<RwLock<HashMap<String, UnixSocketConnection>>>,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        while !shutdown_signal.load(Ordering::SeqCst) {
            // Get client IDs
            let client_ids: Vec<String> = {
                let connections = active_connections.read().unwrap();
                connections.keys().cloned().collect()
            };
            
            // Process each client
            for client_id in client_ids {
                let mut remove_client = false;
                
                // Process messages
                {
                    let mut connections = active_connections.write().unwrap();
                    
                    if let Some(connection) = connections.get_mut(&client_id) {
                        // Check if connection is closed
                        if connection.is_shutdown_signaled() {
                            remove_client = true;
                            continue;
                        }
                        
                        // Receive message
                        match connection.receive_message() {
                            Ok(Some(message)) => {
                                // TODO: Handle message
                                debug!("Received message from client {}: {:?}", client_id, message);
                            }
                            Ok(None) => {
                                // No message
                            }
                            Err(e) => {
                                error!("Error receiving message from client {}: {}", client_id, e);
                                remove_client = true;
                            }
                        }
                    }
                }
                
                // Remove client if needed
                if remove_client {
                    let mut connections = active_connections.write().unwrap();
                    connections.remove(&client_id);
                    info!("Client disconnected: {}", client_id);
                }
            }
            
            // Sleep a bit
            thread::sleep(Duration::from_millis(10));
        }
        
        debug!("Connection handler loop exited");
    }
    
    /// Handle message
    fn handle_message(&self, client_id: &str, message: IPCMessage) -> Result<()> {
        // Find handler
        for (_, handler) in &self.message_handlers {
            if handler.supported_message_types().contains(&message.message_type) {
                // Handle message
                match handler.handle_message(message.clone()) {
                    Ok(Some(response)) => {
                        // Send response
                        self.send_message(client_id, response)?;
                    }
                    Ok(None) => {
                        // No response
                    }
                    Err(e) => {
                        error!("Error handling message: {}", e);
                        
                        // Send error response
                        let error_message = IPCMessage::error(&message, &e.to_string());
                        self.send_message(client_id, error_message)?;
                    }
                }
                
                return Ok(());
            }
        }
        
        // No handler found
        warn!("No handler found for message type {:?}", message.message_type);
        
        // Send error response
        let error_message = IPCMessage::error(&message, "No handler found");
        self.send_message(client_id, error_message)?;
        
        Ok(())
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        if self.listener_thread.is_some() {
            if let Err(e) = self.stop() {
                error!("Error stopping server: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;
    
    use crate::security::authentication::MockAuthenticationProvider;
    use crate::ipc::common::message::{MessagePayload, MessageType};
    
    #[test]
    fn test_server_start_stop() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        let auth_provider = Arc::new(MockAuthenticationProvider::new());
        let mut server = UnixSocketServer::new(&socket_path, auth_provider);
        
        // Start server
        server.start().unwrap();
        assert!(socket_path.exists());
        
        // Stop server
        server.stop().unwrap();
        assert!(!socket_path.exists());
    }
    
    // Additional tests would be added here
}
