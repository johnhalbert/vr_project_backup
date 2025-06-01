//! Secure cookie handling for the VR headset.
//!
//! This module provides secure cookie functionality for the HTTPS server,
//! including encryption, signing, and validation.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac, digest::KeyInit as HmacKeyInit};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Cookie error type.
#[derive(Debug, Error)]
pub enum CookieError {
    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    /// Signing error
    #[error("Signing error: {0}")]
    Signing(String),
    
    /// Verification error
    #[error("Verification error: {0}")]
    Verification(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Expired cookie
    #[error("Expired cookie")]
    Expired,
    
    /// Invalid cookie
    #[error("Invalid cookie: {0}")]
    Invalid(String),
}

/// Cookie result type.
pub type CookieResult<T> = Result<T, CookieError>;

/// Secure cookie manager.
pub struct CookieManager {
    /// Encryption key
    encryption_key: [u8; 32],
    
    /// Signing key
    signing_key: [u8; 32],
}

impl CookieManager {
    /// Create a new cookie manager with random keys.
    pub fn new() -> Self {
        let mut encryption_key = [0u8; 32];
        let mut signing_key = [0u8; 32];
        
        OsRng.fill_bytes(&mut encryption_key);
        OsRng.fill_bytes(&mut signing_key);
        
        Self {
            encryption_key,
            signing_key,
        }
    }
    
    /// Create a new cookie manager with specific keys.
    pub fn with_keys(encryption_key: [u8; 32], signing_key: [u8; 32]) -> Self {
        Self {
            encryption_key,
            signing_key,
        }
    }
    
    /// Create a secure cookie.
    pub fn create_cookie(
        &self,
        name: &str,
        value: &str,
        max_age: Option<Duration>,
        http_only: bool,
        secure: bool,
        same_site: SameSite,
    ) -> CookieResult<SecureCookie> {
        // Create the cookie data
        let expires_at = max_age.map(|d| SystemTime::now() + d);
        
        let data = CookieData {
            value: value.to_string(),
            created_at: SystemTime::now(),
            expires_at,
        };
        
        // Serialize the data
        let data_json = serde_json::to_string(&data)
            .map_err(|e| CookieError::Serialization(format!("Failed to serialize cookie data: {}", e)))?;
        
        // Encrypt the data
        let encrypted = self.encrypt(&data_json)?;
        
        // Create the cookie
        let cookie = SecureCookie {
            name: name.to_string(),
            value: encrypted,
            http_only,
            secure,
            same_site,
            max_age,
        };
        
        Ok(cookie)
    }
    
    /// Parse a secure cookie.
    pub fn parse_cookie(&self, _name: &str, value: &str) -> CookieResult<CookieData> {
        // Decrypt the value
        let decrypted = self.decrypt(value)?;
        
        // Deserialize the data
        let data: CookieData = serde_json::from_str(&decrypted)
            .map_err(|e| CookieError::Serialization(format!("Failed to deserialize cookie data: {}", e)))?;
        
        // Check if the cookie is expired
        if let Some(expires_at) = data.expires_at {
            if SystemTime::now() > expires_at {
                return Err(CookieError::Expired);
            }
        }
        
        Ok(data)
    }
    
    /// Encrypt data.
    fn encrypt(&self, data: &str) -> CookieResult<String> {
        // Create a nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Create the cipher
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| CookieError::Encryption(format!("Failed to create cipher: {}", e)))?;
        
        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| CookieError::Encryption(format!("Failed to encrypt data: {}", e)))?;
        
        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        // Sign the result
        let signature = self.sign(&result)?;
        
        // Combine result and signature
        let mut final_result = Vec::with_capacity(result.len() + signature.len());
        final_result.extend_from_slice(&result);
        final_result.extend_from_slice(&signature);
        
        // Encode as base64
        let encoded = BASE64.encode(final_result);
        
