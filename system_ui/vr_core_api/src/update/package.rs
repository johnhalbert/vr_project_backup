//! Update package format and related utilities.
//!
//! This module defines the structure of update packages, including metadata,
//! versioning, and package creation/extraction utilities.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek};
use anyhow::{Result, Context, anyhow, bail};
use serde::{Serialize, Deserialize};
use semver::Version;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use ring::signature;
use tar::{Archive, Builder};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use log::{info, warn, error, debug};

/// Information about an available update package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePackageInfo {
    /// Version of the update.
    pub version: Version,
    
    /// Size of the update package in bytes.
    pub size_bytes: u64,
    
    /// Release notes for the update.
    pub release_notes: String,
    
    /// Release date of the update.
    pub release_date: DateTime<Utc>,
    
    /// URL to download the update package.
    pub download_url: String,
    
    /// SHA-256 hash of the update package.
    pub sha256_hash: String,
    
    /// Digital signature of the update package.
    pub signature: String,
    
    /// List of components included in the update.
    pub components: Vec<String>,
    
    /// Minimum system version required to install this update.
    pub min_system_version: Option<Version>,
    
    /// Whether this update is a critical security update.
    pub is_security_update: bool,
    
    /// Whether this update requires a system restart.
    pub requires_restart: bool,
    
    /// Estimated installation time in seconds.
    pub estimated_install_time_seconds: u32,
}

/// Information about an installed update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledUpdateInfo {
    /// Version of the update.
    pub version: Version,
    
    /// Date and time when the update was installed.
    pub installation_date: DateTime<Utc>,
    
    /// Whether the installation was successful.
    pub installation_successful: bool,
    
    /// Error message if installation failed.
    pub error_message: Option<String>,
}

/// Metadata for an update package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePackageMetadata {
    /// Version of the update.
    pub version: Version,
    
    /// Release notes for the update.
    pub release_notes: String,
    
    /// Release date of the update.
    pub release_date: DateTime<Utc>,
    
    /// List of components included in the update.
    pub components: Vec<String>,
    
    /// Minimum system version required to install this update.
    pub min_system_version: Option<Version>,
    
    /// Whether this update is a critical security update.
    pub is_security_update: bool,
    
    /// Whether this update requires a system restart.
    pub requires_restart: bool,
    
    /// Estimated installation time in seconds.
    pub estimated_install_time_seconds: u32,
    
    /// SHA-256 hash of the update package content (excluding metadata and signature).
    pub content_hash: String,
    
    /// Digital signature of the content hash.
    pub signature: String,
}

/// Create a new update package.
///
/// # Arguments
///
/// * `source_dir` - Directory containing the files to include in the package
/// * `output_path` - Path where the update package will be written
/// * `metadata` - Metadata for the update package
/// * `private_key_path` - Path to the private key file for signing the package
///
/// # Returns
///
/// The path to the created update package.
pub async fn create_package(
    source_dir: &Path,
    output_path: &Path,
    metadata: UpdatePackageMetadata,
    private_key_path: &Path,
) -> Result<PathBuf> {
    // Create a temporary directory for package assembly
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    
    // Create content archive (tar.gz)
    let content_path = temp_path.join("content.tar.gz");
    let content_file = File::create(&content_path).context("Failed to create content archive")?;
    let encoder = GzEncoder::new(content_file, Compression::default());
    let mut builder = Builder::new(encoder);
    
    // Add all files from source directory to the archive
    for entry in walkdir::WalkDir::new(source_dir) {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_file() {
            let relative_path = path.strip_prefix(source_dir)
                .context("Failed to compute relative path")?;
            
            builder.append_path_with_name(path, relative_path)
                .context("Failed to add file to archive")?;
        }
    }
    
    // Finalize the archive
    let encoder = builder.into_inner().context("Failed to finalize archive")?;
    encoder.finish().context("Failed to finish compression")?;
    
    // Calculate content hash
    let content_hash = calculate_file_hash(&content_path).await?;
    
    // Update metadata with content hash
    let mut metadata = metadata;
    metadata.content_hash = content_hash.clone();
    
    // Sign the content hash
    let private_key_data = fs::read(private_key_path)
        .context("Failed to read private key file")?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(&private_key_data)
        .map_err(|_| anyhow!("Invalid private key format"))?;
    
    let signature = key_pair.sign(content_hash.as_bytes());
    metadata.signature = base64::encode(signature.as_ref());
    
    // Serialize metadata to JSON
    let metadata_path = temp_path.join("metadata.json");
    let metadata_file = File::create(&metadata_path)
        .context("Failed to create metadata file")?;
    serde_json::to_writer_pretty(metadata_file, &metadata)
        .context("Failed to write metadata")?;
    
    // Create the final package (another tar.gz containing metadata and content)
    let package_file = File::create(output_path)
        .context("Failed to create package file")?;
    let encoder = GzEncoder::new(package_file, Compression::default());
    let mut builder = Builder::new(encoder);
    
    // Add metadata and content to the package
    builder.append_path_with_name(&metadata_path, "metadata.json")
        .context("Failed to add metadata to package")?;
    builder.append_path_with_name(&content_path, "content.tar.gz")
        .context("Failed to add content to package")?;
    
    // Finalize the package
    let encoder = builder.into_inner().context("Failed to finalize package")?;
    encoder.finish().context("Failed to finish compression")?;
    
    Ok(output_path.to_path_buf())
}

