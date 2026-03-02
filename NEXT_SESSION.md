# Sruja Development Context

**Strategy:** See [architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md) for the canonical Architecture Intelligence direction, module decisions, and execution plan.

**Current implementation status:** [docs/ARCHITECTURE_INTELLIGENCE.md](docs/ARCHITECTURE_INTELLIGENCE.md) (§ Current state: where we're standing) — CLI full flow (quickstart, drift, why, analyze) implemented; desktop has repo + query only; no `sruja.toml` yet.

**🎉 BUYABLE FEATURES COMPLETE:** See [BUYABLE_FEATURES_TEST_REPORT.md](BUYABLE_FEATURES_TEST_REPORT.md) for comprehensive test results on real GitHub projects.

---

## ✅ Completed: Buyable Features (2026-03-02)

### 1. Context Export for AI Tools
```bash
sruja context -f cursor-rules -r .                    # Cursor IDE
sruja context -f copilot-instructions -r .            # GitHub Copilot
sruja context -f json -o context.json                  # Custom tools
```
**Status**: ✅ Implemented, tested on Express/Gitea/Saleor  
**File**: `crates/sruja-cli/src/commands/context.rs`

### 2. One-Click Baseline Generation
```bash
sruja quickstart -r . --generate-baseline
# Creates: architecture.sruja (editable DSL)
```
**Status**: ✅ Implemented, tested on Express/Sruja  
**File**: `crates/sruja-cli/src/commands/scan.rs:686-759`

### 3. PR-Scoped Drift Detection
```bash
sruja drift-pr -r . -b origin/main -f github-actions  # CI format
sruja drift-pr -r . -b origin/main -f json            # JSON output
```
**Status**: ✅ Implemented, tested with 446 changed files  
**File**: `crates/sruja-cli/src/commands/scan.rs:761-881`

### 4. GitHub Action Integration
**Status**: ✅ Implemented  
**File**: `.github/workflows/sruja-drift.yml`

---

## Next Steps (Prioritized)

### Priority 1: Polish & Documentation (1-2 days)

#### 1.1 Update Documentation
- [ ] Add command examples to README.md
- [ ] Create `docs/AI_INTEGRATION.md` for context export
- [ ] Update `docs/ARCHITECTURE_AGENT.md` with baseline generation
- [ ] Add CI integration examples to docs

#### 1.2 Improve Baseline Generation
**File**: `crates/sruja-cli/src/commands/scan.rs:686-759`

**Issues to Fix**:
- Duplicate IDs (multiple "index" containers)
- Generic naming from file paths
- Missing inferred relationships

**Enhancements**:
- Infer layer from directory structure (api/, services/, data/)
- Use package.json name as system name
- Detect framework patterns (Express, Django, Rails)
- Add inferred boundaries based on layer rules

#### 1.3 Add Tests
Create these test files:
```bash
crates/sruja-cli/tests/context_export_e2e.rs
crates/sruja-cli/tests/baseline_generation_e2e.rs
crates/sruja-cli/tests/drift_pr_e2e.rs
```

### Priority 2: IDE Integration (2-3 days)

#### 2.1 VS Code Extension Enhancements
**File**: `extension/src/extension.ts`

Add commands:
- `sruja.exportContext` - Export cursor-rules
- `sruja.generateBaseline` - Generate architecture.sruja
- `sruja.checkDrift` - Run drift in IDE

#### 2.2 Real-time Drift Feedback
Show drift warnings on save in VS Code

### Priority 3: Performance Optimizations (1-2 days)

#### 3.1 Incremental Scanning
**File**: `crates/sruja-scan/src/lib.rs`

Only rescan changed files for 10x faster large repo performance

#### 3.2 Graph Caching
**File**: `crates/sruja-cli/src/graph_store.rs`

Enhance with git-ref-based caching

### Priority 4: More Language Support (3-5 days)

- [ ] Add Java support (tree-sitter-java)
- [ ] Add C# support (tree-sitter-c-sharp)
- [ ] Test on real Java/C# projects

---

## Known Issues

### Issue 1: Duplicate IDs in Baseline
**Location**: `crates/sruja-cli/src/commands/scan.rs:749`

**Problem**: Multiple files named "index.js" create duplicate container IDs

**Fix**: Use full path or unique identifier

### Issue 2: Git Ref Validation
**Location**: `crates/sruja-cli/src/commands/scan.rs:776`

**Problem**: Doesn't validate if git ref exists before scanning

**Fix**: Add `git rev-parse --verify` check

### Issue 3: Large Repo Memory Usage
**Location**: `crates/sruja-scan/src/lib.rs`

**Problem**: Gitea (14K modules) uses ~2GB memory

**Fix**: Stream processing instead of loading all into memory

---

## Test Repositories

Test on real projects:
```bash
evaluation/real-world-test/test-repos/
├── express/         # Node.js (76 modules)
├── gitea/           # Go (14,742 modules)
├── saleor/          # Python (2,500+ modules)
├── redis/           # C
├── etcd/            # Go
└── temporal/        # Go
```

---

## Quick Start Commands

### Development
```bash
# Build
cargo build --release

# Test new features
./target/release/sruja context -f cursor-rules -r .
./target/release/sruja quickstart -r . --generate-baseline
./target/release/sruja drift-pr -r . -b HEAD~20 -f json
```

### Testing
```bash
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

---

## Key Files to Know

### Core Implementation
```
crates/sruja-cli/src/commands/
├── context.rs          # Context export (NEW)
├── scan.rs             # Baseline gen + drift-pr (ENHANCED)
└── mod.rs              # Command exports

.github/workflows/
└── sruja-drift.yml     # GitHub Action (NEW)
```

### Documentation
```
README.md                         # Updated with new features
BUYABLE_FEATURES_TEST_REPORT.md   # Test results
docs/ARCHITECTURE_INTELLIGENCE.md # Main docs
```

---

## Phase 1 Status (from FINAL §10)

1. ~~Implement `quickstart` command.~~ **Done.**
2. ~~Improve deterministic `why` explanations with evidence templates.~~ **Done.**
3. ~~Unify drift logic with `sruja-diff`.~~ **Done.**
4. ~~Reorder README/docs around no-key first value.~~ **Done.**

### Tactical TODO
- ✅ ~~Refactor `sruja drift` – move heuristics to `sruja-diff` core~~
- ✅ ~~Improve `sruja why` – evidence templates, deterministic answers without LLM~~
- Desktop app: add docs/intent config, drift/analyze in UI (see ARCHITECTURE_INTELLIGENCE.md current state)

---

**Remember**: The goal is to make Sruja so valuable that teams will want to support it as an OSS project. Every feature should deliver daily value in real-world workflows. 🚀
