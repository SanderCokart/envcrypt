use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_hello_world_output() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("env-encrypter-rust"));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello, world!"));
}
