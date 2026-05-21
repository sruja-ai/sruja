# OSS demo fixtures

Pinned structural drift output for docs and CI. No GIF timers—output is the proof.

## Minimal clean scan

```bash
sruja drift -r examples/oss-demo/minimal --structural-only --advisory -f json
```

Golden file: [minimal-structural-drift.json](./minimal-structural-drift.json) (`clean_scan: true`).

## Sruja repo (violations sample)

```bash
sruja drift -r . --structural-only --advisory
```

See [sruja-repo-structural-excerpt.json](./sruja-repo-structural-excerpt.json) for a trimmed envelope (health score + sample findings).

## Regenerate

```bash
./scripts/oss-demo-refresh.sh
```
