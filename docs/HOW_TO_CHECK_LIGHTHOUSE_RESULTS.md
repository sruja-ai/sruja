# How to Check Lighthouse CI Results

## 🔍 Quick Guide

After the Lighthouse CI workflow runs, you can check results in several ways:

## 1. GitHub Actions UI (Easiest)

### Step-by-Step:

1. **Go to your repository on GitHub**
2. **Click on "Actions" tab** (top navigation)
3. **Find the workflow run**:
   - Look for "SEO & Performance (Lighthouse CI)" workflow
   - Click on the latest run (should show ✅ or ❌ status)
4. **View the workflow run**:
   - You'll see all the steps that ran
   - Look for "Run Lighthouse CI" step
   - Click to expand and see output
5. **Check Summary**:
   - Scroll to bottom of workflow run
   - Look for "Summary" section
   - Shows overview of results

### What You'll See:

```
## Lighthouse CI Results

✅ Lighthouse CI ran successfully

**Cost:** FREE - No API keys required!

**Results:** Check workflow artifacts for detailed reports

**Note:** Running against localhost. For production testing, run against https://sruja.ai
```

## 2. Download Artifacts (Detailed Reports)

### Step-by-Step:

1. **Go to workflow run** (as above)
2. **Scroll to "Artifacts" section** (at bottom of page)
3. **Download artifacts**:
   - Click on artifact name (usually "lhci_reports" or similar)
   - Artifacts are ZIP files containing:
     - HTML reports (view in browser)
     - JSON data (for programmatic access)
     - Screenshots (if enabled)
4. **Extract and view**:
   - Extract ZIP file
   - Open `*.html` files in browser
   - See full Lighthouse report with all scores

### What's in Artifacts:

```
lhci_reports/
├── homepage.html          # Full Lighthouse report for homepage
├── courses.html           # Report for /courses/ page
├── docs.html              # Report for /docs/ page
├── manifest.json          # Metadata about the run
└── ...
```

## 3. View in Workflow Logs

### Step-by-Step:

1. **Go to workflow run** → Click on "Run Lighthouse CI" step
2. **Scroll through logs** to see:
   - Individual test scores
   - Performance metrics
   - SEO checks
   - Accessibility audits
   - Any failures or warnings

### Example Log Output:

```
=== Running Lighthouse CI ===
✅ Lighthouse CI is FREE - no API keys required!

✓ Running Lighthouse on http://localhost:1313/
✓ Running Lighthouse on http://localhost:1313/courses/
✓ Running Lighthouse on http://localhost:1313/docs/

Performance: 92
SEO: 100
Accessibility: 95
Best Practices: 100

✅ Lighthouse CI results saved to artifacts
```

## 4. Check Temporary Public Storage Links

If using `--upload.target=temporary-public-storage`, the workflow will:

1. **Upload results** to temporary public storage
2. **Provide links** in the logs
3. **Access links** (valid for 7 days)
4. **View reports** directly in browser

### Finding Links:

- Check workflow logs for "View results at: https://..."
- Links look like: `https://storage.googleapis.com/lighthouse-infrastructure/...`

## 📊 Understanding Results

### Score Thresholds (in `.lighthouserc.json`):

- **SEO**: ≥ 95 (error if below) ✅
- **Performance**: ≥ 80 (error if below) ✅
- **Accessibility**: ≥ 90 (error if below) ✅
- **Best Practices**: ≥ 90 (error if below) ✅

### What Each Score Means:

#### Performance (0-100)
- **90-100**: Excellent ✅
- **80-89**: Good ✅
- **50-79**: Needs improvement ⚠️
- **0-49**: Poor ❌

#### SEO (0-100)
- **90-100**: Excellent ✅
- **50-89**: Needs improvement ⚠️
- **0-49**: Poor ❌

#### Accessibility (0-100)
- **90-100**: Excellent ✅
- **50-89**: Needs improvement ⚠️
- **0-49**: Poor ❌

#### Best Practices (0-100)
- **90-100**: Excellent ✅
- **50-89**: Needs improvement ⚠️
- **0-49**: Poor ❌

## 🔔 Failure Alerts

### If Scores Drop Below Thresholds:

1. **Workflow will show ❌** (failed status)
2. **Build fails** (if enabled in CI)
3. **Check logs** for specific failures:
   ```
   ✗ categories:performance
     Expected >= 0.8, but found 0.65
   ```
4. **Fix issues** and push again

## 📈 Tracking Trends

### Weekly Reports:

Since the workflow runs weekly:
- Compare scores week-over-week
- Track improvements or regressions
- Download artifacts to keep historical data

### Manual Checks:

You can also run manually:
1. Go to **Actions** tab
2. Click **"SEO & Performance (Lighthouse CI)"**
3. Click **"Run workflow"** button (top right)
4. Select branch and click **"Run workflow"**

## 🚀 Quick Access URLs

### GitHub Actions:
- **All workflows**: `https://github.com/[org]/[repo]/actions`
- **Lighthouse workflow**: `https://github.com/[org]/[repo]/actions/workflows/seo-lighthouse.yml`
- **Latest run**: `https://github.com/[org]/[repo]/actions/runs/[run-id]`

## 💡 Pro Tips

### 1. Bookmark the Workflow
- Save the workflow URL for quick access
- Check weekly after scheduled runs

### 2. Set Up Notifications
- GitHub can email you on workflow failures
- Settings → Notifications → Actions

### 3. Compare Results
- Download artifacts from multiple runs
- Compare scores over time
- Track improvements

### 4. Share Results
- Share artifact links with team
- Include in PR reviews
- Document in reports

## 🎯 What to Look For

### Critical Issues:
- ❌ Scores below thresholds
- ❌ Failed accessibility checks
- ❌ Missing SEO elements
- ❌ Performance regressions

### Improvements:
- ✅ Scores above thresholds
- ✅ Performance improvements
- ✅ SEO optimizations
- ✅ Accessibility fixes

## 📝 Example: Checking Results

```bash
# 1. Go to GitHub repository
# 2. Click "Actions" tab
# 3. Find "SEO & Performance (Lighthouse CI)"
# 4. Click on latest run
# 5. Scroll to "Artifacts" section
# 6. Download "lhci_reports"
# 7. Extract and open homepage.html in browser
# 8. Review scores and recommendations
```

## ✅ Checklist

After each Lighthouse CI run:

- [ ] Check workflow status (✅ or ❌)
- [ ] Review summary in workflow
- [ ] Download artifacts if needed
- [ ] Check individual page scores
- [ ] Review failed checks (if any)
- [ ] Fix critical issues
- [ ] Track trends over time

## 🔗 Related Docs

- [Lighthouse CI Free Guide](./LIGHTHOUSE_CI_FREE.md)
- [SEO Implementation Guide](./SEO_IMPLEMENTATION_GUIDE.md)
- [CI/CD Automation](./CI_CD_SEO_AUTOMATION.md)

