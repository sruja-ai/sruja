use assert_cmd::prelude::*;
use std::process::Command;

/// Smoke test: proves the sruja binary compiles and links correctly
/// after adding the dotenvy dependency (T2/T3). A successful run means
/// the new crate resolved and the binary is functional at a basic level.
#[test]
fn sruja_help_runs_successfully() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("sruja")?;
    cmd.arg("--help");
    cmd.assert().success();
    Ok(())
}
