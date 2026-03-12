# Architecture Discovery: Research Summary and Best Practices

This document summarizes **research on architecture recovery from codebases** and translates it into **concrete practices** for the Sruja architecture-agent skill used in AI editors (Cursor, Claude Code, etc.). Goal: **more accurate, detailed, and evidence-based** architecture capture from any repo.

---

## 1. Research Summary

### 1.1 Static analysis and hybrid approaches

- **Static analysis** is the preferred base for architecture discovery: it integrates into CI/CD, does not require running the system, and detects physical coupling (imports, calls, types) and—when combined with version control—logical coupling (change patterns).[^1]
- **Tool combination improves accuracy:** A 2024 comparison of 13 static architecture recovery tools for microservices found the best single tool at F1 ≈ 0.86; **combining four tools reached F1 ≈ 0.91**.[^2] Using multiple signals (dependency graph, config, entry points, deployment manifests) yields better results than any single method.
- **Hybrid = static + LLM:** Research systems like **ArchAgent** combine static analysis, **adaptive code segmentation** (to fit LLM context), and **LLM-powered synthesis** to recover business-aligned, multiview architecture from large legacy codebases. Key findings:[^3]
  - **Entry-point tracing:** Identify top-level entry points, trace downstream call/dependency paths, and use that to structure what the model sees.
  - **Contextual pruning:** Keep only dependency context relevant to the architectural view (e.g. inter-service and key intra-repo dependencies); pruning improves accuracy.
  - **Dependency context matters:** Ablation studies showed that **including contextual dependencies** (intra-repo + inter-service) significantly improved F1 and “business restoration” over diagrams generated without that context.
  - **Adaptive grouping:** For very large repos, partition by token count with overlap between adjacent groups (e.g. DFS order with ~10% overlap) to avoid undersized tail groups and preserve coherence.

### 1.2 AI-assisted C4 and diagram generation

- **C4 + AI:** Practices from “AI-assisted software architecture” workflows (e.g. C4/Structurizr DSL from code):[^4]
  - **Model first, then views:** Generate a single model (e.g. Structurizr/Sruja DSL); derive diagrams from it. Avoid ad-hoc diagramming.
  - **Scope selection for large codebases:** Ask which area to analyze (e.g. one service or the whole repo) so the model stays within a manageable scope.
  - **Iterative refinement:** Generate → review → refine. One-shot perfection is not the goal.
- **CodeBoarding / Swark-style pipelines:** Static analysis (e.g. LSP, control flow) extracts modules and relationships; an LLM synthesizes semantic understanding and produces diagrams (e.g. Mermaid). Pattern: **extract structure → LLM abstracts → output standard format**.[^5]

### 1.3 Evidence and uncertainty

- **Evidence-based documentation:** Prefer code-derived facts (dependencies, config, entry points) and document what is **observed**; distinguish clearly from assumptions.[^6]
- **Transparent uncertainty:** When intent or boundaries are unclear, document assumptions, confidence (high/medium/low), and “Open questions” so humans can correct or refine.

---

## 2. Best Practices (Implementation Checklist)

These practices are implemented or reflected in the **sruja-architecture-agent** skill and REFERENCE.

### 2.1 Use multiple discovery signals (combine tools)

| Signal | What to use | How in Sruja |
|--------|-------------|--------------|
| Dependency graph | Imports, calls, packages | `sruja discover --context -r .` uses scan; REFERENCE read order |
| Deployment/runtime | Docker, K8s, Procfile, fly.toml | REFERENCE: Infra files; playbook Phase 1 |
| Entry points | main, index, app entry files | REFERENCE: Per-language entry points; playbook Phase 2 |
| Config / env | DB DSN, API URLs, queues | REFERENCE: Config files; playbook Phase 3 |
| Docs / ADRs | README, docs/, adr/ | Skill: intent extraction with citations |

**Practice:** Always run `sruja discover --context -r .` first. Then follow the **phased discovery playbook** (see REFERENCE) so the model uses scan + manifests + entry points + config together, not just one.

### 2.2 Entry-point–driven discovery

- **Identify top-level entry points** (e.g. `main.go`, `index.js`, `*Application.java`, `manage.py`) from manifest and conventions.
- **Trace one level of dependencies** from each entry (routes, services, DB clients). Do not read the entire codebase.
- **Use entry points to decide “containers”:** Each runnable entry (app server, worker, CLI) maps to a container; internal modules map to components.

**Practice:** In REFERENCE, the “Discovery playbook” and “Read order” enforce: manifest → entry points → one level of imports → config. The skill tells the agent to follow this order.

### 2.3 Contextual pruning and scope

- **Prune by relevance:** For a given scope (e.g. “services/auth”), only feed files and dependencies relevant to that boundary. For “high-level overview,” focus on systems and main containers, not every class.
- **Explicit modes:** Support at least:
  - **high-level-overview:** Systems, main containers, key externals; no deep components.
  - **subsystem-deep-dive:** One subpath or bounded context; full containers + components inside.
  - **diff-and-refine:** Compare existing `architecture.sruja` to current repo; propose only changes (additions, removals, relationship fixes).

**Practice:** SKILL defines scope ladder (minimal / standard / deep) and “Divide analysis into multiple parts.” Modes (overview, deep-dive, diff-and-refine) are specified in SKILL so the user or editor can request them.

### 2.4 Include dependency context in synthesis

- **When generating a view for a service or system,** include:
  - Its entry points and one level of imports.
  - Config that mentions databases, queues, and external APIs.
  - Any deployment unit (Docker/K8s) that maps this service.
