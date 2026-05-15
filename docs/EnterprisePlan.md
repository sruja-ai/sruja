# 90-Day Enterprise Adoption Plan for Sruja

## Summary

Convert Sruja from a promising OSS architecture tool into a low-risk **company pilot candidate** by focusing on four outcomes in order: **trust**, **one clear pilot workflow**, **lower scanner noise**, and **published proof of value**.  
This plan assumes the goal is **enterprise adoption readiness**, not a hosted enterprise product launch.

## Implementation Changes

### 1. Trust and maturity hardening in the first 30 days
- Make all public versioning consistent across repo, docs, site, release notes, and security policy. Standardize on the current release line and remove stale `0.1.x` support references.
- Publish a short **product maturity page** covering:
  - stable features
  - experimental features
  - known limitations
  - supported operating systems/editors
  - supported language tiers tied to tested fixtures
- Publish a **compatibility and support policy**:
  - semver expectations
  - backward-compatibility promise for DSL and core CLI commands during pilot
  - issue triage/SLA expectations for pilots
- Add visible **maintainership signals**:
  - named maintainers
  - roadmap owner
  - “how to report enterprise blockers”
- Keep positioning narrow in top-level messaging:
  - “context engineering for the AI era”
  - lead with the sruja-architecture skill, deterministic repo evidence, validation, and reusable AI context
  - present CLI-only quickstart as the evaluation and automation path, not the primary product surface

### 2. Define one blessed adoption path and remove rollout ambiguity in days 15-45
- Standardize the default company workflow around existing commands only:
  - `sruja quickstart -r .`
  - `sruja lint repo.sruja`
  - `sruja sync -r .`
  - `sruja status -r . --format json`
  - `sruja review -r . --format json`
  - `sruja drift -r .`
- Rewrite adoption docs so every surface points to the same pilot path:
  - top-level README
  - adoption guide
  - “using in your project”
  - extension onboarding
- Ship a **pilot kit** with no decision-making required:
  - one GitHub Actions template
  - one minimal repo baseline example
  - one PR review workflow
  - one rollback guide
  - one “start in advisory mode, then enforce” guide
- Demote alternate aliases and side workflows from primary docs unless they are necessary for the pilot path.
- Add one sample multi-repo federation walkthrough using `publish` and `compose`, but keep it out of the default single-repo path.

### 3. Prioritize signal-to-noise reduction over new features in days 30-75
- Establish a fixed **noise-reduction backlog** for the next release cycle:
  - exclude or downgrade common non-actionable findings from tests, generated code, fixtures, and entrypoints
  - improve framework handling for the highest-value cases already called out in limitations
  - make first-run CI guidance default to baseline/advisory mode rather than hard fail
- Create a **scanner evaluation corpus** of representative repos:
  - Rust service
  - TS/JS web app
  - Python service
  - mixed-language repo
- Define measurable quality gates for the corpus:
  - no crashes
  - deterministic output
  - seeded cycle/layer/orphan cases detected
  - false-positive budget per repo agreed in advance
- Update docs and CLI guidance to be explicit about what Sruja does not infer reliably yet, instead of implying broad architecture understanding for dynamic systems.

### 4. Produce proof of value and a credible operating story in days 60-90
- Run 3 internal or design-partner pilots with a fixed success rubric:
  - one small repo
  - one medium service repo
  - one multi-repo or platform-style case
- Capture the same evidence for each pilot:
  - time to first useful output
  - number of real findings caught
  - number of ignored/noisy findings
  - developer sentiment
  - whether CI adoption stayed enabled after trial
- Publish 2-3 case studies with concrete before/after outcomes, not feature descriptions.
- Publish a simple **commercial/support posture**:
  - OSS-only with best-effort support, or
  - OSS + pilot support/consulting
  - contact path for enterprise help
- Use these pilot results to decide the next quarter:
  - continue hardening core adoption, or
  - invest in enterprise-facing capabilities later

## Public Interfaces and Product Surface

- Do **not** add new DSL concepts in this cycle.
- Do **not** add major new CLI commands unless a pilot blocker cannot be solved with docs/templates.
- Treat the following as the stable pilot surface for this plan:
  - `quickstart`
  - `lint`
  - `sync`
  - `status`
  - `review`
  - `drift`
  - `publish`
  - `compose`
- Public doc changes should make this stable surface explicit and mark everything else as secondary, experimental, or advanced.

## Test Plan and Acceptance Criteria

- **Docs consistency**
  - Version numbers, support windows, and command names are consistent across README, security docs, adoption docs, and extension docs.
- **Pilot path smoke tests**
  - A new user can go from install to first repo health report in under 15 minutes using only the documented path.
  - A repo can add advisory CI in under 30 minutes using the provided template.
- **Detection quality**
  - Seeded cycle, orphan, and layer-violation examples are caught in the evaluation corpus.
  - Noise from tests/generated files/entrypoints is reduced versus the current baseline.
- **Adoption proof**
  - At least 2 pilot repos keep Sruja enabled after the pilot.
  - At least 1 published case study shows a real issue caught or measurable review/onboarding improvement.
- **Trust signals**
  - Public maturity/support page exists.
  - Support/contact path exists.
  - Version/support inconsistency is eliminated.

## Assumptions and Defaults

- Time horizon is **90 days**.
- Target user is a company evaluating Sruja for a **pilot**, not a full enterprise procurement.
- Focus platforms are **GitHub** and **VS Code/Cursor** first.
- No hosted control plane, RBAC, SSO, or admin console work is included in this cycle.
- Breaking DSL or core CLI changes are out of scope.
- New feature work is deferred unless it directly unblocks trust, pilot setup, or scanner precision.
