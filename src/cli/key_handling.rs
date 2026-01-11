//! Key input and parsing utilities.

use base64::Engine;

/// Strips the optional "base64:" prefix from a key string.
///
/// This function is used to normalize key input, allowing users to provide keys
/// either with or without the "base64:" prefix. The prefix is commonly used
/// when displaying keys to users for clarity.
///
/// # Arguments
///
/// * `key` - A key string that may or may not start with "base64:"
///
/// # Returns
///
/// Returns the key string without the "base64:" prefix if present, otherwise
/// returns the original string unchanged.
///
/// # Example
///
/// ```
/// use envcrypt::cli::strip_base64_prefix;
///
/// assert_eq!(strip_base64_prefix("base64:abc123"), "abc123");
/// assert_eq!(strip_base64_prefix("abc123"), "abc123");
/// ```
pub fn strip_base64_prefix(key: &str) -> &str {
    key.strip_prefix("base64:").unwrap_or(key)
}

/// Gets the key argument from the command-line option.
pub fn get_key_arg(key: &Option<String>) -> Option<&str> {
    key.as_deref()
}

enum KeyChoice {
    GenerateNew,
    UseCustom,
}

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

fn generate_base64_key() -> String {
    use rand::RngCore;
    // Generate 32 random bytes (256 bits) and encode as base64
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    base64::engine::general_purpose::STANDARD.encode(&key_bytes)
}

fn get_encrypt_key_with_menu() -> Result<String, String> {
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

fn get_decrypt_key() -> Result<String, String> {
    print!("Enter decryption key: ");
    use std::io::Write;
    std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
    
    let key = rpassword::read_password()
        .map_err(|e| format!("Failed to read password: {}", e))?;
    Ok(strip_base64_prefix(key.trim()).to_string())
}

/// Gets the encryption/decryption key from command-line argument or prompts the user.
///
/// # Arguments
///
/// * `key_arg` - Optional key provided via command-line flag
/// * `is_encrypt` - `true` for encryption, `false` for decryption
/// * `no_interaction` - If `true`, skip interactive prompts. For encryption, auto-generate key if not provided. For decryption, error if key not provided.
///
/// # Returns
///
/// Returns the key string, or an error if key input fails.
pub fn get_encryption_key(key_arg: Option<&str>, is_encrypt: bool, no_interaction: bool) -> Result<String, String> {
    // If key was provided via flag, use it
    if let Some(key) = key_arg {
        return Ok(strip_base64_prefix(key.trim()).to_string());
    }
    
    if no_interaction {
        if is_encrypt {
            // Auto-generate new key for encryption
            let key = generate_base64_key();
            Ok(key)
        } else {
            // For decryption, cannot proceed without key
            Err("Decryption key is required when using --no-interaction. Please provide --key".to_string())
        }
    } else {
        if is_encrypt {
            get_encrypt_key_with_menu()
        } else {
            get_decrypt_key()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
