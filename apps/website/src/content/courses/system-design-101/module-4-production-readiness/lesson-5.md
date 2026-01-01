---
title: "Lesson 5: Tracking Architecture Evolution"
weight: 5
summary: "Track architecture evolution using Git, ADRs, and SLOs."
---

# Lesson 5: Tracking Architecture Evolution

## Why Track Changes?

Keeping a history of changes improves communication, auditability, and onboarding. Sruja integrates with Git to provide automatic change tracking, while ADRs document decisions and SLOs track evolution over time.

## Git: Automatic Change Tracking

Git automatically tracks all changes to your architecture files. No special syntax needed!

### Viewing Change History

```bash
# View all changes to architecture file
git log --oneline --follow architecture.sruja

# See what changed in a specific commit
git show <commit> -- architecture.sruja

# Compare two versions
git diff v1.0..v2.0 -- architecture.sruja

# View changes by author
git log --author="alice" -- architecture.sruja
```

### Version Tagging

```bash
# Tag major versions
git tag -a v2025.01 -m "Post-Black Friday stabilization"
git tag -a v2025.02 -m "After caching improvements"

# View architecture at specific version
git show v2025.01:architecture.sruja | sruja export

# Compare versions
git diff v2025.01..v2025.02 -- architecture.sruja
```

**Advantages:**
- ✅ **Automatic** - Every change is tracked
- ✅ **Powerful queries** - Git log, blame, diff
- ✅ **Attribution** - Know who made changes
- ✅ **Context** - PR reviews, discussions
- ✅ **Standard** - Every developer knows Git

## SLOs: Track Evolution Over Time

SLOs naturally track evolution through `target` vs `current` values:

```sruja
import { * } from 'sruja.ai/stdlib'

Shop = system "E-Commerce Shop" {
  API = container "API Service" {
    technology "Go"

    // SLOs show evolution: target vs current
    slo {
      availability {
        target "99.9%"
        window "30 days"
        current "99.85%"
      }
      latency {
        p95 "200ms"
        p99 "500ms"
        window "7 days"
        current {
          p95 "250ms"  // Improved from 300ms after adding Redis
          p99 "600ms"  // Improved from 700ms
        }
      }
      errorRate {
        target "< 0.1%"
        window "30 days"
        current "0.12%"
      }
    }
  }

  Cache = database "Redis Cache" {
    technology "Redis"
    description "Added to improve latency SLO (see ADR-005)"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL"
  }

  API -> Cache "Reads"
  API -> Database "Reads/Writes"
}

view index {
  title "Shop System with SLO Tracking"
  include *
}
```

**Advantages:**
- ✅ **Quantitative tracking** - Actual metrics over time
- ✅ **In context** - SLOs live with the components they measure
- ✅ **Self-documenting** - Current vs target shows progress
- ✅ **Link to ADRs** - Reference decisions that affected SLOs

## ADRs: Document Decisions and Rationale

ADRs link architectural changes to their rationale:

```sruja
ADR005 = adr "Add Redis cache for latency SLO" {
  status "accepted"
  context "Latency SLO not met - p95 was 300ms, target is 200ms. Database queries are bottleneck."
  decision "Introduce Redis cache for hot paths (product catalog, user sessions)"
  consequences "Latency improved to 250ms p95 (still above target but progress). Reduced database load by 40%. Trade-off: Added operational complexity for cache invalidation."
}

ADR006 = adr "Optimize database queries" {
  status "accepted"
  context "Latency still above target (250ms vs 200ms). Cache helped but not enough."
  decision "Add database indexes, optimize N+1 queries, implement query result caching"
  consequences "Latency improved to 200ms p95 (target met!). Database CPU usage reduced. Trade-off: More complex queries, slower schema migrations."
}
```

**Advantages:**
- ✅ **Rich context** - Why decisions were made
- ✅ **Status tracking** - accepted, rejected, superseded
- ✅ **Link to SLOs** - Connect decisions to measurable outcomes
- ✅ **Historical record** - Understand evolution of architecture

## Complete Example: Tracking Evolution

```sruja
import { * } from 'sruja.ai/stdlib'

Shop = system "E-Commerce Shop" {
  API = container "API Service" {
    technology "Go"
    slo {
      latency {
        p95 "200ms"
        window "7 days"
        current {
          p95 "200ms"  // Improved from 300ms (see ADR-005, ADR-006)
        }
      }
    }
  }

  Cache = database "Redis Cache" {
    technology "Redis"
    description "Added per ADR-005 to improve latency"
  }

  Database = database "PostgreSQL" {
    technology "PostgreSQL"
    description "Optimized per ADR-006"
  }

  API -> Cache "Reads"
  API -> Database "Reads/Writes"
}

// Link decisions to changes
ADR005 = adr "Add Redis cache for latency SLO" {
  status "accepted"
  context "Latency SLO not met - p95 was 300ms, target is 200ms"
  decision "Introduce Redis cache for hot paths"
  consequences "Latency improved to 250ms p95, reduced DB load by 40%"
}

ADR006 = adr "Optimize database queries" {
  status "accepted"
  context "Latency still above target (250ms vs 200ms)"
  decision "Add indexes, optimize N+1 queries"
  consequences "Latency improved to 200ms p95 - target met!"
}

view index {
  title "Shop System with Evolution Tracking"
  include *
}
```

## Best Practices

### 1. Use Git for Change Tracking

- ✅ Commit architecture changes with descriptive messages
- ✅ Use semantic versioning tags for major releases
- ✅ Link PR descriptions to ADRs when making architectural changes
- ✅ Use `git log` and `git diff` to view evolution

### 2. Document Decisions with ADRs

- ✅ Create ADR for each significant architectural decision
- ✅ Link ADRs to SLOs in descriptions (`"see ADR-005"`)
- ✅ Include context, decision, and consequences
- ✅ Update ADR status when decisions are superseded

### 3. Track Evolution with SLOs

- ✅ Include both `target` and `current` values
- ✅ Add descriptions linking to ADRs
- ✅ Update `current` values as metrics improve
- ✅ Document improvements in descriptions

### 4. Connect the Pieces

- **Git commits** → Track who changed what, when
- **ADRs** → Document why changes were made
- **SLOs** → Measure the impact of changes

## Practice

1. **View your architecture history:**
   ```bash
   git log --oneline architecture.sruja
   ```

2. **Create an ADR** documenting a recent architectural decision

3. **Update SLOs** with current values and link to ADRs

4. **Tag a version:**
   ```bash
   git tag -a v1.0 -m "Initial architecture baseline"
   ```

## Related

- `adr` for documenting decisions
- `slo` for tracking service level objectives
- Git for automatic change tracking
- `deployment` for tracking runtime topology changes
