# Related GitHub repositories

The [sruja-ai](https://github.com/sruja-ai) organization on GitHub hosts the product, generated documentation sites, and (optionally) additional private or downstream repositories used in a composed `system.index.json`. This page summarizes the **public** repos most contributors interact with.

## Public repositories

- **[sruja-ai/sruja](https://github.com/sruja-ai/sruja)** — Source of truth: Rust workspace, VS Code extension, mdBook sources (`book/`), CLI, WASM, skills, and CI. Issues and [Discussions](https://github.com/sruja-ai/sruja/discussions) live here.
- **[sruja-ai/staging-website](https://github.com/sruja-ai/staging-website)** — GitHub Pages deploy target for **staging** documentation built from `main` (path-filtered pushes and manual `workflow_dispatch`). Issues may be disabled; use the main repo for bug reports.
- **[sruja-ai/prod-website](https://github.com/sruja-ai/prod-website)** — GitHub Pages deploy target for **production** documentation. Updated only via a **manual** promote workflow after staging is validated.

Editor or tooling context that mentions **four** composed repositories usually includes a bundle or repo that is **not** listed publicly under `sruja-ai` (private repo, customer fork, or unpublished index). Federation behavior is documented in [FEDERATION.md](FEDERATION.md) and [FEDERATION_SETUP_GUIDE.md](FEDERATION_SETUP_GUIDE.md).

## How documentation is deployed

Workflows in **this** repository (`sruja-ai/sruja`) build the mdBook site (including WASM for diagrams), then push static output to the Pages repos using a GitHub App:

- **Staging** — [.github/workflows/deploy-staging.yml](../.github/workflows/deploy-staging.yml): push to `main` when changed paths include `book/`, selected crates, `scripts/`, or the workflow itself; also `workflow_dispatch`.
- **Production** — [.github/workflows/deploy-production.yml](../.github/workflows/deploy-production.yml): `workflow_dispatch` only (promote after validating staging).

Secrets (names only): `SRUJA_WEBSITE_DEPLOY_APP_ID`, `SRUJA_WEBSITE_DEPLOY_APP_PRIVATE_KEY`. The reusable composite action lives under [.github/actions/deploy-to-github-pages/](../.github/actions/deploy-to-github-pages/).

Each deploy refreshes the target repo’s root (preserving `CNAME` when present), adds `.nojekyll`, copies **MIT OR Apache-2.0** license files from this repo, and writes a short README pointing contributors back here.

## Where to file issues

Use **[sruja-ai/sruja/issues](https://github.com/sruja-ai/sruja/issues)** (or Discussions) for documentation site bugs, CLI, extension, and book content—even if you first noticed the problem on the staging or production URL. That keeps triage and fixes next to the source and deploy definitions.
