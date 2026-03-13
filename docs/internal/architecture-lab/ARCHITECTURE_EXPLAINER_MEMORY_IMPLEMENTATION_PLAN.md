# Architecture Explainer + Memory Loop: Implementation Plan

**Status:** Proposed execution plan  
**Date:** 2026-02-27  
**Target demo date:** 2026-03-14 (15 days)

## 1. Objective

Build a demoable capability that feels like "ask Cursor to explain architecture", but with a durable advantage:

1. Static architecture extraction remains the evidence source.
2. LLM converts structure into human-meaningful explanations.
3. Sruja stores architecture memory and user feedback.
4. Future answers improve from stored memory + commit timeline learning.

This must show clear value beyond a raw health score.

## 2. Product Contract (Demo Scope)

### 2.1 Required user outcomes

1. User asks architecture questions in plain language.
2. Sruja returns a grounded explanation with file evidence and confidence.
3. User marks answer/facts correct or wrong.
4. Sruja stores corrections and reflects them in later answers.
5. Sruja explains architecture evolution across a smart, small set of commits.

### 2.2 Explicit non-goals for this demo

1. Full autonomous architecture understanding without human correction.
2. Perfect semantic accuracy on first run.
3. Exhaustive timeline over all commits.
4. Replacing deterministic scan/drift logic with pure LLM reasoning.

## 3. System Design (Hybrid)

## 3.1 Pipeline

1. `scan` produces graph (`nodes`, `edges`) from repo.
2. context builder extracts high-signal slices relevant to a question.
3. LLM generates explanation constrained by evidence.
4. response parser extracts candidate facts with confidence.
5. memory store writes facts, answers, and feedback.
6. next query reuses memory + fresh graph context.

## 3.2 Core principle

LLM is a reasoning layer over evidence, not a source of truth by itself.

## 4. Data Model

Store memory per repository in `.sruja/` inside the repo.

## 4.1 Files

1. `.sruja/memory/facts.jsonl`
2. `.sruja/memory/interactions.jsonl`
3. `.sruja/memory/feedback.jsonl`
4. `.sruja/memory/state.json`

## 4.2 Fact schema (`facts.jsonl`)

```json
{
  "fact_id": "fact_01HXYZ...",
  "statement": "HTTP requests enter through API gateway before service routing.",
  "fact_type": "flow|boundary|dependency|decision|risk|ownership",
  "status": "candidate|confirmed|disputed|deprecated",
  "confidence": 0.74,
  "source": "scan|llm|user",
  "repo": "/abs/path/repo",
  "commit_sha": "1213b6b",
  "evidence": [
    {
      "kind": "file",
      "path": "crates/sruja-cli/src/commands/scan.rs",
      "line_hint": 120,
      "why_relevant": "contains drift flow orchestration"
    }
  ],
  "tags": ["request-flow", "entrypoint"],
  "created_at": "2026-02-27T20:00:00Z",
  "updated_at": "2026-02-27T20:00:00Z",
  "last_validated_sha": "1213b6b"
}
```

## 4.3 Interaction schema (`interactions.jsonl`)

```json
{
  "answer_id": "ans_01HXYZ...",
  "question": "Explain request flow and boundaries.",
  "response_markdown": "...",
  "used_fact_ids": ["fact_..."],
  "new_fact_ids": ["fact_..."],
  "confidence": 0.71,
  "commit_sha": "1213b6b",
  "created_at": "2026-02-27T20:05:00Z"
}
```

## 4.4 Feedback schema (`feedback.jsonl`)

```json
{
  "feedback_id": "fb_01HXYZ...",
  "answer_id": "ans_01HXYZ...",
  "fact_id": "fact_01HXYZ...",
  "verdict": "correct|wrong|partial",
  "comment": "This flow is outdated after refactor.",
  "actor": "user",
  "created_at": "2026-02-27T20:10:00Z"
}
```

