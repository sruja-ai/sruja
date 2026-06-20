---
name: Sruja
last_updated: 2026-06-20
---

# Sruja Strategy

## Target problem

Engineers using AI coding agents can't tell whether the agent's changes respected the system's architecture, module boundaries, or original intent. The agent writes the code and grades itself — so trust breaks down at exactly the moment the codebase becomes dangerous to touch blindly.

## Our approach

Make architecture and intent explicit and deterministic. Humans declare the architecture and policies as a contract; the agent is bounded and graded against that contract by an independent layer — never self-graded. Because real codebases bury knowledge in code, tribal memory, and silos, we ground the contract by extracting and lifting that existing knowledge rather than demanding teams author it from scratch.

## Who it's for

**Primary:** Staff / principal engineers responsible for architectural integrity across siloed teams — they're hiring Sruja to encode architecture and intent as a contract once, let teams move independently with AI agents, and catch drift early without reading every PR.

**Secondary:** Developers on those teams — daily consumers who code against the contract and get bounded and graded in real time as they use AI agents.

## Key metrics

- **Architecture drift rate (lagging)** — boundary/intent violations found per unit of change; should trend down as the contract matures. *Where: `sruja check` / `drift` runs. Measurable today.*
- **Policy churn (leading)** — edits per policy per quarter; high churn signals policies are too rigid or verbose. *Where: `repo.sruja` git history. Measurable today.*
- **Grounded task success rate (lagging)** — when the contract was injected into the agent, % of tasks that passed verification without rework. *Where: `agent_memory.json`. Measurable today.*
- **Escape rate (leading)** — violations found after merge ÷ total violations found; lower means the grader caught them before shipping. *Needs instrumentation.*
- **Active consumption (leading)** — % of devs whose AI agent pulled the contract in a given week. *Needs instrumentation.*

## Tracks

### Grounded authoring

Bootstrapping the architecture contract out of messy, real-world code — repomap, topology, `author_evidence`, agent-assisted extraction — so teams don't have to author it from scratch.

_Why it serves the approach:_ no contract means nothing to grade; this is what makes the bet tractable in codebases where knowledge is buried or tribal.

### Grading engine depth

Making the deterministic grader sharp and complete (drift, lint, intent, violations across more types), plus a light telemetry slice (escape-rate and active-consumption) so the grading bet becomes measurable where a local tool can't see today.

_Why it serves the approach:_ a weak grader breaks trust; an unmeasured grader can't prove it deserves trust.

### Bounded agent delivery

Getting the grader and grounding into where work actually happens — the `agent loop`, the IDE extension, and MCP inside Cursor, Claude Code, and other hosts.

_Why it serves the approach:_ a grader no one runs protects nothing; delivery is where the contract meets the code being written.

## Not working on

- Full integrated/external-context vision — Slack/Jira decision ingestion, infra and deploy awareness, edge agents, cross-application topology. Deferred this period; revisited once the three core tracks are measurably working. (The light-telemetry slice under grading is the only current toe in that water.)

## Marketing

**One-liner:** Architecture as a contract. AI changes independently graded — never self-graded.
