//! D-Bus object implementation for IPC mechanisms.
//!
//! This module provides object implementation for D-Bus IPC,
//! including object path and interface management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dbus::arg::ArgType;
use dbus::message::{Message, MessageItem, MessageType as DBusMessageType};
use dbus::tree::{Factory, Interface, MTFn, Method, MethodErr, Signal};
use log::{debug, error, info, warn};

use crate::security::authentication::AuthToken;
use crate::ipc::common::{IPCError, Result};
use super::interface::{DBusInterface, DBusMethod, DBusSignal, DBusProperty, DBusPropertyAccess};

/// D-Bus object
pub struct DBusObject {
    /// Object path
    path: String,
    
    /// Interfaces
    interfaces: HashMap<String, Box<dyn DBusInterface>>,
}

impl DBusObject {
    /// Create a new DBusObject
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            interfaces: HashMap::new(),
        }
    }
    
    /// Get object path
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Add interface
    pub fn add_interface(&mut self, interface: Box<dyn DBusInterface>) -> Result<()> {
        let name = interface.name().to_string();
        
        if self.interfaces.contains_key(&name) {
            return Err(IPCError::InternalError(format!("Interface {} already exists", name)));
        }
        
        self.interfaces.insert(name, interface);
        
        Ok(())
    }
    
    /// Remove interface
    pub fn remove_interface(&mut self, name: &str) -> Result<()> {
        if !self.interfaces.contains_key(name) {
            return Err(IPCError::InternalError(format!("Interface {} not found", name)));
        }
        
        self.interfaces.remove(name);
        
        Ok(())
    }
    
    /// Get interface
    pub fn get_interface(&self, name: &str) -> Option<&Box<dyn DBusInterface>> {
        self.interfaces.get(name)
    }
    
    /// Get interfaces
    pub fn get_interfaces(&self) -> Vec<&Box<dyn DBusInterface>> {
        self.interfaces.values().collect()
    }
    
    /// Handle method call
    pub fn handle_method_call(
        &self,
        interface_name: &str,
        method_name: &str,
        args: &[MessageItem],
        auth_token: Option<&AuthToken>,
    ) -> Result<Vec<MessageItem>> {
        // Get interface
        let interface = self.interfaces.get(interface_name).ok_or_else(|| {
            IPCError::InternalError(format!("Interface {} not found", interface_name))
        })?;
        
        // Check if method requires authentication
        let methods = interface.methods();
        let method = methods.iter().find(|m| m.name == method_name).ok_or_else(|| {
            IPCError::InternalError(format!("Method {} not found in interface {}", method_name, interface_name))
        })?;
        
        if method.requires_auth && auth_token.is_none() {
            return Err(IPCError::AuthenticationError(
                "Authentication required".to_string(),
            ));
        }
        
        // Handle method call
        interface.handle_method_call(method_name, args, auth_token)
    }
    
    /// Get property value
    pub fn get_property(
        &self,
        interface_name: &str,
        property_name: &str,
        auth_token: Option<&AuthToken>,
    ) -> Result<MessageItem> {
        // Get interface
        let interface = self.interfaces.get(interface_name).ok_or_else(|| {
            IPCError::InternalError(format!("Interface {} not found", interface_name))
        })?;
        
        // Check if property requires authentication for read
        let properties = interface.properties();
        let property = properties.iter().find(|p| p.name == property_name).ok_or_else(|| {
            IPCError::InternalError(format!("Property {} not found in interface {}", property_name, interface_name))
        })?;
        
        if property.requires_auth_read && auth_token.is_none() {
            return Err(IPCError::AuthenticationError(
                "Authentication required for reading property".to_string(),
            ));
        }
        
        // Check if property is readable
        if property.access == DBusPropertyAccess::Write {
            return Err(IPCError::AuthorizationError(
                "Property is write-only".to_string(),
            ));
        }
        
        // Get property value
        interface.get_property(property_name, auth_token)
    }
    
    /// Set property value
    pub fn set_property(
        &self,
        interface_name: &str,
        property_name: &str,
        value: MessageItem,
        auth_token: Option<&AuthToken>,
    ) -> Result<()> {
        // Get interface
        let interface = self.interfaces.get(interface_name).ok_or_else(|| {
            IPCError::InternalError(format!("Interface {} not found", interface_name))
        })?;
        
        // Check if property requires authentication for write
        let properties = interface.properties();
        let property = properties.iter().find(|p| p.name == property_name).ok_or_else(|| {
            IPCError::InternalError(format!("Property {} not found in interface {}", property_name, interface_name))
        })?;
        
        if property.requires_auth_write && auth_token.is_none() {
            return Err(IPCError::AuthenticationError(
                "Authentication required for writing property".to_string(),
            ));
        }
        
        // Check if property is writable
        if property.access == DBusPropertyAccess::Read {
            return Err(IPCError::AuthorizationError(
                "Property is read-only".to_string(),
            ));
        }
        
        // Set property value
        interface.set_property(property_name, value, auth_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    // Mock DBusInterface implementation for testing
    struct MockInterface {
        name: String,
        methods: Vec<DBusMethod>,
        signals: Vec<DBusSignal>,
        properties: Vec<DBusProperty>,
    }
    
    impl DBusInterface for MockInterface {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn methods(&self) -> Vec<DBusMethod> {
            self.methods.clone()
        }
        
        fn signals(&self) -> Vec<DBusSignal> {
            self.signals.clone()
        }
        
        fn properties(&self) -> Vec<DBusProperty> {
            self.properties.clone()
        }
        
        fn handle_method_call(&self, method_name: &str, args: &[MessageItem], auth_token: Option<&AuthToken>) -> Result<Vec<MessageItem>> {
            Ok(vec![MessageItem::Str("result".to_string())])
        }
        
        fn get_property(&self, property_name: &str, auth_token: Option<&AuthToken>) -> Result<MessageItem> {
            Ok(MessageItem::Str("value".to_string()))
        }
        
        fn set_property(&self, property_name: &str, value: MessageItem, auth_token: Option<&AuthToken>) -> Result<()> {
            Ok(())
        }
        
        fn clone_box(&self) -> Box<dyn DBusInterface> {
            Box::new(MockInterface {
                name: self.name.clone(),
                methods: self.methods.clone(),
                signals: self.signals.clone(),
                properties: self.properties.clone(),
            })
        }
    }
    
    #[test]
    fn test_object_creation() {
        let object = DBusObject::new("/org/vr/CoreAPI");
        assert_eq!(object.path(), "/org/vr/CoreAPI");
        assert_eq!(object.get_interfaces().len(), 0);
    }
    
    #[test]
    fn test_add_interface() {
        let mut object = DBusObject::new("/org/vr/CoreAPI");
        
        let interface = MockInterface {
            name: "org.vr.CoreAPI.Test".to_string(),
            methods: vec![],
            signals: vec![],
            properties: vec![],
        };
        
        object.add_interface(Box::new(interface)).unwrap();
        assert_eq!(object.get_interfaces().len(), 1);
        
        let interface = object.get_interface("org.vr.CoreAPI.Test").unwrap();
        assert_eq!(interface.name(), "org.vr.CoreAPI.Test");
    }
    
    #[test]
    fn test_remove_interface() {
        let mut object = DBusObject::new("/org/vr/CoreAPI");
        
        let interface = MockInterface {
            name: "org.vr.CoreAPI.Test".to_string(),
            methods: vec![],
            signals: vec![],
            properties: vec![],
        };
        
        object.add_interface(Box::new(interface)).unwrap();
        assert_eq!(object.get_interfaces().len(), 1);
        
        object.remove_interface("org.vr.CoreAPI.Test").unwrap();
        assert_eq!(object.get_interfaces().len(), 0);
    }
    
    #[test]
    fn test_handle_method_call() {
        let mut object = DBusObject::new("/org/vr/CoreAPI");
        
        let method = DBusMethod::new("TestMethod", "s", "s")
            .description("Test method")
            .requires_auth(false);
        
        let interface = MockInterface {
            name: "org.vr.CoreAPI.Test".to_string(),
            methods: vec![method],
            signals: vec![],
            properties: vec![],
        };
        
        object.add_interface(Box::new(interface)).unwrap();
        
        let result = object.handle_method_call(
            "org.vr.CoreAPI.Test",
            "TestMethod",
            &[MessageItem::Str("arg".to_string())],
            None,
        ).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], MessageItem::Str("result".to_string()));
    }
    
    #[test]
    fn test_get_property() {
        let mut object = DBusObject::new("/org/vr/CoreAPI");
        
        let property = DBusProperty::new("TestProperty", "s", DBusPropertyAccess::Read)
            .description("Test property")
            .requires_auth_read(false);
        
        let interface = MockInterface {
            name: "org.vr.CoreAPI.Test".to_string(),
            methods: vec![],
            signals: vec![],
            properties: vec![property],
        };
        
        object.add_interface(Box::new(interface)).unwrap();
        
        let result = object.get_property(
            "org.vr.CoreAPI.Test",
            "TestProperty",
            None,
        ).unwrap();
        
        assert_eq!(result, MessageItem::Str("value".to_string()));
    }
    
    #[test]
    fn test_set_property() {
        let mut object = DBusObject::new("/org/vr/CoreAPI");
        
        let property = DBusProperty::new("TestProperty", "s", DBusPropertyAccess::ReadWrite)
            .description("Test property")
            .requires_auth_write(false);
        
        let interface = MockInterface {
            name: "org.vr.CoreAPI.Test".to_string(),
            methods: vec![],
            signals: vec![],
            properties: vec![property],
        };
        
        object.add_interface(Box::new(interface)).unwrap();
        
        let result = object.set_property(
            "org.vr.CoreAPI.Test",
            "TestProperty",
            MessageItem::Str("new_value".to_string()),
            None,
        );
        
        assert!(result.is_ok());
    }
}
