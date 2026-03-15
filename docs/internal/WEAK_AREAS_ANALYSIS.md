# Weak Areas Analysis

Deep-dive into Sruja's functional limitations with code evidence. Generated from codebase analysis on 2026-03-14.

---

## 1. Scanner & Analysis Limitations

### 1.1 Dynamic Code Analysis Blind Spots

**Issue:** Cannot detect runtime-loaded dependencies.

**Code Evidence:**

The scanner only processes static import statements:

```rust
// crates/sruja-scan/src/tree_sitter.rs:140-154
for import in &parsed.imports {
    let target_id = resolve_import_improved(
        repo_root,
        &repo_canon,
        path,
        import,
        &file_path_to_id,
        go_module_path.as_deref(),
        language,
    );
    // Only handles string literal imports, not dynamic expressions
}
```

**Documented Gap** (`KNOWN_LIMITATIONS.md:9-21`):

```markdown
### Dynamic Imports and Reflection
**Issue:** Sruja may miss dependencies that are loaded dynamically at runtime.
- `require()` calls with dynamic paths (e.g., `require('./' + module + '.js')`)
- `import()` expressions with variable paths
- Dependency injection frameworks that configure dependencies at runtime
- Plugin systems that load modules dynamically
```

**Impact:** Incomplete architecture graph requiring manual supplementation.

**Mitigation:** Document dynamic dependencies explicitly in `.sruja` files.

---

### 1.2 Framework Routing Gaps

**Issue:** Misses framework-specific routing conventions.

**Affected Patterns:**

| Framework | Pattern | Status |
|-----------|---------|--------|
| Next.js/Remix | File-based routing | Not detected |
| Spring Boot | `@GetMapping`/`@Route` annotations | Partially detected |
| NestJS | Annotation-based routing | Partially detected |
| Dynamic routes | Generated at runtime | Not detected |

**Code Evidence:**

Service detection relies on content heuristics, not AST-level routing analysis:

```rust
// crates/sruja-scan/src/tree_sitter.rs:420-443
fn is_java_service_entry_point(content_lower: &str, path_str: &str) -> bool {
    let spring_patterns = [
        "@springbootapplication",
        "@restcontroller",
        "@controller",
        "@service",
        // Missing: @GetMapping, @PostMapping, @RequestMapping routing annotations
    ];
    spring_patterns.iter().any(|p| content_lower.contains(p))
}
```

**Impact:** API endpoints/service boundaries may be missed.

---

### 1.3 Language Support Inconsistency

**Kotlin Degraded Mode:**

```rust
// crates/sruja-scan/src/tree_sitter/languages/kotlin.rs:10-12
// tree-sitter-kotlin 0.3 depends on tree-sitter 0.20; we use 0.24, so we cannot
// use the crate's Language with our Parser. Use a minimal line-based extraction instead.
```

**What's Lost:**

```rust
// crates/sruja-scan/src/tree_sitter/languages/kotlin.rs:21-44
pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    // Only extracts: imports, class definitions
    // Missing: nested structures, precise dependency analysis, function calls
    
    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("import ") {
            imports.push(path_str.to_string());
        } else if let Some(rest) = line.strip_prefix("class ") {
            // Basic class extraction only
        }
    }
}
```

**Impact:** Reduced precision for Kotlin codebases.

---

## 2. Scanner Accuracy Issues

### 2.1 Service Detection Heuristics

**Issue:** Complex heuristics may misclassify.

**Code Evidence** (`tree_sitter.rs:344-403`):

```rust
fn infer_node_kind(parsed: &ParsedFile, path: &Path, content: &str) -> NodeKind {
    // Early exit: detect CLI tools and exclude from service detection
    if is_cli_tool(&path_str, &content_lower) {
        return NodeKind::Module;
    }

    // Check for deployment configs (highest confidence)
    if is_deployment_config(&path_str) {
        return NodeKind::Service;
    }

    // Multiple fallback levels with varying confidence:
    // 1. Go cmd/*/main.go pattern
    // 2. Java Spring Boot annotations
    // 3. JS/TS server patterns
    // 4. Python Flask/Django/FastAPI
    // 5. Framework-specific patterns
    // 6. Path/name heuristics (conservative)
    
    NodeKind::Module  // Default fallback
}
```

**Potential Misclassifications:**

| Scenario | Risk |
|----------|------|
| Library with `app.listen()` in examples | May be classified as service |
| CLI tool without common framework patterns | May be classified as service |
| Microservice without obvious framework markers | May be classified as module |

---

### 2.2 Module Boundaries Misalignment

**Issue:** File/directory structure may not match architectural boundaries.

**Code Evidence:**

```rust
// crates/sruja-scan/src/tree_sitter.rs:90-96
let parent_module = path
    .parent()
    .and_then(|p| p.strip_prefix(repo_root).ok())
    .map(|p| p.to_string_lossy().replace(['/', '\\'], "_"))
    .unwrap_or_else(|| "root".to_string());

let module_id = format!("module:{}", parent_module);
```

