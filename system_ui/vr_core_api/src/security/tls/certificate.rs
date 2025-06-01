//! Certificate management for the VR headset.
//!
//! This module provides certificate management functionality for the VR headset,
//! including certificate generation, storage, and retrieval.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::Result;
use log::{error};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType};
use rustls::PrivateKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Certificate error.
#[derive(Debug, Error)]
pub enum CertificateError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Certificate generation error
    #[error("Certificate generation error: {0}")]
    Generation(String),
    
    /// Certificate storage error
    #[error("Certificate storage error: {0}")]
    Storage(String),
    
    /// Certificate retrieval error
    #[error("Certificate retrieval error: {0}")]
    Retrieval(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Certificate result type.
pub type CertificateResult<T> = Result<T, CertificateError>;

/// Certificate manager.
pub struct CertificateManager {
    /// Certificates
    certificates: Arc<RwLock<Vec<StoredCertificate>>>,
    
    /// Certificate directory
    cert_dir: PathBuf,
}

impl CertificateManager {
    /// Create a new certificate manager.
    pub fn new(cert_dir: PathBuf) -> CertificateResult<Self> {
        // Create the certificate directory if it doesn't exist
        if !cert_dir.exists() {
            fs::create_dir_all(&cert_dir)?;
        }
        
        // Load certificates
        let mut certificates = Vec::new();
        for entry in fs::read_dir(&cert_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let cert_str = fs::read_to_string(&path)?;
                let cert: StoredCertificate = serde_json::from_str(&cert_str)
                    .map_err(|e| CertificateError::Serialization(format!("Failed to parse certificate: {}", e)))?;
                
                certificates.push(cert);
            }
        }
        
        Ok(Self {
            certificates: Arc::new(RwLock::new(certificates)),
            cert_dir,
        })
    }
    
    /// Generate a certificate.
    pub fn generate_certificate(&self, name: &str, common_name: &str) -> CertificateResult<StoredCertificate> {
        // Check if the certificate already exists
        {
            let certificates = self.certificates.read().unwrap();
            
            for cert in certificates.iter() {
                if cert.name == name {
                    return Err(CertificateError::Generation(format!("Certificate already exists: {}", name)));
                }
            }
        }
        
        // Create certificate parameters
        let mut params = CertificateParams::new(vec![common_name.to_string()]);
        
        // Set the distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name);
        params.distinguished_name = dn;
        
        // Generate the certificate
        let cert = Certificate::from_params(params)
            .map_err(|e| CertificateError::Generation(format!("Failed to generate certificate: {}", e)))?;
        
        // Get the certificate and private key
        let cert_der = cert.serialize_der()
            .map_err(|e| CertificateError::Generation(format!("Failed to serialize certificate: {}", e)))?;
        
        let key_der = cert.serialize_private_key_der();
        
        // Create the stored certificate
        let stored_cert = StoredCertificate {
            name: name.to_string(),
            common_name: common_name.to_string(),
            certificate: cert_der,
            private_key: key_der,
        };
        
        // Save the certificate
        let cert_path = self.cert_dir.join(format!("{}.json", name));
        let cert_str = serde_json::to_string_pretty(&stored_cert)
            .map_err(|e| CertificateError::Serialization(format!("Failed to serialize certificate: {}", e)))?;
        
        let mut file = File::create(&cert_path)?;
        file.write_all(cert_str.as_bytes())?;
        
        // Add the certificate to the list
        let mut certificates = self.certificates.write().unwrap();
        certificates.push(stored_cert.clone());
        
        Ok(stored_cert)
    }
    
    /// Get a certificate.
    pub fn get_certificate(&self, name: &str) -> CertificateResult<StoredCertificate> {
        // Check if the certificate exists
        let certificates = self.certificates.read().unwrap();
        
        for cert in certificates.iter() {
            if cert.name == name {
                return Ok(cert.clone());
            }
        }
        
        Err(CertificateError::Retrieval(format!("Certificate not found: {}", name)))
    }
    
    /// Delete a certificate.
    pub fn delete_certificate(&self, name: &str) -> CertificateResult<()> {
        // Check if the certificate exists
        let mut certificates = self.certificates.write().unwrap();
        
        let mut index = None;
        for (i, cert) in certificates.iter().enumerate() {
            if cert.name == name {
                index = Some(i);
                break;
            }
        }
        
        if let Some(i) = index {
            // Remove the certificate from the list
            certificates.remove(i);
            
            // Delete the certificate file
            let cert_path = self.cert_dir.join(format!("{}.json", name));
            fs::remove_file(&cert_path)?;
            
            Ok(())
        } else {
            Err(CertificateError::Retrieval(format!("Certificate not found: {}", name)))
        }
    }
    
    /// List certificates.
    pub fn list_certificates(&self) -> Vec<String> {
        let certificates = self.certificates.read().unwrap();
        
        certificates.iter().map(|c| c.name.clone()).collect()
    }
}

/// Stored certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCertificate {
    /// Certificate name
    pub name: String,
    
    /// Common name
    pub common_name: String,
    
    /// Certificate data (DER)
    pub certificate: Vec<u8>,
    
    /// Private key data (DER)
    pub private_key: Vec<u8>,
}
