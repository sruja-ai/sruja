# Offline / air-gapped installation and operation

This guide is for enterprise environments where CI runners or developer machines have **no outbound internet**.

## Goals

- Build or install `sruja` without `curl | bash`.
- Run `verify-task`, `drift`, and AI‑DLC workflow gates offline.
- Vendor AIDLC rules so `workflow install-rules` works without network.

## Option A: build from source (recommended)

Prereqs:
- Rust toolchain available on the machine (or installed from an internal mirror).

Steps:

```bash
just setup   # or: make setup
just build   # or: make build
./target/release/sruja --version
```

## Option B: pre-built binary via internal artifact store

Recommended approach:
- Download a release artifact once from a connected environment.
- Store it in your internal artifact registry.
- Distribute internally.

Then on air‑gapped machines:

```bash
./sruja --version
```

## Vendoring AIDLC rules

Sruja expects AIDLC rules to be vendored under `aidlc-workflows/` (repo root) or provided via `SRUJA_AIDLC_RULES`.

Offline workflow:
- Vendor `aidlc-workflows/` into your mono-repo mirror.
- Run:

```bash
sruja workflow install-rules -r .
```

This copies rules to `.aidlc/aidlc-rules/` and `.aidlc-rule-details/` for editor consumption.

## Offline CI posture

Enterprise-friendly gate set (offline):

- `sruja verify-task --profile review -r . -f json`
- `sruja workflow status --check -r .` (only for AI‑DLC projects)

If you need artifacts for audit trails, enable evidence packs:

```bash
sruja verify-task --profile review -r . --evidence-pack-dir "$CI_PROJECT_DIR/.sruja/evidence-packs" -f json
```

