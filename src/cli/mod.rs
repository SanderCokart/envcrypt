//! Command-line interface for encrypting and decrypting environment files.
//!
//! This module provides the CLI interface for the `envcrypt` tool, including
//! command parsing, file operations, and user interaction for key management.
//!
//! # Usage
//!
//! The CLI is typically invoked through the [`run()`] function with command-line arguments.

use clap::{Parser, Subcommand};

mod encrypt;
mod decrypt;
mod paths;
mod key_handling;
mod cipher;

// Re-export public APIs
pub use paths::derive_output_path;
pub use key_handling::strip_base64_prefix;
pub use cipher::get_cipher;
pub use encrypt::encrypt_env;
pub use decrypt::decrypt_env;

// Internal use
use paths::{resolve_encrypt_input_path, resolve_encrypt_output_path, resolve_decrypt_input};
use key_handling::get_key_arg;

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

/// Main entry point for the CLI application.
///
/// Parses command-line arguments and executes the appropriate command (encrypt or decrypt).
///
/// # Arguments
///
/// * `args` - An iterator of command-line argument strings. Typically `std::env::args()`.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `Err` with an error message on failure.
///
/// # Errors
///
/// This function can return errors for various reasons:
/// - Invalid command-line arguments
/// - File I/O errors (file not found, permission denied, etc.)
/// - Encryption/decryption failures
/// - Key derivation errors
///
/// # Example
///
/// ```no_run
/// use envcrypt::cli::run;
///
/// // Run with command-line arguments
/// if let Err(e) = run(std::env::args()) {
///     eprintln!("Error: {}", e);
///     std::process::exit(1);
/// }
/// ```
pub fn run<I>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = String>,
{
    let cli = Cli::parse_from(args);

    match cli.command {
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
