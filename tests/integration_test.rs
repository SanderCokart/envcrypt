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

#[test]
fn test_encrypt_with_custom_input_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir(&tmp_dir).unwrap();
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.encrypted");

    // Create a test .env file in tmp directory
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with custom input path
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("tmp/.env")
        .assert()
        .success();

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
    let temp_dir = tempfile::tempdir().unwrap();
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir(&tmp_dir).unwrap();
    
    let env_path = tmp_dir.join(".env");

    // Create and encrypt a file in tmp directory
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("tmp/.env")
        .assert()
        .success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with custom input path
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("tmp/.env.encrypted")
        .assert()
        .success();

    // Verify .env was recreated in tmp directory with correct content
    assert!(env_path.exists(), ".env file should be recreated in tmp directory");
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_decrypt_roundtrip_with_custom_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir(&tmp_dir).unwrap();
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.encrypted");

    // Create a test .env file in tmp directory
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with custom input path
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("tmp/.env")
        .assert()
        .success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with custom input path
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("tmp/.env.encrypted")
        .assert()
        .success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_with_nonexistent_input_fails() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Try to encrypt a non-existent file
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("nonexistent/.env")
        .assert()
        .failure()
        .stderr(predicates::str::contains("file not found"));
}

#[test]
fn test_decrypt_with_nonexistent_input_fails() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Try to decrypt a non-existent file
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg("nonexistent/.env.encrypted")
        .assert()
        .failure()
        .stderr(predicates::str::contains("file not found"));
}

#[test]
fn test_decrypt_with_base64_prefix() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has "base64:" prefix (simulating user copying the displayed key)
    let key_with_prefix = format!("base64:{}", TEST_KEY);
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(&key_with_prefix)
        .assert()
        .success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_decrypt_with_key_whitespace() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has whitespace (simulating user copying with extra spaces)
    let key_with_whitespace = format!("  {}  ", TEST_KEY);
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(&key_with_whitespace)
        .assert()
        .success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_decrypt_with_base64_prefix_and_whitespace() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with key
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .assert()
        .success();

    // Delete the original .env file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt with key that has both "base64:" prefix and whitespace
    let key_with_prefix_and_whitespace = format!("  base64:{}  ", TEST_KEY);
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(&key_with_prefix_and_whitespace)
        .assert()
        .success();

    // Verify .env was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_with_env_flag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env.local");
    let encrypted_path = temp_dir.path().join(".env.local.encrypted");

    // Create a test .env.local file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("local")
        .assert()
        .success();

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
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.production.encrypted");

    // Create a test .env file
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and explicit --input
    // ENV should take precedence for output file name
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("production")
        .arg("--input")
        .arg(".env")
        .assert()
        .success();

    // Check that .env.production.encrypted was created (env takes precedence for output)
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created");
    assert!(!temp_dir.path().join(".env.encrypted").exists(), ".env.encrypted should not be created");
}

#[test]
fn test_encrypt_with_env_flag_production() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env.production");
    let encrypted_path = temp_dir.path().join(".env.production.encrypted");

    // Create a test .env.production file
    let env_content = "APP_KEY=prod123\nDB_PASSWORD=prodsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env production
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("production")
        .assert()
        .success();

    // Check that .env.production.encrypted was created
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created");
    
    // Verify original .env.production file still exists
    assert!(env_path.exists(), "Original .env.production file should still exist");
}

#[test]
fn test_encrypt_with_env_flag_development() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env.development");
    let encrypted_path = temp_dir.path().join(".env.development.encrypted");

    // Create a test .env.development file
    let env_content = "APP_KEY=dev123\nDB_PASSWORD=devsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env development
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("development")
        .assert()
        .success();

    // Check that .env.development.encrypted was created
    assert!(encrypted_path.exists(), ".env.development.encrypted file should be created");
    
    // Verify original .env.development file still exists
    assert!(env_path.exists(), "Original .env.development file should still exist");
}

#[test]
fn test_encrypt_decrypt_roundtrip_with_env() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env.local");
    let encrypted_path = temp_dir.path().join(".env.local.encrypted");

    // Create a test .env.local file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456\nANOTHER_VAR=value";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --env flag
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("local")
        .assert()
        .success();

    // Verify encrypted file exists and is different
    assert!(encrypted_path.exists());
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(encrypted_content, original_content);

    // Delete the original .env.local file to test decryption
    fs::remove_file(&env_path).unwrap();

    // Decrypt (using explicit input path since decrypt doesn't have --env yet)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("decrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--input")
        .arg(".env.local.encrypted")
        .assert()
        .success();

    // Verify .env.local was recreated with original content
    assert!(env_path.exists());
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_encrypt_with_env_flag_and_custom_input_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir(&tmp_dir).unwrap();
    
    let env_path = tmp_dir.join(".env");
    let encrypted_path = tmp_dir.join(".env.local.encrypted");

    // Create a test .env file in tmp directory
    let env_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and custom input path
    // Output should be .env.local.encrypted next to the input file (in tmp directory)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("local")
        .arg("--input")
        .arg("tmp/.env")
        .assert()
        .success();

    // Check that .env.local.encrypted was created next to the input file (in tmp directory)
    assert!(encrypted_path.exists(), ".env.local.encrypted file should be created next to input file in tmp directory");
    assert!(!temp_dir.path().join(".env.local.encrypted").exists(), ".env.local.encrypted should not be created in current directory");
    
    // Verify original .env file still exists
    assert!(env_path.exists(), "Original .env file should still exist");
}

#[test]
fn test_encrypt_with_env_flag_preserves_input_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sub_dir = temp_dir.path().join("config");
    fs::create_dir(&sub_dir).unwrap();
    
    let env_path = sub_dir.join(".env");
    let encrypted_path = sub_dir.join(".env.production.encrypted");

    // Create a test .env file in config directory
    let env_content = "APP_KEY=prod123\nDB_PASSWORD=prodsecret";
    fs::write(&env_path, env_content).unwrap();

    // Run encrypt command with --env flag and input in subdirectory
    // Output should be next to the input file
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .arg("--key")
        .arg(TEST_KEY)
        .arg("--env")
        .arg("production")
        .arg("--input")
        .arg("config/.env")
        .assert()
        .success();

    // Check that .env.production.encrypted was created next to the input file
    assert!(encrypted_path.exists(), ".env.production.encrypted file should be created next to input file in config directory");
    assert!(!temp_dir.path().join(".env.production.encrypted").exists(), ".env.production.encrypted should not be created in current directory");
}
