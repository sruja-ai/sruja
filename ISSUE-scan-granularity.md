# feat: language-aware scan granularity to eliminate false positives

## Problem

Static checks (drift, compliance, quickstart) produce high noise across all languages due to treating every file as an architecture element.

### Noise Sources

| Language | Barrel Pattern | False Positive |
|----------|----------------|----------------|
| Rust | mod.rs, lib.rs | Orphan, GodModule |
| Python | __init__.py | Orphan |
| JS/TS | index.ts, barrel exports | Orphan |
| Go | package files | Orphan |
| Java | package-info.java | Orphan |

### Current Impact (sruja repo example)

- 230 GodModule violations (lib.rs with 49 deps is normal)
- 14 OrphanComponent violations (mod.rs are barrel exports)
- 10026 intent drifts (file-level modules not in DSL)
- drift-pr is clean (0 new violations)

## Proposed Solution

### 1. Language-aware barrel detection
Scanner should recognize and skip barrel patterns:
- Rust: mod.rs, lib.rs re-exports
- Python: __init__.py imports
- JS/TS: index.ts barrel exports
- Go: package-level grouping

### 2. Granularity config
Add --granularity flag to scan command:
- file (current default)
- crate/package (matches DSL level)
- function (finer grain)

### 3. Severity gating
Only report Warning+ severity, skip Info-level false positives.

## Acceptance Criteria

- [ ] mod.rs/__init__.py/index.ts not flagged as orphans
- [ ] lib.rs re-exports not flagged as god modules
- [ ] --granularity crate scans at package level
- [ ] Static checks noise reduced by >80%
- [ ] drift-pr remains unchanged (already clean)

## Related

- sruja-check.yml already uses --violations-only for noise filtering
- scan_scope.rs has is_path_production_relevant() but unused in drift