## 5. CLI Contract

Use an `ai` command group to avoid collisions with existing `explain` command.

## 5.1 Commands

1. `sruja ai explain -r . --topic "request-flow" [--format text|json]`
2. `sruja ai ask -r . "How does auth boundary work?" [--format text|json]`
3. `sruja ai feedback -r . --answer-id <id> --fact-id <id> --verdict correct|wrong|partial [--comment "..."]`
4. `sruja ai memory -r . [--format text|json]` (inspect current memory summary)
5. `sruja timeline explain -r . [--max-commits 5] [--format text|json]` (smart commit subset + explanation)

## 5.2 Response requirements (all explain/ask outputs)

1. Answer section (human-readable architecture explanation).
2. Evidence section (file paths from scan graph).
3. Confidence score.
4. Fact IDs used/generated.
5. "What changed since last validated commit" when applicable.

## 6. Smart Commit Selection (High Signal Timeline)

Do not select all commits.

## 6.1 Selection algorithm

1. Take recent window (default last 200 commits, oldest->newest ordering for output).
2. Score each commit with deterministic features:
   1. changed files count under architecture-significant paths (`src/`, `crates/`, `services/`, `api/`, `infra/`, `docs/adr/`).
   2. commit subject keywords (`refactor`, `architecture`, `module`, `service`, `boundary`, `migration`, `split`, `merge`).
   3. touched file extensions in supported languages (`.rs`, `.go`, `.ts`, `.js`, `.py`).
3. Keep top 30 candidates by deterministic score.
4. Optional LLM rerank of these candidates to select final 3-5 architecture-significant commits.
5. Enforce diversity:
   1. avoid adjacent commits from same minute/author unless score gap is large.
   2. prefer commits spanning timeline intervals.

## 6.2 Timeline output

For each selected pair `base -> head`:

1. structural diff summary (`new/removed components`, `new/removed edges`).
2. top changed components (up to 5).
3. LLM explanation of architectural significance grounded in diff evidence.
4. confidence and caveats.

## 7. Implementation by Module

## 7.1 `crates/sruja-cli`

1. Extend CLI with `AiCommand` enum and `TimelineCommand::Explain`.
2. Add command handlers:
   1. `commands/ai_explain.rs`
   2. `commands/ai_feedback.rs`
   3. `commands/ai_memory.rs`
3. Wire to existing `scan`, `drift-diff`, `timeline suggest-refs`.

## 7.2 Reuse/extend existing code

1. Reuse LLM provider resolution from `commands/timeline.rs`.
2. Reuse graph retrieval and context building in `sruja-cli/src/ai/` (context, memory).
3. Persistence for repo-local `.sruja/memory` is in `sruja-cli` (e.g. `ai/memory.rs`).

## 7.3 New library module (inside `sruja-cli` first)

Create `crates/sruja-cli/src/ai/`:

1. `memory.rs` (read/write JSONL, index, lookup by tag/question).
2. `facts.rs` (fact extraction and normalization).
3. `context.rs` (build evidence bundle from scan graph + memory).
4. `prompt.rs` (strict prompt templates for grounded answers).
5. `timeline.rs` (smart commit scoring and selection).

## 8. Prompting and Guardrails

## 8.1 Prompt constraints

Every AI answer must obey:

1. cite only provided evidence file paths.
2. explicitly mark assumptions.
3. provide confidence in `[0.0, 1.0]`.
4. return machine-parsable JSON envelope for ingestion.

## 8.2 JSON envelope from model

```json
{
  "answer_markdown": "...",
  "confidence": 0.73,
  "facts": [
    {
      "statement": "...",
      "fact_type": "flow",
      "confidence": 0.68,
      "evidence_paths": ["crates/.../scan.rs"]
    }
  ],
  "assumptions": ["..."],
  "gaps": ["..."]
}
```

If parse fails, do not write new memory facts.

## 9. Quality Gates

