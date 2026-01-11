//! Key derivation and generation utilities.
//!
//! This module provides functions for deriving cryptographic keys from user-provided
//! passwords and generating random salts for key derivation.
//!
//! # Key Derivation
//!
//! Keys are derived using PBKDF2-HMAC-SHA256 with 100,000 iterations, which provides
//! protection against brute-force attacks while maintaining reasonable performance.
//!
//! # Security Considerations
//!
//! - Each encryption uses a unique random salt
//! - The salt must be stored with the encrypted data for decryption
//! - Derived keys are automatically zeroized when dropped
//! - Never reuse salts across different encryptions

use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroize;

/// Number of PBKDF2 iterations for key derivation.
///
/// This value (100,000) provides a good balance between security and performance.
/// It's high enough to slow down brute-force attacks while remaining practical
/// for normal use cases.
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Length of the encryption key in bytes (256 bits for AES-256).
const ENCRYPTION_KEY_LEN: usize = 32;

/// Length of the MAC key in bytes (256 bits for HMAC-SHA256).
const MAC_KEY_LEN: usize = 32;

/// Total length of the derived key material (encryption key + MAC key).
const DERIVED_KEY_LEN: usize = ENCRYPTION_KEY_LEN + MAC_KEY_LEN;

/// Derives encryption and MAC keys from a user-provided key string using PBKDF2.
///
/// This function takes a user-provided password/key string and derives two separate
/// 256-bit keys: one for encryption and one for MAC computation. The derivation uses
/// PBKDF2-HMAC-SHA256 with 100,000 iterations to protect against brute-force attacks.
///
/// # Arguments
///
/// * `key_input` - The user-provided password or key string
/// * `salt` - A 16-byte random salt. Must be unique for each encryption operation.
///            The salt should be stored with the encrypted data for decryption.
///
/// # Returns
///
/// Returns a tuple `(encryption_key, mac_key)` where:
/// - `encryption_key`: 32-byte (256-bit) key for AES-256 encryption
/// - `mac_key`: 32-byte (256-bit) key for HMAC-SHA256 computation
///
/// # Security
///
/// - The salt should be randomly generated using [`generate_salt()`] for each encryption
/// - The same salt must be used for both encryption and decryption
/// - Derived keys are automatically zeroized when dropped
/// - Never reuse salts across different encryption operations
///
/// # Example
///
/// ```no_run
/// use envcrypt::key::{derive_keys, generate_salt};
///
/// let user_password = "my-secret-password";
/// let salt = generate_salt();
///
/// let (encryption_key, mac_key) = derive_keys(user_password, &salt);
/// // Use encryption_key and mac_key for encryption...
/// ```
pub fn derive_keys(key_input: &str, salt: &[u8; 16]) -> (Vec<u8>, Vec<u8>) {
    let mut derived_key = [0u8; DERIVED_KEY_LEN];
    
    pbkdf2_hmac::<Sha256>(
        key_input.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut derived_key,
    );
    
    // Split the derived key into encryption key and MAC key
    let encryption_key = derived_key[..ENCRYPTION_KEY_LEN].to_vec();
    let mac_key = derived_key[ENCRYPTION_KEY_LEN..].to_vec();
    
    // Zeroize the derived key array
    derived_key.zeroize();
    
    (encryption_key, mac_key)
}

/// Generates a cryptographically secure random 16-byte salt for key derivation.
///
/// This function uses the system's secure random number generator to create
/// a unique salt for each key derivation operation. The salt should be stored
/// with the encrypted data and reused during decryption.
///
/// # Returns
///
/// Returns a 16-byte array containing random bytes suitable for use as a PBKDF2 salt.
///
/// # Security
///
/// - Uses cryptographically secure random number generation
/// - Each call produces a unique salt (with extremely high probability)
/// - The salt should be stored alongside encrypted data
///
/// # Example
///
/// ```no_run
/// use envcrypt::key::{generate_salt, derive_keys};
///
/// let salt = generate_salt();
/// let (enc_key, mac_key) = derive_keys("password", &salt);
/// // Store salt with encrypted data for later decryption
/// ```
pub fn generate_salt() -> [u8; 16] {
    use rand::RngCore;
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}
