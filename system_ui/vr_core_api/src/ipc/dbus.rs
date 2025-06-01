//! D-Bus IPC implementation for the VR headset.
//!
//! This module provides IPC functionality using D-Bus,
//! allowing for communication with system services and other applications.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use dbus::{
    arg::{RefArg, Get},
    blocking::Connection,
    channel::Sender,
    message::{MatchRule, Message as DbusMessage, MessageType},
};
use log::{debug, error, info, warn};
use uuid::Uuid;

use crate::security::auth::AuthContext;
use super::common::{IpcConnection, IpcClient, IpcError, IpcResult, IpcServer, Message, MessagePayload, MessageRouter};

/// D-Bus connection.
pub struct DBusConnection {
    /// Connection ID
    id: String,
    
    /// D-Bus connection
    connection: Arc<Mutex<Connection>>,
    
    /// Authentication context
    auth_context: Arc<RwLock<AuthContext>>,
    
    /// Is the connection open
    is_open: Arc<RwLock<bool>>,
    
    /// Pending responses
    pending_responses: Arc<Mutex<HashMap<String, Arc<Mutex<Option<Message>>>>>>,
    
    /// Service name
    service_name: String,
    
    /// Object path
    object_path: String,
    
    /// Interface name
    interface_name: String,
}

impl DBusConnection {
    /// Create a new D-Bus connection.
    pub fn new(
        connection: Connection,
        service_name: &str,
        object_path: &str,
        interface_name: &str,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        
        let connection = Self {
            id,
            connection: Arc::new(Mutex::new(connection)),
            auth_context: Arc::new(RwLock::new(AuthContext::new())),
            is_open: Arc::new(RwLock::new(true)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            service_name: service_name.to_string(),
            object_path: object_path.to_string(),
            interface_name: interface_name.to_string(),
        };
        
        // Start the receiver thread
        connection.start_receiver();
        
        connection
    }
    
    /// Start the receiver thread.
    fn start_receiver(&self) {
        let connection = Arc::clone(&self.connection);
        let is_open = Arc::clone(&self.is_open);
        let pending_responses = Arc::clone(&self.pending_responses);
        let service_name = self.service_name.clone();
        let object_path = self.object_path.clone();
        let interface_name = self.interface_name.clone();
        
        thread::spawn(move || {
            // Create a match rule for incoming messages
            let mut rule = MatchRule::new();
            rule.path = Some(object_path.clone().into());
            rule.interface = Some(interface_name.clone().into());
            
            // Add the match rule
            let conn = connection.lock().unwrap();
            // Store the token for later removal
            let token_result = conn.add_match(rule.clone(), move |_: (), _, msg| {
                // Process the received message
                if let Some(response) = Self::dbus_to_ipc_message(msg, &service_name, &object_path, &interface_name) {
                    // Check if this is a response to a pending request
                    if let Some(correlation_id) = &response.header.correlation_id {
                        let mut pending = pending_responses.lock().unwrap();
                        if let Some(response_slot) = pending.get(correlation_id) {
                            let mut slot = response_slot.lock().unwrap();
                            *slot = Some(response);
                        }
                    }
                }
                
                true
            });
            
            // Process messages until the connection is closed
            while *is_open.read().unwrap() {
                // Process messages for 100ms
                let _ = conn.process(Duration::from_millis(100));
            }
            
            // Remove the match rule
            if let Ok(token) = token_result {
                let _ = conn.remove_match(token);
            }
            
            debug!("Receiver thread exiting");
        });
    }
    
    /// Convert a D-Bus message to an IPC message.
    fn dbus_to_ipc_message(
        msg: &dbus::Message,
        service_name: &str,
        object_path: &str,
        interface_name: &str,
    ) -> Option<Message> {
        // Check if this is a method call or signal
        if msg.msg_type() != MessageType::MethodCall && msg.msg_type() != MessageType::Signal {
            return None;
        }
        
        // Get the message path, interface, and member
        let path = msg.path()?.to_string();
        let interface = msg.interface()?.to_string();
        let member = msg.member()?.to_string();
        
        // Check if this is for our interface
        if path != object_path || interface != interface_name {
            return None;
        }
        
        // Get the message body
        let body = msg.get_items();
        if body.is_empty() {
            return None;
        }
        
        // Try to extract the IPC message from the body
        if let Some(json_str) = body[0].as_str() {
            // Try to parse the JSON string as an IPC message
            if let Ok(ipc_message) = Message::from_json(json_str) {
                return Some(ipc_message);
            }
        }
        
        // If we couldn't extract an IPC message, create one from the D-Bus message
        let message_type = format!("{}.{}", interface, member);
        let sender = msg.sender().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string());
        let recipient = service_name.to_string();
        
        // Create the payload
        let payload = if body.len() == 1 {
            // Manually convert MessageItem to a JSON string
            let json_value = message_item_to_json_value(&body[0]);
            let json = json_value.to_string();
            MessagePayload::string(&json)
        } else {
            // Manually convert MessageItems to a JSON string
            let mut json_array = Vec::new();
            for item in &body {
                json_array.push(message_item_to_json_value(item));
            }
            let json_value = serde_json::Value::Array(json_array);
            let json = json_value.to_string();
            MessagePayload::string(&json)
        };
        
        // Create the IPC message
        Some(Message::new(&message_type, &sender, &recipient, payload))
    }
    
