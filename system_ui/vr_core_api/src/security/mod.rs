//! Security and authentication module for the VR Core API.
//!
//! This module provides security and authentication functionality for the VR Core API,
//! including authentication providers, authorization, encryption, and secure storage.

pub mod authentication;
pub mod authorization;
pub mod encryption;
pub mod secure_storage;

pub use authentication::{AuthenticationProvider, AuthToken, Credentials};
pub use authorization::{AuthorizationProvider, Permission, Role};
pub use encryption::{EncryptionProvider, EncryptionKey};
pub use secure_storage::SecureStorage;
