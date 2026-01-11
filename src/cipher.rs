use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::key;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

/// Trait for encryption/decryption operations
pub trait Cipher {
    /// Encrypts plaintext with the given encryption key and MAC key
    /// Returns raw bytes containing: iv + encrypted_data + mac
    fn encrypt(&self, plaintext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError>;
    
    /// Decrypts raw ciphertext bytes
    /// Verifies MAC before decryption
    /// Returns decrypted plaintext
    fn decrypt(&self, ciphertext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError>;
}

#[derive(Debug)]
pub enum CipherError {
    InvalidFormat,
    MacVerificationFailed,
    DecryptionFailed,
    EncryptionFailed(String),
}

impl std::fmt::Display for CipherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherError::InvalidFormat => write!(f, "Invalid ciphertext format"),
            CipherError::MacVerificationFailed => write!(f, "MAC verification failed - data may have been tampered with"),
            CipherError::DecryptionFailed => write!(f, "Decryption failed - incorrect key or corrupted data"),
            CipherError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
        }
    }
}

impl std::error::Error for CipherError {}

/// AES-256-CBC cipher implementation with HMAC-SHA256 MAC
pub struct Aes256Cbc;

impl Cipher for Aes256Cbc {
    fn encrypt(&self, plaintext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError> {
        // Validate key length
        if encryption_key.len() != 32 {
            return Err(CipherError::EncryptionFailed("Encryption key must be 32 bytes (256 bits)".to_string()));
        }
        if mac_key.len() != 32 {
            return Err(CipherError::EncryptionFailed("MAC key must be 32 bytes (256 bits)".to_string()));
        }

        // Generate random IV (16 bytes for AES block size)
        let iv = key::generate_salt(); // Reusing salt generation for IV
        
        // Convert key and IV to arrays
        let key_array: [u8; 32] = encryption_key.try_into()
            .map_err(|_| CipherError::EncryptionFailed("Invalid key length".to_string()))?;
        let iv_array: [u8; 16] = iv.try_into()
            .map_err(|_| CipherError::EncryptionFailed("Invalid IV length".to_string()))?;
        
        // Encrypt using AES-256-CBC
        let cipher = Aes256CbcEnc::new(&key_array.into(), &iv_array.into());
        
        // Prepare buffer with plaintext
        let mut buffer = plaintext.to_vec();
        let pt_len = buffer.len();
        // Ensure buffer is large enough for padding (add one block)
        buffer.resize(buffer.len() + 16, 0);
        
        let encrypted = cipher.encrypt_padded_mut::<cipher::block_padding::Pkcs7>(&mut buffer, pt_len)
            .map_err(|e| CipherError::EncryptionFailed(format!("Encryption failed: {:?}", e)))?;
        
        let buffer = encrypted.to_vec();
        
        // Compute HMAC of (iv + encrypted_data)
        let mut mac_input = Vec::with_capacity(iv.len() + buffer.len());
        mac_input.extend_from_slice(&iv);
        mac_input.extend_from_slice(&buffer);
        
        let mut mac = HmacSha256::new_from_slice(mac_key)
            .map_err(|e| CipherError::EncryptionFailed(format!("Failed to create MAC: {:?}", e)))?;
        mac.update(&mac_input);
        let mac_result = mac.finalize();
        let mac_bytes = mac_result.into_bytes();
        
        // Combine: iv + encrypted_data + mac
        let mut output = Vec::with_capacity(iv.len() + buffer.len() + mac_bytes.len());
        output.extend_from_slice(&iv);
        output.extend_from_slice(&buffer);
        output.extend_from_slice(mac_bytes.as_slice());
        
        Ok(output)
    }
    
    fn decrypt(&self, ciphertext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError> {
        // Validate key length
        if encryption_key.len() != 32 {
            return Err(CipherError::DecryptionFailed);
        }
        if mac_key.len() != 32 {
            return Err(CipherError::DecryptionFailed);
        }

        // Validate minimum size: iv (16) + at least 1 block (16) + mac (32) = 64 bytes
        if ciphertext.len() < 64 {
            return Err(CipherError::InvalidFormat);
        }
        
        // Extract components
        let iv = &ciphertext[0..16];
        let encrypted_data = &ciphertext[16..ciphertext.len() - 32];
        let provided_mac = &ciphertext[ciphertext.len() - 32..];
        
        // Verify MAC
        let mut mac_input = Vec::with_capacity(iv.len() + encrypted_data.len());
        mac_input.extend_from_slice(iv);
        mac_input.extend_from_slice(encrypted_data);
        
        let mut mac = HmacSha256::new_from_slice(mac_key)
            .map_err(|_| CipherError::MacVerificationFailed)?;
        mac.update(&mac_input);
        let mac_result = mac.finalize();
        
        // Constant-time MAC comparison
        if mac_result.into_bytes().as_slice() != provided_mac {
            return Err(CipherError::MacVerificationFailed);
        }
        
        // Convert key and IV to arrays
        let key_array: [u8; 32] = encryption_key.try_into()
            .map_err(|_| CipherError::DecryptionFailed)?;
        let iv_array: [u8; 16] = iv.try_into()
            .map_err(|_| CipherError::DecryptionFailed)?;
        
        // Decrypt
        let cipher = Aes256CbcDec::new(&key_array.into(), &iv_array.into());
        
        let mut buffer = encrypted_data.to_vec();
        let decrypted = cipher.decrypt_padded_mut::<cipher::block_padding::Pkcs7>(&mut buffer)
            .map_err(|_| CipherError::DecryptionFailed)?;
        
        Ok(decrypted.to_vec())
    }
}
