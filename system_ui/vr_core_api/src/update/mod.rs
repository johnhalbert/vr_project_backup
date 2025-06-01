//! Update system module for the VR headset.
//!
//! This module provides functionality for managing software updates, including
//! package format, update checking, download management, verification,
//! installation, and rollback capabilities.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context, anyhow, bail};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time;
use ring::signature;
use sha2::{Sha256, Digest};
use semver::Version;
use chrono::{DateTime, Utc};
use log::{info, warn, error, debug};

pub mod package;
pub mod checker;
pub mod downloader;
pub mod verifier;
pub mod installer;
pub mod rollback;
pub mod delta;
pub mod dependency;

/// Update system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// URL of the update server.
    pub server_url: String,
    
    /// How often to check for updates (in hours).
    pub check_interval_hours: u32,
    
    /// Whether to automatically download updates.
    pub auto_download: bool,
    
    /// Whether to automatically install updates.
    pub auto_install: bool,
    
    /// Maximum bandwidth to use for downloads (in KB/s, 0 for unlimited).
    pub max_bandwidth_kbps: u32,
    
    /// Number of update versions to keep for rollback.
    pub rollback_versions_to_keep: u32,
    
    /// Path to store downloaded updates.
    pub download_path: PathBuf,
    
    /// Path to store installed updates.
    pub install_path: PathBuf,
    
    /// Path to store backup data for rollback.
    pub backup_path: PathBuf,
    
    /// Public key for verifying update signatures.
    pub verification_public_key: String,
    
    /// Whether to prefer delta updates when available.
    pub prefer_delta_updates: bool,
    
    /// Whether to enforce dependency checks.
    pub enforce_dependencies: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            server_url: "https://updates.vrheadset.example.com".to_string(),
            check_interval_hours: 24,
            auto_download: true,
            auto_install: false,
            max_bandwidth_kbps: 0,
            rollback_versions_to_keep: 3,
            download_path: PathBuf::from("/var/cache/vr-updates"),
            install_path: PathBuf::from("/opt/vr-system"),
            backup_path: PathBuf::from("/var/lib/vr-backups"),
            verification_public_key: "".to_string(),
            prefer_delta_updates: true,
            enforce_dependencies: true,
        }
    }
}

/// Status of the update system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateStatus {
    /// No updates available.
    NoUpdates,
    
    /// Update check in progress.
    CheckingForUpdates,
    
    /// Update available but not downloaded.
    UpdateAvailable {
        version: Version,
        size_bytes: u64,
        release_notes: String,
        release_date: DateTime<Utc>,
    },
    
    /// Delta update available but not downloaded.
    DeltaUpdateAvailable {
        base_version: Version,
        target_version: Version,
        size_bytes: u64,
        full_size_bytes: u64,
        size_reduction_percent: f32,
        release_notes: String,
        release_date: DateTime<Utc>,
    },
    
    /// Update download in progress.
    Downloading {
        version: Version,
        progress_percent: f32,
        bytes_downloaded: u64,
        total_bytes: u64,
        speed_kbps: f32,
    },
    
    /// Update downloaded and ready to install.
    ReadyToInstall {
        version: Version,
        size_bytes: u64,
        release_notes: String,
    },
    
    /// Update installation in progress.
    Installing {
        version: Version,
        progress_percent: f32,
        stage: String,
    },
    
    /// Update installed successfully.
    InstallationComplete {
        version: Version,
        installation_date: DateTime<Utc>,
        requires_restart: bool,
    },
    
    /// Update installation failed.
    InstallationFailed {
        version: Version,
        error: String,
        can_retry: bool,
    },
    
    /// Rollback in progress.
    RollingBack {
        from_version: Version,
        to_version: Version,
        progress_percent: f32,
    },
    
    /// Rollback completed successfully.
    RollbackComplete {
        from_version: Version,
        to_version: Version,
        rollback_date: DateTime<Utc>,
    },
    
    /// Rollback failed.
    RollbackFailed {
        from_version: Version,
        to_version: Version,
        error: String,
    },
    
    /// Dependency check in progress.
    CheckingDependencies {
        version: Version,
    },
    
    /// Dependencies not satisfied.
    DependenciesNotSatisfied {
        version: Version,
        missing_dependencies: Vec<String>,
        conflicts: Vec<String>,
    },
}

/// Update system manager.
pub struct UpdateManager {
    /// Configuration for the update system.
    config: UpdateConfig,
    
    /// Current status of the update system.
    status: Arc<Mutex<UpdateStatus>>,
    
