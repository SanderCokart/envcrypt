use envcrypt::cli;

fn main() {
    if let Err(e) = cli::run(std::env::args()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
