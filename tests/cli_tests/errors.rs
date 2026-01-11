use crate::common::*;

#[test]
fn test_encrypt_with_nonexistent_input_fails() {
    let temp_dir = create_temp_dir();

    // Try to encrypt a non-existent file
    let mut cmd = create_encrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("nonexistent/.env");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("file not found"));
}

#[test]
fn test_decrypt_with_nonexistent_input_fails() {
    let temp_dir = create_temp_dir();

    // Try to decrypt a non-existent file
    let mut cmd = create_decrypt_command(temp_dir.path(), TEST_KEY);
    cmd.arg("--input").arg("nonexistent/.env.encrypted");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("file not found"));
}
