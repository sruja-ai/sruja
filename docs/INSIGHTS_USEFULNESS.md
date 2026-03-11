# Are Sruja’s insights really useful?

Short answer: **yes, when they point at your own code and at problems you care about.** They are **less useful** (or noisy) when they flag third-party/vendored code, test/demo/story files, or when the suggestion is too generic. Below is when they help and when they don’t.

---

## Where the insights are genuinely useful

### 1. Circular dependencies (with file pairs)

- **What we report:** “A → B → … → A” with concrete file paths (e.g. `ListView.tsx` ↔ `List.tsx`).
- **Why it’s useful:** Cycles make testing, tree-shaking, and reasoning about change harder. Knowing the **exact pair or chain** lets you break the cycle (extract interface, invert dependency, or split module).
- **Evidence:** In react-admin we surface real cycles in app code (e.g. auth hooks → types, form hooks). Maintainers can open those files and refactor.

**Verdict:** **Useful** — cycles are a real architectural smell; concrete paths make them actionable.

---

### 2. God modules in application code

- **What we report:** “Module X has N dependencies (threshold 10)” with file path.
- **Why it’s useful:** High fan-out/fan-in means the file is a bottleneck: changes ripple, tests are heavier. Splitting or extracting abstractions is a standard refactor.
- **Evidence:** In Caddy, `cmd/commandfuncs.go`, `cmd/storagefuncs.go` are real hotspots that aggregate many concerns. In sruja itself, `sruja-intent/src/lib.rs`, `sruja-language/src/parser/elements.rs` are real refactor targets.

**Verdict:** **Useful** when the path is in **your** code (src, app, cmd, packages you own). Less useful when it’s in vendored or generated code (see below).

---

### 3. Orphans (no incoming/outgoing deps)

- **What we report:** “Module X has no incoming or outgoing dependencies” with path.
- **Why it’s useful:** Can mean dead code, or an entry point / script that’s intentionally standalone. Either way it’s a candidate for delete or “why does this exist?”
- **Evidence:** In sruja we flag things like `demo/database.py`, `book/menu-bar-home.js` — some are demo/tooling (intentional), some might be removable.

**Verdict:** **Useful as a list to review** — you decide which are dead code vs intentional. Not useful if you ignore the list and treat every orphan as “must fix.”

---

### 4. Health score as a trend

- **What we report:** 0–100 score and “health went from 85 → 78” in drift-pr.
- **Why it’s useful:** The number itself is narrow (structural only), but **direction of change** is meaningful: “this PR/commit range made things worse” prompts a look at new violations.
- **Evidence:** drift-pr shows “new violations” and health delta; CI can fail on regression.

**Verdict:** **Useful for regression** and CI. Less useful as a single absolute “quality” number without context.

---

### 5. Inventory and domain map

- **What we report:** Module/service/database counts, “top folders by component count.”
- **Why it’s useful:** Quick picture of size and where code lives; helps onboarding and “where do I look?”
- **Caveat:** Domain map is path-based; if you run from a path that includes a parent dir (e.g. `evaluation/.../test-repos/`), the top segment can be “evaluation” and less meaningful.

**Verdict:** **Useful for orientation**; interpret domain map with the repo path in mind.

---

## Where the insights are weak or noisy

### 1. God modules in stories, tests, vendored code

- **What happens:** We flag any file with >10 dependencies. That includes Storybook `*.stories.tsx` (they import many components on purpose), test helpers, and vendored libraries (e.g. jemalloc, lua in Redis).
- **Why it’s noisy:** “Decouple this” is not helpful for a story file or third-party code you don’t refactor.
- **What to do:** Filter by path (e.g. ignore `**/deps/**`, `**/*.stories.*`, `**/vendor/**`) or treat those as low priority. We don’t exclude them by default because “what’s app vs vendor” is repo-specific.

**Verdict:** **Partially useful** — same metric, but only actionable when the file is in code you own and are willing to refactor.

---

### 2. Generic suggestions

- **What we report:** Same boilerplate per type: “Consider splitting into smaller components” (god module), “Consider introducing an interface or event-based communication” (cycle), “Review orphan modules.”
- **Why it’s weak:** We don’t analyze the actual code to suggest *how* to split or *which* interface to introduce. You still need human judgment.
- **What to do:** Use the **path and violation type** as the signal; treat the suggestion as a reminder of the standard fix, not a custom recipe.

**Verdict:** **Useful as a nudge and a list of targets**; not a full “how to fix” guide.

---

### 3. Layer violations without a declared model

- **What we report:** Layer violations only when we have a notion of layers (e.g. from a baseline or rules). In scan-only mode we don’t have your intended architecture.
- **Why it matters:** If you never give us a baseline, layer insights may be absent or heuristic-based. The most actionable layer view is “code vs. this .sruja / ADR.”

**Verdict:** **Useful when you have declared intent** (e.g. drift -a architecture.sruja); otherwise less so.

---

### 4. Single number (health score) as “quality”

- **What we report:** One number 0–100.
- **Why it’s limited:** It’s structural only (cycles, layers, god modules, orphans). It doesn’t capture tests, docs, naming, domain boundaries, or business risk. A 99 can still have tech debt you care about.
- **What to do:** Use the score for **comparison and trend**, not as the only definition of “good architecture.”

**Verdict:** **Useful for trend and CI**; **not** a complete quality score.

---

## Summary table

