//! Device event management system for the Hardware Access API.
//!
//! This module provides a centralized event management system for hardware devices,
//! allowing for registration of event handlers and dispatching of device events.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use log::{debug, error, info, warn};
use uuid::Uuid;

use super::device::{DeviceEvent, DeviceEventHandler, DeviceEventType, DeviceResult, DeviceError};

/// Event subscription information.
pub struct EventSubscription {
    /// Subscription ID
    pub id: String,
    
    /// Device ID filter (None means all devices)
    pub device_id_filter: Option<String>,
    
    /// Event type filter (None means all event types)
    pub event_type_filter: Option<DeviceEventType>,
    
    /// Event handler
    pub handler: DeviceEventHandler,
}

impl EventSubscription {
    /// Create a new EventSubscription.
    pub fn new(
        device_id_filter: Option<String>,
        event_type_filter: Option<DeviceEventType>,
        handler: DeviceEventHandler,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            device_id_filter,
            event_type_filter,
            handler,
        }
    }
    
    /// Check if this subscription matches the given event.
    pub fn matches(&self, event: &DeviceEvent) -> bool {
        // Check device ID filter
        if let Some(device_id) = &self.device_id_filter {
            if *device_id != event.device_id {
                return false;
            }
        }
        
        // Check event type filter
        if let Some(event_type) = &self.event_type_filter {
            // This is a simplified match that only checks the variant, not the contents
            // For a more detailed match, we would need to implement a more complex matching logic
            std::mem::discriminant(event_type) != std::mem::discriminant(&event.event_type)
        } else {
            true
        }
    }
}

/// Device event manager.
pub struct DeviceEventManager {
    /// Event subscriptions
    subscriptions: RwLock<HashMap<String, EventSubscription>>,
    
    /// Event history
    event_history: Arc<Mutex<Vec<DeviceEvent>>>,
    
    /// Maximum event history size
    max_history_size: usize,
}

impl DeviceEventManager {
    /// Create a new DeviceEventManager.
    pub fn new(max_history_size: usize) -> Self {
        Self {
            subscriptions: RwLock::new(HashMap::new()),
            event_history: Arc::new(Mutex::new(Vec::with_capacity(max_history_size))),
            max_history_size,
        }
    }
    
    /// Register an event handler.
    pub fn register_handler(
        &self,
        device_id_filter: Option<String>,
        event_type_filter: Option<DeviceEventType>,
        handler: DeviceEventHandler,
    ) -> DeviceResult<String> {
        let subscription = EventSubscription::new(device_id_filter, event_type_filter, handler);
        let subscription_id = subscription.id.clone();
        
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        subscriptions.insert(subscription_id.clone(), subscription);
        
        Ok(subscription_id)
    }
    
    /// Unregister an event handler.
    pub fn unregister_handler(&self, subscription_id: &str) -> DeviceResult<()> {
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        if subscriptions.remove(subscription_id).is_none() {
            return Err(DeviceError::NotFound(format!("Subscription {} not found", subscription_id)));
        }
        
        Ok(())
    }
    
    /// Unregister all event handlers.
    pub fn unregister_all_handlers(&self) -> DeviceResult<()> {
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        subscriptions.clear();
        
        Ok(())
    }
    
    /// Dispatch an event to all matching handlers.
    pub fn dispatch_event(&self, event: DeviceEvent) -> DeviceResult<()> {
        // Add event to history
        {
            let mut history = self.event_history.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on event history".to_string())
            })?;
            
            history.push(event.clone());
            
            // Trim history if needed
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }
        
        // Dispatch event to matching handlers
        let subscriptions = self.subscriptions.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on subscriptions".to_string())
        })?;
        
        for subscription in subscriptions.values() {
            if subscription.matches(&event) {
                // Call the handler
                (subscription.handler)(&event);
            }
        }
        
        Ok(())
    }
    
    /// Get the event history.
    pub fn get_event_history(&self) -> DeviceResult<Vec<DeviceEvent>> {
        let history = self.event_history.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on event history".to_string())
        })?;
        
        Ok(history.clone())
    }
    
    /// Clear the event history.
    pub fn clear_event_history(&self) -> DeviceResult<()> {
        let mut history = self.event_history.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on event history".to_string())
        })?;
        
        history.clear();
        
        Ok(())
    }
    
    /// Get the number of registered handlers.
    pub fn get_handler_count(&self) -> DeviceResult<usize> {
        let subscriptions = self.subscriptions.read().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire read lock on subscriptions".to_string())
        })?;
        
        Ok(subscriptions.len())
    }
    
    /// Get the event history size.
    pub fn get_history_size(&self) -> DeviceResult<usize> {
        let history = self.event_history.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on event history".to_string())
        })?;
        
        Ok(history.len())
    }
    
    /// Get the maximum event history size.
    pub fn get_max_history_size(&self) -> usize {
        self.max_history_size
    }
    
    /// Set the maximum event history size.
    pub fn set_max_history_size(&mut self, max_size: usize) {
        self.max_history_size = max_size;
        
        // Trim history if needed
        if let Ok(mut history) = self.event_history.lock() {
            while history.len() > max_size {
                history.remove(0);
            }
        }
    }
}

