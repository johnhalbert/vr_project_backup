//! Unix domain socket IPC implementation for the VR headset.
//!
//! This module provides IPC functionality using Unix domain sockets,
//! allowing for efficient local communication between processes.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::security::auth::AuthContext;
use super::common::{IpcConnection, IpcClient, IpcError, IpcResult, IpcServer, Message, MessageRouter};

/// Unix socket connection.
pub struct UnixSocketConnection {
    /// Connection ID
    id: String,
    
    /// Socket stream
    stream: Arc<Mutex<UnixStream>>,
    
    /// Authentication context
    auth_context: Arc<RwLock<AuthContext>>,
    
    /// Is the connection open
    is_open: Arc<RwLock<bool>>,
    
    /// Pending responses
    pending_responses: Arc<Mutex<HashMap<String, Arc<Mutex<Option<Message>>>>>>,
}

impl UnixSocketConnection {
    /// Create a new Unix socket connection.
    pub fn new(stream: UnixStream) -> Self {
        let id = Uuid::new_v4().to_string();
        
        // Set non-blocking mode
        let mut stream_clone = stream.try_clone().unwrap();
        stream_clone.set_nonblocking(true).unwrap();
        
        let connection = Self {
            id,
            stream: Arc::new(Mutex::new(stream)),
            auth_context: Arc::new(RwLock::new(AuthContext::new())),
            is_open: Arc::new(RwLock::new(true)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start the receiver thread
        connection.start_receiver();
        
        connection
    }
    
    /// Start the receiver thread.
    fn start_receiver(&self) {
        let stream = Arc::clone(&self.stream);
        let is_open = Arc::clone(&self.is_open);
        let pending_responses = Arc::clone(&self.pending_responses);
        
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            
            while *is_open.read().unwrap() {
                // Try to read from the socket
                let mut stream_guard = stream.lock().unwrap();
                match stream_guard.read(&mut buffer) {
                    Ok(0) => {
                        // Connection closed
                        *is_open.write().unwrap() = false;
                        break;
                    },
                    Ok(n) => {
                        // Process the received data
                        match Message::from_binary(&buffer[..n]) {
                            Ok(message) => {
                                // Check if this is a response to a pending request
                                if let Some(correlation_id) = &message.header.correlation_id {
                                    let pending = pending_responses.lock().unwrap();
                                    if let Some(response_slot) = pending.get(correlation_id) {
                                        let mut slot = response_slot.lock().unwrap();
                                        *slot = Some(message);
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Failed to deserialize message: {}", e);
                            },
                        }
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data available, sleep for a bit
                        drop(stream_guard);
                        thread::sleep(Duration::from_millis(10));
                    },
                    Err(e) => {
                        // Error reading from socket
                        error!("Error reading from socket: {}", e);
                        *is_open.write().unwrap() = false;
                        break;
                    },
                }
            }
            
            debug!("Receiver thread exiting");
        });
    }
}

impl IpcConnection for UnixSocketConnection {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn send(&self, message: &Message) -> IpcResult<()> {
        // Check if the connection is open
        if !self.is_open() {
            return Err(IpcError::Connection("Connection is closed".to_string()));
        }
        
        // Serialize the message
        let data = message.to_binary()?;
        
        // Send the message
        let mut stream = self.stream.lock().unwrap();
        stream.write_all(&data).map_err(|e| {
            // Mark the connection as closed on error
            *self.is_open.write().unwrap() = false;
            IpcError::Connection(format!("Failed to send message: {}", e))
        })?;
        
        Ok(())
    }
    
    fn send_and_receive(&self, message: &Message, timeout: Duration) -> IpcResult<Message> {
        // Check if the connection is open
        if !self.is_open() {
            return Err(IpcError::Connection("Connection is closed".to_string()));
        }
        
        // Create a response slot
        let response_slot = Arc::new(Mutex::new(None));
        
        // Register the response slot
        {
            let mut pending = self.pending_responses.lock().unwrap();
            pending.insert(message.header.id.clone(), Arc::clone(&response_slot));
        }
        
        // Send the message
        self.send(message)?;
        
        // Wait for the response
        let start_time = Instant::now();
        loop {
            // Check if we have a response
            {
                let slot = response_slot.lock().unwrap();
                if let Some(response) = &*slot {
                    // Remove the response slot
                    let mut pending = self.pending_responses.lock().unwrap();
                    pending.remove(&message.header.id);
                    
                    // Return the response
                    return Ok(response.clone());
                }
            }
            
            // Check if we've timed out
            if start_time.elapsed() >= timeout {
                // Remove the response slot
                let mut pending = self.pending_responses.lock().unwrap();
                pending.remove(&message.header.id);
                
                return Err(IpcError::Timeout(format!("Timed out waiting for response to message {}", message.header.id)));
            }
            
            // Sleep for a bit
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    fn close(&self) -> IpcResult<()> {
        // Mark the connection as closed
        *self.is_open.write().unwrap() = false;
        
        // Close the socket
        let stream = self.stream.lock().unwrap();
        stream.shutdown(std::net::Shutdown::Both).map_err(|e| {
            IpcError::Connection(format!("Failed to close connection: {}", e))
        })?;
        
        Ok(())
    }
    
    fn is_open(&self) -> bool {
        *self.is_open.read().unwrap()
    }
    
    fn auth_context(&self) -> Arc<RwLock<AuthContext>> {
        Arc::clone(&self.auth_context)
    }
    
    fn set_auth_context(&self, auth_context: AuthContext) -> IpcResult<()> {
        let mut current = self.auth_context.write().unwrap();
        *current = auth_context;
        Ok(())
    }
}

/// Unix socket server.
pub struct UnixSocketServer {
    /// Server address (socket path)
    address: PathBuf,
    
    /// Message router
    router: Arc<RwLock<MessageRouter>>,
    
    /// Server listener
    listener: Arc<Mutex<Option<UnixListener>>>,
    
    /// Is the server running
    is_running: Arc<RwLock<bool>>,
    
    /// Connected clients
    clients: Arc<RwLock<HashMap<String, Arc<UnixSocketConnection>>>>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server.
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            address: socket_path.as_ref().to_path_buf(),
            router: Arc::new(RwLock::new(MessageRouter::new())),
            listener: Arc::new(Mutex::new(None)),
            is_running: Arc::new(RwLock::new(false)),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the acceptor thread.
    fn start_acceptor(&self) -> IpcResult<()> {
        let address = self.address.clone();
        let router = Arc::clone(&self.router);
        let is_running = Arc::clone(&self.is_running);
        let clients = Arc::clone(&self.clients);
        
        // Create the listener
        let listener = UnixListener::bind(&address).map_err(|e| {
            IpcError::Connection(format!("Failed to bind to socket {}: {}", address.display(), e))
        })?;
        
        // Store the listener
        {
            let mut listener_guard = self.listener.lock().unwrap();
            *listener_guard = Some(listener.try_clone().unwrap());
        }
        
        // Start the acceptor thread
        thread::spawn(move || {
            for stream in listener.incoming() {
                // Check if the server is still running
                if !*is_running.read().unwrap() {
                    break;
                }
                
                match stream {
                    Ok(stream) => {
                        // Create a new connection
                        let connection = Arc::new(UnixSocketConnection::new(stream));
                        
                        // Store the connection
                        {
                            let mut clients_guard = clients.write().unwrap();
                            clients_guard.insert(connection.id().to_string(), Arc::clone(&connection));
                        }
                        
                        // Start the message handler thread
                        let connection_id = connection.id().to_string();
                        let connection_clone = Arc::clone(&connection);
                        let router_clone = Arc::clone(&router);
                        let clients_clone = Arc::clone(&clients);
                        
                        thread::spawn(move || {
                            // Handle messages until the connection is closed
                            while connection_clone.is_open() {
                                // Sleep to avoid busy-waiting
                                thread::sleep(Duration::from_millis(10));
                            }
                            
                            // Remove the connection
                            let mut clients_guard = clients_clone.write().unwrap();
                            clients_guard.remove(&connection_id);
                            
                            debug!("Client {} disconnected", connection_id);
                        });
                        
                        debug!("Client {} connected", connection.id());
                    },
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                    },
                }
            }
            
            debug!("Acceptor thread exiting");
        });
        
        Ok(())
    }
}

impl IpcServer for UnixSocketServer {
    fn start(&self) -> IpcResult<()> {
        // Check if the server is already running
        if self.is_running() {
            return Ok(());
        }
        
        // Remove the socket file if it exists
        if self.address.exists() {
            std::fs::remove_file(&self.address).map_err(|e| {
                IpcError::Connection(format!("Failed to remove existing socket file {}: {}", self.address.display(), e))
            })?;
        }
        
        // Create the parent directory if it doesn't exist
        if let Some(parent) = self.address.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                IpcError::Connection(format!("Failed to create directory {}: {}", parent.display(), e))
            })?;
        }
        
        // Mark the server as running
        *self.is_running.write().unwrap() = true;
        
        // Start the acceptor thread
        self.start_acceptor()?;
        
        info!("Unix socket server started at {}", self.address.display());
        
        Ok(())
    }
    
