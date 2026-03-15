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
6. **Relations** – Optional (when `include_relations` is true): list of relations as `from → to "label"`.
7. **Runtime view** – **Scenarios** (and flows) with Mermaid sequence diagrams.
8. **Requirements & decisions**
   - **Requirements** – Functional/non-functional/constraint requirements with ID, type, description, tags.
   - **Architecture Decision Records** – ADRs with ID, status, context, decision, consequences.
9. **Governance**
   - **Policies** – ID, category, enforcement, description.
   - **Constraints** – List of constraints.
   - **Conventions** – List of conventions.
10. **Analysis** – **Feedback Loops** and **Causal Loops** with ID, Mermaid diagrams and variables (causal).
11. **Glossary** / **Recommendations** – Optional stub sections when `include_glossary` / `include_recommendations` are true (no AST yet).
12. **Custom views** – When `use_views` and `include_all_views`: one subsection per defined view (title, description, Mermaid from resolved view).

User-controlled text (titles, descriptions, labels) is escaped for Markdown (backslash, backtick, square brackets, and `#` in headings) so output is safe for edge-case content.

### View-driven export

- **Single view** (`use_views` true, `view_name` set): document contains only the chosen view (title, view heading, description, one Mermaid diagram, optional “Elements in this view” list). If the named view is not found, export falls back to the full document.
- **All views** (`use_views` true, `include_all_views` true, no `view_name`): full document as above, plus a **Custom views** section at the end with one subsection per defined view (view title, description, Mermaid from `export_from_resolved_view`).

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
| id           | ✅       | **ID:** when non-empty |
| title        | ✅       | ### heading |
| status       | ✅       | **Status:** |
| context      | ✅       | **Context:** |
| decision     | ✅       | **Decision:** |
| consequences | ✅       | **Consequences:** |

### Policies (`Policy`)

| AST field    | Exported | Notes |
|--------------|----------|-------|
| id           | ✅       | **ID:** when non-empty |
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
| id             | ✅       | **ID:** when non-empty |
| title          | ✅       | ### heading |
| loop_id        | ✅       | **Loop ID:** |
| loop_type      | ✅       | **Type:** and symbol |
| description    | ✅       | **Description:** |
| relationships  | ✅       | Mermaid diagram (from, to, label) |

### Causal loops (`CausalLoop`)

| AST field      | Exported | Notes |
|----------------|----------|-------|
| id             | ✅       | **ID:** when non-empty |
| title          | ✅       | ### heading |
| loop_id        | ✅       | **Loop ID:** |
| loop_type      | ✅       | **Type:** and symbol |
| description    | ✅       | **Description:** |
| variables      | ✅       | **Variables:** list (id and label) |
| relationships  | ✅       | Mermaid diagram (from, to, effect, polarity) |

### Relations (when `include_relations` is true)

| AST field   | Exported | Notes |
|-------------|----------|-------|
| from        | ✅       | Source FQN in list item |
| to          | ✅       | Target FQN in list item |
| label       | ✅       | Quoted in list item |
| description | ❌       | Not in Relations section |
| technology  | ❌       | Not in Relations section |

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
- **KindDef / TagDef / Style / Extend:** DSL-only; not rendered as Markdown sections.
- **Relation description/technology:** in optional Relations section only from/to/label are shown; description and technology are not written there (they appear in Mermaid as edge label where applicable).

---

## Verification

To confirm export behaviour:

1. **Unit/integration tests** in `tests/markdown_export.rs` cover: overview, systems, persons, deployments, requirements (with id/tags), ADRs (with id), policies (with id), constraints, conventions, scenarios (with description), feedback loops (with id), causal loops (with id, variables), element metadata, escaping of special characters in headings/body, view-driven single view and all-views, and optional Relations section when `include_relations` is true.
2. **This doc** is the single place that lists what is and isn’t exported; update it when adding or changing exported fields.
