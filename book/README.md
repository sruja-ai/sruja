# Sruja docs (mdBook)

Rust-based docs. No TypeScript or Node.

## Prerequisites

- [mdBook](https://rust-lang.github.io/mdBook/): `cargo install mdbook`
- **Mermaid + Execute buttons:** `cargo install mdbook-mermaid`, then from the `book/` directory run `mdbook-mermaid install .` so `mermaid.min.js` and `mermaid-init.js` are copied into the book.

**From repo root you can use `just` (recommended) or the Makefile shim:**

```bash
just book-deps    # or: make book-deps
just book         # or: make book
just book-serve   # or: make book-serve
just book-clean   # or: make book-clean
```

## Build

```bash
# From repo root (recommended):
just book         # builds book and copies WASM (run 'just wasm' once if you need Sruja diagrams)

# Or manually from book directory:
mdbook build && ./copy-wasm.sh
```

`just book` (or `make book`) runs `book-build` (mdbook build) then `book-wasm` (copy-wasm.sh). WASM is built with `just wasm` (or `make wasm`) (output: `book/wasm/rust/`); run it once if you want "Show diagram" for ` ```sruja ` blocks.

If the book is deployed under a different base path, set `window.SRUJA_WASM_BASE` before the script runs (e.g. in a custom `head.hbs`).

Output is in `book/book/`.

## Serve locally (live reload)

Use the wrapper so WASM is copied into the output and "Show diagram" works:

```bash
just book-serve   # from repo root (runs build + copy-wasm + serve) (or: make book-serve)
# or from book/:  ./serve.sh
```

Then open http://localhost:3000. **Note:** mdbook's render step wipes the build dir, so `serve.sh` runs `mdbook serve` in the background and re-copies WASM into the output every second so "Show diagram" keeps working after live reloads. If you run `mdbook serve` directly (without `serve.sh`), run `book/copy-wasm.sh` after the first build so `book/book/wasm/rust/` exists.

## Google Analytics

The book includes a GA4 script (`google-analytics.js`). To enable tracking:

- **Option A:** Replace the placeholder in `book/google-analytics.js`: change `'G-XXXXXXXXXX'` to your GA4 Measurement ID (e.g. `G-ABC123XYZ`).
- **Option B:** Before this script runs, set `window.MDBOOK_GA_MEASUREMENT_ID` to your ID (e.g. via a custom theme or build step). If set, the placeholder is ignored.

If the ID is unset or still the placeholder, no tracking runs. The script uses `anonymize_ip: true` for privacy.

## Deploy

Upload the contents of `book/` to GitHub Pages or any static host.

## Migration status

**Done:**
- Courses, tutorials, and challenges copied into `book/src/` and wired in `SUMMARY.md`.
- Docs (intro, getting-started, how-sruja-works, examples, beginner-path, concepts/, reference/, glossary, style-guide, community) copied into `book/src/docs/` and added to SUMMARY.
- Internal links updated: `/docs/`, `/courses/`, `/tutorials/` → relative paths with `.md` where needed.
- Getting started "From Source" documents building from Rust source.
- Cheatsheet added as plain Markdown at `docs/reference/cheatsheet.md`.
- Concepts section expanded in TOC (container, component, person, relations, deployment, requirements, scenario, ADR, policy).
- Adoption guide and adoption playbook copied and added to SUMMARY.
- `tutorials/topics.md` not in TOC (uses Astro shortcodes); community links to first tutorial instead.

**If you need to re-run or add more:**

1. Copy or move the Markdown into `book/src/` (e.g. `book/src/courses/`, `book/src/tutorials/`).
2. Strip or adapt Astro frontmatter (e.g. `weight`, `summary`) if needed; mdBook uses `SUMMARY.md` for order.
3. Add entries to `book/src/SUMMARY.md` so they appear in the sidebar.
4. Fix any internal links (e.g. `/courses/...` → relative paths within the book).

Challenges and quizzes can become “Exercises” sections (Markdown only) or stay as external links. See **PRINCIPLES.md** (repo root) for the overall plan.
