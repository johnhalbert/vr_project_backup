//! Delta update module for the VR headset update system.
//!
//! This module provides functionality for creating and applying delta updates,
//! which contain only the differences between versions to minimize download size.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom};
use anyhow::{Result, Context, anyhow, bail};
use serde::{Serialize, Deserialize};
use log::{info, warn, error, debug};
use semver::Version;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use bsdiff::diff as create_binary_diff;
use bspatch::patch as apply_binary_patch;

use super::package::{UpdatePackageMetadata, InstalledUpdateInfo};

/// Information about a delta update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaUpdateInfo {
    /// Base version from which this delta was created.
    pub base_version: Version,
    
    /// Target version that will be achieved after applying this delta.
    pub target_version: Version,
    
    /// Size of the delta package in bytes.
    pub delta_size_bytes: u64,
    
    /// Size of the full update package in bytes.
    pub full_size_bytes: u64,
    
    /// Percentage of size saved compared to full update.
    pub size_reduction_percent: f32,
    
    /// List of files that are included in the delta.
    pub modified_files: Vec<String>,
    
    /// List of files that will be removed.
    pub removed_files: Vec<String>,
    
    /// List of files that will be added.
    pub added_files: Vec<String>,
}

/// Delta file entry in a delta package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaFileEntry {
    /// Path to the file, relative to the installation directory.
    pub path: String,
    
    /// Type of delta operation for this file.
    pub operation: DeltaOperation,
    
    /// SHA-256 hash of the file after applying the delta.
    pub target_hash: String,
    
    /// Size of the file after applying the delta.
    pub target_size: u64,
}

/// Type of delta operation for a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeltaOperation {
    /// File is unchanged from the base version.
    Unchanged,
    
    /// File is modified from the base version.
    Modified {
        /// Base file hash for verification.
        base_hash: String,
        
        /// Offset in the delta data file where the binary diff starts.
        diff_offset: u64,
        
        /// Size of the binary diff in bytes.
        diff_size: u64,
    },
    
    /// File is added in the target version.
    Added {
        /// Offset in the delta data file where the new file content starts.
        content_offset: u64,
        
        /// Size of the new file content in bytes.
        content_size: u64,
    },
    
    /// File is removed in the target version.
    Removed,
}

