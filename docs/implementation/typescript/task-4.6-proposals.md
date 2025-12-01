# Task 4.6: Visual Changes in Studio

## Goal

Enable developers to create changes and migrations visually in the Studio, integrated with change visualization and Go CLI change commands. **Note**: Discussions and reviews happen in external systems (GitHub PRs, Cloud Studio), not in the language itself.

## Scope

- Create changes from the current architecture view
- Build change sets (add/modify/remove) via drag-and-drop
- Link to ADRs for decision tracking
- Link to requirements/user stories
- Visual diff against current state
- Export to migration/change files for CLI

## UI Components

- Change Panel
  - Create/select change
  - Metadata: title, requirement, user story, ADR
  - Change type: migration or simple change
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
  - `changes`: `{ add, modify, remove }`
  - `version` (for migrations)

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

## Import/Export

- Import change from CLI
  - Load `.json` or `.sruja` change file (via WASM for DSL)
- Export to change file
  - Save as `.sruja` change file for `sruja change apply`

## Integration Points (Go CLI)

- Commands
  - `sruja change create` - Create change file
  - `sruja change apply` - Apply changes
- Files
  - `architecture/changes/xxx-*.sruja` - Change files
  - `architecture/changes/xxx-*.sruja` - Change files
- Studio ↔ CLI contract
  - JSON schema alignment with `docs/implementation/go/json-schema.md`
- External Systems
  - **GitHub PRs**: Use for discussions and reviews
  - **Cloud Studio** (future): Use for collaboration

## Acceptance Criteria

- [ ] Create visual changes with metadata and change sets
- [ ] Edit relations and element properties as change modifications
- [ ] Link to ADRs for decision tracking
- [ ] Link to requirements/user stories
- [ ] Show change diff vs current JSON (diagram + JSON)
- [ ] Export to migration/change files compatible with Go CLI
- [ ] Import migration/change files from CLI

## Dependencies

- Change Visualization (Task 3.8)
- Change Commands (Task 1.5)
- JSON Schema contract

## Notes

- **Discussions/Reviews**: Use external systems (GitHub PRs, Cloud Studio)
- **Decision Tracking**: Use ADRs (already in language)
- **Collaboration**: Use GitHub PRs or Cloud Studio (future)
- **Keep it simple**: Studio creates changes, external systems handle collaboration

## Related

- For commit-ready artifacts and direct CLI operations, see [Task 4.6: Visual Changes](task-4.6-changes.md). Use proposals for collaboration; use change files for application and commit.
