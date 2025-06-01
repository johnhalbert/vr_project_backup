//! Settings management for the OpenVR driver

use std::sync::{Arc, Mutex};
use crate::types::DriverSettings;
use crate::error::{Result, Error};

/// Interface for settings management
pub trait SettingsInterface: Send + Sync {
    /// Get driver settings
    fn get_settings(&self) -> Result<DriverSettings>;
    
    /// Save driver settings
    fn save_settings(&self, settings: &DriverSettings) -> Result<()>;
    
    /// Get a specific setting value
    fn get_setting_value(&self, key: &str) -> Result<serde_json::Value>;
    
    /// Set a specific setting value
    fn set_setting_value(&self, key: &str, value: serde_json::Value) -> Result<()>;
}

/// Settings manager for OpenVR driver
pub struct SettingsManager {
    /// OpenVR settings interface pointer (opaque)
    vr_settings: *mut std::ffi::c_void,
    
    /// Settings section name
    section_name: String,
    
    /// Current settings
    settings: Arc<Mutex<DriverSettings>>,
    
    /// Core API configuration interface
    core_config: Option<Arc<Mutex<dyn ConfigInterface>>>,
}

/// Interface to Core API configuration
pub trait ConfigInterface: Send + Sync {
    /// Get an integer value
    fn get_int(&self, key: &str) -> Result<i32>;
    
    /// Get a float value
    fn get_float(&self, key: &str) -> Result<f32>;
    
    /// Get a boolean value
    fn get_bool(&self, key: &str) -> Result<bool>;
    
    /// Get a string value
    fn get_string(&self, key: &str) -> Result<String>;
    
    /// Set an integer value
    fn set_int(&mut self, key: &str, value: i32) -> Result<()>;
    
    /// Set a float value
    fn set_float(&mut self, key: &str, value: f32) -> Result<()>;
    
    /// Set a boolean value
    fn set_bool(&mut self, key: &str, value: bool) -> Result<()>;
    
    /// Set a string value
    fn set_string(&mut self, key: &str, value: &str) -> Result<()>;
    
    /// Save configuration
    fn save(&mut self) -> Result<()>;
}

// FFI functions for settings handling
mod ffi {
    use super::*;
    use std::ffi::{CString, CStr};
    use std::os::raw::{c_char, c_int, c_float, c_void};
    
    extern "C" {
        pub fn openvr_settings_get_int(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            error: *mut c_int
        ) -> c_int;
        
        pub fn openvr_settings_get_float(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            error: *mut c_int
        ) -> c_float;
        
        pub fn openvr_settings_get_bool(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            error: *mut c_int
        ) -> bool;
        
        pub fn openvr_settings_get_string(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            value: *mut c_char,
            value_len: c_int,
            error: *mut c_int
        );
        
        pub fn openvr_settings_set_int(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            value: c_int
        );
        
        pub fn openvr_settings_set_float(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            value: c_float
        );
        
        pub fn openvr_settings_set_bool(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            value: bool
        );
        
        pub fn openvr_settings_set_string(
            vr_settings: *mut c_void,
            section: *const c_char,
            key: *const c_char,
            value: *const c_char
        );
    }
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(vr_settings: *mut std::ffi::c_void, section_name: &str) -> Self {
        Self {
            vr_settings,
            section_name: section_name.to_string(),
            settings: Arc::new(Mutex::new(DriverSettings::default())),
            core_config: None,
        }
    }
    
    /// Set Core API configuration interface
    pub fn set_core_config(&mut self, core_config: Arc<Mutex<dyn ConfigInterface>>) {
        self.core_config = Some(core_config);
    }
    
    /// Load settings from OpenVR
    pub fn load_from_openvr(&self) -> Result<DriverSettings> {
        let mut settings = DriverSettings::default();
        let mut error: i32 = 0;
        
        let section = std::ffi::CString::new(self.section_name.clone())
            .map_err(|_| Error::FFIError("Invalid section name".to_string()))?;
        
        // Load render width
        let key = std::ffi::CString::new("renderWidth")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        settings.render_width = unsafe {
            ffi::openvr_settings_get_int(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                &mut error
            )
        };
        
        // Load render height
        let key = std::ffi::CString::new("renderHeight")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        settings.render_height = unsafe {
            ffi::openvr_settings_get_int(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                &mut error
            )
        };
        
        // Load refresh rate
        let key = std::ffi::CString::new("refreshRate")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        settings.refresh_rate = unsafe {
            ffi::openvr_settings_get_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                &mut error
            )
        };
        
