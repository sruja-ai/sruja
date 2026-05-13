# Rust MSRV and dependency upgrades

This document records how this repository handles **Minimum Supported Rust Version (MSRV)** and **Dependabot** pull requests so majors that raise MSRV do not stall work indefinitely.

## What CI uses

GitHub Actions workflows use the **current stable** Rust channel (`dtolnay/rust-toolchain` pinned by commit hash in workflow files). There is **no** workspace-wide `rust-version` field in `Cargo.toml` today; compatibility is enforced by CI on stable.

## How Dependabot is configured

See [.github/dependabot.yml](../.github/dependabot.yml):

- **Cargo (workspace root):** weekly updates; **minor and patch** updates for workspace crates are often **grouped** (`workspace-deps`). **Major** version bumps are **not** grouped and open as separate PRs.
- **npm** (extension and e2e): separate entries; semver-major updates for `@types/vscode` / `vscode` are ignored by policy.
- **GitHub Actions:** weekly grouped updates.

## When a Dependabot PR raises MSRV

Some dependencies (for example **hashbrown** 0.17, **getrandom** 0.4.x) have documented **MSRV 1.85+** or similar. Treat those PRs as a **policy decision**, not a drive-by merge:

1. **Confirm** current stable in CI is at or above the crate’s MSRV requirement (or bump the toolchain policy explicitly).
2. **Run** `cargo check --workspace` and `make check` (or the relevant subset) on the PR branch.
3. If the project is **not** ready to adopt that MSRV yet, **close** the PR with a short comment linking this document and reopen or recreate when MSRV is raised intentionally.

Prefer **merging grouped patch/minor** PRs (for example Tokio patch releases) quickly to reduce security and bugfix lag.

## Maintainer checklist (Dependabot triage)

- **Patch/minor grouped cargo PR:** run CI, merge if green.
- **Major cargo PR:** read upstream changelog for MSRV and breaking API notes; decide merge vs defer.
- **Grouped Actions PR** (for example checkout v6, Node 24–based actions): verify [GitHub-hosted runner](https://github.com/actions/runner/releases) version requirements in upstream release notes before merge.
