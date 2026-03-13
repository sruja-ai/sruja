# DSL, Markdown Export, and Skills Coverage

This doc answers: **Can we improve DSL and Markdown export for demo? Is it good already? Can we extract all Sruja Architecture features when using Sruja skills?**

## Summary

| Area | Status | Notes |
|------|--------|--------|
| **DSL** | **Good** | Language spec is complete (C4, overview, requirements, ADRs, scenarios, flows, policies, SLOs, scale, deployment, views). Flat syntax, kinds, imports. |
| **Markdown export** | **Improved** | Now includes **overview block** (summary, audience, scope, goals, non-goals, risks), **policies** (category, enforcement, description), and **constraints/conventions** when present in the AST. L1/L2/L3 Mermaid diagrams, requirements, ADRs, scenarios/flows, feedback/causal loops were already supported. |
| **Demo** | **Improved** | `demo/architecture.sruja` includes an `overview { }` block and a `policy` so `sruja export markdown demo/architecture.sruja` produces a rich doc for demos. |
| **Skills extraction** | **Partial** | **sruja-architecture-agent** focuses on C4 structure (systems, containers, components) and evidence-based **requirements, ADRs, scenarios, flows**. It does *not* auto-generate overview blocks, policies, constraints, conventions, deployment, SLO/scale blocks, or custom views. **Interactive/selective capture** (choose areas, concise output) is done only via this skill—see `skills/sruja-architecture-agent/SKILL.md`. |

---

## 1. DSL

The DSL is in good shape and supports the full feature set described in [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md):

- **Elements:** person, system, container, component, database, queue; custom kinds; imports (stdlib, relative).
- **Relationships:** `->` with labels; dot notation for nesting.
- **Governance:** requirements (functional, nonfunctional, constraint, performance, security), ADRs, policies, constraints, conventions.
- **Behavior:** scenarios, flows (steps).
- **Overview:** `overview { summary, audience, scope, goals, non_goals, risks }`.
- **SLO / scale:** in element bodies (system, container, component).
- **Deployment:** `deployment` nodes.
- **Views:** optional; C4 views auto-generated when omitted.

No DSL changes were required for the demo; the spec already covers what we need.

---

## 2. Markdown Export

### What was improved

- **Overview section**  
  When the file contains an `overview { }` block, the exported Markdown now shows:
  - Summary, **Audience**, **Scope**
  - **Goals**, **Non-goals**, **Risks** (as lists)  
  If there is no overview block, the exporter shows a short placeholder telling users to add one.

- **Policies section**  
  Top-level `policy` definitions are exported with:
  - Title, **Category**, **Enforcement**, description

- **Constraints and conventions**  
  When the AST contains top-level `Constraints` or `Conventions` items, the exporter emits **Constraints** and **Conventions** sections.  
  **Note:** The current program parser does *not* parse top-level `constraints { }` / `conventions { }`; those blocks are only parsed inside element bodies. The exporter is ready for when top-level parsing is added.

- **TOC**  
  Table of contents now includes links to Overview, Policies, Constraints, and Conventions when those sections are present.

### What was already good

- Table of contents
- Systems with L2 (container) and L3 (component) Mermaid diagrams
- Persons, requirements, ADRs
- Scenarios and flows (with Mermaid sequence diagrams)
- Feedback loops and causal loops (with Mermaid)
- Options to toggle TOC, overview, systems, persons, requirements, ADRs, scenarios, Mermaid

### Optional next steps (not done here)

- **Deployment section**  
  `MarkdownOptions` has `include_deployments` but the markdown exporter does not yet write a deployment section. The JSON exporter already supports deployments.

- **SLO / scale in element text**  
  Element bodies can include `slo { }` and `scale { }`; the markdown exporter could surface these in system/container/component sections.

- **Top-level constraints/conventions**  
  Add parsing of top-level `constraints { }` and `conventions { }` in the language so that the existing markdown sections are populated from the DSL as in the spec.

---

## 3. Demo

- **`demo/architecture.sruja`**  
  - Added an **overview** block (summary, audience, scope, goals, non-goals, risks).  
  - Added a **policy** (`NoDirectDB`) describing the “no direct frontend–database access” rule.

- **Usage**  
  From repo root (after `make build` or `cargo build -p sruja-cli --release`):

  ```bash
  sruja export markdown demo/architecture.sruja
  ```

  The output now includes a filled Overview section and a Policies section, suitable for demos and docs.

---

## 4. Skills: What Gets Extracted

### sruja-architecture-agent

The agent is built to **discover and document** architecture from a repo and to **add governance only when there is evidence**.

| DSL feature | Extracted by agent? | Notes |
|-------------|---------------------|--------|
| Systems, containers, components | **Yes** | Core C4 structure from deployables, entry points, dependencies. |
| Relationships | **Yes** | With evidence-based labels (protocol + purpose). |
| Persons, external systems | **Yes** | When evident (e.g. README, config, env). |
| Requirements | **Optional** | Only when found in docs (README, docs/, adr/, SECURITY.md, etc.); otherwise “Open questions”. |
| ADRs | **Optional** | When found in repo (e.g. adr/, decisions/); with citations. |
| Scenarios, flows | **Optional** | When key flows are documented; evidence-first. |
| **Overview block** | **No** | Not auto-generated; user can add manually or in a follow-up. |
| **Policies** | **No** | Not auto-generated; user can add from intent review. |
| **Constraints / conventions** | **No** | Not auto-generated. |
| **Deployment** | **No** | Not discovered from code. |
| **SLO / scale** | **No** | Not discovered. |
| **Custom views** | **No** | Default C4 views only. |

So: **we do *not* extract “all” Sruja Architecture features with the agent.** We extract the C4 model and, when evidence exists, requirements, ADRs, scenarios, and flows. Overview, policies, constraints, conventions, deployment, SLO/scale, and custom views are left for manual authoring or future extensions.

### sruja-architecture (design-time rules)

- Used when **generating or refactoring** `.sruja` files (design, patterns, anti-patterns, trade-offs).
- Does not “extract” from code; it guides how to write valid, well-structured DSL.

### Recommendation

- For **discovery from code**: use **sruja-architecture-agent**; then add **overview**, **policies**, and (if desired) **constraints/conventions** by hand or via a follow-up prompt.
- For **export for demo/docs**: run **`sruja export markdown architecture.sruja`** after adding an overview and policies so the generated doc is complete and demo-ready.

---

## 5. Validation

After editing `.sruja` files, run:

```bash
sruja lint path/to/file.sruja
```

For immediate feedback in the editor, use **Sruja: Run validation (check after AI/edit)** or save the file (validation runs on save). See `.cursorrules` and [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md).
