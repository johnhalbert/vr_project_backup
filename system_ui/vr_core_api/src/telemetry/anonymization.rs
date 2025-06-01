//! Anonymization module for telemetry data.
//!
//! This module provides functionality for anonymizing telemetry data,
//! ensuring that sensitive information is properly obfuscated while
//! maintaining the utility of the data for analysis.

use std::collections::HashMap;
use anyhow::{Result, Context};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

use super::{TelemetryDataPoint, TelemetryValue, PrivacySettings};

/// Anonymize a telemetry data point.
///
/// This function anonymizes sensitive information in telemetry data
/// according to the user's privacy settings, replacing identifiers with
/// consistent but anonymous values.
///
/// # Arguments
///
/// * `data_point` - The telemetry data point to anonymize
/// * `settings` - The privacy settings to apply
///
/// # Returns
///
/// The anonymized telemetry data point.
pub fn anonymize_data_point(
    mut data_point: TelemetryDataPoint,
    settings: &PrivacySettings,
) -> TelemetryDataPoint {
    // Apply anonymization based on privacy settings
    
    // Anonymize device identifiers if needed but allowed
    if settings.include_device_identifiers {
        // We keep device identifiers but hash them for consistency
        for key in ["device_id", "serial", "uuid", "mac"] {
            if let Some(value) = data_point.metadata.get(key) {
                let hashed = hash_identifier(value);
                data_point.metadata.insert(key.to_string(), hashed);
            }
        }
    }
    
    // Anonymize user identifiers if needed but allowed
    if settings.include_user_identifiers {
        // We keep user identifiers but hash them for consistency
        for key in ["user", "username", "user_id", "account", "email"] {
            if let Some(value) = data_point.metadata.get(key) {
                let hashed = hash_identifier(value);
                data_point.metadata.insert(key.to_string(), hashed);
            }
        }
    }
    
    // Anonymize location data if needed but allowed
    if settings.include_location_data {
        // We keep location data but reduce precision
        for key in ["latitude", "longitude"] {
            if let Some(value) = data_point.metadata.get(key) {
                if let Ok(coord) = value.parse::<f64>() {
                    // Reduce precision to ~10km by truncating to 1 decimal place
                    let reduced = (coord * 10.0).trunc() / 10.0;
                    data_point.metadata.insert(key.to_string(), reduced.to_string());
                }
            }
        }
        
        // Replace specific location names with region names
        for key in ["location", "city"] {
            if let Some(value) = data_point.metadata.get(key) {
                let region = generalize_location(value);
                data_point.metadata.insert(key.to_string(), region);
            }
        }
    }
    
    // Anonymize IP addresses if needed but allowed
    if settings.include_ip_addresses {
        // We keep IP addresses but mask the last octet
        for key in ["ip", "address"] {
            if let Some(value) = data_point.metadata.get(key) {
                let masked = mask_ip_address(value);
                data_point.metadata.insert(key.to_string(), masked);
            }
        }
    }
    
    // Process the telemetry value
    data_point.value = anonymize_value(data_point.value, settings);
    
    data_point
}

