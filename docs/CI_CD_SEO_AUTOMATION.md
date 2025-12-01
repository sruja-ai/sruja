# CI/CD SEO Automation

This document describes the automated SEO validation integrated into the CI/CD pipeline.

## 🔄 Automated Workflows

### 1. SEO Validation (`seo-validation.yml`)

**When it runs:**
- On every pull request that touches learn app content
- On pushes to main branch
- Manual trigger via `workflow_dispatch`

**What it checks:**
- ✅ **Missing descriptions**: Finds pages without description/summary
- ✅ **SEO tags**: Validates Open Graph, Twitter Cards, canonical URLs, meta descriptions
- ✅ **Structured data**: Checks JSON-LD presence and validates syntax
- ✅ **Sitemap**: Verifies sitemap.xml exists and contains URLs
- ✅ **robots.txt**: Ensures robots.txt exists with sitemap reference

**Failure conditions:**
- Missing critical SEO tags (Open Graph, Twitter Cards, canonical URLs)
- Missing structured data (JSON-LD)
- Missing or empty sitemap.xml
- Missing robots.txt

**Non-blocking warnings:**
- Pages without descriptions (reported but doesn't fail build)

### 2. Lighthouse CI (`seo-lighthouse.yml`)

**When it runs:**
- Weekly schedule (Mondays 9 AM UTC)
- Manual trigger via `workflow_dispatch`

**What it checks:**
- Performance score (target: ≥80)
- SEO score (target: ≥95)
- Accessibility score (target: ≥90)
- Best practices score (target: ≥90)
- Specific SEO checks:
  - Viewport meta tag
  - Document title
  - Meta description
  - HTML lang attribute
  - Link text quality

**Note**: This runs on localhost during CI. For production monitoring, consider running Lighthouse on the deployed URL.

### 3. Enhanced Learn Site Checks (in `ci.yml`)

**When it runs:**
- On every PR and push to main

**Additional checks:**
- Quick SEO validation (sitemap, robots.txt, Open Graph tags)
- Fails build if critical SEO elements are missing

## 📊 Workflow Integration

### Current CI/CD Pipeline

```
┌─────────────────┐
│   PR Created    │
└────────┬────────┘
         │
         ├─→ [Test] (Go tests)
         │
         ├─→ [Build Examples]
         │
         ├─→ [Learn Site Checks] ← Quick SEO check
         │                         (sitemap, robots.txt, OG tags)
         │
         └─→ [SEO Validation] ← Full SEO audit
                                 (all checks + missing descriptions)

         ┌─────────────────┐
         │ Push to main    │
         └────────┬────────┘
                  │
                  ├─→ [SEO Validation]
                  │
                  └─→ [Deploy Learn App]
                      (builds and deploys to GitHub Pages)
```

## ✅ What Gets Validated

### Critical (Build Fails)
- [x] Open Graph tags present in HTML
- [x] Twitter Card tags present
- [x] Canonical URLs present
- [x] Meta descriptions present
- [x] JSON-LD structured data present
- [x] Valid JSON-LD syntax
- [x] sitemap.xml exists and non-empty
- [x] robots.txt exists

### Warnings (Reported Only)
- [ ] Pages without descriptions (counted and listed)
- [ ] robots.txt missing sitemap reference

### Lighthouse Checks (Weekly)
- [ ] Performance score ≥ 80
- [ ] SEO score ≥ 95
- [ ] Accessibility score ≥ 90
- [ ] Best practices score ≥ 90
- [ ] Viewport meta tag
- [ ] Document title
- [ ] Meta description
- [ ] HTML lang attribute

## 🔍 Viewing Results

### In GitHub Actions

1. **Go to Actions tab** in your repository
2. **Click on a workflow run**
3. **Expand the job** to see individual steps
4. **Check "Summary"** at the bottom for SEO validation summary

### Example Output

```
## SEO Validation Summary

### Results:
- Missing descriptions: 15
- SEO tag errors: 0
- Structured data errors: 0
- Sitemap errors: 0
- robots.txt errors: 0

✅ **All critical SEO checks passed!**

⚠️  **Note**: 15 pages are missing descriptions. This is not a blocker but should be addressed.
```

## 🛠️ Manual Testing

You can also run these checks locally:

```bash
# Run the test script
./scripts/test-seo-local.sh

# Check for missing descriptions
./scripts/check-missing-descriptions.sh

# Build and check manually
cd learn
hugo --minify
grep -r "og:title" public/*.html | wc -l
```

## 🔧 Configuration

### SEO Validation Thresholds

Edit `.github/workflows/seo-validation.yml` to adjust:
- Which checks are critical vs warnings
- Minimum counts for tags
- Custom validation rules

### Lighthouse Thresholds

Edit `.lighthouserc.json` to adjust:
- Score thresholds
- Which assertions are errors vs warnings
- Which URLs to test

Example:
```json
{
  "ci": {
    "assert": {
      "assertions": {
        "categories:seo": ["error", {"minScore": 0.95}],
        "categories:performance": ["error", {"minScore": 0.8}]
      }
    }
  }
}
```

## 📝 Fixing Failures

### Common Issues

**1. Missing Open Graph tags**
- Check that `layouts/partials/seo.html` exists
- Verify it's included in `layouts/partials/docs/inject/head.html`
- Rebuild: `hugo --cleanDestinationDir && hugo --minify`

**2. Missing structured data**
- Check that `layouts/partials/structured-data.html` exists
- Verify it's included in head.html
- Validate JSON-LD syntax with https://validator.schema.org/

**3. Missing sitemap.xml**
- Check `baseURL` in `hugo.toml`
- Hugo should auto-generate sitemap
- Verify `public/sitemap.xml` after build

**4. Missing robots.txt**
- Check that `static/robots.txt` exists in `learn/` directory
- Rebuild and verify it's copied to `public/`

## 🚀 Best Practices

1. **Fix failures immediately** - Don't merge PRs with SEO validation failures
2. **Address warnings gradually** - Missing descriptions are warnings, but should be fixed
3. **Review Lighthouse reports** - Weekly reports help catch regressions
4. **Monitor trends** - Watch for SEO score degradation over time

## 📚 Related Documentation

- [SEO Implementation Guide](./SEO_IMPLEMENTATION_GUIDE.md)
- [SEO Quick Start](./SEO_QUICK_START.md)
- [Learn App Improvements](./LEARN_APP_IMPROVEMENTS.md)

