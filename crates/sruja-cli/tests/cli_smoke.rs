use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("sruja")?;
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("sruja")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("drift"))
        .stdout(predicate::str::contains("start"));
    Ok(())
}

#[test]
fn test_cli_validate_minimal_valid_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("minimal.sruja");
    fs::write(
        &file_path,
        r#"MySystem = system "My System" {
  description "A deployable system"
}
"#,
    )?;

    let mut cmd = Command::cargo_bin("sruja")?;
    cmd.arg("validate").arg(file_path.to_str().unwrap());

    cmd.assert().success();
    Ok(())
}

#[test]
fn test_cli_validate_invalid_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join("invalid.sruja");
    fs::write(&file_path, "invalid syntax {")?;

    let mut cmd = Command::cargo_bin("sruja")?;
    cmd.arg("validate").arg(file_path.to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
    Ok(())
}
