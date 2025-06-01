//! Serialization utilities for IPC mechanisms.
//!
//! This module provides serialization and deserialization utilities
//! for IPC messages, supporting various formats and compression.

use std::io::{self, Read, Write};

use bincode::{config, Decode, Encode};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error, warn};

use super::error::{IPCError, Result};
use super::message::IPCMessage;

/// Serialize an IPC message to bytes.
pub fn serialize_message(message: &IPCMessage) -> Result<Vec<u8>> {
    let config = config::standard();
    let mut bytes = bincode::encode_to_vec(message, config)
        .map_err(|e| IPCError::SerializationError(format!("Failed to serialize message: {}", e)))?;
    
    // Compress if needed
    if message.flags.compressed {
        bytes = compress_data(&bytes)?;
    }
    
    // Encrypt if needed
    if message.flags.encrypted {
        // TODO: Implement encryption
        warn!("Message encryption not implemented yet");
    }
    
    Ok(bytes)
}

/// Deserialize an IPC message from bytes.
pub fn deserialize_message(bytes: &[u8]) -> Result<IPCMessage> {
    // Try to deserialize directly first
    let (message, _): (IPCMessage, usize) = match bincode::decode_from_slice(bytes, config::standard()) {
        Ok((message, size)) => (message, size),
        Err(_) => {
            // Try decompressing first
            let decompressed = decompress_data(bytes)?;
            bincode::decode_from_slice(&decompressed, config::standard())
                .map_err(|e| IPCError::SerializationError(format!("Failed to deserialize message: {}", e)))?
        }
    };
    
    // Decrypt if needed
    if message.flags.encrypted {
        // TODO: Implement decryption
        warn!("Message decryption not implemented yet");
    }
    
    Ok(message)
}

/// Compress data using gzip.
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish().map_err(|e| e.into())
}

/// Decompress data using gzip.
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::common::message::{MessagePayload, MessageType};
    
    #[test]
    fn test_serialize_deserialize() {
        let message = IPCMessage::new(
            MessageType::Request,
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        
        let bytes = serialize_message(&message).unwrap();
        let deserialized = deserialize_message(&bytes).unwrap();
        
        assert_eq!(message, deserialized);
    }
    
    #[test]
    fn test_serialize_deserialize_compressed() {
        let mut message = IPCMessage::new(
            MessageType::Request,
            "source",
            "destination",
            MessagePayload::String("test".to_string()),
        );
        message.flags.compressed = true;
        
        let bytes = serialize_message(&message).unwrap();
        let deserialized = deserialize_message(&bytes).unwrap();
        
        assert_eq!(message, deserialized);
    }
    
    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, world!".repeat(100);
        let compressed = compress_data(&data).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
        assert!(compressed.len() < data.len());
    }
}
