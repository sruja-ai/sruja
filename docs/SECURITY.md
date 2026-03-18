# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Scanning

Sruja uses automated security scanning in CI:

### JavaScript/TypeScript Security

- **npm audit**: Dependency vulnerability scanning
- Runs on: Push, PR, weekly schedule
- Location: `.github/workflows/security.yml`

#### Known audit findings (9: 6 low, 3 moderate)

- **Elliptic (6 low)** – Transitive via `vite-plugin-node-polyfills` → `node-stdlib-browser` → `crypto-browserify`. We **override** `elliptic` to **6.6.1** (patched) in root `package.json`, so the installed version is safe. npm audit still reports the chain by declaration.
- **ESLint &lt;9.26.0 (3 moderate)** – In **unimported** (dev-only, optional: `check:unused:files`). Unimported pins an older `@typescript-eslint/parser` that depends on eslint 8. No upstream fix without replacing unimported; impact is limited to dev tooling (stack overflow when serializing circular refs). Acceptable for optional dev dependency.

### Dependency Review

- **Dependency Review Action**: Reviews dependency changes in PRs
- Runs on: Pull requests only
- Blocks PRs with moderate+ severity vulnerabilities

### Secret Scanning

- **TruffleHog**: Scans for accidentally committed secrets
- Runs on: Push, PR
- Prevents secrets from being committed

## Reporting a Vulnerability

If you discover a security vulnerability, please **DO NOT** open a public issue.

Instead, please email security@sruja.ai with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and work with you to resolve the issue.

## Security Best Practices

### For Contributors

1. **Never commit secrets**: API keys, passwords, tokens, etc.
2. **Validate inputs**: Validate CLI arguments (Clap) and DSL inputs (parser + validator). Treat all user-provided strings as untrusted.
3. **Sanitize file paths**: Resolve and validate paths before reading/writing; avoid path traversal by constraining reads/writes to the workspace root.
4. **Keep dependencies updated**: Run `npm audit` and `cargo update` regularly
5. **Review security reports**: Check CI security scan results

### For Users

1. **Keep Sruja updated**: Use the latest version
2. **Review generated code**: Always review code generated from untrusted sources
3. **Validate inputs**: Validate all user inputs before processing
4. **Use HTTPS**: Always use HTTPS for network requests

## Security Features

### Input Validation

- DSL parsing and validation is centralized in Rust (parser + validator), producing structured diagnostics without leaking sensitive data.
- VS Code extension operations should validate user/workspace inputs (paths, URIs, JSON) before invoking the CLI or WASM.

### Error Handling

- Structured error types that don't leak sensitive information
- Error sanitization in logging
- No stack traces in production error messages

### Dependencies

- Minimal external dependencies
- Regular security audits
- Pinned dependency versions

## Known Security Considerations

### WASM Execution

- WASM modules are executed in isolated environments
- No direct file system access from WASM
- All I/O is mediated through adapters

### LSP Server

- LSP server runs locally (not exposed to network)
- No remote code execution
- File access limited to workspace

### Browser Storage

- The project does not rely on browser storage for core security guarantees.

## Verifying Releases

All release tags are GPG signed for authenticity verification.

### Import the Public Key

```bash
# Download and import the public key
curl -s https://raw.githubusercontent.com/sruja-ai/sruja/main/.github/gpg-public-key.asc | gpg --import

# Or import from the repository
gpg --import .github/gpg-public-key.asc
```

### Verify a Release Tag

```bash
# Verify a specific tag
git tag -v v1.2.3

# List all tags with verification
git tag -v
```

The public key is also available at: [`.github/gpg-public-key.asc`](https://github.com/sruja-ai/sruja/blob/main/.github/gpg-public-key.asc)

**Key Details:**

- **Name**: Sruja Bot (Runs Sruja Workflows)
- **Email**: bot@sruja.ai
- **Purpose**: Signs all release tags created by CI/CD workflows

## Security Updates

Security updates will be:

- Released as patch versions (0.1.x)
- Documented in CHANGELOG.md
- Announced via GitHub security advisories
- GPG signed for verification

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Node.js Security Best Practices](https://nodejs.org/en/docs/guides/security/)
- [Rust Security Best Practices](https://rust-lang.github.io/rust-clippy/)
