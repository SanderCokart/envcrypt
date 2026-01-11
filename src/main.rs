mod cipher;
mod key;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use base64::Engine;

use cipher::{Cipher, Aes256Cbc, CipherError};
use key::{derive_keys, generate_salt};

#[derive(Parser)]
#[command(name = "envcrypt")]
#[command(about = "Encrypt and decrypt environment files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt a .env file to .env.encrypted
    Encrypt {
        /// Cipher to use for encryption (default: AES-256-CBC)
        #[arg(long, default_value = "AES-256-CBC")]
        cipher: String,
        /// Encryption key (will prompt if not provided)
        #[arg(long)]
        key: Option<String>,
        /// Input .env file path (default: .env, or .env.{env} if --env is specified)
        #[arg(long)]
        input: Option<String>,
        /// Environment name (e.g., local, production, development). When specified, defaults input to .env.{env} and output to .env.{env}.encrypted
        #[arg(long)]
        env: Option<String>,
    },
    /// Decrypt a .env.encrypted file to .env
    Decrypt {
        /// Cipher to use for decryption (default: AES-256-CBC)
        #[arg(long, default_value = "AES-256-CBC")]
        cipher: String,
        /// Decryption key (will prompt if not provided)
        #[arg(long)]
        key: Option<String>,
        /// Input .env.encrypted file path (default: .env.encrypted)
        #[arg(long, default_value = ".env.encrypted")]
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt { cipher, key, input, env } => {
            // Determine input path: use explicit input if provided, otherwise .env.{env} or .env
            let input_path = if let Some(input) = input {
                input
            } else if let Some(env_name) = &env {
                format!(".env.{}", env_name)
            } else {
                ".env".to_string()
            };
            
            // Determine output path: if --env is provided, use .env.{env}.encrypted in the same directory as input
            // Otherwise, derive from input path
            let output = if let Some(env_name) = &env {
                let input_path_buf = PathBuf::from(&input_path);
                let parent = input_path_buf.parent();
                let output_filename = format!(".env.{}.encrypted", env_name);
                
                if let Some(parent_dir) = parent {
                    // Input has a directory component, preserve it
                    parent_dir.join(&output_filename).to_string_lossy().to_string()
                } else {
                    // Input is just a filename, output in current directory
                    output_filename
                }
            } else {
                derive_output_path(&input_path, true)
            };
            match encrypt_env(&cipher, key.as_deref(), &input_path, &output) {
                Ok(used_key) => {
                    println!("\n⚠️  IMPORTANT: Store this encryption key in a safe place!");
                    println!("   You will need it to decrypt your .env file later.");
                    println!("\n   Encryption key: base64:{}", used_key);
                    println!("\n   This key will not be shown again. Make sure to save it securely.");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Decrypt { cipher, key, input } => {
            let output = derive_output_path(&input, false);
            if let Err(e) = decrypt_env(&cipher, key.as_deref(), &input, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
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

fn strip_base64_prefix(key: &str) -> &str {
    key.strip_prefix("base64:").unwrap_or(key)
}

fn get_encryption_key(key_arg: Option<&str>, is_encrypt: bool) -> Result<String, String> {
    if let Some(key) = key_arg {
        // Strip "base64:" prefix if present and trim whitespace
        Ok(strip_base64_prefix(key.trim()).to_string())
    } else if is_encrypt {
        // Show menu for encryption
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
                // Strip "base64:" prefix and trim whitespace
                Ok(strip_base64_prefix(key.trim()).to_string())
            }
        }
    } else {
        // For decryption, just prompt for key
        print!("Enter decryption key: ");
        use std::io::Write;
        std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
        
        let key = rpassword::read_password()
            .map_err(|e| format!("Failed to read password: {}", e))?;
        // Strip "base64:" prefix and trim whitespace
        Ok(strip_base64_prefix(key.trim()).to_string())
    }
}

fn get_cipher(cipher_name: &str) -> Result<Box<dyn Cipher>, String> {
    match cipher_name.to_uppercase().as_str() {
        "AES-256-CBC" => Ok(Box::new(Aes256Cbc)),
        _ => Err(format!("Unsupported cipher: {}. Supported ciphers: AES-256-CBC", cipher_name)),
    }
}

fn derive_output_path(input_path: &str, is_encrypt: bool) -> String {
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

fn encrypt_env(cipher_name: &str, key_arg: Option<&str>, input_path: &str, output_path: &str) -> Result<String, String> {
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

fn decrypt_env(cipher_name: &str, key_arg: Option<&str>, input_path: &str, output_path: &str) -> Result<(), String> {
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
