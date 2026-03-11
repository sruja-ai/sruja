# First PR with Sruja (10 minutes)

This walkthrough is the shortest path to **seeing Sruja’s value in a real PR**:

- A validated `.sruja` blueprint checked in to Git
- A GitHub Actions PR gate that runs `sruja lint` and `sruja drift-pr`
- A repeatable workflow your team can standardize on

## Prereqs

- A GitHub repo with a default branch (`main` or `master`)
- You can run the Sruja CLI locally (see [Using Sruja in Your Project](USING_SRUJA_IN_YOUR_PROJECT.md#1-install-your-machine-andor-ci))

## 1) Add the PR gate workflow

Copy the workflow template into your repo:

```bash
mkdir -p .github/workflows
cp templates/github-actions/sruja-architecture-pr.yml .github/workflows/sruja-architecture-pr.yml
```

What it does on every PR:

- `sruja drift-pr` to catch **new** structural issues introduced by the PR
- `sruja lint` for any blueprint files under `architecture/**/*.sruja` or `docs/architecture/**/*.sruja`

## 2) Create your first blueprint (minimal but useful)

Create `docs/architecture/architecture.sruja`:

```bash
mkdir -p docs/architecture
$EDITOR docs/architecture/architecture.sruja
```

If you want a starter to copy/paste, use one of:

- `templates/blueprints/simple-web-service.sruja`
- `templates/blueprints/event-driven-saas.sruja`

## 3) Validate locally (fast feedback loop)

```bash
sruja lint docs/architecture/architecture.sruja
```

If you’re editing in VS Code / Cursor with the Sruja extension, you can also use in-editor validation and diagnostics.

## 4) Open a PR and let CI prove the value

Commit your changes on a branch and open a PR.

In CI you should see:

- **PR-scoped drift** results (new violations only)
- **Blueprint lint** results (your `.sruja` passes)

This is the “aha”: Sruja becomes a **repeatable gate** that prevents architecture regressions and keeps your blueprint valid.

## 5) Optional: attach a diagram to the PR

Export Mermaid (easy to paste into Markdown):

```bash
sruja export mermaid docs/architecture/architecture.sruja > docs/architecture/architecture.mmd
```

Or export Markdown:

```bash
sruja export markdown docs/architecture/architecture.sruja > docs/architecture/architecture.md
```

## FAQ

### Do I need a `.sruja` file to get value?

No. You can start with `sruja quickstart -r .` and `sruja drift -r .` for structural issues without any blueprint. The PR workflow becomes more powerful when you add an explicit baseline.

### What should we model first?

Start with:

- 1–3 systems
- a few deployable containers (API, web, worker)
- primary datastores
- the most important relationships (protocol + intent)

Keep it minimal; expand as the team learns what’s useful in review.

