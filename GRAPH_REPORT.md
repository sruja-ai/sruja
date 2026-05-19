# Sruja Discovery Explanation

**Repo:** .
**Scan summary:** 1732 node(s), 1460 relationship(s)
**Primary language:** Rust
**Architecture style:** monolith
**Element mix:** custom: 291, database: 1, module: 1440

## Why Sruja Thinks That

- No service nodes were detected, so Sruja is currently reading this repo mostly as a module-level graph inside one codebase.
- No strong framework markers were detected, so Sruja is leaning more on file structure and dependency edges.
- Most discovered elements cluster under `crates`, `book`, `extension`, which is where the scanner sees the clearest architectural seams.
- Found 1137 exported interface node(s), which gives the scan stable API surfaces to anchor on.

## Top Directories

- `crates`: 1284 node(s)
- `book`: 205 node(s)
- `extension`: 113 node(s)
- `docs`: 39 node(s)
- `skills`: 26 node(s)

## Discovery Confidence

**Level:** INFERRED
- [✓] Static analysis produced 1732 node(s) and 1460 relationship(s).
- [✓] 1732 of those node(s) map back to concrete file paths.
- [✓] The scan found clear top-level hotspots in `crates`, `book`, `extension`.
- [?] This is static analysis, so runtime-only calls, reflection, and generated code can still be missing.
- [?] Ownership, domain labels, and external system names are strongest after a reviewed repo.sruja baseline exists.
- [?] Because the framework is unclear, boundary naming may need extra human review before you commit a baseline.

## God Nodes (High-Signal Elements)

- `crate:sruja-diagnostics` [module] (crates/sruja-diagnostics/Cargo.toml): Mostly acts as a shared dependency with 8 incoming reference(s).
- `crate:sruja-language` [module] (crates/sruja-language/Cargo.toml): Mostly acts as a shared dependency with 10 incoming reference(s).
- `extension_src_wasm_ts` [module] (extension/src/wasm.ts): Mostly acts as a caller with 17 outgoing dependency(ies).
- `crate:sruja-scan` [module] (crates/sruja-scan/Cargo.toml): Mostly acts as a shared dependency with 5 incoming reference(s).
- `extension_src_skills_ts` [module] (extension/src/skills.ts): Mostly acts as a shared dependency with 4 incoming reference(s).

## Key Relationships

- `crate:sruja-language` -> `crate:sruja-diagnostics` [calls]: Represents a meaningful internal dependency worth validating as a boundary. Backed by 1 evidence item(s).
- `crate:sruja-engine` -> `crate:sruja-diagnostics` [calls]: Represents a meaningful internal dependency worth validating as a boundary. Backed by 1 evidence item(s).
- `crate:sruja-export` -> `crate:sruja-diagnostics` [calls]: Represents a meaningful internal dependency worth validating as a boundary. Backed by 1 evidence item(s).
- `crate:sruja-agent` -> `crate:sruja-diagnostics` [calls]: Represents a meaningful internal dependency worth validating as a boundary. Backed by 1 evidence item(s).
- `crate:sruja-diff` -> `crate:sruja-diagnostics` [calls]: Represents a meaningful internal dependency worth validating as a boundary. Backed by 1 evidence item(s).

## Suggested Questions

- Why is `crate:sruja-diagnostics` a central hub (incoming: 8, outgoing: 0)? Should its responsibilities be split?
- Why is `crate:sruja-language` a central hub (incoming: 10, outgoing: 1)? Should its responsibilities be split?

## Confidence

- Level: INFERRED
- Signal: Static analysis produced 1732 node(s) and 1460 relationship(s).
- Signal: 1732 of those node(s) map back to concrete file paths.
- Signal: The scan found clear top-level hotspots in `crates`, `book`, `extension`.
- Blind spot: This is static analysis, so runtime-only calls, reflection, and generated code can still be missing.
- Blind spot: Ownership, domain labels, and external system names are strongest after a reviewed repo.sruja baseline exists.
- Blind spot: Because the framework is unclear, boundary naming may need extra human review before you commit a baseline.

## Next Steps

- Review the highlighted elements and rename or regroup them in `repo.sruja` if they do not match your team language.
- Run `sruja quickstart -r . --generate-baseline` for a structural draft (`repo.sruja.draft`), then author reviewed intent in `repo.sruja` with the sruja-architecture skill.
- After `repo.sruja` exists, run `sruja drift -r . -a repo.sruja` in CI to keep declared architecture aligned with code.
