# Evidence packs (verify-task)

Evidence packs make Sruja outputs easy to archive as CI artifacts and attach to change records.

## What gets written

When enabled, `verify-task` writes a folder containing:

- `verify-task.json` — `verify_task/v2` output (includes provenance)
- `drift.json` — from the `drift_check` step (when present)
- `intent.json` — from the `intent_check` step (when present)
- `review.json` — from the `review` step (when present)

## How to generate

Default location (recommended for local runs):

```bash
sruja verify-task --profile review -r . --evidence-pack -f json
```

Override output directory (recommended for CI):

```bash
sruja verify-task --profile review -r . --evidence-pack-dir "$RUNNER_TEMP/sruja-evidence" -f json
```

## Confidence Reports

Evidence packs also work with the `confidence` command:

```bash
sruja confidence -r . --evidence-pack -f json
```

The confidence report references the evidence pack location so reviewers can drill into raw verification outputs when the summary isn't enough.

## SBOM guidance (optional)

Sruja does not generate SBOMs automatically, but you can attach them alongside evidence packs:

- Rust: `cargo sbom` (if available in your toolchain) or a third-party SBOM generator
- npm: `npm sbom`

