# Environment-Specific Configuration

This document outlines staging and production deployment and any environment-specific behavior.

## Current deployment (mdBook only)

The **website** is the **mdBook** build from this repo (`book/`). There is no separate Node/React website app in this repository.

| Environment | Repo | Trigger | URL |
|-------------|------|---------|-----|
| **Staging** | `sruja-ai/staging-website` | Push to `main` (book/crates/valid-examples) or manual | https://staging.sruja.ai |
| **Production** | `sruja-ai/prod-website` | Manual only (promote from `main`) | https://sruja.ai |

- Both workflows build the same artifact: mdBook + WASM + install script (see [.github/workflows/README.md](workflows/README.md#deploy-to-staging)).
- Production is a **promote of what’s already on staging** (always builds from `main`).
- Secrets: `SRUJA_WEBSITE_DEPLOY_APP_ID`, `SRUJA_WEBSITE_DEPLOY_APP_PRIVATE_KEY` (GitHub App with write access to both staging-website and prod-website).

No `apps/website`, `packages/shared`, or Node-based site exists in this repo; the public site is static mdBook + WASM.

---

## Legacy / future: app-style environments

The sections below describe environment-specific concerns for a **hypothetical or legacy** Node/React-style app (e.g. PostHog, Algolia, console logging). They do **not** apply to the current mdBook-only deploy. If we add a separate web app later, this can be updated.

### PostHog / Algolia (if applicable)

- **PostHog**: If used, tag events with `environment: "staging"` or `"production"`.
- **Algolia**: If used, use separate indices per environment (e.g. `sruja_docs_staging` vs `sruja_docs`).

### Console logging (if applicable)

- Production: only errors/warnings.
- Staging: all logs for debugging.

### Build configuration (if applicable)

- `PUBLIC_ENV` or equivalent set in workflows; `NODE_ENV=production` for built assets if building a Node app.

---

## Summary

- **Current**: Staging and production are mdBook static sites; deploy is via GitHub Actions to `staging-website` and `prod-website`. No app-specific env (PostHog, Algolia, etc.) in this repo.
- **If you add a web app**: Use the “Legacy / future” section above and point to actual paths (e.g. `apps/website`, `packages/shared`) once they exist.
