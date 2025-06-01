//! Audit event definitions for the VR headset.
//!
//! This module provides definitions for audit events, including
//! categories, severities, and event data structures.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: String,
    
    /// Event timestamp
    pub timestamp: SystemTime,
    
    /// Event category
    pub category: EventCategory,
    
    /// Event severity
    pub severity: EventSeverity,
    
    /// Event message
    pub message: String,
    
    /// Event details
    pub details: Option<serde_json::Value>,
    
    /// User ID (if applicable)
    pub user_id: Option<String>,
    
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    
    /// Source IP (if applicable)
    pub source_ip: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event.
    pub fn new(
        category: EventCategory,
        severity: EventSeverity,
        message: &str,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            category,
            severity,
            message: message.to_string(),
            details,
            user_id: None,
            session_id: None,
            source_ip: None,
        }
    }
    
    /// Set the user ID.
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// Set the session ID.
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
    
    /// Set the source IP.
    pub fn with_source_ip(mut self, source_ip: &str) -> Self {
        self.source_ip = Some(source_ip.to_string());
        self
    }
}

/// Event category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    /// Security events
    Security,
    
    /// System events
    System,
    
    /// User events
    User,
    
    /// Data events
    Data,
}

/// Event severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Debug
    Debug,
    
    /// Info
    Info,
    
    /// Warning
    Warning,
    
    /// Error
    Error,
    
    /// Critical
    Critical,
}
