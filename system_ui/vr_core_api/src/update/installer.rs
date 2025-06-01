//! Installation module for the VR headset update system.
//!
//! This module provides functionality for installing update packages,
//! including extraction, verification, and applying updates to the system.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::process::Command;
use anyhow::{Result, Context, anyhow, bail};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use super::package::{UpdatePackageMetadata, InstalledUpdateInfo};
use super::UpdateStatus;

/// Installation manifest for tracking installed files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationManifest {
    /// Version of the update.
    pub version: semver::Version,
    
    /// Installation date.
    pub installation_date: chrono::DateTime<chrono::Utc>,
    
    /// List of installed files with their hashes.
    pub files: Vec<InstalledFile>,
    
    /// List of configuration files that were modified.
    pub modified_configs: Vec<String>,
    
    /// List of services that need to be restarted.
    pub services_to_restart: Vec<String>,
    
    /// Whether a system restart is required.
    pub requires_restart: bool,
}

/// Information about an installed file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledFile {
    /// Path to the file, relative to the installation directory.
    pub path: String,
    
    /// SHA-256 hash of the file.
    pub hash: String,
    
    /// Size of the file in bytes.
    pub size: u64,
    
    /// Whether the file is a configuration file.
    pub is_config: bool,
    
    /// Whether the file is executable.
    pub is_executable: bool,
}

