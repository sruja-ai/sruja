---
name: sruja-architecture-collaboration
description: >
  Collaborative architecture intelligence with multi-agent teams, knowledge graphs,
  live sessions, and review workflows. Enables teams to build architecture together.
license: MIT
metadata:
  author: sruja-ai
  version: "1.0.0"
  dependencies:
    - sruja-architecture
    - sruja-architecture-agent
---

# Sruja Architecture Collaboration

Multi-agent collaborative architecture intelligence for teams building systems together.

## When to Apply

Use this skill when:
- Multiple AI agents or team members collaborate on architecture
- Architecture decisions need review, approval, or consensus
- Building shared knowledge from past architectures
- Running live architecture design sessions
- Capturing and learning from architecture decisions

## Quick Reference

| Feature | Description |
|---------|-------------|
| `role-analyst` | Gathers requirements, discovers existing systems |
| `role-architect` | Designs solutions, makes structural decisions |
| `role-reviewer` | Reviews proposals, identifies risks |
| `role-validator` | Ensures completeness, runs lint checks |
| `knowledge-patterns` | Shared library of reusable patterns |
| `decision-capture` | ADRs and decision traceability |
| `session-live` | Real-time collaborative editing |
| `workflow-review` | PR-based architecture review |

## Core Concepts

### Multi-Agent Team Roles

Specialized AI agent roles for collaborative architecture:

1. **Analyst** - Discovers requirements, interviews stakeholders, maps existing systems
2. **Architect** - Designs solutions, makes trade-off decisions, creates proposals
3. **Reviewer** - Reviews proposals, identifies risks, suggests improvements
4. **Validator** - Ensures completeness, validates constraints, runs checks
5. **Facilitator** - Coordinates agents, manages sessions, resolves conflicts

### Collaboration Workflows

1. **Discovery Phase**: Analyst gathers requirements, discovers context
2. **Design Phase**: Architect creates initial proposal
3. **Review Phase**: Reviewer evaluates, identifies gaps
4. **Refinement Phase**: Team iterates on feedback
5. **Approval Phase**: Validator confirms completeness

### Knowledge Graph

- Pattern Library: Reusable architecture patterns with provenance
- Decision Registry: Architecture Decision Records (ADRs) with context
- Traceability Matrix: Links requirements to decisions to components

## Installation

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-collaboration
```

## Usage Examples

### Multi-Agent Architecture Session

```
@analyst: Analyze requirements from docs/requirements.md
@architect: Design initial architecture based on analyst findings
@reviewer: Review the architecture for risks and gaps
@validator: Validate completeness and run lint checks
```

### Live Collaboration

```
Start a live architecture session with team roles:
- You are the facilitator
- Start with requirements from @analyst
- Create proposal with @architect
- Get review from @reviewer
- Finalize with @validator
```

### Knowledge Capture

```
After architecture is approved:
- Extract reusable patterns to knowledge library
- Create ADR for key decisions
- Link to requirements and components
```

## Full Guide

See `AGENTS.md` for complete guide with:
- Detailed role definitions and capabilities
- Collaboration workflow patterns
- Knowledge graph schema
- Live session protocols
- Review workflow integration
- Examples and templates

## Related Skills

- `sruja-architecture` - Core architecture design principles
- `sruja-architecture-agent` - Single-agent architecture discovery

## Resources

- [Architecture Decision Records](https://adr.github.io/)
- [C4 Model](https://c4model.com/)
- [Sruja Documentation](https://sruja.ai)
