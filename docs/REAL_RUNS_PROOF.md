# Proof: Sruja Runs on Real Projects

This document records **real execution** of Sruja CLI on a real project to verify that the behaviour described in the docs actually happens. All commands were run from the Sruja repo (March 2025) against the **express** test repo (`evaluation/real-world-test/test-repos/express`).

**How to reproduce:** From the Sruja repo root run `make build`, then use `target/debug/sruja` (or `target/release/sruja`). The express repo is created by `evaluation/real-world-test/setup_repos.sh` or by `run_demo.sh` (clone on first run).

---

## 1. Quickstart (inventory, health, next steps)

**Doc claim:** *"sruja quickstart -r . — Primary entry: inventory, drift summary, health score, actionable fixes — no API key"*

**Command run:**
```bash
sruja quickstart -r evaluation/real-world-test/test-repos/express -f text
```

**Real output (excerpt):**
```
══════════════════════════════════════════════════════════════════════
🚀 Sruja Quickstart - Architecture Intelligence
══════════════════════════════════════════════════════════════════════

📂 Scanning repository...
   ✓ Found 9 components

📊 Repository Context
   • Primary Language: JavaScript
   • Framework: Express
   • Architecture: Monolith

🔍 Analyzing architecture health...
   ✓ Analysis complete

──────────────────────────────────────────────────────────────────────
📊 Architecture Inventory
──────────────────────────────────────────────────────────────────────
  Repository: .../test-repos/express

  Components detected:
    • 8 modules
    • 1 services
    • 0 databases
    • 0 external APIs
    • 7 total dependencies

──────────────────────────────────────────────────────────────────────
💚 Architecture Health Score (structural only): 100/100
──────────────────────────────────────────────────────────────────────
  ████████████████████ ✓ Good

──────────────────────────────────────────────────────────────────────
🔍 Top 3 Critical Findings
──────────────────────────────────────────────────────────────────────

  ✓ No critical issues found!

──────────────────────────────────────────────────────────────────────
🗺️  High-Level Domain Map
──────────────────────────────────────────────────────────────────────

  ├── 📂 Users (7 components)
  └── 📂 lib (1 components)

──────────────────────────────────────────────────────────────────────
🚀 Next Steps
──────────────────────────────────────────────────────────────────────

  1. Review the findings above and prioritize fixes
  2. Run 'sruja drift -r . --format json' for detailed analysis
  ...
```

**Proof:** Inventory (modules, services, dependencies), repository context (language, framework, architecture), health score, domain map, and next steps all appear. No API key used.

---

## 2. Drift (structural analysis)

**Doc claim:** *"sruja drift — Detect architecture drift (circular deps, orphans, layer violations, god modules)"*

**Command run:**
```bash
sruja drift -r evaluation/real-world-test/test-repos/express -f text
```

**Real output:**
```
════════════════════════════════════════════════════════════
Architecture Drift Detection
════════════════════════════════════════════════════════════

📊 Summary
----------------------------------------
  Modules: 8 | Services: 1 | Databases: 0
  Dependencies: 7
  Health Score (structural only): 100/100

════════════════════════════════════════════════════════════
```

**Proof:** Drift runs; summary shows module/service counts and health. With a baseline (e.g. `-a path/to/architecture.sruja`) it compares code vs declared architecture.

---

## 3. Scan (infer graph from code)

**Doc claim:** *"sruja scan — Infer architecture graph from code"*

**Command run:**
```bash
sruja scan evaluation/real-world-test/test-repos/express --output /tmp/sruja_proof_scan.json
```

**Real output:** `Wrote /tmp/sruja_proof_scan.json`. Graph snippet:
```json
{
  "nodes": [
    {
      "id": "index_js",
      "kind": "module",
      "label": "index",
      "technology": "JavaScript",
      "path": ".../express/index.js"
    },
    {
      "id": "lib_response_js",
      "kind": "module",
      ...
    },
    ...
  ],
  ...
}
```

**Proof:** Scan produces a JSON graph with nodes (id, kind, label, technology, path). Use `--output` (not `-o`) for the output path.

