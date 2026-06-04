# Phase 4: Federation as Graph Linking - Implementation Plan

**Goal:** Enable persistent cross-repo context with declarative manifests and automatic drift detection.

**Prerequisites:** Phase 1 complete (unified graph)

---

## Task 4.1: Repo Manifest Format

### Files to Create
- `docs/REPO_MANIFEST_SPEC.md` (specification)
- `crates/sruja-cli/src/commands/repo_manifest.rs`

### Files to Modify
- `crates/sruja-cli/src/commands/federation.rs` (read manifest in `compose()`)
- `crates/sruja-cli/src/commands/mod.rs` (add module)

### Implementation

**1. Define manifest format (`.sruja/repos.toml`):**

```toml
# .sruja/repos.toml

[repos]
api-gateway = { path = "../api-gateway", repo_id = "api-gateway" }
user-service = { path = "../user-service", repo_id = "user-service" }
shared-lib = { path = "../shared-lib", repo_id = "shared-lib" }

[edges]
# Explicit cross-repo dependencies
api-gateway -> user-service = { kind = "calls", label = "REST API" }
user-service -> shared-lib = { kind = "depends_on", label = "shared models" }
```

**2. Create `repo_manifest.rs`:**

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoManifest {
    pub repos: HashMap<String, RepoEntry>,
    pub edges: Vec<CrossRepoEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub path: String,
    pub repo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepoEdge {
    pub source_repo: String,
    pub target_repo: String,
    pub kind: String,
    pub label: Option<String>,
}

pub fn load_manifest(repo_root: &Path) -> Result<RepoManifest, CliError> {
    let manifest_path = repo_root.join(".sruja/repos.toml");
    if !manifest_path.exists() {
        return Ok(RepoManifest {
            repos: HashMap::new(),
            edges: Vec::new(),
        });
    }
    
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: RepoManifest = toml::from_str(&content)
        .map_err(|e| CliError::validation(format!("Invalid repos.toml: {}", e)))?;
    
    Ok(manifest)
}

pub fn save_manifest(repo_root: &Path, manifest: &RepoManifest) -> Result<(), CliError> {
    let manifest_path = repo_root.join(".sruja/repos.toml");
    std::fs::create_dir_all(manifest_path.parent().unwrap())?;
    
    let content = toml::to_string_pretty(manifest)
        .map_err(|e| CliError::validation(format!("Failed to serialize manifest: {}", e)))?;
    std::fs::write(manifest_path, content)?;
    
    Ok(())
}

pub fn resolve_repo_paths(repo_root: &Path, manifest: &RepoManifest) -> HashMap<String, PathBuf> {
    let mut resolved = HashMap::new();
    
    for (name, entry) in &manifest.repos {
        let path = Path::new(&entry.path);
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        
        if resolved_path.exists() {
            resolved.insert(name.clone(), resolved_path);
        }
    }
    
    resolved
}

pub fn auto_discover_sibling_repos(repo_root: &Path) -> Vec<(String, PathBuf)> {
    let mut discovered = Vec::new();
    
    if let Some(parent) = repo_root.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path == repo_root {
                    continue;
                }
                if path.is_dir() && path.join(".git").exists() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    discovered.push((name, path));
                }
            }
        }
    }
    
    discovered
}
```

**3. Update `federation.rs` `compose()`:**

```rust
pub async fn compose(
    inputs: &[String],
    recursive: bool,
    output_path: &str,
) -> Result<(), CliError> {
    let repo_root = Path::new(".");
    
    // If no inputs provided, try to read from manifest
    let paths = if inputs.is_empty() {
        let manifest = load_manifest(repo_root)?;
        let resolved = resolve_repo_paths(repo_root, &manifest);
        
        // Also discover sibling repos
        let siblings = auto_discover_sibling_repos(repo_root);
        
        let mut bundle_paths = Vec::new();
        
        // Publish each discovered repo
        for (name, path) in &resolved {
            let bundle_path = path.join("repo.bundle.json");
            if !bundle_path.exists() {
                // Auto-publish
                publish(&path.to_string_lossy(), Some(name), &bundle_path.to_string_lossy()).await?;
            }
            bundle_paths.push(bundle_path.to_string_lossy().to_string());
        }
        
        bundle_paths
    } else {
        collect_bundle_paths(inputs, recursive)?
    };
    
    // ... rest of compose logic
}
```

### Validation
- Create `.sruja/repos.toml` with sibling repo paths
- Run `sruja compose -r .` without explicit inputs
- `system.index.json` is generated from manifest

---

## Task 4.2: Scoped Cross-Repo Drift

### Files to Modify
- `crates/sruja-cli/src/commands/scan.rs` (add `--cross-repo` flag)

### Implementation

**1. Add cross-repo drift detection:**

```rust
pub fn detect_cross_repo_drift(
    repo_root: &Path,
    current_graph: &sruja_scan::Graph,
) -> Result<Vec<CrossRepoViolation>, CliError> {
    let mut violations = Vec::new();
    
    // Load system index
    let Some(index_path) = find_system_index(repo_root) else {
        return Ok(violations);
    };
    let index = load_system_index(&index_path)?;
    
    let repo_id = infer_repo_id(repo_root);
    
    // Find edges where this repo is the source
    let outgoing_edges: Vec<_> = index.edges.iter()
        .filter(|e| e.source.starts_with(&format!("{}::", repo_id)))
        .collect();
    
    for edge in &outgoing_edges {
        // Extract local source ID
        let local_source = edge.source.strip_prefix(&format!("{}::", repo_id)).unwrap_or(&edge.source);
        
        // Check if source element still exists
        let source_exists = current_graph.nodes.iter().any(|n| n.id == *local_source);
        
        if !source_exists {
            violations.push(CrossRepoViolation {
                severity: ViolationSeverity::Breaking,
                message: format!(
                    "Element {} was removed but {} depends on it",
                    local_source, edge.target
                ),
                source_repo: repo_id.clone(),
                target_repo: edge.target.split("::").next().unwrap_or("unknown").to_string(),
                edge_kind: edge.kind.clone(),
                edge_label: edge.label.clone(),
            });
        }
        
        // Check if edge kind still applies (e.g., if it was calls, check if call still exists)
        // This is more complex and would require analyzing the code
    }
    
    // Find edges where this repo is the target
    let incoming_edges: Vec<_> = index.edges.iter()
        .filter(|e| e.target.starts_with(&format!("{}::", repo_id)))
        .collect();
    
    for edge in &incoming_edges {
        let local_target = edge.target.strip_prefix(&format!("{}::", repo_id)).unwrap_or(&edge.target);
        
        let target_exists = current_graph.nodes.iter().any(|n| n.id == *local_target);
        
        if !target_exists {
            violations.push(CrossRepoViolation {
                severity: ViolationSeverity::Warning,
                message: format!(
                    "Element {} was removed but {} depends on it",
                    local_target, edge.source
                ),
                source_repo: edge.source.split("::").next().unwrap_or("unknown").to_string(),
                target_repo: repo_id.clone(),
                edge_kind: edge.kind.clone(),
                edge_label: edge.label.clone(),
            });
        }
    }
    
    Ok(violations)
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossRepoViolation {
    pub severity: ViolationSeverity,
    pub message: String,
    pub source_repo: String,
    pub target_repo: String,
    pub edge_kind: String,
    pub edge_label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ViolationSeverity {
    Breaking,
    Warning,
}
```

**2. Add CLI flag:**

```bash
sruja drift -r . --cross-repo
```

Output:
```
Cross-Repo Drift:
  ⛔ Breaking: Element UserService was removed but api-gateway::Gateway depends on it
  ⚠️  Warning: Element Helper was removed but user-service::Processor depends on it
```

### Validation
- Remove an element that another repo depends on
- Run `sruja drift -r . --cross-repo`
- Violations are reported

---

## Task 4.3: Shared Schema Imports

### Files to Modify
- `crates/sruja-language/src/parser.rs` (add import resolution)
- `crates/sruja-language/src/ast.rs` (add Import node)

### Implementation

**1. Extend AST to support imports:**

```rust
// In ast.rs
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub items: Vec<ImportItem>,
}

#[derive(Debug, Clone)]
pub enum ImportItem {
    Boundary(String),
    Policy(String),
    All,
}
```

**2. Extend parser to handle imports:**

```rust
// In parser.rs
fn parse_import(&mut self) -> Result<Import, ParseError> {
    self.expect_keyword("import")?;
    
    let items = if self.check_token(Token::LeftBrace) {
        self.advance(); // consume {
        let mut items = Vec::new();
        while !self.check_token(Token::RightBrace) {
            let name = self.expect_identifier()?;
            items.push(ImportItem::Boundary(name));
            if self.check_token(Token::Comma) {
                self.advance();
            }
        }
        self.expect_token(Token::RightBrace)?;
        items
    } else {
        vec![ImportItem::All]
    };
    
    self.expect_keyword("from")?;
    let path = self.expect_string()?;
    
    Ok(Import { path, items })
}
```

**3. Resolve imports during parsing:**

```rust
fn resolve_imports(&mut self, imports: &[Import], base_path: &Path) -> Result<Vec<Definition>, ParseError> {
    let mut resolved = Vec::new();
    
    for import in imports {
        let import_path = base_path.join(&import.path);
        if !import_path.exists() {
            return Err(ParseError::ImportNotFound(import.path.clone()));
        }
        
        let content = std::fs::read_to_string(&import_path)?;
        let imported_program = self.parse(&content)?;
        
        for item in &import.items {
            match item {
                ImportItem::Boundary(name) => {
                    if let Some(boundary) = imported_program.boundaries.iter().find(|b| b.name == *name) {
                        resolved.push(Definition::Boundary(boundary.clone()));
                    }
                }
                ImportItem::Policy(name) => {
                    if let Some(policy) = imported_program.policies.iter().find(|p| p.name == *name) {
                        resolved.push(Definition::Policy(policy.clone()));
                    }
                }
                ImportItem::All => {
                    resolved.extend(imported_program.boundaries.iter().map(|b| Definition::Boundary(b.clone())));
                    resolved.extend(imported_program.policies.iter().map(|p| Definition::Policy(p.clone())));
                }
            }
        }
    }
    
    Ok(resolved)
}
```

**4. Example usage:**

```sruja
// org-standards.sruja
boundary "No Direct DB Access" {
  from service
  to database
  allowed false
  reason "Services must use repository pattern"
}

// repo.sruja
import { boundary } from "../shared-arch/org-standards.sruja"

MySystem = system "My System" {
  MyService = container "My Service" {
    technology "Rust"
  }
  
  MyDB = database "My Database" {
    technology "PostgreSQL"
  }
}
```

### Validation
- Create shared boundary definitions in a central file
- Import them in multiple repos
- `sruja lint` validates against imported boundaries

---

## Build & Test Commands

```bash
# Check compilation
cargo check -p sruja-graph -p sruja-cli -p sruja-language

# Run tests
cargo test -p sruja-graph -p sruja-cli -p sruja-language

# Run specific test
cargo test -p sruja-cli test_cross_repo_drift
```

---

## Dependencies Between Tasks

```
4.1 (Repo Manifest) ──→ 4.2 (Cross-Repo Drift)
4.3 (Shared Schema Imports) - independent, can run in parallel
```

---

## Key Design Decisions

1. **TOML for manifest** - Human-readable, easy to edit, standard format
2. **Auto-discovery** - Sibling repos are found automatically
3. **Breaking vs Warning** - Clear severity levels for cross-repo violations
4. **Import resolution** - Relative paths from importing file
5. **Backward compatible** - Existing repos work without manifest
