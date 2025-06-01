//! Log forwarding module for the VR headset.
//!
//! This module provides functionality for forwarding logs to remote servers,
//! with support for batching, retry logic, and secure transmission.

use std::collections::HashMap;
use anyhow::{Result, Context, anyhow, bail};
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use tokio::time::{self, Duration};
use uuid::Uuid;

use super::{LogEntry, LogForwardingSettings, LogLevel};

/// Forward logs to a remote server.
///
/// This function sends log entries to a remote server according to the
/// provided settings, with support for batching and retry logic.
///
/// # Arguments
///
/// * `logs` - Log entries to forward
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// `Ok(())` if logs were forwarded successfully.
pub async fn forward_logs(logs: &[LogEntry], settings: &LogForwardingSettings) -> Result<()> {
    // Check if log forwarding is enabled
    if !settings.forward_logs {
        return Ok(());
    }
    
    // Check if there are any logs to forward
    if logs.is_empty() {
        return Ok(());
    }
    
    // Filter logs based on minimum level
    let filtered_logs: Vec<&LogEntry> = logs.iter()
        .filter(|log| log.level >= settings.min_level_to_forward)
        .collect();
    
    if filtered_logs.is_empty() {
        return Ok(());
    }
    
    // Create HTTP client
    let client = create_http_client(settings)?;
    
    // Prepare log data for transmission
    let log_data = prepare_log_data(&filtered_logs, settings)?;
    
    // Send logs with retry logic
    send_logs_with_retry(&client, &log_data, settings).await
}

/// Create an HTTP client for log forwarding.
///
/// # Arguments
///
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// An HTTP client configured according to the settings.
fn create_http_client(settings: &LogForwardingSettings) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    
    // Add authorization header if token is provided
    if !settings.auth_token.is_empty() {
        let auth_value = format!("Bearer {}", settings.auth_token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value)
                .context("Invalid authorization header value")?,
        );
    }
    
    // Create client with appropriate settings
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    Ok(client)
}

/// Prepare log data for transmission.
///
/// # Arguments
///
/// * `logs` - Log entries to prepare
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// Serialized log data ready for transmission.
fn prepare_log_data(logs: &[&LogEntry], settings: &LogForwardingSettings) -> Result<String> {
    // Create log batch
    let log_batch = LogBatch {
        batch_id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        logs: logs.iter().map(|&log| log.clone()).collect(),
    };
    
    // Serialize to JSON
    let json_data = serde_json::to_string(&log_batch)
        .context("Failed to serialize log batch")?;
    
    Ok(json_data)
}

/// Send logs with retry logic.
///
/// # Arguments
///
/// * `client` - HTTP client
/// * `log_data` - Serialized log data
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// `Ok(())` if logs were sent successfully.
async fn send_logs_with_retry(
    client: &Client,
    log_data: &str,
    settings: &LogForwardingSettings,
) -> Result<()> {
    let mut attempts = 0;
    let max_attempts = settings.max_retry_attempts as usize + 1; // +1 for the initial attempt
    
    while attempts < max_attempts {
        attempts += 1;
        
        // Send logs
        let result = send_logs(client, log_data, settings).await;
        
        if result.is_ok() {
            return Ok(());
        }
        
        // If this was the last attempt, return the error
        if attempts >= max_attempts || !settings.retry_on_failure {
            return result;
        }
        
        // Exponential backoff for retries
        let backoff_seconds = 2u64.pow(attempts as u32);
        time::sleep(Duration::from_secs(backoff_seconds)).await;
    }
    
    // This should never be reached due to the loop condition
    Err(anyhow!("Failed to send logs after maximum retry attempts"))
}

/// Send logs to the remote server.
///
/// # Arguments
///
/// * `client` - HTTP client
/// * `log_data` - Serialized log data
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// `Ok(())` if logs were sent successfully.
async fn send_logs(
    client: &Client,
    log_data: &str,
    settings: &LogForwardingSettings,
) -> Result<()> {
    // Build request
    let request = client.post(&settings.log_server_url)
        .header(header::CONTENT_TYPE, "application/json")
        .body(log_data.to_string());
    
    // Send request
    let response = request.send().await
        .context("Failed to send logs to server")?;
    
    // Check response
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        
        return Err(anyhow!("Failed to send logs: HTTP {}: {}", status, error_text));
    }
    
    Ok(())
}

