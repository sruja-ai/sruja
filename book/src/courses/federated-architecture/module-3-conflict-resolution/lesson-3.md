---
title: "Lesson 3: Conflict Prevention"
weight: 3
summary: "Prevent future conflicts with conventions and governance."
---

# Lesson 3: Conflict Prevention

## Prevention Strategy 1: Naming Conventions

Enforce naming standards:

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

## Prevention Strategy 2: Architecture Registry

Central registry of services:

```bash
# Register a new service
sruja register --service user-service \
  --domain user \
  --team platform-team \
  --contact platform@company.com

# Check if name is taken
sruja register --check-name "user-service"
```

Registry provides:
- Unique service IDs
- Domain ownership
- Team contacts
- Architecture approval

## Prevention Strategy 3: Governance Rules

Add policies that prevent conflicts:

```sruja
// In .sruja/policies.sruja
policy "No Duplicate Services" {
  description "Prevent modeling same logical service twice"

  rule "no two containers with same responsibility"
  rule "no two systems with same domain"
}

policy "Service Registry Required" {
  description "All services must be registered"

  rule "all deployed services must be in registry"
  enforcement "required"
}
```

## Pre-Composition Validation

```bash
# Before publishing, validate locally
sruja validate -r . --check-conflicts-with-registry

# Check against other published bundles
sruja validate -r . --check-against-bundles ./shared/bundles/
```

## CI/CD Integration

```yaml
# .github/workflows/prevent-conflicts.yml
- name: Check for Conflicts
  run: |
    # Download latest bundles
    sruja fetch-bundles -i ./shared/bundles/

    # Validate no conflicts
    sruja validate -r . --check-conflicts-with-registry
    sruja validate -r . --check-against-bundles ./bundles/
```

## Onboarding New Services

When creating a new service:

```bash
# 1. Register first
sruja register --service new-service \
  --domain new-domain \
  --team new-team

# 2. Get approved name
# new-domain-service

# 3. Create with approved naming
mkdir new-domain-service
cd new-domain-service
sruja init --name new-domain-service

# 4. Publish
sruja publish -r . -o ../bundles/new-domain-service.bundle.json
```

## Module Complete!

You've completed Conflict Resolution. You now understand:
- ✅ Conflict detection in composition
- ✅ Resolution strategies (manual, alias, merge, deprecate)
- ✅ Conflict prevention with conventions and registry

Next, Module 4 covers Federated Governance.