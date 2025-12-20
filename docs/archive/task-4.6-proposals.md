# Task 4.6: Visual Proposals in Studio

## Goal

Enable teams to create, review, modify, and commit proposals visually in the Studio, integrated with change visualization and Go CLI handoff. Collaboration (discussions/reviews) happens in PRs or Cloud Studio.

## Scope

- Create proposals from the current architecture view
- Build change sets (add/modify/remove) via drag-and-drop
- Attach discussions, reviews, and action items
- Visual diff and conflict detection against current state and other proposals
- Export/import proposals compatible with Go CLI/Cloud Studio

## UI Components

- Proposal Panel
  - Create/select proposal
  - Metadata: title, team, status, requirement/user story
  - Iterations timeline
- Change Set Builder
  - Add/Modify/Remove elements
  - Relation edits with validation
  - Technology decisions (ADRs) editor
- Review Threads
  - Discussions (question/comment/concern/suggestion)
  - Responses and status (open/answered/resolved)
  - Action items with assignment/status
- Conflict View
  - Detect overlapping changes across proposals
  - Highlight conflicting elements and relations
- Diff View
  - Side-by-side JSON/diagram diff
  - Filter by team, element type, proposal

## Data Model (Studio)

- Proposal
  - `id`, `title`, `team`, `status`, `requirement`, `userStory`
  - `proposedChanges`: `{ add, modify, remove }`
  - `discussions[]`, `actionItems[]`, `reviews[]`
  - `iterations[]`, `decisionEvolution[]`
  - `change` (link set on commit)

## Interactions

- Create proposal from selection
  - Select nodes/edges → add to `proposedChanges.add`
- Modify existing elements
  - Edit properties/relations → `proposedChanges.modify`
- Remove elements
  - Mark for removal → `proposedChanges.remove`
- Attach discussions/action items
  - Inline thread creation on selected element
- Visual diff
  - Compare proposal vs current JSON, show added/modified/removed
- Conflict detection
  - Compare proposal scopes across all proposals, flag overlaps

## Import/Export

- Import proposal JSON from CLI/Cloud
  - Load `.json` proposal (via WASM for DSL when needed)
- Export proposal JSON for CLI/Cloud
  - Save compatible structure for collaborative review
- Commit handoff
  - Export approved proposal → generates change file via Go

## Integration Points

- Go CLI (handoff)
  - Map proposal to `change create` or use `proposal approve|commit` if retained
- Files
  - `architecture/proposals/PROP-xxx-*.json|.sruja` (optional)
  - `architecture/changes/xxx-*.sruja` (generated on commit)
- Studio ↔ CLI/Cloud contract
  - JSON schema alignment with `docs/implementation/go/json-schema.md`
- External Systems
  - **GitHub PRs**: Discussions and reviews
  - **Cloud Studio**: Collaboration and governance

## Acceptance Criteria

- [ ] Create visual proposals with metadata and change sets
- [ ] Discussions/action items attached with statuses
- [ ] Show proposal diff vs current JSON (diagram + JSON)
- [ ] Detect conflicts between parallel proposals
- [ ] Import/export proposals compatible with CLI/Cloud
- [ ] Export approved proposals for commit handoff
- [ ] Filter and highlight by team ownership

## Dependencies

- Change Visualization (Task 3.8)
- JSON Schema contract
- Cloud/PR workflows

## Notes

- **Discussions/Reviews**: Use external systems (GitHub PRs, Cloud Studio)
- **Decision Tracking**: Use ADRs (already in language)
- **Collaboration**: Use GitHub PRs or Cloud Studio (future)
- **Keep it simple**: Studio creates changes, external systems handle collaboration

## Related

- For commit-ready artifacts and direct CLI operations, see [Task 4.6: Visual Changes](task-4.6-changes.md). Use proposals for collaboration; use change files for application and commit.
