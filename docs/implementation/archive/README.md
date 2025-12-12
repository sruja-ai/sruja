# Sruja Cloud Service (Commercial)

## Positioning

Complement the local CLI/IDE Studio with a hosted service for team collaboration, governance, and automation. Keep JSON as the contract and ensure local-first workflows remain fully supported.

## MVP Scope

- Git provider integration (GitHub/GitLab)
- OAuth login; personal and org workspaces
- Connect repo → read Sruja JSON/DSL; render Viewer and Studio
- Proposal workflows: create/review/modify/approve/commit via cloud UI
- PR automation: generate branches/commits/PRs from approved proposals
- Visual diff and conflict detection across branches/proposals
- Team ownership overlays; filters
- Audit trail for proposal reviews and actions

## Advanced (Paid Tiers)

- Real-time multi-user collaboration in Studio
- Rich discussions: threads, mentions, reactions, attachments
- Advanced analytics: timeline heatmaps, change impact graphs
- SSO (Okta/Azure AD), RBAC, org policies
- Compliance: audit logs export, data residency options
- Custom validators and policy checks in pipelines
- Scheduled snapshots and comparison reports

## Architecture Overview

- Frontend: Same Studio/Viewer React app hosted with auth context
- Backend API:
  - Auth (OAuth 2.0)
  - Repo access (Git provider APIs)
  - Proposal and review storage (Postgres)
  - Job runner for PR automation
- Storage:
  - Metadata DB (proposals, reviews, actions)
  - Minimal code storage; prefer ephemeral clones and PR diffs
- Git Integration:
  - Branch per proposal; commits generated from approved changes
  - PR raised with description and links to visual diff

## Data Handling & Privacy

- Default: do not store repository code; fetch as-needed via provider APIs
- Store proposal metadata, reviews, discussions, action items
- Ephemeral clones for diff generation and validation
- Encryption at rest and in transit

## Pricing Model (Indicative)

- Free: connect repo, view diagrams, local-first workflows
- Team: proposals with PR automation, review threads, basic analytics (per-seat)
- Enterprise: SSO/RBAC, compliance, advanced analytics, policy checks (per-seat + usage)

## Developer Workflow

- Local/IDE Studio remains primary authoring surface
- Cloud augments reviews, collaboration, PR automation
- JSON↔DSL round-trip stays identical; cloud uses the same contract

## Acceptance Criteria

- Connect Git repo; render Viewer/Studio from Sruja JSON
- Create and manage proposals in cloud; approvals tracked
- Generate branch/commit and open PR from approved proposal
- Visual diffs and conflict detection available in UI
- Team filters and ownership overlays
- OAuth auth with personal/org workspaces

## Roadmap

- Phase A: MVP (GitHub, OAuth, proposals, PR automation)
- Phase B: Collaboration (real-time, discussions, analytics)
- Phase C: Enterprise (SSO/RBAC, compliance, policies)



