/// Log batch for transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogBatch {
    /// Unique identifier for this batch.
    batch_id: String,
    
    /// Timestamp when this batch was created.
    timestamp: String,
    
    /// Log entries in this batch.
    logs: Vec<LogEntry>,
}

/// Check if the log server is reachable.
///
/// # Arguments
///
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// `Ok(true)` if the server is reachable, `Ok(false)` otherwise.
pub async fn check_log_server(settings: &LogForwardingSettings) -> Result<bool> {
    // Create HTTP client
    let client = create_http_client(settings)?;
    
    // Send a HEAD request to check if the server is reachable
    let request = client.head(&settings.log_server_url);
    
    // Send request with a short timeout
    let response = request
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    
    match response {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Get the status of the log forwarding service.
///
/// # Arguments
///
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// A string describing the status of the log forwarding service.
pub async fn get_forwarding_status(settings: &LogForwardingSettings) -> String {
    if !settings.forward_logs {
        return "Disabled".to_string();
    }
    
    match check_log_server(settings).await {
        Ok(true) => "Connected".to_string(),
        Ok(false) => "Server unreachable".to_string(),
        Err(_) => "Connection error".to_string(),
    }
}

/// Test the log forwarding configuration.
///
/// This function sends a test log entry to the configured server to
/// verify that the configuration is working correctly.
///
/// # Arguments
///
/// * `settings` - Log forwarding settings
///
/// # Returns
///
/// `Ok(())` if the test was successful.
pub async fn test_log_forwarding(settings: &LogForwardingSettings) -> Result<()> {
    // Check if log forwarding is enabled
    if !settings.forward_logs {
        return Err(anyhow!("Log forwarding is disabled"));
    }
    
    // Create a test log entry
    let test_log = LogEntry {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        level: LogLevel::Info,
        source: "LogForwardingTest".to_string(),
        message: "This is a test log entry".to_string(),
        metadata: HashMap::new(),
    };
    
    // Forward the test log
    forward_logs(&[test_log], settings).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    
    #[tokio::test]
    async fn test_prepare_log_data() {
        let logs = vec![
            LogEntry {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                source: "test".to_string(),
                message: "Test message".to_string(),
                metadata: HashMap::new(),
            },
        ];
        
        let settings = LogForwardingSettings::default();
        
        let log_data = prepare_log_data(&logs.iter().collect::<Vec<_>>(), &settings).unwrap();
        
        // Verify that the log data is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&log_data).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("logs").unwrap().is_array());
        assert_eq!(parsed.get("logs").unwrap().as_array().unwrap().len(), 1);
    }
    
    #[tokio::test]
    async fn test_send_logs() {
        // Set up mock server
        let mock_server = mock("POST", "/logs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"status\":\"success\"}")
            .create();
        
        let client = Client::new();
        
        let mut settings = LogForwardingSettings::default();
        settings.log_server_url = format!("{}/logs", server_url());
        
        let log_data = r#"{"batch_id":"test","timestamp":"2023-01-01T00:00:00Z","logs":[]}"#;
        
        let result = send_logs(&client, log_data, &settings).await;
        
        mock_server.assert();
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_send_logs_error() {
        // Set up mock server
        let mock_server = mock("POST", "/logs")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body("{\"status\":\"error\",\"message\":\"Internal server error\"}")
            .create();
        
        let client = Client::new();
        
        let mut settings = LogForwardingSettings::default();
        settings.log_server_url = format!("{}/logs", server_url());
        
        let log_data = r#"{"batch_id":"test","timestamp":"2023-01-01T00:00:00Z","logs":[]}"#;
        
        let result = send_logs(&client, log_data, &settings).await;
        
        mock_server.assert();
        assert!(result.is_err());
    }
}
