# GitHub Actions

## Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| **unified-ci.yml** | push/PR to main, develop, simplify | Rust: build, test, **format** (`cargo fmt --check`), clippy. **Sruja files**: lint all `**/*.sruja`. Code and .sruja are checked against standards defined in .sruja (e.g. `CodeStyle` → rustfmt, `NoCycles` → sruja lint). |
| **security.yml** | push/PR + weekly Mon | cargo audit, dependency-review (PRs), TruffleHog. |
| **release-please.yml** | push to main | Update CHANGELOG and create release PR from conventional commits. |
| **hn-review.yml** | on release / manual | Prepare HN post. |
| **publish-extension.yml** | release published / manual | Build VS Code extension and publish to **Open VSX Registry** (open-vsx.org). |

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

## Publishing the VS Code extension (Open VSX)

The **publish-extension** workflow builds the extension and publishes it to the [Open VSX Registry](https://open-vsx.org) (used by VS Codium and other editors).

**Prerequisites**

1. Create an [Eclipse Foundation account](https://accounts.eclipse.org) (GitHub username should match).
2. Sign the [Publisher Agreement](https://open-vsx.org) (log in with GitHub, then link Eclipse account in profile).
3. In [open-vsx.org](https://open-vsx.org) → profile → **Access Tokens**, create a Personal Access Token.
4. Ensure org (or repo) secret **`OPEN_VSX_TOKEN`** exists with that token. (Org already has this secret; no change needed if it’s available to this repo.)

**Triggers**

- **Release published:** Creating a release (e.g. from a release PR) runs the workflow and publishes the extension. The extension version is set from the release tag (e.g. `v0.2.0` → `0.2.0`).
- **Manual:** **Actions → Publish extension to Open VSX → Run workflow.** Uses the version in `extension/package.json`.

**Recommendation:** Add `extension/package-lock.json` (run `npm install` in `extension/` and commit the lock file) for reproducible builds; the workflow currently uses `npm install` without a lock file.
