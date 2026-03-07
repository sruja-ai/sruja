# Architecture Explainer + Memory — Demo

This demo runs the full AI memory loop from [ARCHITECTURE_EXPLAINER_MEMORY_IMPLEMENTATION_PLAN.md](./ARCHITECTURE_EXPLAINER_MEMORY_IMPLEMENTATION_PLAN.md) §12.

## Prerequisites

- Built CLI: `cargo build -p sruja-cli` (or use `SRUJA` below).
- **Optional:** LLM API key (`OPENAI_API_KEY`, `OPENROUTER_API_KEY`, etc.) for full explain/ask. Without it, explain/ask use a fallback (evidence preview only).
- **Optional:** `jq` — if present, the script extracts `answer_id` and `fact_id` from step 1 so step 3 (feedback) can run automatically. Without `jq`, step 3 is skipped unless you set `ANSWER_ID`/`FACT_ID` yourself.

## Run the demo

From the repo root:

```bash
chmod +x architecture/demo_ai_memory.sh   # once
./architecture/demo_ai_memory.sh
```

Or with a specific repo path and binary:

```bash
SRUJA="cargo run -p sruja-cli --" ./architecture/demo_ai_memory.sh /path/to/repo
```

## What the script does

1. **ai explain** — Topic "request flow"; outputs answer + evidence + fact IDs (or fallback if no LLM).
2. **ai ask** — "Where are architecture boundary risks?"
3. **ai feedback** — Marks one fact as wrong (uses IDs from step 1; skipped if no IDs).
4. **ai ask** again — Same question after feedback (memory influences future answers when LLM is used).
5. **timeline explain** — Smart commit subset + optional LLM evolution summary.

## Manual commands

```bash
sruja ai explain -r . --topic "request flow" [--format text|json]
sruja ai ask -r . "Your question" [--format text|json]
sruja ai feedback -r . --answer-id <id> --fact-id <id> --verdict correct|wrong|partial [--comment "..."]
sruja ai memory -r . [--format text|json]
sruja timeline explain -r . [--max-commits 5]
```

Memory is stored under `.sruja/memory/` in the repo (`facts.jsonl`, `interactions.jsonl`, `feedback.jsonl`, `state.json`).

## Reproducibility

The demo uses the current (or given) repo as the dataset. For **timeline explain**, commit selection is deterministic (score by subject keywords and changed files under `src/`, `crates/`, etc.), so the same repo yields the same commit list. No separate fixture repo is required. For **explain/ask**, output depends on scan results and optional LLM; set an LLM API key for full answers or rely on the fallback evidence preview.
