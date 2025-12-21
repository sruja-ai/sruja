# Secrets Management

This document describes how secrets and sensitive configuration are managed in the Sruja project.

## Principles

1. **Never commit secrets**: Secrets must never be committed to version control
2. **Use environment variables**: Secrets should be provided via environment variables
3. **Separate by environment**: Different secrets for development, staging, production
4. **Rotate regularly**: Secrets should be rotated periodically
5. **Least privilege**: Secrets should have minimal required permissions

## Secret Types

### CI/CD Secrets

Stored in GitHub Secrets (`.github/workflows/`):

- `CODACY_PROJECT_TOKEN` - Codacy coverage reporting
- `CODECOV_TOKEN` - Codecov coverage reporting
- `NPM_TOKEN` - NPM publishing (if applicable)
- `FIREBASE_*` - Firebase deployment credentials
- `VERCEL_*` - Vercel deployment credentials (if used)

### Application Secrets

For frontend applications, use environment variables:

```bash
# .env.local (not committed)
VITE_FIREBASE_API_KEY=...
VITE_FIREBASE_AUTH_DOMAIN=...
VITE_FIREBASE_PROJECT_ID=...
```

### Backend Secrets

For Go backend (if needed in future):

- Use environment variables
- Consider using secret management services (AWS Secrets Manager, HashiCorp Vault)
- Never hardcode in source code

## Environment Variables

### Frontend (Vite)

Prefix with `VITE_` to expose to client:

```typescript
// ✅ Good
const apiKey = import.meta.env.VITE_API_KEY;

// ❌ Bad - not exposed
const apiKey = import.meta.env.API_KEY;
```

### Backend (Go)

Use `os.Getenv()`:

```go
// ✅ Good
apiKey := os.Getenv("API_KEY")

// ❌ Bad - hardcoded
apiKey := "hardcoded-key"
```

## Secret Scanning

We use automated secret scanning to prevent accidental commits:

1. **TruffleHog**: Scans commits for secrets (in CI)
2. **GitHub Secret Scanning**: GitHub automatically scans repositories
3. **Pre-commit hooks**: Can add secret scanning to git hooks

## Best Practices

### For Developers

1. **Never commit secrets**: Even in "temporary" commits
2. **Use .env files**: Create `.env.local` for local development
3. **Add to .gitignore**: Ensure `.env*` files are ignored
4. **Rotate if exposed**: If a secret is accidentally committed, rotate it immediately
5. **Use different secrets**: Don't reuse production secrets in development

### For CI/CD

1. **Use GitHub Secrets**: Store secrets in GitHub repository settings
2. **Limit scope**: Only grant necessary permissions
3. **Audit access**: Regularly review who has access to secrets
4. **Rotate regularly**: Rotate secrets periodically
5. **Monitor usage**: Monitor secret usage for anomalies

## Secret Rotation

### When to Rotate

- Immediately if exposed (committed, logged, etc.)
- Periodically (every 90 days recommended)
- When team members leave
- When security policies require

### How to Rotate

1. Generate new secret
2. Update in secret store (GitHub Secrets, environment, etc.)
3. Update applications/services
4. Verify functionality
5. Revoke old secret
6. Document rotation

## Emergency Procedures

### If Secret is Exposed

1. **Immediately rotate** the exposed secret
2. **Revoke** the old secret
3. **Audit** what the secret had access to
4. **Review logs** for unauthorized access
5. **Notify team** if necessary
6. **Document** the incident

### If Secret is Committed

1. **Rotate** the secret immediately
2. **Remove from git history** (if possible):
   ```bash
   git filter-branch --force --index-filter \
     "git rm --cached --ignore-unmatch path/to/file" \
     --prune-empty --tag-name-filter cat -- --all
   ```
3. **Force push** (coordinate with team)
4. **Consider** repository history rewrite if critical

## Tools

### Secret Scanning
- **TruffleHog**: https://github.com/trufflesecurity/trufflehog
- **GitGuardian**: https://www.gitguardian.com/
- **GitHub Secret Scanning**: Built into GitHub

### Secret Management
- **GitHub Secrets**: For CI/CD
- **Environment Variables**: For applications
- **AWS Secrets Manager**: For cloud deployments (if applicable)
- **HashiCorp Vault**: For enterprise deployments

## References

- [OWASP Secrets Management](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [GitHub Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [12 Factor App: Config](https://12factor.net/config)