/// Create a delta update package.
///
/// # Arguments
///
/// * `base_dir` - Directory containing the base version files
/// * `target_dir` - Directory containing the target version files
/// * `output_path` - Path where the delta package will be written
/// * `metadata` - Metadata for the update package
/// * `private_key_path` - Path to the private key file for signing the package
///
/// # Returns
///
/// Information about the created delta update.
pub async fn create_delta_package(
    base_dir: &Path,
    target_dir: &Path,
    output_path: &Path,
    metadata: UpdatePackageMetadata,
    private_key_path: &Path,
) -> Result<DeltaUpdateInfo> {
    debug!("Creating delta package from {} to {}", 
           metadata.base_version.unwrap_or_else(|| Version::new(0, 0, 0)), 
           metadata.version);
    
    // Create a temporary directory for package assembly
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    
    // Create delta manifest file
    let manifest_path = temp_path.join("delta_manifest.json");
    let delta_data_path = temp_path.join("delta_data.bin");
    
    // Open delta data file for writing
    let mut delta_data_file = File::create(&delta_data_path)
        .context("Failed to create delta data file")?;
    
    // Track current offset in the delta data file
    let mut current_offset: u64 = 0;
    
    // Collect information about base files
    let mut base_files = std::collections::HashMap::new();
    for entry in walkdir::WalkDir::new(base_dir) {
        let entry = entry.context("Failed to read base directory entry")?;
        let path = entry.path();
        
        if path.is_file() {
            let relative_path = path.strip_prefix(base_dir)
                .context("Failed to compute relative path")?;
            let relative_path_str = relative_path.to_string_lossy().to_string();
            
            // Calculate hash of the base file
            let hash = super::package::calculate_file_hash(path).await?;
            
            base_files.insert(relative_path_str, (path.to_path_buf(), hash));
        }
    }
    
    // Process target files and create delta entries
    let mut delta_entries = Vec::new();
    let mut modified_files = Vec::new();
    let mut added_files = Vec::new();
    let mut removed_files = Vec::new();
    
    for entry in walkdir::WalkDir::new(target_dir) {
        let entry = entry.context("Failed to read target directory entry")?;
        let path = entry.path();
        
        if path.is_file() {
            let relative_path = path.strip_prefix(target_dir)
                .context("Failed to compute relative path")?;
            let relative_path_str = relative_path.to_string_lossy().to_string();
            
            // Calculate hash and size of the target file
            let target_hash = super::package::calculate_file_hash(path).await?;
            let target_size = fs::metadata(path)
                .context("Failed to get target file metadata")?
                .len();
            
            if let Some((base_path, base_hash)) = base_files.get(&relative_path_str) {
                // File exists in both versions
                if base_hash == &target_hash {
                    // File is unchanged
                    delta_entries.push(DeltaFileEntry {
                        path: relative_path_str,
                        operation: DeltaOperation::Unchanged,
                        target_hash,
                        target_size,
                    });
                } else {
                    // File is modified, create binary diff
                    let base_file = File::open(base_path)
                        .context(format!("Failed to open base file: {}", base_path.display()))?;
                    let target_file = File::open(path)
                        .context(format!("Failed to open target file: {}", path.display()))?;
                    
                    let mut base_data = Vec::new();
                    let mut target_data = Vec::new();
                    
                    base_file.take(10 * 1024 * 1024) // Limit to 10MB to avoid excessive memory usage
                        .read_to_end(&mut base_data)
                        .context("Failed to read base file")?;
                    
                    target_file.take(10 * 1024 * 1024)
                        .read_to_end(&mut target_data)
                        .context("Failed to read target file")?;
                    
                    // Create binary diff
                    let diff = create_binary_diff(&base_data, &target_data)
                        .context("Failed to create binary diff")?;
                    
                    // Write diff to delta data file
                    let diff_offset = current_offset;
                    let diff_size = diff.len() as u64;
                    
                    delta_data_file.write_all(&diff)
                        .context("Failed to write diff to delta data file")?;
                    
                    current_offset += diff_size;
                    
                    // Add entry to delta manifest
                    delta_entries.push(DeltaFileEntry {
                        path: relative_path_str.clone(),
                        operation: DeltaOperation::Modified {
                            base_hash: base_hash.clone(),
                            diff_offset,
                            diff_size,
                        },
                        target_hash,
                        target_size,
                    });
                    
                    modified_files.push(relative_path_str);
                }
                
                // Remove from base_files to track removed files
                base_files.remove(&relative_path_str);
            } else {
                // File is new in target version
                let target_file = File::open(path)
                    .context(format!("Failed to open target file: {}", path.display()))?;
                
                let mut target_data = Vec::new();
                target_file.take(10 * 1024 * 1024)
                    .read_to_end(&mut target_data)
                    .context("Failed to read target file")?;
                
                // Write new file content to delta data file
                let content_offset = current_offset;
                let content_size = target_data.len() as u64;
                
                delta_data_file.write_all(&target_data)
                    .context("Failed to write new file content to delta data file")?;
                
                current_offset += content_size;
                
                // Add entry to delta manifest
                delta_entries.push(DeltaFileEntry {
                    path: relative_path_str.clone(),
                    operation: DeltaOperation::Added {
                        content_offset,
                        content_size,
                    },
                    target_hash,
                    target_size,
                });
                
                added_files.push(relative_path_str);
            }
        }
    }
    
    // Process removed files
    for (relative_path_str, _) in base_files {
        delta_entries.push(DeltaFileEntry {
            path: relative_path_str.clone(),
            operation: DeltaOperation::Removed,
            target_hash: String::new(),
            target_size: 0,
        });
        
        removed_files.push(relative_path_str);
    }
    
    // Finalize delta data file
    delta_data_file.flush().context("Failed to flush delta data file")?;
    drop(delta_data_file);
    
    // Write delta manifest
    let manifest_file = File::create(&manifest_path)
        .context("Failed to create delta manifest file")?;
    serde_json::to_writer_pretty(manifest_file, &delta_entries)
        .context("Failed to write delta manifest")?;
    
    // Create the final package (tar.gz containing metadata, delta manifest, and delta data)
    let package_file = File::create(output_path)
        .context("Failed to create package file")?;
    let encoder = flate2::write::GzEncoder::new(package_file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    
    // Add metadata to the package
    let metadata_path = temp_path.join("metadata.json");
    let metadata_file = File::create(&metadata_path)
        .context("Failed to create metadata file")?;
    serde_json::to_writer_pretty(metadata_file, &metadata)
        .context("Failed to write metadata")?;
    
    builder.append_path_with_name(&metadata_path, "metadata.json")
        .context("Failed to add metadata to package")?;
    
    // Add delta manifest to the package
    builder.append_path_with_name(&manifest_path, "delta_manifest.json")
        .context("Failed to add delta manifest to package")?;
    
    // Add delta data to the package
    builder.append_path_with_name(&delta_data_path, "delta_data.bin")
        .context("Failed to add delta data to package")?;
    
    // Finalize the package
    let encoder = builder.into_inner().context("Failed to finalize package")?;
    encoder.finish().context("Failed to finish compression")?;
    
    // Calculate package size
    let delta_size_bytes = fs::metadata(output_path)
        .context("Failed to get delta package metadata")?
        .len();
    
    // Calculate full package size (approximate)
    let full_size_bytes = fs::metadata(target_dir)
        .context("Failed to get target directory metadata")?
        .len();
    
    // Calculate size reduction percentage
    let size_reduction_percent = if full_size_bytes > 0 {
        ((full_size_bytes - delta_size_bytes) as f32 / full_size_bytes as f32) * 100.0
    } else {
        0.0
    };
    
    // Create delta update info
    let delta_info = DeltaUpdateInfo {
        base_version: metadata.base_version.unwrap_or_else(|| Version::new(0, 0, 0)),
        target_version: metadata.version,
        delta_size_bytes,
        full_size_bytes,
        size_reduction_percent,
        modified_files,
        removed_files,
        added_files,
    };
    
    debug!("Delta package created: {} -> {}, size: {} bytes, reduction: {:.1}%",
           delta_info.base_version, delta_info.target_version,
           delta_info.delta_size_bytes, delta_info.size_reduction_percent);
    
    Ok(delta_info)
}

/// Apply a delta update package.
///
/// # Arguments
///
/// * `delta_package_path` - Path to the delta update package
/// * `install_dir` - Directory where the update will be applied
/// * `backup_dir` - Directory for storing backups of replaced files
///
/// # Returns
///
/// Information about the applied update.
pub async fn apply_delta_package(
    delta_package_path: &Path,
    install_dir: &Path,
    backup_dir: &Path,
) -> Result<InstalledUpdateInfo> {
    debug!("Applying delta package: {}", delta_package_path.display());
    
    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let extract_dir = temp_path.path();
    
    // Extract the delta package
    let package_file = File::open(delta_package_path)
        .context("Failed to open delta package file")?;
    let decoder = flate2::read::GzDecoder::new(package_file);
    let mut archive = tar::Archive::new(decoder);
    
    archive.unpack(extract_dir).context("Failed to extract delta package")?;
    
    // Read the metadata
    let metadata_path = extract_dir.join("metadata.json");
    let metadata_file = File::open(&metadata_path)
        .context("Failed to open metadata file")?;
    let metadata: UpdatePackageMetadata = serde_json::from_reader(metadata_file)
        .context("Failed to parse metadata")?;
    
    // Read the delta manifest
    let manifest_path = extract_dir.join("delta_manifest.json");
    let manifest_file = File::open(&manifest_path)
        .context("Failed to open delta manifest file")?;
    let delta_entries: Vec<DeltaFileEntry> = serde_json::from_reader(manifest_file)
        .context("Failed to parse delta manifest")?;
    
    // Open the delta data file
    let delta_data_path = extract_dir.join("delta_data.bin");
    let mut delta_data_file = File::open(&delta_data_path)
        .context("Failed to open delta data file")?;
    
    // Create the backup directory if it doesn't exist
    let backup_path = backup_dir.join(format!("backup-{}", metadata.version));
    fs::create_dir_all(&backup_path).context("Failed to create backup directory")?;
    
    // Apply delta entries
    for entry in &delta_entries {
        let file_path = install_dir.join(&entry.path);
        
        match &entry.operation {
            DeltaOperation::Unchanged => {
                // Verify that the file exists and has the correct hash
                if file_path.exists() {
                    let hash = super::package::calculate_file_hash(&file_path).await?;
                    if hash != entry.target_hash {
                        bail!("Hash mismatch for unchanged file: {}", entry.path);
                    }
                } else {
                    bail!("Missing unchanged file: {}", entry.path);
                }
            },
            
            DeltaOperation::Modified { base_hash, diff_offset, diff_size } => {
                // Verify that the file exists and has the correct base hash
                if !file_path.exists() {
                    bail!("Missing file to modify: {}", entry.path);
                }
                
                let hash = super::package::calculate_file_hash(&file_path).await?;
                if hash != *base_hash {
                    bail!("Base hash mismatch for file: {}", entry.path);
                }
                
                // Create a backup of the existing file
                let backup_file_path = backup_path.join(&entry.path);
                
                // Create parent directories for the backup if needed
                if let Some(parent) = backup_file_path.parent() {
                    fs::create_dir_all(parent).context("Failed to create backup directory")?;
                }
                
                // Copy the existing file to the backup directory
                fs::copy(&file_path, &backup_file_path)
                    .context(format!("Failed to backup file: {}", entry.path))?;
                
                // Read the base file
                let mut base_file = File::open(&file_path)
                    .context(format!("Failed to open base file: {}", file_path.display()))?;
                
                let mut base_data = Vec::new();
                base_file.read_to_end(&mut base_data)
                    .context("Failed to read base file")?;
                
                // Read the diff data
                delta_data_file.seek(SeekFrom::Start(*diff_offset))
                    .context("Failed to seek to diff offset")?;
                
                let mut diff_data = vec![0; *diff_size as usize];
                delta_data_file.read_exact(&mut diff_data)
                    .context("Failed to read diff data")?;
                
                // Apply the binary patch
                let patched_data = apply_binary_patch(&base_data, &diff_data)
                    .context("Failed to apply binary patch")?;
                
                // Write the patched data to the file
                let mut output_file = File::create(&file_path)
                    .context(format!("Failed to create output file: {}", file_path.display()))?;
                
                output_file.write_all(&patched_data)
                    .context("Failed to write patched data")?;
                
                // Verify the hash of the patched file
                let hash = super::package::calculate_file_hash(&file_path).await?;
                if hash != entry.target_hash {
                    bail!("Target hash mismatch after patching: {}", entry.path);
                }
            },
            
            DeltaOperation::Added { content_offset, content_size } => {
                // Create parent directories if needed
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).context("Failed to create directory")?;
                }
                
                // Read the new file content
                delta_data_file.seek(SeekFrom::Start(*content_offset))
                    .context("Failed to seek to content offset")?;
                
                let mut content_data = vec![0; *content_size as usize];
                delta_data_file.read_exact(&mut content_data)
                    .context("Failed to read content data")?;
                
                // Write the new file
                let mut output_file = File::create(&file_path)
                    .context(format!("Failed to create output file: {}", file_path.display()))?;
                
                output_file.write_all(&content_data)
                    .context("Failed to write new file content")?;
                
                // Verify the hash of the new file
                let hash = super::package::calculate_file_hash(&file_path).await?;
                if hash != entry.target_hash {
                    bail!("Target hash mismatch for new file: {}", entry.path);
                }
            },
            
            DeltaOperation::Removed => {
                // Create a backup of the existing file if it exists
                if file_path.exists() {
                    let backup_file_path = backup_path.join(&entry.path);
                    
                    // Create parent directories for the backup if needed
                    if let Some(parent) = backup_file_path.parent() {
                        fs::create_dir_all(parent).context("Failed to create backup directory")?;
                    }
                    
                    // Copy the existing file to the backup directory
                    fs::copy(&file_path, &backup_file_path)
                        .context(format!("Failed to backup file: {}", entry.path))?;
                    
                    // Remove the file
                    fs::remove_file(&file_path)
                        .context(format!("Failed to remove file: {}", entry.path))?;
                }
            },
        }
    }
    
    // Create the installed update info
    let installed_update = InstalledUpdateInfo {
        version: metadata.version,
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
    
    debug!("Delta update applied successfully: {}", metadata.version);
    Ok(installed_update)
}

