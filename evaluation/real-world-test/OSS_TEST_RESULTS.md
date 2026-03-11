# Sruja on other OSS projects – test results

Quickstart (and drift where noted) was run on several open-source repos in `test-repos/`. Summary below.

---

## Repos tested

| Project       | Language   | Components | Health | Notes |
|---------------|------------|------------|--------|--------|
| **Express**   | JavaScript | 9          | 100/100 | Small, clean; no violations. |
| **Caddy**     | Go         | 1977       | 99/100  | God modules in `cmd/` (commandfuncs, storagefuncs, packagesfuncs); 3 orphans. |
| **react-admin** | TypeScript | 1410     | 84/100  | 9 circular deps (ListView↔List, EditView↔Edit, etc.), 186 god-module warnings. |
| **Redis**     | C          | 9793       | 98/100  | God modules in utils + deps (jemalloc, hiredis, lua); many modules from vendored deps. |

---

## Express (Node.js)

- **Inventory:** 8 modules, 1 service, 7 dependencies.
- **Context:** JavaScript, Express, Monolith.
- **Findings:** None (100/100).
- **Domain map:** evaluation, lib (paths relative to test-repos dir).

Useful as a “clean” baseline: small codebase, no structural issues reported.

---

## Caddy (Go)

- **Inventory:** 1975 modules, 1 service, 1 external API, 4356 dependencies.
- **Context:** Go, Chi, Monolith.
- **Top findings:** God modules – `cmd/commandfuncs.go` (25 deps), `cmd/storagefuncs.go` (10), `cmd/packagesfuncs.go` (15).
- **Actionable:** HIGH – decouple bottlenecks (long list of affected files); LOW – 3 orphan modules (e.g. reverseproxy/ascii, fastcgi/header).
- **Domain map:** evaluation (bulk), modules, caddyconfig, cmd, internal.

Insight: Sruja surfaces real hotspots in `cmd/` and the module layout; health stays high because cycles/layer violations are absent and god-module penalty is capped.

---

## react-admin (TypeScript / React)

- **Inventory:** 1298 modules, 110 services, 2 external APIs, 7910 dependencies.
- **Context:** TypeScript, React, Microservices, Admin/Dashboard.
- **Top findings:** **9 circular dependencies** (e.g. ListView↔List, EditView↔Edit, ShowView↔Show, CreateView↔Create; DatagridHeader↔useDatagridStyles↔Datagrid; WithPermissions→useAuthenticated→…→types).
- **Drift:** `sruja drift -r <path>` reports 9 Errors (cycles), 186 Warnings (god modules). Exit code 1 when run with fail-on-errors (expected).
- **Actionable:** Break cycles (Dependency Inversion / events); decouple god modules (many in ra-ui-materialui inputs/forms/stories).
- **Domain map:** evaluation, packages/ra-core, packages/ra-ui-materialui, create-react-admin, etc.

Insight: Cycles and god modules in a real React monorepo are identified with concrete file paths; health 84 reflects the cycle penalty.

---

## Redis (C)

- **Inventory:** 9793 modules, 0 services, 12881 dependencies.
- **Context:** C, Monolith.
- **Top findings:** God modules – e.g. `utils/lru/lfu-simulation.c`, `utils/req-res-log-validator.py`, `utils/generate-command-code.py`; many more in `deps/` (jemalloc, hiredis, lua).
- **Actionable:** HIGH – decouple bottlenecks (utils + deps); LOW – orphans (e.g. jemalloc internals, lua, version.h).
- **Domain map:** evaluation (most), deps, utils, src/modules, src, modules.

Insight: Works on a large C codebase with vendored deps; highlights utils and dependency trees; health 98 (no cycles/layer violations, god/orphan penalties capped).

---

## Conclusion

- **Multi-language:** JavaScript, Go, TypeScript, C all produce sensible inventory, context, and findings.
- **Health score:** Differentiates clean (Express 100), mostly structural (Caddy 99, Redis 98), and cycle-heavy (react-admin 84).
- **Actionable output:** God modules, cycles, and orphans come with file paths and suggestions; drift gives Error vs Warning counts and exit code for CI.
- **Caveats:** Domain map top segment often “evaluation” when run from `evaluation/real-world-test/test-repos/` (path-based). Vendored deps (e.g. Redis’s jemalloc, lua) inflate module counts; findings still point to real hotspots (utils, cmd, packages).

These runs show Sruja providing useful structural insights across different OSS stacks and sizes.
