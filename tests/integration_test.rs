use assert_cmd::Command;
use std::fs;

#[test]
fn test_encrypt_creates_encrypted_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    let encrypted_path = temp_dir.path().join(".env.encrypted");

    // Create a test .env file
    fs::write(&env_path, "APP_KEY=test123\nDB_PASSWORD=secret456").unwrap();

    // Run encrypt command
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("envcrypt"));
    cmd.current_dir(temp_dir.path())
        .arg("encrypt")
        .assert()
        .success();

    // Check that .env.encrypted was created
    assert!(encrypted_path.exists(), ".env.encrypted file should be created");

    // Check that the content matches (for now, just copying)
    let encrypted_content = fs::read_to_string(&encrypted_path).unwrap();
    let env_content = fs::read_to_string(&env_path).unwrap();
    assert_eq!(encrypted_content, env_content);
}
