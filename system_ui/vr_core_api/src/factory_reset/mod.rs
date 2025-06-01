//! Factory reset module for the VR headset.
//!
//! This module provides functionality for performing factory resets,
//! including data backup, system restoration, and configuration reset.

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;
use anyhow::{Result, Context, anyhow, bail};
use chrono::{DateTime, Utc, Local};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Factory reset settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryResetSettings {
    /// Whether to create a backup before reset.
    pub create_backup: bool,
    
    /// Directory to store backups.
    pub backup_dir: PathBuf,
    
    /// Whether to reset user data.
    pub reset_user_data: bool,
    
    /// Whether to reset system settings.
    pub reset_system_settings: bool,
    
    /// Whether to reset network settings.
    pub reset_network_settings: bool,
    
    /// Whether to reset installed applications.
    pub reset_applications: bool,
    
    /// Whether to perform a full system wipe.
    pub full_system_wipe: bool,
}

impl Default for FactoryResetSettings {
    fn default() -> Self {
        Self {
            create_backup: true,
            backup_dir: PathBuf::from("/var/backups/vr-system"),
            reset_user_data: true,
            reset_system_settings: true,
            reset_network_settings: true,
            reset_applications: true,
            full_system_wipe: false,
        }
    }
}

/// Factory reset manager.
#[derive(Debug)]
pub struct FactoryResetManager {
    /// Factory reset settings.
    settings: FactoryResetSettings,
    
    /// System data directory.
    system_dir: PathBuf,
    
    /// User data directory.
    user_data_dir: PathBuf,
    
    /// Applications directory.
    applications_dir: PathBuf,
    
    /// Configuration directory.
    config_dir: PathBuf,
}

impl FactoryResetManager {
    /// Create a new factory reset manager.
    ///
    /// # Arguments
    ///
    /// * `settings` - Factory reset settings
    ///
    /// # Returns
    ///
    /// A new factory reset manager.
    pub fn new(settings: FactoryResetSettings) -> Self {
        Self {
            settings,
            system_dir: PathBuf::from("/var/lib/vr-system"),
            user_data_dir: PathBuf::from("/home/vr-user"),
            applications_dir: PathBuf::from("/opt/vr-apps"),
            config_dir: PathBuf::from("/etc/vr-system"),
        }
    }
    
    /// Perform a factory reset.
    ///
    /// This function performs a factory reset according to the configured settings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    pub fn perform_reset(&self) -> Result<()> {
        // Create a backup if requested
        if self.settings.create_backup {
            self.create_backup()?;
        }
        
        // Perform the reset
        if self.settings.full_system_wipe {
            self.perform_full_system_wipe()?;
        } else {
            // Perform selective reset
            if self.settings.reset_user_data {
                self.reset_user_data()?;
            }
            
            if self.settings.reset_system_settings {
                self.reset_system_settings()?;
            }
            
            if self.settings.reset_network_settings {
                self.reset_network_settings()?;
            }
            
            if self.settings.reset_applications {
                self.reset_applications()?;
            }
        }
        
        // Restore default configuration
        self.restore_default_configuration()?;
        
        // Create reset marker
        self.create_reset_marker()?;
        