        Ok(encoded)
    }
    
    /// Decrypt data.
    fn decrypt(&self, data: &str) -> CookieResult<String> {
        // Decode from base64
        let decoded = BASE64.decode(data)
            .map_err(|e| CookieError::Decryption(format!("Failed to decode base64: {}", e)))?;
        
        // Check if the data is long enough
        if decoded.len() < 12 + 32 {
            return Err(CookieError::Decryption("Data too short".to_string()));
        }
        
        // Split into data and signature
        let signature_start = decoded.len() - 32;
        let data_part = &decoded[..signature_start];
        let signature_part = &decoded[signature_start..];
        
        // Verify the signature
        self.verify(data_part, signature_part)?;
        
        // Split data into nonce and ciphertext
        let nonce_bytes = &data_part[..12];
        let ciphertext = &data_part[12..];
        
        // Create the cipher
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| CookieError::Decryption(format!("Failed to create cipher: {}", e)))?;
        
        // Decrypt the data
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
            .map_err(|e| CookieError::Decryption(format!("Failed to decrypt data: {}", e)))?;
        
        // Convert to string
        let plaintext_str = String::from_utf8(plaintext)
            .map_err(|e| CookieError::Decryption(format!("Failed to convert to string: {}", e)))?;
        
        Ok(plaintext_str)
    }
    
    /// Sign data.
    fn sign(&self, data: &[u8]) -> CookieResult<Vec<u8>> {
        // Create the HMAC with explicit trait usage to avoid ambiguity
        let mut mac = <HmacSha256 as HmacKeyInit>::new_from_slice(&self.signing_key)
            .map_err(|_| CookieError::Signing("Failed to create HMAC".to_string()))?;
        
        // Update with data
        mac.update(data);
        
        // Get the signature
        let signature = mac.finalize().into_bytes().to_vec();
        
        Ok(signature)
    }
    
    /// Verify a signature.
    fn verify(&self, data: &[u8], signature: &[u8]) -> CookieResult<()> {
        // Create the HMAC with explicit trait usage to avoid ambiguity
        let mut mac = <HmacSha256 as HmacKeyInit>::new_from_slice(&self.signing_key)
            .map_err(|_| CookieError::Verification("Failed to create HMAC".to_string()))?;
        
        // Update with data
        mac.update(data);
        
        // Verify the signature
        mac.verify_slice(signature)
            .map_err(|_| CookieError::Verification("Invalid signature".to_string()))?;
        
        Ok(())
    }
    
    /// Parse cookies from a header.
    pub fn parse_cookies(&self, header: &str) -> HashMap<String, CookieData> {
        let mut cookies = HashMap::new();
        
        for cookie_str in header.split(';') {
            let cookie_str = cookie_str.trim();
            
            if let Some(pos) = cookie_str.find('=') {
                let name = cookie_str[..pos].trim().to_string();
                let value = cookie_str[pos + 1..].trim().to_string();
                
                if let Ok(data) = self.parse_cookie(&name, &value) {
                    cookies.insert(name, data);
                }
            }
        }
        
        cookies
    }
}

impl Default for CookieManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CookieManager {
    fn clone(&self) -> Self {
        Self {
            encryption_key: self.encryption_key,
            signing_key: self.signing_key,
        }
    }
}

/// Cookie data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieData {
    /// Cookie value
    pub value: String,
    
    /// Created at
    pub created_at: SystemTime,
    
    /// Expires at
    pub expires_at: Option<SystemTime>,
}

/// Secure cookie.
#[derive(Debug, Clone)]
pub struct SecureCookie {
    /// Cookie name
    pub name: String,
    
    /// Cookie value
    pub value: String,
    
    /// HTTP only
    pub http_only: bool,
    
    /// Secure
    pub secure: bool,
    
    /// Same site
    pub same_site: SameSite,
    
    /// Max age
    pub max_age: Option<Duration>,
}

impl SecureCookie {
    /// Convert to a header value.
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![
            format!("{}={}", self.name, self.value),
        ];
        
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        
        if self.secure {
            parts.push("Secure".to_string());
        }
        
        match self.same_site {
            SameSite::None => parts.push("SameSite=None".to_string()),
            SameSite::Lax => parts.push("SameSite=Lax".to_string()),
            SameSite::Strict => parts.push("SameSite=Strict".to_string()),
        }
        
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age.as_secs()));
        }
        
        parts.join("; ")
    }
}

/// Same site attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// None
    None,
    
    /// Lax
    Lax,
    
    /// Strict
    Strict,
}