/// Install an update package.
///
/// # Arguments
///
/// * `package_path` - Path to the update package
/// * `install_dir` - Directory where the update will be installed
/// * `backup_dir` - Directory for storing backups of replaced files
/// * `status_tx` - Channel for sending status updates
///
/// # Returns
///
/// Information about the installed update.
pub async fn install_update(
    package_path: &Path,
    install_dir: &Path,
    backup_dir: &Path,
    status_tx: mpsc::Sender<UpdateStatus>,
) -> Result<InstalledUpdateInfo> {
    debug!("Installing update package: {}", package_path.display());
    
    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let extract_dir = temp_dir.path();
    
    // Update status to preparing
    let metadata = super::package::get_package_info(package_path).await?;
    let status = UpdateStatus::Installing {
        version: metadata.version.clone(),
        progress_percent: 0.0,
        stage: "Preparing".to_string(),
    };
    let _ = status_tx.send(status.clone()).await;
    
    // Extract the package
    debug!("Extracting update package to: {}", extract_dir.display());
    let metadata = super::package::extract_package(package_path, extract_dir).await?;
    
    // Update status to verifying
    let status = UpdateStatus::Installing {
        version: metadata.version.clone(),
        progress_percent: 10.0,
        stage: "Verifying".to_string(),
    };
    let _ = status_tx.send(status.clone()).await;
    
    // Create the installation manifest
    let mut manifest = InstallationManifest {
        version: metadata.version.clone(),
        installation_date: chrono::Utc::now(),
        files: Vec::new(),
        modified_configs: Vec::new(),
        services_to_restart: Vec::new(),
        requires_restart: metadata.requires_restart,
    };
    
    // Create the backup directory if it doesn't exist
    let backup_path = backup_dir.join(format!("backup-{}", metadata.version));
    fs::create_dir_all(&backup_path).context("Failed to create backup directory")?;
    
    // Create the installation directory if it doesn't exist
    fs::create_dir_all(install_dir).context("Failed to create installation directory")?;
    
    // Scan the extracted files
    let mut total_files = 0;
    let mut processed_files = 0;
    
    for entry in walkdir::WalkDir::new(extract_dir) {
        let entry = entry.context("Failed to read directory entry")?;
        if entry.file_type().is_file() {
            total_files += 1;
        }
    }
    
    // Install the files
    for entry in walkdir::WalkDir::new(extract_dir) {
        let entry = entry.context("Failed to read directory entry")?;
        let source_path = entry.path();
        
        if source_path.is_file() {
            // Get the relative path
            let relative_path = source_path.strip_prefix(extract_dir)
                .context("Failed to compute relative path")?;
            
            // Determine the destination path
            let dest_path = install_dir.join(relative_path);
            
            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).context("Failed to create directory")?;
            }
            
            // Check if the file already exists
            if dest_path.exists() {
                // Create a backup of the existing file
                let backup_file_path = backup_path.join(relative_path);
                
                // Create parent directories for the backup if needed
                if let Some(parent) = backup_file_path.parent() {
                    fs::create_dir_all(parent).context("Failed to create backup directory")?;
                }
                
                // Copy the existing file to the backup directory
                fs::copy(&dest_path, &backup_file_path)
                    .context(format!("Failed to backup file: {}", relative_path.display()))?;
            }
            
            // Copy the new file to the destination
            fs::copy(source_path, &dest_path)
                .context(format!("Failed to install file: {}", relative_path.display()))?;
            
            // Set executable permissions if needed
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                
                let metadata = fs::metadata(source_path)
                    .context(format!("Failed to get metadata for: {}", source_path.display()))?;
                let permissions = metadata.permissions();
                
                if permissions.mode() & 0o111 != 0 {
                    let mut dest_permissions = fs::metadata(&dest_path)
                        .context(format!("Failed to get metadata for: {}", dest_path.display()))?
                        .permissions();
                    
                    dest_permissions.set_mode(permissions.mode());
                    fs::set_permissions(&dest_path, dest_permissions)
                        .context(format!("Failed to set permissions for: {}", dest_path.display()))?;
                }
            }
            
            // Calculate the file hash
            let hash = super::package::calculate_file_hash(&dest_path).await?;
            
            // Get file size
            let size = fs::metadata(&dest_path)
                .context(format!("Failed to get metadata for: {}", dest_path.display()))?
                .len();
            
            // Determine if this is a configuration file
            let is_config = relative_path.to_string_lossy().contains("config") ||
                            relative_path.extension().map_or(false, |ext| ext == "conf" || ext == "cfg" || ext == "toml" || ext == "json");
            
            // Determine if this is an executable file
            let is_executable = relative_path.extension().map_or(false, |ext| ext == "exe" || ext == "sh" || ext == "bin") ||
                               #[cfg(unix)]
                               {
                                   use std::os::unix::fs::PermissionsExt;
                                   fs::metadata(&dest_path)
                                       .map(|m| m.permissions().mode() & 0o111 != 0)
                                       .unwrap_or(false)
                               }
                               #[cfg(not(unix))]
                               { false };
            
            // Add the file to the manifest
            let installed_file = InstalledFile {
                path: relative_path.to_string_lossy().to_string(),
                hash,
                size,
                is_config,
                is_executable,
            };
            
            manifest.files.push(installed_file);
            
            // If this is a configuration file, add it to the modified configs list
            if is_config {
                manifest.modified_configs.push(relative_path.to_string_lossy().to_string());
            }
            
            // Update progress
            processed_files += 1;
            let progress_percent = 10.0 + (processed_files as f32 / total_files as f32) * 80.0;
            
            let status = UpdateStatus::Installing {
                version: metadata.version.clone(),
                progress_percent,
                stage: format!("Installing files ({}/{})", processed_files, total_files),
            };
            let _ = status_tx.send(status.clone()).await;
        }
    }
    
    // Update status to finalizing
    let status = UpdateStatus::Installing {
        version: metadata.version.clone(),
        progress_percent: 90.0,
        stage: "Finalizing".to_string(),
    };
    let _ = status_tx.send(status.clone()).await;
    
    // Save the installation manifest
    let manifest_path = install_dir.join("manifest.json");
    let manifest_file = File::create(&manifest_path)
        .context("Failed to create manifest file")?;
    serde_json::to_writer_pretty(manifest_file, &manifest)
        .context("Failed to write manifest")?;
    
    // Create the installed update info
    let installed_update = InstalledUpdateInfo {
        version: metadata.version.clone(),
        installation_date: chrono::Utc::now(),
        installation_successful: true,
        error_message: None,
    };
    
    // Save the installed update info
    let info_path = install_dir.join("installed_update.json");
    let info_file = File::create(&info_path)
        .context("Failed to create installed update info file")?;
    serde_json::to_writer_pretty(info_file, &installed_update)
        .context("Failed to write installed update info")?;
    
    // Update status to completed
    let status = UpdateStatus::InstallationComplete {
        version: metadata.version.clone(),
        requires_restart: metadata.requires_restart,
    };
    let _ = status_tx.send(status.clone()).await;
    
    debug!("Update installation completed: {}", metadata.version);
    Ok(installed_update)
}

