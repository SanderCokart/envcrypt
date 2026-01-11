//! Decryption command implementation.

use std::fs;
use std::path::Path;
use base64::Engine;
use zeroize::Zeroize;

use crate::cipher::CipherError;
use crate::key::derive_keys;
use crate::cli::cipher::get_cipher;
use crate::cli::key_handling::get_encryption_key;

/// Decrypts an encrypted environment file using the specified cipher and key.
///
/// This function reads an encrypted environment file, verifies its authenticity,
/// decrypts it, and writes the plaintext to the output path.
///
/// # Arguments
///
/// * `cipher_name` - Name of the cipher to use (must match the cipher used for encryption)
/// * `key_arg` - Optional decryption key. If `None`, the user will be prompted.
///               Keys can include the "base64:" prefix which will be stripped.
/// * `input_path` - Path to the encrypted file (typically `.env.encrypted`)
/// * `output_path` - Path where the decrypted `.env` file will be written
///
/// # Returns
///
/// Returns `Ok(())` on successful decryption, or an error if decryption fails.
///
/// # Errors
///
/// Returns an error string if:
/// - The input file doesn't exist
/// - File I/O operations fail
/// - The cipher name is unsupported
/// - The encrypted file format is invalid
/// - MAC verification fails (indicating tampering or incorrect key)
/// - Decryption fails (incorrect key or corrupted data)
/// - The decrypted data is not valid UTF-8
///
/// # Security
///
/// - MAC verification is performed before decryption (authenticate-then-decrypt)
/// - MAC comparison is performed in constant time to prevent timing attacks
/// - Derived keys are automatically zeroized after use
///
/// # File Format
///
/// Expects the encrypted file to contain base64-encoded data with the format:
/// `base64([Salt (16 bytes)][IV (16 bytes)][Encrypted Data][MAC (32 bytes)])`
///
/// # Example
///
/// ```no_run
/// use envcrypt::cli::decrypt_env;
///
/// decrypt_env("AES-256-CBC", Some("my-key"), ".env.encrypted", ".env")?;
/// # Ok::<(), String>(())
/// ```
pub fn decrypt_env(cipher_name: &str, key_arg: Option<&str>, input_path: &str, output_path: &str) -> Result<(), String> {
    let encrypted_path = Path::new(input_path);
    let env_path = Path::new(output_path);

    if !encrypted_path.exists() {
        return Err(format!("{} file not found", input_path));
    }

    // Get decryption key
    let key_input = get_encryption_key(key_arg, false)?;
    
    // Get cipher
    let cipher = get_cipher(cipher_name)?;
    
    // Read encrypted file
    let encrypted_content = fs::read_to_string(encrypted_path)
        .map_err(|e| format!("Error reading {} file: {}", input_path, e))?;
    
    // Decode base64
    let data = base64::engine::general_purpose::STANDARD.decode(encrypted_content.trim())
        .map_err(|e| format!("Invalid base64 in encrypted file: {}", e))?;
    
    // Extract salt (first 16 bytes) and encrypted data (iv + encrypted_data + mac)
    if data.len() < 16 {
        return Err("Invalid encrypted file format".to_string());
    }
    
    let salt: [u8; 16] = data[0..16].try_into()
        .map_err(|_| "Invalid salt in encrypted file".to_string())?;
    let encrypted_data = &data[16..];
    
    // Derive keys using the stored salt
    let (mut encryption_key, mut mac_key) = derive_keys(&key_input, &salt);
    
    // Decrypt (encrypted_data contains: iv + encrypted_data + mac)
    let plaintext = cipher.decrypt(encrypted_data, &encryption_key, &mac_key)
        .map_err(|e| {
            // Zeroize keys on error
            encryption_key.zeroize();
            mac_key.zeroize();
            match e {
                CipherError::MacVerificationFailed => "MAC verification failed - the encrypted file may have been tampered with or the key is incorrect".to_string(),
                CipherError::DecryptionFailed => "Decryption failed - incorrect key or corrupted data".to_string(),
                _ => format!("Decryption error: {}", e),
            }
        })?;
    
    // Zeroize keys after use
    encryption_key.zeroize();
    mac_key.zeroize();
    
    let plaintext_str = String::from_utf8(plaintext)
        .map_err(|e| format!("Decrypted data is not valid UTF-8: {}", e))?;
    
    // Write decrypted file
    fs::write(env_path, plaintext_str)
        .map_err(|e| format!("Error writing {}: {}", output_path, e))?;
    
    println!("Successfully decrypted {} to {}", input_path, output_path);
    Ok(())
}
