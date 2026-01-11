//! Encryption command implementation.

use std::fs;
use std::path::Path;
use base64::Engine;
use zeroize::Zeroize;

use crate::key::{derive_keys, generate_salt};
use crate::cli::cipher::get_cipher;
use crate::cli::key_handling::get_encryption_key;
use crate::cli::output::{OutputConfig, info, verbose, debug};
// Note: resolve_encrypt_input_path and resolve_encrypt_output_path are only used in mod.rs

/// Encrypts an environment file using the specified cipher and key.
///
/// This function reads a plaintext environment file, encrypts it using the specified
/// cipher, and writes the encrypted data to the output path. The encryption uses
/// authenticated encryption with a random salt and IV for each operation.
///
/// # Arguments
///
/// * `cipher_name` - Name of the cipher to use (e.g., "AES-256-CBC")
/// * `key_arg` - Optional encryption key. If `None`, the user will be prompted (unless `no_interaction` is true).
///               Keys can include the "base64:" prefix which will be stripped.
/// * `input_path` - Path to the plaintext `.env` file to encrypt
/// * `output_path` - Path where the encrypted file will be written
/// * `output_config` - Output configuration for verbosity control
/// * `force` - If `true`, overwrite existing output file without error
/// * `prune` - If `true`, delete the original input file after successful encryption
/// * `no_interaction` - If `true`, skip interactive prompts (auto-generate key if not provided)
///
/// # Returns
///
/// Returns `Ok(key_string)` where `key_string` is the encryption key that was used
/// (for display to the user). Returns an error if encryption fails.
///
/// # Errors
///
/// Returns an error string if:
/// - The input file doesn't exist
/// - The output file exists and `force` is `false`
/// - File I/O operations fail
/// - The cipher name is unsupported
/// - Key derivation or encryption fails
///
/// # Security
///
/// - A random salt is generated for each encryption
/// - A random IV is generated for each encryption
/// - Derived keys are automatically zeroized after use
/// - The encryption key is returned for user storage (should be kept secure)
///
/// # File Format
///
/// The encrypted file contains base64-encoded data with the format:
/// `base64([Salt (16 bytes)][IV (16 bytes)][Encrypted Data][MAC (32 bytes)])`
///
/// # Example
///
/// ```no_run
/// use envcrypt::cli::{encrypt_env, output::OutputConfig};
///
/// let output_config = OutputConfig::new(false, false, 0);
/// let key = encrypt_env("AES-256-CBC", Some("my-key"), ".env", ".env.encrypted", &output_config, false, false, false)?;
/// # Ok::<(), String>(())
/// ```
pub fn encrypt_env(
    cipher_name: &str,
    key_arg: Option<&str>,
    input_path: &str,
    output_path: &str,
    output_config: &OutputConfig,
    force: bool,
    prune: bool,
    no_interaction: bool,
) -> Result<String, String> {
    let env_path = Path::new(input_path);
    let encrypted_path = Path::new(output_path);

    if !env_path.exists() {
        return Err(format!("{} file not found", input_path));
    }

    // Check if output file exists and handle --force flag
    if encrypted_path.exists() && !force {
        return Err(format!("Output file {} already exists. Use --force to overwrite.", output_path));
    }

    debug(output_config, &format!("Starting encryption: {} -> {}", input_path, output_path));
    debug(output_config, &format!("Cipher: {}", cipher_name));
    verbose(output_config, &format!("Input file: {}", input_path));
    verbose(output_config, &format!("Output file: {}", output_path));

    // Get encryption key
    let key_input = get_encryption_key(key_arg, true, no_interaction)?;
    
    // Get cipher
    let cipher = get_cipher(cipher_name)?;
    
    // Read plaintext
    let plaintext = fs::read_to_string(env_path)
        .map_err(|e| format!("Error reading {} file: {}", input_path, e))?;
    
    // Generate salt for key derivation
    let salt = generate_salt();
    
    // Derive keys
    let (mut encryption_key, mut mac_key) = derive_keys(&key_input, &salt);
    
    // Encrypt (returns: iv + encrypted_data + mac)
    let encrypted = cipher.encrypt(plaintext.as_bytes(), &encryption_key, &mac_key)
        .map_err(|e| {
            // Zeroize keys on error
            encryption_key.zeroize();
            mac_key.zeroize();
            format!("Encryption failed: {}", e)
        })?;
    
    // Zeroize keys after use
    encryption_key.zeroize();
    mac_key.zeroize();
    
    // Store salt + encrypted data
    // Format: base64(salt + iv + encrypted_data + mac)
    let mut output = Vec::with_capacity(salt.len() + encrypted.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&encrypted);
    let final_output = base64::engine::general_purpose::STANDARD.encode(&output);
    
    // Write encrypted file
    debug(output_config, "Writing encrypted data to file");
    fs::write(encrypted_path, final_output)
        .map_err(|e| format!("Error writing {}: {}", output_path, e))?;
    
    info(output_config, &format!("\nSuccessfully encrypted {} to {}", input_path, output_path));

    // Handle --prune flag: delete original file after successful encryption
    if prune {
        debug(output_config, &format!("Pruning original file: {}", input_path));
        fs::remove_file(env_path)
            .map_err(|e| format!("Error removing original file {}: {}", input_path, e))?;
        verbose(output_config, &format!("Removed original file: {}", input_path));
    }

    Ok(key_input)
}