    fn stop(&self) -> IpcResult<()> {
        // Check if the server is running
        if !self.is_running() {
            return Ok(());
        }
        
        // Mark the server as stopped
        *self.is_running.write().unwrap() = false;
        
        // Close the listener
        {
            let mut listener_guard = self.listener.lock().unwrap();
            if let Some(listener) = listener_guard.take() {
                drop(listener);
            }
        }
        
        // Close all client connections
        {
            let mut clients_guard = self.clients.write().unwrap();
            for (_, client) in clients_guard.drain() {
                let _ = client.close();
            }
        }
        
        // Remove the socket file
        if self.address.exists() {
            std::fs::remove_file(&self.address).map_err(|e| {
                IpcError::Connection(format!("Failed to remove socket file {}: {}", self.address.display(), e))
            })?;
        }
        
        info!("Unix socket server stopped");
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        *self.is_running.read().unwrap()
    }
    
    fn address(&self) -> &str {
        self.address.to_str().unwrap_or("invalid-path")
    }
    
    fn router(&self) -> Arc<RwLock<MessageRouter>> {
        Arc::clone(&self.router)
    }
    
    fn set_router(&mut self, router: MessageRouter) -> IpcResult<()> {
        let mut router_guard = self.router.write().unwrap();
        *router_guard = router;
        Ok(())
    }
    
