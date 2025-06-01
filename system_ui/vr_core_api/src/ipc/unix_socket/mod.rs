//! Unix domain socket implementation for IPC mechanisms.
//!
//! This module provides Unix domain socket implementation for IPC,
//! including server, client, and connection handling.

mod server;
mod client;
mod connection;

pub use server::UnixSocketServer;
pub use client::UnixSocketClient;
pub use connection::UnixSocketConnection;

use std::path::Path;

use crate::security::authentication::AuthenticationProvider;
use super::common::{IPCError, Result};
