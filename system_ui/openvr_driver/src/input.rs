//! Input handling for the OpenVR driver

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::{Button, ButtonState, Axis};
use crate::error::{Result, Error};

/// Interface for input handling
pub trait InputInterface: Send + Sync {
    /// Update button state
    fn update_button(&mut self, device_serial: &str, button: Button, state: ButtonState) -> Result<()>;
    
    /// Update axis value
    fn update_axis(&mut self, device_serial: &str, axis: Axis, x: f32, y: f32) -> Result<()>;
    
    /// Trigger haptic pulse
    fn trigger_haptic_pulse(&mut self, device_serial: &str, duration_micros: u16, frequency: u16, amplitude: f32) -> Result<()>;
}

/// Input handler for OpenVR
pub struct InputHandler {
    /// OpenVR driver input interface pointer (opaque)
    driver_input: *mut std::ffi::c_void,
    
    /// Map of device serials to their input handles
    device_handles: HashMap<String, u32>,
    
    /// Map of device serials to their button states
    button_states: HashMap<String, HashMap<Button, ButtonState>>,
    
    /// Map of device serials to their axis states
    axis_states: HashMap<String, HashMap<Axis, (f32, f32)>>,
}

// FFI functions for input handling
mod ffi {
    use super::*;
    
    extern "C" {
        pub fn openvr_update_boolean_component(
            driver_input: *mut std::ffi::c_void,
            component_handle: u64,
            value: bool,
            time_offset_seconds: f32
        ) -> bool;
        
        pub fn openvr_update_scalar_component(
            driver_input: *mut std::ffi::c_void,
            component_handle: u64,
            value: f32,
            time_offset_seconds: f32
        ) -> bool;
        
        pub fn openvr_trigger_haptic_pulse(
            driver_input: *mut std::ffi::c_void,
            component_handle: u64,
            haptic_index: u32,
            duration_micros: u16,
            frequency: u16,
            amplitude: f32
        ) -> bool;
    }
}

impl InputHandler {
    /// Create a new input handler
    pub fn new(driver_input: *mut std::ffi::c_void) -> Self {
        Self {
            driver_input,
            device_handles: HashMap::new(),
            button_states: HashMap::new(),
            axis_states: HashMap::new(),
        }
    }
    
    /// Register a device for input handling
    pub fn register_device(&mut self, serial: &str, device_handle: u32) {
        self.device_handles.insert(serial.to_string(), device_handle);
        self.button_states.insert(serial.to_string(), HashMap::new());
        self.axis_states.insert(serial.to_string(), HashMap::new());
    }
    
    /// Unregister a device
    pub fn unregister_device(&mut self, serial: &str) {
        self.device_handles.remove(serial);
        self.button_states.remove(serial);
        self.axis_states.remove(serial);
    }
    
    /// Get current button state
    pub fn get_button_state(&self, device_serial: &str, button: &Button) -> Option<ButtonState> {
        self.button_states.get(device_serial)
            .and_then(|buttons| buttons.get(button).cloned())
    }
    
    /// Get current axis state
    pub fn get_axis_state(&self, device_serial: &str, axis: &Axis) -> Option<(f32, f32)> {
        self.axis_states.get(device_serial)
            .and_then(|axes| axes.get(axis).cloned())
    }
}

impl InputInterface for InputHandler {
    fn update_button(&mut self, device_serial: &str, button: Button, state: ButtonState) -> Result<()> {
        // Store the state locally
        if let Some(buttons) = self.button_states.get_mut(device_serial) {
            buttons.insert(button, state);
        } else {
            return Err(Error::InvalidDeviceIndex(0));
        }
        
        // Update through OpenVR
        let success = unsafe {
            ffi::openvr_update_boolean_component(
                self.driver_input,
                button.component_handle,
                state.pressed,
                0.0 // Immediate update
            )
        };
        
        if !success {
            return Err(Error::FFIError("Failed to update button state".to_string()));
        }
        
        // If the button is also touchable, update touch state
        if state.touched != state.pressed {
            // In a real implementation, there would be separate component handles for touch
            // This is simplified for the example
        }
        
        Ok(())
    }
    
    fn update_axis(&mut self, device_serial: &str, axis: Axis, x: f32, y: f32) -> Result<()> {
        // Store the state locally
        if let Some(axes) = self.axis_states.get_mut(device_serial) {
            axes.insert(axis, (x, y));
        } else {
            return Err(Error::InvalidDeviceIndex(0));
        }
        
        // Update X component through OpenVR
        let success_x = unsafe {
            ffi::openvr_update_scalar_component(
                self.driver_input,
                axis.x_handle,
                x,
                0.0 // Immediate update
            )
        };
        
        if !success_x {
            return Err(Error::FFIError("Failed to update axis X value".to_string()));
        }
        
        // Update Y component if applicable
        if axis.has_y {
            let success_y = unsafe {
                ffi::openvr_update_scalar_component(
                    self.driver_input,
                    axis.y_handle,
                    y,
                    0.0 // Immediate update
                )
            };
            
            if !success_y {
                return Err(Error::FFIError("Failed to update axis Y value".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn trigger_haptic_pulse(&mut self, device_serial: &str, duration_micros: u16, frequency: u16, amplitude: f32) -> Result<()> {
        // Get device handle
        let handle = self.device_handles.get(device_serial)
            .ok_or_else(|| Error::InvalidDeviceIndex(0))?;
        
        // Trigger haptic pulse through OpenVR
        let success = unsafe {
            ffi::openvr_trigger_haptic_pulse(
                self.driver_input,
                *handle as u64, // Component handle would be different in real implementation
                0, // First haptic
                duration_micros,
                frequency,
                amplitude
            )
        };
        
        if !success {
            return Err(Error::FFIError("Failed to trigger haptic pulse".to_string()));
        }
        
        Ok(())
    }
}
