use crate::common::*;
use std::fs;

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_decrypt_roundtrip_with_custom_paths() {
    let temp_dir = create_temp_dir();
    let tmp_dir = create_subdir(temp_dir.path(), "tmp");
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.encrypted");

    // Create a test .env file in tmp directory
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with custom input path
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("tmp/.env");
    cmd.assert().success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with custom input path
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("tmp/.env.encrypted");
    cmd.assert().success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}
