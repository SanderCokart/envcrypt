use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Read the Cargo.toml file
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    // Simple string parsing to find release-date
    // Look for: release-date = "YYYY-MM-DD"
    for line in cargo_toml_content.lines() {
        let line = line.trim();
        if line.starts_with("release-date") {
            // Extract the date value
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start {
                        let date = &line[start + 1..end];
                        println!("cargo:rustc-env=RELEASE_DATE={}", date);
                        return;
                    }
                }
            }
        }
    }
    
    panic!("release-date not found in [package.metadata] in Cargo.toml");
}
