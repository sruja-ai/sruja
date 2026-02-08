# Skill-Lint CI/CD Documentation

## Overview

Phase 2.2 implements comprehensive CI/CD validation for skill files using GitHub Actions. The system includes automated checks on every push and pull request to ensure skill files maintain quality standards.

## Components

### 1. Reusable Action: skill-lint-validate

**Location:** `.github/actions/skill-lint-validate/action.yml`

A reusable GitHub Actions composite action that builds skill-lint and runs validation checks on skill markdown files.

**Inputs:**

- `working-directory`: Repository root (default: ".")
- `files`: Glob pattern for skill files (default: "skills/\*_/_.md")
- `check-links`: Enable link checking (default: "true")
- `check-xrefs`: Enable cross-reference validation (default: "true")
- `test-code`: Enable code example testing (default: "true")
- `check-format`: Enable format checking (default: "true")

**Usage Example:**

```yaml
- uses: ./.github/actions/skill-lint-validate
  with:
    working-directory: .
    files: "skills/**/*.md"
    check-links: "true"
    check-xrefs: "true"
    test-code: "true"
    check-format: "true"
```

---

### 2. Workflow: unified-ci.yml (Updated)

**Location:** `.github/workflows/unified-ci.yml`

**Added Job:** `skill-files`

Runs on every push to main/develop and every pull request.

**Triggers:**

- Push to: main, develop
- Pull request to: main, develop

**Steps:**

1. Checkout repository
2. Use skill-lint-validate action
3. Run all four validation checks on all skill files

**Purpose:** Ensures all skill files in the repository are valid.

---

### 3. Workflow: skill-validation.yml

**Location:** `.github/workflows/skill-validation.yml`

A dedicated workflow for comprehensive skill file validation.

**Triggers:**

- Pull request to main/develop (paths: `skills/**`, `crates/skill-lint/**`, `.github/actions/skill-lint-validate/**`)
- Push to main/develop (same paths)

**Jobs:**

#### skill-validation

**Steps:**

1. Checkout repository
2. Set up Rust (with clippy)
3. Cache Rust dependencies
4. Build skill-lint
5. Check for broken links
6. Check for broken cross-references
7. Test code examples
8. Check formatting
9. Validate metadata schema (if schema file exists)
10. Comment on PR with results (only on PRs)

**Special Features:**

- Non-blocking warnings for link and xref checks (uses `|| exit 0`)
- Blocking errors for code tests and formatting
- PR comments with validation results
- Uses GitHub annotations for error/warning messages

**Example PR Comment:**

```markdown
## 📚 Skill Files Validation

✅ All validation checks passed!

The following checks were run:

- ✅ Link validation
- ✅ Cross-reference validation
- ✅ Code example testing
- ✅ Format checking
- ✅ Metadata schema validation

No issues found in skill files.
```

---

### 4. Workflow: skill-pr-check.yml

**Location:** `.github/workflows/skill-pr-check.yml`

Efficient PR validation that only checks changed skill files.

**Triggers:**

- Pull request to main/develop (paths: `skills/**`)

**Jobs:**

#### skill-pr-check

**Steps:**

1. Checkout repository (fetch-depth: 0 for full history)
2. Set up Rust
3. Cache Rust dependencies
4. Build skill-lint
5. Get changed files (using tj-actions/changed-files)
6. List changed skill files
7. Check links in changed files
8. Check cross-references in changed files
9. Test code examples in changed files
10. Check formatting of changed files
11. Generate summary

**Outputs:**

- `files-changed`: Boolean indicating if any skill files changed
- `changed-files`: Space-separated list of changed files

**Special Features:**

- Uses GitHub step grouping for better readability
- Only processes files that actually changed
- Generates GitHub Actions summary
- File-specific error annotations
- Non-blocking warnings for link/xref checks

**Example GitHub Actions Summary:**

```markdown
## Skill Files Validation Summary

Checked 5 skill file(s)

### Validation Checks

- ✅ Link validation
- ✅ Cross-reference validation
- ✅ Code example testing
- ✅ Format checking
```

---

### 5. Schema File

**Location:** `crates/skill-lint/skill-schema.json`

JSON Schema for validating skill metadata frontmatter.

**Required Fields:**

- `metadata.complexity`: Integer (1-5)
- `metadata.frequency`: String (rare, uncommon, common, very common)
- `metadata.confidence`: String (low, medium, high, very high)
- `metadata.category`: String (critical, high, medium, low)
- `metadata.level`: String (beginner, intermediate, advanced)

**Optional Fields:**

- `metadata.applicable`: Object with async flags
- `metadata.rust_version`: String (e.g., "1.56+")
- `metadata.alternatives`: Array of strings
- `metadata.related_rules`: Array of strings
- `metadata.tags`: Array of strings

**Usage:**

```bash
skill-lint validate --schema crates/skill-lint/skill-schema.json --path skills/
```

---

## Workflow Matrix

