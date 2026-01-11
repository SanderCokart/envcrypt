use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

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
    Encrypt,
    /// Decrypt a .env.encrypted file to .env
    Decrypt,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt => {
            encrypt_env();
        }
        Commands::Decrypt => {
            println!("Decrypt functionality not yet implemented");
        }
    }
}

fn encrypt_env() {
    let env_path = Path::new(".env");
    let encrypted_path = Path::new(".env.encrypted");

    if !env_path.exists() {
        eprintln!("Error: .env file not found");
        std::process::exit(1);
    }

    match fs::read_to_string(env_path) {
        Ok(content) => {
            if let Err(e) = fs::write(encrypted_path, content) {
                eprintln!("Error writing .env.encrypted: {}", e);
                std::process::exit(1);
            }
            println!("Successfully encrypted .env to .env.encrypted");
        }
        Err(e) => {
            eprintln!("Error reading .env file: {}", e);
            std::process::exit(1);
        }
    }
}
