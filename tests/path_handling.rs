mod common;

use common::*;
use std::fs;

#[test]
fn test_encrypt_with_custom_input_path() {
    let temp_dir = create_temp_dir();
    let tmp_dir = create_subdir(temp_dir.path(), "tmp");
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.encrypted");

    // Create a test .env file in tmp directory
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with custom input path
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("tmp/.env");
    cmd.assert().success();

    // Check that .env.encrypted was created in tmp directory
    assert!(encrypted_path.exists(), ".env.encrypted file should be created in tmp directory");
    
    // Verify original .env file still exists
    assert!(env_path.exists(), "Original .env file should still exist");

    // Check that the encrypted content is different from plaintext
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, env_content, "Encrypted content should differ from plaintext");
    assert!(!encrypted_content.is_empty(), "Encrypted content should not be empty");
}

#[test]
fn test_decrypt_with_custom_input_path() {
    let temp_dir = create_temp_dir();
    let tmp_dir = create_subdir(temp_dir.path(), "tmp");
    
    let env_path = tmp_dir.join(".env");

    // Create and encrypt a file in tmp directory
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("tmp/.env");
    cmd.assert().success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with custom input path
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("tmp/.env.encrypted");
    cmd.assert().success();

    // Verify .env was recreated in tmp directory with correct content
    assert!(env_path.exists(), ".env file should be recreated in tmp directory");
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}
