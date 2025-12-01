# Simplified Implementation Plan: Core Value First

## Philosophy

**Build what developers actually need, not what's technically impressive.**

Focus on solving real problems:
1. ✅ Visual editing saves hours
2. ✅ Easy sharing (HTML export)
3. ✅ Architecture evolution tracking (changes)
4. ✅ Catch mistakes early (LSP)

## Phase 1: Core Value (Weeks 1-8)

### Sprint 1: Foundation (Week 1-2)
**Goal**: Enable DSL ↔ JSON round-trip

- ✅ Task 1.1: JSON Exporter (AST → JSON)
- ✅ Task 1.2: JSON to AST Converter (JSON → AST)
- ✅ Task 1.3: CLI Commands (`sruja export json`, `sruja json-to-dsl`)

**Deliverable**: `sruja export json` and `sruja json-to-dsl` work

### Sprint 2: HTML Export (Week 2-3)
**Goal**: Generate shareable HTML diagrams

- ✅ Task 2.1: HTML Exporter (JSON → HTML template)
- ✅ Task 2.2: CLI Command (`sruja export html`)

**Deliverable**: `sruja export html` generates interactive diagrams

### Sprint 3: Interactive Viewer (Week 3-5)
**Goal**: Make diagrams interactive

- ✅ Task 3.1: Viewer Library Core (JSON → Diagram)
- ✅ Task 3.2: Basic Layout
- ✅ Task 3.3: Basic Styling
- ✅ Task 3.5: Interactivity (zoom, pan, click)

**Deliverable**: Interactive diagrams in HTML export

### Sprint 4: Visual Studio (Week 5-8)
**Goal**: Visual drag-and-drop editor

- ✅ Task 4.1: Studio Core (React UI)
- ✅ Task 4.2: Drag-and-Drop Editor
- ✅ Task 4.3: File Operations (read/write `.sruja` files via Go API)
- ✅ Task 4.5: Export (SVG/PNG from Studio)

**Deliverable**: Visual Studio for creating/editing architecture diagrams

**Studio Architecture**: One Studio, multiple modes
- **Local Mode**: `sruja studio` - reads/writes files directly
- **Preview Mode**: View-only, shareable links (future)

## Phase 2: Polish (Weeks 9-11)

### Sprint 5: Change Tracking (Week 9-10)
**Goal**: Track architecture evolution

- ✅ Task 1.5: Change Commands (create, apply, validate)
- ✅ Task 3.8: Change Visualization (diff, timeline)

**Deliverable**: Track and visualize architecture changes

### Sprint 6: Developer Experience (Week 10-11)
**Goal**: Catch mistakes early, smoother workflow

- ✅ Task 1.7: LSP (error diagnostics, basic completion)
- ✅ Task 4.4: Studio Polish (undo/redo, shortcuts)

**Deliverable**: LSP shows errors in IDE, Studio has basic polish

## Phase 3: Adoption (Weeks 12-14) - Optional

### Sprint 7: Easy Tryout (Week 12-13)
**Goal**: Lower barrier to entry

- ⚠️ Task 4.7: Public Studio (simplified)
  - **Simplified**: Use Go API backend (no WASM initially)
  - **Goal**: Let people try without installing CLI
  - **Defer if**: Adoption is fine without it

### Sprint 8: IDE Integration (Week 13-14) - Optional
**Goal**: IDE support

- ⚠️ Task 5.1: VS Code Extension
  - **Simplified**: Basic LSP integration, Studio webview
  - **Defer if**: CLI + Studio is sufficient

- ❌ Task 5.2: JetBrains Plugin
  - **Defer**: Build only if users request it

## What We're NOT Building (Initially)

### Deferred Features

❌ **Self-Hosted Studio**
- **Why**: Most teams can use CLI Studio or Public Studio
- **When**: Build if teams specifically request it

❌ **JetBrains Plugin**
- **Why**: VS Code is more popular, CLI works for others
- **When**: Build if JetBrains users request it

❌ **Complex Deployment Options**
- **Why**: Most teams use Docker or K8s
- **When**: Document additional options as needed

❌ **Canary Publishing**
- **Why**: Regular releases work fine
- **When**: Build if needed for testing

