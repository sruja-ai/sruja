# Testing Learn App Content

This document describes the test suite that validates all Sruja code samples in the learn app (playground examples, course content, and documentation).

## Overview

The test suite in `tests/learn_test.go` ensures that all Sruja code examples compile correctly and can be exported to both D2 and SVG formats. This prevents broken examples from appearing in the learn app.

## Test Coverage

### 1. Playground Examples (`TestPlaygroundExamples`)

**What it tests:**
- All `.sruja` files in the `examples/` directory
- Full compilation pipeline: parse → validate → export to D2 → export to SVG → render D2 to SVG

**Metadata support:**
- Uses `examples/manifest.json` for metadata
- Supports `skipPlayground`, `skipOrphanCheck`, `expectedFailure` flags
- Legacy comment support: `// SKIP_TEST`, `// SKIP_ORPHAN_CHECK`, `// SKIP_SVG_RENDER`, `// EXPECTED_FAILURE: reason`

**Test behavior:**
- Playground examples must compile successfully (unless marked as `expectedFailure`)
- Full SVG rendering is tested (unless `skipSVGRender` is set)
- Orphan detection can be skipped per example

### 2. Course Code Blocks (`TestCourseCodeBlocks`)

**What it tests:**
- All ````sruja` code blocks in `learn/content/courses/` markdown files
- Extracts code blocks using regex pattern: ````sruja\n...\n````

**Test behavior:**
- Skips orphan detection (course examples may be intentionally standalone)
- Skips SVG rendering (only tests parse/validate/export)
- Filters out syntax examples (placeholders, standalone elements, etc.)

### 3. Documentation Code Blocks (`TestDocsCodeBlocks`)

**What it tests:**
- All ````sruja` code blocks in `learn/content/docs/` markdown files
- Same extraction logic as course code blocks

**Test behavior:**
- Skips orphan detection (docs examples may be intentionally standalone)
- Skips SVG rendering (only tests parse/validate/export)
- Filters out syntax examples

## Validation Rules

The test suite uses the same validation rules as the CLI:

1. **UniqueIDRule**: Ensures all IDs are unique
2. **ValidReferenceRule**: Ensures all relations reference existing elements
3. **CycleDetectionRule**: Detects cycles (informational only - cycles are valid)
4. **ExternalDependencyRule**: Validates external dependencies
5. **SimplicityRule**: Provides guidance on system vs domain usage (warnings only)
6. **OrphanDetectionRule**: Detects unused elements (can be skipped)

## Filtering Non-Blocking Messages

The test suite filters out informational messages that don't block compilation:

1. **Cycle Detection**: Cycles are valid patterns (feedback loops, event-driven, mutual dependencies)
   - Filtered if message contains "Cycle detected" and "valid"

2. **Simplicity Guidance**: Warnings that suggest better syntax choices
   - Filtered if message contains "Consider using"
   - These are suggestions, not errors

## Export Testing

### D2 Export
- All code samples are exported to D2 format
- D2 script is validated for syntax correctness

### SVG Export (Sruja Format)
- Playground examples are also exported to SVG format
- SVG export failures are logged but don't fail tests (some edge cases may not render perfectly)
- Course and docs examples skip SVG rendering for performance

### D2 Rendering
- Playground examples render D2 to SVG (full pipeline test)
- Uses D2's dagre layout engine
- Validates that diagrams can be rendered successfully

## Running Tests

```bash
# Run all learn content tests
go test ./tests -v

# Run specific test
go test ./tests -v -run TestPlaygroundExamples
go test ./tests -v -run TestCourseCodeBlocks
go test ./tests -v -run TestDocsCodeBlocks
```

## Adding New Examples

### Playground Examples

1. Add `.sruja` file to `examples/` directory
2. Optionally add metadata to `examples/manifest.json`:
   ```json
   {
     "file": "my_example.sruja",
     "name": "My Example",
     "order": 100,
     "skipOrphanCheck": false,
     "expectedFailure": ""
   }
   ```

### Course/Docs Examples

1. Add code block to markdown file:
   ````markdown
   ```sruja
   system MySystem {
     container Web
   }
   ```
   ````

2. The test will automatically extract and validate it

## Syntax Example Filtering

The test suite automatically filters out syntax examples that aren't meant to be compiled:

- **Placeholders**: Code containing `...`
- **Standalone tags**: `tags [...]`
- **Standalone elements**: Single elements without architecture wrapper
- **Relations without definitions**: Relations without element definitions

## Troubleshooting

### Test Failures

1. **Compilation Error**: Check if the code is valid Sruja syntax
2. **Validation Error**: Check if validation rules are satisfied
3. **Export Error**: Check if the architecture can be exported (rare)

### Common Issues

1. **Orphan Elements**: Add `skipOrphanCheck: true` to manifest or use `// SKIP_ORPHAN_CHECK`
2. **Expected Failures**: Add `expectedFailure: "reason"` to manifest or use `// EXPECTED_FAILURE: reason`
3. **SVG Rendering Issues**: Add `skipSVGRender: true` to manifest or use `// SKIP_SVG_RENDER`

## Best Practices

1. **Keep Examples Simple**: Examples should demonstrate concepts clearly
2. **Test Locally**: Run tests before committing new examples
3. **Use Metadata**: Use manifest.json for playground examples instead of comments
4. **Document Intent**: If an example is intentionally incomplete, document why

## Future Improvements

- [ ] Add SVG export testing for course/docs examples (currently skipped)
- [ ] Add performance benchmarks for large examples
- [ ] Add visual regression testing for rendered diagrams
- [ ] Add syntax highlighting validation