        Ok(())
    }
    
    /// Create a backup before reset.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the backup was successful.
    fn create_backup(&self) -> Result<()> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&self.settings.backup_dir)
            .context("Failed to create backup directory")?;
        
        // Generate backup ID
        let backup_id = Uuid::new_v4();
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("factory_reset_backup_{}_{}", timestamp, backup_id);
        
        // Create backup directory
        let backup_dir = self.settings.backup_dir.join(&backup_name);
        fs::create_dir(&backup_dir)
            .context("Failed to create backup directory")?;
        
        // Backup user data if requested
        if self.settings.reset_user_data {
            self.backup_directory(&self.user_data_dir, &backup_dir.join("user_data"))?;
        }
        
        // Backup system settings if requested
        if self.settings.reset_system_settings {
            self.backup_directory(&self.config_dir, &backup_dir.join("system_settings"))?;
        }
        
        // Backup network settings if requested
        if self.settings.reset_network_settings {
            self.backup_network_settings(&backup_dir.join("network_settings"))?;
        }
        
        // Backup application data if requested
        if self.settings.reset_applications {
            self.backup_directory(&self.applications_dir, &backup_dir.join("applications"))?;
        }
        
        // Create backup manifest
        self.create_backup_manifest(&backup_dir, &backup_name)?;
        
        Ok(())
    }
    
    /// Backup a directory.
    ///
    /// # Arguments
    ///
    /// * `source_dir` - Directory to backup
    /// * `target_dir` - Directory to store the backup
    ///
    /// # Returns
    ///
    /// `Ok(())` if the backup was successful.
    fn backup_directory(&self, source_dir: &Path, target_dir: &Path) -> Result<()> {
        // Create target directory
        fs::create_dir_all(target_dir)
            .context("Failed to create target directory")?;
        
        // Use tar to create a compressed archive
        let archive_path = target_dir.join("backup.tar.gz");
        
        let output = Command::new("tar")
            .arg("-czf")
            .arg(&archive_path)
            .arg("-C")
            .arg(source_dir)
            .arg(".")
            .output()
            .context("Failed to execute tar command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create backup archive: {}", error);
        }
        
        Ok(())
    }
    
    /// Backup network settings.
    ///
    /// # Arguments
    ///
    /// * `target_dir` - Directory to store the backup
    ///
    /// # Returns
    ///
    /// `Ok(())` if the backup was successful.
    fn backup_network_settings(&self, target_dir: &Path) -> Result<()> {
        // Create target directory
        fs::create_dir_all(target_dir)
            .context("Failed to create target directory")?;
        
        // Backup network configuration files
        let network_config_dir = PathBuf::from("/etc/network");
        if network_config_dir.exists() {
            self.backup_directory(&network_config_dir, &target_dir.join("network"))?;
        }
        
        // Backup NetworkManager configuration files
        let nm_config_dir = PathBuf::from("/etc/NetworkManager");
        if nm_config_dir.exists() {
            self.backup_directory(&nm_config_dir, &target_dir.join("NetworkManager"))?;
        }
        
        // Backup wpa_supplicant configuration files
        let wpa_config_dir = PathBuf::from("/etc/wpa_supplicant");
        if wpa_config_dir.exists() {
            self.backup_directory(&wpa_config_dir, &target_dir.join("wpa_supplicant"))?;
        }
        
        Ok(())
    }
    
    /// Create a backup manifest.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory containing the backup
    /// * `backup_name` - Name of the backup
    ///
    /// # Returns
    ///
    /// `Ok(())` if the manifest was created successfully.
    fn create_backup_manifest(&self, backup_dir: &Path, backup_name: &str) -> Result<()> {
        // Create manifest
        let manifest = BackupManifest {
            backup_id: Uuid::new_v4(),
            backup_name: backup_name.to_string(),
            timestamp: Utc::now(),
            settings: self.settings.clone(),
            system_version: self.get_system_version()?,
        };
        
        // Write manifest to file
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .context("Failed to serialize backup manifest")?;
        
        fs::write(&manifest_path, manifest_json)
            .context("Failed to write backup manifest")?;
        
        Ok(())
    }
    
    /// Get the current system version.
    ///
    /// # Returns
    ///
    /// The current system version.
    fn get_system_version(&self) -> Result<String> {
        // Read version file
        let version_path = self.system_dir.join("version");
        if version_path.exists() {
            let version = fs::read_to_string(&version_path)
                .context("Failed to read version file")?;
            Ok(version.trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
    
    /// Perform a full system wipe.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the wipe was successful.
    fn perform_full_system_wipe(&self) -> Result<()> {
        // This is a dangerous operation, so we'll just simulate it
        // In a real implementation, this would use system tools to wipe and reinstall
        
        // Wipe user data
        self.reset_user_data()?;
        
        // Wipe system settings
        self.reset_system_settings()?;
        
        // Wipe network settings
        self.reset_network_settings()?;
        
        // Wipe applications
        self.reset_applications()?;
        
        // Wipe system data
        self.reset_system_data()?;
        
        Ok(())
    }
    
    /// Reset user data.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    fn reset_user_data(&self) -> Result<()> {
        // In a real implementation, this would delete user data files
        // For simulation, we'll just log the action
        println!("Resetting user data in {}", self.user_data_dir.display());
        
        // Create a marker file
        let marker_path = self.user_data_dir.join(".factory_reset");
        fs::write(&marker_path, "User data reset")
            .context("Failed to create user data reset marker")?;
        
        Ok(())
    }
    
    /// Reset system settings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    fn reset_system_settings(&self) -> Result<()> {
        // In a real implementation, this would reset system configuration files
        // For simulation, we'll just log the action
        println!("Resetting system settings in {}", self.config_dir.display());
        
        // Create a marker file
        let marker_path = self.config_dir.join(".factory_reset");
        fs::write(&marker_path, "System settings reset")
            .context("Failed to create system settings reset marker")?;
        
        Ok(())
    }
    
    /// Reset network settings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    fn reset_network_settings(&self) -> Result<()> {
        // In a real implementation, this would reset network configuration files
        // For simulation, we'll just log the action
        println!("Resetting network settings");
        
        // Create a marker file
        let marker_path = PathBuf::from("/etc/network/.factory_reset");
        fs::write(&marker_path, "Network settings reset")
            .context("Failed to create network settings reset marker")?;
        
        Ok(())
    }
    
    /// Reset applications.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    fn reset_applications(&self) -> Result<()> {
        // In a real implementation, this would uninstall or reset applications
        // For simulation, we'll just log the action
        println!("Resetting applications in {}", self.applications_dir.display());
        
        // Create a marker file
        let marker_path = self.applications_dir.join(".factory_reset");
        fs::write(&marker_path, "Applications reset")
            .context("Failed to create applications reset marker")?;
        
        Ok(())
    }
    
    /// Reset system data.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the reset was successful.
    fn reset_system_data(&self) -> Result<()> {
        // In a real implementation, this would reset system data files
        // For simulation, we'll just log the action
        println!("Resetting system data in {}", self.system_dir.display());
        
        // Create a marker file
        let marker_path = self.system_dir.join(".factory_reset");
        fs::write(&marker_path, "System data reset")
            .context("Failed to create system data reset marker")?;
        
        Ok(())
    }
    
    /// Restore default configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the restoration was successful.
    fn restore_default_configuration(&self) -> Result<()> {
        // In a real implementation, this would copy default configuration files
        // For simulation, we'll just log the action
        println!("Restoring default configuration");
        
        // Create a marker file
        let marker_path = self.config_dir.join(".default_config_restored");
        fs::write(&marker_path, "Default configuration restored")
            .context("Failed to create default configuration marker")?;
        
        Ok(())
    }
    
    /// Create a reset marker.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the marker was created successfully.
    fn create_reset_marker(&self) -> Result<()> {
        // Create a marker file with timestamp
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let marker_path = self.system_dir.join(".factory_reset_complete");
        fs::write(&marker_path, format!("Factory reset completed at {}", timestamp))
            .context("Failed to create factory reset marker")?;
        
        Ok(())
    }
    
    /// Restore from a backup.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - ID of the backup to restore
    ///
    /// # Returns
    ///
    /// `Ok(())` if the restoration was successful.
    pub fn restore_from_backup(&self, backup_id: &str) -> Result<()> {
        // Find the backup
        let backup_dir = self.find_backup(backup_id)?;
        
        // Read the manifest
        let manifest = self.read_backup_manifest(&backup_dir)?;
        
        // Restore user data if it was backed up
        if manifest.settings.reset_user_data {
            self.restore_directory(&backup_dir.join("user_data"), &self.user_data_dir)?;
        }
        
        // Restore system settings if they were backed up
        if manifest.settings.reset_system_settings {
            self.restore_directory(&backup_dir.join("system_settings"), &self.config_dir)?;
        }
        
        // Restore network settings if they were backed up
        if manifest.settings.reset_network_settings {
            self.restore_network_settings(&backup_dir.join("network_settings"))?;
        }
        
        // Restore application data if it was backed up
        if manifest.settings.reset_applications {
            self.restore_directory(&backup_dir.join("applications"), &self.applications_dir)?;
        }
        
        // Create restoration marker
        self.create_restoration_marker(&manifest)?;
        
        Ok(())
    }
    
    /// Find a backup by ID.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - ID of the backup to find
    ///
    /// # Returns
    ///
    /// The path to the backup directory.
    fn find_backup(&self, backup_id: &str) -> Result<PathBuf> {
        // List all backup directories
        let entries = fs::read_dir(&self.settings.backup_dir)
            .context("Failed to read backup directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() && path.file_name().unwrap().to_string_lossy().contains(backup_id) {
                return Ok(path);
            }
        }
        
        bail!("Backup with ID {} not found", backup_id)
    }
    
    /// Read a backup manifest.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory containing the backup
    ///
    /// # Returns
    ///
    /// The backup manifest.
    fn read_backup_manifest(&self, backup_dir: &Path) -> Result<BackupManifest> {
        // Read manifest file
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_json = fs::read_to_string(&manifest_path)
            .context("Failed to read backup manifest")?;
        
        // Parse manifest
        let manifest: BackupManifest = serde_json::from_str(&manifest_json)
            .context("Failed to parse backup manifest")?;
        
        Ok(manifest)
    }
    
    /// Restore a directory from backup.
    ///
    /// # Arguments
    ///
    /// * `source_dir` - Directory containing the backup
    /// * `target_dir` - Directory to restore to
    ///
    /// # Returns
    ///
    /// `Ok(())` if the restoration was successful.
    fn restore_directory(&self, source_dir: &Path, target_dir: &Path) -> Result<()> {
        // Check if the backup exists
        let archive_path = source_dir.join("backup.tar.gz");
        if !archive_path.exists() {
            bail!("Backup archive not found: {}", archive_path.display());
        }
        
        // Create target directory if it doesn't exist
        fs::create_dir_all(target_dir)
            .context("Failed to create target directory")?;
        
        // Use tar to extract the archive
        let output = Command::new("tar")
            .arg("-xzf")
            .arg(&archive_path)
            .arg("-C")
            .arg(target_dir)
            .output()
            .context("Failed to execute tar command")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to extract backup archive: {}", error);
        }
        
        Ok(())
    }
    
    /// Restore network settings from backup.
    ///
    /// # Arguments
    ///
    /// * `source_dir` - Directory containing the backup
    ///
    /// # Returns
    ///
    /// `Ok(())` if the restoration was successful.
    fn restore_network_settings(&self, source_dir: &Path) -> Result<()> {
        // Restore network configuration files
        let network_backup_dir = source_dir.join("network");
        if network_backup_dir.exists() {
            self.restore_directory(&network_backup_dir, &PathBuf::from("/etc/network"))?;
        }
        
        // Restore NetworkManager configuration files
        let nm_backup_dir = source_dir.join("NetworkManager");
        if nm_backup_dir.exists() {
            self.restore_directory(&nm_backup_dir, &PathBuf::from("/etc/NetworkManager"))?;
        }
        
        // Restore wpa_supplicant configuration files
        let wpa_backup_dir = source_dir.join("wpa_supplicant");
        if wpa_backup_dir.exists() {
            self.restore_directory(&wpa_backup_dir, &PathBuf::from("/etc/wpa_supplicant"))?;
        }
        
        Ok(())
    }
    
    /// Create a restoration marker.
    ///
    /// # Arguments
    ///
    /// * `manifest` - Backup manifest
    ///
    /// # Returns
    ///
    /// `Ok(())` if the marker was created successfully.
    fn create_restoration_marker(&self, manifest: &BackupManifest) -> Result<()> {
        // Create a marker file with timestamp
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let marker_path = self.system_dir.join(".backup_restored");
        let marker_content = format!(
            "Backup restored at {}\nBackup ID: {}\nBackup Name: {}\nBackup Timestamp: {}\nSystem Version: {}",
            timestamp,
            manifest.backup_id,
            manifest.backup_name,
            manifest.timestamp,
            manifest.system_version
        );
        
        fs::write(&marker_path, marker_content)
            .context("Failed to create backup restoration marker")?;
        
        Ok(())
    }
    
    /// List available backups.
    ///
    /// # Returns
    ///
    /// A list of available backups.
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        
        // Check if backup directory exists
        if !self.settings.backup_dir.exists() {
            return Ok(backups);
        }
        
        // List all backup directories
        let entries = fs::read_dir(&self.settings.backup_dir)
            .context("Failed to read backup directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Read manifest
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    let manifest_json = fs::read_to_string(&manifest_path)
                        .context("Failed to read backup manifest")?;
                    
                    let manifest: BackupManifest = serde_json::from_str(&manifest_json)
                        .context("Failed to parse backup manifest")?;
                    
                    backups.push(BackupInfo {
                        id: manifest.backup_id.to_string(),
                        name: manifest.backup_name,
                        timestamp: manifest.timestamp,
                        system_version: manifest.system_version,
                    });
                }
            }
        }
        
        // Sort backups by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
    }
}

