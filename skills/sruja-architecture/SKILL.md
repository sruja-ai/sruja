---
name: sruja-architecture
description: >
  Discovers architecture from codebases and authors Sruja DSL (repo.sruja). Use when
  discovering architecture, generating or refactoring repo.sruja, validating architecture
  against code, maintaining architecture docs, or when the user mentions architecture-as-code,
  C4, or .sruja files.
license: Apache-2.0
---

# Sruja Architecture Skill

Deterministic, evidence-first workflow for generating and maintaining `repo.sruja` (or `architecture.sruja`). The skill guides the AI to write valid architecture from real code evidence; no DSL expertise required.

## Quick Start

1. **Evidence** — If `.sruja/context.json` exists and is recent (e.g. `updated_at` within last hour), use it. Else run `sruja sync -r .` (do not ask the user to run it).
2. **Questions** — Ask 2–5 targeted questions only when evidence is ambiguous (boundaries, externals, datastores, deployment, data flows).
3. **Generate** — Produce minimal `repo.sruja` from evidence (C4 context + containers; components only when justified).
4. **Validate** — Run `sruja lint repo.sruja` and fix all errors before considering complete.
5. **Refine** — Optionally run `sruja drift -r .` and update DSL from drift results.
6. **Impact (Optional)** — Before refactoring code, run `sruja impact <target> -r . --depth 3` to quickly see upstream dependents and downstream dependencies.

Workflow checklist: `[ ] Evidence gathered → [ ] Questions (if needed) → [ ] repo.sruja generated → [ ] sruja lint passed → [ ] Open questions listed (no guessing)`

## Core Principles

- **Evidence-first** — Gather evidence before modeling.
- **No guessing** — Surface open questions instead of fabricating answers.
- **Minimal DSL** — Generate only what evidence supports.
- **Validation** — Always lint and fix errors before considering complete.

## Evidence and Discovery

