# AI Integration

# 📌 Scope
AI actions integrated in the VSCode Extension Studio Webview; produce IR or DSL patches that participate in two-way editing and respect ELK layouting.

# 🤖 AI Action Engine (in Studio)
Capabilities:
- Propose boundary refactoring
- Detect duplicate entities
- Clean naming conventions
- Merge/split systems
- Infer missing components
- Propose policy rules
- Explain node purpose
- Simulate architecture change
- Autocomplete architecture updates

Architecture:
```
AIActionPanel → AICommand (DSL-level or IR-level) → Kernel IR changes or DSL patches → Editor applies
```
Large structural changes preview before apply.

# 🟦 Recommended Implementation Order (AI Phase)
- Phase 3 – AI Integration: brownfield inference, refinement suggestions, code alignment, architecture queries

# 🟣 Summary
AI is first-class in Studio: embedded actions, deterministic patch generation, diff/preview UX, integrates with binding pipeline.
