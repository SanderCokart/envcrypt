mod common;

use common::*;
use std::fs;

#[test]
fn test_encrypt_creates_encrypted_file() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with key
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Check that .env.encrypted was created
    assert!(encrypted_path.exists(), ".env.encrypted file should be created");

    // Check that the encrypted content is different from plaintext
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, env_content, "Encrypted content should differ from plaintext");
    assert!(!encrypted_content.is_empty(), "Encrypted content should not be empty");
}

#[test]
fn test_default_cipher() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");

    let original_content = "TEST=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with default cipher (should be AES-256-CBC)
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Decrypt with default cipher
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}
