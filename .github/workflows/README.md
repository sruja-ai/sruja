# GitHub Actions

## Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| **unified-ci.yml** | push/PR to main, develop, simplify | Rust: build, test, **format** (`cargo fmt --check`), clippy. **Sruja files**: lint all `**/*.sruja`. Code and .sruja are checked against standards defined in .sruja (e.g. `CodeStyle` → rustfmt, `NoCycles` → sruja lint). |
| **deploy-staging.yml** | push to main (book, crates, examples) / manual | Build mdBook + WASM, deploy to `sruja-ai/staging-website` via `deploy-to-github-pages`. |
| **deploy-production.yml** | manual only | Same build as staging; deploy to `sruja-ai/prod-website` (production, sruja.ai). |
| **skill-validation.yml** | push/PR (skills, skill-lint) | Validate skill files: links, xrefs, code examples, format, schema. |
| **skill-pr-check.yml** | PR (skills) | Validate only changed skill files. |
| **security.yml** | push/PR + weekly Mon | cargo audit, dependency-review (PRs), TruffleHog. |
| **release-please.yml** | push to main | Update CHANGELOG and create release PR from conventional commits. |
| **publish-extension.yml** | push tag v* / manual / workflow_call | Build VS Code extension; publish to Open VSX and (if `AZURE_DEVOPS_PAT` set) Visual Studio Marketplace. |
| **trigger-extension-publish.yml** | release published | Calls publish-extension with release version so extension publishes when a release is created (e.g. by Release Please). |
| **release-cli.yml** | release published, **workflow_call** (from release-please), or **workflow_dispatch** (manual) | Build Sruja CLI for Linux (x86_64), macOS (aarch64), and Windows (x86_64); attach binaries to the GitHub Release. **release-please** calls this via `workflow_call` when it creates a release (GITHUB_TOKEN cannot trigger `workflow_dispatch`). Install script is served at https://sruja.ai/install.sh; users run `curl -fsSL https://sruja.ai/install.sh | bash`. |

## Standards defined in .sruja → CI checks

Policies and requirements in the repo’s .sruja files (e.g. `docs/architecture/sruja-platform.sruja`) are enforced in CI as follows:

| Standard (in .sruja) | CI check |
|----------------------|----------|
| **CodeStyle** – "Rust code must be formatted with rustfmt" | `cargo fmt -- --check` (Rust job) |
| **NoCycles** – "Architecture graph must be acyclic" | `sruja lint` (Sruja files job; cycle rule) |
| Other structural/semantic rules (refs, orphans, governance) | `sruja lint` on all `**/*.sruja` |

To add a new code-level standard: (1) add or reference the policy in the architecture .sruja file, (2) add the corresponding CI step (e.g. a new tool or script) to `unified-ci.yml`.

## Composite actions

| Action | Purpose |
|--------|---------|
| **build-wasm** | `make wasm` (Rust WASM build). |
| **sruja-validate** | Rust toolchain, build sruja-cli, lint .sruja files (glob configurable), optional markdown export. |
| **deploy-to-github-pages** | Checkout target repo, copy site contents, push. |
| **setup-gpg** | GPG for signing. |

## Deploy to Staging

The **deploy-staging** workflow builds the mdBook site (book + WASM) and deploys to `sruja-ai/staging-website` on push to `main`.

**Prerequisites**

1. GitHub App with write access to `sruja-ai/staging-website`.
2. Repo secrets: `SRUJA_WEBSITE_DEPLOY_APP_ID`, `SRUJA_WEBSITE_DEPLOY_APP_PRIVATE_KEY` (likely already configured from prior deployment setup).

**Triggers**

- **Push to main:** When paths under `book/`, `crates/`, `book/valid-examples/` change.
- **Manual:** **Actions → Deploy to Staging → Run workflow.**

## Deploy to Production

