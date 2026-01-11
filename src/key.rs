use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000;
const ENCRYPTION_KEY_LEN: usize = 32; // 256 bits for AES-256
const MAC_KEY_LEN: usize = 32; // 256 bits for HMAC-SHA256
const DERIVED_KEY_LEN: usize = ENCRYPTION_KEY_LEN + MAC_KEY_LEN;

/// Derives encryption and MAC keys from a user-provided key string
/// 
/// Uses PBKDF2 with SHA-256, 100,000 iterations, and a random salt.
/// Returns the salt (to be stored with encrypted data), encryption key, and MAC key.
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
    
    (encryption_key, mac_key)
}

/// Generates a random 16-byte salt for key derivation
pub fn generate_salt() -> [u8; 16] {
    use rand::RngCore;
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}
