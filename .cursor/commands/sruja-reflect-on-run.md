# Sruja: Reflect on agent run

Use this after an **`sruja agent loop`**, **`sruja agent apply`**, or **`sruja agent run --mode apply`** completes. This command guides **your editor agent** to distill lessons into agent memory.

## Prerequisites

- Repo root is the workspace root (`-r .`).
- A recent run exists under `.sruja/agent/runs/<run_id>/facts_bundle.json`.

## Steps

1. **Locate the latest run bundle**
   - List `.sruja/agent/runs/` and open the newest `facts_bundle.json`.
   - Note `run_id`, verification step outcomes, and any drift/lint facts referenced.

2. **Read current memory**
   - Open `.sruja/agent_memory.json` (or run `sruja agent history -r . -f json`).
   - Avoid duplicating existing `guardrail_advice` entries.

3. **Reflect (narrative only)**
   - Summarize: what was attempted, what passed/failed verification, what should change next time.
   - Optional: pipe `facts_bundle.json` to local LLM via Sruja enrichment pattern:
     ```bash
     jq -c . .sruja/agent/runs/<run_id>/facts_bundle.json | ollama run llama3
     ```
   - Do **not** treat model narrative as reviewed architecture truth.

4. **Record learnings**
   - For each actionable guardrail:
     ```bash
     sruja agent record -r . \
       -c "<short context>" \
       -H "<what was tried>" \
       -o success|failed \
       -g "<guardrail for future agents>" \
       --hitl-kind precedent|correction|guardrail
     ```
   - Or call MCP `sruja_record_learning` with the same fields.

5. **Curate**
   ```bash
   sruja agent curate -r .
   ```
   - Apply merges/updates only after human review (`sruja agent merge`, `agent update`, `agent delete --force`).

6. **Optional: team process**
   - If the learning is stable policy, add a Decision Record under `.sruja/decisions/` or update editor rules / `sruja-architecture` skill—not only agent memory.

## References

- [docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md)
- [docs/CONTEXT_ENGINEERING.md](../../docs/CONTEXT_ENGINEERING.md)
- [AGENTS.md](../../AGENTS.md#agentic-memory-utility-srujaagent_memoryjson)
