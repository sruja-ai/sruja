# Making the Sruja Skill Super Awesome

Concrete improvements to the Sruja architecture skill so it produces better output, is easier to use, and feels "super awesome" to users and AI agents.

**Status:** Many items below are **implemented** (see `skills/sruja-architecture-agent/SKILL.md` 0.10.x, `REFERENCE.md`, and [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md)): canonical prompt, mandatory lint, scope ladder, minimal example, relationship patterns, discovery modes, phased playbook, CHANGELOG. This doc is kept for remaining ideas and historical context.

---

## 1. Prompt & Process (Agent Skill)

### 1.1 One-shot "thorough capture" prompt in the skill

**Today:** The agent skill describes a process (Understand → Collect → Generate → Validate → Present) but doesn't give a single, copy-paste prompt that guarantees thorough capture.

**Improve:** Add a **canonical prompt** block in `sruja-architecture-agent/SKILL.md` or `REFERENCE.md` that the model (or user) can use when generating from a codebase:

- "Thoroughly capture: (1) main systems and entry points, (2) containers and their technologies, (3) every container/component has a description and technology, (4) relationships with specific labels (protocol + purpose), (5) external systems and persons. Target 10–30 components. Run `sruja lint` and fix until pass."

This makes "good prompt in both cases" (Mermaid vs Sruja) easy to replicate and improves Sruja output quality consistently.

### 1.2 Explicit "run lint after generate" step

**Today:** Step 4 says "Validate – Run `sruja lint`" and "Fix any validation errors." It's easy to skip.

**Improve:** Make it **mandatory** in the skill text: "You MUST run `sruja lint` on the generated file before returning. If there are errors, fix them and re-run lint. Do not present a file that fails lint." Add a short list of common fixes (missing description, undefined ref, orphan, cycle) with one-line DSL fixes. Optionally add a "pre-flight" checklist: every element has description? technology on containers? every component in at least one relationship?

### 1.3 Scope and level of detail

**Today:** REFERENCE says "Target 10–30 components" in a few places; the agent may still over- or under-represent.

**Improve:** Add a clear **scope ladder** in the skill:

- **Minimal:** 1 system, 3–7 containers, entry points and main dependencies only.
- **Standard (recommended):** 1–2 systems, 10–30 components (containers + components), all key relationships and technologies.
- **Deep:** Multiple systems, 30–50 components, include internal components and key external systems.

Tell the agent to pick "Standard" unless the user asks for minimal or deep. This reduces noise (100+ components) and under-spec (3 components with no relationships).

---

## 2. DSL Consistency & Examples

### 2.1 Single canonical syntax in the skill

**Today:** `.cursorrules` and working examples use **assignment form**: `Id = system "Label" { }`, `Id = container "Label" { }`. REFERENCE.md sometimes shows **block form** without IDs (e.g. `system "Name" { }`, `external_system "Stripe" { }`, `person "End User"`) and `metadata { }` which may not match the current language spec.

**Improve:** Align all skill examples and REFERENCE to the **canonical form** used by the linter and LANGUAGE_SPECIFICATION.md:

- Always use assignment: `SystemId = system "Human Label" { ... }`, `ApiId = container "API" { ... }`.
- Use `database` (not only `datastore`) for data stores.
- Relationships: `SourceId -> TargetId "label"`.
- If `metadata` or `external_system` is not in the spec, remove or replace with documented constructs (e.g. external systems as `system "External Name"` with description).

This reduces "valid in skill example but fails lint" and makes the skill the single source of truth for syntax.

### 2.2 Minimal "hello architecture" snippet

**Today:** Examples are full files (ecommerce, microservices). There's no 10-line "smallest valid .sruja" in the skill.

**Improve:** Add a **minimal valid example** at the top of the agent SKILL or REFERENCE:

```sruja
// Smallest valid architecture (passes sruja lint)
User = person "User" { description "End user" }
App = system "My App" {
  description "Main application"
  Web = container "Web" { technology "React"; description "UI" }
  Api = container "API" { technology "Node.js"; description "REST API" }
  Web -> Api "HTTPS"
}
User -> App "uses"
```

So the model has a template that is guaranteed to lint and can scale up from there.

### 2.3 Relationship label patterns

**Today:** "Be specific" and "protocol + purpose" are stated; examples vary.

**Improve:** Add **relationship label patterns** to the skill:

- `"HTTPS - auth"`, `"gRPC - order validation"`, `"reads from"`, `"writes to"`, `"publishes events to"`, `"invokes"`.
- Bad: `"uses"`, `"calls"` (too vague unless combined with protocol).

This improves consistency and quality of relationship labels with minimal extra tokens.

---

## 3. Discovery & File Patterns

### 3.1 Language- and framework-specific hints

**Today:** REFERENCE has "Finding REST APIs", "Finding Databases", etc. with generic snippets.

**Improve:** Add a short **per-language/framework** subsection (e.g. Express, FastAPI, Django, Spring Boot):

- Where entry points live (e.g. `lib/express.js`, `main.py`, `Application.java`).
- How to recognize routes, services, and data access (file paths and naming).
- How to infer technology strings (e.g. "Node.js", "Express", "lib/application.js").

This helps the agent produce more accurate `technology` and `description` with less hallucination.

### 3.2 "What to read first" priority

**Today:** "Find key files" lists many patterns; no explicit order.

**Improve:** Add **read order**: (1) README + package/manifest (stack, scripts), (2) entry point(s), (3) one level of imports or route registration, (4) config for DB/queues/external APIs. "Do not read entire codebase; infer structure from entry points and dependencies." This keeps analysis fast and focused.

---

## 4. Validation & Feedback

### 4.1 Map lint errors to fixes

**Today:** "Fix any validation errors" lists missing descriptions, undefined refs, cycles, etc., but not how to fix them in DSL.

**Improve:** Add a **lint error → fix** table in REFERENCE or SKILL:

| Lint error / symptom | Fix |
|----------------------|-----|
| Missing description | Add `description "..."` to the element. |
| Undefined reference | Define the referenced ID before use, or fix typo in relationship. |
| Orphan component | Add at least one relationship `X -> Orphan "..."` or `Orphan -> Y "..."`. |
| Circular dependency | Break cycle: extract shared dependency or invert direction. |
| Missing technology (container) | Add `technology "..."` to the container. |

This makes the "validate and iterate" step deterministic and faster.

### 4.2 Post-generate checklist

**Improve:** Add a short **checklist** the agent should self-check before presenting:

- [ ] Every `system`, `container`, `component`, `database`, `person` has `description`.
- [ ] Every `container` has `technology`.
- [ ] Every element appears in at least one relationship (no orphans).
- [ ] Relationship labels are specific (protocol and/or purpose).
- [ ] `sruja lint` passes.

This reinforces thorough capture and reduces back-and-forth.

---

## 5. UX & Packaging

### 5.1 "Why Sruja" in one paragraph

**Today:** Installation and when-to-apply are clear; "why use this over Mermaid/PlantUML" is in IS_SRUJA_HELPFUL.md but not in the skill.

**Improve:** Add a **one-paragraph** to the main SKILL (sruja-architecture and/or sruja-architecture-agent): "Sruja gives you machine-readable architecture: every element has description and technology, relationships are explicit and labeled. So you can lint, diff, and run drift/baseline checks against code. Use it when you need architecture-as-data, not only diagrams." This sets expectations and differentiates from diagram-only tools.

### 5.2 Single "super prompt" for users

**Improve:** Document one **user-facing prompt** that works well in Cursor/IDE chat:

- "Analyze this repo and generate a Sruja architecture file (architecture.sruja). Be thorough: main systems, containers, technologies, descriptions for every element, and relationships with clear labels. Run sruja lint and fix until it passes. Use the sruja-architecture-agent skill."

Link this from README or INSTALL_AS_SKILL so users get great results with one ask.

### 5.3 Version and changelog

**Today:** Some skills have `metadata.version: "1.0.0"`.

**Improve:** Keep a short CHANGELOG in the skill repo (e.g. `skills/sruja-architecture-agent/CHANGELOG.md`) and bump version when you add the canonical prompt, scope ladder, or lint→fix table. So users and evaluators can tie "super awesome" behavior to a version.

---

## 6. Summary: Top 5 for "Super Awesome"

1. **Canonical thorough-capture prompt** in the agent skill + mandatory "run lint and fix until pass."
2. **One canonical DSL form** in all examples (assignment form, no unsupported constructs).
3. **Lint error → fix table** and a **post-generate checklist** so validation is fast and repeatable.
4. **Scope ladder** (minimal / standard / deep) so output is the right size by default.
5. **One user-facing "super prompt"** and a short "why Sruja" paragraph so adoption and quality are clear.

Implementing these will make the Sruja skill more consistent, thorough, and easy to use—without changing the DSL or the linter.
