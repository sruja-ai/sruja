# Lint JSON Output

When using the Sruja CLI for lint in CI or from tools (e.g. extension, agents), use `sruja lint --format json` to get machine-readable output. This document describes the schema.

## Invocation

```bash
sruja lint path/to/file.sruja --format json
```

- **Exit code:** 0 if no errors (warnings allowed); non-zero if there are errors or parse failure.
- **stdout:** Single JSON object (one line if compact; use for piping). On parse failure, JSON is still emitted with `ok: false` and diagnostics from the parser.

## Schema

```json
{
  "ok": true,
  "error_count": 0,
  "warning_count": 0,
  "diagnostics": [
    {
      "code": "E204",
      "severity": "error",
      "message": "Circular dependency between [A, B, C]",
      "location": {
        "file": "path/to/file.sruja",
        "line": 10,
        "column": 1
      }
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `ok` | boolean | `true` if there are no errors (file is valid); `false` if there are errors or parse failed. |
| `error_count` | number | Count of diagnostics with severity `error`. |
| `warning_count` | number | Count of diagnostics with severity `warning`. |
| `diagnostics` | array | List of diagnostic objects. |
| `diagnostics[].code` | string | Stable code (e.g. E201, E204, W001). Use for code→fix lookup; see [REFERENCE.md](../skills/sruja-architecture-agent/REFERENCE.md) § Lint error → fix. |
| `diagnostics[].severity` | string | `"error"` \| `"warning"` \| `"info"`. |
| `diagnostics[].message` | string | Human-readable message. |
| `diagnostics[].location` | object \| null | Source location when available. |
| `diagnostics[].location.file` | string | File path. |
| `diagnostics[].location.line` | number | 1-based line. |
| `diagnostics[].location.column` | number | 1-based column. |

## Diagnostic codes (overview)

- **E1xx** – Parse/syntax (E101–E104).
- **E2xx** – Semantic/structural (E201 duplicate id, E202 undefined ref, E203 invalid relation, E204 cycle, E205 orphan, E206 layer violation).
- **E3xx** – Validation (E301–E305).
- **E4xx** – Policy (E401).
- **W001** – Best practice.

Full code→fix table: [skills/sruja-architecture-agent/REFERENCE.md](../skills/sruja-architecture-agent/REFERENCE.md) (Step 4).

## Use in extension

When `sruja.lsp.path` is set, the extension can run `sruja lint --format json` and parse this JSON instead of scraping stderr for more reliable diagnostics.
