//! OpenVR driver for the VR headset
//! 
//! This crate implements the Rust side of the OpenVR driver,
//! providing integration between the Core API and SteamVR.

mod device;
mod driver;
mod error;
mod ffi;
mod tracking;
mod input;
mod settings;
mod types;
mod utils;

use std::sync::Arc;
use parking_lot::Mutex;
use thread_local::ThreadLocal;

// Re-exports
pub use device::{VRDevice, DeviceClass, DeviceType};
pub use driver::{DriverCore, DriverSettings};
pub use error::{Result, Error};
pub use tracking::{Pose, TrackingProvider};
pub use input::{InputHandler, Button, ButtonState, Axis};
pub use settings::SettingsManager;
pub use types::*;

// Global driver instance
thread_local! {
    static DRIVER: ThreadLocal<Arc<Mutex<DriverCore>>> = ThreadLocal::new();
}

// FFI exports
pub use ffi::exports::*;
