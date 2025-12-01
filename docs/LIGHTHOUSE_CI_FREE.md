# Lighthouse CI - FREE, No API Keys Required

## ✅ Quick Answer

**Lighthouse CI is 100% FREE and requires NO API keys!**

- ✅ Completely free to use
- ✅ No API keys needed
- ✅ No authentication required
- ✅ Works immediately in CI/CD
- ✅ Results saved as artifacts

## 🔍 What is Lighthouse CI?

Lighthouse CI is Google's tool for running Lighthouse audits automatically in CI/CD pipelines. It:
- Runs Lighthouse tests automatically
- Validates performance, SEO, accessibility scores
- Saves results as artifacts
- Can fail builds if scores drop below thresholds

## 💰 Cost Breakdown

### Lighthouse CI (What We Use)

**Cost:** ✅ **FREE**  
**API Keys:** ❌ **None needed**  
**Authentication:** ❌ **Not required**  
**Setup:** ⚡ **Minimal** (just install CLI)

**How it works:**
- Runs Lighthouse locally in your CI environment
- Tests your site directly (localhost or deployed URL)
- Generates reports and artifacts
- No external API calls needed

### PageSpeed Insights API (Different Service)

**Cost:** ✅ **FREE** (with usage limits)  
**API Keys:** ✅ **Required** (free to get)  
**Use case:** Programmatic access to Lighthouse via API

**Note:** This is a different service. We use Lighthouse CI, not PageSpeed Insights API.

## 🚀 How It Works

### Current Setup

Our workflow (`.github/workflows/seo-lighthouse.yml`):
1. Builds the Hugo site
2. Starts local server
3. Runs Lighthouse CI against localhost
4. Saves results as artifacts
5. **No API keys or authentication needed**

### Results

After running, you get:
- ✅ Performance score
- ✅ SEO score
- ✅ Accessibility score
- ✅ Best practices score
- ✅ Detailed reports (HTML, JSON)
- ✅ All saved as GitHub Actions artifacts

## 📊 Optional: Storing Results in GitHub

If you want to store results in GitHub (for history/trends):

### Option 1: GitHub App Token (Optional)

**Cost:** ✅ **FREE**  
**Setup:** 5 minutes

1. **Create GitHub App** (optional):
   - Go to repository settings
   - Create GitHub App with read/write access
   - Generate token

2. **Add to GitHub Secrets**:
   - Secret name: `LHCI_GITHUB_APP_TOKEN`
   - Value: Your GitHub App token

3. **Update workflow**:
   ```yaml
   env:
     LHCI_GITHUB_APP_TOKEN: ${{ secrets.LHCI_GITHUB_APP_TOKEN }}
   ```

**Benefit:** Results stored in GitHub, visible in UI

**Note:** This is completely optional. Without it, results are still saved as artifacts.

### Option 2: Temporary Public Storage (Default)

**Cost:** ✅ **FREE**  
**Setup:** ✅ **Already configured**

- Results uploaded to temporary public storage
- Links provided in workflow logs
- Accessible for 7 days
- No authentication needed

## 🎯 Running Lighthouse

### In CI/CD (Current)

```yaml
- name: Run Lighthouse CI
  run: |
    lhci autorun \
      --collect.url=http://localhost:1313/ \
      --collect.numberOfRuns=1
```

**Free:** ✅ Yes  
**API Keys:** ❌ No  
**Works:** ✅ Immediately

### Locally

```bash
# Install (one time)
npm install -g @lhci/cli

# Run against local site
lhci autorun --collect.url=http://localhost:1313/

# Run against deployed site
lhci autorun --collect.url=https://sruja.ai/
```

**Free:** ✅ Yes  
**API Keys:** ❌ No  
**Works:** ✅ Immediately

### Against Deployed Site

```bash
# Test production site
lhci autorun \
  --collect.url=https://sruja.ai/ \
  --collect.url=https://sruja.ai/courses/ \
  --collect.numberOfRuns=3
```

**Free:** ✅ Yes  
**API Keys:** ❌ No  
**Works:** ✅ Immediately

## 📈 What Gets Tested

Lighthouse CI tests:

### Performance
- First Contentful Paint
- Time to Interactive
- Largest Contentful Paint
- Cumulative Layout Shift
- Total Blocking Time

### SEO
- Meta description
- Document title
- HTML lang attribute
- Viewport meta tag
- Structured data
- Crawlable links

### Accessibility
- ARIA attributes
- Color contrast
- Keyboard navigation
- Screen reader support

### Best Practices
- HTTPS usage
- No console errors
- Image optimization
- Modern JavaScript

## 🔧 Configuration

### Current Config (`.lighthouserc.json`)

```json
{
  "ci": {
    "collect": {
      "numberOfRuns": 3
    },
    "assert": {
      "assertions": {
        "categories:seo": ["error", {"minScore": 0.95}],
        "categories:performance": ["error", {"minScore": 0.8}],
        "categories:accessibility": ["error", {"minScore": 0.9}]
      }
    }
  }
}
```

**Thresholds:**
- SEO: ≥ 95 (error if below)
- Performance: ≥ 80 (error if below)
- Accessibility: ≥ 90 (error if below)
- Best Practices: ≥ 90 (error if below)

Adjust these in `.lighthouserc.json` as needed.

## 🎉 Summary

| Feature | Status | Cost | API Keys |
|---------|--------|------|----------|
| **Lighthouse CI** | ✅ Working | $0 | ❌ None |
| **Results Storage** | ✅ Artifacts | $0 | ❌ None |
| **GitHub Integration** | ⚠️ Optional | $0 | ⚠️ Optional |
| **Score Validation** | ✅ Enabled | $0 | ❌ None |

## ✅ What We Have

**Current Setup:**
- ✅ Lighthouse CI configured
- ✅ Runs weekly automatically
- ✅ No API keys needed
- ✅ Completely free
- ✅ Results in artifacts
- ✅ Build fails if scores drop

**Optional Enhancements:**
- Store results in GitHub (requires optional token)
- Run against deployed URL (free, just change URL)

## 📚 Resources

- **Lighthouse CI**: https://github.com/GoogleChrome/lighthouse-ci
- **Lighthouse Docs**: https://github.com/GoogleChrome/lighthouse
- **Configuration Guide**: https://github.com/GoogleChrome/lighthouse-ci/blob/main/docs/configuration.md

## ❓ FAQ

**Q: Do I need an API key?**
A: No! Lighthouse CI is completely free and requires no API keys.

**Q: Is there a cost?**
A: No, it's 100% free.

**Q: What about PageSpeed Insights API?**
A: That's a different service. We use Lighthouse CI, which doesn't need it.

**Q: Can I store results in GitHub?**
A: Yes, but it's optional. Results are saved as artifacts by default.

**Q: Does it work against localhost?**
A: Yes! That's how our current workflow runs.

**Q: Can I test production URLs?**
A: Yes, just change the URL in the workflow.

## 🚀 Quick Start

**Everything is already configured and FREE!**

1. ✅ Workflow runs weekly automatically
2. ✅ No setup needed
3. ✅ No API keys required
4. ✅ Results in artifacts
5. ✅ Completely free

Just wait for the workflow to run or trigger it manually!

