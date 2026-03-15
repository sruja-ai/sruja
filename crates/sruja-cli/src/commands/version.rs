//! Version command.

use super::CliError;

pub fn version() -> Result<(), CliError> {
    println!("sruja version {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