The **deploy-production** workflow builds the same mdBook site (book + WASM) and deploys to `sruja-ai/prod-website` (production at https://sruja.ai). It **always checks out and builds from `main`**, so production is a promote of what’s already on staging.

**Prerequisites**

1. GitHub App (credentials in `SRUJA_WEBSITE_DEPLOY_APP_ID` / `SRUJA_WEBSITE_DEPLOY_APP_PRIVATE_KEY`) must be installed with access to **staging-website** and **prod-website**.
2. Target repo `sruja-ai/prod-website` must exist; CNAME for sruja.ai if using custom domain.

**Triggers**

- **Manual only:** **Actions → Deploy to Production → Run workflow.** Validates on staging first, then run this to promote **main** to production.

## Publishing the VS Code extension (Open VSX)

The **publish-extension** workflow builds the extension and publishes to Open VSX and (optionally) the Visual Studio Marketplace. **`extension/package.json`** uses **publisher `SrujaAI`** and **name `sruja`** so that the existing [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja) listing is updated rather than creating a new extension.

**Prerequisites**

1. Create an [Eclipse Foundation account](https://accounts.eclipse.org) (GitHub username should match).
2. Sign the [Publisher Agreement](https://open-vsx.org) (log in with GitHub, then link Eclipse account in profile).
3. In [open-vsx.org](https://open-vsx.org) → profile → **Access Tokens**, create a Personal Access Token.
4. Ensure org (or repo) secret **`OPEN_VSX_TOKEN`** exists with that token.
5. For **Visual Studio Marketplace** (marketplace.visualstudio.com): Create an [Azure DevOps PAT](https://dev.azure.com) with **Marketplace → Manage** scope and set org/repo secret **`AZURE_DEVOPS_PAT`**. If unset, the workflow skips Marketplace and only publishes to Open VSX.

**Triggers**

- **Release published (e.g. Release Please):** **Trigger extension publish on release** runs on `release: published` and calls this workflow with the release tag version. Use this when releases are created by a workflow (tags pushed by `GITHUB_TOKEN` do not trigger other workflows).
- **Manual:** **Actions → Publish extension to Open VSX → Run workflow.** Set the **version** input (e.g. `0.3.5`) or leave empty to use `extension/package.json`.
- **Push tag yourself:** `git tag v0.3.5 && git push origin v0.3.5` — only triggers if the tag push is from your machine (not from another workflow).

**Registries:**
- **VS Code (Visual Studio Marketplace):** [Sruja – SrujaAI.sruja](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja) — updates when this workflow runs and **`AZURE_DEVOPS_PAT`** is set. The PAT must be for the **SrujaAI** publisher so the existing listing is updated (not a new one).
- **Open VSX:** Same extension; may appear as [srujaai/sruja](https://open-vsx.org/extension/srujaai/sruja) (lowercase) due to registry normalization. Used by VS Codium and other editors.

**Recommendation:** Add `extension/package-lock.json` (run `npm install` in `extension/` and commit the lock file) for reproducible builds; the workflow currently uses `npm install` without a lock file.

## CLI release assets

When a release is published, **release-cli.yml** runs. Because releases created by `GITHUB_TOKEN` do not trigger other workflows and `GITHUB_TOKEN` cannot trigger `workflow_dispatch` (403), **release-please.yml** calls both **release-cli.yml** and **publish-extension.yml** via **workflow_call** when it creates a release, so they run in the same Actions run. After Release Please merges a release PR:

1. Builds the Sruja CLI (`cargo build --release -p sruja-cli`) on Linux (x86_64), macOS (aarch64), and Windows (x86_64).
2. Packs each binary into a tarball (`.tar.gz`) or Windows `.zip`.
3. Attaches the binaries to the existing GitHub Release (install script is in the repo, not attached).

Users can install the CLI by running the install script from the repo (it downloads the appropriate binary from the release):

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Or a specific version (tag as shown on [Releases](https://github.com/sruja-ai/sruja/releases), e.g. `sruja-v0.7.7` or `v0.6.1`):

```bash
curl -fsSL https://sruja.ai/install.sh | bash -s -- sruja-v0.7.7
```
