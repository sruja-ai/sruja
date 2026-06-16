//! Phase-based file-access policy for TDD enforcement.
//!
//! Core principle: **tests and code are never in flux simultaneously.**
//! One side is always the frozen anchor while the other changes. This prevents
//! the agent from gaming its own tests by weakening them alongside the fix.
//!
//! ## Pipeline
//!
//! ```text
//! TestAuthor  ──→  TestReview  ──→  Implement  ──→  Review
//! (tests only)    (HITL gate)    (code only,       (Critic,
//!                                  tests frozen)    read-only)
//!                       ↑               │
//!                       │  test wrong?  │
//!                       └───────────────┘
//! ```
//!
//! - **TestAuthor**: only test files are writable. Code is frozen.
//! - **TestReview**: human approves the tests. Nothing is writable.
//! - **Implement**: only non-test files are writable. Tests are frozen.
//! - **Review**: Critic inspects. Nothing is writable.

use std::sync::{Arc, Mutex};

/// The current phase of the TDD pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Read-only comprehension (no writes).
    #[default]
    Comprehend,
    /// Write tests based on the spec. Only test files are writable.
    TestAuthor,
    /// Human review gate on tests. Read-only.
    TestReview,
    /// Write code to pass the frozen tests. Non-test files only.
    Implement,
    /// Critic review. Read-only.
    Review,
}

impl Phase {
    /// Whether any writes are allowed in this phase.
    pub fn allows_writes(self) -> bool {
        matches!(self, Phase::TestAuthor | Phase::Implement)
    }

    /// Whether test files are writable in this phase.
    pub fn allows_test_writes(self) -> bool {
        self == Phase::TestAuthor
    }

    /// Whether code (non-test) files are writable in this phase.
    pub fn allows_code_writes(self) -> bool {
        self == Phase::Implement
    }
}

/// Classify whether a path is a test file.
///
/// Uses configurable glob patterns with sensible defaults that cover
/// Rust, TypeScript/JavaScript, Python, Go, and Java conventions.
#[derive(Debug, Clone)]
pub struct TestPathClassifier {
    patterns: Vec<String>,
}

impl Default for TestPathClassifier {
    fn default() -> Self {
        Self {
            patterns: vec![
                // Directories
                "**/tests/**".into(),
                "**/test/**".into(),
                "**/__tests__/**".into(),
                "**/spec/**".into(),
                // Suffix patterns
                "**/*_test.*".into(),
                "**/*_spec.*".into(),
                "**/*.test.*".into(),
                "**/*.spec.*".into(),
                "**/*Test.java".into(),
                // Bare test entry points
                "**/test.rs".into(),
                "**/test_*.rs".into(),
                "**/*_test.rs".into(),
            ],
        }
    }
}

impl TestPathClassifier {
    /// Create with custom patterns (replaces defaults).
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    /// Add a pattern to the defaults.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Whether the given relative path is classified as a test file.
    pub fn is_test_path(&self, path: &str) -> bool {
        self.patterns
            .iter()
            .any(|p| super::builtin::glob_match(p, path))
    }
}

/// Shared, mutable file-access guard consulted by the [`ToolRegistry`](super::ToolRegistry).
///
/// The cognition loop updates the [`Phase`]; mutating tools check this guard
/// before writing. One side is always frozen.
#[derive(Debug, Clone)]
pub struct FileGuard {
    inner: Arc<Mutex<FileGuardInner>>,
}

#[derive(Debug)]
struct FileGuardInner {
    phase: Phase,
    classifier: TestPathClassifier,
    /// Explicitly frozen paths (regardless of classification).
    frozen: Vec<String>,
}

impl FileGuard {
    /// Create a guard starting in [`Phase::Comprehend`] with default test classification.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(FileGuardInner {
                phase: Phase::Comprehend,
                classifier: TestPathClassifier::default(),
                frozen: Vec::new(),
            })),
        }
    }

    /// Create with a custom test-path classifier.
    pub fn with_classifier(classifier: TestPathClassifier) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FileGuardInner {
                phase: Phase::Comprehend,
                classifier,
                frozen: Vec::new(),
            })),
        }
    }

    /// Current phase.
    pub fn phase(&self) -> Phase {
        self.inner.lock().unwrap().phase
    }

    /// Transition to a new phase.
    pub fn set_phase(&self, phase: Phase) {
        self.inner.lock().unwrap().phase = phase;
    }

    /// Freeze specific paths explicitly (e.g. the agreed test files after review).
    pub fn freeze(&self, paths: &[String]) {
        self.inner
            .lock()
            .unwrap()
            .frozen
            .extend(paths.iter().cloned());
    }

    /// Check whether a write to `path` is permitted under the current phase.
    pub fn can_write(&self, path: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        if inner.frozen.iter().any(|f| path == f) {
            return false;
        }
        let is_test = inner.classifier.is_test_path(path);
        match inner.phase {
            Phase::TestAuthor => is_test,
            Phase::Implement => !is_test,
            _ => false,
        }
    }

    /// Human-readable reason for a denied write.
    pub fn deny_reason(&self, path: &str) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        if inner.frozen.iter().any(|f| path == f) {
            return Some(format!("'{path}' is explicitly frozen"));
        }
        let is_test = inner.classifier.is_test_path(path);
        match inner.phase {
            Phase::Comprehend => Some("in Comprehend phase (read-only)".into()),
            Phase::TestReview => {
                Some("in TestReview phase (read-only, awaiting human approval)".into())
            }
            Phase::Review => Some("in Review phase (read-only)".into()),
            Phase::TestAuthor if !is_test => Some(format!(
                "'{path}' is not a test file; TestAuthor phase allows test writes only"
            )),
            Phase::Implement if is_test => Some(format!(
                "'{path}' is a test file; tests are FROZEN during Implement phase"
            )),
            _ => None,
        }
    }
}

impl Default for FileGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_defaults() {
        let c = TestPathClassifier::default();
        assert!(c.is_test_path("tests/foo.rs"));
        assert!(c.is_test_path("src/utils_test.rs"));
        assert!(c.is_test_path("components/Button.test.tsx"));
        assert!(c.is_test_path("__tests__/index.js"));
        assert!(!c.is_test_path("src/main.rs"));
        assert!(!c.is_test_path("lib/utils.rs"));
    }

    #[test]
    fn phase_freezes_correct_side() {
        let guard = FileGuard::new();

        guard.set_phase(Phase::TestAuthor);
        assert!(guard.can_write("tests/foo.rs"));
        assert!(!guard.can_write("src/main.rs"));

        guard.set_phase(Phase::Implement);
        assert!(!guard.can_write("tests/foo.rs"));
        assert!(guard.can_write("src/main.rs"));

        guard.set_phase(Phase::Review);
        assert!(!guard.can_write("tests/foo.rs"));
        assert!(!guard.can_write("src/main.rs"));
    }

    #[test]
    fn explicit_freeze_overrides_phase() {
        let guard = FileGuard::new();
        guard.freeze(&["tests/agreed.rs".into()]);

        guard.set_phase(Phase::TestAuthor);
        assert!(!guard.can_write("tests/agreed.rs"));
        assert!(guard.can_write("tests/new.rs"));
    }

    #[test]
    fn deny_reason_is_helpful() {
        let guard = FileGuard::new();
        guard.set_phase(Phase::Implement);
        let reason = guard.deny_reason("tests/foo.rs").unwrap();
        assert!(reason.contains("FROZEN"));
    }
}