Discovery is backed by a **static analysis graph** (Tree-sitter): modules, imports, and dependencies from supported languages (TypeScript, Python, Go, Rust, Java, C#, Ruby, etc.). Use this graph as the single source of truth; prefer it over guessing.

**Progressive loading (large or multi-repo):**

| Tier | Source | Use when |
|------|--------|----------|
| 1 | `.sruja/context.json` (summary) | Default: "what areas exist?", "how big?" |
| 2 | `.sruja/graph.json` (slice by area/module) | Reasoning about a specific area (e.g. "dependencies of auth") |
| 3 | Full `.sruja/graph.json` or `sruja scan -r . -o -` | Deep task: full dependency list, export |

If context is missing or stale, run `sruja sync -r .` or `sruja discover --context -r . --format json` yourself. After generating, suggest **Sruja: Refresh repo context** or `sruja sync -r .` for next time.

Evidence provides: repository structure, detected technologies, module boundaries, entry points, external dependencies, scan scope.

## Workflow (Detail)

### 1. Collect Evidence

Prefer `.sruja/context.json` when present and recent (`updated_at`, `truth_status`, `baseline_path`, `git_commit`). If missing or stale, run:

```bash
sruja sync -r .
```

Or JSON only: `sruja discover --context -r . --format json`

### 2. Ask Targeted Questions

Only when evidence is ambiguous:

- Main system boundaries? External services? Datastores? Deployment? Main data flows?

Do not ask about information already clear from evidence.

### 3. Generate Minimal DSL

Minimal `repo.sruja` from evidence. C4 context and container levels first; component level only when evidence justifies it.

```sruja
import { * } from 'sruja.ai/stdlib'

Person = person "Person"
ExternalAPI = system "External API" { description "Third-party or backend service" }

System = system "System" {
  Container = container "Container" {
    technology "Technology"
    description "Description"
  }
}

Person -> System.Container "Protocol"
System.Container -> ExternalAPI "HTTPS"
```

**Component knowledge (opt-in):** Ask the user if they want per-component markdown under `.sruja/knowledge/`. If yes: one file per component (e.g. `.sruja/knowledge/PaymentService.md`), add `doc ".sruja/knowledge/<Id>.md"` on the element, use `references/KNOWLEDGE_TEMPLATE.md`. Scope: containers and key systems; quality over coverage.

**Doc references — markdown only:** In `.sruja` files, `doc` must point to **markdown** (e.g. `.sruja/knowledge/Component.md`, `docs/README.md`, or a generated knowledge base). Do **not** set `doc` to source code paths (e.g. `.rs`, `.ts`, `crates/...`). The markdown itself may link to or reference code; the separation is: Sruja DSL elements reference markdown via `doc`, and that markdown can refer to files.

**Sources (specs and infra artifacts):** Use `source <kind> "<path>"` inside an element body to bind code-adjacent artifacts (OpenAPI specs, Kubernetes manifests, Terraform modules, docs) to the architecture element they describe. This keeps architecture grounded in real, reviewable files without requiring the user to write DSL.

```sruja
Payments = container "Payments API" {
  technology "Go"
  description "Handles payment processing"

  source openapi "./specs/payments.yaml"
  source kubernetes "./k8s/payments/"
  source terraform "./infra/payments/"
  source docs "./docs/payments.md"
}
```

Supported kinds include: `openapi`, `asyncapi`, `kubernetes` (`k8s`), `dockerfile` (`docker`), `terraform` (`tf`), `docs` (`doc`), `readme`, `proto` (`protobuf`), `config`, `graphql` (`gql`), `helm`, and `custom`.

### 4. Validate and Repair

```bash
sruja lint repo.sruja
```

Fix all errors. Common: E201 (invalid kind), E204 (circular deps), E205 (orphans), E206 (invalid refs).

### 5. Refine (Optional)

```bash
sruja drift -r .
```

Use drift results to refine `repo.sruja`. Use `-a path` to specify a different architecture file.

## Operating Modes

1. **Local authoring** — Create/update `repo.sruja` from evidence + questions; generate minimal DSL; run `sruja lint`.
2. **System context** — Use discover output or `.sruja/context.json` for the slice relevant to the task; prefer canonical IDs from evidence.
3. **Drift refinement** — After code/intent changes, use `sruja drift -r .` (and optionally `sruja intent propose`) to turn deltas into DSL updates or open questions; do not invent without evidence.
4. **Multi-repo** — When `system.index.json` exists (from `sruja compose`), load only the **impacted slice**; use canonical IDs `repo_id::local_id`; check `conflicts` and ownership. See **docs/FEDERATION.md**.

## When to Apply

- Discovering architecture from a new codebase
- Generating initial repo.sruja from requirements
- Refactoring or updating existing architecture
- Validating architecture against code
- Maintaining architecture documentation

## What NOT to Do

- Do not guess missing information — list open questions.
- Do not add framework/domain/security narratives without evidence.
- Do not create components for completeness only.
- Do not add relationships unsupported by evidence.
- Do not skip linting.
- Do not set `doc` on elements to source code paths (e.g. `crates/foo/src/lib.rs`, `extension/src/bar.ts`). Use markdown only; the markdown can reference code.

## Open Questions

Surface missing or unclear information explicitly in comments:

```sruja
// OPEN QUESTIONS:
// - How is authentication implemented?
// - What message queue is used for async operations?
// - External API integrations not detected?
```

## Multi-repo (Publish and Compose)

**Setup:** See [FEDERATION_SETUP_GUIDE.md](../../docs/FEDERATION_SETUP_GUIDE.md) for step-by-step instructions.

**Workflow:**
1. **Per-repo:** Generate `repo.sruja` in each repository (same as single-repo workflow)
2. **Publish:** In each repo: `sruja publish -r . -o repo.bundle.json`
3. **Collect:** Copy bundles to shared location (rename to avoid collisions: `api.repo.bundle.json`)
4. **Compose:** `sruja compose -i <dir> -o system.index.json` builds unified graph with canonical IDs
5. **Resolve conflicts:** Check `conflicts` array for duplicate kind+label across repos

**When working across repos:**
- Use canonical IDs: `repo_id::local_id` (e.g., `api-service::PaymentAPI`)
- Load only impacted slice from `system.index.json`, not full graph
- Check ownership before modifying cross-repo relationships

**Prompt for multi-repo setup:**
```
Use sruja-architecture. Help me set up federation across multiple repos.
Guide me through: 1) generating repo.sruja in each repo, 2) publishing bundles,
3) composing system.index.json. Reference docs/FEDERATION_SETUP_GUIDE.md for steps.
```

See **docs/FEDERATION.md** for artifact schemas and retrieval behavior.

## Progressive Discovery — What to Load When

Do not load `references/AGENTS.md`, `references/REFERENCE.md`, or entire `rules/` at once. Load only what the task needs:

| Task | Load only |
|------|-----------|
| Baseline / discovery | `rules/sdlc/create-phase.md`, `references/PROMPTS.md` (Discovery) |
| Component knowledge | `references/KNOWLEDGE_TEMPLATE.md` |
| Update / drift | `rules/sdlc/update-phase.md`, `references/REFERENCE.md` (SDLC update) |
| Impact analysis | `rules/query/impact-analysis.md` |
| Requirement traceability | `rules/requirements/capture-requirements.md`, `rules/requirements/link-requirements.md`; `references/PROMPTS.md` if needed |
| Compliance | `rules/query/compliance-check.md` |
| Deep semantic discovery | `rules/query/scip-discovery.md` |
| Full design / refactor / patterns | `references/AGENTS.md` |

## Related References

- Discovery and refinement: `references/REFERENCE.md`
- Modeling rules: `rules/` (per-task; see table above)
- Prompts: `references/PROMPTS.md`; `references/AGENTS.md` only for full design/refactor
- Multi-repo setup: **docs/FEDERATION_SETUP_GUIDE.md** (step-by-step)
- Multi-repo reference: **docs/FEDERATION.md** (schemas, commands)

## Prerequisites and Installation

**CLI:** Workflow uses `sruja sync`, `sruja discover`, `sruja lint`, `sruja drift`, and (optionally) `sruja impact`. Install a CLI that includes `sync` and `discover`:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Or from repo: `cargo install --path crates/sruja-cli`. Verify with `sruja --help` (must list `sync` and `discover`). If the CLI lacks these, the skill falls back to repo structure and codebase only; upgrade via the install script or repo build.

**Extension (optional):** [Sruja extension](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja) — syntax, diagnostics, **Sruja: Run validation**, diagram preview, export to Markdown, **Sruja: Refresh repo context**. When installed, run Refresh once (or after big changes); the skill prefers `.sruja/context.json`.

**Install skill:**

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Retrieval Order (Architecture-Aware Codegen/Review)

1. Local repo truth — `repo.sruja` (or `architecture.sruja`)
2. Fresh evidence — `.sruja/context.json` (Tree-sitter); if missing/stale, run `sruja sync -r .` or suggest **Sruja: Refresh repo context**. For deep semantic context, check/generate `index.scip` (SCIP) first.
3. Slice from system index — If `system.index.json` exists, load only impacted slice; use canonical IDs `repo_id::local_id`
4. Intent and contract refs — ADRs, intent files from repo or bundle
5. Truth/drift — `.sruja/context.json` or `sruja status -r . --format json`

Prefer canonical IDs; do not invent when context is missing — ask or mark `unknown`.

## Quick Start Prompt

**Single repo:**
```
Use sruja-architecture skill. If .sruja/context.json exists and is recent, use it for evidence; otherwise run `sruja sync -r .` or `sruja discover --context -r . --format json`. Gather evidence, ask targeted questions only if scope or externals are unclear, generate a minimal repo.sruja with evidence-based components and relationships, then run `sruja lint` and fix all errors until it passes. Do not guess; list open questions instead.
```

**Multi-repo:**
```
Use sruja-architecture skill. Help me set up federation across multiple repos.
1) First, help me generate repo.sruja in each repository.
2) Then guide me through publishing bundles and composing system.index.json.
Reference docs/FEDERATION_SETUP_GUIDE.md for detailed steps.
```

**Optional Cursor rule:** "For architecture tasks, use `.sruja/context.json` when present and recent; else run `sruja sync -r .` or `sruja discover --context -r . --format json`. For multi-repo, use impacted slice from `system.index.json` when available; see docs/FEDERATION_SETUP_GUIDE.md."