/// Check if a delta update is available for the current version.
///
/// # Arguments
///
/// * `server_url` - URL of the update server
/// * `current_version` - Current system version
///
/// # Returns
///
/// Information about the available delta update, or `None` if no delta update is available.
pub async fn check_for_delta_update(
    server_url: &str,
    current_version: &Version,
) -> Result<Option<DeltaUpdateInfo>> {
    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    // Build the request URL
    let url = format!("{}/api/delta-updates?version={}", server_url, current_version);
    
    // Send the request
    debug!("Checking for delta updates at {}", url);
    let response = client.get(&url)
        .header("User-Agent", "VR-Headset-Updater/1.0")
        .send()
        .await
        .context("Failed to send delta update check request")?;
    
    // Check the response status
    if !response.status().is_success() {
        let status = response.status();
        
        // If the server returns 404, it means no delta update is available
        if status.as_u16() == 404 {
            return Ok(None);
        }
        
        let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        bail!("Update server returned error: {} - {}", status, text);
    }
    
    // Parse the response
    let delta_info: DeltaUpdateInfo = response.json()
        .await
        .context("Failed to parse delta update server response")?;
    
    debug!("Found delta update: {} -> {}, size: {} bytes, reduction: {:.1}%",
           delta_info.base_version, delta_info.target_version,
           delta_info.delta_size_bytes, delta_info.size_reduction_percent);
    
    Ok(Some(delta_info))
}
