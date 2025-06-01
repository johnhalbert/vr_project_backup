//! D-Bus interface definitions for IPC mechanisms.
//!
//! This module provides interface definitions for D-Bus IPC,
//! including methods, signals, and properties.

use std::fmt;
use std::sync::Arc;

use dbus::arg::{Arg, ArgType};
use dbus::message::MessageItem;
use log::{debug, error, warn};

use crate::security::authentication::AuthToken;
use crate::ipc::common::{IPCError, Result};

/// D-Bus interface trait
pub trait DBusInterface: Send + Sync {
    /// Get interface name
    fn name(&self) -> &str;
    
    /// Get interface methods
    fn methods(&self) -> Vec<DBusMethod>;
    
    /// Get interface signals
    fn signals(&self) -> Vec<DBusSignal>;
    
    /// Get interface properties
    fn properties(&self) -> Vec<DBusProperty>;
    
    /// Handle method call
    fn handle_method_call(&self, method_name: &str, args: &[MessageItem], auth_token: Option<&AuthToken>) -> Result<Vec<MessageItem>>;
    
    /// Get property value
    fn get_property(&self, property_name: &str, auth_token: Option<&AuthToken>) -> Result<MessageItem>;
    
    /// Set property value
    fn set_property(&self, property_name: &str, value: MessageItem, auth_token: Option<&AuthToken>) -> Result<()>;
    
    /// Clone interface
    fn clone_box(&self) -> Box<dyn DBusInterface>;
}

impl Clone for Box<dyn DBusInterface> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// D-Bus method
#[derive(Debug, Clone)]
pub struct DBusMethod {
    /// Method name
    pub name: String,
    
    /// Method input signature
    pub input_signature: String,
    
    /// Method output signature
    pub output_signature: String,
    
    /// Method description
    pub description: String,
    
    /// Required authentication
    pub requires_auth: bool,
}

impl DBusMethod {
    /// Create a new DBusMethod
    pub fn new(name: &str, input_signature: &str, output_signature: &str) -> Self {
        Self {
            name: name.to_string(),
            input_signature: input_signature.to_string(),
            output_signature: output_signature.to_string(),
            description: String::new(),
            requires_auth: false,
        }
    }
    
    /// Set method description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    /// Set required authentication
    pub fn requires_auth(mut self, requires_auth: bool) -> Self {
        self.requires_auth = requires_auth;
        self
    }
}

/// D-Bus signal
#[derive(Debug, Clone)]
pub struct DBusSignal {
    /// Signal name
    pub name: String,
    
    /// Signal signature
    pub signature: String,
    
    /// Signal description
    pub description: String,
}

impl DBusSignal {
    /// Create a new DBusSignal
    pub fn new(name: &str, signature: &str) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.to_string(),
            description: String::new(),
        }
    }
    
    /// Set signal description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
}

/// D-Bus property
#[derive(Debug, Clone)]
pub struct DBusProperty {
    /// Property name
    pub name: String,
    
    /// Property signature
    pub signature: String,
    
    /// Property description
    pub description: String,
    
    /// Property access
    pub access: DBusPropertyAccess,
    
    /// Required authentication for read
    pub requires_auth_read: bool,
    
    /// Required authentication for write
    pub requires_auth_write: bool,
}

impl DBusProperty {
    /// Create a new DBusProperty
    pub fn new(name: &str, signature: &str, access: DBusPropertyAccess) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.to_string(),
            description: String::new(),
            access,
            requires_auth_read: false,
            requires_auth_write: true,
        }
    }
    
    /// Set property description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    /// Set required authentication for read
    pub fn requires_auth_read(mut self, requires_auth: bool) -> Self {
        self.requires_auth_read = requires_auth;
        self
    }
    
    /// Set required authentication for write
    pub fn requires_auth_write(mut self, requires_auth: bool) -> Self {
        self.requires_auth_write = requires_auth;
        self
    }
}

/// D-Bus property access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DBusPropertyAccess {
    /// Read-only property
    Read,
    
    /// Write-only property
    Write,
    
    /// Read-write property
    ReadWrite,
}

impl fmt::Display for DBusPropertyAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBusPropertyAccess::Read => write!(f, "read"),
            DBusPropertyAccess::Write => write!(f, "write"),
            DBusPropertyAccess::ReadWrite => write!(f, "readwrite"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dbus_method_creation() {
        let method = DBusMethod::new("TestMethod", "s", "i")
            .description("Test method")
            .requires_auth(true);
        
        assert_eq!(method.name, "TestMethod");
        assert_eq!(method.input_signature, "s");
        assert_eq!(method.output_signature, "i");
        assert_eq!(method.description, "Test method");
        assert_eq!(method.requires_auth, true);
    }
    
    #[test]
    fn test_dbus_signal_creation() {
        let signal = DBusSignal::new("TestSignal", "s")
            .description("Test signal");
        
        assert_eq!(signal.name, "TestSignal");
        assert_eq!(signal.signature, "s");
        assert_eq!(signal.description, "Test signal");
    }
    
    #[test]
    fn test_dbus_property_creation() {
        let property = DBusProperty::new("TestProperty", "s", DBusPropertyAccess::ReadWrite)
            .description("Test property")
            .requires_auth_read(true)
            .requires_auth_write(true);
        
        assert_eq!(property.name, "TestProperty");
        assert_eq!(property.signature, "s");
        assert_eq!(property.description, "Test property");
        assert_eq!(property.access, DBusPropertyAccess::ReadWrite);
        assert_eq!(property.requires_auth_read, true);
        assert_eq!(property.requires_auth_write, true);
    }
    
    #[test]
    fn test_dbus_property_access_display() {
        assert_eq!(format!("{}", DBusPropertyAccess::Read), "read");
        assert_eq!(format!("{}", DBusPropertyAccess::Write), "write");
        assert_eq!(format!("{}", DBusPropertyAccess::ReadWrite), "readwrite");
    }
}
