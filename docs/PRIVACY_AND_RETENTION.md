# Privacy and retention

Sruja is a local-first **deterministic harness**. It does not transmit repository content to external services unless you explicitly configure an enrichment provider.

## Data emitted by default

### CLI output

Commands like `drift`, `intent check`, `review`, and `verify-task` can print:
- File paths and line numbers (from scan evidence)
- Architecture element IDs and labels
- Violation messages and suggestions

### On-disk artifacts (under `.sruja/`)

Depending on which commands you run, Sruja may write:
- `.sruja/context.json` (sync output)
- `.sruja/cache/*.json` (scan caches for speed)
- `.sruja/workflows/<id>/...` (AI‑DLC durable artifacts)
- `.sruja/violations.baseline.json` (baseline suppressions)
- `.sruja/evidence-packs/<timestamp>/...` (when enabled via `verify-task`)

## No-telemetry posture

Sruja does not collect or send telemetry by default. Network calls occur only if you enable enrichment via:
- CLI flags like `--enrich` / `--enrich-cmd`, or
- environment variables / `.sruja/config.toml` for an enrichment provider.

## Retention guidance

Enterprise defaults:
- Treat `.sruja/workflows/<id>/` as **change record artifacts** (retain per your SDLC policy).
- Treat `.sruja/cache/` as **ephemeral** (safe to delete; will be rebuilt).
- Treat `.sruja/evidence-packs/` as **CI artifacts** (upload to your artifact store, then delete locally).

## Redaction guidance

If your organization treats file paths as sensitive metadata:
- Prefer running Sruja only on internal runners.
- Avoid publishing raw JSON outputs externally.
- Store evidence packs in access-controlled artifact stores.