    fn broadcast(&self, message: &Message) -> IpcResult<()> {
        // Check if the server is running
        if !self.is_running() {
            return Err(IpcError::Connection("Server is not running".to_string()));
        }
        
        // Send the message to all clients
        let clients_guard = self.clients.read().unwrap();
        for (_, client) in clients_guard.iter() {
            let _ = client.send(message);
        }
        
        Ok(())
    }
    
    fn client_count(&self) -> usize {
        self.clients.read().unwrap().len()
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Unix socket client.
pub struct UnixSocketClient {
    /// Client ID
    id: String,
    
    /// Server address (socket path)
    server_address: PathBuf,
    
    /// Current connection
    connection: Arc<Mutex<Option<Arc<UnixSocketConnection>>>>,
    
    /// Is the client connected
    is_connected: Arc<RwLock<bool>>,
}

impl UnixSocketClient {
    /// Create a new Unix socket client.
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            server_address: socket_path.as_ref().to_path_buf(),
            connection: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }
}

impl IpcClient for UnixSocketClient {
    fn connect(&self) -> IpcResult<Box<dyn IpcConnection>> {
        // Check if already connected
        if self.is_connected() {
            let connection_guard = self.connection.lock().unwrap();
            if let Some(connection) = &*connection_guard {
                return Ok(Box::new(UnixSocketConnection {
                    id: connection.id().to_string(),
                    stream: Arc::clone(&connection.stream),
                    auth_context: Arc::clone(&connection.auth_context),
                    is_open: Arc::clone(&connection.is_open),
                    pending_responses: Arc::clone(&connection.pending_responses),
                }));
            }
        }
        
        // Connect to the server
        let stream = UnixStream::connect(&self.server_address).map_err(|e| {
            IpcError::Connection(format!("Failed to connect to socket {}: {}", self.server_address.display(), e))
        })?;
        
        // Create a new connection
        let connection = Arc::new(UnixSocketConnection::new(stream));
        
        // Store the connection
        {
            let mut connection_guard = self.connection.lock().unwrap();
            *connection_guard = Some(Arc::clone(&connection));
        }
        
        // Mark as connected
        *self.is_connected.write().unwrap() = true;
        
        // Return a new connection object
        Ok(Box::new(UnixSocketConnection {
            id: connection.id().to_string(),
            stream: Arc::clone(&connection.stream),
            auth_context: Arc::clone(&connection.auth_context),
            is_open: Arc::clone(&connection.is_open),
            pending_responses: Arc::clone(&connection.pending_responses),
        }))
    }
    
