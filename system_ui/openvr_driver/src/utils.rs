//! Utility functions for the OpenVR driver

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use crate::error::{Result, Error};

/// Convert a Rust string to a C string
pub fn to_c_string(s: &str) -> Result<CString> {
    CString::new(s).map_err(|e| Error::FFIError(format!("Failed to convert string to C string: {}", e)))
}

/// Convert a C string to a Rust string
pub fn from_c_string(s: *const c_char) -> Result<String> {
    if s.is_null() {
        return Err(Error::FFIError("Null C string pointer".to_string()));
    }
    
    unsafe {
        let c_str = CStr::from_ptr(s);
        Ok(c_str.to_string_lossy().into_owned())
    }
}

/// Log a message to the OpenVR log
pub fn log_message(vr_driver_log: *mut c_void, message: &str) -> Result<()> {
    let c_message = to_c_string(&format!("{}\n", message))?;
    
    unsafe {
        openvr_log(vr_driver_log, c_message.as_ptr());
    }
    
    Ok(())
}

/// Convert quaternion to Euler angles (in radians)
pub fn quaternion_to_euler(q: &[f32; 4]) -> [f32; 3] {
    let [x, y, z, w] = *q;
    
    // Roll (x-axis rotation)
    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);
    
    // Pitch (y-axis rotation)
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp) // Use 90 degrees if out of range
    } else {
        sinp.asin()
    };
    
    // Yaw (z-axis rotation)
    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);
    
    [roll, pitch, yaw]
}

/// Convert Euler angles (in radians) to quaternion
pub fn euler_to_quaternion(euler: &[f32; 3]) -> [f32; 4] {
    let [roll, pitch, yaw] = *euler;
    
    let cr = (roll * 0.5).cos();
    let sr = (roll * 0.5).sin();
    let cp = (pitch * 0.5).cos();
    let sp = (pitch * 0.5).sin();
    let cy = (yaw * 0.5).cos();
    let sy = (yaw * 0.5).sin();
    
    let w = cr * cp * cy + sr * sp * sy;
    let x = sr * cp * cy - cr * sp * sy;
    let y = cr * sp * cy + sr * cp * sy;
    let z = cr * cp * sy - sr * sp * cy;
    
    [x, y, z, w]
}

/// Convert a 4x4 matrix to a quaternion and position
pub fn matrix_to_pose(matrix: &[[f32; 4]; 4]) -> ([f32; 3], [f32; 4]) {
    // Extract position
    let position = [matrix[0][3], matrix[1][3], matrix[2][3]];
    
    // Extract rotation matrix
    let m = [
        [matrix[0][0], matrix[0][1], matrix[0][2]],
        [matrix[1][0], matrix[1][1], matrix[1][2]],
        [matrix[2][0], matrix[2][1], matrix[2][2]],
    ];
    
    // Convert rotation matrix to quaternion
    let trace = m[0][0] + m[1][1] + m[2][2];
    
    let mut quaternion = [0.0, 0.0, 0.0, 0.0];
    
    if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        quaternion[3] = 0.25 / s;
        quaternion[0] = (m[2][1] - m[1][2]) * s;
        quaternion[1] = (m[0][2] - m[2][0]) * s;
        quaternion[2] = (m[1][0] - m[0][1]) * s;
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = 2.0 * (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt();
        quaternion[3] = (m[2][1] - m[1][2]) / s;
        quaternion[0] = 0.25 * s;
        quaternion[1] = (m[0][1] + m[1][0]) / s;
        quaternion[2] = (m[0][2] + m[2][0]) / s;
    } else if m[1][1] > m[2][2] {
        let s = 2.0 * (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt();
        quaternion[3] = (m[0][2] - m[2][0]) / s;
        quaternion[0] = (m[0][1] + m[1][0]) / s;
        quaternion[1] = 0.25 * s;
        quaternion[2] = (m[1][2] + m[2][1]) / s;
    } else {
        let s = 2.0 * (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt();
        quaternion[3] = (m[1][0] - m[0][1]) / s;
        quaternion[0] = (m[0][2] + m[2][0]) / s;
        quaternion[1] = (m[1][2] + m[2][1]) / s;
        quaternion[2] = 0.25 * s;
    }
    
    (position, quaternion)
}

// FFI functions
extern "C" {
    fn openvr_log(vr_driver_log: *mut c_void, message: *const c_char);
}
