//! Common IPC utilities for the VR Core API.
//!
//! This module provides common utilities for IPC mechanisms,
//! including message definitions, serialization, authentication,
//! and error handling.

pub mod message;
pub mod serialization;
pub mod error;

pub use message::{IPCMessage, MessageType, MessagePayload, MessageFlags, MessagePriority, MessageHandler};
pub use serialization::{serialize_message, deserialize_message};
pub use error::{IPCError, Result};
