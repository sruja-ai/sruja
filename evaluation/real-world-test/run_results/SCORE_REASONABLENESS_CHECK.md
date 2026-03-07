# Score Reasonableness Check: Look at Repo Directly, Then See If Sruja Is Reasonable

**How to validate:** For each test repo, (1) look at the repo yourself (README, structure, what it is), (2) run Sruja and get the score, (3) decide: **is the score reasonable for this repo?**

---

## How to run this check yourself

```bash
# From Sruja repo root (after make build)
SRUJA=./target/release/sruja
REPOS=evaluation/real-world-test/test-repos

# 1) Look at the repo
cat $REPOS/<repo>/README.md | head -40
ls -la $REPOS/<repo>/

# 2) Run Sruja
$SRUJA quickstart -r $REPOS/<repo>

# 3) Ask: given what this repo is, is the score plausible?
```

---

## Validation results (run 2026-02-27)

### express (expressjs/express)

| What you see in the repo | Sruja score | Verdict |
|--------------------------|-------------|--------|
| Fast, minimalist Node.js web framework. Small core (`lib/`), many `examples/` and `test/`. Mature, widely used. | **98/100** (Excellent). 7 orphan modules (core lib files). | **Reasonable.** Clean framework with a small surface; high score matches “minimal, well-structured” product. Orphans are in product code (e.g. `lib/response.js`); we report but don’t crush the score. |

---

### gitea (go-gitea/gitea)

| What you see in the repo | Sruja score | Verdict |
|--------------------------|-------------|--------|
| Self-hosted Git service (Go + TS). `routers/`, `services/`, `models/`, `cmd/`, `tools/`, `tests/`. Large, production-grade, many contributors. | **91/100** (Excellent). 1 circular dependency (ComboMarkdownEditor ↔ EasyMDEToolbarActions), 27 orphans, 891 large modules. | **Reasonable.** One real cycle in the frontend is called out; score stays high. Orphans/large modules are capped so a big, successful project isn’t penalized to a meaningless number. |

---

### etcd (etcd-io/etcd)

| What you see in the repo | Sruja score | Verdict |
|--------------------------|-------------|--------|
| CNCF distributed key-value store (Go). Used by Kubernetes. `Documentation/`, `tools/`, `etcdctl/`, robustness testing. Production-critical. | **97/100** (Excellent). 3 orphans, 183 large modules. | **Reasonable.** No cycles; a few orphans (likely doc/tool paths excluded or minimal). High score fits a well-architected, critical infrastructure project. |

---

### caddy (caddyserver/caddy)

| What you see in the repo | Sruja score | Verdict |
|--------------------------|-------------|--------|
| Extensible server platform, TLS by default (Go). `modules/`, `caddyconfig/`, `caddytest/`. Mature, used in production. | **97/100** (Excellent). 3 orphans, 129 large modules. | **Reasonable.** No cycles; few orphans. Score reflects a structured, modular codebase. |

---

### Sruja (this repo)

| What you see in the repo | Sruja score | Verdict |
|--------------------------|-------------|--------|
| Architecture-as-code + drift tool. Rust workspace: `crates/sruja-*` (language, scan, diff, engine, export, CLI, etc.), `book/`, `extension/`. ~1190 modules, 3012 deps. | **97/100** (Excellent). 19 orphans (e.g. lib.rs roots, book JS), 102 large modules. No cycles. | **Reasonable.** Multi-crate Rust workspace; some “orphans” are crate roots or re-exports. Score matches a structured codebase with no circular deps. |

---

## Summary

| Repo    | Score  | Look at repo → expectation      | Sruja reasonable? |
|---------|--------|----------------------------------|-------------------|
| express | 98/100 | Small, clean framework          | Yes               |
| gitea   | 91/100 | Large app, one known cycle      | Yes               |
| etcd    | 97/100 | Critical infra, well-structured | Yes               |
| caddy   | 97/100 | Mature server, modular         | Yes               |
| **sruja** (this repo) | **97/100** | Rust workspace, many crates, no cycles | Yes |

**Conclusion:** When you look at each repo directly and then at Sruja’s score, the scores are **reasonable**: successful, well-used projects get high scores; the one real red flag (Gitea’s cycle) is reported and reflected in a slightly lower (but still high) score. No repo that is clearly “good” in practice gets a meaningless low score.

---

## How to add another repo to this check

1. Clone or use the repo under `evaluation/real-world-test/test-repos/<name>`.
2. Open README and directory layout; note what the project is and how it’s structured.
3. Run: `sruja quickstart -r evaluation/real-world-test/test-repos/<name>`.
4. Add a row to the table above: “What you see” | Score | Verdict (reasonable or not and why).
