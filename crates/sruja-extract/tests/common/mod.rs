pub use std::fs;

pub use sruja_extract::{
    DiscoveredSource, Extractor, FileContext,
};
pub use sruja_language::ast::SourceKind;

pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

pub fn check(
    extractor: &dyn Extractor,
    path: &std::path::Path,
    root: &std::path::Path,
) -> Vec<DiscoveredSource> {
    let ctx = FileContext::new(path, root);
    extractor
        .check_file(&ctx)
        .expect("check_file should not error")
}
