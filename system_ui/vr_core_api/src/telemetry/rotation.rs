//! Log rotation module for the VR headset.
//!
//! This module provides functionality for rotating log files based on
//! size or time, compressing old logs, and managing log file retention.

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::time::{Duration, SystemTime};
use anyhow::{Result, Context, anyhow, bail};
use chrono::{DateTime, Utc, Local};
use flate2::write::GzEncoder;
use flate2::Compression;
use glob::glob;

use super::LogRotationSettings;

/// Rotate log files in the specified directory.
///
/// This function rotates log files according to the provided settings,
/// either based on size, time, or both.
///
/// # Arguments
///
/// * `log_dir` - Directory containing log files
/// * `settings` - Log rotation settings
///
/// # Returns
///
/// `Ok(())` if logs were rotated successfully.
pub fn rotate_logs(log_dir: &Path, settings: &LogRotationSettings) -> Result<()> {
    // Get the main log file path
    let log_file_path = log_dir.join("vr-system.log");
    
    // Check if the log file exists
    if !log_file_path.exists() {
        // Nothing to rotate
        return Ok(());
    }
    
    // Check if rotation is needed based on size
    let mut rotate_by_size = false;
    if settings.rotate_by_size {
        let metadata = fs::metadata(&log_file_path)?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        rotate_by_size = file_size_mb >= settings.max_log_size_mb as f64;
    }
    
    // Check if rotation is needed based on time
    let mut rotate_by_time = false;
    if settings.rotate_by_time {
        if let Ok(metadata) = fs::metadata(&log_file_path) {
            if let Ok(modified_time) = metadata.modified() {
                let now = SystemTime::now();
                if let Ok(duration) = now.duration_since(modified_time) {
                    let hours = duration.as_secs() / 3600;
                    rotate_by_time = hours >= settings.rotation_interval_hours as u64;
                }
            }
        }
    }
    
    // Perform rotation if needed
    if rotate_by_size || rotate_by_time {
        // Generate rotation timestamp
        let timestamp = if settings.timestamp_filenames {
            Local::now().format("_%Y%m%d_%H%M%S").to_string()
        } else {
            String::new()
        };
        
        // Create rotated log filename
        let rotated_log_path = if settings.timestamp_filenames {
            log_dir.join(format!("vr-system{}.log", timestamp))
        } else {
            // Find the next available number
            let mut index = 1;
            loop {
                let path = log_dir.join(format!("vr-system.{}.log", index));
                if !path.exists() {
                    break path;
                }
                index += 1;
            }
        };
        
        // Rename the current log file
        fs::rename(&log_file_path, &rotated_log_path)?;
        
        // Create a new empty log file
        File::create(&log_file_path)?;
        
        // Compress the rotated log file if needed
        if settings.compress_rotated_logs {
            compress_log_file(&rotated_log_path)?;
            
            // Remove the original uncompressed file
            fs::remove_file(&rotated_log_path)?;
        }
        
        // Clean up old log files
        cleanup_old_logs(log_dir, settings)?;
    }
    
    Ok(())
}

/// Compress a log file using gzip.
///
/// # Arguments
///
/// * `log_path` - Path to the log file to compress
///
/// # Returns
///
/// `Ok(())` if the file was compressed successfully.
fn compress_log_file(log_path: &Path) -> Result<()> {
    // Open the input file
    let mut input_file = File::open(log_path)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;
    
    // Create the output file with .gz extension
    let gz_path = log_path.with_extension("log.gz");
    let output_file = File::create(&gz_path)?;
    
    // Compress the data
    let mut encoder = GzEncoder::new(output_file, Compression::default());
    encoder.write_all(&input_data)?;
    encoder.finish()?;
    
    Ok(())
}

/// Clean up old log files, keeping only the specified number of files.
///
/// # Arguments
///
/// * `log_dir` - Directory containing log files
/// * `settings` - Log rotation settings
///
/// # Returns
///
/// `Ok(())` if old logs were cleaned up successfully.
fn cleanup_old_logs(log_dir: &Path, settings: &LogRotationSettings) -> Result<()> {
    // Get all log files (both .log and .log.gz)
    let mut log_files = Vec::new();
    
    // Add .log files
    for entry in glob(&format!("{}/*.log", log_dir.display()))? {
        if let Ok(path) = entry {
            // Skip the main log file
            if path.file_name().unwrap_or_default() != "vr-system.log" {
                log_files.push(path);
            }
        }
    }
    
    // Add .log.gz files
    for entry in glob(&format!("{}/*.log.gz", log_dir.display()))? {
        if let Ok(path) = entry {
            log_files.push(path);
        }
    }
    
    // Sort files by modification time (newest first)
    log_files.sort_by(|a, b| {
        let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });
    
    // Remove excess files
    if log_files.len() > settings.files_to_keep as usize {
        for file in log_files.iter().skip(settings.files_to_keep as usize) {
            fs::remove_file(file)?;
        }
    }
    
    Ok(())
}

