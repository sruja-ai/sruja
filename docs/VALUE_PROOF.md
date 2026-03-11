# Why Sruja is worth running on your repo

This page summarizes **real output** from running Sruja on the Sruja repo itself. No config, no API keys, no `.sruja` file—just `sruja quickstart -r .` and `sruja drift -r .`.

---

## What you get in under 5 seconds

**Command:** `sruja quickstart -r .`  
**Repo:** This repo (Rust/TypeScript/Go, 1141 components, 3813 dependencies)

| Output | Example (this repo) |
|--------|----------------------|
| **Architecture inventory** | 1138 modules, 1 service, 1 database, 1 external API |
| **Health score** | 99/100 with a clear label (e.g. “Good”) |
| **Top critical findings** | God modules with **exact file paths** and a concrete suggestion |
| **Actionable fixes** | Prioritized (HIGH/LOW), impact text, and **list of affected files** |
| **Domain map** | Which top-level folders/crates dominate (e.g. sruja-cli, sruja-language, sruja-export…) |
| **Next steps** | Suggested follow-up commands (drift, scan, why) |

So in one run you get: *what we have*, *how healthy it is*, *what’s risky*, and *where to look*.

---

## Evidence from a real run (this repo)

### 1. Quickstart – health and findings

```
💚 Architecture Health Score: 99/100
   ████████████████████ ✓ Good

🔍 Top 3 Critical Findings
  1. ⚠️ Bottleneck: 'crates_sruja-intent_src_lib_rs' – God Module (32 deps, threshold 10)
     📍 ./crates/sruja-intent/src/lib.rs
     💡 Consider splitting into smaller, focused components

  2. ⚠️ Bottleneck: 'crates_sruja-intent_src_parser_adr_rs' – God Module (19 deps)
     📍 ./crates/sruja-intent/src/parser/adr.rs

  3. ⚠️ Bottleneck: 'crates_sruja-intent_src_compare_mod_rs' – God Module (32 deps)
     📍 ./crates/sruja-intent/src/compare/mod.rs

🎯 Top Actionable Fixes
  1. 🔴 [HIGH] Decouple God Modules – high regression risk; list of 80+ affected files
  2. 🟢 [LOW] Review orphan modules – e.g. demo/database.py, book/*.js, …
```

So Sruja **names the problem**, **points to the file**, and **suggests an action**. That’s immediately useful for a new contributor or a tech lead.

### 2. Drift – structural issues with file paths

```
⚠️ Warnings (127) – God modules with file paths
ℹ️ Info (14) – Orphan modules (no incoming/outgoing deps) with file paths
   e.g. ./demo/database.py, ./crates/sruja-export/src/context/mod.rs, ./book/menu-bar-home.js
```

You get a **count** of issues and **exact paths** for follow-up or refactors.

### 3. JSON for CI and tooling

```bash
sruja quickstart -r . --format json
```

You get the same health score, inventory, `top_findings`, and `actionable_fixes` in JSON—so you can:

- Fail a CI job if health drops below a threshold
- Feed dashboards or internal tools
- Track health over time

---

## Why this is worth it for “someone’s repo”

| Need | How Sruja helps |
|------|-------------------|
| **Onboard faster** | One command → inventory + health + top risks + file locations. No reading the whole codebase first. |
| **Find refactor targets** | God modules and orphans with file paths; prioritized fixes with impact. |
| **Gate quality** | `sruja drift -r .` exits non-zero on errors; use in CI to block regressions. |
| **Explain “why”** | `sruja why "question" -r .` uses the scanned graph (and optional ADRs) to answer with evidence. |
| **No commitment** | No `.sruja` file or API keys required for quickstart/drift. Optional: add a baseline later and use drift vs. intent. |

---

## Try it yourself

From this repo (after `make build`):

```bash
./target/release/sruja quickstart -r .
./target/release/sruja drift -r .
./target/release/sruja quickstart -r . --format json
```

On any other repo (with [Sruja installed](https://sruja.ai)):

```bash
sruja quickstart -r /path/to/your/repo
sruja drift -r /path/to/your/repo
```

**Bottom line:** You get an architecture snapshot, a health score, and actionable findings with file-level evidence in seconds. That’s the proof Sruja is worth running on a repo.