        // Load IPD
        let key = std::ffi::CString::new("ipd")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        settings.ipd = unsafe {
            ffi::openvr_settings_get_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                &mut error
            )
        };
        
        // Load prediction time
        let key = std::ffi::CString::new("predictionTimeMs")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        settings.prediction_time_ms = unsafe {
            ffi::openvr_settings_get_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                &mut error
            )
        };
        
        // Update stored settings
        if let Ok(mut stored_settings) = self.settings.lock() {
            *stored_settings = settings.clone();
        }
        
        Ok(settings)
    }
    
    /// Save settings to OpenVR
    pub fn save_to_openvr(&self, settings: &DriverSettings) -> Result<()> {
        let section = std::ffi::CString::new(self.section_name.clone())
            .map_err(|_| Error::FFIError("Invalid section name".to_string()))?;
        
        // Save render width
        let key = std::ffi::CString::new("renderWidth")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        unsafe {
            ffi::openvr_settings_set_int(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                settings.render_width
            )
        };
        
        // Save render height
        let key = std::ffi::CString::new("renderHeight")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        unsafe {
            ffi::openvr_settings_set_int(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                settings.render_height
            )
        };
        
        // Save refresh rate
        let key = std::ffi::CString::new("refreshRate")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        unsafe {
            ffi::openvr_settings_set_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                settings.refresh_rate
            )
        };
        
        // Save IPD
        let key = std::ffi::CString::new("ipd")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        unsafe {
            ffi::openvr_settings_set_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                settings.ipd
            )
        };
        
        // Save prediction time
        let key = std::ffi::CString::new("predictionTimeMs")
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        unsafe {
            ffi::openvr_settings_set_float(
                self.vr_settings,
                section.as_ptr(),
                key.as_ptr(),
                settings.prediction_time_ms
            )
        };
        
        // Update stored settings
        if let Ok(mut stored_settings) = self.settings.lock() {
            *stored_settings = settings.clone();
        }
        
        Ok(())
    }
}

impl SettingsInterface for SettingsManager {
    fn get_settings(&self) -> Result<DriverSettings> {
        // Try to get from Core API first if available
        if let Some(core_config) = &self.core_config {
            if let Ok(config) = core_config.lock() {
                let mut settings = DriverSettings::default();
                
                // Try to load settings from Core API
                settings.render_width = config.get_int("openvr.render_width").unwrap_or(settings.render_width);
                settings.render_height = config.get_int("openvr.render_height").unwrap_or(settings.render_height);
                settings.refresh_rate = config.get_float("openvr.refresh_rate").unwrap_or(settings.refresh_rate);
                settings.ipd = config.get_float("openvr.ipd").unwrap_or(settings.ipd);
                settings.prediction_time_ms = config.get_float("openvr.prediction_time_ms").unwrap_or(settings.prediction_time_ms);
                
                // Update stored settings
                if let Ok(mut stored_settings) = self.settings.lock() {
                    *stored_settings = settings.clone();
                }
                
                return Ok(settings);
            }
        }
        
        // Fall back to OpenVR settings
        self.load_from_openvr()
    }
    
    fn save_settings(&self, settings: &DriverSettings) -> Result<()> {
        // Try to save to Core API first if available
        if let Some(core_config) = &self.core_config {
            if let Ok(mut config) = core_config.lock() {
                // Save settings to Core API
                config.set_int("openvr.render_width", settings.render_width)?;
                config.set_int("openvr.render_height", settings.render_height)?;
                config.set_float("openvr.refresh_rate", settings.refresh_rate)?;
                config.set_float("openvr.ipd", settings.ipd)?;
                config.set_float("openvr.prediction_time_ms", settings.prediction_time_ms)?;
                
                // Save configuration
                config.save()?;
            }
        }
        
        // Also save to OpenVR settings
        self.save_to_openvr(settings)
    }
    
