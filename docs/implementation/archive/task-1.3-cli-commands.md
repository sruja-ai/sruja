# Task 1.3: CLI Commands

**Task Status**
- Status: Pending
- Owner: Unassigned
- Target Date: -
- Dependencies: Task 1.1, Task 1.2
- Last Updated: 2025-12-01

**Priority**: 🟡 High (User-facing)
**Technology**: Go
**Estimated Time**: 1 day
**Dependencies**: Task 1.1, Task 1.2

## Files to Create/Modify

* `cmd/sruja/json.go` - JSON export command
* `cmd/sruja/json_to_dsl.go` - JSON to DSL command

## Commands

```bash
# Export DSL to JSON
sruja export json <input.sruja> <output.json>

# Convert JSON to DSL (single file output)
sruja json-to-dsl <input.json> <output-dir> --format single

# Convert JSON to DSL (multiple files output - default)
sruja json-to-dsl <input.json> <output-dir> --format multiple

# Default format is "multiple" if not specified
sruja json-to-dsl <input.json> <output-dir>

# Launch local Studio (Go API server + React app)
sruja studio --port 5173
# Starts:
# - Go API server on http://localhost:5173
# - Serves React app (static files)
# - Provides API endpoints for file operations
# - Opens browser automatically

# Launch read-only viewer for JSON/HTML
sruja viewer <input.json|input.sruja.html> --port 8080

# Generate PR preview and output comment markdown
sruja pr preview [--changes <change1,change2>] [--pr-number <number>]

# Example
sruja pr preview --changes 003-add-analytics,004-add-payment --pr-number 123
# Outputs markdown comment for PR
```

**Output Format Options**:
- `--format single`: Generate one file with all sections
- `--format multiple`: Generate concept-based files (architecture.sruja, requirements.sruja, etc.)

**No other customization** - these are the only two options.

**Example**:

```bash
# Export to JSON
sruja export json ecommerce-platform.sruja ecommerce-platform.json

# Convert back to DSL - single file
sruja json-to-dsl ecommerce-platform.json ./output --format single
# Generates: ./output/ecommerce-platform.sruja (one file)

# Convert back to DSL - multiple files
sruja json-to-dsl ecommerce-platform.json ./output --format multiple
# Generates:
#   ./output/architecture.sruja
#   ./output/requirements.sruja
#   ./output/decisions.sruja
#   ./output/stories.sruja
#   ./output/scenarios.sruja

# Round-trip test
sruja export json input.sruja output.json
sruja json-to-dsl output.json ./output --format multiple
# Output files should reconstruct the original structure

# Start Studio locally
sruja studio --port 5173
# Then open browser: http://localhost:5173
```

## Acceptance Criteria

* [ ] `export json` command works
* [ ] `json-to-dsl` command works with `--format single` option
* [ ] `json-to-dsl` command works with `--format multiple` option (default)
* [ ] Single file output generates one file with all sections
* [ ] Multiple files output generates standard concept-based files
* [ ] Only standard file names are used (no customization)
* [ ] Commands handle errors gracefully
* [ ] Help text is clear and explains format options
* [ ] Output directory is created if it doesn't exist
* [ ] `studio` command starts Go API server + serves React app
* [ ] API endpoints work (load/save files)
* [ ] Studio can read/write `.sruja` files directly
* [ ] `viewer` command serves read-only diagrams
* [ ] `pr preview` command generates preview and PR comment markdown
* [ ] `pr preview` auto-detects changes from git diff if not specified
