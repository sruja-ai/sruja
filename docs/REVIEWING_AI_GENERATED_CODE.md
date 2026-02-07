# Reviewing AI-Generated Code

AI-generated code ships fast but can introduce subtle bugs and architectural drift. This guide helps reviewers and authors catch issues before merge. CI and the PR template are set up to make this practical.

## How we catch bugs

| Layer | What runs | When |
|-------|-----------|------|
| **Local** | You run `sruja lint`, `cargo clippy`, `make test`, etc. | Before push |
| **CI** | Rust: fmt, clippy, test. TS: lint, test. **Sruja: lint all `**/*.sruja`** when any .sruja changes | Every push/PR |
| **PR template** | Checklist for AI-generated code and .sruja changes | Author fills before request review |
| **Review** | Human checks for intent, architecture, and subtle bugs | Reviewer uses this doc |

## Sruja (.sruja) – what to check

### CI already checks

- **Syntax and references**: `sruja lint` fails on undefined components, circular dependencies, missing required fields, invalid relationship targets.
- **Scope**: When any `.sruja` file changes (including in `docs/`, `lib/`, `examples/`, `test-examples/`), CI lints **every** `.sruja` in the repo so one bad file doesn’t slip in.

### Reviewer checklist

1. **Intent** – Does the model match the described architecture (bounded contexts, data flow)?
2. **Naming** – Component IDs and display names clear and consistent?
3. **Relationships** – Labels specific (e.g. "HTTPS", "publishes events to") rather than vague ("uses", "connects to")?
4. **Orphans / cycles** – Lint catches these; if something looks odd (e.g. a system with no in/out edges), double-check.
5. **Docs** – If the .sruja is the source of truth for a doc, does the change require a doc or diagram update?

### Common AI slip-ups

- Referencing a component before it’s defined.
- Wrong relationship target (e.g. `frontend.web` instead of `frontend`).
- Missing `technology` on containers or `description` on components.
- Copy-paste leaving wrong names or duplicate IDs.

**Quick verify:** `sruja lint path/to/file.sruja` (and rely on CI for full repo).

---

## Rust – what to check

### CI already checks

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

### Reviewer checklist

1. **Error handling** – No `unwrap()` in production paths; use `?`, `Result`, and context (see [AGENTS.md](../AGENTS.md) / Rust guidelines).
2. **Ownership** – Prefer `&str` over `&String`, `&[T]` over `&Vec<T>` where applicable.
3. **API boundaries** – New public types/functions documented and consistent with existing style.
4. **Tests** – Non-trivial logic has unit or integration tests; no silent removal of coverage.

### Common AI slip-ups

- Overuse of `clone()`; ignoring clippy’s borrow suggestions.
- Panics or unwraps in library or error paths.
- Missing `#[must_use]` or docs on new public APIs.

---

## TypeScript / JavaScript – what to check

### CI already checks

- `npm run lint`, `npm run typecheck` (e.g. Astro check for website), `npm run test` (with coverage).

### Reviewer checklist

1. **Types** – No `any` or broad casts without a short comment.
2. **Null/undefined** – Optional chaining and guards where needed; no silent fallbacks that hide bugs.
3. **Async** – Correct use of async/await; no unhandled promise rejections.
4. **Tests** – New behavior covered; no snapshot-only tests for real logic.

### Common AI slip-ups

- Introducing `any` or `as unknown as T` to satisfy the type checker.
- Missing error handling in async code or event handlers.
- Overly large or duplicated components that could be split.

---

## Before you request review (author)

1. Run the relevant checks locally (see PR template): `make test`, `sruja lint` for .sruja, `cargo clippy` for Rust, `npm run lint` for TS.
2. Fill the “AI-generated code” and “Architecture / .sruja review” sections in the PR template.
3. In the PR description, call out any non-obvious design choices or trade-offs so reviewers know what to focus on.

## As a reviewer

1. Use the checklists above for the languages/areas touched.
2. Prefer asking “what happens when X?” over only style nits—especially for AI-generated code.
3. If CI is green but something looks wrong (e.g. architecture inconsistency), request changes and optionally suggest a follow-up (test, doc, or refactor).

---

## References

- **AI editor setup** – [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md)
- **Sruja DSL** – [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md), [.cursorrules](../.cursorrules)
- **Rust** – [AGENTS.md](../AGENTS.md), [CODING_GUIDELINES.md](CODING_GUIDELINES.md)
- **CI** – [.github/workflows/README.md](../.github/workflows/README.md)
