//! Update checker module for the VR headset.
//!
//! This module provides functionality for checking for available updates
//! from the update server.

use std::time::{Duration, SystemTime};
use anyhow::{Result, Context, anyhow, bail};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use semver::Version;
use log::{info, warn, error, debug};

use super::package::UpdatePackageInfo;

/// Check for available updates from the update server.
///
/// # Arguments
///
/// * `server_url` - URL of the update server
///
/// # Returns
///
/// A list of available updates, sorted by version (newest first).
pub async fn check_for_updates(server_url: &str) -> Result<Vec<UpdatePackageInfo>> {
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    // Build the request URL
    let url = format!("{}/api/updates", server_url);
    
    // Send the request
    debug!("Checking for updates at {}", url);
    let response = client.get(&url)
        .header("User-Agent", "VR-Headset-Updater/1.0")
        .send()
        .await
        .context("Failed to send update check request")?;
    
    // Check the response status
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        bail!("Update server returned error: {} - {}", status, text);
    }
    
    // Parse the response
    let updates: Vec<UpdatePackageInfo> = response.json()
        .await
        .context("Failed to parse update server response")?;
    
    // Sort updates by version (newest first)
    let mut sorted_updates = updates;
    sorted_updates.sort_by(|a, b| b.version.cmp(&a.version));
    
    debug!("Found {} available updates", sorted_updates.len());
    Ok(sorted_updates)
}

/// Check if an update is compatible with the current system.
///
/// # Arguments
///
/// * `update` - Information about the update
/// * `current_version` - Current system version
///
/// # Returns
///
/// `true` if the update is compatible, `false` otherwise.
pub fn is_update_compatible(update: &UpdatePackageInfo, current_version: &Version) -> bool {
    // Check if the update requires a minimum system version
    if let Some(min_version) = &update.min_system_version {
        if current_version < min_version {
            debug!("Update {} requires minimum version {}, but current version is {}",
                update.version, min_version, current_version);
            return false;
        }
    }
    
    // Check if the update version is newer than the current version
    if &update.version <= current_version {
        debug!("Update version {} is not newer than current version {}",
            update.version, current_version);
        return false;
    }
    
    true
}

/// Filter updates to only include compatible ones.
///
/// # Arguments
///
/// * `updates` - List of available updates
/// * `current_version` - Current system version
///
/// # Returns
///
/// A list of compatible updates, sorted by version (newest first).
pub fn filter_compatible_updates(
    updates: &[UpdatePackageInfo],
    current_version: &Version,
) -> Vec<UpdatePackageInfo> {
    updates.iter()
        .filter(|update| is_update_compatible(update, current_version))
        .cloned()
        .collect()
}

/// Get the latest compatible update.
///
/// # Arguments
///
/// * `updates` - List of available updates
/// * `current_version` - Current system version
///
/// # Returns
///
/// The latest compatible update, or `None` if no compatible updates are available.
pub fn get_latest_compatible_update(
    updates: &[UpdatePackageInfo],
    current_version: &Version,
) -> Option<UpdatePackageInfo> {
    let compatible_updates = filter_compatible_updates(updates, current_version);
    compatible_updates.first().cloned()
}

/// Check if a critical security update is available.
///
/// # Arguments
///
/// * `updates` - List of available updates
/// * `current_version` - Current system version
///
/// # Returns
///
/// `true` if a critical security update is available, `false` otherwise.
pub fn has_critical_security_update(
    updates: &[UpdatePackageInfo],
    current_version: &Version,
) -> bool {
    let compatible_updates = filter_compatible_updates(updates, current_version);
    compatible_updates.iter().any(|update| update.is_security_update)
}
