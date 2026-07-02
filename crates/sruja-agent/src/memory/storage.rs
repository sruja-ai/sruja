//! File I/O operations for agentic memory.

use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::types::MemoryError;
use super::AgenticMemory;

/// Loads agentic memory from the specified repository root.
///
/// Memory is stored in `.sruja/agent_memory.json`. If the file does not exist,
/// an empty memory is returned.
pub fn load(repo_root: &Path) -> Result<AgenticMemory, MemoryError> {
    let path = get_path(repo_root);
    load_from_path(&path)
}

/// Loads agentic memory from a specific path.
pub fn load_from_path(path: &Path) -> Result<AgenticMemory, MemoryError> {
    if !path.exists() {
        return Ok(AgenticMemory::default());
    }
    let file = File::open(path)?;
    file.lock_shared()?;
    let mut content = String::new();
    let mut reader = std::io::BufReader::new(&file);
    reader.read_to_string(&mut content)?;
    file.unlock()?;
    let mut memory: AgenticMemory = serde_json::from_str(&content)?;
    // Rebuild the in-memory dedup index (not serialized).
    memory.rebuild_dedup_index();
    Ok(memory)
}

/// Saves the current memory to the specified repository root.
///
/// This will create the `.sruja` directory if it doesn't exist.
pub fn save(memory: &AgenticMemory, repo_root: &Path) -> Result<(), MemoryError> {
    let path = get_path(repo_root);
    save_to_path(memory, &path)
}

/// Saves the current memory to a specific path.
pub fn save_to_path(memory: &AgenticMemory, path: &Path) -> Result<(), MemoryError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.lock_exclusive()?;
    let content = serde_json::to_string_pretty(memory)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    {
        let mut writer = std::io::BufWriter::new(&mut file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;
    }
    file.unlock()?;
    Ok(())
}

/// Clears the agentic memory for the specified repository.
pub fn clear(repo_root: &Path) -> Result<(), MemoryError> {
    let path = get_path(repo_root);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Checks if the agentic memory file exists for the given repository.
pub fn exists(repo_root: &Path) -> bool {
    get_path(repo_root).exists()
}

/// Returns the path to the agentic memory file for the given repository root.
pub fn get_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".sruja").join("agent_memory.json")
}
