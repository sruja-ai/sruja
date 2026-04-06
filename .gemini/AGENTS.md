# Gemini Agent Instructions

You are assisting with the Sruja project—an architecture-as-code and context engineering platform.

## Global Rules
You MUST read and strictly adhere to the instructions located in `AGENTS.md` at the project root. This is the single source of truth for:
- Build, test, and lint commands
- Coding style (Rust and TypeScript)
- Sruja DSL syntax and validation
- Architecture patterns

## Key Responsibilities
1. **Maintain Architectural Integrity**: Always validate `.sruja` changes with `sruja lint`.
2. **Context Awareness**: Use `sruja context` or `sruja mcp` if available to understand cross-repo dependencies.
3. **Dogfooding**: Use Sruja's own tools (`doctor`, `daily`, `drift`) when working on this repo.

Refer to `AGENTS.md` for all technical details.