## 9.1 Acceptance criteria (must pass before demo)

1. `ai explain` returns answer + evidence + confidence + fact IDs.
2. `ai feedback` updates fact status/confidence deterministically.
3. Re-ask after feedback shows changed answer behavior.
4. `timeline explain` selects max 5 commits by smart selection and explains each step.
5. All commands function without modifying git working tree state.

## 9.2 Test plan

1. Unit:
   1. fact schema roundtrip.
   2. confidence update rules.
   3. commit scoring function.
2. Integration:
   1. explain -> memory write -> feedback -> explain loop.
   2. timeline explain with mocked `git log`.
3. Golden tests:
   1. deterministic JSON envelope parsing.
   2. stable formatting for demo scripts.

## 10. Confidence Update Rules

Deterministic rules for feedback:

1. `correct`: `confidence = min(1.0, confidence + 0.15)`, status -> `confirmed`.
2. `wrong`: `confidence = max(0.0, confidence - 0.35)`, status -> `disputed`.
3. `partial`: `confidence = max(0.0, confidence - 0.10)`, status unchanged unless below 0.4.
4. Fact with confidence `< 0.25` on two consecutive `wrong` verdicts -> `deprecated`.

## 11. Execution Timeline (15 Days)

## Day 1-2 (2026-02-27 to 2026-02-28)

1. Add CLI skeleton for `ai` and `timeline explain`.
2. Implement memory schemas + file IO.
3. Add JSON envelope parser and validation.

## Day 3-5 (2026-03-01 to 2026-03-03)

1. Implement `ai explain` with static graph context.
2. Add evidence extraction and citations.
3. Persist interactions and facts.

## Day 6-7 (2026-03-04 to 2026-03-05)

1. Implement `ai ask` reuse path.
2. Implement `ai feedback`.
3. Apply confidence update rules and re-query behavior.

## Day 8-10 (2026-03-06 to 2026-03-08)

1. Implement deterministic smart commit scoring.
2. Add optional LLM rerank on top candidates.
3. Implement `timeline explain` output.

## Day 11-12 (2026-03-09 to 2026-03-10)

1. Add integration tests and golden outputs.
2. Harden failure modes (LLM unavailable, parse failure, empty memory).
3. Add non-LLM fallback messaging.

## Day 13-14 (2026-03-11 to 2026-03-12)

1. Create demo script (single command chain).
2. Create fixed demo dataset with known commit diffs.
3. Dry-run and tune prompt + output wording.

## Day 15 (2026-03-13)

1. Final rehearsal and bug fix buffer.
2. Freeze demo command outputs for presentation.

## 12. Demo Script (Target)

See **[DEMO_README.md](DEMO_README.md)** for how to run the demo script and manual commands.

1. `sruja ai explain -r . --topic "request flow"`
2. `sruja ai ask -r . "Where are architecture boundary risks?"`
3. `sruja ai feedback -r . --answer-id ... --fact-id ... --verdict wrong --comment "..."`
4. `sruja ai ask -r . "Where are architecture boundary risks?"` (show improved answer)
5. `sruja timeline explain -r . --max-commits 5`

This sequence proves static+LLM grounding, memory persistence, and learning over interaction + git history.

## 13. Risks and Mitigations

1. LLM response variance.
   1. Mitigation: strict JSON envelope + parser validation + retries.
2. Hallucinated facts.
   1. Mitigation: reject facts without evidence path match.
3. Over-selecting noisy commits.
   1. Mitigation: deterministic pre-score and top-k cap before LLM.
4. Timeline scan cost.
   1. Mitigation: max 5 commits for demo, cache graph snapshots.

## 14. Post-Demo Extensions

1. Promote memory module into dedicated crate (`sruja-memory`).
2. Add UI layer in extension/app for answer diff over time.
3. Add cross-repo memory federation for multi-repo systems.
4. Add "memory quality report" command for trust diagnostics.
