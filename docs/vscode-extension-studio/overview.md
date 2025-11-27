# Sruja Studio Overview

# 📌 Scope
All specs and features are built for the VSCode Extension Studio: two-way DSL↔diagram editing via IR patches, ELK-based layouting in the Webview, and tight LSP/Kernel integration.

# ⭐ Short Answer
Do NOT build a standalone GUI editor.
Build a “Studio inside VSCode” with a webview + LSP + Sruja Kernel + Notebook cells.
Add a web-based standalone mode later as a thin layer over the same APIs.

# 🧠 Why NOT build your own standalone Studio (at least not now)
Standalone studios require heavy investment and pull users out of their coding environment.
Developers want Sruja inside VSCode, Cursor, PRs, notebooks, and terminal flows.

# ⭐ The Correct Strategy
## Sruja Studio INSIDE VSCode (and Cursor)
- Webview-based Architecture Studio UI: diagrams, inspector, AI, two-way binding, notebook cells
- LSP for syntax, completions, hover docs, validation, diagnostics, goto definition, refactor, rename, symbols
- Sruja Kernel (Go/WASM) for parsing DSL, generating IR, diffing, policy evaluation, AI orchestration, layout
- Two-Way Binding Engine: editing DSL updates diagrams; editing diagrams updates DSL via IR patches

# 🧩 Studio Architecture (in VSCode)
```
            ┌──────────────────────────┐
            │       VSCode LSP         │
            │  (sruja-language-server) │
            └──────────▲──────────────┘
                       │
         text edits    │   diagnostics, completions
                       │
┌──────────────────────┴───────────────────────────┐
│                VSCode Extension                  │
│  - Webview panel (React)                          │
│  - Commands                                        │
│  - Kernel process / WASM                           │
│  - AI agent bridge                                 │
└──────────────────────▲───────────────────────────┘
                       │
       IR updates      │      UI actions
                       │
            ┌──────────┴─────────┐
            │     Sruja Kernel   │
            │  (Go→WASM runtime) │
            └──────────▲─────────┘
                       │
               IR / DSL / DIFF / Graph
```

# 🟦 Core Components
- VSCode Extension: registers DSL, starts LSP, hosts Webview, bridges IR updates and patches
- Sruja Language Server: parse DSL, produce IR, diagnostics, completions, diffs
- Sruja Kernel: compile notebooks, import brownfield, diff, policy engine, layout, DSL patches
- Webview Sruja Studio: React + Vite + Cytoscape/ELK; diagrams, flows, domain maps, diff visualizer, AI sidebar

# 🟧 Two-Way Binding Details (Overview)
- Text → Diagram: LSP → Kernel → Studio
- Diagram → Text: Studio → IR patch → Kernel → DSL patch → LSP → File
- Uses AST + source maps for minimal edits

# 🟩 Why this approach is PERFECT
- Works for large teams, no context switching
- Leverages editor ecosystem; supports multi-file architecture
- DSL, IR, Notebook, Diagrams together; AI integration first-class
- Git diff & PR flows remain natural; maintainable long-term

# 🟦 Recommended Implementation Order
- Phase 1 – VSCode Extension: syntax highlighting, basic LSP, read-only webview, DSL→diagram binding
- Phase 2 – Two-way Binding: Diagram→IR patches, IR→DSL patches, notebook integration
- Phase 3 – AI Integration: brownfield inference, refinement, code alignment, queries
- Phase 4 – Web Studio (optional): stakeholder client for diagrams and approvals

# 🟣 Final Recommendation
Build Sruja Studio inside VSCode/Cursor using Webview + LSP + Kernel.
Implement full two-way binding via IR patches + AST rewriting.
Only build a standalone Studio once the VSCode experience stabilizes.
