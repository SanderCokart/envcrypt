//! Cipher selection and instantiation utilities.

use crate::cipher::{Cipher, Aes256Cbc};

/// Creates a cipher instance from a cipher name string.
///
/// This function maps cipher name strings to their corresponding [`Cipher`] implementations.
/// The cipher name is case-insensitive.
///
/// # Arguments
///
/// * `cipher_name` - The name of the cipher (e.g., "AES-256-CBC"). Case-insensitive.
///
/// # Returns
///
/// Returns a boxed trait object implementing [`Cipher`], or an error if the cipher
/// name is not supported.
///
/// # Errors
///
/// Returns an error string if the cipher name is not recognized. Currently, only
/// "AES-256-CBC" is supported.
///
/// # Example
///
/// ```no_run
/// use envcrypt::cli::get_cipher;
///
/// let cipher = get_cipher("AES-256-CBC")?;
/// // Use cipher for encryption/decryption...
/// # Ok::<(), String>(())
/// ```
pub fn get_cipher(cipher_name: &str) -> Result<Box<dyn Cipher>, String> {
    match cipher_name.to_uppercase().as_str() {
        "AES-256-CBC" => Ok(Box::new(Aes256Cbc)),
        _ => Err(format!("Unsupported cipher: {}. Supported ciphers: AES-256-CBC", cipher_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
