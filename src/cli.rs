use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use base64::Engine;

use crate::cipher::{Cipher, Aes256Cbc, CipherError};
use crate::key::{derive_keys, generate_salt};

#[derive(Parser)]
#[command(name = "envcrypt")]
#[command(about = "Encrypt and decrypt environment files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encrypt a .env file to .env.encrypted
    #[cfg(feature = "encrypt")]
    Encrypt {
        /// Cipher to use for encryption (default: AES-256-CBC)
        #[arg(long, default_value = "AES-256-CBC")]
        cipher: String,
        /// Encryption key (will prompt if not provided)
        #[cfg_attr(feature = "key-flag", arg(long))]
        #[cfg_attr(not(feature = "key-flag"), arg(skip))]
        key: Option<String>,
        /// Input .env file path (default: .env, or .env.{env} if --env is specified)
        #[cfg_attr(feature = "input-flag", arg(long))]
        #[cfg_attr(not(feature = "input-flag"), arg(skip))]
        input: Option<String>,
        /// Environment name (e.g., local, production, development). When specified, defaults input to .env.{env} and output to .env.{env}.encrypted
        #[cfg_attr(feature = "env-flag", arg(long))]
        #[cfg_attr(not(feature = "env-flag"), arg(skip))]
        env: Option<String>,
    },
    /// Decrypt a .env.encrypted file to .env
    #[cfg(feature = "decrypt")]
    Decrypt {
        /// Cipher to use for decryption (default: AES-256-CBC)
        #[arg(long, default_value = "AES-256-CBC")]
        cipher: String,
        /// Decryption key (will prompt if not provided)
        #[cfg_attr(feature = "key-flag", arg(long))]
        #[cfg_attr(not(feature = "key-flag"), arg(skip))]
        key: Option<String>,
        /// Input .env.encrypted file path (default: .env.encrypted)
        #[cfg_attr(feature = "input-flag", arg(long, default_value = ".env.encrypted"))]
        #[cfg_attr(not(feature = "input-flag"), arg(skip))]
        input: String,
    },
}

// Helper functions to handle conditional feature logic
#[cfg(feature = "encrypt")]
fn resolve_encrypt_input_path(input: &Option<String>, env: &Option<String>) -> String {
    #[cfg(feature = "input-flag")]
    if let Some(input) = input {
        return input.clone();
    }
    
    #[cfg(feature = "env-flag")]
    if let Some(env_name) = env {
        return format!(".env.{}", env_name);
    }
    
    ".env".to_string()
}

#[cfg(feature = "encrypt")]
fn resolve_encrypt_output_path(input_path: &str, env: &Option<String>) -> String {
    #[cfg(feature = "env-flag")]
    if let Some(env_name) = env {
        let input_path_buf = PathBuf::from(input_path);
        let output_filename = format!(".env.{}.encrypted", env_name);
        
        if let Some(parent_dir) = input_path_buf.parent() {
            return parent_dir.join(&output_filename).to_string_lossy().to_string();
        } else {
            return output_filename;
        }
    }
    
    derive_output_path(input_path, true)
}

fn get_key_arg(key: &Option<String>) -> Option<&str> {
    #[cfg(feature = "key-flag")]
    return key.as_deref();
    
    #[cfg(not(feature = "key-flag"))]
    None
}

#[cfg(feature = "decrypt")]
fn resolve_decrypt_input(input: String) -> String {
    #[cfg(feature = "input-flag")]
    {
        input
    }
    #[cfg(not(feature = "input-flag"))]
    {
        ".env.encrypted".to_string()
    }
}

pub fn run<I>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = String>,
{
    let cli = Cli::parse_from(args);

    match cli.command {
        #[cfg(feature = "encrypt")]
        Commands::Encrypt { cipher, key, input, env } => {
            let input_path = resolve_encrypt_input_path(&input, &env);
            let output = resolve_encrypt_output_path(&input_path, &env);
            let key_arg = get_key_arg(&key);
            
            match encrypt_env(&cipher, key_arg, &input_path, &output) {
                Ok(used_key) => {
                    println!("\n⚠️  IMPORTANT: Store this encryption key in a safe place!");
                    println!("   You will need it to decrypt your .env file later.");
                    println!("\n   Encryption key: base64:{}", used_key);
                    println!("\n   This key will not be shown again. Make sure to save it securely.");
                    Ok(())
                }
                Err(e) => {
                    anyhow::bail!("{}", e);
                }
            }
        }
        #[cfg(feature = "decrypt")]
        Commands::Decrypt { cipher, key, input } => {
            let input = resolve_decrypt_input(input);
            let output = derive_output_path(&input, false);
            let key_arg = get_key_arg(&key);
            
            decrypt_env(&cipher, key_arg, &input, &output)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(())
        }
    }
}

