# Sruja Scan

> Repository scanner that infers architecture graphs from source code.

## Purpose

Scans a repository using Tree-sitter to parse files across 14+ languages and merges results from package manifests (Cargo.toml, package.json, OpenAPI, Docker, Kubernetes). Enriches with SCIP data and auto-discovers deployment context (docker-compose, CI, terraform, README).

## Responsibilities

- Parse source files using Tree-sitter (TypeScript, Python, Go, Rust, Java, C#, Ruby, PHP, Kotlin, Scala, C, C++, JavaScript)
- Merge results from package manifests (Cargo.toml, package.json)
- Discover deployment context (Docker, Kubernetes, Terraform, CI)
- Generate repo maps for AI context
- Compute blast radius and confidence scores
- Detect communities in dependency graphs

## Dependencies

- **Internal**: Sruja_Language (AST parsing), Sruja_Graph_Core (graph traits)
- **External**: tree-sitter, ignore, regex, rayon, scip, prost, blake3

## Key Types

- `scan_repo(repo_root) -> Graph` — Full repository scan
- `scan_repo_incremental(repo_root) -> Graph` — Incremental scan
- `ScanConfig` — Scan configuration
- `ConfidenceScorer` — Confidence scoring for discovered elements
- `BlastRadiusResult` — Blast radius analysis result

## Code Locations

- `crates/sruja-scan/` — Scanner crate
- `src/tree_sitter.rs` — Tree-sitter integration
- `src/graph/` — Scan-scoped graph types
- `src/manifest/` — Package manifest parsing
- `src/repomap.rs` — Repo map generation

---
*Last updated: 2026-06-06*
