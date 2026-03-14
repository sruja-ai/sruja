# Sruja Architecture Helper Scripts

Deterministic helper scripts for common Sruja architecture workflows.

## Scripts

### collect-evidence.sh

Collects deterministic evidence from a codebase for use with the sruja-architecture skill.

**Usage:**

```bash
./collect-evidence.sh [path] [output_file]
```

**Examples:**

```bash
# Collect from current directory (outputs to evidence.json)
./collect-evidence.sh

# Collect from specific directory
./collect-evidence.sh ./src

# Collect to custom output file
./collect-evidence.sh . my-evidence.json
```

**What it does:**

1. Runs `sruja discover --context -r <path> --format json`
2. Saves evidence to specified output file
3. Displays summary of collected evidence

**Output includes:**

- Repository structure
- Detected technologies
- Module boundaries
- Entry points
- Dependencies
- Scan scope

**Next steps:**

Use the generated evidence file with the sruja-architecture skill in your AI editor to generate repo.sruja.

### validate-refine.sh

Validates and refines an existing repo.sruja file.

**Usage:**

```bash
./validate-refine.sh [repo.sruja] [repo_path]
```

**Examples:**

```bash
# Validate repo.sruja in current directory
./validate-refine.sh

# Validate specific architecture file
./validate-refine.sh my-arch.sruja

# Validate against specific repo path
./validate-refine.sh repo.sruja ./src
```

**What it does:**

1. Lints the architecture file (catches errors)
2. Checks for drift against code (if baseline exists)
3. Formats the architecture file

**Output includes:**

- Linting results
- Drift detection results (saved to drift-results.json)
- Formatted architecture file

**Next steps:**

1. Review drift-results.json if drift was detected
2. Use sruja-architecture skill to address drift
3. Export documentation: `sruja export markdown repo.sruja`

## Requirements

Both scripts require the Sruja CLI to be installed:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

## Integration with AI Skills

### Workflow 1: New Codebase

```bash
# 1. Collect evidence
./collect-evidence.sh

# 2. In AI editor, run:
# "Use sruja-architecture. Read evidence.json, generate repo.sruja based on evidence, then run sruja lint and fix until it passes."

# 3. Validate
./validate-refine.sh
```

### Workflow 2: Existing Architecture

```bash
# 1. In AI editor, run:
# "Use sruja-architecture. Read evidence.json and existing repo.sruja, compare, propose updates, then run sruja lint and fix."

# 2. Validate
./validate-refine.sh
```

### Workflow 3: CI/CD

```bash
# In CI pipeline:
./validate-refine.sh repo.sruja

# This will fail if:
# - Linting errors exist
# - Drift is detected (if baseline exists)
```

## Exit Codes

- `0`: Success
- `1`: Error (CLI not found, file not found, linting failed)

## See Also

- [SKILL.md](../SKILL.md) - Core skill orchestration guide
- [REFERENCE.md](../REFERENCE.md) - Detailed discovery and modeling guide
- [PROMPTS.md](../PROMPTS.md) - Reusable AI prompts
