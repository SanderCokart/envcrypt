use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

const TEST_KEY: &str = "test-encryption-key-12345";

#[test]
fn test_encrypt_creates_encrypted_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with key
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Check that .env.encrypted was created
    assert!(encrypted_path.exists(), ".env.encrypted file should be created");

    // Check that the encrypted content is different from plaintext
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, env_content, "Encrypted content should differ from plaintext");
    assert!(!encrypted_content.is_empty(), "Encrypted content should not be empty");
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_decrypt_with_wrong_key_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Try to decrypt with wrong key
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg("wrong-key")
        .assert()
        .failure()
        .stderr(predicates::str::contains("MAC verification failed").or(predicates::str::contains("Decryption failed")));
}

#[test]
fn test_default_cipher() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    let original_content = "TEST=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with default cipher (should be AES-256-CBC)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Decrypt with default cipher
    fs::remove_file(&env_path).unwrap();
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}
