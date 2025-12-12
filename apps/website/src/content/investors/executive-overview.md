---
title: "Executive Overview"
weight: 0
summary: "A living, executable architecture language for faster decisions, safer systems, and consistent teams."
---

# Executive Overview

Sruja is an **open source** architecture-as-code language that helps teams define, validate, and evolve their software architecture. Built by and for the community, Sruja provides a foundation for better architectural governance and can evolve into a platform for live system review and gap analysis.

## What Sruja Delivers
- **Living architecture**: A concise DSL that stays in sync with reality, version-controlled alongside your code.
- **Enforceable guardrails**: Lint rules, policies, constraints, and conventions validated in CI/CD.
- **Decision velocity**: Model, validate, and export diagrams in minutes.
- **Operational clarity**: Native `slo`, `scale`, `change`, and `snapshot` blocks for production concerns.
- **Interoperability**: Export to `markdown`, `mermaid`, `svg`, `json`, `d2` for integration with your toolchain.

## Why It Wins
- **Open source foundation**: Community-driven development, transparent roadmap, and extensible architecture.
- **Measurable outcomes**: Fewer incidents, faster reviews, consistent designs across teams.
- **One source of truth**: Code, docs, and diagrams generated from the same model.
- **Governance built‑in**: Codify standards and verify in CI, with future support for live system analysis.

## Quick Proof
```bash
sruja fmt architecture.sruja
sruja lint architecture.sruja
sruja export markdown architecture.sruja
```

## Adoption Snapshot
- Week 1: baseline model, enable lint in CI.
- Week 2: add `slo`, `scale`, constraints; export docs.
- Week 3: introduce views and governance; track `change` and `snapshot`.

## Making the Decision

**New to Sruja?** Start here:

1. **[Decision Framework](/docs/decision-framework)** - 5-minute quick assessment
2. **[Adoption Guide](/docs/adoption-guide)** - Complete evaluation guide
3. **[Executive FAQ](/investors/executive-faq)** - Common questions answered
4. **[Adoption Playbook](/docs/adoption-playbook)** - Step-by-step rollout plan
5. **[Community](/docs/community)** - Join the community and contribute

**For Investors**: See our **[Investor Deck](/investors/investor-deck)** for market opportunity, solution approach, and investment details.

**Key Questions:**
- Do you have architecture documentation challenges? → [Self-Assessment](/docs/adoption-guide#is-sruja-right-for-your-organization)
- What's the value for your organization? → [Value Calculator](/docs/adoption-guide#step-2-calculate-value--roi)
- Is your team ready? → [Technical Fit Assessment](/docs/adoption-guide#step-3-assess-technical-fit)

## Open Source & Community

Sruja is **free and open source** (MIT licensed) - no purchase required. The project is developed in the open with community contributions, transparent decision-making, and an extensible architecture that welcomes plugins and integrations.

**Need help with adoption?** Professional consulting services are available to help organizations implement Sruja, establish best practices, and integrate with existing toolchains. Contact the team through [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) to learn more.

## Future Platform Vision

Sruja is designed to evolve into a comprehensive platform for architectural governance:

- **Live System Review**: Compare actual runtime behavior against architectural models to detect drift and violations.
- **Gap Analysis**: Automatically identify missing components, undocumented dependencies, and architectural gaps.
- **Continuous Validation**: Monitor production systems against architectural policies and constraints in real-time.
- **Compliance Monitoring**: Track and report on architectural compliance across services and deployments.

These capabilities are planned for future releases as the platform matures. The current open source foundation provides the building blocks for this evolution.

---

**Note**: This guide helps you evaluate fit and plan adoption. Sruja is free to use, and the community welcomes contributions, feedback, and feature requests.
