use crate::error::Result;
use std::path::PathBuf;

pub async fn run(path: PathBuf) -> Result<()> {
    println!("Checking skill files: {:?}", path);
    Ok(())
}