**Impact:** Module nodes may not reflect:
- Logical bounded contexts (DDD)
- Actual runtime service boundaries
- Architectural modules spanning multiple directories

---

## 3. Performance & Scalability

### 3.1 Sequential Scanning (No Parallelism)

**Issue:** Scanner processes files sequentially.

**Code Evidence:**

```rust
// crates/sruja-scan/src/tree_sitter.rs:70-193
let walker = build_walker(repo_root, config);

for entry in walker {  // Sequential iteration, no rayon/parallel
    let entry = entry.map_err(|e| ScanError::Walk(e.to_string()))?;
    // ... process file
}
```

**Contrast with Validator:**

```rust
// crates/sruja-engine/src/validator/core.rs:144-161
#[cfg(feature = "async")]
pub async fn validate(&self, program: Arc<Program>) -> Vec<Diagnostic> {
    if !self.config.parallel || self.rules.len() <= 1 {
        return self.validate_sync(&program);
    }
    
    let mut tasks = Vec::new();
    for rule in self.rules.clone() {
        let task = tokio::spawn(async move { /* parallel execution */ });
        tasks.push(task);
    }
}
```

**Impact:** Large monorepos (10k+ files) may have slow scan times.

---

### 3.2 Real-World Scale Testing

**Tested Repos:**

| Repo | Language | Components | Health | Notes |
|------|----------|------------|--------|-------|
| Express | JavaScript | 9 | 100/100 | Clean baseline |
| Caddy | Go | 1,977 | 99/100 | God modules in cmd/ |
| react-admin | TypeScript | 1,410 | 84/100 | 9 circular deps |
| Redis | C | 9,793 | 98/100 | Vendored deps inflate count |
| Gitea | Go | 14,742 | 78/100 | 892 god modules, 1 cycle |

**Evidence:** `evaluation/real-world-test/OSS_TEST_RESULTS.md`, `VALUE_ON_REAL_PROJECTS.md`

**Missing:**
- No benchmark suite
- No memory profiling
- No performance regression tests
- No parallel scanning implementation

---

## 4. CI/CD Integration Challenges

### 4.1 First-Run Blocking in Brownfield Projects

**Issue:** Initial CI run fails on all existing violations.

**Documented** (`KNOWN_LIMITATIONS.md:116-128`):

```markdown
### First-Run False Positives
**Issue:** When first adding Sruja to CI/CD, existing architectural violations 
will fail the pipeline.

**Mitigation:**
- Use `sruja drift-pr` in CI to detect only NEW violations introduced in a PR
- Gradually address existing violations outside of CI
- Configure `sruja drift --fail-on` to fail only on specific violation types
```

### 4.2 drift-pr Solution (Partial)

**Implementation:**

```rust
// crates/sruja-cli/src/commands/scan.rs:953-1048
pub async fn drift_pr(
    repo_root: &str,
    base_ref: Option<&str>,
    head_ref: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    // Only checks NEW violations between base and head
    let changed_files_output = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{}...{}", base, head)])
        .output();
    
    // Caches graphs by commit SHA for CI retry performance
    let head_cache_path = cache_dir.join(format!("head_{}.json", head_sha));
}
```

**Limitation:** Still requires full repo scan for both base and head (not file-incremental).

**CI Template:**

```yaml
# templates/github-actions/sruja-architecture-pr.yml:27-29
- name: PR-scoped drift check (new violations only)
  run: |
    sruja drift-pr -r . --base origin/${{ github.base_ref }} --head HEAD -f github-actions
```

---

## 5. IDE Integration

### 5.1 VS Code Extension (Stable)

**Features** (`extension/README.md:7-16`):

| Feature | Implementation |
|---------|----------------|
| Syntax highlighting | TextMate grammar |
| Diagnostics | Bundled WASM (no CLI needed) |
| Diagram preview | WASM → Mermaid in webview |
| Export to Markdown | WASM-based |
| Drift detection | Requires CLI |

```markdown
// extension/README.md:3-4
VS Code extension for the Sruja architecture DSL. **Lint and Markdown export run 
in-process using bundled WebAssembly** (no CLI required).
```

### 5.2 Missing IDE Support

| IDE | Status |
|-----|--------|
| VS Code | ✅ Stable |
| JetBrains (IntelliJ, WebStorm, etc.) | ❌ Not implemented |
| LSP Server | ❌ Not implemented |
| Real-time feedback on save | ⚠️ Debounced lint only |

---

## 6. Intent System Limitations

### 6.1 Documentation Dependence

**Issue:** Intent features require pre-existing documentation.

**Documented** (`KNOWN_LIMITATIONS.md:79-93`):

