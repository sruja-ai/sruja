# Documentation Consistency Report

This report summarizes consistency checks across Sruja documentation (README, `docs/`, book, AGENTS.md, .cursorrules, skills) and the fixes applied.

## Scope

- **Terminology**: `database` vs `datastore`, flat syntax, define-before-use
- **CLI**: `sruja lint`, `sruja compile`, command examples
- **Links**: Language spec, examples, cross-references between book and `docs/`
- **Single source of truth**: Canonical examples in `book/valid-examples/`, language spec in `docs/LANGUAGE_SPECIFICATION.md` and book `reference/language-spec.md`

---

## Fixes Applied

### 1. Broken link: Language spec (book)

- **File**: `book/src/getting-started.md`
- **Issue**: Link `docs/reference/language-spec.md` is wrong from book context (would resolve to `book/src/docs/reference/`, which does not exist; the spec lives at `book/src/reference/language-spec.md`).
- **Fix**: Updated to `reference/language-spec.md`.

### 2. Broken / misleading examples URL

- **Files**: `book/src/docs/using-sruja-in-your-project.md`, `docs/USING_SRUJA_IN_YOUR_PROJECT.md`
- **Issue**: Pointed to `https://github.com/sruja-ai/sruja/tree/main/examples`, but there is no top-level `examples/` directory; canonical examples are in `book/valid-examples/` and the book Examples Gallery.
- **Fix**: Book now links to the [Examples Gallery](../examples/index.md). Root `docs/` version now references `book/valid-examples/` and the built book Examples Gallery.

### 3. Terminology: `datastore` → `database`

- **Standard** (AGENTS.md, .cursorrules, glossary): Prefer `database` for data stores; `datastore` is an alias.
- **Files updated**:
  - `book/src/challenges/fix-relations.md`: `datastore` kind and usages → `database` (initialDsl and solution).
  - `book/src/tutorials/basic/systems-thinking.md`: All `datastore` usages → `database`.

---

## Verified Consistent

- **Flat syntax**: Described consistently as “flat, top-level declarations; no wrapper” in LANGUAGE_SPECIFICATION.md, book reference, AGENTS.md, .cursorrules, and concept docs.
- **`sruja lint`**: Documented with and without explicit file (e.g. `sruja lint repo.sruja` or `sruja lint [file]`); all usages are valid.
- **Language spec duality**: `docs/LANGUAGE_SPECIFICATION.md` is the repo source; `book/src/reference/language-spec.md` is the book copy; both are kept in sync and linked appropriately from `docs/` vs book.
- **Glossary**: States “`database` is the recommended term” and “`datastore` as an alias”; aligned with above changes.

---

## Recommendations (no code changes)

1. **`sruja compile`**: The CLI still exposes a `Compile` subcommand. CONTRIBUTING.md and `book/src/tutorials/basic/cli-basics.md` mention “compile and lint.” If `compile` is deprecated, consider documenting that and steering users to `lint` (and optionally removing the command in a future release).
2. **Policy lesson wording**: `book/src/courses/advanced-architects/module-1-policy-as-code/lesson-1.md` uses the word “datastores” in rule text (e.g. “containers in layer 'presentation' must not have relations to datastores”). This is conceptual and does not conflict with using the `database` kind in DSL examples; no change required unless you want rule text to say “databases” everywhere.
3. **Link checks**: Add a CI or pre-commit step to validate internal links (e.g. mdBook link check, or a link checker over `docs/` and `book/src/`) to catch broken paths and wrong URLs early.

---

## Summary

| Area           | Status | Notes                                                |
|----------------|--------|------------------------------------------------------|
| Language spec  | Fixed  | Book link corrected                                 |
| Examples URL   | Fixed  | Points to book/valid-examples and Examples Gallery  |
| database term  | Fixed  | fix-relations and systems-thinking use `database`   |
| Flat syntax    | OK     | Consistent across specs and guides                   |
| sruja lint     | OK     | Usage is consistent                                  |
| sruja compile  | Note   | Still in CLI; clarify if deprecated                 |