/// Extract an update package.
///
/// # Arguments
///
/// * `package_path` - Path to the update package
/// * `output_dir` - Directory where the package contents will be extracted
///
/// # Returns
///
/// The metadata of the extracted package.
pub async fn extract_package(
    package_path: &Path,
    output_dir: &Path,
) -> Result<UpdatePackageMetadata> {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    
    // Open the package file
    let package_file = File::open(package_path)
        .context("Failed to open package file")?;
    let decoder = GzDecoder::new(package_file);
    let mut archive = Archive::new(decoder);
    
    // Create a temporary directory for package extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    
    // Extract the package to the temporary directory
    archive.unpack(temp_path).context("Failed to extract package")?;
    
    // Read the metadata
    let metadata_path = temp_path.join("metadata.json");
    let metadata_file = File::open(&metadata_path)
        .context("Failed to open metadata file")?;
    let metadata: UpdatePackageMetadata = serde_json::from_reader(metadata_file)
        .context("Failed to parse metadata")?;
    
    // Extract the content archive
    let content_path = temp_path.join("content.tar.gz");
    let content_file = File::open(&content_path)
        .context("Failed to open content archive")?;
    let decoder = GzDecoder::new(content_file);
    let mut archive = Archive::new(decoder);
    
    // Extract the content to the output directory
    archive.unpack(output_dir).context("Failed to extract content")?;
    
    Ok(metadata)
}

/// Calculate the SHA-256 hash of a file.
///
/// # Arguments
///
/// * `file_path` - Path to the file
///
/// # Returns
///
/// The SHA-256 hash as a hexadecimal string.
pub async fn calculate_file_hash(file_path: &Path) -> Result<String> {
    // Open the file
    let mut file = File::open(file_path)
        .context("Failed to open file for hashing")?;
    
    // Create a hasher
    let mut hasher = Sha256::new();
    
    // Read the file in chunks and update the hasher
    let mut buffer = [0; 65536]; // 64 KB buffer
    loop {
        let bytes_read = file.read(&mut buffer)
            .context("Failed to read file for hashing")?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    // Finalize the hash and convert to hexadecimal
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);
    
    Ok(hash_hex)
}

