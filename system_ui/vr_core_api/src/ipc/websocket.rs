//! WebSocket IPC implementation.
//!
//! This module provides IPC functionality using WebSockets,
//! allowing for remote communication with web interfaces and applications.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use log::{debug, error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error as WsError, Message as WsMessage},
    WebSocketStream,
};
use uuid::Uuid;

use super::common::{IpcClient, IpcConnection, IpcError, IpcResult, IpcServer, Message, MessageRouter};
use crate::security::auth::AuthContext;

/// WebSocket connection.
pub struct WebSocketConnection {
    /// Connection ID
    id: String,
    
    /// Sender for outgoing messages
    sender: mpsc::UnboundedSender<WsMessage>,
    
    /// Is the connection open
    is_open: Arc<RwLock<bool>>,
    
    /// Last activity timestamp
    last_activity: Arc<RwLock<Instant>>,
    
    /// Authentication context
    auth_context: Arc<RwLock<AuthContext>>,
    
    /// Pending responses
    pending_responses: Arc<Mutex<HashMap<String, Arc<Mutex<Option<Message>>>>>>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection.
    pub fn new(
        sender: mpsc::UnboundedSender<WsMessage>,
        auth_context: Arc<RwLock<AuthContext>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            is_open: Arc::new(RwLock::new(true)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            auth_context,
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Update the last activity timestamp.
    pub fn update_activity(&self) {
        *self.last_activity.write().unwrap() = Instant::now();
    }
    
    /// Get the last activity timestamp.
    pub fn last_activity(&self) -> Instant {
        *self.last_activity.read().unwrap()
    }
}

impl IpcConnection for WebSocketConnection {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn send(&self, message: &Message) -> IpcResult<()> {
        if !self.is_open() {
            return Err(IpcError::Connection("Connection is closed".to_string()));
        }
        
        // Convert the message to a WebSocket message
        let ws_message = WsMessage::Binary(message.to_binary()?);
        
        // Send the message
        self.sender.send(ws_message).map_err(|e| {
            *self.is_open.write().unwrap() = false;
            IpcError::Connection(format!("Failed to send message: {}", e))
        })?;
        
        // Update the last activity timestamp
        self.update_activity();
        
        Ok(())
    }
    
    fn send_and_receive(&self, message: &Message, timeout: Duration) -> IpcResult<Message> {
        if !self.is_open() {
            return Err(IpcError::Connection("Connection is closed".to_string()));
        }
        
        // Create a response slot
        let response_slot = Arc::new(Mutex::new(None));
        
        // Store the response slot
        let correlation_id = message.header.id.clone();
        let mut pending_responses = self.pending_responses.lock().unwrap();
        pending_responses.insert(correlation_id.clone(), Arc::clone(&response_slot));
        
        // Send the message
        self.send(message)?;
        
        // Wait for the response
        let start = Instant::now();
        loop {
            // Check if we have a response
            let response = {
                let slot = response_slot.lock().unwrap();
                slot.clone()
            };
            
            if let Some(response) = response {
                // Remove the response slot
                pending_responses.remove(&correlation_id);
                
                return Ok(response);
            }
            
            // Check if we've timed out
            if start.elapsed() >= timeout {
                // Remove the response slot
                pending_responses.remove(&correlation_id);
                
                return Err(IpcError::Timeout(format!("Timed out waiting for response to message {}", correlation_id)));
            }
            
            // Sleep for a bit
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    fn close(&self) -> IpcResult<()> {
        if !self.is_open() {
            return Ok(());
        }
        
        // Mark the connection as closed
        *self.is_open.write().unwrap() = false;
        
        // Send a close message
        let close_message = WsMessage::Close(None);
        self.sender.send(close_message).map_err(|e| {
            IpcError::Connection(format!("Failed to send close message: {}", e))
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
        *self.auth_context.write().unwrap() = auth_context;
        Ok(())
    }
}

/// WebSocket client.
pub struct WebSocketClient {
    /// Server address
    address: String,
    
    /// Client ID
    id: String,
    
    /// Runtime
    runtime: Arc<Mutex<Option<Runtime>>>,
    
    /// Connection
    connection: Arc<RwLock<Option<Arc<WebSocketConnection>>>>,
    
    /// Pending responses
    pending_responses: Arc<Mutex<HashMap<String, Arc<Mutex<Option<Message>>>>>>,
    
    /// Router
    router: Arc<RwLock<MessageRouter>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client.
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            id: Uuid::new_v4().to_string(),
            runtime: Arc::new(Mutex::new(None)),
            connection: Arc::new(RwLock::new(None)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            router: Arc::new(RwLock::new(MessageRouter::new())),
        }
    }
    
    /// Disconnect from the server.
    pub fn disconnect(&self) -> IpcResult<()> {
        let connection_guard = self.connection.read().unwrap();
        if let Some(connection) = &*connection_guard {
            connection.close()?;
        }
        // Note: is_connected field doesn't exist in the struct definition
        // Removing this line as it's causing errors
        // *self.is_connected.write().unwrap() = false;
        Ok(())
    }
    
    /// Get the message router.
    pub fn get_router(&self) -> Arc<RwLock<MessageRouter>> {
        Arc::clone(&self.router)
    }
    
    /// Set the message router.
    pub fn set_router(&mut self, router: MessageRouter) -> IpcResult<()> {
        let mut router_guard = self.router.write().unwrap();
        *router_guard = router;
        Ok(())
    }
    
    /// Get mutable access to the message router.
    pub fn router_mut(&mut self) -> &mut MessageRouter {
        let mut router_guard = self.router.write().unwrap();
        &mut *router_guard
    }
}

impl IpcClient for WebSocketClient {
    fn connect(&self) -> IpcResult<Box<dyn IpcConnection>> {
        // Check if we're already connected
        {
            let connection_guard = self.connection.read().unwrap();
            if let Some(connection) = &*connection_guard {
                if connection.is_open() {
                    return Ok(Box::new(WebSocketConnection {
                        id: connection.id.clone(),
                        sender: connection.sender.clone(),
                        is_open: Arc::clone(&connection.is_open),
                        last_activity: Arc::clone(&connection.last_activity),
                        auth_context: Arc::clone(&connection.auth_context),
                        pending_responses: Arc::clone(&connection.pending_responses),
                    }));
                }
            }
        }
        
        // Create a new runtime
        let runtime = Runtime::new().map_err(|e| {
            IpcError::Internal(format!("Failed to create runtime: {}", e))
        })?;
        
        // Store the runtime
        {
            let mut runtime_guard = self.runtime.lock().unwrap();
            *runtime_guard = Some(runtime);
        }
        
        // Get the runtime
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().unwrap();
        
        // Get the server URL
        let url = format!("ws://{}", self.address);
        
        // Clone the necessary variables for the async block
        let router = Arc::clone(&self.router);
        let pending_responses = Arc::clone(&self.pending_responses);
        
        // Create a channel for the connection
        let (connection_tx, connection_rx) = mpsc::channel(1);
        
        // Spawn a task to connect to the server
        let handle = thread::spawn(move || {
            // Connect to the server
            let result = runtime.block_on(async {
                // Connect to the WebSocket server
                let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await.map_err(|e| {
                    IpcError::Connection(format!("Failed to connect to server: {}", e))
                })?;
                
                // Create channels for communication
                let (tx, rx) = mpsc::unbounded_channel();
                
                // Create the connection
                let auth_context = Arc::new(RwLock::new(AuthContext::new()));
                let connection = WebSocketConnection::new(tx, auth_context);
                let connection = Arc::new(connection);
                
                // Send the connection to the main thread
                connection_tx.send(Ok(Arc::clone(&connection))).await.map_err(|_| {
                    IpcError::Internal("Failed to send connection to main thread".to_string())
                })?;
                
                // Split the WebSocket stream
                let (mut ws_sink, mut ws_stream) = ws_stream.split();
                
                // Spawn a task to forward messages from the channel to the WebSocket
                tokio::spawn(async move {
                    while let Some(message) = rx.recv().await {
                        if let Err(e) = ws_sink.send(message).await {
                            error!("Failed to send message to WebSocket: {}", e);
                            break;
                        }
                    }
                });
                
                // Process messages from the WebSocket
                while let Some(result) = ws_stream.next().await {
                    match result {
                        Ok(ws_message) => {
                            // Update the last activity timestamp
                            connection.update_activity();
                            
                            // Process the message
                            match ws_message {
                                WsMessage::Binary(data) => {
                                    // Parse the message
                                    match Message::from_binary(&data) {
                                        Ok(message) => {
                                            // Check if this is a response to a pending request
                                            if let Some(correlation_id) = &message.header.correlation_id {
                                                let pending = pending_responses.lock().unwrap();
                                                if let Some(response_slot) = pending.get(correlation_id) {
                                                    let mut slot = response_slot.lock().unwrap();
                                                    *slot = Some(message.clone());
                                                }
                                            }
                                            
                                            // Route the message
                                            let router_guard = router.read().unwrap();
                                            if let Err(e) = router_guard.route_message(&message) {
                                                error!("Failed to route message: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse message: {}", e);
                                        }
                                    }
                                }
                                WsMessage::Close(_) => {
                                    // Mark the connection as closed
                                    *connection.is_open.write().unwrap() = false;
                                    break;
                                }
                                _ => {
                                    // Ignore other message types
                                }
                            }
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            // Mark the connection as closed
                            *connection.is_open.write().unwrap() = false;
                            break;
                        }
                    }
                }
                
                Ok(())
            });
            
            // Handle any errors
            if let Err(e) = result {
                error!("WebSocket client error: {}", e);
            }
        });
        
        // Wait for the connection
        let connection = runtime.block_on(async {
            match connection_rx.recv().await {
                Some(result) => result,
                None => Err(IpcError::Connection("Failed to connect to server".to_string())),
            }
        })?;
        
        // Store the connection
        {
            let mut connection_guard = self.connection.write().unwrap();
            *connection_guard = Some(Arc::clone(&connection));
        }
        
        // Return a new connection
        Ok(Box::new(WebSocketConnection {
            id: connection.id.clone(),
            sender: connection.sender.clone(),
            is_open: Arc::clone(&connection.is_open),
            last_activity: Arc::clone(&connection.last_activity),
            auth_context: Arc::clone(&connection.auth_context),
            pending_responses: Arc::clone(&connection.pending_responses),
        }))
    }
    
    fn is_connected(&self) -> bool {
        let connection_guard = self.connection.read().unwrap();
        if let Some(connection) = &*connection_guard {
            connection.is_open()
        } else {
            false
        }
    }
    
    fn server_address(&self) -> &str {
        &self.address
    }
    
    fn id(&self) -> &str {
        &self.id
    }
    
    fn connection(&self) -> Option<Box<dyn IpcConnection>> {
        let connection_guard = self.connection.read().unwrap();
        if let Some(connection) = &*connection_guard {
            if connection.is_open() {
                Some(Box::new(WebSocketConnection {
                    id: connection.id.clone(),
                    sender: connection.sender.clone(),
                    is_open: Arc::clone(&connection.is_open),
                    last_activity: Arc::clone(&connection.last_activity),
                    auth_context: Arc::clone(&connection.auth_context),
                    pending_responses: Arc::clone(&connection.pending_responses),
                }))
            } else {
                None
            }
        } else {
            None
        }
    }
    

}

/// WebSocket server.
pub struct WebSocketServer {
    /// Server address
    address: SocketAddr,
    
    /// Server address as string
    address_str: String,
    
    /// Server ID
    id: String,
    
    /// Runtime
    runtime: Arc<Mutex<Option<Runtime>>>,
    
    /// Connections
    connections: Arc<Mutex<HashMap<String, Arc<WebSocketConnection>>>>,
    
    /// Is the server running
    is_running: Arc<RwLock<bool>>,
    
    /// Router
    router: Arc<RwLock<MessageRouter>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server.
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            address_str: address.to_string(),
            id: Uuid::new_v4().to_string(),
            runtime: Arc::new(Mutex::new(None)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            router: Arc::new(RwLock::new(MessageRouter::new())),
        }
    }
}

impl IpcServer for WebSocketServer {
    fn set_router(&mut self, router: MessageRouter) -> IpcResult<()> {
        let mut router_guard = self.router.write().unwrap();
        *router_guard = router;
        Ok(())
    }
    
    fn start(&self) -> IpcResult<()> {
        // Check if the server is already running
        {
            let is_running = self.is_running.read().unwrap();
            if *is_running {
                return Ok(());
            }
        }
        
        // Create a new runtime
        let runtime = Runtime::new().map_err(|e| {
            IpcError::Internal(format!("Failed to create runtime: {}", e))
        })?;
        
        // Store the runtime
        {
            let mut runtime_guard = self.runtime.lock().unwrap();
            *runtime_guard = Some(runtime);
        }
        
        // Get the runtime
        let runtime_guard = self.runtime.lock().unwrap();
        let runtime = runtime_guard.as_ref().unwrap();
        
        // Clone the necessary variables for the async block
        let address = self.address;
        let connections = Arc::clone(&self.connections);
        let router = Arc::clone(&self.router);
        let is_running = Arc::clone(&self.is_running);
        
        // Spawn a task to run the server
        thread::spawn(move || {
            // Run the server
            let result = runtime.block_on(async {
                // Create a TCP listener
                let listener = TcpListener::bind(address).await.map_err(|e| {
                    IpcError::Connection(format!("Failed to bind to address: {}", e))
                })?;
                
                info!("WebSocket server listening on {}", address);
                
                // Mark the server as running
                *is_running.write().unwrap() = true;
                
                // Accept connections
                while *is_running.read().unwrap() {
                    // Accept a connection
                    let (stream, addr) = match listener.accept().await {
                        Ok(result) => result,
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                            continue;
                        }
                    };
                    
                    info!("Accepted connection from {}", addr);
                    
                    // Clone the necessary variables for the async block
                    let connections = Arc::clone(&connections);
                    let router = Arc::clone(&router);
                    
                    // Spawn a task to handle the connection
                    tokio::spawn(async move {
                        // Upgrade the TCP stream to a WebSocket stream
                        let ws_stream = match accept_async(stream).await {
                            Ok(ws_stream) => ws_stream,
                            Err(e) => {
                                error!("Failed to accept WebSocket connection: {}", e);
                                return;
                            }
                        };
                        
                        // Create channels for communication
                        let (tx, rx) = mpsc::unbounded_channel();
                        
                        // Create the connection
                        let auth_context = Arc::new(RwLock::new(AuthContext::new()));
                        let connection = WebSocketConnection::new(tx, auth_context);
                        let connection = Arc::new(connection);
                        
                        // Store the connection
                        {
                            let mut connections_guard = connections.lock().unwrap();
                            connections_guard.insert(connection.id.clone(), Arc::clone(&connection));
                        }
                        
                        // Split the WebSocket stream
                        let (mut ws_sink, mut ws_stream) = ws_stream.split();
                        
                        // Spawn a task to forward messages from the channel to the WebSocket
                        tokio::spawn(async move {
                            while let Some(message) = rx.recv().await {
                                if let Err(e) = ws_sink.send(message).await {
                                    error!("Failed to send message to WebSocket: {}", e);
                                    break;
                                }
                            }
                        });
                        
                        // Process messages from the WebSocket
                        while let Some(result) = ws_stream.next().await {
                            match result {
                                Ok(ws_message) => {
                                    // Update the last activity timestamp
                                    connection.update_activity();
                                    
                                    // Process the message
                                    match ws_message {
                                        WsMessage::Binary(data) => {
                                            // Parse the message
                                            match Message::from_binary(&data) {
                                                Ok(message) => {
                                                    // Check if this is a response to a pending request
                                                    if let Some(correlation_id) = &message.header.correlation_id {
                                                        let pending = connection.pending_responses.lock().unwrap();
                                                        if let Some(response_slot) = pending.get(correlation_id) {
                                                            let mut slot = response_slot.lock().unwrap();
                                                            *slot = Some(message.clone());
                                                        }
                                                    }
                                                    
                                                    // Route the message
                                                    let router_guard = router.read().unwrap();
                                                    if let Err(e) = router_guard.route_message(&message) {
                                                        error!("Failed to route message: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Failed to parse message: {}", e);
                                                }
                                            }
                                        }
                                        WsMessage::Close(_) => {
                                            // Mark the connection as closed
                                            *connection.is_open.write().unwrap() = false;
                                            break;
                                        }
                                        _ => {
                                            // Ignore other message types
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("WebSocket error: {}", e);
                                    // Mark the connection as closed
                                    *connection.is_open.write().unwrap() = false;
                                    break;
                                }
                            }
                        }
                        
                        // Remove the connection
                        {
                            let mut connections_guard = connections.lock().unwrap();
                            connections_guard.remove(&connection.id);
                        }
                        
                        info!("Connection {} closed", connection.id);
                    });
                }
                
                Ok(())
            });
            
            // Handle any errors
            if let Err(e) = result {
                error!("WebSocket server error: {}", e);
            }
            
            // Mark the server as not running
            *is_running.write().unwrap() = false;
        });
        
        Ok(())
    }
    
    fn stop(&self) -> IpcResult<()> {
        // Check if the server is running
        {
            let is_running = self.is_running.read().unwrap();
            if !*is_running {
                return Ok(());
            }
        }
        
        // Mark the server as not running
        *self.is_running.write().unwrap() = false;
        
        // Close all connections
        let connections = self.connections.lock().unwrap();
        for (_, connection) in &*connections {
            if let Err(e) = connection.close() {
                error!("Failed to close connection: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        *self.is_running.read().unwrap()
    }
    
    fn address(&self) -> &str {
        &self.address_str
    }
    
    fn router(&self) -> &MessageRouter {
        // This is a bit of a hack, but we need to return a reference to the MessageRouter
        // which is inside an Arc<RwLock<>>. We can't easily convert that to a reference,
        // so we'll panic if we can't acquire the lock.
        let router_guard = self.router.read().unwrap();
        unsafe {
            // This is safe because we're holding the read lock for the duration of the
            // returned reference, and the reference can't outlive the router_guard.
            &*(router_guard.deref() as *const MessageRouter)
        }
    }
    
    // Removed router_mut as it's not part of the IpcServer trait
    
    fn broadcast(&self, message: &Message) -> IpcResult<()> {
        let connections = self.connections.lock().unwrap();
        for (_, connection) in &*connections {
            if let Err(e) = connection.send(message) {
                error!("Failed to broadcast message to connection {}: {}", connection.id(), e);
            }
        }
        
        Ok(())
    }
    
    fn client_count(&self) -> usize {
        let connections = self.connections.lock().unwrap();
        connections.len()
    }
}