/// Anonymize a telemetry value.
///
/// This function recursively anonymizes telemetry values, applying
/// anonymization to nested structures.
///
/// # Arguments
///
/// * `value` - The telemetry value to anonymize
/// * `settings` - The privacy settings to apply
///
/// # Returns
///
/// The anonymized telemetry value.
fn anonymize_value(value: TelemetryValue, settings: &PrivacySettings) -> TelemetryValue {
    match value {
        TelemetryValue::String(s) => {
            // Anonymize strings that might contain sensitive information
            let mut result = s;
            
            // Anonymize IP addresses if needed but allowed
            if settings.include_ip_addresses {
                result = anonymize_ip_addresses_in_string(&result);
            }
            
            // Anonymize user identifiers if needed but allowed
            if settings.include_user_identifiers {
                result = anonymize_user_identifiers_in_string(&result);
            }
            
            // Anonymize device identifiers if needed but allowed
            if settings.include_device_identifiers {
                result = anonymize_device_identifiers_in_string(&result);
            }
            
            // Anonymize location data if needed but allowed
            if settings.include_location_data {
                result = anonymize_location_data_in_string(&result);
            }
            
            TelemetryValue::String(result)
        },
        TelemetryValue::Array(arr) => {
            // Anonymize each element in the array
            let anonymized = arr.into_iter()
                .map(|v| anonymize_value(v, settings))
                .collect();
            
            TelemetryValue::Array(anonymized)
        },
        TelemetryValue::Map(map) => {
            // Anonymize each value in the map
            let mut anonymized = HashMap::new();
            
            for (k, v) in map {
                // Anonymize the value
                let anonymized_value = anonymize_value(v, settings);
                
                // Check if the key needs to be anonymized
                let anonymized_key = if (k.contains("user") || k.contains("account") || k.contains("email")) && settings.include_user_identifiers {
                    format!("{}_anon", k)
                } else if (k.contains("device") || k.contains("serial") || k.contains("mac") || k.contains("uuid")) && settings.include_device_identifiers {
                    format!("{}_anon", k)
                } else if (k.contains("location") || k.contains("gps") || k.contains("latitude") || k.contains("longitude") || k.contains("geo") || k.contains("country") || k.contains("city")) && settings.include_location_data {
                    format!("{}_anon", k)
                } else if (k.contains("ip") || k.contains("address") || k.contains("host")) && settings.include_ip_addresses {
                    format!("{}_anon", k)
                } else {
                    k
                };
                
                anonymized.insert(anonymized_key, anonymized_value);
            }
            
            TelemetryValue::Map(anonymized)
        },
        // Other value types don't need anonymization
        _ => value,
    }
}

/// Hash an identifier for anonymization.
///
/// This function creates a consistent hash of an identifier, allowing
/// for correlation without revealing the original value.
///
/// # Arguments
///
/// * `identifier` - The identifier to hash
///
/// # Returns
///
/// A hashed version of the identifier.
fn hash_identifier(identifier: &str) -> String {
    // Create a SHA-256 hash of the identifier
    let mut hasher = Sha256::new();
    hasher.update(identifier.as_bytes());
    let result = hasher.finalize();
    
    // Convert to base64 and truncate for readability
    let b64 = general_purpose::STANDARD.encode(&result);
    format!("anon_{}", &b64[0..8])
}

/// Generalize a location name.
///
/// This function replaces specific location names with more general
/// region names to protect privacy.
///
/// # Arguments
///
/// * `location` - The location name to generalize
///
/// # Returns
///
/// A generalized version of the location.
fn generalize_location(location: &str) -> String {
    // This is a simplified implementation
    // In a real implementation, this would use a geographic database
    
    // For now, just return a general region based on the first letter
    let first_char = location.chars().next().unwrap_or('A');
    
    match first_char {
        'A'..='C' => "Region North".to_string(),
        'D'..='F' => "Region East".to_string(),
        'G'..='J' => "Region South".to_string(),
        'K'..='M' => "Region West".to_string(),
        'N'..='P' => "Region Central".to_string(),
        'Q'..='S' => "Region Northwest".to_string(),
        'T'..='V' => "Region Northeast".to_string(),
        'W'..='Z' => "Region Southwest".to_string(),
        _ => "Region Unknown".to_string(),
    }
}

/// Mask an IP address.
///
/// This function masks the last octet of an IPv4 address or a portion
/// of an IPv6 address to protect privacy while maintaining network information.
///
/// # Arguments
///
/// * `ip` - The IP address to mask
///
/// # Returns
///
/// A masked version of the IP address.
fn mask_ip_address(ip: &str) -> String {
    // Check if it's an IPv4 address
    if ip.contains('.') {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            // Mask the last octet
            return format!("{}.{}.{}.xxx", parts[0], parts[1], parts[2]);
        }
    }
    
    // Check if it's an IPv6 address
    if ip.contains(':') {
        let parts: Vec<&str> = ip.split(':').collect();
        if parts.len() > 4 {
            // Mask the last 64 bits (last 4 segments)
            let visible = parts.iter().take(4).cloned().collect::<Vec<_>>().join(":");
            return format!("{}:xxxx:xxxx:xxxx:xxxx", visible);
        }
    }
    
    // If we can't parse it, return a generic masked value
    "xxx.xxx.xxx.xxx".to_string()
}

