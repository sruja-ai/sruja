# Sruja Feature Alignment Report

Generated: 2026-05-14

## Product Spine

Sruja's product spine is:

> Give AI-assisted software teams repo-grounded architecture truth: define intent, supply usable context, detect drift, and review changes against intent.

This report asks the same gate for each public feature family:

- **Define intent**: Does it help create or maintain reviewed architecture truth?
- **Understand context**: Does it help humans or AI agents understand the repo/task?
- **Detect drift**: Does it expose mismatch between declared truth, code, and evidence?
- **Review change**: Does it help evaluate whether a proposed change should ship?

Verdict labels:

- **Core**: directly strengthens the spine; removing it weakens a main workflow.
- **Support**: useful plumbing or UX for core workflows; do not lead marketing with it.
- **Tighten**: aligned, but overlaps with other features or needs a clearer user moment.
- **Question**: likely product noise unless a sharper workflow is defined.

## Executive Verdict

Sruja is not random. The strongest through-line is: **AI skill + `.sruja` truth + deterministic evidence + lint/drift/intent checks + task-scoped context**.

The main product risk is not lack of unity; it is **surface sprawl**. The repo exposes many commands and concepts that are individually defensible, but users can see them as a toolbox instead of one loop. The product should lead with the skill and the core loop, then treat many CLI commands as implementation surfaces for CI, automation, and advanced workflows.

Evidence reviewed:

- `docs/MESSAGING.md` for skill-first positioning and the "better evidence or better validation" feature principle.
- `docs/SCOPE.md` for declared product scope.
- `crates/sruja-cli/src/cli.rs` for public CLI commands and aliases.
- `extension/package.json` for VS Code command surfaces.
- `docs/LANGUAGE_SPECIFICATION.md` for DSL feature breadth.
- `docs/CONTEXT_ENGINEERING.md` and `docs/mcp_tools_reference.md` for AI/MCP/context surfaces.
- `docs/architecture/*.sruja` and `repo.sruja` for implementation-aligned architecture.

## Canonical Workflows

Every feature should fit at least one of these workflows:

1. **Create truth**: AI skill gathers evidence, asks targeted questions, writes `repo.sruja`, runs lint.
2. **Refresh truth**: sync/daily/review detect drift and suggest updates.
3. **Brief an agent**: ai/focus/ai-context/MCP give task-scoped context before coding.
4. **Review a change**: drift-pr/intent/compliance/critique/propose evaluate whether a change matches intent.
5. **Scale across repos**: publish/compose/MCP retrieve the relevant federated slice.

## Feature Map

