# Incremental Architecture Capture and Stitching

Capture architecture in small pieces over time and stitch them into a single, coherent model. This doc describes the workflow, what exists today, and planned improvements.

## Vision

- **Incremental capture** – Add one module, one service, or one PR at a time instead of one big upfront architecture doc. In a **large repo** you may not be able to capture the whole at once—capture by area (subpath) or by bounded context.
- **Stitching** – Combine those pieces into one model: one graph, one validated `.sruja` (or a composed view), so you can lint, drift, and export the whole.

## What Exists Today

### 1. Scan → baseline (one-shot)

You can generate a single baseline from the current repo and then refine it:

```bash
sruja quickstart -r . --generate-baseline
# Creates architecture.sruja from scan; then:
sruja lint architecture.sruja
sruja drift -r . -a architecture.sruja
```

This gives you one file that represents “current code as architecture.” You can then edit it incrementally (add descriptions, split systems, add external refs) and re-run drift.

### 2. Knowledge graph merge (scan + scan)

The **knowledge graph** merges multiple **scans** into one graph (same repo or conceptually multiple passes):

- `merge_scan_into_graph(&mut kg, &scan_graph, repo_path)` – merges a code scan into the graph.
- Nodes/edges are merged by ID; repeated merge is idempotent.

So “incremental” at the **scan level** is supported: you can run scan on different subtrees or branches and merge into one graph. The CLI uses this for `quickstart` and `why`; it does not yet expose “merge this scan into that graph file” as a single command.

### 3. Multi-repo: external systems (logical stitch)

For **multiple repos**, each repo can own its own `architecture.sruja` and reference others as **external systems**:

- In `user-service/architecture.sruja` you define your system and declare `order_service = external_system "Order Service" { ... }`.
- Relationships can point to external systems (e.g. `user_service.api -> order_service "REST"`).
- Each repo lints and validates independently; the “stitch” is logical (named references), not a single physical file.

See [examples/multi-repo/README.md](../examples/multi-repo/README.md). A **central aggregator** that imports all per-repo `.sruja` files is planned but not yet implemented.

### 4. Intent merge (ADRs / intent model)

`sruja-intent` has an intent model that supports **merge**: `IntentModel::merge(other)`. This is used for combining intent from multiple sources (e.g. ADRs) before comparing to the scan. So “stitching intent” across multiple docs is supported at the model level.

## Recommended Workflow: Large single repo (capture by subpath)

When the repo is too big to capture in one shot, capture **one area at a time** by passing a subpath to `-r`:

```bash
# One area per run (e.g. monorepo: services/auth, services/orders, apps/web)
sruja quickstart -r services/auth --generate-baseline
# → Writes services/auth/architecture.sruja (only the auth subtree)

sruja quickstart -r services/orders --generate-baseline
# → Writes services/orders/architecture.sruja (only the orders subtree)
```

- **Validate per area:** `sruja lint services/auth/architecture.sruja`
- **Drift per area:** `sruja drift -r services/auth -a services/auth/architecture.sruja`
- **Cross-area:** In each fragment, treat other areas as **external systems** (same pattern as multi-repo). A single stitched file for the whole repo (e.g. `sruja stitch` or DSL `import`) is planned.

### Intelligent capture with the skill (LLM)