    /// Convert an IPC message to a D-Bus message.
    fn ipc_to_dbus_message(&self, message: &Message) -> dbus::Message {
        // Create a new method call
        let mut msg = dbus::Message::new_method_call(
            &self.service_name,
            &self.object_path,
            &self.interface_name,
            &message.header.message_type,
        ).unwrap();
        
        // Add the serialized IPC message as the body
        if let Ok(json) = message.to_json() {
            msg = msg.append1(json);
        }
        
        msg
    }
}

impl IpcConnection for DBusConnection {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn send(&self, message: &Message) -> IpcResult<()> {
        // Check if the connection is open
        if !self.is_open() {
            return Err(IpcError::Connection("Connection is closed".to_string()));
        }
        
        // Convert the IPC message to a D-Bus message
        let dbus_message = self.ipc_to_dbus_message(message);
        
        // Send the message
        let conn = self.connection.lock().unwrap();
        conn.send(dbus_message).map_err(|e| {
            // Mark the connection as closed on error
            *self.is_open.write().unwrap() = false;
            IpcError::Connection(format!("Failed to send message: {:?}", e))
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

/// D-Bus server.
pub struct DBusServer {
    /// Server address (service name)
    address: String,
    
    /// Message router
    router: Arc<RwLock<MessageRouter>>,
    
    /// D-Bus connection
    connection: Arc<Mutex<Option<Connection>>>,
    
    /// Is the server running
    is_running: Arc<RwLock<bool>>,
    
    /// Object path
    object_path: String,
    
    /// Interface name
    interface_name: String,
    
    /// Connected clients
    clients: Arc<RwLock<HashMap<String, Arc<DBusConnection>>>>,
}

impl DBusServer {
    /// Create a new D-Bus server.
    pub fn new(
        service_name: &str,
        object_path: &str,
        interface_name: &str,
    ) -> Self {
        Self {
            address: service_name.to_string(),
            router: Arc::new(RwLock::new(MessageRouter::new())),
            connection: Arc::new(Mutex::new(None)),
            is_running: Arc::new(RwLock::new(false)),
            object_path: object_path.to_string(),
            interface_name: interface_name.to_string(),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the server thread.
    fn start_server(&self) -> IpcResult<()> {
        let service_name = self.address.clone();
        let object_path = self.object_path.clone();
        let interface_name = self.interface_name.clone();
        let router = Arc::clone(&self.router);
        let is_running = Arc::clone(&self.is_running);
        let clients = Arc::clone(&self.clients);
        
        // Connect to the session bus
        let connection = Connection::new_session().map_err(|e| {
            IpcError::Connection(format!("Failed to connect to D-Bus session bus: {}", e))
        })?;
        
        // Request a name on the bus
        let result = connection.request_name(
            &service_name,
            dbus::name_flag::NameFlag::ReplaceExisting as u32,
            false,
            false
        ).map_err(|e| {
            IpcError::Connection(format!("Failed to request name on D-Bus: {}", e))
        })?;
        
        // Convert the result to the expected type
        let result_enum = dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::from(result);
        
        if result_enum != dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner {
            return Err(IpcError::Connection(format!("Failed to become primary owner of name {}", service_name)));
        }
                // Store the connection
        {
            let mut connection_guard = self.connection.lock().unwrap();
            *connection_guard = Some(connection);
        }
        
        // Create a match rule for incoming messages
        let mut rule = MatchRule::new();
        rule.path = Some(object_path.clone().into());
        rule.interface = Some(interface_name.clone().into());
        
        // Add the match rule
        connection.add_match(rule.clone(), move |_: (), _, msg| {
            // Check if the server is still running
            if !*is_running.read().unwrap() {
                return true;
            }
            
            // Process the received message
            if let Some(ipc_message) = DBusConnection::dbus_to_ipc_message(msg, &service_name, &object_path, &interface_name) {
                // Create an auth context
                let auth_context = AuthContext::new();
                
                // Route the message
                let router_guard = router.read().unwrap();
                if let Ok(response) = router_guard.route(&ipc_message, &auth_context) {
                    // Send the response
                    let dbus_response = msg.method_return().append1(response.to_json().unwrap_or_default());
                    let _ = connection.send(dbus_response);
                }
            }
            
            true
        }).map_err(|e| {
            IpcError::Connection(format!("Failed to add match rule: {}", e))
        })?;
        
        // Start the server thread
        thread::spawn(move || {
            // Process messages until the server is stopped
            while *is_running.read().unwrap() {
                // Process messages for 100ms
                let _ = connection.process(Duration::from_millis(100));
            }
            
            // Remove the match rule
            // Note: We don't store match tokens in this implementation,
            // so we can't remove specific match rules
            debug!("Match rule removal would happen here if tokens were stored");
            
            debug!("Server thread exiting");
        });
        
        Ok(())
    }
}

impl IpcServer for DBusServer {
    fn start(&self) -> IpcResult<()> {
        // Check if the server is already running
        if self.is_running() {
            return Ok(());
        }
        
        // Mark the server as running
        *self.is_running.write().unwrap() = true;
        
        // Start the server thread
        self.start_server()?;
        
        info!("D-Bus server started with service name {}", self.address);
        
        Ok(())
    }
    
    fn stop(&self) -> IpcResult<()> {
        // Check if the server is running
        if !self.is_running() {
            return Ok(());
        }
        
        // Mark the server as stopped
        *self.is_running.write().unwrap() = false;
        
        // Close the connection
        {
            let mut connection_guard = self.connection.lock().unwrap();
            *connection_guard = None;
        }
        
        // Close all client connections
        {
            let mut clients_guard = self.clients.write().unwrap();
            for (_, client) in clients_guard.drain() {
                let _ = client.close();
            }
        }
        
        info!("D-Bus server stopped");
        
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        *self.is_running.read().unwrap()
    }
    
    fn address(&self) -> &str {
        &self.address
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
        
        // Get the connection
        let connection_guard = self.connection.lock().unwrap();
        let connection = connection_guard.as_ref().ok_or_else(|| {
            IpcError::Connection("Server connection is not available".to_string())
        })?;
        
        // Create a signal message
        let mut signal = dbus::Message::new_signal(
            &self.object_path,
            &self.interface_name,
            &message.header.message_type,
        ).map_err(|e| {
            IpcError::Connection(format!("Failed to create signal: {}", e))
        })?;
        
        // Add the serialized IPC message as the body
        if let Ok(json) = message.to_json() {
            signal = signal.append1(json);
        }
        
        // Send the signal
        connection.send(signal).map_err(|e| {
            IpcError::Connection(format!("Failed to send signal: {:?}", e))
        })?;
        
        Ok(())
    }
    
    fn client_count(&self) -> usize {
        self.clients.read().unwrap().len()
    }
}

impl Drop for DBusServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// D-Bus client.
pub struct DBusClient {
    /// Client ID
    id: String,
    
    /// Server address (service name)
    server_address: String,
    
    /// Current connection
    connection: Arc<Mutex<Option<Arc<DBusConnection>>>>,
    
    /// Is the client connected
    is_connected: Arc<RwLock<bool>>,
    
    /// Object path
    object_path: String,
    
    /// Interface name
    interface_name: String,
}

impl DBusClient {
    /// Create a new D-Bus client.
    pub fn new(
        service_name: &str,
        object_path: &str,
        interface_name: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            server_address: service_name.to_string(),
            connection: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(RwLock::new(false)),
            object_path: object_path.to_string(),
            interface_name: interface_name.to_string(),
        }
    }
}

impl IpcClient for DBusClient {
    fn connect(&self) -> IpcResult<Box<dyn IpcConnection>> {
        // Check if already connected
        if self.is_connected() {
            let connection_guard = self.connection.lock().unwrap();
            if let Some(connection) = &*connection_guard {
                return Ok(Box::new(DBusConnection {
                    id: connection.id().to_string(),
                    connection: Arc::clone(&connection.connection),
                    auth_context: Arc::clone(&connection.auth_context),
                    is_open: Arc::clone(&connection.is_open),
                    pending_responses: Arc::clone(&connection.pending_responses),
                    service_name: connection.service_name.clone(),
                    object_path: connection.object_path.clone(),
                    interface_name: connection.interface_name.clone(),
                }));
            }
        }
        
        // Connect to the session bus
        let connection = Connection::new_session().map_err(|e| {
            IpcError::Connection(format!("Failed to connect to D-Bus session bus: {}", e))
        })?;
        
        // Create a new connection
        let dbus_connection = Arc::new(DBusConnection::new(
            connection,
            &self.server_address,
            &self.object_path,
            &self.interface_name,
        ));
        
        // Store the connection
        {
            let mut connection_guard = self.connection.lock().unwrap();
            *connection_guard = Some(Arc::clone(&dbus_connection));
        }
        
        // Mark as connected
        *self.is_connected.write().unwrap() = true;
        
        // Return a new connection object
        Ok(Box::new(DBusConnection {
            id: dbus_connection.id().to_string(),
            connection: Arc::clone(&dbus_connection.connection),
            auth_context: Arc::clone(&dbus_connection.auth_context),
            is_open: Arc::clone(&dbus_connection.is_open),
            pending_responses: Arc::clone(&dbus_connection.pending_responses),
            service_name: dbus_connection.service_name.clone(),
            object_path: dbus_connection.object_path.clone(),
            interface_name: dbus_connection.interface_name.clone(),
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
        &self.server_address
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
            return Some(Box::new(DBusConnection {
                id: connection.id().to_string(),
                connection: Arc::clone(&connection.connection),
                auth_context: Arc::clone(&connection.auth_context),
                is_open: Arc::clone(&connection.is_open),
                pending_responses: Arc::clone(&connection.pending_responses),
                service_name: connection.service_name.clone(),
                object_path: connection.object_path.clone(),
                interface_name: connection.interface_name.clone(),
            }));
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::common::MessagePayload;
    
    // These tests require a running D-Bus session bus, which may not be available
    // in all environments. They are marked as ignored by default.
    
    #[test]
    #[ignore]
    fn test_dbus_server_client() {
        // Create a server
        let mut server = DBusServer::new(
            "org.vr.test",
            "/org/vr/test",
            "org.vr.test.Interface",
        );
        
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
        let client = DBusClient::new(
            "org.vr.test",
            "/org/vr/test",
            "org.vr.test.Interface",
        );
        
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

// Convert MessageItem to a JSON-compatible value
fn message_item_to_json_value(item: &MessageItem) -> serde_json::Value {
    match item {
        MessageItem::Array(items) => {
            let values: Vec<serde_json::Value> = items.iter()
                .map(|i| message_item_to_json_value(i))
                .collect();
            serde_json::Value::Array(values)
        },
        MessageItem::Bool(b) => serde_json::Value::Bool(*b),
        MessageItem::Byte(b) => serde_json::Value::Number((*b).into()),
        MessageItem::Int16(i) => serde_json::Value::Number((*i).into()),
        MessageItem::Int32(i) => serde_json::Value::Number((*i).into()),
        MessageItem::Int64(i) => serde_json::json!(*i),
        MessageItem::UInt16(i) => serde_json::Value::Number((*i).into()),
        MessageItem::UInt32(i) => serde_json::json!(*i),
        MessageItem::UInt64(i) => serde_json::json!(*i),
        MessageItem::Double(d) => serde_json::json!(*d),
        MessageItem::String(s) => serde_json::Value::String(s.clone()),
        MessageItem::ObjectPath(p) => serde_json::Value::String(p.to_string()),
        MessageItem::Signature(s) => serde_json::Value::String(s.to_string()),
        MessageItem::Variant(v) => message_item_to_json_value(v),
        MessageItem::Dict(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                if let MessageItem::String(key) = k {
                    map.insert(key.clone(), message_item_to_json_value(v));
                }
            }
            serde_json::Value::Object(map)
        },
        _ => serde_json::Value::Null,
    }
}