| Feature family | Primary bucket | Decision | Product read |
|---|---:|---:|---|
| `sruja-architecture` skill | Define intent | **Core** | This is the clearest primary product surface. It turns Sruja from "learn a DSL" into "AI generates grounded architecture and validates it." |
| `.sruja` DSL / `repo.sruja` reviewed truth | Define intent | **Core** | The persistent artifact that makes the product more than a chat answer or generated diagram. |
| C4 elements, relationships, nesting, sources, ownership, aliases, criticality | Define intent | **Core** | These are the minimum useful architecture index: structure, identity, and links to real evidence. |
| Advanced DSL blocks: requirements, ADRs, scenarios/flows, SLOs, scale, deployment, contracts, state machines, policies, incidents, views, styles | Define intent / Review change | **Tighten** | Strong when used as progressive disclosure. Risky if marketed as "model everything." Keep them tied to validation, review, and AI context. |
| Parser, AST, diagnostics, formatter, compiler | Define intent | **Support** | Necessary foundation. User value is "valid, maintainable truth," not the parser itself. |
| `lint`, `validate`, lint baselines, GitHub annotations | Define intent / Review change | **Core** | This is the main trust gate after AI edits. |
| `fmt`, `list`, `tree`, `explain`, `diff`, `import`, `completions` | Define intent | **Support** | Good authoring/debugging tools. Keep available, but they are not the product story. |
| Export: JSON, Markdown, Mermaid, D2, context, GraphML, Neo4j, Obsidian, hydrate, inject, views | Understand context | **Support** | Valuable outputs, but docs correctly say Sruja is not a diagramming product. Lead with truth and validation, then export. |
| `scan` | Understand context | **Support** | Evidence engine for the skill, drift, context, and CI. Avoid positioning it as the standalone hero. |
| `discover context/explain/repomap/questions` and community detection | Understand context | **Tighten** | Good evidence and explanation layer, but overlaps with quickstart/onboard/ai. Needs a crisp "for AI generation and debug" role. |
| `quickstart` / `overview` | Understand context | **Tighten** | Good first value for CLI evaluation, but product messaging says skill first. Treat it as evaluation/diagnostic, not the main path. |
| `init` / `start` / prompt / hook / CI setup | Define intent / Detect drift | **Core** | Strong onboarding utility if it funnels users into the skill, repo truth, and CI checks. |
| `onboard` | Understand context | **Tighten** | Useful full-repo brief. Needs clear separation from `quickstart`, `discover explain`, and `ai`. |
| `sync` | Understand context / Detect drift | **Core** | Writes fresh evidence and truth status; central to keeping context current. |
| `status` / `doctor` | Detect drift | **Core** | Tells the user whether truth is fresh and trustworthy. |
| `review` / `daily` | Detect drift / Review change | **Core** | Strong daily loop. This should be one of the canonical product flows. |
| `watch` | Detect drift | **Support** | Useful live feedback while coding; keep as workflow convenience. |
| `drift`, `drift-pr`, `baseline`, deprecated `check` | Detect drift / Review change | **Core** | One of the defining product capabilities. `check` should stay hidden/deprecated. |
| `intent check`, `intent propose`, `intent evaluate`, `intent history` | Review change | **Core** | Essential for the "does code match intent?" promise, but depends on ADR/intent quality. |
| `compliance` | Review change | **Tighten** | Aligned as an aggregate gate, but should be framed as structural + intent + policy, not broad enterprise compliance unless patterns are concrete. |
| `critique` | Review change | **Core** | Strong AI-era feature: adversarial review grounded in architecture. |
| `propose create/list/approve` | Define intent / Review change | **Core** | Gives architectural edits a reviewable lifecycle instead of silently changing truth. |
| `impact` | Understand context / Review change | **Core** | Clear job: blast radius before refactor/review. |
| `focus` | Understand context | **Core** | One of the best-aligned features: task-scoped context, blast radius, decisions, AI instructions. |
| `ai` brief | Understand context | **Core** | Paste-ready agent briefing is directly aligned with context engineering. |
| `ai-context` / `context` alias | Understand context | **Core** | Structured editor/agent context export. Deprecate or hide old `context` naming in public docs. |
| MCP server and tools | Understand context / Review change | **Core** | Excellent structured interface for AI tools, especially if readonly mode is emphasized for safety. |
| `why`, `query`, hybrid/semantic/BM25 retrieval | Understand context | **Tighten** | Useful when linked ADRs/docs exist. Without intent docs, value is lower; keep but present as retrieval over evidence, not magic reasoning. |
| `context-score` | Understand context / Detect drift | **Core** | Strong metric if each score dimension produces action. Avoid becoming vanity score. |
| `health` | Detect drift | **Tighten** | Overlaps with context-score/status/quickstart. Define whether it is architecture health, not AI context readiness. |
| `context-graph` and registry dashboard | Understand context | **Tighten** | Useful inspection tools. Risk: visual product drift. Keep as derived views, not primary diagramming. |
| `index semantic`, `index registry`, `query registry` | Understand context | **Tighten** | Aligned with retrieval and architecture index, but needs a simpler user story: "make context findable." |
| `ingest` and `.sruja/context/` docs | Define intent / Understand context | **Core** | Critical bridge between code evidence and human decisions. |
| `learn`, evidence graph, learned facts, learn feedback | Understand context / Review change | **Tighten** | Very aligned if positioned as "hypotheses from evidence, never truth." Needs strong UX labels so users do not confuse learned facts with reviewed architecture. |
| Agentic memory: history/record/clusters/clear | Understand context / Review change | **Tighten** | Good as guardrails for future AI work. Keep scoped to architectural learnings, failed hypotheses, and review advice. |
| Agent loop: `agent run`, `agent plan`, `agent apply` | Review change | **Question / Tighten** | Potentially powerful, but easiest to feel random because it edges into general agent orchestration. Keep only if every run is bounded by Sruja evidence, plan artifacts, verification, and recorded learnings. |
| Run snapshots: `run show` | Review change | **Support** | Useful audit trail for agent loops and context generation. |
| Federation: `publish`, `compose`, `repo.bundle.json`, `system.index.json` | Understand context / Review change | **Core** | Strong scale story: local truth becomes cross-repo context with lineage and conflict handling. |
| VS Code extension: syntax, snippets, diagnostics, validation | Define intent | **Core** | Brings validation into the editing moment, especially after AI edits. |
| VS Code diagrams, focused diagrams, sequence diagrams, markdown preview/export | Understand context | **Support** | Good editor UX. Keep framed as derived previews, not the product center. |
| VS Code context commands: refresh context, drift, intent, context score, status, review, copy context pack | Understand context / Detect drift / Review change | **Core** | Strong because they put the core loop inside the editor. |
| VS Code skills/rules/agent guide/docs thread/component knowledge | Understand context | **Tighten** | Useful, but should feed the core AI workflow. Avoid becoming a generic docs browser. |
| VS Code MCP registration | Understand context | **Core** | Directly lowers setup friction for AI-tool integration. |
| WASM bindings | Support | **Core support** | Essential for browser/editor parity, but not a standalone user-facing promise. |
| mdBook docs, tutorials, examples, courses, challenges, quizzes | Understand context | **Support / Tighten** | Useful adoption engine. Courses/quizzes can dilute product focus if they read like a separate education product. |
| Templates, GitHub Actions workflows, install scripts | Define intent / Detect drift | **Support** | Good setup and CI accelerators. |
| Shell completions, version, packaging/release surfaces | Support | **Support** | Expected CLI polish; not strategic differentiators. |

