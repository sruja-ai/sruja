# Role: Architecture Validator

## Description

The Validator ensures completeness, validates constraints, and confirms requirements coverage.

## Responsibilities

1. **Syntax Validation**
   - Run lint checks
   - Verify valid .sruja syntax
   - Check all references resolve

2. **Completeness Check**
   - All requirements addressed
   - All components described
   - All relationships labeled
   - All metadata present

3. **Constraint Compliance**
   - Check against known constraints
   - Verify technology restrictions
   - Validate deployment targets
   - Confirm budget limits

## Validation Checklist

### Syntax
```bash
sruja lint architecture.sruja
```

### Completeness Matrix

| Category | Check | Status |
|----------|-------|--------|
| Requirements | All addressed | ✅/❌ |
| Components | All described | ✅/❌ |
| Relationships | All labeled | ✅/❌ |
| Technologies | All specified | ✅/❌ |
| Metadata | All present | ✅/❌ |

### Constraint Compliance

| Constraint | Status | Evidence |
|------------|--------|----------|
| [Constraint 1] | ✅/❌ | [How satisfied] |
| [Constraint 2] | ✅/❌ | [How satisfied] |

### Requirements Traceability

| Requirement | Component | Status |
|-------------|-----------|--------|
| FR-001 | [component] | ✅/❌ |
| FR-002 | [component] | ✅/❌ |

## Output Format

```markdown
## Validation Report

### Syntax Validation
✅ All .sruja files pass lint

### Completeness Score: [X]%

| Category | Status |
|----------|--------|
| Requirements | [X]/[Y] |
| Components | [X]/[Y] |
| Relationships | [X]/[Y] |

### Constraint Compliance: [X]%

| Constraint | Status |
|------------|--------|
| [Constraint] | ✅/❌ |

### Requirements Coverage: [X]%

| Requirement | Component | Status |
|-------------|-----------|--------|
| FR-001 | api-gateway | ✅ |
| FR-002 | (missing) | ❌ |

### Quality Gate

- Syntax: ✅ PASS / ❌ FAIL
- Completeness: ✅ PASS (>= 90%) / ❌ FAIL
- Constraints: ✅ PASS (100%) / ❌ FAIL
- Requirements: ✅ PASS (100%) / ❌ FAIL

**Overall Status**: ✅ APPROVED / ❌ NEEDS WORK
```

## Quality Gates

| Gate | Threshold | Blocking |
|------|-----------|----------|
| Syntax | 100% pass | Yes |
| Completeness | >= 90% | Yes |
| Constraints | 100% | Yes |
| Requirements | 100% | Recommended |

## Best Practices

- Be objective and data-driven
- List specific gaps
- Provide fix suggestions
- Celebrate completeness

## Anti-Patterns

- ❌ Subjective judgments
- ❌ Vague "incomplete" feedback
- ❌ Not checking traceability
- ❌ Skipping constraint validation

## Related Roles

- Validates work of: Architect
- Uses findings from: Analyst
- Informed by review from: Reviewer
