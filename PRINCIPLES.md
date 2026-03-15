# Sruja Principles

These principles guide all decisions about the Sruja project, including feature development, API design, and community governance.

---

## Core Principles

### 1. Architecture as Code

Architecture documentation should live alongside code, be version-controlled, and stay synchronized with the implementation. We believe in treating architecture as a first-class artifact that evolves with the software it describes.

**Implications:**
- `.sruja` files are text files that can be diffed, reviewed, and merged
- AI generates architecture from code, not manually drawn diagrams
- Validation catches errors before they become problems

### 2. AI-First, Human-Readable

Sruja is designed for the AI era. AI agents write and maintain `.sruja` files, but humans can read, review, and understand them.

**Implications:**
- The DSL is simple enough for AI to generate reliably
- Error messages are actionable and helpful for both humans and AI
- The sruja-architecture skill does the heavy lifting

### 3. Minimalism

We do fewer things, but do them well. Sruja focuses on architecture-as-code, not diagramming or visualization.

**Implications:**
- See [SCOPE.md](docs/SCOPE.md) for what's in and out of scope
- We reject features that don't align with core goals
- Export formats (Mermaid, Markdown, JSON) are outputs, not the source of truth

### 4. Developer Experience

Sruja should feel natural to developers. Installation should be one command, validation should be instant, and errors should be helpful.

**Implications:**
- Single-binary CLI with no runtime dependencies
- Fast parser and validator
- Clear error messages with suggestions

### 5. Interoperability

Sruja integrates with existing tools and workflows, not replaces them.

**Implications:**
- Export to standard formats (Mermaid, Markdown, JSON, C4)
- Works in CI/CD pipelines
- LSP support for editors

---

## Design Principles

### Language Design

- **Flat structure**: No deeply nested hierarchies in the DSL
- **Explicit over implicit**: Relationships are declared, not inferred
- **Validation by default**: `sruja lint` catches common mistakes
- **Progressive disclosure**: Simple for beginners, powerful for experts

### API Stability

- **No breaking changes without major version bump**
- **Deprecation path**: Features are deprecated before removal
- **Migration guides**: Provided for all breaking changes

### Community

- **Open by default**: All discussions in public GitHub issues/PRs
- **Inclusive**: Code of Conduct applies to all interactions
- **Transparent**: Roadmap and decisions are documented

---

## Trade-offs

| We prefer | Over |
|-----------|------|
| Text-based architecture | Diagramming tools |
| AI-generated | Manual authoring |
| Validation | Permissiveness |
| Simplicity | Feature completeness |
| Interoperability | Lock-in |

---

## References

- [SCOPE.md](docs/SCOPE.md) – What's in and out of scope
- [GOVERNANCE.md](GOVERNANCE.md) – How decisions are made
- [DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md) – Language design philosophy
