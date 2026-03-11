# Changelog

All notable changes to the sruja-architecture-agent skill are documented here.

## [1.2.0] – 2026-03

### Added

- **Canonical thorough-capture prompt** – Single prompt for generating from a codebase: systems, containers, technologies, descriptions, relationships (10–30 components), run `sruja lint` until pass.
- **Scope ladder** – Minimal (3–7) / Standard (10–30) / Deep (30–50); default Standard.
- **Minimal valid example** – Smallest valid .sruja template in the skill for scaling up.
- **Relationship label patterns** – Good vs bad examples (e.g. "HTTPS - auth" vs "uses").
- **Post-generate checklist** – Self-check before presenting: descriptions, technology, no orphans, specific labels, lint passes.
- **Lint error → fix table** – In REFERENCE: missing description, undefined ref, orphan, circular dependency, missing technology.
- **Circular dependency fix** – Explicit instruction: break cycle (e.g. remove one relationship), re-run lint.
- **Read order** – What to read first: README/manifest → entry points → one level of imports → config.
- **Per-language hints** – Table for Express, FastAPI, Django, Spring Boot, Next.js (entry points, technology strings).
- **User-facing super prompt** – One copy-paste prompt for Cursor/IDE chat; linked from skill.

### Changed

- **Mandatory lint** – Skill text requires running `sruja lint` before returning and fixing until pass.
- **Process step 4** – Validate step now includes "if circular dependency, break cycle and re-run lint".

## [1.1.0] – 2026-03

### Added

- **Why Sruja** – One-paragraph differentiation (machine-readable, lint/drift).
- **Canonical form** – Assignment form, `database`, specific relationship labels in REFERENCE.

## [1.0.0] – Initial

- Architecture discovery from codebases.
- REFERENCE.md with file patterns, DSL templates, detection guides.
- Support for OpenAPI, GraphQL, AsyncAPI import (documented).
