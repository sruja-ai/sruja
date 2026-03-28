use std::fs;
use std::path::Path;

use crate::commands::CliError;

/// Collect all .sruja files from a directory recursively
///
/// # Arguments
/// * `dir_path` - Path to the directory to search
///
/// # Returns
/// A vector of file paths
///
/// # Errors
/// Returns `CliError::Io` if the directory cannot be read
pub fn collect_sruja_files(dir_path: &Path) -> Result<Vec<String>, CliError> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively collect from subdirectories
            let sub_files = collect_sruja_files(&path)?;
            files.extend(sub_files);
        } else if let Some(ext) = path.extension() {
            if ext == std::ffi::OsStr::new("sruja") {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_collect_sruja_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        let file1 = temp_dir.path().join("test1.sruja");
        let file2 = temp_dir.path().join("test2.sruja");
        let file3 = temp_dir.path().join("test.txt"); // Should be ignored
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file4 = subdir.join("test3.sruja");

        File::create(&file1).unwrap().write_all(b"test").unwrap();
        File::create(&file2).unwrap().write_all(b"test").unwrap();
        File::create(&file3).unwrap().write_all(b"test").unwrap();
        File::create(&file4).unwrap().write_all(b"test").unwrap();

        let files = collect_sruja_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.contains("test1.sruja")));
        assert!(files.iter().any(|f| f.contains("test2.sruja")));
        assert!(files.iter().any(|f| f.contains("test3.sruja")));
        assert!(!files.iter().any(|f| f.contains("test.txt")));
    }
}