    fn is_connected(&self) -> bool {
        // Check the connected flag
        if !*self.is_connected.read().unwrap() {
            return false;
        }
        
        // Check if we have a connection
        let connection_guard = self.connection.lock().unwrap();
        if let Some(connection) = &*connection_guard {
            // Check if the connection is open
            if !connection.is_open() {
                // Update the connected flag
                drop(connection_guard);
                *self.is_connected.write().unwrap() = false;
                return false;
            }
            return true;
        }
        
        // No connection
        *self.is_connected.write().unwrap() = false;
        false
    }
    
    fn server_address(&self) -> &str {
        self.server_address.to_str().unwrap_or("invalid-path")
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn connection(&self) -> Option<Box<dyn IpcConnection>> {
        if !self.is_connected() {
            return None;
        }
        
        let connection_guard = self.connection.lock().unwrap();
        if let Some(connection) = &*connection_guard {
            return Some(Box::new(UnixSocketConnection {
                id: connection.id().to_string(),
                stream: Arc::clone(&connection.stream),
                auth_context: Arc::clone(&connection.auth_context),
                is_open: Arc::clone(&connection.is_open),
                pending_responses: Arc::clone(&connection.pending_responses),
            }));
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::Path;
    use super::super::common::MessagePayload;
    
    #[test]
    fn test_unix_socket_connection() {
        // Create a temporary directory for the socket
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        // Create a server and client socket
        let listener = UnixListener::bind(&socket_path).unwrap();
        let client_stream = UnixStream::connect(&socket_path).unwrap();
        
        // Accept the connection
        let (server_stream, _) = listener.accept().unwrap();
        
        // Create connections
        let server_connection = UnixSocketConnection::new(server_stream);
        let client_connection = UnixSocketConnection::new(client_stream);
        
        // Test sending a message from client to server
        let message = Message::new(
            "test.message",
            "client",
            "server",
            MessagePayload::string("test data"),
        );
        
        // Send the message
        client_connection.send(&message).unwrap();
        
        // Wait for the message to be received
        thread::sleep(Duration::from_millis(100));
        
        // Close the connections
        server_connection.close().unwrap();
        client_connection.close().unwrap();
    }
    
    #[test]
    fn test_unix_socket_server_client() {
        // Create a temporary directory for the socket
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        // Create a server
        let mut server = UnixSocketServer::new(&socket_path);
        
        // Register a message handler
        let handler = super::super::common::MessageHandler::new(
            "test.request",
            |message, _| {
                Ok(Message::response(
                    message,
                    MessagePayload::string("response data"),
                ))
            },
            false,
        );
        
        server.router_mut().register_handler(handler);
        
        // Start the server
        server.start().unwrap();
        
        // Create a client
        let client = UnixSocketClient::new(&socket_path);
        
        // Connect to the server
        let connection = client.connect().unwrap();
        
        // Send a request
        let request = Message::request(
            "test.request",
            "client",
            "server",
            MessagePayload::string("request data"),
        );
        
        // Send the request and wait for a response
        let response = connection.send_and_receive(&request, Duration::from_secs(1)).unwrap();
        
        // Check the response
        assert_eq!(response.header.message_type, "test.request.response");
        assert_eq!(response.header.sender, "server");
        assert_eq!(response.header.recipient, "client");
        assert_eq!(response.header.correlation_id, Some(request.header.id.clone()));
        assert_eq!(response.payload.as_string().unwrap(), "response data");
        
        // Close the connection
        connection.close().unwrap();
        
        // Stop the server
        server.stop().unwrap();
    }
}
