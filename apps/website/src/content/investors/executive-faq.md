---
title: "Executive FAQ"
weight: 2
summary: "Answers to common questions about adopting Sruja."
---

# Executive FAQ

## How is this different from static diagrams?
Diagrams are generated from an executable model. The same source drives docs, diagrams, and validation, keeping everything consistent.

## Will this slow down teams?
No. The DSL is concise; `fmt` and `lint` run fast in CI. Most teams see faster reviews and fewer rework cycles.

## Do we have to rewrite everything?
Start incrementally. Model core systems first, then add SLOs, scale, constraints, and views. Existing docs can be exported and aligned over time.

## How does governance work?
Policies, constraints, and conventions are codified in the model. `sruja lint` enforces them in CI, preventing drift and accidental violations.

## What about lock‑in?
Models export to `markdown`, `mermaid`, `svg`, `json`, and `d2`. Your architecture remains portable and reviewable outside the tool.

## How do we measure ROI?
Track review cycle time, incident rates tied to architecture mistakes, and cross‑service consistency. Most organizations see measurable improvements within 4–6 weeks.

## Is Sruja really free?
Yes. Sruja is open source (MIT licensed) and free to use. There are no licensing fees or restrictions. Professional consulting services are available for organizations that need implementation support, training, or custom integrations.

## What's the future roadmap?
Sruja is designed to evolve into a comprehensive platform for architectural governance:
- **Live System Review**: Compare actual runtime behavior against architectural models to detect drift and violations.
- **Gap Analysis**: Automatically identify missing components, undocumented dependencies, and architectural gaps.
- **Continuous Validation**: Monitor production systems against architectural policies and constraints in real-time.
- **Compliance Monitoring**: Track and report on architectural compliance across services and deployments.

These capabilities are planned for future releases as the platform matures. The current open source foundation provides the building blocks for this evolution. See the [Roadmap Discussions](https://github.com/sruja-ai/sruja/discussions) for details.
