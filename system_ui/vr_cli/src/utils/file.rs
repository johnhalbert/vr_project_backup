// File operations utilities for CLI
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Check if a directory exists
pub fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Create a directory if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).context(format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Read a file to string
pub fn read_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).context(format!("Failed to open file: {}", path.display()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context(format!("Failed to read file: {}", path.display()))?;
    Ok(contents)
}

/// Write string to file
pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }
    
    let mut file = File::create(path).context(format!("Failed to create file: {}", path.display()))?;
    file.write_all(contents.as_bytes()).context(format!("Failed to write to file: {}", path.display()))?;
    Ok(())
}

/// Get the config directory
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("vr");
    
    ensure_dir_exists(&config_dir)?;
    Ok(config_dir)
}

/// Get the data directory
pub fn get_data_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("vr");
    
    ensure_dir_exists(&data_dir)?;
    Ok(data_dir)
}

/// Get the cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("vr");
    
    ensure_dir_exists(&cache_dir)?;
    Ok(cache_dir)
}

/// List files in a directory with a specific extension
pub fn list_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !dir.exists() {
        return Ok(files);
    }
    
    for entry in fs::read_dir(dir).context(format!("Failed to read directory: {}", dir.display()))? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path);
                }
            }
        }
    }
    
    Ok(files)
}

/// Create a backup of a file
pub fn backup_file(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    
    let backup_path = path.with_extension(format!("{}.bak", chrono::Local::now().format("%Y%m%d%H%M%S")));
    fs::copy(path, &backup_path).context(format!("Failed to create backup of file: {}", path.display()))?;
    
    Ok(backup_path)
}

/// Get file size
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path).context(format!("Failed to get metadata for file: {}", path.display()))?;
    Ok(metadata.len())
}

/// Get file modification time
pub fn get_file_modification_time(path: &Path) -> Result<chrono::DateTime<chrono::Local>> {
    let metadata = fs::metadata(path).context(format!("Failed to get metadata for file: {}", path.display()))?;
    let time = metadata.modified().context(format!("Failed to get modification time for file: {}", path.display()))?;
    Ok(chrono::DateTime::from(time))
}