---

## 4. Why (deterministic answers with evidence)

**Doc claim:** *"sruja why — Quick 'why' queries against scanned repo; deterministic answers with evidence"*

**Command run:**
```bash
sruja why "what services or main components exist?" -r evaluation/real-world-test/test-repos/express
```

**Real output:**
```
Found 1 service(s): application

Confidence: 90%

Evidence (from graph):
  - [scanned: .../express] Component 'application' (kind=service, technology=JavaScript)

File references (from scan):
  - .../express/index.js
  - .../express/lib/application.js
  - .../express/lib/express.js
  - .../express/lib/request.js
  - .../express/lib/response.js
  - .../express/lib/utils.js
  - .../express/lib/view.js
```

**Proof:** Answer is derived from the scanned graph; evidence and file references are shown. No LLM required.

---

## 5. Discover context (for contextual questions)

**Doc claim:** Skill says run `sruja discover --context -r .` to get repo context for 2–5 contextual questions.

**Command run:**
```bash
sruja discover --context -r evaluation/real-world-test/test-repos/express
```

**Real output:**
```
# Repo context (for contextual discovery questions)

**Repo:** .../test-repos/express
**Components (scan):** 9
**Edges:** 7
**Primary language:** JavaScript
**Framework:** Express
**Architecture style:** monolith
**Suggested areas (from paths):** lib

Use this context to derive 2–5 questions tailored to this repo (see skill: contextual discovery).
```

**Proof:** Context includes component count, edges, language, framework, architecture style, and suggested areas—usable by an agent to ask tailored questions.

---

## 6. Lint (validate .sruja files)

**Doc claim:** `.cursorrules` and docs say run `sruja lint` to validate DSL files.

**Command run:**
```bash
sruja lint evaluation/real-world-test/comparison-express/with-skill/architecture.sruja
```

**Real output:**
```
✓ No issues found
```

**Proof:** Lint runs on a real `.sruja` file and reports success. Invalid syntax or rule violations would produce diagnostics.

---

## 7. Full E2E demo script

**Doc claim:** *"make demo or cd evaluation/real-world-test && ./run_demo.sh — Quickstart + drift on a real repo (Express)"*

**Command run:**
```bash
cd evaluation/real-world-test && ./run_demo.sh
```

**Real output (summary):** Demo completes with Phase 1 (Quickstart) and Phase 2 (Drift) output as above, then:
```
╔══════════════════════════════════════════════════════════════════╗
║  ✅ Demo complete                                               ║
╚══════════════════════════════════════════════════════════════════╝

Next steps:
  • sruja quickstart -r .     # Try on your own repo
  • sruja drift -r .           # Structural drift
  • sruja analyze -r .         # Full analysis
```

**Proof:** Single script runs quickstart and drift on the express repo with no config or API key.

---

## CLI reference (verified)

| Doc / claim | Correct CLI |
|-------------|-------------|
| `sruja quickstart -r .` | ✅ `-r` / `--repo` |
| `sruja drift -r .` | ✅ |
| `sruja drift -r . -a path/to/arch.sruja` | ✅ `-a` / `--architecture` |
| `sruja scan . -o graph.json` | ⚠️ Use `sruja scan [PATH] --output graph.json` (option is `--output`, not `-o`) |
| `sruja why "question" -r .` | ✅ |
| `sruja discover --context -r .` | ✅ |
| `sruja lint file.sruja` | ✅ |

---

## Summary

| Capability | Documented | Ran successfully |
|------------|------------|------------------|
| Quickstart (inventory, health, next steps) | Yes | Yes |
| Drift (structural, optional baseline) | Yes | Yes |
| Scan (graph to JSON) | Yes | Yes (`--output`) |
| Why (evidence-based answers) | Yes | Yes |
| Discover context | Yes (skill) | Yes |
| Lint .sruja | Yes | Yes |
| run_demo.sh E2E | Yes | Yes |

All runs were on a real project (express) with no API keys. To re-verify: `make build` then `cd evaluation/real-world-test && ./run_demo.sh`, then run the individual commands above from the repo root.
