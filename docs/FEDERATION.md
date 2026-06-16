# Multi-Repo Federation

This document describes the federation artifacts and commands for multi-repo architecture truth, and how editors and skills use them for retrieval.

## Quick Start

Federation lets you:
- Keep separate `repo.sruja` files in each repository
- Export each repo's architecture as a `repo.bundle.json`
- Combine all bundles into a single `system.index.json` with canonical IDs
- Detect conflicts (same logical element in multiple repos)

**Use when:** You have architecture spread across multiple repos and want a system-wide view.

### Prerequisites

1. **Sruja CLI installed**
   ```bash
   curl -fsSL https://sruja.ai/install.sh | bash
   sruja --version  # Verify: should show version
   ```

2. **Two or more repositories to test with**
   - Use existing repos OR create test repos
   - Can be any language (TypeScript, Python, Go, Rust, Java, etc.)

3. **Git initialized in each repo**
   ```bash
   cd /path/to/each/repo
   git init  # If not already a git repo
   ```

### Step 1: Generate Architecture in Each Repo

For **each repository**, use the AI skill to generate `repo.sruja`:

```bash
# Install the AI skill (once)
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

In your AI editor (Cursor, Copilot, Claude, etc.), run this prompt in **each repo**:

```
Use sruja-architecture. Generate repo.sruja for this repository.
Run sruja lint and fix any errors.
```

### Step 2: Publish Each Repo as a Bundle

In **each repository**, run the publish command to create a bundle:

```bash
sruja publish -r . -o repo.bundle.json
```

### Step 3: Compose System Index

Now combine all bundles into a single system index:

```bash
# Create a bundles directory
mkdir bundles

# Copy all bundles (rename to avoid collisions)
cp api-service/repo.bundle.json bundles/api-service.repo.bundle.json
cp payment-service/repo.bundle.json bundles/payment-service.repo.bundle.json
cp frontend/repo.bundle.json bundles/frontend.repo.bundle.json

# Compose into a single system index
sruja compose -i bundles -o system.index.json
```

### Step 4: Verify Results

```bash
# Check system index structure
cat system.index.json | jq 'keys'

# List all repos
cat system.index.json | jq '.repos[] | {repo_id, repo_path, truth_status, git_commit}'

# Check for conflicts
cat system.index.json | jq '.conflicts'
```

---

## Artifacts

### repo.bundle.json

Published repo truth + evidence artifact. Contains:

- **schema_version** — Version of the bundle schema (currently 1).
- **repo_id** — Identifier for the repo (inferred from directory name or git remote).
- **repo_path** — Repository path as provided.
- **git_commit** — Git HEAD short commit, if available.
- **baseline_path** — Path to the baseline DSL file (e.g. `repo.sruja`).
- **baseline_dsl** — Full content of the baseline DSL file.
- **context** — Latest context (same shape as `.sruja/context.json`: components, edges, truth_status, updated_at, etc.).
- **truth_status** — `"reviewed"` | `"drifted"` | `"unknown"`.
- **intent_refs** — (Optional) Paths to ADRs or intent files.
- **contracts** — (Optional) Reserved for exposed interfaces.
- **owners** — (Optional) Reserved for ownership.

**Producing a bundle:** Run `sruja publish -r <repo> -o repo.bundle.json`. The CLI scans the repo, resolves baseline and truth status, and writes the bundle.

### system.index.json

Composed multi-repo graph. Contains:

- **schema_version** — Version of the index schema (currently 1).
- **repos** — List of composed repos (repo_id, repo_path, truth_status, git_commit).
- **nodes** — All nodes from all repo baselines, with **canonical_id** = `repo_id::local_id` to avoid cross-repo collisions.
- **edges** — All edges with source/target as canonical IDs; **repo_id** indicates which repo contributed the edge.
- **conflicts** — List of conflicts (e.g. same kind+label in multiple repos); never silently merged.

**Producing the index:** Run `sruja compose -i <bundle-or-dir> -o system.index.json`. Input can be a single `repo.bundle.json` path or a directory; the CLI finds all `repo.bundle.json` files in that directory and composes them. Conflicts (e.g. same logical element in multiple repos) are recorded as `conflicted`/reported, not merged.

## Commands

| Command | Description |
|--------|-------------|
| `sruja publish -r <repo> -o repo.bundle.json` | Publish repo metadata, DSL snapshot, context, and truth state to a bundle file. |
| `sruja compose -i <bundle-or-dir> -o system.index.json` | Compose one or more bundles into a single system index. |

**Examples:**

```bash
# Publish current repo
sruja publish -r . -o repo.bundle.json

# Publish to a shared location
sruja publish -r ./services/api -o /shared/bundles/api.repo.bundle.json

# Compose from a directory of bundles
sruja compose -i ./bundles -o system.index.json

# Compose from a single bundle (one-repo system)
sruja compose -i repo.bundle.json -o system.index.json
```

## Composition Rules

- Each repo owns its local elements and contracts; canonical IDs are `repo_id::local_id`.
- The same logical service/API/queue may appear in multiple repos; composition does not auto-merge them. Duplicate kind+label across repos are reported in **conflicts** so humans or tooling can resolve (e.g. map to a single canonical service or document ownership).
- Conflicts produce `conflicted` or `unknown` in downstream use; there are no silent merges.

## Retrieval Order for Editors and Skills

When doing architecture-aware codegen or review, use this order to load context:

1. **Local repo truth** — `repo.sruja` (or `architecture.sruja`) in the current repo.
2. **Fresh evidence** — `.sruja/context.json` (from `sruja sync`); if missing or stale (> ~1 hour), run `sruja sync -r .` or prompt the user to refresh.
3. **Relevant slice from system index** — If `system.index.json` is available (e.g. from a shared path or CI artifact), load only the **impacted system slice** (nodes/edges relevant to the current task), not the full index.
4. **Intent and contract refs** — From bundle or repo (ADRs, intent files, contracts).
5. **Current truth/drift status** — From context or `sruja status -r . --format json`.

**Retrieval behavior:**

- Fetch only the impacted system slice (e.g. by repo_id, or by element IDs mentioned in the task).
- Prefer **canonical element IDs** (`repo_id::local_id`) over path guesses when referencing cross-repo elements.
- Include ownership, contracts, and recent drift when present.
- If context is missing or insufficient, ask a targeted question or mark `unknown`; do not invent.

## Stakeholder Views

The same system index can drive derived views; no stakeholder-facing DSL editing:

- **Engineering** — Dependency and drift map (from index + per-repo status).
- **Architecture leads** — System topology and ownership (from index + repos).
- **Platform/Security** — Boundary and critical dependency views (from index).
- **Product/Exec** — Capability and ownership map (from index).

Every important view must show lineage back to: reviewed DSL, intent source, code evidence, and unresolved drift.

## Contract Tests

For CI or validation:

- **.sruja/context.json** — Must have `schema_version`, `updated_at`, `truth_status`; optional `git_commit`, `baseline_path`.
- **repo.bundle.json** — Must have `schema_version`, `repo_id`, `context`, `truth_status`.
- **system.index.json** — Must have `schema_version`, `repos`, `nodes`, `edges`; `conflicts` array may be empty.
