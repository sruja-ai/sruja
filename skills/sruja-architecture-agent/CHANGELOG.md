# Changelog

All notable changes to the sruja-architecture-agent skill are documented here.

**Versioning:** Skill version aligns with Sruja repo (workspace) version. Repo is at `0.10.x`; skill follows same.

## [0.10.4] – 2026-03

### Added

- **Gather → Ask → Build** – Explicit three-step principle: (1) Gather evidence (discover + playbook), (2) Ask 2–5 targeted questions when scope/boundaries/externals/flows are ambiguous, (3) Build architecture only after answers or "proceed with defaults." Reduces guessing; improves accuracy.
- **Question taxonomy** – In SKILL: table of question categories (Scope/area, Boundaries, Externals, Entry/flows, Intent) with "when to ask" and example questions. Agent picks by what's ambiguous.
- **REFERENCE: Deriving the right questions from repo context** – Repo-signal → question-category table (multiple dirs, monorepo, env vars, multiple entry points, no deploy files, docs) with example questions and workflow: gather → pick 2–5 questions → ask → then generate.
- **Recommended developer experience** – In SKILL "For end users": encourage answering the agent's questions for best results; single-prompt still supported.

### Changed

- **Core principle** – Skill intro now states "do not guess": gather evidence first, ask the right questions when information is missing or ambiguous, then build from confirmed information.
- **Contextual discovery** – Added "Only after answers (or explicit 'proceed with defaults') should you generate the full architecture." Link to REFERENCE "Deriving the right questions."
- **Evidence / question triggers** – Renamed "Good question triggers" to "Question triggers (ask instead of guessing)"; added vague request on large repo.

### Documentation

- **docs/ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md** – New §2.6 "Ask the right questions before building (do not guess)" with practice and REFERENCE link; new summary-table row. §2.7/2.8 renumbered.

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
