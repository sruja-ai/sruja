# Structurizr / LikeC4 vs Sruja

| | Structurizr / LikeC4 | Sruja |
|---|---------------------|-------|
| **Primary user** | Humans documenting architecture | Agents and CI enforcing topology from code |
| **Source of truth** | Authoritative DSL/workspace views | Scan-derived graph; optional `repo.sruja` overlay |
| **Day-one value** | Draw and share C4 views | `sruja drift -r . --structural-only` — violations with file paths |
| **Editor story** | Diagrams and workspace UX | MCP `coding` profile + `focus` / `verify-task` |
| **Exports** | Core product | Tier 2 — Mermaid/Markdown from snapshot |

**One line:** Structurizr documents architecture for humans; Sruja extracts topology from code and gates agents (MCP + CI) before and after generation.

**Not the same as SonarQube:** Sruja reports **structural** topology (cycles, layers, god modules), not style or security rule packs.

**Honest limits:** See [KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md) — dynamic imports, reflection/DI, heuristic layers, orphan false positives on greenfield (use `drift --advisory`).
