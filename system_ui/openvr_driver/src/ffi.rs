//! FFI interface for the OpenVR driver

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Arc, Mutex};
use crate::driver::DriverCore;
use crate::error::{Result, Error};
use crate::DRIVER;

/// FFI exports for the OpenVR driver
pub mod exports {
    use super::*;
    
    /// Entry point for OpenVR to get the server tracked device provider
    #[no_mangle]
    pub extern "C" fn vr_driver_get_server_provider() -> *mut c_void {
        // Create a new server provider wrapper
        let provider = Box::new(ServerProviderWrapper::new());
        
        // Convert to raw pointer
        Box::into_raw(provider) as *mut c_void
    }
    
    /// Initialize the driver with the given context
    #[no_mangle]
    pub extern "C" fn vr_driver_init(
        context: *mut c_void,
        driver_log: *mut c_void,
        driver_host: *mut c_void,
        driver_input: *mut c_void,
        driver_properties: *mut c_void,
        driver_settings: *mut c_void,
    ) -> c_int {
        // Create a new driver core
        let mut driver = DriverCore::new(
            context,
            driver_log,
            driver_host,
            driver_input,
            driver_properties,
            driver_settings,
        );
        
        // Initialize the driver
        match driver.initialize() {
            Ok(_) => {
                // Store the driver in thread local storage
                let driver_arc = Arc::new(Mutex::new(driver));
                DRIVER.with(|d| {
                    d.set(driver_arc);
                });
                
                0 // Success
            },
            Err(e) => {
                // Log error
                let _ = driver.log(&format!("Failed to initialize driver: {:?}", e));
                
                // Return error code
                match e {
                    Error::OpenVRInitFailed(code) => code,
                    _ => 1, // Generic error
                }
            }
        }
    }
    
    /// Run a single frame update
    #[no_mangle]
    pub extern "C" fn vr_driver_run_frame() -> c_int {
        DRIVER.with(|driver| {
            if let Some(driver_arc) = driver.get() {
                if let Ok(mut driver) = driver_arc.lock() {
                    match driver.run_frame() {
                        Ok(_) => 0, // Success
                        Err(e) => {
                            // Log error
                            let _ = driver.log(&format!("Failed to run frame: {:?}", e));
                            
                            1 // Error
                        }
                    }
                } else {
                    1 // Failed to lock driver
                }
            } else {
                1 // Driver not initialized
            }
        })
    }
    
    /// Clean up the driver
    #[no_mangle]
    pub extern "C" fn vr_driver_cleanup() {
        DRIVER.with(|driver| {
            if let Some(driver_arc) = driver.get() {
                if let Ok(driver) = driver_arc.lock() {
                    let _ = driver.log("Cleaning up driver");
                }
            }
            
            // Clear the driver
            driver.set(None);
        });
    }
    
    /// Enter standby mode
    #[no_mangle]
    pub extern "C" fn vr_driver_enter_standby() -> c_int {
        DRIVER.with(|driver| {
            if let Some(driver_arc) = driver.get() {
                if let Ok(driver) = driver_arc.lock() {
                    let _ = driver.log("Entering standby mode");
                    0 // Success
                } else {
                    1 // Failed to lock driver
                }
            } else {
                1 // Driver not initialized
            }
        })
    }
    
    /// Leave standby mode
    #[no_mangle]
    pub extern "C" fn vr_driver_leave_standby() -> c_int {
        DRIVER.with(|driver| {
            if let Some(driver_arc) = driver.get() {
                if let Ok(driver) = driver_arc.lock() {
                    let _ = driver.log("Leaving standby mode");
                    0 // Success
                } else {
                    1 // Failed to lock driver
                }
            } else {
                1 // Driver not initialized
            }
        })
    }
    
    /// Get the interface versions
    #[no_mangle]
    pub extern "C" fn vr_driver_get_interface_versions() -> *const *const c_char {
        // In a real implementation, this would return the actual interface versions
        // For now, return a null-terminated array of null-terminated strings
        static mut INTERFACE_VERSIONS: [*const c_char; 2] = [std::ptr::null(), std::ptr::null()];
        static mut INTERFACE_VERSION_STR: *const c_char = std::ptr::null();
        
        unsafe {
            if INTERFACE_VERSION_STR.is_null() {
                // Create the interface version string
                let version = CString::new("IServerTrackedDeviceProvider_004").unwrap();
                INTERFACE_VERSION_STR = version.into_raw();
                INTERFACE_VERSIONS[0] = INTERFACE_VERSION_STR;
            }
            
            INTERFACE_VERSIONS.as_ptr()
        }
    }
}

/// Wrapper for the OpenVR server tracked device provider
pub struct ServerProviderWrapper {
    /// Whether the provider has been initialized
    initialized: bool,
}

impl ServerProviderWrapper {
    /// Create a new server provider wrapper
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
}