/// Verify the integrity of an update package.
///
/// # Arguments
///
/// * `package_path` - Path to the update package
/// * `public_key` - Public key for verifying the signature (base64-encoded)
///
/// # Returns
///
/// `Ok(())` if the package is valid, or an error if it's invalid.
pub async fn verify_package_integrity(
    package_path: &Path,
    public_key: &str,
) -> Result<()> {
    // Create a temporary directory for package extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    
    // Open the package file
    let package_file = File::open(package_path)
        .context("Failed to open package file")?;
    let decoder = GzDecoder::new(package_file);
    let mut archive = Archive::new(decoder);
    
    // Extract the package to the temporary directory
    archive.unpack(temp_path).context("Failed to extract package")?;
    
    // Read the metadata
    let metadata_path = temp_path.join("metadata.json");
    let metadata_file = File::open(&metadata_path)
        .context("Failed to open metadata file")?;
    let metadata: UpdatePackageMetadata = serde_json::from_reader(metadata_file)
        .context("Failed to parse metadata")?;
    
    // Calculate the hash of the content archive
    let content_path = temp_path.join("content.tar.gz");
    let content_hash = calculate_file_hash(&content_path).await?;
    
    // Verify that the calculated hash matches the one in the metadata
    if content_hash != metadata.content_hash {
        bail!("Content hash mismatch: package may be corrupted");
    }
    
    // Verify the signature
    let public_key_bytes = base64::decode(public_key)
        .context("Invalid public key format")?;
    let public_key = signature::UnparsedPublicKey::new(
        &signature::ED25519,
        &public_key_bytes,
    );
    
    let signature_bytes = base64::decode(&metadata.signature)
        .context("Invalid signature format")?;
    
    public_key.verify(metadata.content_hash.as_bytes(), &signature_bytes)
        .map_err(|_| anyhow!("Invalid signature: package may be tampered with"))?;
    
    Ok(())
}

/// Get information about an update package.
///
/// # Arguments
///
/// * `package_path` - Path to the update package
///
/// # Returns
///
/// Metadata of the update package.
pub async fn get_package_info(package_path: &Path) -> Result<UpdatePackageMetadata> {
    // Create a temporary directory for package extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    
    // Open the package file
    let package_file = File::open(package_path)
        .context("Failed to open package file")?;
    let decoder = GzDecoder::new(package_file);
    let mut archive = Archive::new(decoder);
    
    // Extract only the metadata file
    for entry in archive.entries().context("Failed to read package entries")? {
        let mut entry = entry.context("Failed to read package entry")?;
        let path = entry.path().context("Failed to get entry path")?;
        
        if path.to_string_lossy() == "metadata.json" {
            let output_path = temp_path.join("metadata.json");
            let mut output_file = File::create(&output_path)
                .context("Failed to create metadata file")?;
            
            io::copy(&mut entry, &mut output_file)
                .context("Failed to extract metadata file")?;
            
            // Read the metadata
            let metadata_file = File::open(&output_path)
                .context("Failed to open metadata file")?;
            let metadata: UpdatePackageMetadata = serde_json::from_reader(metadata_file)
                .context("Failed to parse metadata")?;
            
            return Ok(metadata);
        }
    }
    
    bail!("Metadata not found in package")
}

/// Convert an update package to package info.
///
/// # Arguments
///
/// * `package_path` - Path to the update package
/// * `download_url` - URL where the package can be downloaded
///
/// # Returns
///
/// Information about the update package.
pub async fn package_to_info(
    package_path: &Path,
    download_url: &str,
) -> Result<UpdatePackageInfo> {
    // Get the package metadata
    let metadata = get_package_info(package_path).await?;
    
    // Get the package size
    let size_bytes = fs::metadata(package_path)
        .context("Failed to get package file size")?
        .len();
    
    // Calculate the package hash
    let sha256_hash = calculate_file_hash(package_path).await?;
    
    // Create the package info
    let info = UpdatePackageInfo {
        version: metadata.version,
        size_bytes,
        release_notes: metadata.release_notes,
        release_date: metadata.release_date,
        download_url: download_url.to_string(),
        sha256_hash,
        signature: metadata.signature,
        components: metadata.components,
        min_system_version: metadata.min_system_version,
        is_security_update: metadata.is_security_update,
        requires_restart: metadata.requires_restart,
        estimated_install_time_seconds: metadata.estimated_install_time_seconds,
    };
    
    Ok(info)
}