#[cfg(all(feature = "encrypt", not(feature = "key-flag")))]
enum KeyChoice {
    GenerateNew,
    UseCustom,
}

#[cfg(all(feature = "encrypt", not(feature = "key-flag")))]
fn show_key_menu() -> Result<KeyChoice, String> {
    use std::io::{self, Write};
    
    println!("\nSelect encryption key option:");
    println!("  1) Generate a new key (default)");
    println!("  2) Use a custom key");
    print!("\nEnter choice [1]: ");
    io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    
    let choice = input.trim();
    match choice {
        "2" => Ok(KeyChoice::UseCustom),
        "1" | "" => Ok(KeyChoice::GenerateNew),
        _ => {
            println!("Invalid choice, defaulting to generate new key");
            Ok(KeyChoice::GenerateNew)
        }
    }
}

#[cfg(feature = "encrypt")]
fn generate_base64_key() -> String {
    use rand::RngCore;
    // Generate 32 random bytes (256 bits) and encode as base64
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    base64::engine::general_purpose::STANDARD.encode(&key_bytes)
}

pub fn strip_base64_prefix(key: &str) -> &str {
    key.strip_prefix("base64:").unwrap_or(key)
}

#[cfg(feature = "encrypt")]
fn get_encrypt_key_with_menu() -> Result<String, String> {
    #[cfg(not(feature = "key-flag"))]
    {
        match show_key_menu()? {
            KeyChoice::GenerateNew => {
                let key = generate_base64_key();
                println!("\nGenerated new encryption key");
                Ok(key)
            }
            KeyChoice::UseCustom => {
                print!("Enter encryption key: ");
                use std::io::Write;
                std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
                
                let key = rpassword::read_password()
                    .map_err(|e| format!("Failed to read password: {}", e))?;
                Ok(strip_base64_prefix(key.trim()).to_string())
            }
        }
    }
    #[cfg(feature = "key-flag")]
    {
        let key = generate_base64_key();
        println!("\nGenerated new encryption key");
        Ok(key)
    }
}

fn get_decrypt_key() -> Result<String, String> {
    print!("Enter decryption key: ");
    use std::io::Write;
    std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
    
    let key = rpassword::read_password()
        .map_err(|e| format!("Failed to read password: {}", e))?;
    Ok(strip_base64_prefix(key.trim()).to_string())
}

fn get_encryption_key(key_arg: Option<&str>, is_encrypt: bool) -> Result<String, String> {
    // If key was provided via flag, use it
    #[cfg(feature = "key-flag")]
    if let Some(key) = key_arg {
        return Ok(strip_base64_prefix(key.trim()).to_string());
    }
    
    if is_encrypt {
        #[cfg(feature = "encrypt")]
        {
            get_encrypt_key_with_menu()
        }
        #[cfg(not(feature = "encrypt"))]
        {
            unreachable!("encrypt feature required")
        }
    } else {
        get_decrypt_key()
    }
}

pub fn get_cipher(cipher_name: &str) -> Result<Box<dyn Cipher>, String> {
    match cipher_name.to_uppercase().as_str() {
        "AES-256-CBC" => Ok(Box::new(Aes256Cbc)),
        _ => Err(format!("Unsupported cipher: {}. Supported ciphers: AES-256-CBC", cipher_name)),
    }
}

pub fn derive_output_path(input_path: &str, is_encrypt: bool) -> String {
    if is_encrypt {
        // For encryption: .env -> .env.encrypted, .env.{env} -> .env.{env}.encrypted
        if input_path.ends_with(".encrypted") {
            // Already encrypted, just return as-is (shouldn't happen normally)
            input_path.to_string()
        } else if input_path == ".env" {
            // Simple .env case
            ".env.encrypted".to_string()
        } else if input_path.starts_with(".env.") {
            // .env.{something} case - append .encrypted
            format!("{}.encrypted", input_path)
        } else {
            // Other paths - append .encrypted
            format!("{}.encrypted", input_path)
        }
    } else {
        // For decryption: .env.encrypted -> .env, .env.{env}.encrypted -> .env.{env}
        if input_path.ends_with(".env.encrypted") {
            input_path.replace(".env.encrypted", ".env")
        } else if input_path.ends_with(".encrypted") {
            input_path.strip_suffix(".encrypted").unwrap_or(input_path).to_string()
        } else {
            // Fallback: return input as-is (shouldn't happen in normal usage)
            input_path.to_string()
        }
    }
}

