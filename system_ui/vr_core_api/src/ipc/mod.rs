//! Inter-Process Communication (IPC) module for the VR Core API.
//!
//! This module provides functionality for communication between different
//! components of the VR headset system, including Unix domain sockets,
//! D-Bus, and WebSocket implementations.

pub mod common;
pub mod unix_socket;
pub mod dbus;
pub mod websocket;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};

use crate::security::authentication::AuthenticationProvider;
use common::{IPCError, Result, IPCMessage, MessageHandler};

/// IPC manager for the VR system.
#[derive(Debug)]
pub struct IPCManager {
    /// Unix socket server
    unix_socket_server: Option<unix_socket::UnixSocketServer>,
    
    /// D-Bus service
    dbus_service: Option<dbus::DBusService>,
    
    /// WebSocket server
    websocket_server: Option<websocket::WebSocketServer>,
    
    /// Authentication provider
    auth_provider: Arc<dyn AuthenticationProvider>,
    
    /// Message handlers
    message_handlers: Arc<RwLock<HashMap<String, Box<dyn MessageHandler>>>>,
    
    /// Configuration
    config: IPCConfig,
}

/// IPC configuration.
#[derive(Debug, Clone)]
pub struct IPCConfig {
    /// Unix socket configuration
    pub unix_socket: UnixSocketConfig,
    
    /// D-Bus configuration
    pub dbus: DBusConfig,
    
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
}

/// Unix socket configuration.
#[derive(Debug, Clone)]
pub struct UnixSocketConfig {
    /// Whether Unix socket is enabled
    pub enabled: bool,
    
    /// Socket path
    pub socket_path: PathBuf,
}

/// D-Bus configuration.
#[derive(Debug, Clone)]
pub struct DBusConfig {
    /// Whether D-Bus is enabled
    pub enabled: bool,
    
    /// Service name
    pub service_name: String,
    
    /// Object path
    pub object_path: String,
}

/// WebSocket configuration.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Whether WebSocket is enabled
    pub enabled: bool,
    
    /// Bind address
    pub bind_address: String,
    
    /// Port
    pub port: u16,
    
    /// Whether to use TLS
    pub use_tls: bool,
    
    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,
    
    /// TLS key path
    pub tls_key_path: Option<PathBuf>,
}