/// Anonymize IP addresses in a string.
///
/// This function finds and anonymizes IP addresses in a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with IP addresses anonymized.
fn anonymize_ip_addresses_in_string(s: &str) -> String {
    let mut result = s.to_string();
    
    // Find IPv4 addresses
    let ipv4_pattern = r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}";
    
    // This is a simplified implementation
    // In a real implementation, this would use a proper regex library
    
    // Look for patterns like "192.168.1.1"
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
                // Looks like an IP address, anonymize it
                let ip = &result[start..end];
                if ip.split('.').count() == 4 && ip.split('.').all(|part| {
                    if let Ok(num) = part.parse::<u8>() {
                        true
                    } else {
                        false
                    }
                }) {
                    let masked = mask_ip_address(ip);
                    result.replace_range(start..end, &masked);
                    i = start + masked.len();
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

/// Anonymize user identifiers in a string.
///
/// This function finds and anonymizes user identifiers in a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with user identifiers anonymized.
fn anonymize_user_identifiers_in_string(s: &str) -> String {
    let mut result = s.to_string();
    
    // Anonymize email addresses
    // This is a simplified implementation
    // In a real implementation, this would use a proper regex library
    
    // Look for patterns like "user@example.com"
    let mut i = 0;
    while i < result.len() {
        let slice = &result[i..];
        if let Some(pos) = slice.find('@') {
            // Look backwards for the start of the email
            let mut start = i;
            let mut j = 1;
            while j <= pos && !slice[pos-j..pos-j+1].chars().next().unwrap().is_whitespace() {
                start = i + pos - j;
                j += 1;
                if start == i {
                    break;
                }
            }
            
            // Look forwards for the end of the email
            let mut end = i + pos;
            while end < result.len() && !result[end..end+1].chars().next().unwrap().is_whitespace() {
                end += 1;
                if end == result.len() {
                    break;
                }
            }
            
            let email = &result[start..end];
            if email.contains('.') {
                let hashed = hash_identifier(email);
                result.replace_range(start..end, &hashed);
                i = start + hashed.len();
                continue;
            }
            
            i = i + pos + 1;
        } else {
            break;
        }
    }
    
    // Anonymize usernames
    // This is a simplified implementation that looks for common patterns
    
    // Pattern like "user: username"
    let user_pattern = "user: ";
    if result.contains(user_pattern) {
        for (i, _) in result.match_indices(user_pattern) {
            let start = i + user_pattern.len();
            let end = start + result[start..].find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let username = &result[start..end];
            let hashed = hash_identifier(username);
            result.replace_range(start..end, &hashed);
        }
    }
    
    // Pattern like "username: username"
    let username_pattern = "username: ";
    if result.contains(username_pattern) {
        for (i, _) in result.match_indices(username_pattern) {
            let start = i + username_pattern.len();
            let end = start + result[start..].find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let username = &result[start..end];
            let hashed = hash_identifier(username);
            result.replace_range(start..end, &hashed);
        }
    }
    
    result
}

/// Anonymize device identifiers in a string.
///
/// This function finds and anonymizes device identifiers in a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with device identifiers anonymized.
fn anonymize_device_identifiers_in_string(s: &str) -> String {
    let mut result = s.to_string();
    
    // Anonymize MAC addresses
    // This is a simplified implementation
    // In a real implementation, this would use a proper regex library
    
    // Look for patterns like "00:11:22:33:44:55"
    let mut i = 0;
    while i < result.len() {
        let slice = &result[i..];
        if let Some(pos) = slice.find(':') {
            // Check if it's a MAC address pattern
            let start = i;
            let mut valid = true;
            let mut colons = 0;
            
            for j in 0..17 {
                if start + j >= result.len() {
                    valid = false;
                    break;
                }
                
                let c = result[start+j..start+j+1].chars().next().unwrap();
                
                if j % 3 == 2 {
                    // Should be a colon
                    if c != ':' {
                        valid = false;
                        break;
                    }
                    colons += 1;
                } else {
                    // Should be a hex digit
                    if !c.is_digit(16) {
                        valid = false;
                        break;
                    }
                }
            }
            
            if valid && colons == 5 {
                // Looks like a MAC address, anonymize it
                let mac = &result[start..start+17];
                let hashed = hash_identifier(mac);
                result.replace_range(start..start+17, &hashed);
                i = start + hashed.len();
                continue;
            }
            
            i = i + pos + 1;
        } else {
            break;
        }
    }
    
    // Anonymize UUIDs
    // This is a simplified implementation
    // In a real implementation, this would use a proper regex library
    
    // Look for patterns like "123e4567-e89b-12d3-a456-426614174000"
    let mut i = 0;
    while i < result.len() {
        let slice = &result[i..];
        if let Some(pos) = slice.find('-') {
            // Check if it's a UUID pattern
            let start = i;
            let mut valid = true;
            let mut hyphens = 0;
            
            for j in 0..36 {
                if start + j >= result.len() {
                    valid = false;
                    break;
                }
                
                let c = result[start+j..start+j+1].chars().next().unwrap();
                
                if j == 8 || j == 13 || j == 18 || j == 23 {
                    // Should be a hyphen
                    if c != '-' {
                        valid = false;
                        break;
                    }
                    hyphens += 1;
                } else {
                    // Should be a hex digit
                    if !c.is_digit(16) {
                        valid = false;
                        break;
                    }
                }
            }
            
            if valid && hyphens == 4 {
                // Looks like a UUID, anonymize it
                let uuid = &result[start..start+36];
                let hashed = hash_identifier(uuid);
                result.replace_range(start..start+36, &hashed);
                i = start + hashed.len();
                continue;
            }
            
            i = i + pos + 1;
        } else {
            break;
        }
    }
    
    // Anonymize device IDs
    // This is a simplified implementation that looks for common patterns
    
    // Pattern like "device: device_id"
    let device_pattern = "device: ";
    if result.contains(device_pattern) {
        for (i, _) in result.match_indices(device_pattern) {
            let start = i + device_pattern.len();
            let end = start + result[start..].find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let device_id = &result[start..end];
            let hashed = hash_identifier(device_id);
            result.replace_range(start..end, &hashed);
        }
    }
    
    // Pattern like "device_id: device_id"
    let device_id_pattern = "device_id: ";
    if result.contains(device_id_pattern) {
        for (i, _) in result.match_indices(device_id_pattern) {
            let start = i + device_id_pattern.len();
            let end = start + result[start..].find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let device_id = &result[start..end];
            let hashed = hash_identifier(device_id);
            result.replace_range(start..end, &hashed);
        }
    }
    
    result
}

/// Anonymize location data in a string.
///
/// This function finds and anonymizes location data in a string.
///
/// # Arguments
///
/// * `s` - The string to process
///
/// # Returns
///
/// The processed string with location data anonymized.
fn anonymize_location_data_in_string(s: &str) -> String {
    let mut result = s.to_string();
    
    // Anonymize coordinates
    // This is a simplified implementation
    // In a real implementation, this would use a proper regex library
    
    // Pattern like "latitude: 37.7749, longitude: -122.4194"
    if result.contains("latitude") && result.contains("longitude") {
        for (i, _) in result.match_indices("latitude") {
            let slice = &result[i..];
            if slice.starts_with("latitude: ") {
                let lat_start = i + "latitude: ".len();
                let lat_end = lat_start + slice["latitude: ".len()..].find(|c: char| c == ',' || c.is_whitespace())
                    .unwrap_or(slice["latitude: ".len()..].len());
                
                if let Some(long_pos) = slice.find("longitude: ") {
                    let long_start = i + long_pos + "longitude: ".len();
                    let long_end = long_start + slice[long_pos + "longitude: ".len()..].find(|c: char| c == ',' || c.is_whitespace())
                        .unwrap_or(slice[long_pos + "longitude: ".len()..].len());
                    
                    // Parse and reduce precision
                    if let (Ok(lat), Ok(long)) = (
                        result[lat_start..lat_end].parse::<f64>(),
                        result[long_start..long_end].parse::<f64>(),
                    ) {
                        let reduced_lat = (lat * 10.0).trunc() / 10.0;
                        let reduced_long = (long * 10.0).trunc() / 10.0;
                        
                        // Replace with reduced precision
                        result.replace_range(lat_start..lat_end, &reduced_lat.to_string());
                        
                        // Adjust long_start and long_end for any change in string length
                        let diff = reduced_lat.to_string().len() as isize - (lat_end - lat_start) as isize;
                        let adjusted_long_start = (long_start as isize + diff) as usize;
                        let adjusted_long_end = (long_end as isize + diff) as usize;
                        
                        result.replace_range(adjusted_long_start..adjusted_long_end, &reduced_long.to_string());
                    }
                }
            }
        }
    }
    
    // Pattern like "lat: 37.7749, long: -122.4194"
    if result.contains("lat") && result.contains("long") {
        for (i, _) in result.match_indices("lat") {
            let slice = &result[i..];
            if slice.starts_with("lat: ") {
                let lat_start = i + "lat: ".len();
                let lat_end = lat_start + slice["lat: ".len()..].find(|c: char| c == ',' || c.is_whitespace())
                    .unwrap_or(slice["lat: ".len()..].len());
                
                if let Some(long_pos) = slice.find("long: ") {
                    let long_start = i + long_pos + "long: ".len();
                    let long_end = long_start + slice[long_pos + "long: ".len()..].find(|c: char| c == ',' || c.is_whitespace())
                        .unwrap_or(slice[long_pos + "long: ".len()..].len());
                    
                    // Parse and reduce precision
                    if let (Ok(lat), Ok(long)) = (
                        result[lat_start..lat_end].parse::<f64>(),
                        result[long_start..long_end].parse::<f64>(),
                    ) {
                        let reduced_lat = (lat * 10.0).trunc() / 10.0;
                        let reduced_long = (long * 10.0).trunc() / 10.0;
                        
                        // Replace with reduced precision
                        result.replace_range(lat_start..lat_end, &reduced_lat.to_string());
                        
                        // Adjust long_start and long_end for any change in string length
                        let diff = reduced_lat.to_string().len() as isize - (lat_end - lat_start) as isize;
                        let adjusted_long_start = (long_start as isize + diff) as usize;
                        let adjusted_long_end = (long_end as isize + diff) as usize;
                        
                        result.replace_range(adjusted_long_start..adjusted_long_end, &reduced_long.to_string());
                    }
                }
            }
        }
    }
    
    // Pattern like "37.7749,-122.4194"
    let mut i = 0;
    while i < result.len() {
        let slice = &result[i..];
        if let Some(pos) = slice.find(|c: char| c.is_digit(10) || c == '-' || c == '.') {
            let start = i + pos;
            let mut comma_pos = None;
            let mut end = start;
            
            for (j, c) in slice[pos..].char_indices() {
                if c.is_digit(10) || c == '-' || c == '.' {
                    end = start + j + 1;
                } else if c == ',' {
                    comma_pos = Some(start + j);
                    end = start + j + 1;
                } else {
                    break;
                }
            }
            
            if let Some(comma_pos) = comma_pos {
                // Try to parse as coordinates
                let lat_str = &result[start..comma_pos];
                let long_str = &result[comma_pos+1..end];
                
                if let (Ok(lat), Ok(long)) = (lat_str.parse::<f64>(), long_str.parse::<f64>()) {
                    if lat >= -90.0 && lat <= 90.0 && long >= -180.0 && long <= 180.0 {
                        // Looks like coordinates, reduce precision
                        let reduced_lat = (lat * 10.0).trunc() / 10.0;
                        let reduced_long = (long * 10.0).trunc() / 10.0;
                        
                        let anonymized = format!("{},{}", reduced_lat, reduced_long);
                        result.replace_range(start..end, &anonymized);
                        i = start + anonymized.len();
                        continue;
                    }
                }
            }
            
            i = start + 1;
        } else {
            break;
        }
    }
    
    // Anonymize location names
    // This is a simplified implementation that looks for common patterns
    
    // Pattern like "location: San Francisco"
    let location_pattern = "location: ";
    if result.contains(location_pattern) {
        for (i, _) in result.match_indices(location_pattern) {
            let start = i + location_pattern.len();
            let end = start + result[start..].find(|c: char| c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let location = &result[start..end];
            let generalized = generalize_location(location);
            result.replace_range(start..end, &generalized);
        }
    }
    
    // Pattern like "city: San Francisco"
    let city_pattern = "city: ";
    if result.contains(city_pattern) {
        for (i, _) in result.match_indices(city_pattern) {
            let start = i + city_pattern.len();
            let end = start + result[start..].find(|c: char| c == ',' || c == ';' || c == '.')
                .unwrap_or(result[start..].len());
            
            let city = &result[start..end];
            let generalized = generalize_location(city);
            result.replace_range(start..end, &generalized);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_identifier() {
        let id1 = "test_user";
        let id2 = "test_user";
        let id3 = "other_user";
        
        let hash1 = hash_identifier(id1);
        let hash2 = hash_identifier(id2);
        let hash3 = hash_identifier(id3);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        assert_ne!(hash1, hash3);
        
        // Hash should start with "anon_"
        assert!(hash1.starts_with("anon_"));
    }
    
    #[test]
    fn test_mask_ip_address() {
        let ipv4 = "192.168.1.1";
        let ipv6 = "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        
        let masked_ipv4 = mask_ip_address(ipv4);
        let masked_ipv6 = mask_ip_address(ipv6);
        
        assert_eq!(masked_ipv4, "192.168.1.xxx");
        assert!(masked_ipv6.contains("xxxx"));
    }
    
    #[test]
    fn test_anonymize_ip_addresses_in_string() {
        let s = "The server IP is 192.168.1.1 and the gateway is 10.0.0.1";
        let result = anonymize_ip_addresses_in_string(s);
        
        assert!(result.contains("192.168.1.xxx"));
        assert!(result.contains("10.0.0.xxx"));
    }
    
    #[test]
    fn test_anonymize_user_identifiers_in_string() {
        let s = "The user is user: john and the email is john@example.com";
        let result = anonymize_user_identifiers_in_string(s);
        
        assert!(!result.contains("john"));
        assert!(!result.contains("john@example.com"));
        assert!(result.contains("anon_"));
    }
    
    #[test]
    fn test_anonymize_device_identifiers_in_string() {
        let s = "The device is device: abc123 and the MAC is 00:11:22:33:44:55";
        let result = anonymize_device_identifiers_in_string(s);
        
        assert!(!result.contains("abc123"));
        assert!(!result.contains("00:11:22:33:44:55"));
        assert!(result.contains("anon_"));
    }
    
    #[test]
    fn test_anonymize_location_data_in_string() {
        let s = "The coordinates are 37.7749,-122.4194 and latitude: 37.7749, longitude: -122.4194";
        let result = anonymize_location_data_in_string(s);
        
        assert!(!result.contains("37.7749"));
        assert!(!result.contains("-122.4194"));
        assert!(result.contains("37.7"));
        assert!(result.contains("-122.4"));
    }
    
    #[test]
    fn test_anonymize_data_point() {
        let mut settings = PrivacySettings::default();
        settings.include_ip_addresses = true;
        settings.include_location_data = true;
        settings.include_user_identifiers = true;
        settings.include_device_identifiers = true;
        
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
        
        let result = anonymize_data_point(data_point, &settings);
        
        // Check that values are anonymized but still present
        assert!(result.metadata.contains_key("ip"));
        assert!(result.metadata.contains_key("user"));
        assert!(result.metadata.contains_key("device_id"));
        assert!(result.metadata.contains_key("location"));
        
        assert!(result.metadata.get("ip").unwrap().contains("xxx"));
        assert!(result.metadata.get("user").unwrap().contains("anon_"));
        assert!(result.metadata.get("device_id").unwrap().contains("anon_"));
        assert!(result.metadata.get("location").unwrap().contains("Region"));
        
        if let TelemetryValue::String(s) = result.value {
            assert!(s.contains("xxx"));
        } else {
            panic!("Expected String value");
        }
    }
}
