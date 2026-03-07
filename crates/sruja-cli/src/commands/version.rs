//! Version command.

use super::CliError;

pub fn version() -> Result<(), CliError> {
    println!("sruja version 2.0.0");
    Ok(())
}
