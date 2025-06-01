//! Privacy module for telemetry data.
//!
//! This module provides functionality for applying privacy settings to telemetry
//! data, ensuring that sensitive information is properly handled according to
//! user preferences.

use std::collections::HashMap;
use anyhow::{Result, Context};

use super::{TelemetryDataPoint, TelemetryValue, PrivacySettings};

/// Apply privacy settings to a telemetry data point.
///
/// This function filters and modifies telemetry data according to the user's
/// privacy settings, removing or anonymizing sensitive information.
///
/// # Arguments
///
/// * `data_point` - The telemetry data point to process
/// * `settings` - The privacy settings to apply
///
/// # Returns
///
/// The processed telemetry data point with privacy settings applied.
pub fn apply_privacy_settings(
    mut data_point: TelemetryDataPoint,
    settings: &PrivacySettings,
) -> TelemetryDataPoint {
    // Apply category-specific privacy settings
    match data_point.category {
        super::TelemetryCategory::Application => {
            if !settings.collect_application_usage {
                // Replace with minimal data
                data_point.value = TelemetryValue::String("Data redacted due to privacy settings".to_string());
                data_point.metadata = HashMap::new();
            }
        },
        super::TelemetryCategory::Network => {
            if !settings.collect_network_diagnostics {
                // Replace with minimal data
                data_point.value = TelemetryValue::String("Data redacted due to privacy settings".to_string());
                data_point.metadata = HashMap::new();
            } else {
                // Remove IP addresses if not allowed
                if !settings.include_ip_addresses {
                    data_point.metadata.remove("ip");
                    data_point.metadata.remove("address");
                    data_point.metadata.remove("host");
                    
                    // Also check for IP addresses in the value
                    data_point.value = redact_ip_addresses(data_point.value);
                }
            }
        },
        super::TelemetryCategory::Error => {
            if !settings.collect_crash_reports {
                // Replace with minimal data
                data_point.value = TelemetryValue::String("Data redacted due to privacy settings".to_string());
                data_point.metadata = HashMap::new();
            }
        },
        super::TelemetryCategory::UserInteraction => {
            if !settings.collect_usage_statistics {
                // Replace with minimal data
                data_point.value = TelemetryValue::String("Data redacted due to privacy settings".to_string());
                data_point.metadata = HashMap::new();
            }
        },
        _ => {
            // Other categories are processed by general rules below
        }
    }
    
    // Apply general privacy settings
    
    // Remove user identifiers if not allowed
    if !settings.include_user_identifiers {
        data_point.metadata.remove("user");
        data_point.metadata.remove("username");
        data_point.metadata.remove("user_id");
        data_point.metadata.remove("account");
        data_point.metadata.remove("email");
    }
    
    // Remove device identifiers if not allowed
    if !settings.include_device_identifiers {
        data_point.metadata.remove("device_id");
        data_point.metadata.remove("serial");
        data_point.metadata.remove("mac");
        data_point.metadata.remove("uuid");
    }
    
    // Remove location data if not allowed
    if !settings.include_location_data {
        data_point.metadata.remove("location");
        data_point.metadata.remove("gps");
        data_point.metadata.remove("latitude");
        data_point.metadata.remove("longitude");
        data_point.metadata.remove("geo");
        data_point.metadata.remove("country");
        data_point.metadata.remove("city");
    }
    
    // Remove any explicitly excluded fields
    for field in &settings.excluded_fields {
        data_point.metadata.remove(field);
    }
    
    // Process nested values in the telemetry value
    data_point.value = process_value_privacy(data_point.value, settings);
    
    data_point
}

