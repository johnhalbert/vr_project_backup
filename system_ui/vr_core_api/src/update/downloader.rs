//! Download management module for the VR headset update system.
//!
//! This module provides functionality for downloading update packages
//! from the update server, with progress tracking and bandwidth control.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Write, Seek, SeekFrom};
use std::time::{Duration, Instant};
use anyhow::{Result, Context, anyhow, bail};
use reqwest::{Client, StatusCode};
use tokio::sync::mpsc;
use tokio::time;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn, error, debug};

use super::package::UpdatePackageInfo;
use super::UpdateStatus;

/// Download an update package from the server.
///
/// # Arguments
///
/// * `update` - Information about the update to download
/// * `download_dir` - Directory where the package will be saved
/// * `max_bandwidth_kbps` - Maximum bandwidth to use (0 for unlimited)
/// * `status_tx` - Channel for sending status updates
///
/// # Returns
///
/// The path to the downloaded package.
pub async fn download_update(
    update: &UpdatePackageInfo,
    download_dir: &Path,
    max_bandwidth_kbps: u32,
    status_tx: mpsc::Sender<UpdateStatus>,
) -> Result<PathBuf> {
    // Create the download directory if it doesn't exist
    fs::create_dir_all(download_dir).context("Failed to create download directory")?;
    
    // Determine the output file path
    let file_name = format!("update-{}.vpk", update.version);
    let output_path = download_dir.join(&file_name);
    
    // Check if the file already exists and has the correct hash
    if output_path.exists() {
        let file_size = fs::metadata(&output_path)
            .context("Failed to get file metadata")?
            .len();
        
        if file_size == update.size_bytes {
            // File exists and has the correct size, check the hash
            let hash = super::package::calculate_file_hash(&output_path).await?;
            if hash == update.sha256_hash {
                // File is already downloaded and verified
                debug!("Update package already downloaded and verified: {}", file_name);
                
                // Update status
                let status = UpdateStatus::ReadyToInstall {
                    version: update.version.clone(),
                    size_bytes: update.size_bytes,
                    release_notes: update.release_notes.clone(),
                };
                let _ = status_tx.send(status).await;
                
                return Ok(output_path);
            }
        }
        
        // File exists but is invalid, remove it
        fs::remove_file(&output_path).context("Failed to remove invalid download")?;
    }
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(3600)) // 1 hour timeout
        .build()
        .context("Failed to create HTTP client")?;
    
    // Create the output file
    let mut file = File::create(&output_path)
        .context("Failed to create output file")?;
    
    // Initialize download state
    let mut bytes_downloaded: u64 = 0;
    let total_bytes = update.size_bytes;
    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut last_bytes = 0;
    let mut current_speed: f32 = 0.0;
    
    // Update status to downloading
    let status = UpdateStatus::Downloading {
        version: update.version.clone(),
        progress_percent: 0.0,
        bytes_downloaded: 0,
        total_bytes,
        speed_kbps: 0.0,
    };
    let _ = status_tx.send(status).await;
    
    // Start the download
    debug!("Downloading update package: {}", update.download_url);
    let response = client.get(&update.download_url)
        .header("User-Agent", "VR-Headset-Updater/1.0")
        .send()
        .await
        .context("Failed to send download request")?;
    
    // Check the response status
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        bail!("Update server returned error: {} - {}", status, text);
    }
    
    // Stream the response body to the file
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::with_capacity(65536); // 64 KB buffer
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Failed to download chunk")?;
        
        // Write the chunk to the file
        file.write_all(&chunk).context("Failed to write to output file")?;
        
        // Update progress
        bytes_downloaded += chunk.len() as u64;
        let progress_percent = (bytes_downloaded as f32 / total_bytes as f32) * 100.0;
        
        // Calculate download speed
        let now = Instant::now();
        let elapsed = now.duration_since(last_update).as_millis();
        if elapsed >= 1000 { // Update speed every second
            let bytes_since_last = bytes_downloaded - last_bytes;
            current_speed = (bytes_since_last as f32 / elapsed as f32) * 1000.0 / 1024.0; // KB/s
            last_update = now;
            last_bytes = bytes_downloaded;
            
            // Update status
            let status = UpdateStatus::Downloading {
                version: update.version.clone(),
                progress_percent,
                bytes_downloaded,
                total_bytes,
                speed_kbps: current_speed,
            };
            let _ = status_tx.send(status).await;
        }
        
        // Apply bandwidth limit if specified
        if max_bandwidth_kbps > 0 {
            let target_speed = max_bandwidth_kbps as f32; // KB/s
            if current_speed > target_speed {
                let bytes_per_ms = target_speed / 1000.0 * 1024.0;
                let expected_time_ms = chunk.len() as f32 / bytes_per_ms;
                let actual_time_ms = elapsed as f32;
                if expected_time_ms > actual_time_ms {
                    let delay_ms = (expected_time_ms - actual_time_ms) as u64;
                    time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
        
        // Add to buffer for hash verification
        buffer.extend_from_slice(&chunk);
        
        // If buffer is large enough, calculate partial hash
        if buffer.len() >= 1048576 { // 1 MB
            // TODO: Implement incremental hashing if needed
            buffer.clear();
        }
    }
    
    // Flush and close the file
    file.flush().context("Failed to flush output file")?;
    drop(file);
    
    // Calculate and verify the hash
    let hash = super::package::calculate_file_hash(&output_path).await?;
    if hash != update.sha256_hash {
        // Hash mismatch, remove the file
        fs::remove_file(&output_path).context("Failed to remove invalid download")?;
        bail!("Downloaded file hash mismatch: expected {}, got {}", update.sha256_hash, hash);
    }
    
    // Update status to ready to install
    let status = UpdateStatus::ReadyToInstall {
        version: update.version.clone(),
        size_bytes: update.size_bytes,
        release_notes: update.release_notes.clone(),
    };
    let _ = status_tx.send(status).await;
    
    debug!("Download completed: {}", file_name);
    Ok(output_path)
}

/// Resume a partial download.
///
/// # Arguments
///
/// * `update` - Information about the update to download
/// * `download_dir` - Directory where the package will be saved
/// * `max_bandwidth_kbps` - Maximum bandwidth to use (0 for unlimited)
/// * `status_tx` - Channel for sending status updates
///
/// # Returns
///
/// The path to the downloaded package.
pub async fn resume_download(
    update: &UpdatePackageInfo,
    download_dir: &Path,
    max_bandwidth_kbps: u32,
    status_tx: mpsc::Sender<UpdateStatus>,
) -> Result<PathBuf> {
    // Create the download directory if it doesn't exist
    fs::create_dir_all(download_dir).context("Failed to create download directory")?;
    
    // Determine the output file path
    let file_name = format!("update-{}.vpk", update.version);
    let output_path = download_dir.join(&file_name);
    let temp_path = download_dir.join(format!("{}.part", file_name));
    
    // Check if the file already exists and has the correct hash
    if output_path.exists() {
        let file_size = fs::metadata(&output_path)
            .context("Failed to get file metadata")?
            .len();
        
        if file_size == update.size_bytes {
            // File exists and has the correct size, check the hash
            let hash = super::package::calculate_file_hash(&output_path).await?;
            if hash == update.sha256_hash {
                // File is already downloaded and verified
                debug!("Update package already downloaded and verified: {}", file_name);
                
                // Update status
                let status = UpdateStatus::ReadyToInstall {
                    version: update.version.clone(),
                    size_bytes: update.size_bytes,
                    release_notes: update.release_notes.clone(),
                };
                let _ = status_tx.send(status).await;
                
                return Ok(output_path);
            }
        }
        
        // File exists but is invalid, remove it
        fs::remove_file(&output_path).context("Failed to remove invalid download")?;
    }
    
    // Check if a partial download exists
    let mut bytes_downloaded: u64 = 0;
    let mut file = if temp_path.exists() {
        // Open the partial download file
        let file_size = fs::metadata(&temp_path)
            .context("Failed to get partial file metadata")?
            .len();
        
        if file_size > 0 && file_size < update.size_bytes {
            // Partial download exists, open it for appending
            bytes_downloaded = file_size;
            let file = fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&temp_path)
                .context("Failed to open partial download file")?;
            
            debug!("Resuming download from {} bytes", bytes_downloaded);
            file
        } else {
            // Partial download is invalid, remove it and start fresh
            if temp_path.exists() {
                fs::remove_file(&temp_path).context("Failed to remove invalid partial download")?;
            }
            
            File::create(&temp_path).context("Failed to create download file")?
        }
    } else {
        // No partial download, create a new file
        File::create(&temp_path).context("Failed to create download file")?
    };
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(3600)) // 1 hour timeout
        .build()
        .context("Failed to create HTTP client")?;
    
    // Initialize download state
    let total_bytes = update.size_bytes;
    let start_time = Instant::now();
    let mut last_update = Instant::now();
    let mut last_bytes = bytes_downloaded;
    let mut current_speed: f32 = 0.0;
    
    // Update status to downloading
    let progress_percent = (bytes_downloaded as f32 / total_bytes as f32) * 100.0;
    let status = UpdateStatus::Downloading {
        version: update.version.clone(),
        progress_percent,
        bytes_downloaded,
        total_bytes,
        speed_kbps: 0.0,
    };
    let _ = status_tx.send(status).await;
    
    // Start the download with Range header if resuming
    debug!("Downloading update package: {}", update.download_url);
    let mut request = client.get(&update.download_url)
        .header("User-Agent", "VR-Headset-Updater/1.0");
    
    if bytes_downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", bytes_downloaded));
    }
    
    let response = request.send()
        .await
        .context("Failed to send download request")?;
    
    // Check the response status
    let status_code = response.status();
    if status_code != StatusCode::OK && status_code != StatusCode::PARTIAL_CONTENT {
        let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        bail!("Update server returned error: {} - {}", status_code, text);
    }
    
    // If the server doesn't support range requests, start over
    if bytes_downloaded > 0 && status_code == StatusCode::OK {
        debug!("Server doesn't support range requests, starting download from beginning");
        bytes_downloaded = 0;
        drop(file);
        fs::remove_file(&temp_path).context("Failed to remove partial download")?;
        file = File::create(&temp_path).context("Failed to create download file")?;
    }
    
    // Stream the response body to the file
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::with_capacity(65536); // 64 KB buffer
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Failed to download chunk")?;
        
        // Write the chunk to the file
        file.write_all(&chunk).context("Failed to write to output file")?;
        
        // Update progress
        bytes_downloaded += chunk.len() as u64;
        let progress_percent = (bytes_downloaded as f32 / total_bytes as f32) * 100.0;
        
        // Calculate download speed
        let now = Instant::now();
        let elapsed = now.duration_since(last_update).as_millis();
        if elapsed >= 1000 { // Update speed every second
            let bytes_since_last = bytes_downloaded - last_bytes;
            current_speed = (bytes_since_last as f32 / elapsed as f32) * 1000.0 / 1024.0; // KB/s
            last_update = now;
            last_bytes = bytes_downloaded;
            
            // Update status
            let status = UpdateStatus::Downloading {
                version: update.version.clone(),
                progress_percent,
                bytes_downloaded,
                total_bytes,
                speed_kbps: current_speed,
            };
            let _ = status_tx.send(status).await;
        }
        
        // Apply bandwidth limit if specified
        if max_bandwidth_kbps > 0 {
            let target_speed = max_bandwidth_kbps as f32; // KB/s
            if current_speed > target_speed {
                let bytes_per_ms = target_speed / 1000.0 * 1024.0;
                let expected_time_ms = chunk.len() as f32 / bytes_per_ms;
                let actual_time_ms = elapsed as f32;
                if expected_time_ms > actual_time_ms {
                    let delay_ms = (expected_time_ms - actual_time_ms) as u64;
                    time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
        
        // Add to buffer for hash verification
        buffer.extend_from_slice(&chunk);
        
        // If buffer is large enough, calculate partial hash
        if buffer.len() >= 1048576 { // 1 MB
            // TODO: Implement incremental hashing if needed
            buffer.clear();
        }
    }
    
    // Flush and close the file
    file.flush().context("Failed to flush output file")?;
    drop(file);
    
    // Verify the download size
    let file_size = fs::metadata(&temp_path)
        .context("Failed to get file metadata")?
        .len();
    
    if file_size != update.size_bytes {
        bail!("Downloaded file size mismatch: expected {}, got {}", update.size_bytes, file_size);
    }
    
    // Calculate and verify the hash
    let hash = super::package::calculate_file_hash(&temp_path).await?;
    if hash != update.sha256_hash {
        // Hash mismatch, remove the file
        fs::remove_file(&temp_path).context("Failed to remove invalid download")?;
        bail!("Downloaded file hash mismatch: expected {}, got {}", update.sha256_hash, hash);
    }
    
    // Rename the temporary file to the final name
    fs::rename(&temp_path, &output_path).context("Failed to rename downloaded file")?;
    
    // Update status to ready to install
    let status = UpdateStatus::ReadyToInstall {
        version: update.version.clone(),
        size_bytes: update.size_bytes,
        release_notes: update.release_notes.clone(),
    };
    let _ = status_tx.send(status).await;
    
    debug!("Download completed: {}", file_name);
    Ok(output_path)
}

/// Cancel a download in progress.
///
/// # Arguments
///
/// * `version` - Version of the update being downloaded
/// * `download_dir` - Directory where the package is being saved
///
/// # Returns
///
/// `Ok(())` if the download was cancelled successfully.
pub fn cancel_download(version: &str, download_dir: &Path) -> Result<()> {
    // Determine the file paths
    let file_name = format!("update-{}.vpk", version);
    let output_path = download_dir.join(&file_name);
    let temp_path = download_dir.join(format!("{}.part", file_name));
    
    // Remove the files if they exist
    if temp_path.exists() {
        fs::remove_file(&temp_path).context("Failed to remove partial download")?;
    }
    
    if output_path.exists() {
        fs::remove_file(&output_path).context("Failed to remove download")?;
    }
    
    debug!("Download cancelled: {}", file_name);
    Ok(())
}
