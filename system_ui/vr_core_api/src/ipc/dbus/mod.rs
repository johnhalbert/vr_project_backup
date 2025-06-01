//! D-Bus implementation for IPC mechanisms.
//!
//! This module provides D-Bus implementation for IPC,
//! including service, client, object, and interface definitions.

mod service;
mod client;
mod object;
mod interface;

pub use service::DBusService;
pub use client::DBusClient;
pub use object::DBusObject;
pub use interface::{DBusInterface, DBusMethod, DBusSignal, DBusProperty, DBusPropertyAccess};

use std::path::Path;

use crate::security::authentication::AuthenticationProvider;
use super::common::{IPCError, Result};
