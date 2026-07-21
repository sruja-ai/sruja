use std::path::PathBuf;

#[derive(Clone)]
pub struct ScanConfig {
    pub include_tests: bool,
    pub include_node_modules: bool,
    pub exclude_examples: bool,
    pub exclude_benches: bool,
    pub exclude_fixtures: bool,
    pub exclude_docs: bool,
    pub max_file_size: usize,
    pub classification_rules_path: Option<PathBuf>,
    pub incremental: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            include_tests: false,
            include_node_modules: false,
            exclude_examples: true,
            exclude_benches: true,
            exclude_fixtures: true,
            exclude_docs: true,
            max_file_size: 500 * 1024,
            classification_rules_path: None,
            incremental: false,
        }
    }
}