/// Process a telemetry value for privacy.
///
/// This function recursively processes telemetry values, applying privacy
/// settings to nested structures.
///
/// # Arguments
///
/// * `value` - The telemetry value to process
/// * `settings` - The privacy settings to apply
///
/// # Returns
///
/// The processed telemetry value with privacy settings applied.
fn process_value_privacy(value: TelemetryValue, settings: &PrivacySettings) -> TelemetryValue {
    match value {
        TelemetryValue::String(s) => {
            // Check if the string contains any sensitive information
            let mut processed = s;
            
            // Redact IP addresses if not allowed
            if !settings.include_ip_addresses {
                processed = redact_ip_pattern(&processed);
            }
            
            // Redact location data if not allowed
            if !settings.include_location_data {
                processed = redact_location_pattern(&processed);
            }
            
            // Redact user identifiers if not allowed
            if !settings.include_user_identifiers {
                processed = redact_user_pattern(&processed);
            }
            
            // Redact device identifiers if not allowed
            if !settings.include_device_identifiers {
                processed = redact_device_pattern(&processed);
            }
            
            // Redact excluded fields
            for field in &settings.excluded_fields {
                processed = redact_pattern(&processed, field);
            }
            
            TelemetryValue::String(processed)
        },
        TelemetryValue::Array(arr) => {
            // Process each element in the array
            let processed = arr.into_iter()
                .map(|v| process_value_privacy(v, settings))
                .collect();
            
            TelemetryValue::Array(processed)
        },
        TelemetryValue::Map(map) => {
            // Process each value in the map
            let mut processed = HashMap::new();
            
            for (k, v) in map {
                // Skip excluded fields
                if settings.excluded_fields.contains(&k) {
                    continue;
                }
                
                // Skip user identifiers if not allowed
                if !settings.include_user_identifiers && 
                   (k.contains("user") || k.contains("account") || k.contains("email")) {
                    continue;
                }
                
                // Skip device identifiers if not allowed
                if !settings.include_device_identifiers && 
                   (k.contains("device") || k.contains("serial") || k.contains("mac") || k.contains("uuid")) {
                    continue;
                }
                
                // Skip location data if not allowed
                if !settings.include_location_data && 
                   (k.contains("location") || k.contains("gps") || k.contains("latitude") || 
                    k.contains("longitude") || k.contains("geo") || k.contains("country") || 
                    k.contains("city")) {
                    continue;
                }
                
                // Skip IP addresses if not allowed
                if !settings.include_ip_addresses && 
                   (k.contains("ip") || k.contains("address") || k.contains("host")) {
                    continue;
                }
                
                // Process the value
                processed.insert(k, process_value_privacy(v, settings));
            }
            
            TelemetryValue::Map(processed)
        },
        // Other value types don't contain sensitive information
        _ => value,
    }
}

/// Redact IP addresses from a telemetry value.
///
/// # Arguments
///
/// * `value` - The telemetry value to process
///
/// # Returns
///
/// The processed telemetry value with IP addresses redacted.
fn redact_ip_addresses(value: TelemetryValue) -> TelemetryValue {
    match value {
        TelemetryValue::String(s) => {
            TelemetryValue::String(redact_ip_pattern(&s))
        },
        TelemetryValue::Array(arr) => {
            let processed = arr.into_iter()
                .map(redact_ip_addresses)
                .collect();
            
            TelemetryValue::Array(processed)
        },
        TelemetryValue::Map(map) => {
            let mut processed = HashMap::new();
            
            for (k, v) in map {
                if k.contains("ip") || k.contains("address") || k.contains("host") {
                    // Skip this field
                    continue;
                }
                
                processed.insert(k, redact_ip_addresses(v));
            }
            
            TelemetryValue::Map(processed)
        },
        // Other value types don't contain IP addresses
        _ => value,
    }
}

/// Redact IP addresses from a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with IP addresses redacted.
fn redact_ip_pattern(s: &str) -> String {
    // Simple regex-like pattern for IPv4 addresses
    // This is a simplified implementation and may not catch all cases
    let mut result = s.to_string();
    
    // Find patterns like "192.168.1.1" or "10.0.0.1"
    let mut i = 0;
    while i < result.len() {
        let slice = &result[i..];
        if let Some(pos) = slice.find(|c: char| c.is_digit(10)) {
            let start = i + pos;
            let mut end = start;
            let mut dots = 0;
            let mut valid = true;
            
            for (j, c) in slice[pos..].chars().enumerate() {
                if c.is_digit(10) {
                    end = start + j + 1;
                } else if c == '.' {
                    dots += 1;
                    end = start + j + 1;
                    if dots > 3 {
                        valid = false;
                        break;
                    }
                } else {
                    break;
                }
            }
            
            if valid && dots == 3 && end > start + 6 {
                // Looks like an IP address, redact it
                let ip = &result[start..end];
                if ip.split('.').count() == 4 && ip.split('.').all(|part| {
                    if let Ok(num) = part.parse::<u8>() {
                        true
                    } else {
                        false
                    }
                }) {
                    result.replace_range(start..end, "[REDACTED_IP]");
                    i = start + 13; // Length of "[REDACTED_IP]"
                    continue;
                }
            }
            
            i = start + 1;
        } else {
            break;
        }
    }
    
    result
}

