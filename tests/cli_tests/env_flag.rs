use crate::common::*;
use std::fs;

#[test]
fn test_encrypt_with_env_flag() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env.local");
    let encrypted_path = temp_dir.path().join(".env.local.encrypted");

    // Create a test .env.local file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("local");
    cmd.assert().success();

    // Check that .env.local.encrypted was created
    assert!(encrypted_path.exists(), ".env.local.encrypted file should be created");
    
    // Verify original .env.local file still exists
    assert!(env_path.exists(), "Original .env.local file should still exist");

    // Check that the encrypted content is different from plaintext
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, env_content, "Encrypted content should differ from plaintext");
    assert!(!encrypted_content.is_empty(), "Encrypted content should not be empty");
}

#[test]
fn test_encrypt_with_env_flag_env_takes_precedence_for_output() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.production.encrypted");

    // Create a test .env file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and explicit --input
    // ENV should take precedence for output file name
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("production").arg("--input").arg(".env");
    cmd.assert().success();

    // Check that .env.production.encrypted was created (env takes precedence for output)
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created");
    assert!(!temp_dir.path().join(".env.encrypted").exists(), ".env.encrypted should not be created");
}

#[test]
fn test_encrypt_with_env_flag_production() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env.production");
    let encrypted_path = temp_dir.path().join(".env.production.encrypted");

    // Create a test .env.production file
    let env_content = "APP_KEY=prod123\nDB_PASSWORD=prodsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env production
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("production");
    cmd.assert().success();

    // Check that .env.production.encrypted was created
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created");
    
    // Verify original .env.production file still exists
    assert!(env_path.exists(), "Original .env.production file should still exist");
}

#[test]
fn test_encrypt_with_env_flag_development() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env.development");
    let encrypted_path = temp_dir.path().join(".env.development.encrypted");

    // Create a test .env.development file
    let env_content = "APP_KEY=dev123\nDB_PASSWORD=devsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env development
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("development");
    cmd.assert().success();

    // Check that .env.development.encrypted was created
    assert!(encrypted_path.exists(), ".env.development.encrypted file should be created");
    
    // Verify original .env.development file still exists
    assert!(env_path.exists(), "Original .env.development file should still exist");
}

#[test]
fn test_encrypt_decrypt_roundtrip_with_env() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env.local");
    let encrypted_path = temp_dir.path().join(".env.local.encrypted");

    // Create a test .env.local file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --env flag
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("local");
    cmd.assert().success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env.local file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt (using explicit input path since decrypt doesn't have --env yet)
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg(".env.local.encrypted");
    cmd.assert().success();

    // Verify .env.local was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_with_env_flag_and_custom_input_path() {
    let temp_dir = create_temp_dir();
    let tmp_dir = create_subdir(temp_dir.path(), "tmp");
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.local.encrypted");

    // Create a test .env file in tmp directory
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and custom input path
    // Output should be .env.local.encrypted next to the input file (in tmp directory)
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("local").arg("--input").arg("tmp/.env");
    cmd.assert().success();

    // Check that .env.local.encrypted was created next to the input file (in tmp directory)
    assert!(encrypted_path.exists(), ".env.local.encrypted file should be created next to input file in tmp directory");
    assert!(!temp_dir.path().join(".env.local.encrypted").exists(), ".env.local.encrypted should not be created in current directory");
    
    // Verify original .env file still exists
    assert!(env_path.exists(), "Original .env file should still exist");
}

#[test]
fn test_encrypt_with_env_flag_preserves_input_directory() {
    let temp_dir = create_temp_dir();
    let sub_dir = create_subdir(temp_dir.path(), "config");
    
    let env_path = sub_dir.join(".env");
    let encrypted_path = sub_dir.join(".env.production.encrypted");

    // Create a test .env file in config directory
    let env_content = "APP_KEY=prod123\nDB_PASSWORD=prodsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and input in subdirectory
    // Output should be next to the input file
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--env").arg("production").arg("--input").arg("config/.env");
    cmd.assert().success();

    // Check that .env.production.encrypted was created next to the input file
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created next to input file in config directory");
    assert!(!temp_dir.path().join(".env.production.encrypted").exists(), ".env.production.encrypted should not be created in current directory");
}
