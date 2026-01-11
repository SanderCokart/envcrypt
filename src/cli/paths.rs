//! Path resolution utilities for encryption and decryption operations.

use std::path::PathBuf;

/// Derives the output file path from an input path based on the operation type.
///
/// This function automatically determines the output filename based on the input filename
/// and whether encryption or decryption is being performed.
///
/// # Arguments
///
/// * `input_path` - The path to the input file
/// * `is_encrypt` - `true` for encryption, `false` for decryption
///
/// # Returns
///
/// Returns the derived output path:
/// - For encryption: appends `.encrypted` to the input path (e.g., `.env` → `.env.encrypted`)
/// - For decryption: removes `.encrypted` suffix (e.g., `.env.encrypted` → `.env`)
///
/// # Behavior
///
/// **Encryption:**
/// - `.env` → `.env.encrypted`
/// - `.env.local` → `.env.local.encrypted`
/// - `custom/path/file` → `custom/path/file.encrypted`
///
/// **Decryption:**
/// - `.env.encrypted` → `.env`
/// - `.env.local.encrypted` → `.env.local`
/// - `file.encrypted` → `file`
///
/// # Example
///
/// ```
/// use envcrypt::cli::derive_output_path;
///
/// assert_eq!(derive_output_path(".env", true), ".env.encrypted");
/// assert_eq!(derive_output_path(".env.encrypted", false), ".env");
/// ```
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

/// Resolves the input path for encryption operations.
pub fn resolve_encrypt_input_path(input: &Option<String>, env: &Option<String>) -> String {
    if let Some(input) = input {
        return input.clone();
    }
    
    if let Some(env_name) = env {
        return format!(".env.{}", env_name);
    }
    
    ".env".to_string()
}

/// Resolves the output path for encryption operations.
pub fn resolve_encrypt_output_path(input_path: &str, env: &Option<String>) -> String {
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

/// Resolves the input path for decryption operations.
pub fn resolve_decrypt_input(input: String) -> String {
    input
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
}