## Biggest Consolidation Opportunities

1. **Onboarding overlap**: `quickstart`, `discover explain`, `onboard`, `ai`, and `init --prompt` all help a user understand or start. Pick one primary "start here" narrative:
   - Product marketing: install/use the skill.
   - CLI evaluation: `sruja start -r . --prompt` or `sruja quickstart -r . --generate-baseline`.
   - Human/AI briefing: `sruja onboard` for whole repo, `sruja ai` for task.

2. **Health metric overlap**: `status`, `doctor`, `health`, `context-score`, `quickstart`, and `daily` all report some form of health. Define separate jobs:
   - `status/doctor`: truth freshness and baseline state.
   - `context-score`: AI-readiness.
   - `health`: architecture graph health, or fold it into quickstart/status.
   - `daily/review`: action list.

3. **Retrieval naming overlap**: `why`, `query`, `ai-context`, `focus`, MCP retrieval, semantic/BM25/hybrid search can feel like many ways to ask the same thing. Publicly teach:
   - Use `focus` before a task.
   - Use `ai` for a paste-ready task brief.
   - Use MCP inside AI tools.
   - Use `why/query` for investigation.

4. **Agent orchestration boundary**: `agent run/apply` is the riskiest surface. It must not become "Sruja is a coding agent." The durable promise should be: Sruja supplies evidence, guardrails, plans, verification, and memory for agent work.

5. **Advanced DSL breadth**: contracts, state machines, loops, incidents, policy rules, deployment, SLOs, and views are all defensible. They should be introduced only as needed by workflow, not as a giant language checklist.

## Keep / Tighten / Cut Guidance

### Keep as core

- Skill-first architecture generation and maintenance.
- `repo.sruja` as reviewed truth.
- `lint`, `sync`, `drift`, `daily/review`, `focus`, `ai`, `ai-context`, `intent`, `critique`, `impact`, `context-score`, `ingest`, MCP, federation, and VS Code validation/context commands.

### Keep but position as support

- Exports, visualizations, syntax highlighting, snippets, formatting, listing/tree/explain helpers, completions, WASM, templates, docs, release tooling.

### Tighten before pushing harder

- `learn`, agent memory, agent loop, `health`, registry/index dashboard, `why/query` retrieval, advanced DSL modeling blocks, and education/course content.

### Candidate cuts or demotions

- Hidden/deprecated command aliases should stay hidden in docs: `check`, `evaluate`, `evolution`, old `context` naming.
- Any feature that cannot name one canonical workflow above should be moved out of primary docs until it can.

## Suggested Feature Contract Template

Use this for every new feature or major command:

| Field | Required answer |
|---|---|
| User | Human architect, app dev, reviewer, AI agent, maintainer |
| Moment | Before coding, during coding, after AI edit, PR review, daily sync, federation |
| Primary bucket | Define intent, understand context, detect drift, or review change |
| Input | Code, `.sruja`, git diff, docs, MCP request, editor selection |
| Output | Decision enabled for user or agent |
| Proof | Test, example workflow, score improvement, CI gate, or user evidence |
| Kill rule | Remove/fold if not used by a canonical workflow |

## Bottom Line

The product is unified when framed as:

> Sruja gives AI agents and humans durable, evidence-backed architecture context that can be validated, kept fresh, and reviewed.

The product feels random when every command is presented as equal. Keep the public story ruthlessly centered on the skill and the four-part loop: **define truth, provide context, detect drift, review change**.
