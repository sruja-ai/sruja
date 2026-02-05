//! File operations module for CLI
//!
//! Provides reusable functions for file I/O, parsing, and collection operations.

#![allow(dead_code)]

use std::fs;
use std::path::Path;

use sruja_diagnostics::Diagnostic;
use sruja_language::{Parser, Program};

use crate::commands::CliError;

/// Result type for file operations that includes the program content and any parse diagnostics
#[allow(dead_code)]
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed program
    pub program: Program,
    /// Any diagnostics from parsing
    pub diagnostics: Vec<Diagnostic>,
    /// The file content
    pub content: String,
}

#[allow(dead_code)]
impl ParseResult {
    /// Check if parsing was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| d.severity != sruja_diagnostics::Severity::Error)
    }

    /// Check if there are any parse errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == sruja_diagnostics::Severity::Error)
    }
}

/// Read a file and return its content
///
/// # Arguments
/// * `file_path` - Path to the file to read
///
/// # Returns
/// The file content as a string
///
/// # Errors
/// Returns `CliError::Io` if the file cannot be read
pub fn read_file(file_path: &str) -> Result<String, CliError> {
    fs::read_to_string(file_path).map_err(Into::into)
}

/// Parse a Sruja file and return the program
///
/// # Arguments
/// * `file_path` - Path to the file to parse
///
/// # Returns
/// A `ParseResult` containing the program and any diagnostics
///
/// # Errors
/// Returns `CliError::Parse` if there are parsing errors
pub fn parse_file(file_path: &str) -> Result<ParseResult, CliError> {
    let content = read_file(file_path)?;
    let parser = Parser::new(file_path.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            // Print diagnostics for user feedback
            for diag in &diagnostics {
                eprintln!("{}", sruja_diagnostics::format_diagnostic(diag));
            }

            return Err(CliError::Parse(format!(
                "Parsing failed with {} error(s)",
                diagnostics.len()
            )));
        }
    };

    Ok(ParseResult {
        program,
        diagnostics: Vec::new(),
        content,
    })
}

/// Parse a file with detailed diagnostics handling
///
/// This version captures all diagnostics (warnings and errors) without failing on errors,
/// allowing the caller to decide how to handle them.
///
/// # Arguments
/// * `file_path` - Path to the file to parse
///
/// # Returns
/// A `ParseResult` containing the program and any diagnostics
pub fn parse_file_with_diagnostics(file_path: &str) -> Result<ParseResult, CliError> {
    let content = read_file(file_path)?;
    let parser = Parser::new(file_path.to_string());

    let program = match parser.parse(&content) {
        Ok(program) => program,
        Err(diagnostics) => {
            return Ok(ParseResult {
                program: Program::default(),
                diagnostics,
                content,
            });
        }
    };

    Ok(ParseResult {
        program,
        diagnostics: Vec::new(),
        content,
    })
}

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

/// Check if a path exists and is accessible
///
/// # Arguments
/// * `file_path` - Path to check
///
/// # Returns
/// true if the path exists, false otherwise
pub fn file_exists(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

/// Check if a path is a directory
///
/// # Arguments
/// * `file_path` - Path to check
///
/// # Returns
/// true if the path is a directory, false otherwise
pub fn is_directory(file_path: &str) -> bool {
    Path::new(file_path).is_dir()
}

/// Write content to a file
///
/// # Arguments
/// * `file_path` - Path to write to
/// * `content` - Content to write
///
/// # Returns
/// Ok(()) on success
///
/// # Errors
/// Returns `CliError::Io` if the file cannot be written
pub fn write_file(file_path: &str, content: &str) -> Result<(), CliError> {
    fs::write(file_path, content).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file("nonexistent.sruja");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.sruja");

        let content = r#"
person = kind "Person"
user = person "User" {
    description "A test user"
}
"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let result = parse_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_success());
    }

    #[test]
    fn test_parse_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.sruja");

        // Use syntax that might cause parsing issues
        let content = r#"
invalid syntax structure that may not parse correctly
system "Test" {
    unmatched brace
"#;

        File::create(&file_path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        // parse_file_with_diagnostics should always return a result
        // (even if parsing fails, it returns a ParseResult with diagnostics)
        let result = parse_file_with_diagnostics(file_path.to_str().unwrap());
        assert!(
            result.is_ok(),
            "parse_file_with_diagnostics should always return Ok result"
        );

        // Verify we get a ParseResult with some content
        let parse_result = result.unwrap();
        // The content should have been read
        assert!(!parse_result.content.is_empty(), "Content should be read");
    }

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

    #[test]
    fn test_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.sruja");

        let content = "test content";
        write_file(file_path.to_str().unwrap(), content).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_file_exists() {
        assert!(!file_exists("nonexistent.sruja"));

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exists.sruja");
        File::create(&file_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();

        assert!(file_exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        assert!(is_directory(temp_dir.path().to_str().unwrap()));

        let file_path = temp_dir.path().join("file.sruja");
        File::create(&file_path)
            .unwrap()
            .write_all(b"test")
            .unwrap();
        assert!(!is_directory(file_path.to_str().unwrap()));
    }
}