```markdown
### Semantic Layer Weakness Without ADRs
**Issue:** The "why" and semantic analysis features provide limited value 
without rich Architecture Decision Records (ADRs) or intent documentation.

**Impact:**
- Answers to "why are we using X?" may be generic or low-confidence
- Semantic analysis may not detect the intended purpose of components
- Context exports may lack business rationale

**Mitigation:**
- Maintain ADRs in your repository (e.g., in `docs/adr/`)
- Use `sruja intent check` to validate alignment
- For code-only repos, rely on structural analysis (`quickstart`, `drift`)
```

### 6.2 Limited Intent Expression Forms

**Current Support:**
- ADRs in `docs/adr/` directory
- `.sruja` files

**Missing:**
- Design documents (Google Doc style)
- Wiki pages
- RFC documents
- Issue tracker integration

---

## 7. Missing DSL Features

**Documented in DESIGN_PHILOSOPHY.md:277-294:**

```markdown
### Currently Missing (but important):
1. **Constraints/Validation**: How to express "email must be valid", "age > 0"?
2. **Relationships with Cardinality**: `User -> Order[1:*]` (one-to-many)?
3. **Inheritance/Polymorphism**: How to model "Payment extends Transaction"?
4. **Enums**: `status: OrderStatus` where `OrderStatus = [PENDING, COMPLETED, CANCELLED]`
5. **Optional Fields**: `email?: string` vs `email string`
6. **Defaults**: `status string = "PENDING"`
7. **Computed Fields**: `total: float = items.sum(price * qty)`

### Recommendations:
- Add `enum` keyword for enumerations
- Support `?` for optional fields: `email? string`
- Support `=` for defaults: `status string = "PENDING"`
- Consider `constraint` keyword for validation rules
- Consider relationship syntax: `User -> Order[1:*]`
```

**Priority Order** (from DESIGN_PHILOSOPHY.md:568-577):

```markdown
**Next Steps:**
1. Add `enum` support
2. Add optional fields (`?` syntax)
3. Add relationship cardinality
4. Add constraint/validation syntax
5. ✅ `flow` and `scenario`/`story` already implemented
6. Improve error messages for beginners
7. Add validation rules to guide simplicity
```

---

## 8. Scope Limitations (Intentional)

### 8.1 Not a Full Architecture Evaluation Tool

**Documented** (`KNOWN_LIMITATIONS.md:131-142`):

```markdown
### Not a Replacement for Design Review
**Issue:** Sruja is a tool for structural analysis and drift detection, 
not a complete replacement for architectural design review.

**Limitations:**
- Does not evaluate non-functional requirements (performance, security, scalability)
- Does not assess business logic correctness
- Does not replace human judgment in architectural decisions
```

---

## 9. Summary Table

| Area | Severity | Status | Code Location |
|------|----------|--------|---------------|
| Dynamic imports not detected | High | Documented gap | `tree_sitter.rs`, `KNOWN_LIMITATIONS.md` |
| Framework routing gaps | Medium | Partial heuristics | `tree_sitter.rs:344-627` |
| Kotlin degraded mode | Medium | Tree-sitter version conflict | `languages/kotlin.rs:10-12` |
| Sequential scanning | Medium | No parallel impl | `tree_sitter.rs:70-193` |
| CI brownfield friction | High | Partial fix via `drift-pr` | `commands/scan.rs:953-1048` |
| JetBrains IDE support | Low | Not implemented | N/A |
| LSP server | Low | Not implemented | N/A |
| Intent without ADRs | Medium | Documented limitation | `KNOWN_LIMITATIONS.md:79-93` |
| Missing DSL features | Medium | Planned in roadmap | `DESIGN_PHILOSOPHY.md:277-294` |
| Large repo performance | Unknown | Tested to 14k components | `OSS_TEST_RESULTS.md` |

---

## 10. Recommendations

### High Priority

1. **Parallel scanning** - Add rayon or tokio-based parallel file processing
2. **File-incremental drift-pr** - Only scan changed files, not full repo
3. **Baseline/snapshot feature** - Allow teams to snapshot current violations as baseline

### Medium Priority

4. **Tree-sitter version alignment** - Resolve Kotlin and other language grammar version conflicts
5. **Framework routing detection** - AST-level routing annotation analysis
6. **Performance benchmark suite** - Automated performance regression testing

### Low Priority

7. **LSP server** - Enable broader IDE support
8. **Intent auto-generation** - Generate initial ADRs from inferred patterns
9. **Memory profiling** - Document memory characteristics for large repos

---

## References

- `docs/KNOWN_LIMITATIONS.md` - Official limitations documentation
- `docs/DESIGN_PHILOSOPHY.md` - DSL roadmap and missing features
- `evaluation/real-world-test/OSS_TEST_RESULTS.md` - Scale testing results
- `extension/README.md` - VS Code extension capabilities
- `templates/github-actions/sruja-architecture-pr.yml` - CI integration pattern
