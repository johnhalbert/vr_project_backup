//! WebSocket implementation for IPC mechanisms.
//!
//! This module provides WebSocket implementation for IPC,
//! including server, client, connection, and protocol definitions.

mod server;
mod client;
mod connection;
mod protocol;

pub use server::WebSocketServer;
pub use client::WebSocketClient;
pub use connection::WebSocketConnection;
pub use protocol::{WebSocketProtocol, WebSocketMessage};

use std::path::Path;
use std::net::SocketAddr;

use crate::security::authentication::AuthenticationProvider;
use super::common::{IPCError, Result};
