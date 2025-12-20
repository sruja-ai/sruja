# Task 4.6: Visual Changes in Studio

## Goal

Enable developers to create changes visually in the Studio, integrated with change visualization and Go CLI change commands. **Note**: Discussions and reviews happen in external systems (GitHub PRs, Cloud Studio), not in the language itself.

## Scope

- Create changes from the current architecture view
- Build change sets (add/modify/remove) via drag-and-drop
- Link to ADRs for decision tracking
- Link to requirements/user stories
- Visual diff against current state
- Export change files for CLI

## UI Components

- Change Panel
  - Create/select change
  - Metadata: title, requirement, user story, ADR, owner, stakeholders
  - Change sequence (optional): versioned series for linear history
- Change Set Builder
  - Add/Modify/Remove elements
  - Relation edits with validation
  - Link to ADR (for decision tracking)
- Diff View
  - Side-by-side JSON/diagram diff
  - Show added/modified/removed elements
  - Filter by element type

## Data Model (Studio)

- Change
  - `id`, `title`, `requirement`, `userStory`, `adr`
  - `metadata`: `{ owner, stakeholders }`
  - `changes`: `{ add, modify, remove }`
  - `sequenceNumber` (optional) - linear change sequence identifier

## Interactions

- Create change from selection
  - Select nodes/edges → add to `changes.add`
- Modify existing elements
  - Edit properties/relations → `changes.modify`
- Remove elements
  - Mark for removal → `changes.remove`
- Link to ADR
  - Select ADR from list or create new ADR
- Link to requirement/user story
  - Select from existing or create new
- Visual diff
  - Compare change vs current JSON, show added/modified/removed

## File Operations

- Load change file
  - Load `.sruja` change file via API (Go reads file, converts to JSON)
- Save change file
  - Save directly to `.sruja` change file via API (Go converts JSON to DSL, writes file)
- No export step needed - direct file operations via Go API

## Integration Points (Go CLI)

- Commands
  - `sruja change create` - Create change file
  - `sruja change apply` - Apply changes
- Files
  - `architecture/changes/xxx-*.sruja` - Change files
- Studio ↔ CLI contract
  - JSON schema alignment with `docs/implementation/go/json-schema.md`
- External Systems
  - **GitHub PRs**: Use for discussions and reviews
  - **Cloud Studio** (future): Use for collaboration

## Related

- For proposal-oriented collaboration UI, see [Task 4.6: Visual Proposals](task-4.6-proposals.md). Use proposals in Cloud/PR workflows; use change files for CLI application and commit.

## Acceptance Criteria

- [ ] Create visual changes with metadata and change sets
- [ ] Edit relations and element properties as change modifications
- [ ] Link to ADRs for decision tracking
- [ ] Link to requirements/user stories
- [ ] Show change diff vs current JSON (diagram + JSON)
- [ ] Save change files directly via Go API
- [ ] Load change files directly via Go API

## Dependencies

- Change Visualization (Task 3.8)
- Change Commands (Task 1.5)
- JSON Schema contract

## Notes

- **Discussions/Reviews**: Use external systems (GitHub PRs, Cloud Studio)
- **Decision Tracking**: Use ADRs (already in language)
- **Collaboration**: Use GitHub PRs or Cloud Studio (future)
- **Keep it simple**: Studio creates changes, external systems handle collaboration
