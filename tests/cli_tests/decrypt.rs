use crate::common::*;
use predicates::prelude::*;

#[test]
fn test_decrypt_with_wrong_key_fails() {
    let temp_dir = create_temp_dir();
    let env_path = temp_dir.path().join(".env");

    // Create and encrypt a file
    let original_content = "APP_KEY=test123\nDB_PASSWORD=secret456";
    std::fs::write(&env_path, original_content).unwrap();

    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.assert().success();

    // Delete the original .env file so decryption can create it
    std::fs::remove_file(&env_path).unwrap();

    // Try to decrypt with wrong key
    let mut cmd = create_decrypt_command(temp_dir.path(), "wrong-key");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("MAC verification failed").or(predicates::str::contains("Decryption failed")));
}
