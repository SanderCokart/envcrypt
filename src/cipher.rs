//! Cryptographic cipher implementations for encryption and decryption.
//!
//! This module provides the [`Cipher`] trait and implementations for encrypting and decrypting
//! data. All implementations use authenticated encryption with HMAC for integrity verification.
//!
//! # Security Considerations
//!
//! - All MAC comparisons are performed in constant time to prevent timing attacks
//! - Keys should be zeroized after use (handled by the `zeroize` crate when enabled)
//! - Random IVs are generated for each encryption operation
//! - HMAC verification occurs before decryption to prevent padding oracle attacks
//!
//! # Example
//!
//! ```no_run
//! use envcrypt::cipher::{Cipher, Aes256Cbc};
//!
//! let cipher = Aes256Cbc;
//! let encryption_key = [0u8; 32];
//! let mac_key = [0u8; 32];
//! let plaintext = b"Hello, world!";
//!
//! // Encrypt
//! let ciphertext = cipher.encrypt(plaintext, &encryption_key, &mac_key)?;
//!
//! // Decrypt
//! let decrypted = cipher.decrypt(&ciphertext, &encryption_key, &mac_key)?;
//! # Ok::<(), envcrypt::cipher::CipherError>(())
//! ```

use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::key;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

/// Trait for encryption/decryption operations with authenticated encryption.
///
/// Implementations of this trait provide both confidentiality (encryption) and
/// authenticity (MAC verification) for data protection.
///
/// # Security Model
///
/// All implementations must:
/// - Use separate keys for encryption and MAC operations
/// - Generate random IVs for each encryption
/// - Verify MAC before decryption (authenticate-then-decrypt)
/// - Perform MAC comparisons in constant time
pub trait Cipher {
    /// Encrypts plaintext with the given encryption key and MAC key.
    ///
    /// This function performs authenticated encryption:
    /// 1. Generates a random IV
    /// 2. Encrypts the plaintext using the encryption key
    /// 3. Computes an HMAC over the IV and encrypted data
    /// 4. Returns the concatenated IV, encrypted data, and MAC
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The data to encrypt
    /// * `encryption_key` - 32-byte (256-bit) key for encryption
    /// * `mac_key` - 32-byte (256-bit) key for HMAC computation
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` containing the encrypted data in the format:
    /// `[IV (16 bytes)][Encrypted Data][MAC (32 bytes)]`
    ///
    /// # Errors
    ///
    /// Returns [`CipherError::EncryptionFailed`] if:
    /// - Key lengths are incorrect (must be 32 bytes each)
    /// - Encryption operation fails
    /// - MAC computation fails
    ///
    /// # Security
    ///
    /// The encryption key and MAC key should be derived from a user-provided password
    /// using a key derivation function like PBKDF2. Never use user passwords directly.
    fn encrypt(&self, plaintext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError>;
    
    /// Decrypts raw ciphertext bytes and verifies authenticity.
    ///
    /// This function performs authenticated decryption:
    /// 1. Extracts IV, encrypted data, and MAC from the ciphertext
    /// 2. Verifies the MAC using constant-time comparison
    /// 3. Only decrypts if MAC verification succeeds
    /// 4. Returns the decrypted plaintext
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted data in format: `[IV][Encrypted Data][MAC]`
    /// * `encryption_key` - 32-byte (256-bit) key for decryption
    /// * `mac_key` - 32-byte (256-bit) key for HMAC verification
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` containing the decrypted plaintext.
    ///
    /// # Errors
    ///
    /// Returns [`CipherError::InvalidFormat`] if the ciphertext format is invalid.
    /// Returns [`CipherError::MacVerificationFailed`] if MAC verification fails
    /// (indicating tampering or incorrect key).
    /// Returns [`CipherError::DecryptionFailed`] if decryption fails.
    ///
    /// # Security
    ///
    /// MAC verification is performed in constant time to prevent timing attacks.
    /// Decryption only occurs after successful MAC verification to prevent padding oracle attacks.
    fn decrypt(&self, ciphertext: &[u8], encryption_key: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, CipherError>;
}

/// Errors that can occur during encryption or decryption operations.
///
/// This enum represents all possible error conditions when using a [`Cipher`] implementation.
#[derive(Debug)]
pub enum CipherError {
    /// The ciphertext format is invalid (e.g., too short, missing components).
    ///
    /// This typically indicates corrupted data or an incorrect file format.
    InvalidFormat,
    
    /// MAC verification failed during decryption.
    ///
    /// This indicates either:
    /// - The encrypted data has been tampered with
    /// - An incorrect MAC key was provided
    /// - The ciphertext is corrupted
    ///
    /// **Security Note:** This error is returned after constant-time MAC comparison
    /// to prevent timing attacks.
    MacVerificationFailed,
    
    /// Decryption operation failed.
    ///
    /// This can occur due to:
    /// - Incorrect encryption key
    /// - Corrupted encrypted data
    /// - Invalid padding
    DecryptionFailed,
    
    /// Encryption operation failed.
    ///
    /// Contains a detailed error message describing the failure reason.
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

/// AES-256-CBC cipher implementation with HMAC-SHA256 authentication.
///
/// This implementation provides authenticated encryption using:
/// - **Encryption:** AES-256 in CBC mode with PKCS7 padding
/// - **Authentication:** HMAC-SHA256 computed over IV + encrypted data
/// - **IV Generation:** Random 16-byte IV for each encryption
///
/// # Security Properties
///
/// - **Confidentiality:** AES-256 provides strong encryption
/// - **Integrity:** HMAC-SHA256 prevents tampering
/// - **Authenticity:** MAC verification before decryption prevents padding oracle attacks
/// - **Timing Safety:** Constant-time MAC comparison prevents timing attacks
///
/// # Format
///
/// Encrypted output format: `[IV (16 bytes)][Encrypted Data (variable)][MAC (32 bytes)]`
///
/// # Example
///
/// ```no_run
/// use envcrypt::cipher::{Cipher, Aes256Cbc};
///
/// let cipher = Aes256Cbc;
/// let encryption_key = [0u8; 32];
/// let mac_key = [0u8; 32];
///
/// let plaintext = b"secret data";
/// let ciphertext = cipher.encrypt(plaintext, &encryption_key, &mac_key)?;
/// let decrypted = cipher.decrypt(&ciphertext, &encryption_key, &mac_key)?;
/// assert_eq!(plaintext, decrypted.as_slice());
/// # Ok::<(), envcrypt::cipher::CipherError>(())
/// ```
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
        
        // Constant-time MAC comparison to prevent timing attacks
        if mac_result.into_bytes().as_slice().ct_eq(provided_mac).unwrap_u8() == 0 {
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
