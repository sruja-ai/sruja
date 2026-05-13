---
title: "Lesson 3: Conflict Prevention"
weight: 3
summary: "Prevent future conflicts with conventions and governance."
---

# Lesson 3: Conflict Prevention

## Prevention Strategy 1: Naming Conventions

Establish and enforce naming standards across teams:

```yaml
# .sruja/naming-conventions.yaml
containerNaming:
  pattern: "{team}-{function}"
  examples:
    - "user-api"
    - "order-processor"
    - "payment-gateway"

systemNaming:
  pattern: "{domain}-system"
  examples:
    - "user-system"
    - "order-system"
```

## Prevention Strategy 2: Architecture Governance

Central governance of service definitions:

```bash
# Review architecture before publishing
sruja lint repo.sruja

# Check for violations
sruja drift -r . -a repo.sruja

# Run compliance check
sruja compliance -r . -a repo.sruja
```

Governance provides:
- Centralized policies
- Compliance reporting
- Policy violation detection
- Architecture review workflow

## Prevention Strategy 3: Policy Enforcement

Add policies that prevent issues:

```sruja
<!-- partial -->
// In repo.sruja
policy "Architecture Quality" {
  description "Ensure architecture follows team standards"

  constraint "All containers need descriptions" {
    rule "container.description must be non-empty"
  }

  constraint "No unnamed relationships" {
    rule "edge.label must be non-empty"
  }
}
```

## Pre-Publish Validation

```bash
# Before publishing, validate locally
sruja lint repo.sruja

# Check for drift from baseline
sruja drift -r . -a repo.sruja

# In CI, enforce compliance
sruja drift --ci -r . -a repo.sruja
```

## CI/CD Integration

```yaml
# .github/workflows/architecture-check.yml
- name: Architecture Validation
  run: |
    # Lint DSL syntax
    sruja lint repo.sruja

    # Check for drift
    sruja drift --ci -r . -a repo.sruja

    # Publish bundle on main branch
    if [ "$GITHUB_REF" = "refs/heads/main" ]; then
      sruja publish -r . -o bundle.json
    fi
```

## Onboarding New Services

When creating a new service:

```bash
# 1. Initialize with Sruja
mkdir new-domain-service
cd new-domain-service
sruja init --auto

# 2. Follow naming conventions
# Name your system: new-domain-system
# Name your containers: team-function

# 3. Validate before publishing
sruja lint repo.sruja
sruja drift -r . -a repo.sruja

# 4. Publish
sruja publish -r . -o ../bundles/new-domain-service.bundle.json
```

## Hands-On: Prevent Conflicts

1. **Establish naming conventions for your team:**
   - Use pattern like `{team}-{function}` for containers
   - Document conventions in `docs/naming-conventions.md`

2. **Add policies to enforce standards:**
   ```sruja
   policy "Naming Standards" {
     constraint "Container names must match pattern" {
       rule "container.id matches '^[a-z]+-[a-z]+$'"
     }
   }
   ```

3. **Run validation in CI:**
   ```bash
   sruja lint repo.sruja
   sruja drift --ci -r . -a repo.sruja
   ```

4. **Review before publishing:**
   - Check existing bundles in shared location
   - Ensure your names don't conflict

## Learning Outcomes

- ✅ Establish naming conventions to prevent conflicts
- ✅ Use governance and compliance checks
- ✅ Define policies to enforce architecture standards
- ✅ Integrate validation into CI/CD pipelines

## Quiz: Test Your Understanding

### Q1: What is the primary benefit of naming conventions?

A) Faster compilation
B) Reduced memory usage
C) Preventing naming collisions and improving clarity
D) Better network performance

### Q2: What command should you run before publishing a bundle?

A) `sruja start`
B) `sruja lint` and `sruja drift`
C) `sruja build`
D) `sruja compile`

### Q3: How can you enforce architecture policies in CI/CD?

A) Using Git pre-commit hooks only
B) Running `sruja drift --ci` in your pipeline
C) Manual review only
D) Using a separate policy server

## Module Complete!

You've completed Conflict Resolution. You now understand:
- ✅ Conflict detection in composition
- ✅ Resolution strategies (manual, alias, merge, deprecate)
- ✅ Conflict prevention with conventions and governance

Next, Module 4 covers Federated Governance.