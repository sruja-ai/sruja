# Architecture Intelligence: Current Best Practices (Research Summary)

This document summarizes research into **architecture intelligence**—tools and practices for understanding, documenting, and governing software architecture from real codebases—and how Sruja’s approach aligns or could improve.

**Scope:** Understanding/documenting *existing* systems (not building AI applications). Focus: C4-style modelling, architecture-as-code, drift detection, evidence-based documentation, and AI-assisted discovery.

---

## 1. Modelling vs diagramming

**Best practice (C4 model site, tooling ecosystem):**

- **Diagramming** = boxes and lines; low barrier but you **cannot query** or **validate** diagrams.
- **Modelling** = single non-visual model (nodes + edges); views are derived. Enables:
  - Querying (“show all dependencies of X”)
  - Validation (rules, cycles, layer violations)
  - Diff-friendly, PR-ready artifacts
  - Export to other tools

**Sruja alignment:** Sruja is modelling-first (`.sruja` DSL → graph → lint/validate). Aligns with “model is data; diagrams are views.”

---

## 2. Architecture-as-code and validation

**Best practices:**

- **Single source of truth** in version control next to code; text/DSL format for easy diff and PR review.
- **Explicit validation** in CI: structural rules (no circular deps, no orphans), C4 level semantics (system/container/component used correctly).
- **Tool examples:** Structurizr (`validate` CLI), C4 InterFlow (DSL + CLI + CI), LikeC4 (Model API, watch mode, MCP), Archicode (C4 + ArchiMate).

**Sruja alignment:** `.sruja` in git, `sruja lint` for validation, CI integration. Skill and REFERENCE stress C4 semantics (deployable = container, in-process = component). Continue ensuring lint rules encode C4 and project conventions.

---

## 3. Drift detection and structural analysis

**Best practices:**

- **Drift** = mismatch between documented/intended architecture and code. Detection via:
  - Dependency graph from code (static analysis) vs declared model
  - Cycle detection (DFS or Kahn’s algorithm) for cycles at component/module/service level
  - Orphans, layer violations, “god” modules as structural anti-patterns
- **Health metrics** (e.g. 0–100 score) from violations; useful for comparing refs and spotting regression when defined and scoped (e.g. exclude vendor/stories).
- **CI integration:** fail or warn on new violations or on health regression (e.g. vs baseline).

**Sruja alignment:** `sruja drift`, baseline comparison, health score, cycle/orphan/layer/god-module checks. Documented in ARCHITECTURE_INTELLIGENCE.md and HEALTH_SCORE.md. Best practice: keep health formula and “what’s in scope” explicit to avoid noise.

---

## 4. Dependency and code-derived graph

**Best practices:**

- **Parse → dependency graph → query/validate.** Primary method for automated architecture documentation (Softagram, CodeScene, Sokrates, etc.).
- **Use dependency graph for:** documentation, change-impact analysis, responsibility assessment, anti-pattern detection.
- **Cycle detection:** standard approach is graph-based (DFS or topological sort); Sonar and others do this automatically on dependency graphs.

**Sruja alignment:** `sruja scan` builds graph from code; drift and complexity use it. Incremental capture (by subpath, then stitch) fits “build graph from part of repo then reconcile with intent.”

---

## 5. ADR and governance

**Best practices:**

- **ADRs** capture context, decision, consequences, alternatives; lifecycle (Proposed → Accepted → Deprecated/Superseded).
- **Tooling:** MADR/Nygard templates, CLI and web tools (e.g. adr-tooling, Backstage ADR plugin), often with status and cross-linking.
- **Governance:** ADRs as first-class artifacts next to code; link decisions to components where relevant.

**Sruja alignment:** Extraction and “why” answers can reference decisions; optional LLM for drafting. Opportunity: tighter linkage from architecture model elements to ADRs (e.g. “decisions affecting this container”) and optional ADR lint/lifecycle in tooling.

---

## 6. Evidence-based vs speculative documentation

**Best practice (from evidence-based design and speculative modelling):**

- **Evidence-based:** Prefer empirical, code-derived facts (dependencies, modules, call graphs) and document what is **observed**; use analysis to inform and iterate.
- **Speculative/interpretive:** When intent or boundaries are unclear, document **assumptions and uncertainty** (e.g. “Open questions”, confidence, “inferred from X”).
- **Transparency:** Avoid silent guessing; expose reasoning and gaps so others can correct or refine.

