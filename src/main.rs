#[cfg(any(feature = "encrypt", feature = "decrypt"))]
use envcrypt::cli;

#[cfg(any(feature = "encrypt", feature = "decrypt"))]
fn main() {
    if let Err(e) = cli::run(std::env::args()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(any(feature = "encrypt", feature = "decrypt")))]
fn main() {
    eprintln!("Error: At least one of 'encrypt' or 'decrypt' features is required for the binary");
    std::process::exit(1);
}
