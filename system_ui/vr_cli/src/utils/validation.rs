// Validation utilities for CLI
use anyhow::{Result, anyhow};
use regex::Regex;
use std::path::Path;

/// Validate that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{} cannot be empty", field_name));
    }
    Ok(())
}

/// Validate that a string matches a regex pattern
pub fn validate_pattern(value: &str, pattern: &str, field_name: &str) -> Result<()> {
    let re = Regex::new(pattern).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
    if !re.is_match(value) {
        return Err(anyhow!("{} does not match the required pattern", field_name));
    }
    Ok(())
}

/// Validate that a string is a valid username
pub fn validate_username(username: &str) -> Result<()> {
    validate_not_empty(username, "Username")?;
    validate_pattern(username, r"^[a-zA-Z0-9_-]{3,32}$", "Username")?;
    Ok(())
}

/// Validate that a string is a valid password
pub fn validate_password(password: &str) -> Result<()> {
    validate_not_empty(password, "Password")?;
    
    if password.len() < 8 {
        return Err(anyhow!("Password must be at least 8 characters long"));
    }
    
    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(anyhow!("Password must contain at least one uppercase letter"));
    }
    
    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(anyhow!("Password must contain at least one lowercase letter"));
    }
    
    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(anyhow!("Password must contain at least one digit"));
    }
    
    // Check for at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(anyhow!("Password must contain at least one special character"));
    }
    
    Ok(())
}

/// Validate that a file exists
pub fn validate_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(anyhow!("Path is not a file: {}", path.display()));
    }
    Ok(())
}

/// Validate that a directory exists
pub fn validate_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("Directory does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", path.display()));
    }
    Ok(())
}

/// Validate that a string is a valid IP address
pub fn validate_ip_address(ip: &str) -> Result<()> {
    let re = Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap();
    if !re.is_match(ip) {
        return Err(anyhow!("Invalid IP address format"));
    }
    
    // Check that each octet is in range
    for octet in ip.split('.') {
        let num = octet.parse::<u8>().map_err(|_| anyhow!("Invalid IP address octet"))?;
        if num > 255 {
            return Err(anyhow!("IP address octet out of range"));
        }
    }
    
    Ok(())
}

/// Validate that a string is a valid port number
pub fn validate_port(port: &str) -> Result<()> {
    let port_num = port.parse::<u16>().map_err(|_| anyhow!("Invalid port number"))?;
    if port_num == 0 {
        return Err(anyhow!("Port number cannot be 0"));
    }
    Ok(())
}

/// Validate that a string is a valid URL
pub fn validate_url(url: &str) -> Result<()> {
    let re = Regex::new(r"^(http|https)://[a-zA-Z0-9]+([\-\.]{1}[a-zA-Z0-9]+)*\.[a-zA-Z]{2,}(:[0-9]{1,5})?(/.*)?$").unwrap();
    if !re.is_match(url) {
        return Err(anyhow!("Invalid URL format"));
    }
    Ok(())
}

/// Validate that a string is a valid email address
pub fn validate_email(email: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(email) {
        return Err(anyhow!("Invalid email address format"));
    }
    Ok(())
}

/// Validate that a string is a valid version number (semver)
pub fn validate_version(version: &str) -> Result<()> {
    let re = Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap();
    if !re.is_match(version) {
        return Err(anyhow!("Invalid version format (should be semver)"));
    }
    Ok(())
}