- **Cross-service:** If the repo or config shows calls to other services (URLs, client SDKs), include those as relationships with labels (e.g. “REST - auth check”).

**Practice:** REFERENCE “Discovery playbook” phases 3–4 explicitly ask for data stores and service-to-service relationships from config and code. Skill “Accuracy contract” says: only include externals with evidence (imports, env, docker-compose, ADRs).

### 2.5 Adaptive grouping for very large repos

- **Do not feed the whole repo in one go.** Partition by:
  - **Subpath:** e.g. `-r services/auth`, then `-r services/orders`.
  - **Bounded context:** One area per run; use “external systems” for the rest.
  - **Depth:** First pass = systems + containers; second pass = expand one large container into components.
- **Overlap at boundaries:** When splitting by area, mention adjacent areas (e.g. “auth service calls user service”) so the next pass can add the relationship.

**Practice:** SKILL “Divide analysis into multiple parts” and REFERENCE “Divide and stitch” describe exactly this. `sruja discover --context -r <subpath>` supports subpath; suggested areas come from the scan.

### 2.6 Ask the right questions before building (do not guess)

- **Gather first, then ask:** Run discovery and the phased playbook to collect evidence. From what is **ambiguous or missing** (scope, boundaries, externals, key flows), derive 2–5 **targeted questions**; ask the user instead of inventing answers.
- **Derive questions from repo context:** Map repo signals (multiple dirs, monorepo, env vars pointing to unknown services, multiple entry points, no deployment files) to question categories (scope/area, boundaries, externals, entry/flows, intent). See REFERENCE "Deriving the right questions from repo context."
- **Build only after answers:** Generate the full architecture only after the user has answered (or said "proceed with defaults"). This yields optimal, intent-aligned architecture instead of guesswork.

**Practice:** Skill "Gather → Ask → Build" and "Contextual discovery" mandate evidence-first discovery, a question taxonomy, and "only after answers… generate." REFERENCE has the repo-signal → question table and workflow. Developer experience: encourage users to answer the agent's questions for best results (GETTING_STARTED_SKILL.md, SKILL "For end users").

### 2.7 Confidence and diagnostics

- **Attach confidence to uncertain elements:** In `description`, add “(confidence: low|medium; evidence: …)” when something is inferred from weak signals.
- **Run lint and fix before presenting:** Always run `sruja lint` and iterate until pass. No invalid DSL.
- **Report gaps:** List “Open questions” and “Not detected” (e.g. end users, SLAs) so the human knows what was not found.

**Practice:** Skill “Evidence, uncertainty, and questions” and “Accuracy contract” require evidence-first descriptions and confidence markers. Post-generate checklist and REFERENCE lint→fix table enforce validation and gap reporting.

### 2.8 Single model, then views

- **Output one authoritative model** (e.g. `architecture.sruja`). Diagrams and docs are derived (e.g. `sruja export markdown`, `sruja export mermaid`).
- **No ad-hoc diagramming:** The agent produces or updates the DSL; views are generated by the tooling.

**Practice:** Skill and REFERENCE use only Sruja DSL as the artifact; export is a separate step. Aligns with “model as data; diagrams are views” (ARCHITECTURE_INTELLIGENCE_BEST_PRACTICES.md).

---

## 3. References

- [^1] Code coupling and static analysis: Codepulse, “Code Coupling Analysis: Finding Hidden Architectural Dependencies.”
- [^2] Schneider et al., “Comparison of Static Analysis Architecture Recovery Tools for Microservice Applications,” Empirical Software Engineering, 2025; combination of four tools reached F1 ≈ 0.91.
- [^3] ArchAgent: “Scalable Legacy Software Architecture Recovery with LLMs,” arXiv:2601.13007 — adaptive grouping, entry-point tracing, README-style synthesis, contextual pruning, dependency context ablation.
- [^4] Working Software, “AI-Assisted Software Architecture: Generating the C4 Model and Views Directly from Code”; C4-skill (Claude Code), scope selection, iterative refinement.
- [^5] CodeBoarding, Swark: static analysis + LLM synthesis → Mermaid/architecture diagrams.
- [^6] Sruja ARCHITECTURE_INTELLIGENCE_BEST_PRACTICES.md: evidence-based vs speculative documentation.

---

## 4. Summary Table

| Research finding | Best practice | Where in Sruja |
|------------------|---------------|----------------|
| Combine multiple signals | Use scan + manifests + entry points + config | `sruja discover --context`; REFERENCE playbook |
| Entry-point tracing | Manifest → entry points → one level of deps → config | REFERENCE read order; playbook Phase 2 |
| Contextual pruning | Scope and relevance; avoid feeding whole repo | SKILL scope ladder; divide by subpath/depth |
| Dependency context improves F1 | Include intra-repo + inter-service deps in context | Playbook phases 3–4; “Accuracy contract” |
| Ask before building (do not guess) | Gather → derive questions from repo context → ask → then generate | SKILL "Gather → Ask → Build"; REFERENCE "Deriving the right questions" |
| Adaptive grouping for large repos | By subpath or bounded context; overlap at boundaries | SKILL “Divide analysis”; discover -r \<path\> |
| Confidence and gaps | Mark uncertainty; list open questions | Skill “Evidence, uncertainty”; post-generate checklist |
| Single model, then views | One DSL artifact; export for diagrams | Skill output = .sruja; sruja export for views |
| Explicit modes | Overview / deep-dive / diff-and-refine | SKILL “Discovery modes” |

This document should be updated when new research or tooling (e.g. better static recovery, MCP integrations) becomes relevant to architecture discovery.