**Sruja alignment:** Skill and REFERENCE enforce “ask questions instead of guessing”; document confidence and open questions; no invented externals without evidence. Aligns with evidence-based + transparent speculation.

---

## 7. Knowledge graphs and semantic code understanding

**Best practices:**

- **Code knowledge graphs:** AST + dependencies + optional semantics (e.g. Code-Graph-RAG, GraphGen4Code); store in graph DB; support NL queries and “intent” search.
- **Use cases:** “What depends on X?”, “Where is Y used?”, documentation and onboarding.
- **MCP / API:** Expose model and tools (e.g. LikeC4 MCP, Sruja MCP) for editor/agent integration.

**Sruja alignment:** Knowledge graph + scan + optional extraction; “why” and query; MCP for tools. Incremental capture and contextual discovery (e.g. `sruja discover --context`) improve “intelligent” use of the graph by agents.

---

## 8. AI-assisted discovery and contextual questions

**Best practices:**

- **Context first:** Derive questions from repo (languages, frameworks, layout, existing docs) rather than a fixed global list.
- **Two-step when ambiguous:** For large or unclear scope, discover then refine (e.g. ask → answer → update model) instead of one-shot generation.
- **Divide and stitch:** Very large repos → analyse by subpath or bounded context; produce fragments then stitch or document boundaries.
- **Ask vs guess:** Prefer prompting for clarification over inventing boundaries or externals; mark confidence and document gaps.

**Sruja alignment:** Skill: “always run `sruja discover --context -r .`”; contextual 2–5 questions; table for one-go vs two-step vs divide; “do not guess”; open questions and confidence. Matches current best-practice emphasis on context and evidence.

---

## 9. Summary table

| Area | Best practice | Sruja today |
|------|----------------|-------------|
| Modelling | Model as data; query & validate | ✅ DSL, lint, graph |
| Validation | CI, structural + C4 semantics | ✅ `sruja lint`, C4 in skill |
| Drift | Code vs intent; cycles, layers, health | ✅ drift, baseline, health score |
| Evidence | Prefer code-derived; document uncertainty | ✅ ask-don’t-guess, open questions |
| Discovery | Contextual questions; two-step / divide | ✅ discover --context, skill flow |
| ADR | Lifecycle, link to architecture | ⚠️ Extraction/why; optional deeper ADR link |
| Knowledge graph | Graph + NL/query + MCP | ✅ Scan, why, MCP |

---

For **architecture discovery from code** (how to get more accurate and detailed `.sruja` from any repo in AI editors), see **[ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md)** — research summary plus phased playbook, discovery modes, and implementation checklist for the sruja-architecture-agent skill.

## 10. Suggested follow-ups (no implementation in this doc)

- **ADR linkage:** Optional links from model elements to ADRs; optional ADR lint or status checks in CLI/skill.
- **Health and scope:** Document “useful vs noisy” (e.g. INSIGHTS_USEFULNESS.md) and consider scoping/filters (e.g. exclude paths) for health so it stays actionable.
- **Stitching UX:** Document or tool “merge fragments” and conflict handling when doing incremental capture by subpath (see INCREMENTAL_ARCHITECTURE_CAPTURE.md).
- **C4 InterFlow / LikeC4:** Periodically compare features (e.g. query API, watch, MCP) for cross-pollination and to avoid duplication.

---

## References (high level)

- C4 model tooling: https://c4model.com/tooling  
- Structurizr DSL and validate: https://docs.structurizr.com  
- LikeC4 Model API and MCP: https://likec4.dev/tooling/model-api  
- C4 InterFlow: https://github.com/SlavaVedernikov/C4InterFlow  
- Dependency/architecture analysis: Softagram, CodeScene, Sokrates, doc-architect  
- ADR tooling: https://adr.github.io/adr-tooling/  
- Code knowledge graphs: Code-Graph-RAG, GraphGen4Code  
- Sruja: ARCHITECTURE_INTELLIGENCE.md, INCREMENTAL_ARCHITECTURE_CAPTURE.md, HEALTH_SCORE.md, skills/sruja-architecture-agent
