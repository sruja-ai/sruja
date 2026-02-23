# Workflow: Architecture Review Cycle

## Description

Structured process for reviewing and approving architecture proposals.

## Phases

### Phase 1: Proposal Submission

```
1. Architect creates .sruja proposal
2. Facilitator assigns reviewer
3. Validator runs initial lint
4. Proposal enters review queue
```

### Phase 2: Technical Review

```
1. Reviewer analyzes proposal
2. Checks against principles
3. Identifies risks and issues
4. Creates review report
5. Categorizes issues by severity
```

### Phase 3: Address Feedback

```
1. Architect reviews feedback
2. Addresses HIGH issues (required)
3. Addresses MEDIUM issues (recommended)
4. Documents changes made
5. Re-submits for review
```

### Phase 4: Validation

```
1. Validator checks completeness
2. Runs constraint compliance
3. Validates requirements coverage
4. Issues go/no-go decision
```

### Phase 5: Approval

```
1. All HIGH issues resolved
2. Completeness >= 90%
3. Constraints satisfied
4. Stakeholder sign-off obtained
5. Architecture approved
```

## Exit Criteria

| Criteria | Threshold |
|----------|-----------|
| HIGH issues | 0 |
| MEDIUM issues | 0 (recommended) |
| Completeness | >= 90% |
| Constraint compliance | 100% |
| Requirements coverage | 100% |
| Stakeholder approval | Yes |

## Issue Severity

### HIGH - Must Fix
- Security vulnerabilities
- Single points of failure
- Missing critical requirements
- Constraint violations

### MEDIUM - Should Fix
- Performance concerns
- Scalability limitations
- Operational gaps
- Documentation incomplete

### LOW - Nice to Have
- Minor improvements
- Future considerations
- Nice-to-have features

## Review Cadence

```
Initial Review: 1-2 days
Re-review: 1 day
Final Validation: Same day
Approval: Same day
```

## Best Practices

- Don't skip phases
- Address all HIGH issues
- Document all decisions
- Keep audit trail
- Communicate status

## Anti-Patterns

- ❌ Skipping review
- ❌ Ignoring HIGH issues
- ❌ Not documenting rationale
- ❌ Approving incomplete proposals
- ❌ No stakeholder involvement
