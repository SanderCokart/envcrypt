use crate::common::*;
use std::fs;

#[test]
fn test_decrypt_with_base64_prefix() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has "base64:" prefix (simulating user copying the displayed key)
    let key_with_prefix = format!("base64:{}", TEST_KEY);
    let mut cmd = create_decrypt_command(temp_dir.path(), &key_with_prefix);
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_decrypt_with_key_whitespace() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has whitespace (simulating user copying with extra spaces)
    let key_with_whitespace = format!("  {}  ", TEST_KEY);
    let mut cmd = create_decrypt_command(temp_dir.path(), &key_with_whitespace);
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_decrypt_with_base64_prefix_and_whitespace() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has both "base64:" prefix and whitespace
    let key_with_prefix_and_whitespace = format!("  base64:{}  ", TEST_KEY);
    let mut cmd = create_decrypt_command(temp_dir.path(), &key_with_prefix_and_whitespace);
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}
