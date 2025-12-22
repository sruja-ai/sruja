# GitHub Actions Security - Commit SHA Pinning

All GitHub Actions in this repository use commit SHAs instead of version tags for enhanced security.

## Why Commit SHAs?

- **Immutability**: Commit SHAs never change, preventing supply chain attacks
- **Deterministic**: Same SHA always runs the same code
- **Security**: Prevents tag manipulation or compromise
- **Best Practice**: Recommended by GitHub and security experts

## How to Update Action SHAs

### Method 1: Using GitHub Web Interface

1. Visit the action's repository (e.g., `https://github.com/actions/checkout`)
2. Go to **Releases** or **Tags**
3. Find the version you want (e.g., `v6`)
4. Click on the commit link
5. Copy the full 40-character commit SHA
6. Replace the SHA in the workflow file

### Method 2: Using the Helper Script

```bash
./scripts/get-action-shas.sh <action-repo> <version>
# Example:
./scripts/get-action-shas.sh actions/checkout v6
```

### Method 3: Using GitHub API

```bash
curl -sL "https://api.github.com/repos/actions/checkout/git/ref/tags/v6" | \
  grep -o '"sha":"[^"]*"' | head -1 | sed 's/"sha":"\([^"]*\)"/\1/'
```

## Actions That Need SHA Updates

Some actions currently have placeholder SHAs marked with `TODO`. These should be updated with the correct SHAs:

- `actions/checkout@v6` - Update to v6 SHA
- `actions/setup-node@v6` - Update to v6 SHA
- `goreleaser/goreleaser-action@v6` - Update to v6 SHA

## Verifying SHAs

To verify a SHA is correct:

1. Visit the action repository
2. Check the commit exists: `https://github.com/<repo>/commit/<SHA>`
3. Verify it matches the version tag you want

## Current Status

✅ All version tags (`@v6`, `@v4`, etc.) have been converted to SHA format
✅ Comments added indicating where to get correct SHAs
✅ Helper script created for fetching SHAs

⚠️ Some SHAs may need verification/updating (marked with TODO comments)