    fn get_setting_value(&self, key: &str) -> Result<serde_json::Value> {
        // Try to get from Core API first if available
        if let Some(core_config) = &self.core_config {
            if let Ok(config) = core_config.lock() {
                // Try different types
                if let Ok(value) = config.get_int(&format!("openvr.{}", key)) {
                    return Ok(serde_json::Value::Number(serde_json::Number::from(value)));
                }
                
                if let Ok(value) = config.get_float(&format!("openvr.{}", key)) {
                    return Ok(serde_json::Value::Number(serde_json::Number::from_f64(value as f64).unwrap_or(serde_json::Number::from(0))));
                }
                
                if let Ok(value) = config.get_bool(&format!("openvr.{}", key)) {
                    return Ok(serde_json::Value::Bool(value));
                }
                
                if let Ok(value) = config.get_string(&format!("openvr.{}", key)) {
                    return Ok(serde_json::Value::String(value));
                }
            }
        }
        
        // Fall back to OpenVR settings
        let section = std::ffi::CString::new(self.section_name.clone())
            .map_err(|_| Error::FFIError("Invalid section name".to_string()))?;
        
        let c_key = std::ffi::CString::new(key)
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        
        let mut error: i32 = 0;
        
        // Try as int
        let int_value = unsafe {
            ffi::openvr_settings_get_int(
                self.vr_settings,
                section.as_ptr(),
                c_key.as_ptr(),
                &mut error
            )
        };
        
        if error == 0 {
            return Ok(serde_json::Value::Number(serde_json::Number::from(int_value)));
        }
        
        // Try as float
        error = 0;
        let float_value = unsafe {
            ffi::openvr_settings_get_float(
                self.vr_settings,
                section.as_ptr(),
                c_key.as_ptr(),
                &mut error
            )
        };
        
        if error == 0 {
            return Ok(serde_json::Value::Number(serde_json::Number::from_f64(float_value as f64).unwrap_or(serde_json::Number::from(0))));
        }
        
        // Try as bool
        error = 0;
        let bool_value = unsafe {
            ffi::openvr_settings_get_bool(
                self.vr_settings,
                section.as_ptr(),
                c_key.as_ptr(),
                &mut error
            )
        };
        
        if error == 0 {
            return Ok(serde_json::Value::Bool(bool_value));
        }
        
        // Try as string
        error = 0;
        let mut buffer = [0u8; 1024];
        unsafe {
            ffi::openvr_settings_get_string(
                self.vr_settings,
                section.as_ptr(),
                c_key.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as i32,
                &mut error
            )
        };
        
        if error == 0 {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr() as *const i8) };
            let str_value = c_str.to_string_lossy().into_owned();
            return Ok(serde_json::Value::String(str_value));
        }
        
        Err(Error::Unknown(format!("Setting not found: {}", key)))
    }
    
    fn set_setting_value(&self, key: &str, value: serde_json::Value) -> Result<()> {
        // Try to set in Core API first if available
        if let Some(core_config) = &self.core_config {
            if let Ok(mut config) = core_config.lock() {
                let full_key = format!("openvr.{}", key);
                
                // Set based on value type
                match value {
                    serde_json::Value::Number(n) => {
                        if n.is_i64() {
                            config.set_int(&full_key, n.as_i64().unwrap_or(0) as i32)?;
                        } else {
                            config.set_float(&full_key, n.as_f64().unwrap_or(0.0) as f32)?;
                        }
                    },
                    serde_json::Value::Bool(b) => {
                        config.set_bool(&full_key, b)?;
                    },
                    serde_json::Value::String(s) => {
                        config.set_string(&full_key, &s)?;
                    },
                    _ => return Err(Error::Unknown(format!("Unsupported value type for setting: {}", key))),
                }
                
                // Save configuration
                config.save()?;
            }
        }
        
        // Also set in OpenVR settings
        let section = std::ffi::CString::new(self.section_name.clone())
            .map_err(|_| Error::FFIError("Invalid section name".to_string()))?;
        
        let c_key = std::ffi::CString::new(key)
            .map_err(|_| Error::FFIError("Invalid key name".to_string()))?;
        
        // Set based on value type
        match value {
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    unsafe {
                        ffi::openvr_settings_set_int(
                            self.vr_settings,
                            section.as_ptr(),
                            c_key.as_ptr(),
                            n.as_i64().unwrap_or(0) as i32
                        )
                    };
                } else {
                    unsafe {
                        ffi::openvr_settings_set_float(
                            self.vr_settings,
                            section.as_ptr(),
                            c_key.as_ptr(),
                            n.as_f64().unwrap_or(0.0) as f32
                        )
                    };
                }
            },
            serde_json::Value::Bool(b) => {
                unsafe {
                    ffi::openvr_settings_set_bool(
                        self.vr_settings,
                        section.as_ptr(),
                        c_key.as_ptr(),
                        b
                    )
                };
            },
            serde_json::Value::String(s) => {
                let c_value = std::ffi::CString::new(s)
                    .map_err(|_| Error::FFIError("Invalid string value".to_string()))?;
                
                unsafe {
                    ffi::openvr_settings_set_string(
                        self.vr_settings,
                        section.as_ptr(),
                        c_key.as_ptr(),
                        c_value.as_ptr()
                    )
                };
            },
            _ => return Err(Error::Unknown(format!("Unsupported value type for setting: {}", key))),
        }
        
        Ok(())
    }
}