❌ **Multiple Studio Codebases**
- **Why**: One Studio with modes is simpler
- **When**: Split only if maintenance becomes an issue

## Simplified Architecture

### Studio: One Codebase, Multiple Modes

```
local-studio/
├── src/
│   ├── App.tsx              # Main app
│   ├── modes/
│   │   ├── LocalMode.tsx   # Reads/writes files (CLI Studio)
│   │   ├── PreviewMode.tsx # View-only (future)
│   │   └── PublicMode.tsx   # No auth, try it out (future)
│   └── ...
```

**Modes**:
- **Local Mode** (default): `sruja studio` - connected to local files
- **Preview Mode** (future): View-only, shareable links
- **Public Mode** (future): No auth, try without install

**Benefit**: One codebase, less maintenance, same functionality

### Deployment: Focus on Common Cases

**Document**:
1. Docker (most common)
2. Kubernetes (production)
3. Serverless (AWS Lambda, GCP Cloud Functions) - if needed

**Defer**: Extensive deployment docs for all platforms

### Publishing: Start Simple

**Phase 1**: CLI binaries only (GitHub releases)
- Build for Linux, macOS, Windows
- Simple, works for everyone

**Phase 2**: Add npm package (if needed)
- Viewer library as npm package
- Only if teams request it

**Defer**: VS Code marketplace, JetBrains marketplace, canary builds

## Timeline: Simplified

| Sprint | Duration | Focus | Deliverable |
|--------|----------|-------|------------|
| **Sprint 1** | Week 1-2 | Foundation | DSL ↔ JSON round-trip |
| **Sprint 2** | Week 2-3 | HTML Export | Shareable HTML diagrams |
| **Sprint 3** | Week 3-5 | Viewer | Interactive diagrams |
| **Sprint 4** | Week 5-8 | Studio | Visual editor |
| **Sprint 5** | Week 9-10 | Changes | Change tracking |
| **Sprint 6** | Week 10-11 | Polish | LSP + Studio polish |
| **Sprint 7** | Week 12-13 | Adoption | Public Studio (optional) |
| **Sprint 8** | Week 13-14 | IDE | VS Code extension (optional) |

**Total**: 11-14 weeks (depending on optional features)

## Success Criteria: Simplified

### Must Have (Phase 1-2)

✅ **Visual Studio works**
- Can create/edit architecture diagrams visually
- Can export to DSL
- Can export to HTML
- Can export as PNG/SVG

✅ **HTML Export works**
- Generates interactive diagrams
- Shareable links work
- All view types render correctly

✅ **Change Tracking works**
- Can create changes
- Can apply changes
- Can visualize diffs

✅ **LSP works**
- Shows errors in IDE
- Basic completion works

### Nice to Have (Phase 3)

⚠️ **Public Studio** (if adoption is slow)
⚠️ **VS Code Extension** (if users request it)

## Decision Framework

**When to add a feature:**

1. **Users request it** → Build it
2. **Blocks adoption** → Build it
3. **Saves significant time** → Build it
4. **Nice to have** → Defer it

**When to defer a feature:**

1. **No user requests** → Don't build it
2. **Workaround exists** → Don't build it
3. **Complex to maintain** → Don't build it
4. **Low usage expected** → Don't build it

## Migration Path

**If we need features later:**

1. **Self-Hosted Studio**: Add preview mode to existing Studio
2. **JetBrains Plugin**: Build if users request (similar to VS Code)
3. **Complex Deployments**: Document as teams request
4. **Marketplace Publishing**: Add when ready to distribute

**Key**: Build incrementally, based on actual need.

## Focus Areas

### Week 1-8: Core Value
**Goal**: Developers can create, edit, and share architecture diagrams easily

### Week 9-11: Polish
**Goal**: Smooth workflow, fewer errors

### Week 12-14: Adoption (Optional)
**Goal**: Lower barrier to entry, IDE integration

## Summary

**Build Now**:
- ✅ Visual Studio (core value)
- ✅ HTML export (easy sharing)
- ✅ Change tracking (architecture evolution)
- ✅ LSP (error diagnostics)

**Build Later** (if needed):
- ⚠️ Public Studio (if adoption slow)
- ⚠️ VS Code Extension (if users request)
- ❌ Everything else (build on demand)

**Result**: Focused, valuable features that solve real problems.

