//! Storage backends for the audit logging system.
//!
//! This module provides storage backends for the audit logging system,
//! including file-based and memory-based storage.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use chrono::{Local};
use log::error;
use serde_json;

use super::event::AuditEvent;
use super::query::AuditQuery;

/// Audit storage trait.
pub trait AuditStorage: Send + Sync {
    /// Store an audit event.
    fn store_event(&mut self, event: &AuditEvent) -> Result<()>;
    
    /// Query audit events.
    fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>>;
    
    /// Flush the storage.
    fn flush(&mut self) -> Result<()>;
}

/// File-based audit storage.
pub struct FileStorage {
    /// Storage directory
    storage_dir: PathBuf,
    
    /// Current log file
    current_file: PathBuf,
    
    /// Current writer
    writer: BufWriter<File>,
    
    /// Event count
    event_count: usize,
    
    /// Max events per file
    max_events_per_file: usize,
}

impl FileStorage {
    /// Create a new file-based storage.
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        // Create the storage directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        
        // Create the current log file
        let current_file = Self::create_log_file(&storage_dir)?;
        
        // Open the file for writing
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_file)?;
        
        let writer = BufWriter::new(file);
        
        Ok(Self {
            storage_dir,
            current_file,
            writer,
            event_count: 0,
            max_events_per_file: 10000,
        })
    }
    
    /// Create a new log file.
    fn create_log_file(storage_dir: &Path) -> Result<PathBuf> {
        let now = Local::now();
        let filename = format!("audit_{}.log", now.format("%Y%m%d_%H%M%S"));
        let file_path = storage_dir.join(filename);
        
        Ok(file_path)
    }
    
    /// Rotate the log file if needed.
    fn rotate_if_needed(&mut self) -> Result<()> {
        if self.event_count >= self.max_events_per_file {
            // Flush the current writer
            self.writer.flush()?;
            
            // Create a new log file
            let new_file = Self::create_log_file(&self.storage_dir)?;
            
            // Open the file for writing
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_file)?;
            
            // Update the writer
            self.writer = BufWriter::new(file);
            self.current_file = new_file;
            self.event_count = 0;
        }
        
        Ok(())
    }
}

impl AuditStorage for FileStorage {
    fn store_event(&mut self, event: &AuditEvent) -> Result<()> {
        // Rotate the log file if needed
        self.rotate_if_needed()?;
        
        // Serialize the event
        let event_json = serde_json::to_string(event)?;
        
        // Write the event to the file
        writeln!(self.writer, "{}", event_json)?;
        
        // Increment the event count
        self.event_count += 1;
        
        Ok(())
    }
    
    fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        let mut events = Vec::new();
        
        // Get all log files
        let mut log_files = Vec::new();
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
                log_files.push(path);
            }
        }
        
        // Sort log files by name (which includes timestamp)
        log_files.sort();
        
        // Process each log file
        for log_file in log_files {
            // Open the file for reading
            let file = File::open(&log_file)?;
            let reader = BufReader::new(file);
            
            // Read each line
            for line in io::BufRead::lines(reader) {
                let line = line?;
                
                // Parse the event
                let event: AuditEvent = serde_json::from_str(&line)?;
                
                // Apply the query
                if query.matches(&event) {
                    events.push(event);
                }
            }
        }
        
        Ok(events)
    }
    
    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// Memory-based audit storage.
pub struct MemoryStorage {
    /// Events
    events: Vec<AuditEvent>,
    
    /// Max events
    max_events: usize,
}

impl MemoryStorage {
    /// Create a new memory-based storage.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }
}

impl AuditStorage for MemoryStorage {
    fn store_event(&mut self, event: &AuditEvent) -> Result<()> {
        // Add the event
        self.events.push(event.clone());
        
        // Trim if needed
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
        
        Ok(())
    }
    
    fn query_events(&self, query: &AuditQuery) -> Result<Vec<AuditEvent>> {
        let mut events = Vec::new();
        
        for event in &self.events {
            if query.matches(event) {
                events.push(event.clone());
            }
        }
        
        Ok(events)
    }
    
    fn flush(&mut self) -> Result<()> {
        // Nothing to flush
        Ok(())
    }
}
