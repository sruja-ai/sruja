# Sruja Internal Guide

## Vision

Architecture‑as‑code standard with open‑core model: free CLI + language; collaborative cloud and enterprise features.

## Product & Roadmap

- Core OSS: exporters (Mermaid, PlantUML, C4‑PlantUML, Structurizr), validation packs, LSP + VS Code.
- Cloud: collaboration, registry, policy engine, integrations, advanced rendering.
- Enterprise: SSO/SAML, audit logs, compliance, on‑prem.
- Releases: signed binaries, SBOM, Homebrew/Scoop/Winget; GoReleaser v2 with installer‑compatible archives.

## 90‑Day Plan

- Weeks 1–2: Fix CI failures; Language Reference v0.1; VS Code scaffold.
- Weeks 3–6: Mermaid/PlantUML exporters; LSP prototype; distribution/signing.
- Weeks 7–10: Cloud MVP; policy PR checks.
- Weeks 11–13: Launch docs/tutorials; announcement; early adopters; pricing/support.

## Content Guidelines

- Channels: Docs (reference), Tutorials (step‑by‑step), Courses (curricula), Blogs (narrative).
- Decisions: Stable → docs; evolving → blog/tutorial; single task → tutorial; multi‑module → course.
- Quality: Docs unambiguous/versioned; Tutorials with objectives, commands, outputs; Courses with exercises/capstones; Blogs with context and links.
- Maintenance: Doc stubs with features; migration blogs on breaks; CI compiles examples/docs; owners per feature.
- Cross‑links: Docs↔Tutorials; Courses→Tutorials; Blogs→Docs/Tutorials.
- Maturity: Idea → blog/RFC; Prototype → experimental tutorial; Stabilized → docs+tutorial; Deprecated → doc note + blog.

## Community & Monetization

- Governance/RFCs; contributor onboarding; curated templates; training/certification.
- Tiers: Free, Pro (Cloud), Enterprise; training, templates, support.

