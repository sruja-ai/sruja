# Search & Command Palette Spec

# 📌 Scope
VSCode Extension Studio feature: keyboard-first modeling and navigation inside the Webview, integrated with two-way binding (DSL↔diagram) and ELK layouting.

# ⭐ Purpose
Enable fast navigation, lookup, commands, AI actions, filtered views, jump-to-DSL/diagram, modeling actions; keyboard-driven workflow.

# 🧱 Global Trigger Rules
Open via `Ctrl+Shift+P` (commands), `Ctrl+K` (search), toolbar, diagram right-click, inspector field. Modes: `>` command, default node search, `?` AI query, `:` filter, `→` relation finder.

# ⭐ Query Modes & Syntax
- Default Node Search: fuzzy across systems, containers, components, entities, events, contexts, relations
- Command Mode: `>add container`, `>delete node`, `>refactor boundary`, etc.
- AI Query Mode: `?what depends on BillingAPI`, `?find circular dependencies`
- Filter Mode: `:events`, `:domains`, `:violations`, `:inferred`
- Relation Finder: `→BillingAPI`

# 🧠 Search Ranking (Fuzzy + Semantic)
Rank by exact ID/label, fuzzy, semantic (AI), type priority, usage frequency, graph proximity.

# 🟦 Search Result Types
`SearchResult { id, label, type, action, location, score, metadata }` with icons and badges.

# 🟥 Command Palette Specification
Declarative commands; categories (Architecture, View, AI, Layout, Edit, Navigate, Undo/Redo).

# 🟩 User Flow Examples
- Search → Navigate centers and focuses inspector
- Command → Action prompts for parent/name, emits IR patch, DSL+diagram update
- AI Query → Insight computes and zooms
- Filter Mode hides nodes accordingly
- Relation Finder highlights edges

# 🟨 Palette Interaction Model
Keyboard: Enter, Esc, arrows, Tab, Ctrl+Space. Micro interactions: hover preview, select focuses inspector, Shift opens in DSL.

# 🟧 Architecture Integration
Palette → EventBus → SearchEngine → PatchRouter → Kernel → LSP → DSL → Diagram.

# 🟥 Performance Optimization
Fuse.js for fuzzy; IR indexes; worker AI search; cache results; prefetch common commands.

# 🧠 Advanced Features
- Command Templates (NL → command)
- Global Replace (rename across IR + DSL)
- Multi-Select Operations
- Smart Relation Editing (NL → connect patches)

# ⭐ Final Summary
Fast, fuzzy, semantic search; commands, filters, navigation, AI queries; modeling actions; two-way binding; natural-language modeling; view filters; contextual previews; keyboard productivity.
