use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub const TEST_KEY: &str = "test-encryption-key-12345";

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// Create a subdirectory within a temp directory
#[allow(dead_code)] // Used across multiple test files (each compiled separately)
pub fn create_subdir(temp_dir: &Path, name: &str) -> PathBuf {
    let subdir = temp_dir.join(name);
    fs::create_dir(&subdir).unwrap();
    subdir
}

/// Create a command with the binary and current directory set
pub fn create_command(temp_dir: &Path) -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir);
    cmd
}

/// Create an encrypt command with key
pub fn create_encrypt_command(temp_dir: &Path, key: &str) -> Command {
    let mut cmd = create_command(temp_dir);
    cmd.arg("encrypt").arg("--key").arg(key);
    cmd
}

/// Create a decrypt command with key
pub fn create_decrypt_command(temp_dir: &Path, key: &str) -> Command {
    let mut cmd = create_command(temp_dir);
    cmd.arg("decrypt").arg("--key").arg(key);
    cmd
}
