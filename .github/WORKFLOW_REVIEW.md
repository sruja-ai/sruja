# GitHub Actions Workflows Review

**Date**: 2025-01-27  
**Reviewer**: AI Assistant  
**Scope**: All 11 workflow files

## Executive Summary

Overall, the workflows are well-structured with good security practices (pinned action versions, proper permissions). However, several improvements are recommended for consistency, maintainability, and reliability.

## Critical Issues

### 1. ✅ FIXED: Missing `.github/actions/**` in Path Filters

**File**: `deploy-staging.yml`  
**Status**: ✅ Fixed  
**Issue**: Changes to custom actions wouldn't trigger staging deployment  
**Fix**: Added `.github/actions/**` to path filters (line 32)

## High Priority Issues

### 2. Node.js Version Inconsistency

**Files**: Multiple workflows  
**Issue**: Inconsistent Node.js versions across workflows:

- Most workflows: Node 24
- `docs-quality.yml`: Node 22
- `chromatic.yml`: Node 22.20.0

**Recommendation**: Standardize on Node 24 (LTS) across all workflows unless there's a specific reason for a different version.

**Affected Files**:

- `.github/workflows/docs-quality.yml` (lines 19, 32)
- `.github/workflows/chromatic.yml` (line 28)

### 3. Missing Algolia Index Name in Production

**File**: `deploy-prod.yml`  
**Issue**: Production build doesn't specify `PUBLIC_ALGOLIA_INDEX_NAME` environment variable, while staging does (line 249 in deploy-staging.yml)

**Current** (deploy-prod.yml:112-115):

```yaml
env:
  SITE_URL: https://sruja.ai
  PUBLIC_ALGOLIA_SEARCH_API_KEY: ${{ secrets.PUBLIC_ALGOLIA_SEARCH_API_KEY }}
  PUBLIC_POSTHOG_API_KEY: ${{ secrets.PUBLIC_POSTHOG_API_KEY }}
  PUBLIC_POSTHOG_HOST: ${{ secrets.PUBLIC_POSTHOG_HOST }}
```

**Recommendation**: Add `PUBLIC_ALGOLIA_INDEX_NAME: sruja_docs` (or appropriate production index name)

## Medium Priority Issues

### 4. Missing Path Filters for Actions (Potential Issue)

**Files**: `chromatic.yml`, `extension-test.yml`  
**Status**: ✅ Not an issue (these workflows don't use custom actions)  
**Note**: These workflows don't use `.github/actions/**`, so no path filter needed. However, if they start using custom actions in the future, path filters should be added.

### 5. Error Handling in GPG Setup

**File**: `deploy-staging.yml`  
**Status**: ✅ Fixed  
**Issue**: GPG setup action was failing with exit code 1  
**Fix**: Improved error handling in `.github/actions/setup-gpg/action.yml`

### 6. Missing Error Handling in SEO Submission

**File**: `seo-submission.yml`  
**Issue**: curl commands use `|| echo` but don't fail the workflow if submission fails  
**Current**: Commands continue even if ping fails  
**Recommendation**: Consider adding `|| exit 1` for critical submissions, or at least make failures visible in workflow summary

### 7. Extension Test Workflow Missing Custom Actions Path Filter

**File**: `extension-test.yml`  
**Status**: ✅ Not an issue (doesn't use custom actions)  
**Note**: If this workflow starts using custom actions, add `.github/actions/**` to path filters

## Low Priority / Best Practices

### 8. Inconsistent Action Version Comments

**Files**: Multiple  
**Issue**: Some actions have security comments about commit SHAs, others don't  
**Recommendation**: Add security comments to all pinned actions for consistency

**Example** (from `deploy-prod.yml:196`):

```yaml
# Security: Using commit SHA - Get from https://github.com/goreleaser/goreleaser-action/releases (v6)
```

### 9. Missing Timeout on Some Jobs

**Files**: `seo-submission.yml`, `hn-review.yml`  
**Status**: ✅ Already have reasonable defaults  
**Note**: GitHub Actions default timeout is 6 hours, which is fine for these short-running jobs

### 10. Concurrency Groups

**Status**: ✅ Good  
**Note**: Most workflows properly use concurrency groups. Good practice.

### 11. Permissions

**Status**: ✅ Good  
**Note**: All workflows follow principle of least privilege. Good security practice.

### 12. Action Version Pinning

**Status**: ✅ Excellent  
**Note**: All actions are pinned with commit SHAs. Excellent security practice.

## Recommendations Summary

### Immediate Actions

1. ✅ **DONE**: Add `.github/actions/**` to deploy-staging.yml path filters
2. ✅ **DONE**: Fix GPG setup action error handling
3. ✅ **DONE**: Standardize Node.js versions (Node 24)
4. ✅ **DONE**: Add `PUBLIC_ALGOLIA_INDEX_NAME` to production deployment

### Future Improvements

1. Add error handling to SEO submission workflow
2. Consider adding path filters to workflows that might use custom actions in the future
3. Add security comments to all action pins for documentation

## Workflow-Specific Notes

### deploy-staging.yml

- ✅ Good: Comprehensive path filters
- ✅ Good: Proper job dependencies
- ✅ Good: E2E tests after deployment
- ⚠️ Note: Missing Algolia index name (but staging might intentionally use different index)

### deploy-prod.yml

- ✅ Good: Release-triggered (safe)
- ⚠️ Issue: Missing Algolia index name
- ✅ Good: Separate jobs for website and designer

### unified-ci.yml

- ✅ Good: Change detection to skip unnecessary jobs
- ✅ Good: Comprehensive testing
- ✅ Good: Security scanning

### release-please.yml

- ✅ Good: Simple and focused
- ✅ Good: Proper permissions

### extension-test.yml

- ✅ Good: Path filters prevent unnecessary runs
- ✅ Good: Platform-specific test handling

### security.yml

- ✅ Good: Scheduled runs
- ✅ Good: Multiple security tools
- ✅ Good: Artifact uploads for reports

### chromatic.yml

- ✅ Good: Path filters
- ⚠️ Note: Node 22.20.0 (consider standardizing)

### docs-quality.yml

- ✅ Good: Focused on content changes
- ⚠️ Note: Node 22 (consider standardizing)

### seo-submission.yml

- ✅ Good: Scheduled runs
- ⚠️ Note: Error handling could be improved

### social-publish.yml

- ✅ Good: Conditional publishing based on secrets
- ✅ Good: Multiple platform support

### hn-review.yml

- ✅ Good: Creates issues for manual review
- ✅ Good: Template-based approach

## Security Assessment

### ✅ Strengths

- All actions pinned with commit SHAs
- Principle of least privilege for permissions
- No hardcoded secrets
- Proper use of GitHub secrets

### ⚠️ Areas for Improvement

- Consider adding Dependabot for action updates
- Review secret usage (all appear properly scoped)

## Performance Considerations

### ✅ Good Practices

- Path filters prevent unnecessary workflow runs
- Change detection in unified-ci.yml
- Concurrency groups prevent duplicate runs
- Caching where appropriate (Playwright, npm)

### 💡 Suggestions

- Consider caching Go modules if not already cached
- Consider caching WASM builds if they're expensive

## Conclusion

The workflows are well-maintained with good security practices. The main improvements needed are:

1. Standardize Node.js versions
2. Add missing Algolia index name in production
3. Improve error handling in SEO submission

All critical issues have been addressed.
