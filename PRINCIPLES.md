# Sruja: Ultra Simple, Highly Functional

## What we are

- **Architecture-as-code tool for the AI SDLC process.** You define architecture in `.sruja` files; we help you validate, document, and keep it in sync with the rest of your workflow (design, review, CI, docs). We are **not** a diagramming product—diagrams are one output, not the product.
- **Ultra simple**: Minimal surface area. No unnecessary apps, UIs, or frameworks.
- **Highly functional**: What we ship works really well for its scope.

## North star

- **Ultra simple**: Minimal surface area. No unnecessary apps, UIs, or frameworks.
- **Highly functional**: What we ship works really well for its scope.

## In scope

| Surface | Purpose |
|--------|---------|
| **Rust (CLI, engine, LSP, WASM)** | Parse, validate, export. Single language for core logic. |
| **VS Code extension** | Edit `.sruja`, diagnostics, and optional WASM-based diagram preview. No web server. |
| **WASM** | Render architecture views from DSL (used by extension and book). Output, not the product. |
| **Docs web app** | Rust-based static generator (e.g. [mdBook](https://rust-lang.github.io/mdBook/)). Markdown in repo → static HTML. No TypeScript/Node for docs. |
| **WASM in docs** | Optional: small JS + WASM in the generated site for diagram preview in code blocks. |

## Out of scope (by design)

- **TypeScript/Node-based** docs site (e.g. Astro, Docusaurus) for the official docs.
- **Web server** for docs (static only; host on GitHub Pages or similar).
- **Designer app** or rich web UI.
- **React / Node front-end** for product-facing UI.

## Courses, tutorials, challenges, quizzes

This content lives in **Markdown** under `book/src/` and is rendered by the **book** (mdBook) as the only docs site.

**With the Rust-based docs (mdBook):**

- **Courses and tutorials** → Migrate into the **book** as chapters/sections. Same Markdown; we add them to `book/src/` and wire them in `SUMMARY.md`. No TypeScript; one docs site.
- **Challenges** → Move into the book as “Exercises” or “Challenges” sections (Markdown + links to `.sruja` examples in the repo). No challenge-runner UI unless we add a minimal static page later.
- **Quizzes** → Either: (a) simple “Check your understanding” sections in the book (no interactive quiz UI), or (b) keep quiz JSON and a tiny standalone HTML+JS page linked from the book. Prefer (a) for ultra simple.
- **Blog** → Optional: add a “Blog” or “News” chapter that links to Markdown files or leave blog on a legacy/optional Astro deploy.

**Source of truth:** Markdown in the repo. The **mdBook** build is the single place that turns that into the official docs site. The Astro website has been removed; the book is deployed to GitHub Pages (staging and production).

## How we decide

Before adding something:

1. **Simple?** Does it add the minimum surface (one way to do the thing)?
2. **Functional?** Does it work reliably and stay maintainable?

If either is no, we simplify or drop it.
