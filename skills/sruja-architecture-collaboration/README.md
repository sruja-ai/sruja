# Sruja Architecture Collaboration Skill

Multi-agent collaborative architecture intelligence for teams building systems together.

## Quick Start

```bash
# Install the skill
npx skills add sruja-ai/sruja --skill sruja-architecture-collaboration

# Recommended: Also install dependencies
npx skills add sruja-ai/sruja --skill sruja-architecture
npx skills add sruja-ai/sruja --skill sruja-architecture-agent
```

## What It Does

Enables teams and AI agents to collaborate on architecture design through:

1. **Multi-Agent Roles** - Specialized AI roles (Analyst, Architect, Reviewer, Validator, Facilitator)
2. **Structured Workflows** - Discovery → Design → Review → Approve
3. **Knowledge Management** - Pattern library, ADRs, traceability
4. **Live Sessions** - Real-time collaborative design
5. **CI/CD Integration** - Automated architecture review in pull requests

## Agent Roles

| Role | Responsibility |
|------|----------------|
| **Analyst** | Discovers requirements, maps context |
| **Architect** | Designs solutions, creates proposals |
| **Reviewer** | Evaluates proposals, identifies risks |
| **Validator** | Ensures completeness, runs checks |
| **Facilitator** | Coordinates agents, manages sessions |

## Usage Patterns

### Multi-Agent Collaboration

```
@analyst: Analyze requirements from docs/
@architect: Design architecture based on findings
@reviewer: Review for risks and improvements
@validator: Validate completeness
@facilitator: Coordinate and approve
```

### Live Architecture Session

```
/session start "New Feature Architecture"
/session invite @analyst @architect @reviewer
/session goal "Design event-driven order processing"
/session context docs/requirements.md
... [collaborative design] ...
/session approve
/session archive
```

### CI/CD Integration

```yaml
# .github/workflows/architecture-review.yml
name: Architecture Review
on:
  pull_request:
    paths: ['**/*.sruja']
jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Architecture Review
        uses: sruja-ai/architecture-review-action@v1
```

## Skill Structure

```
sruja-architecture-collaboration/
├── SKILL.md              # Skill metadata and overview
├── AGENTS.md             # Full agent guide
├── rules/
│   ├── role-analyst.md   # Analyst role definition
│   ├── role-architect.md # Architect role definition
│   ├── role-reviewer.md  # Reviewer role definition
│   ├── role-validator.md # Validator role definition
│   ├── role-facilitator.md   # Facilitator role definition
│   ├── workflow-review.md    # Review workflow
│   ├── workflow-session.md   # Live session workflow
│   ├── knowledge-patterns.md # Pattern library
│   ├── knowledge-decisions.md # ADRs
│   └── knowledge-traceability.md # Traceability
└── examples/
    ├── session-chat-application.md   # Full session example
    └── session-microservices-migration.md # Migration example
```

## Examples

### Real-Time Chat Application
Full multi-agent session designing a chat platform:
- Requirements analysis
- Architecture proposal
- Review with feedback
- Validation and approval

### Microservices Migration
Migration from monolith to microservices:
- Current state analysis
- Target architecture design
- Migration strategy (Strangler Fig)
- Review and risk assessment

## Knowledge Management

### Pattern Library
```sruja
pattern "API Gateway" {
  applies_when ["Multiple services", "Need centralized auth"]
  benefits ["Reduced complexity", "Centralized concerns"]
  drawbacks ["Single point of failure", "Additional hop"]
}
```

### Decision Records (ADRs)
```markdown
# ADR-001: Use API Gateway

## Context
Multiple services need unified entry point

## Decision
Implement Kong API Gateway

## Consequences
+ Centralized auth
+ Rate limiting
- Additional latency
- HA configuration needed
```

### Traceability
```
Requirement FR-001
  → Decision ADR-002
    → Component api-gateway
      → Pattern api-gateway
```

## Related Skills

- **sruja-architecture** - Core design principles
- **sruja-architecture-agent** - Single-agent discovery

## Resources

- [Full Guide](./AGENTS.md)
- [Examples](./examples/)
- [Sruja Documentation](https://sruja.ai)
- [Skills.sh](https://skills.sh)
