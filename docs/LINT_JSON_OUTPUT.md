# Lint JSON output

When you run `sruja lint <file> --format json`, the CLI emits a single JSON object (to stdout) suitable for CI and IDE integrations.

## Schema

```json
{
  "ok": false,
  "error_count": 2,
  "warning_count": 1,
  "diagnostics": [
    {
      "code": "E204",
      "severity": "error",
      "message": "Cycle detected: A -> B -> A",
      "location": {
        "file": "architecture.sruja",
        "line": 10,
        "column": 1
      }
    }
  ]
}
```

- **ok**: `true` when there are no errors; `false` otherwise.
- **error_count**, **warning_count**: Counts by severity.
- **diagnostics**: List of issues. Each has **code**, **severity** (`"error"` | `"warning"` | `"info"`), **message**, and **location** (file, line, column).

## Diagnostic codes

Codes are defined in `crates/sruja-diagnostics/src/codes.rs`. Summary:

| Range   | Category        | Examples                          |
|---------|------------------|-----------------------------------|
| E1xx    | Parse/syntax     | E101 syntax error, E102 unexpected token |
| E2xx    | Semantic/structural | E201 duplicate id, E202 undefined ref, E204 cycle, E205 orphan, E206 layer violation |
| E3xx    | Validation rules | E301 invalid property, E302 missing field |
| E4xx    | Policy/governance| Policy and constraint violations  |
| W001+   | Warnings         | Best practice, style              |

Exit code: 0 when `ok` is true, non-zero when there are errors.
