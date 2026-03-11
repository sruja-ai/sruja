# Representation, Drift, Policies, and Compliance

**Goal:** Focus on a single **essential representation** of architecture so we can reliably do **drift** (reality vs intent), **policies** (rules that must hold), and **compliance** (help teams get to green and stay there).

---

## 1. The Pipeline

```
  Representation (essential)     Reality (scan)        Output
  ─────────────────────────     ──────────────        ──────
  • Components & kinds              │
  • Allowed relationships           │
  • Boundaries / layers              │
  • Policy rules                     │
            │                        │
            ▼                        ▼
            └──────►  Drift  ◄────────┘   →  Violations (structural)
            │              
            └──────►  Policy check  ◄────  Scan graph  →  Policy violations
            │
            ▼
       Compliance report  →  What’s wrong, what to fix, checklist to green
```

- **Representation** = the minimal, declarative “essential architecture” we want to enforce.
- **Drift** = compare scan graph to that representation (structural: cycles, layers, god modules, orphans; and, when baseline exists, component/relationship diff).
- **Policies** = rules evaluated against the graph (e.g. “ExternalApi must not call Database”).
- **Compliance** = one place that combines drift + policy results and helps teams get (and stay) compliant.

---

## 2. Essential Representation (What to Capture)

We need a single conceptual model that is enough for drift, policies, and compliance. Below is the **essential schema**; it can be authored via `.sruja` + ADRs and normalized into this form.

### 2.1 Components

- **Id**, **kind** (module, service, database, external_api, system, container, etc.), **label**, optional **technology**, optional **description**.
- Purpose: “What exists (or is allowed) and what kind is it?”  
- Source today: DSL elements, intent `DeclaredComponent`, scan nodes.

### 2.2 Relationships

- **Source**, **target**, **kind** (calls, reads_from, writes_to), optional **label**.
- Purpose: “Which dependencies are declared or allowed?”  
- Source today: DSL relations, intent `DeclaredRelationship`, scan edges.

### 2.3 Boundaries / Layers

- **Boundary id**, **name**, **members** (component ids), **allowed_connections** (which other boundaries this one may depend on, and how, e.g. ApiCall, EventBus).
- Optional: **layer order** (e.g. UI → API → Domain → Data) so “layer violation” = dependency against the allowed direction.
- Purpose: “Where are the lines between parts of the system, and who can talk to whom?”  
- Source today: intent `DeclaredBoundary` (inside, allowed_connections, rules).

### 2.4 Policy Rules

- **Policy id**, **name**, **severity** (error, warning, info), **rules**: list of constraints.
- Each **constraint**: “when (source kind, target kind) then allowed = true/false” plus **message**.
- Purpose: “What dependencies are forbidden (or required) by policy?”  
- Source today: DSL `Policy`, graph `Policy` + `PolicyRule` + `Constraint`, intent `DeclaredPolicy` (constraint as string; needs parsing/normalization).

### 2.5 Single “Essential” Model (Target)

A minimal, serializable **EssentialArchitecture** (or equivalent) could look like:

```text
EssentialArchitecture:
  components: [ { id, kind, label, technology?, description? } ]
  relationships: [ { source, target, kind, label? } ]
  boundaries: [ { id, name, members[], allowed_connections[] } ]
  policies: [ { id, name, severity, rules: [ { source_kind?, target_kind?, allowed, message } ] } ]
```

- **Drift** uses: components + relationships (and optionally boundaries for boundary violations).
- **Policy check** uses: policies (evaluated against scan graph; kind-based constraints already exist in `sruja-graph`).
- **Compliance** uses: full model + drift result + policy violations to produce one report.

Today this is split across:

- **sruja-language** (DSL) + **sruja-diff** `program_to_graph`: components and relationships from .sruja (policies flattened to “module” in conversion).
- **sruja-intent**: DeclaredComponent, DeclaredRelationship, DeclaredBoundary, DeclaredPolicy (policy rules as strings; `detect_policy_violations` is a stub).
- **sruja-graph**: Policy + PolicyRule + Constraint (source_kind, target_kind, allowed, message); `find_policy_violations` works when policies are loaded into the graph.

So the **representation** exists in pieces; the focus is to **treat this essential model as the single source of truth** and feed drift and policy evaluation from it.

---

## 3. Drift (Representation vs Reality)

- **Reality** = scan graph (nodes and edges from code).
- **Representation** = essential model derived from .sruja (and optionally ADRs/intent).

**Structural drift (no baseline):**  
Already implemented in `sruja-diff`: cycles, orphans, layer heuristics, god modules → violations + health score.

**Drift with baseline:**  
- **sruja-diff** `program_to_graph` + `compare_graphs`: node/edge diff (added/removed components and relationships), plus structural violations.
- **sruja-intent** compare: undocumented/missing components, boundary violations, **policy violations** (currently stub).

**What we need for “essential representation → drift”:**

1. **One way to build the “proposal” graph** from the essential model (or keep using `program_to_graph` from DSL and ensure DSL captures components, relationships, and—where possible—boundaries).
2. **Boundary-aware drift:** map scan nodes to boundaries (e.g. by path or explicit mapping); flag edges that cross boundaries in a disallowed way. Intent’s boundary comparison is the right place; ensure it receives both DeclaredBoundary and scan graph.
3. **Policy violations in drift:** either implement `detect_policy_violations` in sruja-intent (e.g. by normalizing DeclaredPolicy rules to Constraint and evaluating against scan graph), or merge graph policy evaluation into the same drift/compliance pipeline so “policy violations” are a first-class part of drift output.

---

## 4. Policies (Setting and Evaluating)

**Setting policies:**

- **DSL:** `policy "Name" { category "…", enforcement "…", description "…" }` — today not converted to graph Constraint automatically.
- **Intent/ADRs:** DeclaredPolicy with PolicyRule (description + constraint string); needs a small grammar or convention to map to (source_kind, target_kind, allowed, message).
- **Graph:** Policy + PolicyRule + Constraint (source_kind, target_kind, allowed, message) — **evaluation is implemented** in `find_policy_violations`.

**Evaluating policies:**

- **Input:** Scan graph (reality) + list of Policy with Constraint.
- **Logic:** For each edge, check each rule; if (source_kind, target_kind) matches and `allowed == false` → violation.  
  Already in `sruja-graph::KnowledgeGraph::find_policy_violations`.
- **Gap:** Policies from DSL or intent are not consistently **loaded into the graph** (or into a single evaluation path) when running drift or compliance. So we need:
  - A single path: “essential representation” → policies normalized to Constraint list → run against scan graph → policy violations.
  - Optionally: populate KnowledgeGraph policies from .sruja or intent so existing `find_policy_violations` can be reused.

---

## 5. Compliance (Helping Them Get Compliance Done)

**Compliance** = “Are we aligned with the representation and all policies? If not, what exactly must we fix?”

### 5.1 Compliance report (target shape)

- **Status:** compliant | non_compliant.
- **Summary:** counts of structural violations, policy violations, boundary violations (if any).
- **Structural:** list of drift violations (cycles, layers, god modules, orphans, component/relationship diff) with **sources** (file:line) and **suggestions**.
- **Policy:** list of policy violations (policy name, rule, edge, message, severity).
- **Boundary:** list of boundary violations (which boundary, which edge, rule broken).
- **Checklist / remediation:** optional ordered list of “fix X then re-run” so teams can close the gap step by step.

### 5.2 Workflow

1. **Capture essential representation**  
   Author (or generate) .sruja + ADRs so we have components, relationships, boundaries, policies. Normalize to the essential model (or equivalent in code).
2. **Set policies**  
   Policies are part of the representation; ensure they are in a form that can be evaluated (Constraint with source_kind, target_kind, allowed, message).