/// Redact location data from a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with location data redacted.
fn redact_location_pattern(s: &str) -> String {
    let mut result = s.to_string();
    
    // Redact patterns that look like coordinates
    // This is a simplified implementation and may not catch all cases
    
    // Pattern like "latitude: 37.7749, longitude: -122.4194"
    if result.contains("latitude") && result.contains("longitude") {
        result = result.replace(
            &regex_replace(r"latitude: -?\d+\.\d+, longitude: -?\d+\.\d+", &result),
            "latitude: [REDACTED], longitude: [REDACTED]"
        );
    }
    
    // Pattern like "lat: 37.7749, long: -122.4194"
    if result.contains("lat") && result.contains("long") {
        result = result.replace(
            &regex_replace(r"lat: -?\d+\.\d+, long: -?\d+\.\d+", &result),
            "lat: [REDACTED], long: [REDACTED]"
        );
    }
    
    // Pattern like "37.7749,-122.4194"
    result = result.replace(
        &regex_replace(r"-?\d+\.\d+,-?\d+\.\d+", &result),
        "[REDACTED_COORDINATES]"
    );
    
    result
}

/// Redact user identifiers from a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with user identifiers redacted.
fn redact_user_pattern(s: &str) -> String {
    let mut result = s.to_string();
    
    // Redact patterns that look like user identifiers
    // This is a simplified implementation and may not catch all cases
    
    // Pattern like "user: username"
    result = result.replace(
        &regex_replace(r"user: \w+", &result),
        "user: [REDACTED]"
    );
    
    // Pattern like "username: username"
    result = result.replace(
        &regex_replace(r"username: \w+", &result),
        "username: [REDACTED]"
    );
    
    // Pattern like "email: user@example.com"
    result = result.replace(
        &regex_replace(r"email: \S+@\S+\.\S+", &result),
        "email: [REDACTED]"
    );
    
    // Pattern like "user@example.com"
    result = result.replace(
        &regex_replace(r"\S+@\S+\.\S+", &result),
        "[REDACTED_EMAIL]"
    );
    
    result
}

