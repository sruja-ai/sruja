# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Scanning

Sruja uses automated security scanning in CI:

### Go Security

- **Gosec**: Static analysis security scanner for Go code
- Runs on: Push, PR, weekly schedule
- Location: `.github/workflows/security.yml`

### JavaScript/TypeScript Security

- **npm audit**: Dependency vulnerability scanning
- Runs on: Push, PR, weekly schedule
- Location: `.github/workflows/security.yml`

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
2. **Validate inputs**: Use validation utilities from `@sruja/shared/utils/validation`
3. **Sanitize file paths**: Use `validateFilePath` from `@sruja/shared/utils/pathValidation`
4. **Keep dependencies updated**: Run `npm audit` and `go mod tidy` regularly
5. **Review security reports**: Check CI security scan results

### For Users

1. **Keep Sruja updated**: Use the latest version
2. **Review generated code**: Always review code generated from untrusted sources
3. **Validate inputs**: Validate all user inputs before processing
4. **Use HTTPS**: Always use HTTPS for network requests

## Security Features

### Input Validation

- Comprehensive validation utilities in `packages/shared/src/utils/validation.ts`
- Path validation to prevent path traversal attacks
- Type guards for runtime type checking

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

- Uses IndexedDB for browser storage
- No sensitive data stored without encryption
- Storage keys are namespaced

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
- [Go Security Best Practices](https://go.dev/doc/security/best-practices)
- [Node.js Security Best Practices](https://nodejs.org/en/docs/guides/security/)