    /// Channel for sending status updates.
    status_tx: mpsc::Sender<UpdateStatus>,
    
    /// Channel for receiving status updates.
    status_rx: mpsc::Receiver<UpdateStatus>,
    
    /// Last time an update check was performed.
    last_check_time: Arc<Mutex<Option<SystemTime>>>,
    
    /// Available updates that have been discovered.
    available_updates: Arc<Mutex<Vec<package::UpdatePackageInfo>>>,
    
    /// Installed updates history.
    update_history: Arc<Mutex<Vec<package::InstalledUpdateInfo>>>,
}

impl UpdateManager {
    /// Create a new update manager with the given configuration.
    pub async fn new(config: UpdateConfig) -> Result<Self> {
        // Create necessary directories
        fs::create_dir_all(&config.download_path)
            .context("Failed to create download directory")?;
        fs::create_dir_all(&config.backup_path)
            .context("Failed to create backup directory")?;
            
        // Create channels for status updates
        let (status_tx, status_rx) = mpsc::channel(100);
        
        // Initialize with default status
        let status = Arc::new(Mutex::new(UpdateStatus::NoUpdates));
        
        // Load update history if it exists
        let update_history_path = config.install_path.join("update_history.json");
        let update_history = if update_history_path.exists() {
            let file = File::open(&update_history_path)
                .context("Failed to open update history file")?;
            serde_json::from_reader(file)
                .context("Failed to parse update history")?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            config,
            status: Arc::clone(&status),
            status_tx,
            status_rx,
            last_check_time: Arc::new(Mutex::new(None)),
            available_updates: Arc::new(Mutex::new(Vec::new())),
            update_history: Arc::new(Mutex::new(update_history)),
        })
    }
    
    /// Start the update manager background tasks.
    pub async fn start(&self) -> Result<()> {
        // Clone necessary values for the background task
        let config = self.config.clone();
        let status = Arc::clone(&self.status);
        let last_check_time = Arc::clone(&self.last_check_time);
        let available_updates = Arc::clone(&self.available_updates);
        let status_tx = self.status_tx.clone();
        
        // Start background task for periodic update checks
        tokio::spawn(async move {
            loop {
                // Sleep for a while before checking
                time::sleep(Duration::from_secs(60)).await;
                
                // Check if it's time to look for updates
                let should_check = {
                    let last_check = last_check_time.lock().unwrap();
                    match *last_check {
                        Some(time) => {
                            let elapsed = time.elapsed().unwrap_or(Duration::from_secs(0));
                            elapsed > Duration::from_secs(config.check_interval_hours as u64 * 3600)
                        },
                        None => true, // First run, should check
                    }
                };
                
                if should_check {
                    debug!("Performing scheduled update check");
                    
                    // Update status
                    {
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = UpdateStatus::CheckingForUpdates;
                    }
                    let _ = status_tx.send(UpdateStatus::CheckingForUpdates).await;
                    
                    // Get current version
                    let current_version = Self::get_current_version().unwrap_or_else(|_| Version::new(0, 0, 0));
                    
                    // Check for delta updates if enabled
                    let mut delta_update = None;
                    if config.prefer_delta_updates {
                        match delta::check_for_delta_update(&config.server_url, &current_version).await {
                            Ok(Some(delta_info)) => {
                                delta_update = Some(delta_info);
                            },
                            Ok(None) => {
                                debug!("No delta updates available");
                            },
                            Err(e) => {
                                warn!("Failed to check for delta updates: {}", e);
                            }
                        }
                    }
                    
                    // Perform the regular update check
                    match checker::check_for_updates(&config.server_url).await {
                        Ok(updates) => {
                            // Store available updates
                            {
                                let mut updates_guard = available_updates.lock().unwrap();
                                *updates_guard = updates.clone();
                            }
                            
                            // Update last check time
                            {
                                let mut last_check = last_check_time.lock().unwrap();
                                *last_check = Some(SystemTime::now());
                            }
                            
                            // Update status based on results
                            if !updates.is_empty() {
                                let latest = &updates[0]; // Assuming sorted by version
                                
                                // If delta update is available, use it
                                if let Some(delta_info) = delta_update {
                                    if delta_info.target_version == latest.version {
                                        let new_status = UpdateStatus::DeltaUpdateAvailable {
                                            base_version: delta_info.base_version,
                                            target_version: delta_info.target_version,
                                            size_bytes: delta_info.delta_size_bytes,
                                            full_size_bytes: delta_info.full_size_bytes,
                                            size_reduction_percent: delta_info.size_reduction_percent,
                                            release_notes: latest.release_notes.clone(),
                                            release_date: latest.release_date,
                                        };
                                        
                                        {
                                            let mut status_guard = status.lock().unwrap();
                                            *status_guard = new_status.clone();
                                        }
                                        let _ = status_tx.send(new_status).await;
                                        
                                        // Auto-download if enabled
                                        if config.auto_download {
                                            // TODO: Implement delta update download
                                        }
                                    }
                                } else {
                                    let new_status = UpdateStatus::UpdateAvailable {
                                        version: latest.version.clone(),
                                        size_bytes: latest.size_bytes,
                                        release_notes: latest.release_notes.clone(),
                                        release_date: latest.release_date,
                                    };
                                    
                                    {
                                        let mut status_guard = status.lock().unwrap();
                                        *status_guard = new_status.clone();
                                    }
                                    let _ = status_tx.send(new_status).await;
                                    
                                    // Auto-download if enabled
                                    if config.auto_download {
                                        // Start download in a separate task
                                        let config_clone = config.clone();
                                        let status_clone = Arc::clone(&status);
                                        let status_tx_clone = status_tx.clone();
                                        let latest_clone = latest.clone();
                                        
                                        tokio::spawn(async move {
                                            match downloader::download_update(
                                                &latest_clone,
                                                &config_clone.download_path,
                                                config_clone.max_bandwidth_kbps,
                                                status_tx_clone.clone(),
                                            ).await {
                                                Ok(download_path) => {
                                                    // Verify the downloaded package
                                                    match verifier::verify_package(
                                                        &download_path,
                                                        &config_clone.verification_public_key,
                                                    ).await {
                                                        Ok(()) => {
                                                            // Check dependencies if enabled
                                                            if config_clone.enforce_dependencies {
                                                                let status = UpdateStatus::CheckingDependencies {
                                                                    version: latest_clone.version.clone(),
                                                                };
                                                                let _ = status_tx_clone.send(status).await;
                                                                
                                                                // Get package metadata
                                                                match package::get_package_info(&download_path).await {
                                                                    Ok(metadata) => {
                                                                        // Get installed packages
                                                                        match dependency::get_installed_packages(&config_clone.install_path) {
                                                                            Ok(installed_packages) => {
                                                                                // Get system info
                                                                                match dependency::get_system_info() {
                                                                                    Ok(system_info) => {
                                                                                        // Check dependencies
                                                                                        let dep_result = dependency::check_dependencies(
                                                                                            &metadata,
                                                                                            &installed_packages,
                                                                                            &system_info,
                                                                                        );
                                                                                        
                                                                                        if dep_result.satisfied {
                                                                                            let new_status = UpdateStatus::ReadyToInstall {
                                                                                                version: latest_clone.version.clone(),
                                                                                                size_bytes: latest_clone.size_bytes,
                                                                                                release_notes: latest_clone.release_notes.clone(),
                                                                                            };
                                                                                            
                                                                                            {
                                                                                                let mut status_guard = status_clone.lock().unwrap();
                                                                                                *status_guard = new_status.clone();
                                                                                            }
                                                                                            let _ = status_tx_clone.send(new_status).await;
                                                                                            
                                                                                            // Auto-install if enabled
                                                                                            if config_clone.auto_install {
                                                                                                // Start installation
                                                                                                Self::start_installation(
                                                                                                    &download_path,
                                                                                                    &config_clone,
                                                                                                    Arc::clone(&status_clone),
                                                                                                    status_tx_clone.clone(),
                                                                                                    latest_clone.clone(),
                                                                                                ).await;
                                                                                            }
                                                                                        } else {
                                                                                            // Dependencies not satisfied
                                                                                            let missing_deps: Vec<String> = dep_result.missing_dependencies
                                                                                                .iter()
                                                                                                .map(|d| format!("{} {}", d.package_name, d.version_req))
                                                                                                .collect();
                                                                                            
                                                                                            let conflicts: Vec<String> = dep_result.conflicts
                                                                                                .iter()
                                                                                                .map(|c| format!("{} {} ({})", c.package_name, c.version_range, c.reason))
                                                                                                .collect();
                                                                                            
                                                                                            let new_status = UpdateStatus::DependenciesNotSatisfied {
                                                                                                version: latest_clone.version.clone(),
                                                                                                missing_dependencies: missing_deps,
                                                                                                conflicts,
                                                                                            };
                                                                                            
                                                                                            {
                                                                                                let mut status_guard = status_clone.lock().unwrap();
                                                                                                *status_guard = new_status.clone();
                                                                                            }
                                                                                            let _ = status_tx_clone.send(new_status).await;
                                                                                        }
                                                                                    },
                                                                                    Err(e) => {
                                                                                        error!("Failed to get system info: {}", e);
                                                                                        let new_status = UpdateStatus::InstallationFailed {
                                                                                            version: latest_clone.version.clone(),
                                                                                            error: format!("Failed to get system info: {}", e),
                                                                                            can_retry: true,
                                                                                        };
                                                                                        
                                                                                        {
                                                                                            let mut status_guard = status_clone.lock().unwrap();
                                                                                            *status_guard = new_status.clone();
                                                                                        }
                                                                                        let _ = status_tx_clone.send(new_status).await;
                                                                                    }
                                                                                }
                                                                            },
                                                                            Err(e) => {
                                                                                error!("Failed to get installed packages: {}", e);
                                                                                let new_status = UpdateStatus::InstallationFailed {
                                                                                    version: latest_clone.version.clone(),
                                                                                    error: format!("Failed to get installed packages: {}", e),
                                                                                    can_retry: true,
                                                                                };
                                                                                
                                                                                {
                                                                                    let mut status_guard = status_clone.lock().unwrap();
                                                                                    *status_guard = new_status.clone();
                                                                                }
                                                                                let _ = status_tx_clone.send(new_status).await;
                                                                            }
                                                                        }
                                                                    },
                                                                    Err(e) => {
                                                                        error!("Failed to get package info: {}", e);
                                                                        let new_status = UpdateStatus::InstallationFailed {
                                                                            version: latest_clone.version.clone(),
                                                                            error: format!("Failed to get package info: {}", e),
                                                                            can_retry: true,
                                                                        };
                                                                        
                                                                        {
                                                                            let mut status_guard = status_clone.lock().unwrap();
                                                                            *status_guard = new_status.clone();
                                                                        }
                                                                        let _ = status_tx_clone.send(new_status).await;
                                                                    }
                                                                }
                                                            } else {
                                                                // Dependencies not enforced, proceed with installation
                                                                let new_status = UpdateStatus::ReadyToInstall {
                                                                    version: latest_clone.version.clone(),
                                                                    size_bytes: latest_clone.size_bytes,
                                                                    release_notes: latest_clone.release_notes.clone(),
                                                                };
                                                                
                                                                {
                                                                    let mut status_guard = status_clone.lock().unwrap();
                                                                    *status_guard = new_status.clone();
                                                                }
                                                                let _ = status_tx_clone.send(new_status).await;
                                                                
                                                                // Auto-install if enabled
                                                                if config_clone.auto_install {
                                                                    // Start installation
                                                                    Self::start_installation(
                                                                        &download_path,
                                                                        &config_clone,
                                                                        Arc::clone(&status_clone),
                                                                        status_tx_clone.clone(),
                                                                        latest_clone.clone(),
                                                                    ).await;
                                                                }
                                                            }
                                                        },
                                                        Err(e) => {
                                                            error!("Package verification failed: {}", e);
                                                            let new_status = UpdateStatus::InstallationFailed {
                                                                version: latest_clone.version.clone(),
                                                                error: format!("Package verification failed: {}", e),
                                                                can_retry: true,
                                                            };
                                                            
                                                            {
                                                                let mut status_guard = status_clone.lock().unwrap();
                                                                *status_guard = new_status.clone();
                                                            }
                                                            let _ = status_tx_clone.send(new_status).await;
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    error!("Download failed: {}", e);
                                                    let new_status = UpdateStatus::InstallationFailed {
                                                        version: latest_clone.version.clone(),
                                                        error: format!("Download failed: {}", e),
                                                        can_retry: true,
                                                    };
                                                    
                                                    {
                                                        let mut status_guard = status_clone.lock().unwrap();
                                                        *status_guard = new_status.clone();
                                                    }
                                                    let _ = status_tx_clone.send(new_status).await;
                                                }
                                            }
                                        });
                                    }
                                }
                            } else {
                                let new_status = UpdateStatus::NoUpdates;
                                
                                {
                                    let mut status_guard = status.lock().unwrap();
                                    *status_guard = new_status.clone();
                                }
                                let _ = status_tx.send(new_status).await;
                            }
                        },
                        Err(e) => {
                            error!("Update check failed: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Start the installation process.
    async fn start_installation(
        download_path: &Path,
        config: &UpdateConfig,
        status: Arc<Mutex<UpdateStatus>>,
        status_tx: mpsc::Sender<UpdateStatus>,
        update_info: package::UpdatePackageInfo,
    ) {
        tokio::spawn(async move {
            match installer::install_update(
                download_path,
                &config.install_path,
                &config.backup_path,
                status_tx.clone(),
            ).await {
                Ok(installed_info) => {
                    let new_status = UpdateStatus::InstallationComplete {
                        version: update_info.version.clone(),
                        installation_date: installed_info.installation_date,
                        requires_restart: true, // Assume restart is required
                    };
                    
                    {
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = new_status.clone();
                    }
                    let _ = status_tx.send(new_status).await;
                },
                Err(e) => {
                    error!("Installation failed: {}", e);
                    let new_status = UpdateStatus::InstallationFailed {
                        version: update_info.version.clone(),
                        error: e.to_string(),
                        can_retry: true,
                    };
                    
                    {
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = new_status.clone();
                    }
                    let _ = status_tx.send(new_status).await;
                }
            }
        });
    }
    
    /// Get the current system version.
    fn get_current_version() -> Result<Version> {
        // Try to read the version file
        let version_path = PathBuf::from("/opt/vr-system/version");
        if version_path.exists() {
            let version_str = fs::read_to_string(&version_path)
                .context("Failed to read version file")?;
            
            let version = Version::parse(version_str.trim())
                .context("Failed to parse version")?;
            
            return Ok(version);
        }
        
        // If version file doesn't exist, return a default version
        Ok(Version::new(0, 0, 0))
    }
    
    /// Get the current status of the update system.
    pub fn get_status(&self) -> UpdateStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }
    
    /// Check for updates manually.
    pub async fn check_for_updates(&self) -> Result<Vec<package::UpdatePackageInfo>> {
        // Update status
        {
            let mut status = self.status.lock().unwrap();
            *status = UpdateStatus::CheckingForUpdates;
        }
        let _ = self.status_tx.send(UpdateStatus::CheckingForUpdates).await;
        
        // Perform the update check
        let updates = checker::check_for_updates(&self.config.server_url).await?;
        
        // Store available updates
        {
            let mut updates_guard = self.available_updates.lock().unwrap();
            *updates_guard = updates.clone();
        }
        
        // Update last check time
        {
            let mut last_check = self.last_check_time.lock().unwrap();
            *last_check = Some(SystemTime::now());
        }
        
        // Update status based on results
        if !updates.is_empty() {
            let latest = &updates[0]; // Assuming sorted by version
            let new_status = UpdateStatus::UpdateAvailable {
                version: latest.version.clone(),
                size_bytes: latest.size_bytes,
                release_notes: latest.release_notes.clone(),
                release_date: latest.release_date,
            };
            
            {
                let mut status = self.status.lock().unwrap();
                *status = new_status.clone();
            }
            let _ = self.status_tx.send(new_status).await;
        } else {
            let new_status = UpdateStatus::NoUpdates;
            
            {
                let mut status = self.status.lock().unwrap();
                *status = new_status.clone();
            }
            let _ = self.status_tx.send(new_status).await;
        }
        
        Ok(updates)
    }
    
    /// Download an update.
    pub async fn download_update(&self, update: &package::UpdatePackageInfo) -> Result<PathBuf> {
        // Start the download
        let download_path = downloader::download_update(
            update,
            &self.config.download_path,
            self.config.max_bandwidth_kbps,
            self.status_tx.clone(),
        ).await?;
        
        // Verify the downloaded package
        verifier::verify_package(
            &download_path,
            &self.config.verification_public_key,
        ).await?;
        
        // Update status to ready to install
        let new_status = UpdateStatus::ReadyToInstall {
            version: update.version.clone(),
            size_bytes: update.size_bytes,
            release_notes: update.release_notes.clone(),
        };
        
        {
            let mut status = self.status.lock().unwrap();
            *status = new_status.clone();
        }
        let _ = self.status_tx.send(new_status).await;
        
        Ok(download_path)
    }
    
    /// Install an update.
    pub async fn install_update(&self, package_path: &Path) -> Result<()> {
        // Get package info
        let metadata = package::get_package_info(package_path).await?;
        
        // Check dependencies if enabled
        if self.config.enforce_dependencies {
            let status = UpdateStatus::CheckingDependencies {
                version: metadata.version.clone(),
            };
            let _ = self.status_tx.send(status).await;
            
            // Get installed packages
            let installed_packages = dependency::get_installed_packages(&self.config.install_path)?;
            
            // Get system info
            let system_info = dependency::get_system_info()?;
            
            // Check dependencies
            let dep_result = dependency::check_dependencies(
                &metadata,
                &installed_packages,
                &system_info,
            );
            
            if !dep_result.satisfied {
                // Dependencies not satisfied
                let missing_deps: Vec<String> = dep_result.missing_dependencies
                    .iter()
                    .map(|d| format!("{} {}", d.package_name, d.version_req))
                    .collect();
                
                let conflicts: Vec<String> = dep_result.conflicts
                    .iter()
                    .map(|c| format!("{} {} ({})", c.package_name, c.version_range, c.reason))
                    .collect();
                
                let new_status = UpdateStatus::DependenciesNotSatisfied {
                    version: metadata.version.clone(),
                    missing_dependencies: missing_deps,
                    conflicts,
                };
                
                {
                    let mut status = self.status.lock().unwrap();
                    *status = new_status.clone();
                }
                let _ = self.status_tx.send(new_status).await;
                
                bail!("Dependencies not satisfied");
            }
        }
        
        // Install the update
        let installed_info = installer::install_update(
            package_path,
            &self.config.install_path,
            &self.config.backup_path,
            self.status_tx.clone(),
        ).await?;
        
        // Update status to installation complete
        let new_status = UpdateStatus::InstallationComplete {
            version: metadata.version.clone(),
            installation_date: installed_info.installation_date,
            requires_restart: true, // Assume restart is required
        };
        
        {
            let mut status = self.status.lock().unwrap();
            *status = new_status.clone();
        }
        let _ = self.status_tx.send(new_status).await;
        
        // Update the update history
        {
            let mut history = self.update_history.lock().unwrap();
            history.push(installed_info);
            
            // Save the update history
            let history_path = self.config.install_path.join("update_history.json");
            let history_file = File::create(&history_path)
                .context("Failed to create update history file")?;
            
            serde_json::to_writer_pretty(history_file, &*history)
                .context("Failed to write update history")?;
        }
        
        Ok(())
    }
    
    /// Rollback to a previous version.
    pub async fn rollback(&self, version: &Version) -> Result<()> {
        // Update status to rolling back
        let current_version = Self::get_current_version()?;
        let status = UpdateStatus::RollingBack {
            from_version: current_version.clone(),
            to_version: version.clone(),
            progress_percent: 0.0,
        };
        let _ = self.status_tx.send(status).await;
        
        // Perform the rollback
        installer::rollback_update(
            version,
            &self.config.install_path,
            &self.config.backup_path,
            self.status_tx.clone(),
        ).await?;
        
        // Update status to rollback complete
        let new_status = UpdateStatus::RollbackComplete {
            from_version: current_version,
            to_version: version.clone(),
            rollback_date: Utc::now(),
        };
        
        {
            let mut status = self.status.lock().unwrap();
            *status = new_status.clone();
        }
        let _ = self.status_tx.send(new_status).await;
        
        Ok(())
    }
    
    /// Get available updates.
    pub fn get_available_updates(&self) -> Vec<package::UpdatePackageInfo> {
        let updates = self.available_updates.lock().unwrap();
        updates.clone()
    }
    
    /// Get update history.
    pub fn get_update_history(&self) -> Vec<package::InstalledUpdateInfo> {
        let history = self.update_history.lock().unwrap();
        history.clone()
    }
    
    /// Apply a delta update.
    pub async fn apply_delta_update(&self, delta_package_path: &Path) -> Result<()> {
        // Apply the delta update
        let installed_info = delta::apply_delta_package(
            delta_package_path,
            &self.config.install_path,
            &self.config.backup_path,
        ).await?;
        
        // Update status to installation complete
        let new_status = UpdateStatus::InstallationComplete {
            version: installed_info.version.clone(),
            installation_date: installed_info.installation_date,
            requires_restart: true, // Assume restart is required
        };
        
        {
            let mut status = self.status.lock().unwrap();
            *status = new_status.clone();
        }
        let _ = self.status_tx.send(new_status).await;
        
        // Update the update history
        {
            let mut history = self.update_history.lock().unwrap();
            history.push(installed_info);
            
            // Save the update history
            let history_path = self.config.install_path.join("update_history.json");
            let history_file = File::create(&history_path)
                .context("Failed to create update history file")?;
            
            serde_json::to_writer_pretty(history_file, &*history)
                .context("Failed to write update history")?;
        }
        
        Ok(())
    }
}