#[cfg(feature = "encrypt")]
pub fn encrypt_env(cipher_name: &str, key_arg: Option<&str>, input_path: &str, output_path: &str) -> Result<String, String> {
    let env_path = Path::new(input_path);
    let encrypted_path = Path::new(output_path);

    if !env_path.exists() {
        return Err(format!("{} file not found", input_path));
    }

    // Get encryption key
    let key_input = get_encryption_key(key_arg, true)?;
    
    // Get cipher
    let cipher = get_cipher(cipher_name)?;
    
    // Read plaintext
    let plaintext = fs::read_to_string(env_path)
        .map_err(|e| format!("Error reading {} file: {}", input_path, e))?;
    
    // Generate salt for key derivation
    let salt = generate_salt();
    
    // Derive keys
    let (encryption_key, mac_key) = derive_keys(&key_input, &salt);
    
    // Encrypt (returns: iv + encrypted_data + mac)
    let encrypted = cipher.encrypt(plaintext.as_bytes(), &encryption_key, &mac_key)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Store salt + encrypted data
    // Format: base64(salt + iv + encrypted_data + mac)
    let mut output = Vec::with_capacity(salt.len() + encrypted.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&encrypted);
    let final_output = base64::engine::general_purpose::STANDARD.encode(&output);
    
    // Write encrypted file
    fs::write(encrypted_path, final_output)
        .map_err(|e| format!("Error writing {}: {}", output_path, e))?;
    
    println!("\nSuccessfully encrypted {} to {}", input_path, output_path);
    Ok(key_input)
}

#[cfg(feature = "decrypt")]
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
    let (encryption_key, mac_key) = derive_keys(&key_input, &salt);
    
    // Decrypt (encrypted_data contains: iv + encrypted_data + mac)
    let plaintext = cipher.decrypt(encrypted_data, &encryption_key, &mac_key)
        .map_err(|e| {
            match e {
                CipherError::MacVerificationFailed => "MAC verification failed - the encrypted file may have been tampered with or the key is incorrect".to_string(),
                CipherError::DecryptionFailed => "Decryption failed - incorrect key or corrupted data".to_string(),
                _ => format!("Decryption error: {}", e),
            }
        })?;
    
    let plaintext_str = String::from_utf8(plaintext)
        .map_err(|e| format!("Decrypted data is not valid UTF-8: {}", e))?;
    
    // Write decrypted file
    fs::write(env_path, plaintext_str)
        .map_err(|e| format!("Error writing {}: {}", output_path, e))?;
    
    println!("Successfully decrypted {} to {}", input_path, output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_output_path_encrypt_simple_env() {
        assert_eq!(derive_output_path(".env", true), ".env.encrypted");
    }

    #[test]
    fn test_derive_output_path_encrypt_env_local() {
        assert_eq!(derive_output_path(".env.local", true), ".env.local.encrypted");
    }

    #[test]
    fn test_derive_output_path_encrypt_custom_path() {
        assert_eq!(derive_output_path("config/.env", true), "config/.env.encrypted");
    }

    #[test]
    fn test_derive_output_path_decrypt_simple() {
        assert_eq!(derive_output_path(".env.encrypted", false), ".env");
    }

    #[test]
    fn test_derive_output_path_decrypt_env_local() {
        assert_eq!(derive_output_path(".env.local.encrypted", false), ".env.local");
    }

    #[test]
    fn test_derive_output_path_decrypt_custom_encrypted() {
        assert_eq!(derive_output_path("file.encrypted", false), "file");
    }

    #[test]
    fn test_strip_base64_prefix_with_prefix() {
        assert_eq!(strip_base64_prefix("base64:test123"), "test123");
    }

    #[test]
    fn test_strip_base64_prefix_without_prefix() {
        assert_eq!(strip_base64_prefix("test123"), "test123");
    }

    #[test]
    fn test_strip_base64_prefix_empty() {
        assert_eq!(strip_base64_prefix(""), "");
    }

    #[test]
    fn test_get_cipher_aes256cbc() {
        let result = get_cipher("AES-256-CBC");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_cipher_aes256cbc_lowercase() {
        let result = get_cipher("aes-256-cbc");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_cipher_unsupported() {
        let result: Result<Box<dyn Cipher>, String> = get_cipher("UNSUPPORTED");
        assert!(result.is_err());
        if let Err(err_msg) = result {
            assert!(err_msg.contains("Unsupported cipher"));
        }
    }
}
