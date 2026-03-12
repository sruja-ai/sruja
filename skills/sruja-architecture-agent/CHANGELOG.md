# Changelog

All notable changes to the sruja-architecture-agent skill are documented here.

**Versioning:** Skill version aligns with Sruja repo (workspace) version. Repo is at `0.10.x`; skill follows same.

## [0.10.3] – 2026-03

### Added

- **Discovery modes** – Four explicit modes: high-level-overview (systems + main containers only), standard (scope ladder), subsystem-deep-dive (one subpath, others as external systems), diff-and-refine (update existing architecture.sruja from current code; propose only changes).
- **Phased discovery playbook** – Ordered steps in SKILL and REFERENCE: (1) Deployables/runtime (Docker, K8s, Procfile, etc.) → (2) Entry points → (3) Data stores/queues → (4) Service-to-service & externals → (5) UI/frontend. Agent must follow this order; no full-codebase read.
- **REFERENCE: Discovery playbook table** – §2.0 with phases, "What to find," "Where to look," "Map to DSL."
- **REFERENCE: Deployable detection** – Table for Phase 1 (Dockerfile, docker-compose, K8s, Procfile, fly.toml, vercel.json, etc.) and how to set technology.
- **REFERENCE: Extra stacks** – Rails, Flask, Rust (Actix/Axum) in per-language hints.
- **docs/ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md** – Research summary (ArchAgent, static-analysis combination, C4+AI) and best-practices checklist; linked from SKILL, REFERENCE, AI_DOCS_INDEX, ARCHITECTURE_INTELLIGENCE_BEST_PRACTICES.

### Changed

- **Process** – Steps renumbered; added "Choose discovery mode" and "Collect (phased playbook)" with links to modes and playbook.

## [0.10.2] – 2026-03

### Changed

- **Contextual discovery** – Questions are no longer a static list. Agent must gather repo context first (e.g. run `sruja discover --context -r .` or read README/dirs/manifest), then derive 2–5 questions **tailored to the repo** (reference observed dirs, tech, size). Categories remain as guidance; each question must reference what was observed.

### Added

- **Process step 1** – Explicit “gather context, then ask contextual questions” before Understand/Collect.
- **Contextual discovery section** – Replaces static question bank with “how to derive” rules and examples (multiple dirs → which area first; single app → externals/scope; monorepo → deployables vs one system).

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
