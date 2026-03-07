# Sruja Adoption Plan: Making It Easy for Developers

**Goal:** Reduce friction so developers can adopt Sruja quickly and get value without heavy setup.

**Principle:** Make developer life easy—minimal setup, clear feedback, actionable next steps.

---

## 1. Adoption Goals & Success Metrics

| Goal | Metric |
|------|--------|
| **Time to first value** | Developer sees useful output in &lt; 5 minutes from install |
| **Zero-config baseline** | `sruja quickstart -r .` works on any repo without config |
| **Clear next steps** | User knows what to do after quickstart/drift/analyze |
| **CI adoption** | Documented, copy-paste CI recipe for drift gates |
| **IDE integration** | Validation and feedback where devs already work |

---

## 2. Current Adoption Barriers

| Barrier | Impact | Who it affects |
|---------|--------|----------------|
| **Bootstrapping** | "I have a repo, I want architecture" → manual .sruja or agent skill. No one-step path. | New users |
| **Config sprawl** | No `sruja.toml` → flags and env vars everywhere. Hard to share team config. | Teams |
| **Output overload** | Reports can be noisy. "What should I fix first?" unclear. | All users |
| **CI unclear** | Drift/analyze in CI is possible but not a documented pattern. | DevOps, platform teams |
| **Context switch** | Dev is in IDE; drift/why live in CLI. No feedback loop in editor. | Day-to-day devs |
| **Install friction** | Multiple install options; PATH issues; no package manager integration. | New users |

---

## 3. Phased Plan

### Phase 1: Quick Wins (1–2 weeks)

*Low effort, high impact. Reduce friction for existing users.*

| Initiative | Action | Owner |
|------------|--------|-------|
| **1.1 Drift baseline alias** | Add `--baseline` as alias for `--architecture` so `sruja drift --baseline foo.sruja` matches spec. | CLI |
| **1.2 Top 3 recommendations** | Ensure quickstart/drift output surfaces top 3–5 items with effort/impact, not a long list. | Report |
| **1.3 Quickstart "next steps"** | Add explicit "Next steps" section to quickstart output: e.g. "Run `sruja drift -a arch.sruja` to compare against baseline", "Add `sruja.toml` for team config (coming soon)". | CLI |
| **1.4 CI recipe** | Document in README: "Add drift to CI" with GitHub Actions / GitLab snippet. Exit code semantics: 0 = pass, non-zero = drift found. | Docs |

**Deliverables:** Updated CLI help, README CI section, quickstart output polish.

---

### Phase 2: Config & Bootstrapping (2–4 weeks)

*Align with sruja-config from two-dev plan. Reduce setup for teams.*

| Initiative | Action | Owner |
|------------|--------|-------|
| **2.1 sruja.toml schema** | Add `sruja-config` crate with schema: `repo_path`, `intent_path`, `architecture_path`, `god_module_threshold`, optional semantic/runtime config. | Config |
| **2.2 CLI auto-load config** | When `repo_path` not specified, look for `sruja.toml` in cwd or parent. Use config for defaults. | CLI |
| **2.3 Quickstart → baseline** | Enhance `sruja quickstart` to optionally generate a minimal `.sruja` baseline from scan: heuristics (e.g. top-level dirs → systems, package.json → containers). | CLI + Scan |
| **2.4 Architecture Agent skill** | Ensure skill is discoverable: "Install with `npx skills add sruja-ai/sruja --skill sruja-architecture-agent`" in README. Link to ARCHITECTURE_AGENT.md. | Docs |

**Deliverables:** `sruja.toml` support, improved quickstart, agent skill visibility.

---

### Phase 3: IDE & Flow (3–4 weeks)

*Surface feedback where developers work.*

| Initiative | Action | Owner |
|------------|--------|-------|
| **3.1 LSP diagnostics** | Ensure LSP already validates on save; document "Validate after AI edit" in extension. | LSP + Extension |
| **3.2 Drift in IDE** | Explore: VS Code extension command or panel to run `sruja drift` and show results inline. | Extension |
| **3.3 Why in IDE** | Explore: "Ask why" from context menu or command palette, show evidence in panel. | Extension |
| **3.4 Status bar** | Optional: Show "Sruja: ✓" or "Sruja: 3 issues" in status bar when .sruja file is open. | Extension |

**Deliverables:** Extension enhancements, docs for IDE workflow.

---

### Phase 4: Polish & Scale (4–6 weeks)

*Improve adoption at scale.*

| Initiative | Action | Owner |
|------------|--------|-------|
| **4.1 Package manager** | Add `brew install sruja` (Homebrew), `npm install -g sruja-cli` (if feasible), or `cargo install sruja-cli`. Document all options. | Release |
| **4.2 Onboarding flow** | Create "Getting Started" doc: Install → quickstart → scan → (optional) baseline → drift → CI. | Docs |
| **4.3 Prioritized recommendations** | Ensure `ComprehensiveReport.recommendations` are sorted by priority/impact; cap at 10; include effort estimate. | Report |
| **4.4 Team config** | Document shared `sruja.toml` in repo root; CI loads it; team members get same thresholds. | Docs |

**Deliverables:** Multi-channel install, onboarding doc, report polish.

---

## 4. Dependencies & Priorities

```
Phase 1 (Quick Wins)     →  Can start immediately
Phase 2 (Config)         →  Depends on sruja-config (Week 14–15 in two-dev plan)
Phase 3 (IDE)            →  Can start in parallel with Phase 2
Phase 4 (Polish)         →  After Phase 1–2; can overlap with Phase 3
```

**Recommended order:** Phase 1 → Phase 2 (config) + Phase 3 (IDE) in parallel → Phase 4.

---

## 5. Out of Scope (For Now)

| Item | Reason |
|------|--------|
| Web UI for analysis | mdBook is the site; CLI + extension is primary. |
| Slack/bot integration | Later channel; not core adoption. |
| Self-hosted semantic | Stub provider for zero-key; OpenAI optional. |

---

## 6. Success Criteria

After this plan:

- **New user:** Installs Sruja, runs `sruja quickstart -r .`, gets actionable output in &lt; 5 minutes.
- **Team:** Adds `sruja.toml` to repo, runs drift in CI, gates on health score.
- **Day-to-day dev:** Gets validation in IDE; runs drift/why when needed; sees clear next steps.

---

## 7. References

- [ARCHITECTURE_AGENT.md](ARCHITECTURE_AGENT.md) — AI-assisted discovery
- [USING_SRUJA_IN_YOUR_PROJECT.md](USING_SRUJA_IN_YOUR_PROJECT.md) — Project integration