| Insight type        | When it’s really useful                    | When it’s weak or noisy                |
|---------------------|--------------------------------------------|----------------------------------------|
| **Circular deps**   | App code; you want to break cycles         | False positives (e.g. type-only cycles) |
| **God modules**     | Your src/cmd/app code                      | Stories, tests, vendored/deps           |
| **Orphans**         | As a review list (dead code vs intentional)| If you ignore the list                 |
| **Health score**    | Trend, drift-pr, CI                        | As sole “quality” metric               |
| **Inventory / map** | Onboarding, “where is the code”            | When path prefix dominates the map     |
| **Suggestions**     | As a reminder of standard fixes             | As a full “how to fix” recipe           |

---

## Bottom line

- **Yes, they can be really useful:** Cycles with file pairs, god modules in your own code, orphans as a review list, and health/drift for regression and CI are **concrete and actionable** when you use them in context.
- **No, they’re not magic:** You have to filter out noise (vendor, stories, tools), interpret the list (which orphans to delete, which to keep), and apply your own design judgment. The suggestions are generic; the **locations and violation types** are the main value.

So: **the insights are useful where they point at real code you control and at problems you care about (cycles, bottlenecks, dead code, regression).** They’re less useful when taken raw on big monorepos with lots of deps/stories/vendor without any filtering or context.

---

## Improvements (done and planned)

### God modules in stories, tests, vendor — we now ignore these

**Done.** The drift pipeline excludes certain paths from **god-module** and **orphan** detection (and from health-score counting). Excluded patterns include:

- `/vendor/`, `/third_party/`, **`/deps/`**
- **`/stories/`**, **`.stories.`** (Storybook)
- `node_modules/`
- `/test/`, `/tests/`, `__tests__`, `.spec.`, `.test.`
- `/examples/`, `/fixtures/`, `/mocks/`, `/scripts/`, `/build/`, `/migrations/`, `/setup/`
- Common config filenames (e.g. `vite.config.ts`, `jest.config.js`)

So files under those paths are no longer reported as god modules or orphans. If your repo uses different conventions (e.g. `submodules/` for vendor), you can treat those as low priority until we support configurable exclusions.

---

### Generic suggestions — should we use an LLM?

**Possible improvement.** Today every suggestion is fixed text (“Consider splitting…”, “Consider introducing an interface…”). To make suggestions more actionable we could:

- **Option A — LLM-backed “suggest fix”:** For each violation (or a subset), call an LLM with: violation kind, file path, and optionally a snippet or file content; return a short, concrete suggestion (e.g. “Extract `useAuthState` into a small module and have `WithPermissions` depend on that instead of the full auth chain”). Could be behind a flag (e.g. `sruja drift --suggest-fixes`) or a separate command (`sruja ai suggest-fix --violation-id …`), and require an API key.
- **Option B — Rule-based templates:** Keep no LLM but add violation-specific templates (e.g. for cycles: “Break the cycle between A and B by extracting an interface in …” with A/B filled in). Better than one generic line but still not code-aware.
- **Option C — Link to docs:** Emit a stable “tip ID” per violation type and link to a doc with patterns (e.g. “See https://…/break-cycles”). No LLM, still improves usefulness.

**Recommendation:** Start with **Option C** (docs + tip IDs); add **Option A** as an optional, key-gated feature for users who want tailored suggestions.

---

### Layer violations without a baseline — how to improve?

Today, **layer** insights are strongest when you have a declared baseline (e.g. `sruja drift -a architecture.sruja` or intent from ADRs). In scan-only mode we only use a simple heuristic (e.g. frontend → database), so layer coverage is limited.

**Ways to improve:**

1. **Generate a baseline from the repo:** Use `sruja quickstart -r . --generate-baseline` to produce an initial `architecture.sruja`, then run `sruja drift -a architecture.sruja`. That gives you a first-cut “intent” and drift against it. Document this flow as the recommended path when you don’t have an existing .sruja or ADRs.
2. **Expand heuristics (no baseline):** In scan-only mode, detect more layer patterns (e.g. “API” → “DB”, “UI” → “API”) from path/keyword heuristics and report them as “possible layer violations.” Label them as heuristic so users know they’re not from a declared model.
3. **Prompt for intent:** In docs or CLI, prompt the user: “To get layer drift, add a baseline: run … or place architecture.sruja in the repo.”

**Recommendation:** Do (1) and (3) first (generate-baseline + docs); consider (2) if we want more value in zero-baseline mode without over-promising.

---

### Single health number — how to improve?

The score is **structural only** (cycles, layers, god modules, orphans). It doesn’t cover tests, docs, or domain design. Improvements:

1. **Label it clearly:** In CLI and JSON, label the metric as “Structural health” or “Health (structural only)” so it’s not mistaken for a full “quality” score.
2. **Expose a breakdown:** The code already computes per-category penalties (cycle, layer, god, orphan). Expose them in `sruja drift --format json` (e.g. `health_breakdown: { cycle_penalty, layer_penalty, god_penalty, orphan_penalty }`). That way users see “why” the score is 84 (e.g. cycle penalty 15, rest small) and can track dimensions over time.
3. **Optional separate scores later:** If we add semantic or other analyses, we could report e.g. `structural_health`, `semantic_health` separately instead of one blended number.

**Recommendation:** Do (1) and (2): add the “structural only” label and a health breakdown in JSON. That keeps a single headline number but makes its meaning and drivers explicit.
