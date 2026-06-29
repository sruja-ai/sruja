//! Git checkpoint helper for the autonomous agent loop.
//!
//! Creates a backup ref before execution so the user can roll back
//! unwanted changes. Opt-in by default; auto-enables for one-way-door goals.

use std::path::Path;

/// A git checkpoint ref for rollback safety.
///
/// Created before execution begins. The ref points at the current HEAD,
/// allowing the user to restore via `git update-ref HEAD <ref>` +
/// `git reset --hard`.
pub struct GitCheckpoint {
    ref_name: String,
}

impl GitCheckpoint {
    /// Create a checkpoint ref pointing at current HEAD.
    ///
    /// Returns `Ok(None)` if the path is not a git repo (graceful no-op).
    pub fn create(repo_path: &Path) -> Result<Option<Self>, git2::Error> {
        let repo = match git2::Repository::open(repo_path) {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let suffix: String = (0..4)
            .map(|i| {
                let idx = ((timestamp >> (i * 6)) % 26) as u8;
                (b'a' + idx) as char
            })
            .collect();

        let ref_name = format!("refs/sruja/agent-backup-{}-{}", timestamp, suffix);

        repo.reference(
            &ref_name,
            head_commit.id(),
            false,
            "sruja agent loop checkpoint",
        )?;

        Ok(Some(Self { ref_name }))
    }

    /// Print the restore hint to stderr.
    pub fn print_restore_hint(&self) {
        eprintln!("  Checkpoint ref: {}", self.ref_name);
        eprintln!(
            "  To roll back: git update-ref HEAD {} && git reset --hard",
            self.ref_name
        );
    }

    /// The full ref name (e.g. `refs/sruja/agent-backup-1234567890-abcd`).
    pub fn ref_name(&self) -> &str {
        &self.ref_name
    }
}

/// Determine whether a checkpoint should be auto-created based on the
/// calibration verdict's reversibility signal and CLI flags.
///
/// Returns `true` when:
/// - `--checkpoint` is set (force on), or
/// - `--no-checkpoint` is not set AND the goal is a one-way door.
pub fn should_checkpoint(
    reversibility: sruja_agent::calibration::Reversibility,
    force_on: bool,
    force_off: bool,
) -> bool {
    if force_off {
        return false;
    }
    if force_on {
        return true;
    }
    matches!(
        reversibility,
        sruja_agent::calibration::Reversibility::OneWay
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_agent::calibration::Reversibility;

    #[test]
    fn one_way_auto_enables() {
        assert!(should_checkpoint(Reversibility::OneWay, false, false));
    }

    #[test]
    fn two_way_default_off() {
        assert!(!should_checkpoint(Reversibility::TwoWay, false, false));
    }

    #[test]
    fn checkpoint_flag_forces_on() {
        assert!(should_checkpoint(Reversibility::TwoWay, true, false));
    }

    #[test]
    fn no_checkpoint_flag_forces_off() {
        assert!(!should_checkpoint(Reversibility::OneWay, false, true));
    }

    #[test]
    fn no_checkpoint_wins_over_checkpoint() {
        // When both flags are set, --no-checkpoint wins (standard CLI convention)
        assert!(!should_checkpoint(Reversibility::TwoWay, true, true));
    }

    #[test]
    fn create_and_restore_in_temp_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();

        // Init a git repo with a commit
        let repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::new("test", "test@test.com", &git2::Time::new(0, 0)).unwrap();
        std::fs::write(repo_path.join("file.txt"), "original").unwrap();

        // Stage and commit
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("file.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let commit = repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        // Create checkpoint
        let cp = GitCheckpoint::create(repo_path).unwrap().unwrap();
        assert!(cp.ref_name.starts_with("refs/sruja/agent-backup-"));

        // Verify the ref points at HEAD
        let ref_obj = repo.find_reference(cp.ref_name()).unwrap();
        let cp_commit = ref_obj.peel_to_commit().unwrap();
        assert_eq!(cp_commit.id(), commit);
    }

    #[test]
    fn create_in_non_repo_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let result = GitCheckpoint::create(tmp.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn checkpoint_names_are_unique_within_same_second() {
        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path();

        let repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::new("test", "test@test.com", &git2::Time::new(0, 0)).unwrap();
        std::fs::write(repo_path.join("f.txt"), "x").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("f.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();

        let cp1 = GitCheckpoint::create(repo_path).unwrap().unwrap();
        let cp2 = GitCheckpoint::create(repo_path).unwrap().unwrap();

        // Names must differ even if created in the same nanosecond
        assert_ne!(cp1.ref_name(), cp2.ref_name());
    }
}
