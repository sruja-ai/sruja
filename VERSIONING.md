# Versioning

Sruja follows [Semantic Versioning 2.0.0](https://semver.org/) with the guidelines below.

---

## Version Format

Versions follow the format `MAJOR.MINOR.PATCH`:

- **MAJOR**: Breaking changes
- **MINOR**: New features, backwards-compatible
- **PATCH**: Bug fixes, backwards-compatible

Examples: `0.17.2`, `1.0.0`, `2.1.3`

---

## Pre-1.0 Behavior

While Sruja is in `0.x.x`, we reserve the right to make breaking changes in **minor** releases. However, we strive to:

1. Minimize breaking changes
2. Provide migration guides when needed
3. Deprecate features before removing them

Once we reach `1.0.0`, full semver semantics will apply.

---

## Release Channels

| Channel | Version Pattern | Stability |
|---------|-----------------|-----------|
| Stable | `0.17.0`, `1.0.0` | Production-ready |
| Pre-release | `0.18.0-alpha.1`, `1.0.0-rc.2` | Testing only |

### Pre-release Identifiers

- `alpha`: Early testing, may have known issues
- `beta`: Feature complete, needs more testing
- `rc`: Release candidate, final testing before stable

---

## Breaking Changes

A change is **breaking** if it:

1. Removes a public API (CLI flag, exported function, config option)
2. Changes the behavior of a public API in an incompatible way
3. Requires changes to user code or configuration

### Deprecation Policy

1. **Announce**: Deprecation is announced in a release and documented
2. **Warn**: Users see warnings when using deprecated features
3. **Remove**: After at least one minor release with deprecation, feature may be removed in the next major (or minor, pre-1.0)

---

## Release Cadence

| Type | Frequency | Example |
|------|-----------|---------|
| Patch | As needed | Bug fixes, typo corrections |
| Minor | ~Monthly | New features, enhancements |
| Major | Rare | Breaking changes, significant rewrites |

---

## Changelog

All releases are documented in [CHANGELOG.md](CHANGELOG.md) following [Keep a Changelog](https://keepachangelog.com/) format.

---

## Release Artifacts

Each release includes:

- **CLI binaries**: Linux (x64, ARM), macOS (x64, ARM), Windows (x64)
- **VS Code extension**: Published to Marketplace and Open VSX
- **WASM package**: Published to npm
- **Source code**: Tagged git commits

---

## Version Consistency

All components in the Sruja workspace share the same version number:

- `sruja-cli`
- `sruja-language`
- `sruja-wasm`
- `sruja-lsp`
- VS Code extension

This is verified by CI on every pull request.
