# OSS traction metrics (weekly)

Track **substance**, not extension installs or diagram exports.

## Weekly checklist

| Signal | What to record |
|--------|----------------|
| GitHub stars | Count + note any post/demo link that drove it |
| Issues / PRs | New contributors; drift/MCP questions vs “how to draw” |
| Drift screenshots | External posts or issues with **terminal/JSON drift output** |
| False positives | Issues labeled or tagged `false-positive` with **technical** repro (rule, file, why wrong) |
| Repeat scans | Same repo/user runs `drift` more than once (retention) |
| CI pilots | Teams that keep `drift --ci` or `verify-task` after trial |

## Do not optimize for

- VS Code marketplace install count alone
- Mermaid/diagram preview usage as primary KPI
- “Architecture intelligence” buzzword mentions without drift evidence

## Internal dogfood

Run on this repo:

```bash
sruja drift -r . --structural-only --advisory
```

Compare to [examples/oss-demo/sruja-repo-structural-excerpt.json](../examples/oss-demo/sruja-repo-structural-excerpt.json).

## Deferred

Community rehab post — after output-quality-bar holds on dogfood + one external sample repo ([KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md) transparent).