/// Rollback an update.
///
/// # Arguments
///
/// * `version` - Version to rollback from
/// * `install_dir` - Directory where the update is installed
/// * `backup_dir` - Directory where backups are stored
/// * `status_tx` - Channel for sending status updates
///
/// # Returns
///
/// `Ok(())` if the rollback was successful.
pub async fn rollback_update(
    version: &semver::Version,
    install_dir: &Path,
    backup_dir: &Path,
    status_tx: mpsc::Sender<UpdateStatus>,
) -> Result<()> {
    debug!("Rolling back update: {}", version);
    
    // Update status to rolling back
    let status = UpdateStatus::RollingBack {
        version: version.clone(),
        progress_percent: 0.0,
    };
    let _ = status_tx.send(status.clone()).await;
    
    // Check if a backup exists
    let backup_path = backup_dir.join(format!("backup-{}", version));
    if !backup_path.exists() {
        bail!("Backup not found for version: {}", version);
    }
    
    // Read the installation manifest
    let manifest_path = install_dir.join("manifest.json");
    let manifest_file = File::open(&manifest_path)
        .context("Failed to open manifest file")?;
    let manifest: InstallationManifest = serde_json::from_reader(manifest_file)
        .context("Failed to parse manifest")?;
    
    // Verify that the manifest version matches
    if manifest.version != *version {
        bail!("Manifest version mismatch: expected {}, got {}", version, manifest.version);
    }
    
    // Count the total files to restore
    let total_files = manifest.files.len();
    let mut processed_files = 0;
    
    // Restore files from the backup
    for file_entry in &manifest.files {
        let file_path = PathBuf::from(&file_entry.path);
        let installed_path = install_dir.join(&file_path);
        let backup_file_path = backup_path.join(&file_path);
        
        // If the backup file exists, restore it
        if backup_file_path.exists() {
            // Create parent directories if needed
            if let Some(parent) = installed_path.parent() {
                fs::create_dir_all(parent).context("Failed to create directory")?;
            }
            
            // Copy the backup file to the installation directory
            fs::copy(&backup_file_path, &installed_path)
                .context(format!("Failed to restore file: {}", file_path.display()))?;
            
            // Set executable permissions if needed
            #[cfg(unix)]
            if file_entry.is_executable {
                use std::os::unix::fs::PermissionsExt;
                
                let mut permissions = fs::metadata(&installed_path)
                    .context(format!("Failed to get metadata for: {}", installed_path.display()))?
                    .permissions();
                
                permissions.set_mode(permissions.mode() | 0o111);
                fs::set_permissions(&installed_path, permissions)
                    .context(format!("Failed to set permissions for: {}", installed_path.display()))?;
            }
        } else {
            // If the file didn't exist before the update, remove it
            if installed_path.exists() {
                fs::remove_file(&installed_path)
                    .context(format!("Failed to remove file: {}", file_path.display()))?;
            }
        }
        
        // Update progress
        processed_files += 1;
        let progress_percent = (processed_files as f32 / total_files as f32) * 100.0;
        
        let status = UpdateStatus::RollingBack {
            version: version.clone(),
            progress_percent,
        };
        let _ = status_tx.send(status.clone()).await;
    }
    
    // Update status to rollback complete
    let status = UpdateStatus::RollbackComplete {
        version: version.clone(),
    };
    let _ = status_tx.send(status.clone()).await;
    
    debug!("Update rollback completed: {}", version);
    Ok(())
}

/// Apply post-installation actions.
///
/// # Arguments
///
/// * `install_dir` - Directory where the update is installed
///
/// # Returns
///
/// `Ok(())` if all actions were applied successfully.
pub async fn apply_post_install_actions(install_dir: &Path) -> Result<()> {
    debug!("Applying post-installation actions");
    
    // Read the installation manifest
    let manifest_path = install_dir.join("manifest.json");
    let manifest_file = File::open(&manifest_path)
        .context("Failed to open manifest file")?;
    let manifest: InstallationManifest = serde_json::from_reader(manifest_file)
        .context("Failed to parse manifest")?;
    
    // Restart services if needed
    for service in &manifest.services_to_restart {
        debug!("Restarting service: {}", service);
        
        // Use systemctl to restart the service
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("systemctl")
                .arg("restart")
                .arg(service)
                .status()
                .context(format!("Failed to restart service: {}", service))?;
            
            if !status.success() {
                warn!("Failed to restart service: {}", service);
            }
        }
    }
    
    // Check if a restart is required
    if manifest.requires_restart {
        info!("System restart required to complete the update");
    }
    
    debug!("Post-installation actions completed");
    Ok(())
}
