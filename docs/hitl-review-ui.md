# Human-in-the-Loop Review UI (Specification)

Goals
- AI proposes architecture; humans review/approve/reject/refine.
- Incremental refinement, visual + textual diffs, explanations + confidence.
- Notebook cells, PR-style change panel, interactive diagrams; fully diff-aware.

UI Modes
- Notebook Review Mode: cell-based UX with architecture cells, AI suggestion cells, diff/explain, diagrams, accept/reject, replay.
- Review Panel Mode (PR-style): change list with accept/reject/edit/ask‑AI/refine; headers show counts, policy violations, drift.
- Visual Diagram Review Mode: interactive C4/domain/event flows; pending changes highlighted; node controls; grouping proposals; diagram diff mode.

Core Concepts
- PendingChanges staging area with id/type/target/origin/confidence/explanation; users stage/unstage/apply/reject/edit.
- Confidence heatmap for quick risk assessment; trace explanation viewer with sources + reason.
- Dual View: IR JSON + DSL; editing syncs both; inline natural language editing with AI proposals and diff preview.

Notebook Spec
- Cell types: `%%sruja.show.systems`, `%%sruja.suggest.refine <Element>`, `%%sruja.import.summary`, `%%sruja.diff old=baseline new=current`, `%%sruja.apply`.
- Outputs: diagrams, tables, JSON previews, NL summaries, interactive diff widgets.

PR-style Panel
- Header: inferred changes, low-confidence items, policy violations, drift.
- Change list examples: added/removed/updated; actions: Accept, Reject, Ask AI, Refine.
- Detail panel: origin, explanation, confidence, related policies, affected flows, diagrams, history.

Diagram Review
- Node controls: details, inferred attrs, relationships, AI trail, suggestions.
- Suggested groupings: merge/split/ignore; diagram diff (Before → After) highlights.

Human Override
- Confirm, reject permanently (`.sruja/ignore.json`), lock boundaries, annotate, request alternatives.

Collaboration
- Inline comments on entities/boundaries/event schemas/policy violations.
- Approval workflows for architecture changes; ADRs in context.

Kernel Integration
- UI drives kernel: `sruja.kernel.apply_diff()`, `generate_ir()`, `visualize()`, `validate()`; maintains history/undo/snapshots/policy violations/IR correctness.

Cursor AI Integration
- Inline corrections, explanations, refactoring suggestions, “why?” queries; batch updates.
- UI ensures structured, validated architecture with deterministic diffs and version history.

Summary
- Smooth brownfield adoption; trust in AI outputs; human control; visual + textual review; versioned refinements; diagrams; notebook + PR workflows; collaboration + governance; AI integration.
