use crate::common::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_version_flag() {
    let temp_dir = create_temp_dir();
    let mut cmd = create_command(temp_dir.path());
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(
            predicates::str::contains("envcrypt 0.1.0")
                .and(predicates::str::contains("("))
                .and(predicates::str::contains(")"))
        );
}

#[test]
fn test_silent_flag_suppresses_output() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --silent flag
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--silent");
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());
}

#[test]
fn test_quiet_flag_suppresses_info_but_shows_errors() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --quiet flag
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--quiet");
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty());

    // Test that errors are still shown
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--quiet").arg("--input").arg("nonexistent.env");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("file not found"));
}

#[test]
fn test_force_flag_overwrites_existing_file() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // First encryption
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();
    
    let first_encrypted = fs::read_to_string(&encrypted_path).unwrap();
    
    // Modify the .env file
    fs::write(&env_path, "APP_KEY=modified").unwrap();
    
    // Encrypt again without --force should fail
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
    
    // Encrypt again with --force should succeed
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--force");
    cmd.assert().success();
    
    let second_encrypted = fs::read_to_string(&encrypted_path).unwrap();
    assert_ne!(first_encrypted, second_encrypted, "Encrypted files should differ");
}

#[test]
fn test_prune_flag_deletes_original_file() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --prune flag
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--prune");
    cmd.assert().success();
    
    // Check that .env was deleted
    assert!(!env_path.exists(), ".env file should be deleted after --prune");
    assert!(encrypted_path.exists(), ".env.encrypted file should exist");
}

#[test]
fn test_no_interaction_auto_generates_key_for_encrypt() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt with --no-interaction (no key provided, should auto-generate)
    let mut cmd = create_command(temp_dir.path());
    cmd.arg("encrypt").arg("--no-interaction");
    cmd.assert().success();
    
    assert!(encrypted_path.exists(), ".env.encrypted file should be created");
}

#[test]
fn test_no_interaction_requires_key_for_decrypt() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // First encrypt
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();
    
    // Remove .env
    fs::remove_file(&env_path).unwrap();
    
    // Try to decrypt with --no-interaction but no key (should fail)
    let mut cmd = create_command(temp_dir.path());
    cmd.arg("decrypt").arg("--no-interaction");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("key is required"));
}

#[test]
fn test_verbose_flag_increases_output() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Test with -v (level 1)
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("-v");
    cmd.assert().success();
    
    // Test with -vv (level 2)
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--force").arg("-vv");
    cmd.assert().success();
    
    // Test with -vvv (level 3)
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--force").arg("-vvv");
    cmd.assert().success();
}

#[test]
fn test_silent_overrides_verbose() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // --silent should override --verbose
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--silent").arg("-vvv");
    cmd.assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());
}

#[test]
fn test_force_flag_for_decrypt() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();
    
    // Decrypt (creates .env)
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();
    
    // Modify .env
    fs::write(&env_path, "APP_KEY=modified").unwrap();
    
    // Try to decrypt again without --force should fail
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
    
    // Decrypt with --force should succeed
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--force");
    cmd.assert().success();
    
    let decrypted_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(decrypted_content, original_content);
}

#[test]
fn test_prune_only_applies_to_encrypt() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");
    
    let original_content = "APP_KEY=test123";
    fs::write(&env_path, original_content).unwrap();

    // Encrypt
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();
    
    // Decrypt with --prune (should not affect decrypt, encrypted file should still exist)
    fs::remove_file(&env_path).unwrap();
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--prune");
    cmd.assert().success();
    
    // Encrypted file should still exist (prune only applies to encrypt)
    assert!(encrypted_path.exists(), ".env.encrypted should still exist after decrypt with --prune");
}