/// Check if log rotation is needed.
///
/// # Arguments
///
/// * `log_path` - Path to the log file
/// * `settings` - Log rotation settings
///
/// # Returns
///
/// `true` if rotation is needed, `false` otherwise.
pub fn is_rotation_needed(log_path: &Path, settings: &LogRotationSettings) -> Result<bool> {
    // Check if the log file exists
    if !log_path.exists() {
        return Ok(false);
    }
    
    // Check if rotation is needed based on size
    if settings.rotate_by_size {
        let metadata = fs::metadata(log_path)?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if file_size_mb >= settings.max_log_size_mb as f64 {
            return Ok(true);
        }
    }
    
    // Check if rotation is needed based on time
    if settings.rotate_by_time {
        if let Ok(metadata) = fs::metadata(log_path) {
            if let Ok(modified_time) = metadata.modified() {
                let now = SystemTime::now();
                if let Ok(duration) = now.duration_since(modified_time) {
                    let hours = duration.as_secs() / 3600;
                    if hours >= settings.rotation_interval_hours as u64 {
                        return Ok(true);
                    }
                }
            }
        }
    }
    
    Ok(false)
}

/// Get a list of rotated log files.
///
/// # Arguments
///
/// * `log_dir` - Directory containing log files
///
/// # Returns
///
/// A list of paths to rotated log files, sorted by modification time (newest first).
pub fn get_rotated_logs(log_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut log_files = Vec::new();
    
    // Add .log files
    for entry in glob(&format!("{}/*.log", log_dir.display()))? {
        if let Ok(path) = entry {
            // Skip the main log file
            if path.file_name().unwrap_or_default() != "vr-system.log" {
                log_files.push(path);
            }
        }
    }
    
    // Add .log.gz files
    for entry in glob(&format!("{}/*.log.gz", log_dir.display()))? {
        if let Ok(path) = entry {
            log_files.push(path);
        }
    }
    
    // Sort files by modification time (newest first)
    log_files.sort_by(|a, b| {
        let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });
    
    Ok(log_files)
}

/// Read a compressed log file.
///
/// # Arguments
///
/// * `log_path` - Path to the compressed log file
///
/// # Returns
///
/// The decompressed log content.
pub fn read_compressed_log(log_path: &Path) -> Result<String> {
    // Open the compressed file
    let file = File::open(log_path)?;
    
    // Create a gzip decoder
    let mut decoder = flate2::read::GzDecoder::new(file);
    
    // Read the decompressed content
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;
    
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_is_rotation_needed_by_size() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.log");
        
        // Create a log file with some content
        let mut file = File::create(&log_path).unwrap();
        file.write_all(&vec![0; 1024 * 1024]).unwrap(); // 1MB
        
        let settings = LogRotationSettings {
            rotate_by_size: true,
            max_log_size_mb: 2,
            rotate_by_time: false,
            rotation_interval_hours: 24,
            files_to_keep: 5,
            compress_rotated_logs: true,
            timestamp_filenames: true,
        };
        
        // File is smaller than max size, should not rotate
        assert!(!is_rotation_needed(&log_path, &settings).unwrap());
        
        // Add more content to exceed max size
        file.write_all(&vec![0; 2 * 1024 * 1024]).unwrap(); // 2MB more
        
        // File is larger than max size, should rotate
        assert!(is_rotation_needed(&log_path, &settings).unwrap());
    }
    
    #[test]
    fn test_compress_log_file() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.log");
        
        // Create a log file with some content
        let mut file = File::create(&log_path).unwrap();
        file.write_all(b"Test log content").unwrap();
        
        // Compress the file
        compress_log_file(&log_path).unwrap();
        
        // Check that the compressed file exists
        let gz_path = log_path.with_extension("log.gz");
        assert!(gz_path.exists());
        
        // Check that the compressed file can be read
        let content = read_compressed_log(&gz_path).unwrap();
        assert_eq!(content, "Test log content");
    }
    
    #[test]
    fn test_cleanup_old_logs() {
        let dir = tempdir().unwrap();
        
        // Create some log files
        for i in 1..=10 {
            let log_path = dir.path().join(format!("vr-system.{}.log", i));
            let mut file = File::create(&log_path).unwrap();
            file.write_all(format!("Log file {}", i).as_bytes()).unwrap();
            
            // Add a small delay to ensure different modification times
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let settings = LogRotationSettings {
            rotate_by_size: true,
            max_log_size_mb: 10,
            rotate_by_time: false,
            rotation_interval_hours: 24,
            files_to_keep: 5,
            compress_rotated_logs: true,
            timestamp_filenames: true,
        };
        
        // Clean up old logs
        cleanup_old_logs(dir.path(), &settings).unwrap();
        
        // Check that only the specified number of files remain
        let log_files = get_rotated_logs(dir.path()).unwrap();
        assert_eq!(log_files.len(), 5);
        
        // Check that the newest files were kept
        for file in log_files {
            let filename = file.file_name().unwrap().to_string_lossy();
            assert!(filename.contains("10") || 
                   filename.contains("9") || 
                   filename.contains("8") || 
                   filename.contains("7") || 
                   filename.contains("6"));
        }
    }
}
