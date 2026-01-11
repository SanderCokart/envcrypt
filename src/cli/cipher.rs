//! Cipher selection and instantiation utilities.

use crate::cipher::{Cipher, Aes256Cbc};

#[cfg(feature = "cipher")]
use crate::cipher::{Aes256Gcm, ChaCha20Poly1305};

/// Returns a list of all supported cipher names.
///
/// The list respects feature flags, so it only includes ciphers that are
/// available in the current build configuration.
///
/// # Returns
///
/// Returns a vector of cipher name strings. The first cipher in the list
/// is the default cipher.
pub fn get_supported_ciphers() -> Vec<&'static str> {
    let mut ciphers = vec!["AES-256-CBC"];
    
    #[cfg(feature = "cipher")]
    {
        ciphers.push("AES-256-GCM");
        ciphers.push("CHACHA20-POLY1305");
    }
    
    ciphers
}

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
/// Returns an error string if the cipher name is not recognized. Supported ciphers:
/// - AES-256-CBC (default)
/// - AES-256-GCM
/// - CHACHA20-POLY1305
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
        #[cfg(feature = "cipher")]
        "AES-256-GCM" => Ok(Box::new(Aes256Gcm)),
        #[cfg(feature = "cipher")]
        "CHACHA20-POLY1305" => Ok(Box::new(ChaCha20Poly1305)),
        _ => {
            #[cfg(feature = "cipher")]
            {
                Err(format!("Unsupported cipher: {}. Supported ciphers: AES-256-CBC, AES-256-GCM, CHACHA20-POLY1305", cipher_name))
            }
            #[cfg(not(feature = "cipher"))]
            {
                Err(format!("Unsupported cipher: {}. Supported ciphers: AES-256-CBC", cipher_name))
            }
        }
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

    #[cfg(feature = "cipher")]
    #[test]
    fn test_get_cipher_aes256gcm() {
        let result = get_cipher("AES-256-GCM");
        assert!(result.is_ok());
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn test_get_cipher_aes256gcm_lowercase() {
        let result = get_cipher("aes-256-gcm");
        assert!(result.is_ok());
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn test_get_cipher_chacha20poly1305() {
        let result = get_cipher("CHACHA20-POLY1305");
        assert!(result.is_ok());
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn test_get_cipher_chacha20poly1305_no_dash_fails() {
        let result = get_cipher("CHACHA20POLY1305");
        assert!(result.is_err());
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn test_get_cipher_chacha20poly1305_lowercase() {
        let result = get_cipher("chacha20-poly1305");
        assert!(result.is_ok());
    }
}
