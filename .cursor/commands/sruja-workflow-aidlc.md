---
description: Run AI-DLC phase with Sruja workflow gates (inception, construction, operations)
---

You are driving **one phase** of an AI-DLC workflow integrated with Sruja. Obey `AGENTS.md` and [docs/AIDLC_INTEGRATION.md](../../docs/AIDLC_INTEGRATION.md).

The user must provide a **phase**: `inception`, `construction`, or `operations`.

## Shared setup (all phases)

1. Confirm repo root (`-r .`).
2. If no workflow exists, run:
   ```bash
   sruja workflow init -r . --title "<short title>" --with-aidlc --install-aidlc-rules
   ```
3. Otherwise ensure rules are installed:
   ```bash
   sruja workflow install-rules -r .
   ```
4. Load AIDLC steering from `.aidlc/aidlc-rules/` and rule details from `.aidlc-rule-details/` (follow `core-workflow.md`).
5. Before large edits, run:
   ```bash
   sruja workflow status -r . --check
   ```
   If it fails, fix listed `missing` / `aidlc_missing` items before proceeding.

## Phase — inception

- Ensure `inception/scope.md` and run `sruja workflow record-impact -r . --id <id>` when `target_elements` are set.
- Produce AIDLC artifacts under `inception/aidlc-docs/` (never mix app code into aidlc-docs).
- Optional grounded review:
  ```bash
  sruja workflow design-review -r . --id <id>
  ```
- When ready: `sruja workflow approve -r . --id <id> --phase inception` then `sruja workflow advance`.

## Phase — construction

- Gate must pass (`workflow status --check`); inception must be approved in strict mode.
- Use `.cursor/commands/sruja-plan-feature.md` for **task-layer** `construction/task-plan.md`.
- Link proposals in `construction/linked_proposal_ids.json`; use `sruja propose create --workflow-id <id>`.
- App code in repo root or `workspace/`; run project tests / `make check` as appropriate.
- Approve construction phase when artifacts and proposals are ready.

## Phase — operations

- Ensure `operations/deploy-scope.json` exists.
- Run `sruja drift -r . -a repo.sruja` and resolve violations before operations approval.
- Approve operations when deploy scope and drift are clean.

## Audit

After significant stage steps:
```bash
sruja workflow audit -r . --id <id> --event "stage complete: <name>"
```

Do not promote full `repo.sruja` rewrites; use proposals and `sruja propose approve`.