impl Default for DeviceEventManager {
    fn default() -> Self {
        Self::new(1000) // Default to storing the last 1000 events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::device::{DeviceEvent, DeviceEventType};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_event_subscription_matching() {
        // Create a test event
        let event = DeviceEvent {
            id: "event1".to_string(),
            device_id: "device1".to_string(),
            event_type: DeviceEventType::StateChanged {
                previous: crate::hardware::device::DeviceState::Connected,
                current: crate::hardware::device::DeviceState::Ready,
            },
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        // Test with no filters
        let subscription = EventSubscription::new(
            None,
            None,
            Box::new(|_| {}),
        );
        assert!(subscription.matches(&event));
        
        // Test with matching device ID filter
        let subscription = EventSubscription::new(
            Some("device1".to_string()),
            None,
            Box::new(|_| {}),
        );
        assert!(subscription.matches(&event));
        
        // Test with non-matching device ID filter
        let subscription = EventSubscription::new(
            Some("device2".to_string()),
            None,
            Box::new(|_| {}),
        );
        assert!(!subscription.matches(&event));
        
        // Test with matching event type filter
        let subscription = EventSubscription::new(
            None,
            Some(DeviceEventType::StateChanged {
                previous: crate::hardware::device::DeviceState::Disconnected,
                current: crate::hardware::device::DeviceState::Connected,
            }),
            Box::new(|_| {}),
        );
        assert!(subscription.matches(&event));
        
        // Test with non-matching event type filter
        let subscription = EventSubscription::new(
            None,
            Some(DeviceEventType::Connected),
            Box::new(|_| {}),
        );
        assert!(!subscription.matches(&event));
        
        // Test with both filters matching
        let subscription = EventSubscription::new(
            Some("device1".to_string()),
            Some(DeviceEventType::StateChanged {
                previous: crate::hardware::device::DeviceState::Disconnected,
                current: crate::hardware::device::DeviceState::Connected,
            }),
            Box::new(|_| {}),
        );
        assert!(subscription.matches(&event));
        
        // Test with one filter matching and one not
        let subscription = EventSubscription::new(
            Some("device1".to_string()),
            Some(DeviceEventType::Connected),
            Box::new(|_| {}),
        );
        assert!(!subscription.matches(&event));
    }
    
    #[test]
    fn test_event_manager() {
        let manager = DeviceEventManager::new(10);
        
        // Create a counter to track handler calls
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Register a handler
        let handler: DeviceEventHandler = Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let subscription_id = manager.register_handler(None, None, handler).unwrap();
        
        // Create and dispatch an event
        let event = DeviceEvent {
            id: "event1".to_string(),
            device_id: "device1".to_string(),
            event_type: DeviceEventType::Connected,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        manager.dispatch_event(event.clone()).unwrap();
        
        // Check that the handler was called
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        
        // Check that the event was added to history
        let history = manager.get_event_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, event.id);
        
        // Unregister the handler
        manager.unregister_handler(&subscription_id).unwrap();
        
        // Dispatch another event
        let event2 = DeviceEvent {
            id: "event2".to_string(),
            device_id: "device1".to_string(),
            event_type: DeviceEventType::Connected,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        manager.dispatch_event(event2.clone()).unwrap();
        
        // Check that the handler was not called again
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        
        // Check that the event was added to history
        let history = manager.get_event_history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].id, event2.id);
        
        // Clear the history
        manager.clear_event_history().unwrap();
        
        // Check that the history is empty
        let history = manager.get_event_history().unwrap();
        assert_eq!(history.len(), 0);
    }
    
    #[test]
    fn test_history_size_limit() {
        let manager = DeviceEventManager::new(2);
        
        // Create and dispatch 3 events
        for i in 0..3 {
            let event = DeviceEvent {
                id: format!("event{}", i),
                device_id: "device1".to_string(),
                event_type: DeviceEventType::Connected,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            
            manager.dispatch_event(event).unwrap();
        }
        
        // Check that only the last 2 events are in history
        let history = manager.get_event_history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, "event1");
        assert_eq!(history[1].id, "event2");
    }
}
