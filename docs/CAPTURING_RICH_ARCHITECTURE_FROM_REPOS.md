# Capturing as Rich Architecture Information as Possible from Repos

This doc describes **how to capture the richest possible architecture information** from a repository for evaluation, agent input, and baseline comparison. It aligns with [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md): combine multiple signals, entry-point–driven discovery, and evidence-based documentation.

---

## 1. Goal

- **For evaluation:** Persist a full “architecture snapshot” per repo (code graph, context, intent, optional generated DSL) so we can compare runs, tune the skill, and measure recall/accuracy.
- **For the agent:** Feed the agent with every signal we have (scan + discover context + deployables + config + ADRs) so generated `architecture.sruja` is as accurate and complete as possible.
- **For baselines:** Use the same capture as the source of truth when running drift, intent check, and “why” explanations.

---

## 2. Current Signals (What Sruja Already Captures)

| Signal | Command / source | What it gives |
|--------|------------------|----------------|
| **Dependency graph** | `sruja scan <repo> --output graph.json` | Nodes (modules, services, databases, external APIs), edges (imports/calls), paths; language-specific parsing (JS/TS, Python, Go, Rust, etc.). |
| **Repo context (discovery)** | `sruja discover --context -r <repo>` | Component count, edges, primary language, framework, inferred domain, architecture style (monolith/microservices), **suggested areas** (top-level path segments). Text summary for prompts. |
| **AI context export** | `sruja context -r <repo> -f json -o context.json` | Module/service/database/external-API counts, **inferred layers** (api, services, data, models, utils, ui), **inferred boundaries** (e.g. UI→data forbidden), ready for Cursor/Copilot rules. |
| **Intent / ADRs** | `sruja intent check -r <repo> -i <dir> -f json` | Parsed ADRs (path, number, title, status, date, tags; optional context/decision/consequences). Drift between code graph and documented intent. |
| **ADR index** | `sruja intent adr-index` (when available) | List of ADR metadata from `docs/adr`, `doc/adr`, `adr/decisions`. |
| **Quickstart** | `sruja quickstart -r <repo>` | Scan + health score + findings (orphans, cycles, layer violations, god modules) + optional baseline DSL. |
| **Drift** | `sruja drift -r <repo> [-a architecture.sruja]` | Structural drift vs baseline or vs “ideal”; violations with evidence (file paths). |

**Not yet automated by Sruja (read manually or via agent):**

- **Deployables:** Dockerfile(s), docker-compose, K8s manifests, Procfile, fly.toml — number and identity of runnable units.
- **Entry points:** From package.json / pyproject.toml / Cargo.toml scripts and conventions (main, index, *Application.java).
- **Config:** .env.example, config files — DB DSN, queue URLs, external API URLs.
- **Docs:** README, docs/architecture.md, OpenAPI/AsyncAPI/GraphQL specs — requirements, flows, external systems.

The **sruja-architecture-agent** skill uses the discovery playbook (REFERENCE.md) to read these manually or via tools; combining them with `discover --context` and scan improves accuracy (research: combination of tools → higher F1).

---

## 3. Rich-Capture Pipeline (What to Run and Store)

To capture as rich architecture information as possible in one go:

### 3.1 One-shot capture (recommended for evaluation)

Run the following from the **Sruja repo** (or with `sruja` on PATH). All paths below are relative to an **output directory** per repo, e.g. `run_results/rich_capture_<repo>_<timestamp>/`.

| Step | Command | Output file | Purpose |
|------|--------|-------------|---------|
| 1 | `sruja scan <repo> --output graph.json` | `graph.json` | Full dependency graph (nodes, edges, kinds, paths). |
| 2 | `sruja discover --context -r <repo>` | `discover_context.txt` | Summary + suggested areas for discovery questions. |
| 3 | `sruja context -r <repo> -f json -o context_export.json` | `context_export.json` | Layers, boundaries, counts for AI tools. |
| 4 | `sruja intent check -r <repo> -i <repo> -f json` (if intent dirs exist) | `intent_report.json` | ADRs + drift vs code. |
| 5 | (Optional) Copy or generate `architecture.sruja` | `architecture.sruja` | Baseline or agent-generated DSL for drift/lint. |

**Optional for even richer capture:**

- **Prompt + context for any LLM:** `sruja generate -r <repo> --prompt-only -o prompt.txt` — embeds discover context + skill; use with any LLM to produce `architecture.sruja`.
- **Cursor rules:** `sruja context -r <repo> -f cursor-rules -o .cursorrules` — store in the bundle if you want to compare “what the AI saw.”

### 3.2 Script

From `evaluation/real-world-test`, use:

```bash
./run_rich_architecture_capture.sh <repo_name>   # e.g. saleor
# or
./run_rich_architecture_capture.sh --list saleor documenso cal.com
```

This runs steps 1–5 (and optional generate --prompt-only) and writes everything into `run_results/rich_capture_<repo>_<timestamp>/`. See script help for options.

### 3.3 Using the bundle for evaluation

- **Compare graphs over time:** Diff `graph.json` between runs or commits to see structural evolution.
- **Compare generated vs golden:** If you have `architecture.sruja` (golden or generated), run `sruja lint` and `compare_architecture.sh` against it; use `context_export.json` and `discover_context.txt` to explain gaps (e.g. “suggested areas” vs what the agent produced).
- **Tune the agent:** Feed the same bundle (discover_context.txt + context_export.json + graph.json excerpt) into the agent; change the skill or playbook; re-run and compare new `architecture.sruja` to golden or to previous run.
- **Intent drift:** Use `intent_report.json` to see documented vs code alignment and missing/extra components.

---

## 4. Maximizing Richness (Best Practices)

### 4.1 Combine multiple signals

- **Always run scan + discover before generating DSL.** The agent (or human) should have:
  - `graph.json` (or quickstart summary) for structure;
  - `discover_context.txt` for language, framework, areas, style;
  - `context_export.json` for layers and boundaries.
- **Add intent when present:** If the repo has `docs/adr`, `doc/adr`, or `docs/architecture`, run `sruja intent check` and include the report in the bundle so the agent can align with documented decisions and requirements.

### 4.2 Follow the discovery playbook order

When reading the repo (manually or via agent), use this order so entry points and deployables drive the mapping:

1. **Deployables** — Dockerfile(s), docker-compose, K8s, Procfile → systems/containers and technologies.
2. **Entry points** — Manifest scripts + main/index/*Application → containers and first-level components.
3. **Data stores & queues** — Config, .env.example, ORM/queue imports → database/queue containers and relationships.
4. **Service-to-service & externals** — HTTP/gRPC clients, env URLs → relationships with labels.
5. **UI / frontend** — Next/React/Vue layout → frontend container and link to API.

The playbook is in [REFERENCE.md](../skills/sruja-architecture-agent/REFERENCE.md) and [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md).

### 4.3 Include dependency context in synthesis

- When generating a view for a service/system, include: its entry points, one level of imports, config that mentions DBs/queues/APIs, and the deployment unit that maps to it.
- Cross-service: use config and code (URLs, client SDKs) to add relationships with specific labels (e.g. “REST - auth”).

### 4.4 Scope and pruning

- **High-level overview:** Systems, main containers, key externals; no deep components.
- **Standard:** 10–30 components (containers + components), all key relationships.
- **Deep / subsystem:** One subpath or bounded context; full containers + components inside; others as external systems.

Use `sruja discover --context -r <subpath>` for subpath scope; suggested areas in `discover_context.txt` help choose subpaths.

### 4.5 Evidence and confidence

- Prefer code- and config-derived facts; mark assumptions and low-confidence elements in descriptions (e.g. “confidence: low; evidence: env var only”).
- List “Open questions” and “Not detected” (e.g. end users, SLOs) in the capture or in the generated DSL so evaluators know what was missed.

---

## 5. Summary Table

| What to capture | How | Stored in bundle |
|-----------------|-----|-------------------|
| Dependency graph | `sruja scan <repo> --output graph.json` | `graph.json` |
| Discovery context (language, framework, areas, style) | `sruja discover --context -r <repo>` | `discover_context.txt` |
| AI context (layers, boundaries, counts) | `sruja context -r <repo> -f json` | `context_export.json` |
| Intent / ADRs / drift | `sruja intent check -r <repo> -i <repo> -f json` | `intent_report.json` |
| Baseline or generated DSL | Copy or agent | `architecture.sruja` |
| Optional: prompt for any LLM | `sruja generate -r <repo> --prompt-only -o prompt.txt` | `prompt.txt` |

Running all of these per repo and storing them in a single directory gives a **rich architecture capture** suitable for evaluation, agent input, and baseline comparison. Use `run_rich_architecture_capture.sh` in `evaluation/real-world-test` to automate the pipeline.

**Interactive and selective capture:** For **choosing areas** and **concise, non-verbose** extraction (one area at a time, bullet summaries only), use the **sruja-architecture-agent** skill only: see [skills/sruja-architecture-agent/SKILL.md](../skills/sruja-architecture-agent/SKILL.md) section **Interactive and selective capture (use Sruja only)** and the **Concise extraction summary** template. The skill instructs the AI to run `sruja discover --context -r .` (and optionally `-r <subpath>`), offer area selection when there are multiple, and output short useful summaries — no full graph dumps or long prose.