impl IPCManager {
    /// Create a new IPCManager.
    pub fn new(config: IPCConfig, auth_provider: Arc<dyn AuthenticationProvider>) -> Self {
        Self {
            unix_socket_server: None,
            dbus_service: None,
            websocket_server: None,
            auth_provider,
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Start all enabled IPC services.
    pub fn start(&mut self) -> Result<()> {
        // Start Unix socket server if enabled
        if self.config.unix_socket.enabled {
            info!("Starting Unix socket server at {:?}", self.config.unix_socket.socket_path);
            let mut server = unix_socket::UnixSocketServer::new(
                &self.config.unix_socket.socket_path,
                self.auth_provider.clone(),
            );
            
            // Register message handlers
            let handlers = self.message_handlers.read().unwrap();
            for (_, handler) in handlers.iter() {
                server.register_handler(Box::new(handler.clone()))?;
            }
            
            server.start()?;
            self.unix_socket_server = Some(server);
        }
        
        // Start D-Bus service if enabled
        if self.config.dbus.enabled {
            info!("Starting D-Bus service {} at {}", self.config.dbus.service_name, self.config.dbus.object_path);
            let mut service = dbus::DBusService::new(
                &self.config.dbus.service_name,
                &self.config.dbus.object_path,
                self.auth_provider.clone(),
            )?;
            
            // Register interfaces
            // TODO: Register D-Bus interfaces
            
            service.start()?;
            self.dbus_service = Some(service);
        }
        
        // Start WebSocket server if enabled
        if self.config.websocket.enabled {
            info!("Starting WebSocket server at {}:{}", self.config.websocket.bind_address, self.config.websocket.port);
            let mut server = websocket::WebSocketServer::new(
                &self.config.websocket.bind_address,
                self.config.websocket.port,
                self.auth_provider.clone(),
                self.config.websocket.use_tls,
                self.config.websocket.tls_cert_path.as_deref(),
                self.config.websocket.tls_key_path.as_deref(),
            )?;
            
            // Register message handlers
            let handlers = self.message_handlers.read().unwrap();
            for (_, handler) in handlers.iter() {
                server.register_handler(Box::new(handler.clone()))?;
            }
            
            server.start()?;
            self.websocket_server = Some(server);
        }
        
        Ok(())
    }
    
    /// Stop all IPC services.
    pub fn stop(&mut self) -> Result<()> {
        // Stop Unix socket server
        if let Some(ref mut server) = self.unix_socket_server {
            info!("Stopping Unix socket server");
            server.stop()?;
        }
        self.unix_socket_server = None;
        
        // Stop D-Bus service
        if let Some(ref mut service) = self.dbus_service {
            info!("Stopping D-Bus service");
            service.stop()?;
        }
        self.dbus_service = None;
        
        // Stop WebSocket server
        if let Some(ref mut server) = self.websocket_server {
            info!("Stopping WebSocket server");
            server.stop()?;
        }
        self.websocket_server = None;
        
        Ok(())
    }
    
    /// Register a message handler.
    pub fn register_handler(&self, handler: Box<dyn MessageHandler>) -> Result<()> {
        let handler_id = handler.id().to_string();
        
        // Register with message handlers
        let mut handlers = self.message_handlers.write().unwrap();
        handlers.insert(handler_id.clone(), handler);
        
        // Register with active services
        if let Some(ref mut server) = self.unix_socket_server {
            if let Some(handler) = handlers.get(&handler_id) {
                server.register_handler(Box::new(handler.clone()))?;
            }
        }
        
        if let Some(ref mut server) = self.websocket_server {
            if let Some(handler) = handlers.get(&handler_id) {
                server.register_handler(Box::new(handler.clone()))?;
            }
        }
        
        Ok(())
    }
    
    /// Unregister a message handler.
    pub fn unregister_handler(&self, id: &str) -> Result<()> {
        // Unregister from message handlers
        let mut handlers = self.message_handlers.write().unwrap();
        handlers.remove(id);
        
        // Unregister from active services
        if let Some(ref mut server) = self.unix_socket_server {
            server.unregister_handler(id)?;
        }
        
        if let Some(ref mut server) = self.websocket_server {
            server.unregister_handler(id)?;
        }
        
        Ok(())
    }
    
    /// Send a message to a client.
    pub fn send_message(&self, client_id: &str, message: IPCMessage) -> Result<()> {
        // Try to send via Unix socket
        if let Some(ref server) = self.unix_socket_server {
            if let Ok(()) = server.send_message(client_id, message.clone()) {
                return Ok(());
            }
        }
        
        // Try to send via WebSocket
        if let Some(ref server) = self.websocket_server {
            if let Ok(()) = server.send_message(client_id, message.clone()) {
                return Ok(());
            }
        }
        
        Err(IPCError::ConnectionError(format!("Failed to send message to client {}", client_id)))
    }
    
    /// Broadcast a message to all clients.
    pub fn broadcast_message(&self, message: IPCMessage) -> Result<()> {
        let mut success = false;
        
        // Broadcast via Unix socket
        if let Some(ref server) = self.unix_socket_server {
            if server.broadcast_message(message.clone()).is_ok() {
                success = true;
            }
        }
        
        // Broadcast via WebSocket
        if let Some(ref server) = self.websocket_server {
            if server.broadcast_message(message.clone()).is_ok() {
                success = true;
            }
        }
        
        if success {
            Ok(())
        } else {
            Err(IPCError::ConnectionError("Failed to broadcast message to any clients".to_string()))
        }
    }
    
    /// Get active client connections.
    pub fn get_active_connections(&self) -> Vec<String> {
        let mut connections = Vec::new();
        
        // Get Unix socket connections
        if let Some(ref server) = self.unix_socket_server {
            connections.extend(server.get_active_connections());
        }
        
        // Get WebSocket connections
        if let Some(ref server) = self.websocket_server {
            connections.extend(server.get_active_connections());
        }
        
        connections
    }
}

impl Default for IPCConfig {
    fn default() -> Self {
        Self {
            unix_socket: UnixSocketConfig {
                enabled: true,
                socket_path: PathBuf::from("/tmp/vr_core_api.sock"),
            },
            dbus: DBusConfig {
                enabled: true,
                service_name: "org.vr.CoreAPI".to_string(),
                object_path: "/org/vr/CoreAPI".to_string(),
            },
            websocket: WebSocketConfig {
                enabled: true,
                bind_address: "127.0.0.1".to_string(),
                port: 9000,
                use_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::authentication::MockAuthenticationProvider;
    use std::path::Path;
    
    #[test]
    fn test_ipc_manager_creation() {
        let auth_provider = Arc::new(MockAuthenticationProvider::new());
        let config = IPCConfig::default();
        let manager = IPCManager::new(config, auth_provider);
        
        assert!(manager.unix_socket_server.is_none());
        assert!(manager.dbus_service.is_none());
        assert!(manager.websocket_server.is_none());
    }
    
    // Additional tests would be added here
}
