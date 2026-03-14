# Sruja Architecture Discovery Reference

Use this reference when the request starts from a repo or spec, not from an existing `.sruja` file.

## Core workflow

1. **Gather evidence first**
   - Run `sruja discover --context -r .` or `sruja discover --context -r <subpath>`.
   - Read manifests, entry points, config, docs, ADRs, and specs before modeling.
   - Prefer evidence over inference. Do not invent externals, technologies, or deployables.
2. **Ask targeted questions when needed**
   - Ask 2-5 questions only when scope, boundaries, or key flows are unclear.
   - Good questions: which area first, what the deployables are, which externals must appear, which flows matter most.
3. **Build the DSL**
   - Model only what is evidenced or confirmed.
   - Prefer fewer, correct elements over a large speculative diagram.
   - Mark uncertainty as open questions instead of guessing.
4. **Validate before returning**
   - Run `sruja lint <file>.sruja`.
   - Fix until it passes.

## When to ask instead of guess

- The repo looks like a monorepo or has several plausible boundaries
- It is unclear whether something is a library, app, worker, or service
- External calls exist but the target system is not identifiable
- The user asks for "the architecture" of a large repo without naming a scope
- Several entry points exist and it is unclear which are production-relevant

## Discovery modes

| Mode | Use when | Output |
|------|----------|--------|
| `high-level-overview` | User wants the big picture only | Systems, top containers, key externals |
| `standard` | Default for one repo or one area | Systems, containers, main components, labeled relationships |
| `subsystem-deep-dive` | User points to a subpath or bounded context | One area in detail; other areas as external systems |
| `diff-and-refine` | `architecture.sruja` already exists | Proposed additions/removals/fixes only |

## Discovery playbook

Follow this order:

1. Deployables and runtime: Docker, compose, K8s, Procfile, `package.json` scripts, `pyproject.toml`, `go.mod`, `Cargo.toml`
2. Entry points: `main`, `index`, app boot files, route registration, workers, CLIs
3. Data stores and queues: Postgres, MySQL, MongoDB, Redis, Kafka, RabbitMQ, SQS
4. Service relationships: HTTP/gRPC clients, SDKs, env vars, webhooks, event publishers/consumers
5. Frontend or public entry: web app, mobile backend, BFF, API gateway
6. Docs and intent: `README`, `docs/`, `adr/`, `SECURITY.md`, specs

If `sruja discover` is unavailable, follow the same order manually.

## Output shapes

### Concise extraction

Use this when the user wants "just the relevant area" or "extract the architecture for X":

- Area
- Entry points
- Main components
- Outbound
- Tech
- Open questions

Keep it to 5-10 bullets. No graph dumps.

### Full DSL generation

Use this when the user wants `architecture.sruja`:

- Generate the DSL only or DSL plus a short summary
- Use systems, containers, components, and labeled relationships
- Keep other uncertain areas as open questions or external systems
- Run `sruja lint` before returning

## Prompt patterns

### Whole repo baseline

`Use sruja-architecture. Run sruja discover --context -r ., gather evidence, ask targeted questions if scope or externals are unclear, generate architecture.sruja, then run sruja lint and fix until it passes.`

### One area first

`Use sruja-architecture. Run sruja discover --context -r ., list suggested areas, ask me which one to capture first, then generate architecture-<area>.sruja for that area only and treat other areas as external systems. Run sruja lint before returning.`

### Diff and refine

`Use sruja-architecture in diff-and-refine mode. Compare the repo to the existing architecture.sruja and propose only additions, removals, or relationship fixes. Do not rewrite the file from scratch. Run sruja lint on the updated file.`