/// Redact device identifiers from a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with device identifiers redacted.
fn redact_device_pattern(s: &str) -> String {
    let mut result = s.to_string();
    
    // Redact patterns that look like device identifiers
    // This is a simplified implementation and may not catch all cases
    
    // Pattern like "device: device_id"
    result = result.replace(
        &regex_replace(r"device: \w+", &result),
        "device: [REDACTED]"
    );
    
    // Pattern like "device_id: device_id"
    result = result.replace(
        &regex_replace(r"device_id: \w+", &result),
        "device_id: [REDACTED]"
    );
    
    // Pattern like "serial: serial_number"
    result = result.replace(
        &regex_replace(r"serial: \w+", &result),
        "serial: [REDACTED]"
    );
    
    // Pattern like "mac: 00:11:22:33:44:55"
    result = result.replace(
        &regex_replace(r"mac: (?:[0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}", &result),
        "mac: [REDACTED]"
    );
    
    // Pattern like "00:11:22:33:44:55"
    result = result.replace(
        &regex_replace(r"(?:[0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}", &result),
        "[REDACTED_MAC]"
    );
    
    // Pattern like "uuid: 123e4567-e89b-12d3-a456-426614174000"
    result = result.replace(
        &regex_replace(r"uuid: [0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", &result),
        "uuid: [REDACTED]"
    );
    
    // Pattern like "123e4567-e89b-12d3-a456-426614174000"
    result = result.replace(
        &regex_replace(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", &result),
        "[REDACTED_UUID]"
    );
    
    result
}

/// Redact a specific pattern from a string.
///
/// # Arguments
///
/// * `s` - The string to process
/// * `pattern` - The pattern to redact
///
/// # Returns
///
/// The processed string with the pattern redacted.
fn redact_pattern(s: &str, pattern: &str) -> String {
    let mut result = s.to_string();
    
    // Redact the pattern and any value associated with it
    // This is a simplified implementation and may not catch all cases
    
    // Pattern like "pattern: value"
    let pattern_colon = format!("{}: ", pattern);
    if result.contains(&pattern_colon) {
        let start = result.find(&pattern_colon).unwrap();
        let end = result[start..].find('\n').map(|pos| start + pos).unwrap_or(result.len());
        
        result.replace_range(start..end, &format!("{}: [REDACTED]", pattern));
    }
    
    // Pattern like "pattern=value"
    let pattern_equals = format!("{}=", pattern);
    if result.contains(&pattern_equals) {
        let start = result.find(&pattern_equals).unwrap();
        let end = result[start..].find(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .map(|pos| start + pos)
            .unwrap_or(result.len());
        
        result.replace_range(start..end, &format!("{}=[REDACTED]", pattern));
    }
    
    result
}

/// Simple regex-like pattern replacement.
///
/// This is a very simplified implementation that only handles a few basic patterns.
/// In a real implementation, this would use a proper regex library.
///
/// # Arguments
///
/// * `pattern` - The pattern to match
/// * `s` - The string to search
///
/// # Returns
///
/// The matched substring, or an empty string if no match.
fn regex_replace(pattern: &str, s: &str) -> String {
    // This is a very simplified implementation
    // In a real implementation, this would use a proper regex library
    
    match pattern {
        r"latitude: -?\d+\.\d+, longitude: -?\d+\.\d+" => {
            if s.contains("latitude") && s.contains("longitude") {
                for (i, _) in s.match_indices("latitude") {
                    let slice = &s[i..];
                    if slice.starts_with("latitude: ") {
                        let end = slice.find('\n').unwrap_or(slice.len());
                        let line = &slice[..end];
                        if line.contains("longitude") {
                            return line.to_string();
                        }
                    }
                }
            }
            String::new()
        },
        r"lat: -?\d+\.\d+, long: -?\d+\.\d+" => {
            if s.contains("lat") && s.contains("long") {
                for (i, _) in s.match_indices("lat") {
                    let slice = &s[i..];
                    if slice.starts_with("lat: ") {
                        let end = slice.find('\n').unwrap_or(slice.len());
                        let line = &slice[..end];
                        if line.contains("long") {
                            return line.to_string();
                        }
                    }
                }
            }
            String::new()
        },
        r"-?\d+\.\d+,-?\d+\.\d+" => {
            for (i, c) in s.char_indices() {
                if c.is_digit(10) || c == '-' {
                    let slice = &s[i..];
                    let mut j = 0;
                    let mut dots = 0;
                    let mut commas = 0;
                    
                    for (k, c) in slice.char_indices() {
                        if c.is_digit(10) || c == '.' || c == '-' || c == ',' {
                            j = k + 1;
                            if c == '.' {
                                dots += 1;
                            } else if c == ',' {
                                commas += 1;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    if dots == 2 && commas == 1 && j > 10 {
                        return slice[..j].to_string();
                    }
                }
            }
            String::new()
        },
        r"user: \w+" => {
            if s.contains("user: ") {
                for (i, _) in s.match_indices("user: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    return slice[..end].to_string();
                }
            }
            String::new()
        },
        r"username: \w+" => {
            if s.contains("username: ") {
                for (i, _) in s.match_indices("username: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    return slice[..end].to_string();
                }
            }
            String::new()
        },
        r"email: \S+@\S+\.\S+" => {
            if s.contains("email: ") {
                for (i, _) in s.match_indices("email: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace())
                        .unwrap_or(slice.len());
                    let email = &slice[..end];
                    if email.contains('@') && email.contains('.') {
                        return email.to_string();
                    }
                }
            }
            String::new()
        },
        r"\S+@\S+\.\S+" => {
            for (i, c) in s.char_indices() {
                if c == '@' {
                    // Look backwards for the start of the email
                    let mut start = i;
                    while start > 0 && !s[start-1..start].chars().next().unwrap().is_whitespace() {
                        start -= 1;
                    }
                    
                    // Look forwards for the end of the email
                    let mut end = i;
                    while end < s.len() && !s[end..end+1].chars().next().unwrap().is_whitespace() {
                        end += 1;
                    }
                    
                    let email = &s[start..end];
                    if email.contains('.') {
                        return email.to_string();
                    }
                }
            }
            String::new()
        },
        r"device: \w+" => {
            if s.contains("device: ") {
                for (i, _) in s.match_indices("device: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    return slice[..end].to_string();
                }
            }
            String::new()
        },
        r"device_id: \w+" => {
            if s.contains("device_id: ") {
                for (i, _) in s.match_indices("device_id: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    return slice[..end].to_string();
                }
            }
            String::new()
        },
        r"serial: \w+" => {
            if s.contains("serial: ") {
                for (i, _) in s.match_indices("serial: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    return slice[..end].to_string();
                }
            }
            String::new()
        },
        r"mac: (?:[0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}" => {
            if s.contains("mac: ") {
                for (i, _) in s.match_indices("mac: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    let mac = &slice[..end];
                    if mac.matches(':').count() == 5 {
                        return mac.to_string();
                    }
                }
            }
            String::new()
        },
        r"(?:[0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}" => {
            for (i, c) in s.char_indices() {
                if (c.is_digit(16) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')) && i + 16 < s.len() {
                    let slice = &s[i..];
                    if slice.len() >= 17 && slice.matches(':').count() == 5 {
                        let mut valid = true;
                        let mut j = 0;
                        
                        for k in 0..6 {
                            if !slice[j..j+2].chars().all(|c| c.is_digit(16) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')) {
                                valid = false;
                                break;
                            }
                            j += 2;
                            
                            if k < 5 && slice[j..j+1].chars().next().unwrap() != ':' {
                                valid = false;
                                break;
                            }
                            j += 1;
                        }
                        
                        if valid {
                            return slice[..17].to_string();
                        }
                    }
                }
            }
            String::new()
        },
        r"uuid: [0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}" => {
            if s.contains("uuid: ") {
                for (i, _) in s.match_indices("uuid: ") {
                    let slice = &s[i..];
                    let end = slice.find(|c: char| c.is_whitespace() && c != ' ')
                        .unwrap_or(slice.len());
                    let uuid = &slice[..end];
                    if uuid.matches('-').count() == 4 {
                        return uuid.to_string();
                    }
                }
            }
            String::new()
        },
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}" => {
            for (i, c) in s.char_indices() {
                if (c.is_digit(16) || (c >= 'a' && c <= 'f')) && i + 35 < s.len() {
                    let slice = &s[i..];
                    if slice.len() >= 36 && slice.matches('-').count() == 4 {
                        let parts = slice[..36].split('-').collect::<Vec<_>>();
                        if parts.len() == 5 && 
                           parts[0].len() == 8 && 
                           parts[1].len() == 4 && 
                           parts[2].len() == 4 && 
                           parts[3].len() == 4 && 
                           parts[4].len() == 12 {
                            return slice[..36].to_string();
                        }
                    }
                }
            }
            String::new()
        },
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_redact_ip_pattern() {
        let s = "The server IP is 192.168.1.1 and the gateway is 10.0.0.1";
        let result = redact_ip_pattern(s);
        assert_eq!(result, "The server IP is [REDACTED_IP] and the gateway is [REDACTED_IP]");
    }
    
    #[test]
    fn test_redact_location_pattern() {
        let s = "The coordinates are 37.7749,-122.4194 and latitude: 37.7749, longitude: -122.4194";
        let result = redact_location_pattern(s);
        assert_eq!(result, "The coordinates are [REDACTED_COORDINATES] and latitude: [REDACTED], longitude: [REDACTED]");
    }
    
    #[test]
    fn test_redact_user_pattern() {
        let s = "The user is user: john and the email is email: john@example.com";
        let result = redact_user_pattern(s);
        assert_eq!(result, "The user is user: [REDACTED] and the email is email: [REDACTED]");
    }
    
    #[test]
    fn test_redact_device_pattern() {
        let s = "The device is device: abc123 and the MAC is mac: 00:11:22:33:44:55";
        let result = redact_device_pattern(s);
        assert_eq!(result, "The device is device: [REDACTED] and the MAC is mac: [REDACTED]");
    }
    
    #[test]
    fn test_apply_privacy_settings() {
        let mut settings = PrivacySettings::default();
        settings.include_ip_addresses = false;
        settings.include_location_data = false;
        settings.include_user_identifiers = false;
        settings.include_device_identifiers = false;
        
        let mut metadata = HashMap::new();
        metadata.insert("ip".to_string(), "192.168.1.1".to_string());
        metadata.insert("user".to_string(), "john".to_string());
        metadata.insert("device_id".to_string(), "abc123".to_string());
        metadata.insert("location".to_string(), "San Francisco".to_string());
        
        let data_point = TelemetryDataPoint {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            category: super::super::TelemetryCategory::System,
            name: "test".to_string(),
            value: TelemetryValue::String("The IP is 192.168.1.1".to_string()),
            metadata,
        };
        
        let result = apply_privacy_settings(data_point, &settings);
        
        assert!(!result.metadata.contains_key("ip"));
        assert!(!result.metadata.contains_key("user"));
        assert!(!result.metadata.contains_key("device_id"));
        assert!(!result.metadata.contains_key("location"));
        
        if let TelemetryValue::String(s) = result.value {
            assert_eq!(s, "The IP is [REDACTED_IP]");
        } else {
            panic!("Expected String value");
        }
    }
}