Use the **sruja-architecture-agent** skill so the AI **asks you questions** before and during discovery. The skill includes a **discovery question bank** (context, scope, large-repo focus, boundaries, entry points, refinement), **discovery modes** (overview / standard / deep-dive / diff), and a **phased playbook** (deployables → entry points → data stores → relationships) for more accurate capture. Install: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent`. See [SKILL.md](../skills/sruja-architecture-agent/SKILL.md) (modes, playbook, question bank) and [REFERENCE.md](../skills/sruja-architecture-agent/REFERENCE.md) (Discovery playbook table, interview). Research and practices: [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md).

## Recommended Workflow: Incremental in One Repo

1. **Start** – Generate a first baseline or create a minimal `architecture.sruja`:
   ```bash
   sruja quickstart -r . --generate-baseline
   # or create architecture.sruja by hand / with AI (sruja-architecture-agent skill)
   ```
2. **Capture in pieces** – As you add or refactor code:
   - Re-run `sruja quickstart -r .` to see current inventory and health.
   - Edit `architecture.sruja` incrementally: add one system, one container, or one relationship at a time.
   - Run `sruja lint architecture.sruja` after each change (or use the editor’s “Run validation”).
3. **Keep code and intent aligned** – Run drift to see gaps:
   ```bash
   sruja drift -r . -a architecture.sruja
   ```
4. **Optional: export** – Export the stitched result for docs or diagrams:
   ```bash
   sruja export markdown architecture.sruja -o docs/architecture.md
   sruja export mermaid architecture.sruja -o docs/diagram.mmd
   ```

## Recommended Workflow: Multi-Repo (Logical Stitch)

1. **Per service** – In each repo, maintain `architecture.sruja` with:
   - Internal components (containers, datastores).
   - External systems for other services (same names across repos).
2. **Validate per repo** – CI in each repo runs `sruja lint architecture.sruja`.
3. **Aggregation (today)** – Manually aggregate docs (e.g. each repo exports Markdown; a docs site combines them). **Future:** DSL `import` or CLI stitch to produce one composed file or view.

## Planned Improvements

| Improvement | Description |
|-------------|-------------|
| **DSL `import`** | Language spec already mentions importing from other Sruja files; implementation is planned. That would allow a single “master” file to `import "user-service/architecture.sruja"` etc. and validate the composed model. |
| **CLI `sruja stitch`** | A command that takes multiple `.sruja` files (or a directory) and produces one merged `.sruja` (or merged graph). Merge strategy: concatenate items with optional namespace/prefix to avoid ID clashes; then run lint on the result. Supports both **large single repo** (e.g. `docs/architecture/*.sruja`) and **multi-repo** aggregation. |
| **Incremental baseline update** | A mode that “merges” a new scan into an existing baseline (e.g. add new nodes/edges from scan into `architecture.sruja` without overwriting hand-edited descriptions). |
| **Central aggregator** | As in [examples/multi-repo/README.md](../examples/multi-repo/README.md): a repo or file that imports or references all service architectures and validates the combined view. |

## Summary

| Goal | Today | Future |
|------|--------|--------|
| Capture one repo incrementally | Edit `architecture.sruja` by hand or AI; use `--generate-baseline` once | Optional “merge scan into baseline” |
| Large repo (can't capture whole at once) | `-r` subpath: quickstart per area; one fragment per area; drift per area | CLI `stitch` or DSL `import` to merge in-repo fragments |
| Stitch multiple scans (same repo) | Knowledge graph merge in code; CLI uses it internally | Exposed as “merge scan into graph” option |
| Stitch multiple repos | External system refs per repo; validate per repo | DSL `import` + central aggregator; CLI `stitch` |
| Stitch intent (ADRs) | `IntentModel::merge` in sruja-intent | Already supported |

You can **capture architecture incrementally** today by editing one (or multiple per-repo) `.sruja` file(s) and validating with `sruja lint` and `sruja drift`. **Stitching** is logical across repos (external systems) and supported in-memory for scans (graph merge) and intent (model merge); file-level stitch (multiple `.sruja` → one) and DSL `import` are the next steps to tie everything together.

**Using the skill (LLM):** For better capture, use the [sruja-architecture-agent](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture-agent) skill in your editor. The agent is instructed to **ask you intelligent questions** (context, scope, large-repo focus, boundaries, entry points) before and during discovery, so the generated architecture matches your intent. See the skill’s **Discovery question bank** in [SKILL.md](../skills/sruja-architecture-agent/SKILL.md) and **Discovery interview** in [REFERENCE.md](../skills/sruja-architecture-agent/REFERENCE.md).