| Workflow             | Trigger                 | Scope        | Files Checked   | Speed |
| -------------------- | ----------------------- | ------------ | --------------- | ----- |
| unified-ci.yml       | Push/PR to main/develop | All          | skills/\*_/_.md | Full  |
| skill-validation.yml | PR with skill changes   | All          | skills/\*_/_.md | Full  |
| skill-pr-check.yml   | PR with skill changes   | Changed only | Modified files  | Fast  |

---

## Validation Checks

### 1. Link Checking

- Validates external HTTP/HTTPS URLs with HEAD requests
- Checks internal relative paths exist on filesystem
- 10 second timeout for HTTP requests
- Reports broken links with file, line, and reason

### 2. Cross-Reference Validation

- Validates internal markdown links
- Checks metadata references (related_rules, alternatives)
- Verifies referenced files exist
- Reports broken references with file location

### 3. Code Example Testing

- Extracts Rust code blocks (```rust)
- Validates syntax using syn parser
- Handles incomplete snippets automatically
- Auto-wraps code in main() function
- Reports syntax errors with simplified messages

### 4. Format Checking

- Formats YAML frontmatter (trim, normalize)
- Formats body content (trim, normalize spacing)
- Removes excessive blank lines
- Normalizes line endings (CRLF → LF)
- Reports formatting issues

### 5. Schema Validation

- Validates metadata structure against JSON Schema
- Ensures required fields are present
- Checks field types and values
- Validates enums and patterns

---

## GitHub Actions Features

### Step Grouping

```yaml
echo "::group::Checking links in $file"
# ... validation code ...
echo "::endgroup::"
```

### Error Annotations

```yaml
echo "::error file=$file::Format check failed"
```

### Warning Annotations

```yaml
echo "::warning file=$file::Link check failed"
```

### PR Comments

Automatically creates or updates comments on PRs with validation results.

### Job Summary

Generates GitHub Actions summary for quick overview of validation results.

---

## Continuous Integration Flow

```
Push/PR → unified-ci.yml
    ├─ rust job (build, test, format, clippy)
    └─ skill-files job (validate all skill files)

PR with skill changes → skill-validation.yml
    └─ skill-validation job
        ├─ Build skill-lint
        ├─ Check all validation types
        └─ Comment on PR

PR with skill changes → skill-pr-check.yml
    └─ skill-pr-check job
        ├─ Build skill-lint
        ├─ Get changed files
        ├─ Validate only changed files
        └─ Generate summary
```

---

## Performance

### unified-ci.yml

- **Time:** ~3-5 minutes
- **Scope:** All skill files
- **Use Case:** Full repository validation

### skill-validation.yml

- **Time:** ~4-6 minutes
- **Scope:** All skill files
- **Use Case:** PR validation with detailed feedback

### skill-pr-check.yml

- **Time:** ~1-2 minutes
- **Scope:** Changed files only
- **Use Case:** Fast PR feedback

---

## Maintenance

### Adding New Validation Checks

To add a new validation check:

1. Implement check in `crates/skill-lint/src/commands/`
2. Add to `skill-lint-validate` action
3. Add to relevant workflows

Example:

```yaml
- name: New validation check
  shell: bash
  run: ./crates/skill-lint/target/release/skill-lint new-check skills/
```

### Updating Schema

Modify `crates/skill-lint/skill-schema.json` and test:

```bash
cargo build --release -p skill-lint
./crates/skill-lint/target/release/skill-lint validate \
  --schema crates/skill-lint/skill-schema.json \
  --path skills/
```

---

## Troubleshooting

### Workflow Fails on Link Check

- **Issue:** External URLs timeout or are unavailable
- **Fix:** Check URLs are accessible, add to allow list if needed

### Workflow Fails on Format Check

- **Issue:** File formatting issues
- **Fix:** Run `cargo run -p skill-lint -- format skills/` locally

### Workflow Fails on Code Test

- **Issue:** Syntax errors in Rust examples
- **Fix:** Fix syntax errors, ensure complete code snippets

### Too Many Changed Files

- **Issue:** PR changes too many files
- **Fix:** Split PR into smaller, focused changes

---

## Future Enhancements

1. **Skill Metrics Dashboard**
   - Track validation trends over time
   - Identify common issues
   - Show skill coverage

2. **Automated Fix Suggestions**
   - Suggest fixes for common formatting issues
   - Provide code examples for failing tests
   - Auto-fix trivial issues with PR bot

3. **Skill Dependency Graph**
   - Visualize rule dependencies
   - Detect circular references
   - Suggest rule ordering

4. **Performance Optimization**
   - Parallel validation of multiple files
   - Caching of validation results
   - Incremental validation

---

## Summary

Phase 2.2 successfully integrates skill-lint into the CI/CD pipeline with:

- ✅ Reusable action for skill validation
- ✅ Full validation on every push
- ✅ Fast PR checks on changed files only
- ✅ Detailed PR comments
- ✅ GitHub Actions summaries
- ✅ JSON Schema for metadata validation

All skill files are now automatically validated on every push and pull request, ensuring high quality standards.
