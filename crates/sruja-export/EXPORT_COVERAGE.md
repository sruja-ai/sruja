# Export coverage: AST → Markdown / Mermaid

This document verifies which AST fields are included in Markdown and Mermaid export and how the document is organized for professional architecture documentation.

## Document structure (Markdown)

The export order and section names follow a **context → building blocks → deployment → runtime → decisions → governance → analysis** flow, aligned with common practices (e.g. arc42, C4, and enterprise architecture docs):

1. **Document title** – `#` heading at the top. Uses `document_title` option, or first line of overview summary, or default "Architecture Overview".
2. **Table of contents** – Links to all sections in the same order as the body.
3. **Introduction & context**
   - **Overview** – Summary, audience, scope, goals, non-goals, risks. Optional L1 (context) Mermaid diagram.
   - **Stakeholders** – Persons and their descriptions (replaces legacy "Persons" heading).
4. **Building blocks** – **Systems** with per-system L2 container diagrams and per-container L3 component diagrams.
5. **Deployment view** – **Deployments** (nested deployment tree).
6. **Runtime view** – **Scenarios** (and flows) with Mermaid sequence diagrams.
7. **Requirements & decisions**
   - **Requirements** – Functional/non-functional/constraint requirements with ID, type, description, tags.
   - **Architecture Decision Records** – ADRs with status, context, decision, consequences.
8. **Governance**
   - **Policies** – Category, enforcement, description.
   - **Constraints** – List of constraints.
   - **Conventions** – List of conventions.
9. **Analysis** – **Feedback Loops** and **Causal Loops** with Mermaid diagrams and variables (causal).

This order helps readers understand context first, then structure, deployment, behaviour, and finally requirements, decisions, and governance.

## Markdown export – field coverage

### Overview (`OverviewBlock`)

| AST field   | Exported | Notes                    |
|-------------|----------|--------------------------|
| summary     | ✅       |                          |
| audience    | ✅       | **Audience:**            |
| scope       | ✅       | **Scope:**               |
| goals       | ✅       | **Goals:** list          |
| non_goals   | ✅       | **Non-goals:** list      |
| risks       | ✅       | **Risks:** list          |

### Elements (person, system, container, component, database, queue)

| AST field      | Exported | Notes                                |
|----------------|----------|--------------------------------------|
| name (id)      | ✅       | Used for title fallback              |
| title          | ✅       | Section heading                      |
| body.description | ✅     |                                      |
| body.technology | ✅     | **Technology:** (containers/systems)  |
| body.metadata  | ✅       | When `include_metadata`; **key:** value |
| body.scale     | ❌       | Not written to Markdown              |
| body.slo       | ❌       | Not written to Markdown              |
| body.constraints | ❌     | Per-element; use top-level block     |
| body.conventions | ❌     | Per-element; use top-level block     |
| body.style    | ❌       | Not written to Markdown              |
| tag_refs      | ❌       | Not written to Markdown              |

### Requirements (`Requirement`)

| AST field   | Exported | Notes                    |
|-------------|----------|--------------------------|
| id          | ✅       | **ID:** when id ≠ title  |
| title       | ✅       | ### heading              |
| type        | ✅       | **Type:**                |
| description | ✅       |                          |
| tags        | ✅       | **Tags:** comma-separated |

### ADRs (`Adr`)

| AST field    | Exported | Notes |
|--------------|----------|-------|
| id           | ❌       | Not written |
| title        | ✅       | ### heading |
| status       | ✅       | **Status:** |
| context      | ✅       | **Context:** |
| decision     | ✅       | **Decision:** |
| consequences | ✅       | **Consequences:** |

### Policies (`Policy`)

| AST field    | Exported | Notes |
|--------------|----------|-------|
| id           | ❌       | Not written |
| title        | ✅       | ### heading |
| category     | ✅       | **Category:** |
| enforcement  | ✅       | **Enforcement:** |
| description  | ✅       | |

### Constraints / Conventions blocks

| AST field     | Exported | Notes |
|---------------|----------|-------|
| entries[].key | ❌       | Only `value` written (constraints/conventions are list items) |
| entries[].value | ✅    | List item `- value` |

### Deployments (`DeploymentNode`)

| AST field  | Exported | Notes |
|------------|----------|-------|
| id         | ✅       | Used when label is None |
| label      | ✅       | Section heading         |
| technology | ✅       | **Technology:**         |
| children   | ✅       | Recursive headings     |

### Scenarios & flows (`Scenario` / `Flow`)

| AST field   | Exported | Notes |
|-------------|----------|-------|
| id          | ✅       | Passed to sequence diagram |
| title       | ✅       | ### heading |
| description | ✅       | Paragraph under heading |
| steps       | ✅       | Mermaid sequence diagram; step.from, step.to, step.description used |

### Feedback loops (`FeedbackLoop`)

| AST field      | Exported | Notes |
|----------------|----------|-------|
| id             | ❌       | Not written |
| title          | ✅       | ### heading |
| loop_id        | ✅       | **Loop ID:** |
| loop_type      | ✅       | **Type:** and symbol |
| description    | ✅       | **Description:** |
| relationships  | ✅       | Mermaid diagram (from, to, label) |

### Causal loops (`CausalLoop`)

| AST field      | Exported | Notes |
|----------------|----------|-------|
| id             | ❌       | Not written |
| title          | ✅       | ### heading |
| loop_id        | ✅       | **Loop ID:** |
| loop_type      | ✅       | **Type:** and symbol |
| description    | ✅       | **Description:** |
| variables      | ✅       | **Variables:** list (id and label) |
| relationships  | ✅       | Mermaid diagram (from, to, effect, polarity) |

---

## Mermaid export – coverage

### C4 flowcharts (L1/L2/L3)

| Source           | Exported | Notes |
|------------------|----------|--------|
| Element FQN      | ✅       | Node id (sanitized) |
| title            | ✅       | Node label (title or name) |
| body.description | ✅       | Second line of label (truncated) |
| body.technology  | ✅       | Third line of label (truncated) |
| kind             | ✅       | classDef (person, system, container, database, queue, component, external) |
| Relation from/to | ✅       | Edge endpoints (projected by view level) |
| Relation label   | ✅       | Edge label (truncated) |

### Sequence diagrams (scenarios/flows)

| Source              | Exported | Notes |
|---------------------|----------|--------|
| step.from           | ✅       | Participant and message source |
| step.to             | ✅       | Participant and message target |
| step.description    | ✅       | Message label on arrow |

### Feedback / causal loop diagrams

| Source        | Exported | Notes |
|---------------|----------|--------|
| relationships | ✅       | Nodes and edges |
| loop_type     | ✅       | Comment and class |
| effect/polarity (causal) | ✅ | Edge labels |

---

## Not exported (by design or future work)

- **Element:** scale, slo, per-element constraints/conventions, style, tag_refs.
- **ADR / Policy:** id (could add for traceability).
- **View definitions:** no dedicated “Views” section in current Markdown path (view-driven path exists in options but may not be wired in all code paths).
- **KindDef / TagDef / Style / Extend:** DSL-only; not rendered as Markdown sections.
- **Relation tags/technology:** relation metadata not shown in Markdown (only in Mermaid as edge label).

---

## Verification

To confirm export behaviour:

1. **Unit/integration tests** in `tests/markdown_export.rs` cover: overview, systems, persons, deployments, requirements (with id/tags), ADRs, policies, constraints, conventions, scenarios (with description), feedback loops, causal loops (with variables), and element metadata.
2. **This doc** is the single place that lists what is and isn’t exported; update it when adding or changing exported fields.
