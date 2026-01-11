use crate::common::*;
use std::fs;

#[test]
fn test_aes256_gcm_roundtrip() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with AES-256-GCM
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("AES-256-GCM");
    cmd.assert().success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with AES-256-GCM
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("AES-256-GCM");
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_chacha20_poly1305_roundtrip() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with ChaCha20-Poly1305
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("CHACHA20-POLY1305");
    cmd.assert().success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with ChaCha20-Poly1305
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("CHACHA20-POLY1305");
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_chacha20_poly1305_no_dash() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with ChaCha20-Poly1305
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("CHACHA20-POLY1305");
    cmd.assert().success();

    assert!(encrypted_path.exists());

    // Decrypt
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("CHACHA20-POLY1305");
    cmd.assert().success();

    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_cipher_case_insensitive() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with lowercase cipher name
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("aes-256-gcm");
    cmd.assert().success();

    assert!(encrypted_path.exists());

    // Decrypt with mixed case
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("AES-256-GCM");
    cmd.assert().success();

    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_wrong_cipher_fails() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with AES-256-GCM
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("AES-256-GCM");
    cmd.assert().success();

    assert!(encrypted_path.exists());

    // Try to decrypt with wrong cipher (AES-256-CBC instead of AES-256-GCM)
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("AES-256-CBC");
    cmd.assert().failure(); // Should fail because cipher doesn't match
}

#[test]
fn test_unsupported_cipher_fails() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");

    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Try to encrypt with unsupported cipher
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--cipher").arg("UNSUPPORTED-CIPHER");
    cmd.assert().failure();
}