3. **Run drift**  
   Scan repo → compare to representation (structural + boundary); run policy evaluation against scan graph.
4. **Produce compliance report**  
   Merge drift result + policy violations (+ boundary violations) into one report (status, summary, structural, policy, boundary, optional checklist).
5. **Remediate and re-run**  
   User fixes code (or updates representation); re-run until status is compliant.

### 5.3 Commands

- **Existing:** `sruja drift -r . -a architecture.sruja` (structural + baseline diff), `sruja intent check` (intent vs reality, includes policy count but policy violations stub).
- **Proposed:** A single **compliance** entry point, e.g.  
  `sruja compliance -r . -a architecture.sruja [-i intent-dir] [--format json|text]`  
  that:
  - Builds or loads essential representation from -a (and -i if present).
  - Runs scan, drift, and policy evaluation.
  - Outputs the compliance report (status, structural, policy, boundary, remediation hints).
  - Exit code 1 if non_compliant (for CI).

---

## 6. Implementation Priorities

### P0 – Representation

1. **Document** the essential model (components, relationships, boundaries, policies) as the single schema that drift and policies consume. Done above.
2. **Ensure** .sruja and intent can populate it: components and relationships already flow via `program_to_graph` and intent; boundaries and policies need to flow into one place (intent model and/or graph).

### P1 – Policies to compliance

3. **Wire policies into one path:** From DSL or intent, normalize policy rules to Constraint (source_kind, target_kind, allowed, message). Either populate sruja-graph policies and call `find_policy_violations`, or implement the same evaluation in sruja-intent/sruja-diff and emit policy violations alongside drift.
4. **Implement** `detect_policy_violations` in sruja-intent (or delegate to graph) so intent compare reports real policy violations, not a stub.

### P2 – Compliance report and UX

5. **Add** a **compliance report** type (e.g. in sruja-report): status, structural violations, policy violations, boundary violations, optional remediation checklist.
6. **Add** `sruja compliance` (or equivalent) that runs drift + policy check and prints/writes this report; support `--format json` and exit code for CI.

### P3 – Boundaries and remediation

7. **Boundary-aware drift:** Use DeclaredBoundary + scan graph to report boundary violations (e.g. “component A in boundary X called B in boundary Y but that connection is not allowed”).
8. **Remediation hints:** From violation types, attach concrete suggestions or doc links (e.g. “Break cycle: see …”, “Policy X: only ApiCall allowed between frontend and backend”).

---

## 7. Summary

- **Representation:** Define and use a single **essential architecture** (components, relationships, boundaries, policies) as the source of truth for drift and policy evaluation.
- **Drift:** Keep structural drift (cycles, layers, god modules, orphans) and baseline diff; add boundary violations when representation has boundaries.
- **Policies:** Express as rules (e.g. kind-based constraints); evaluate against the scan graph in one place; surface policy violations in the same pipeline as drift.
- **Compliance:** One report (structural + policy + boundary), one workflow (capture representation → set policies → run check → remediate → re-run), and one command (e.g. `sruja compliance`) so teams can get compliance done and keep it.

This keeps the focus on **representation first**, then **drift and policies** as consumers of that representation, and **compliance** as the outcome that helps teams get to green and stay there.

---

## Related documentation

- [CRATES_AND_ARCHITECTURE_INTELLIGENCE.md](CRATES_AND_ARCHITECTURE_INTELLIGENCE.md) – Crates and demo flow (scan, drift, analyze, why).
- [HEALTH_SCORE.md](HEALTH_SCORE.md) – How the structural health score is computed and how to interpret it.
- [INSIGHTS_USEFULNESS.md](INSIGHTS_USEFULNESS.md) – When insights (cycles, god modules, orphans) are actionable.
- [RUN_GUIDE.md](RUN_GUIDE.md) – How to run the CLI and demos.
