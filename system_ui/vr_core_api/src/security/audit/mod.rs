//! Audit logging system for the VR headset.
//!
//! This module provides comprehensive audit logging functionality for the VR headset,
//! including event tracking, storage, and querying.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::error;
use thiserror::Error;

pub mod event;
pub mod storage;
pub mod query;

use event::{AuditEvent, EventCategory, EventSeverity};
use storage::{AuditStorage, FileStorage};
use query::AuditQuery;

/// Audit logger for the VR headset.
pub struct AuditLogger {
    /// Storage backend
    storage: Arc<Mutex<Box<dyn AuditStorage>>>,
    
    /// Audit directory
    audit_dir: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(audit_dir: PathBuf) -> Result<Self> {
        // Create the storage backend
        let storage: Box<dyn AuditStorage> = Box::new(FileStorage::new(audit_dir.clone())?);
        let storage = Arc::new(Mutex::new(storage));
        
        Ok(Self {
            storage,
            audit_dir,
        })
    }
    
    /// Initialize the audit logger.
    pub fn initialize(&self) -> Result<()> {
        // Log initialization event
        self.log_event(
            EventCategory::System,
            EventSeverity::Info,
            "Audit logger initialized",
            None,
        )?;
        
        Ok(())
    }
    
    /// Shutdown the audit logger.
    pub fn shutdown(&self) -> Result<()> {
        // Log shutdown event
        self.log_event(
            EventCategory::System,
            EventSeverity::Info,
            "Audit logger shutting down",
            None,
        )?;
        
        // Flush the storage
        self.storage.lock().unwrap().flush()?;
        
        Ok(())
    }
    
    /// Log an audit event.
    pub fn log_event(
        &self,
        category: EventCategory,
        severity: EventSeverity,
        message: &str,
        details: Option<serde_json::Value>,
    ) -> Result<AuditEvent> {
        // Create the event
        let event = AuditEvent::new(category, severity, message, details);
        
        // Store the event
        self.storage.lock().unwrap().store_event(&event)?;
        
        Ok(event)
    }
    
    /// Query audit events.
    pub fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        self.storage.lock().unwrap().query_events(query)
    }
    
    /// Get the audit directory.
    pub fn audit_dir(&self) -> PathBuf {
        self.audit_dir.clone()
    }
}

/// Audit error.
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