/// Backup manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    /// Unique identifier for this backup.
    backup_id: Uuid,
    
    /// Name of this backup.
    backup_name: String,
    
    /// Timestamp when this backup was created.
    timestamp: DateTime<Utc>,
    
    /// Factory reset settings used for this backup.
    settings: FactoryResetSettings,
    
    /// System version at the time of backup.
    system_version: String,
}

/// Backup information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Unique identifier for this backup.
    pub id: String,
    
    /// Name of this backup.
    pub name: String,
    
    /// Timestamp when this backup was created.
    pub timestamp: DateTime<Utc>,
    
    /// System version at the time of backup.
    pub system_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_factory_reset_manager_creation() {
        let settings = FactoryResetSettings::default();
        let manager = FactoryResetManager::new(settings);
        
        assert_eq!(manager.system_dir, PathBuf::from("/var/lib/vr-system"));
        assert_eq!(manager.user_data_dir, PathBuf::from("/home/vr-user"));
        assert_eq!(manager.applications_dir, PathBuf::from("/opt/vr-apps"));
        assert_eq!(manager.config_dir, PathBuf::from("/etc/vr-system"));
    }
    
    #[test]
    fn test_backup_manifest_serialization() {
        let manifest = BackupManifest {
            backup_id: Uuid::new_v4(),
            backup_name: "test_backup".to_string(),
            timestamp: Utc::now(),
            settings: FactoryResetSettings::default(),
            system_version: "1.0.0".to_string(),
        };
        
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: BackupManifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(manifest.backup_id, deserialized.backup_id);
        assert_eq!(manifest.backup_name, deserialized.backup_name);
        assert_eq!(manifest.system_version, deserialized.system_version);
    }
}
