//! Query functionality for the audit logging system.
//!
//! This module provides query functionality for the audit logging system,
//! allowing filtering and retrieval of audit events.

use std::time::SystemTime;

use super::event::{AuditEvent, EventCategory, EventSeverity};

/// Audit query.
pub struct AuditQuery {
    /// Start time
    pub start_time: Option<SystemTime>,
    
    /// End time
    pub end_time: Option<SystemTime>,
    
    /// Categories
    pub categories: Option<Vec<EventCategory>>,
    
    /// Severities
    pub severities: Option<Vec<EventSeverity>>,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// Session ID
    pub session_id: Option<String>,
    
    /// Source IP
    pub source_ip: Option<String>,
    
    /// Message contains
    pub message_contains: Option<String>,
}

impl AuditQuery {
    /// Create a new audit query.
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            categories: None,
            severities: None,
            user_id: None,
            session_id: None,
            source_ip: None,
            message_contains: None,
        }
    }
    
    /// Set the start time.
    pub fn with_start_time(mut self, start_time: SystemTime) -> Self {
        self.start_time = Some(start_time);
        self
    }
    
    /// Set the end time.
    pub fn with_end_time(mut self, end_time: SystemTime) -> Self {
        self.end_time = Some(end_time);
        self
    }
    
    /// Set the categories.
    pub fn with_categories(mut self, categories: Vec<EventCategory>) -> Self {
        self.categories = Some(categories);
        self
    }
    
    /// Set the severities.
    pub fn with_severities(mut self, severities: Vec<EventSeverity>) -> Self {
        self.severities = Some(severities);
        self
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
    
    /// Set the message contains.
    pub fn with_message_contains(mut self, message_contains: &str) -> Self {
        self.message_contains = Some(message_contains.to_string());
        self
    }
    
    /// Check if an event matches the query.
    pub fn matches(&self, event: &AuditEvent) -> bool {
        // Check start time
        if let Some(start_time) = self.start_time {
            if event.timestamp < start_time {
                return false;
            }
        }
        
        // Check end time
        if let Some(end_time) = self.end_time {
            if event.timestamp > end_time {
                return false;
            }
        }
        
        // Check categories
        if let Some(categories) = &self.categories {
            if !categories.contains(&event.category) {
                return false;
            }
        }
        
        // Check severities
        if let Some(severities) = &self.severities {
            if !severities.contains(&event.severity) {
                return false;
            }
        }
        
        // Check user ID
        if let Some(user_id) = &self.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }
        
        // Check session ID
        if let Some(session_id) = &self.session_id {
            if event.session_id.as_ref() != Some(session_id) {
                return false;
            }
        }
        
        // Check source IP
        if let Some(source_ip) = &self.source_ip {
            if event.source_ip.as_ref() != Some(source_ip) {
                return false;
            }
        }
        
        // Check message contains
        if let Some(message_contains) = &self.message_contains {
            if !event.message.contains(message_contains) {
                return false;
            }
        }
        
        true
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self::new()
    }
}
