# GitHub PR Integration: Easy Preview Access

## Overview

Make it easy for developers to view architecture changes and preview snapshots directly from GitHub PRs. Multiple approaches from simple to advanced.

## Problem

When reviewing a PR with architecture changes:
- Hard to see what changed visually
- Need to clone repo and run commands locally
- Preview snapshots not easily accessible
- No direct link to view changes

## Solutions

### Option 1: GitHub Actions (Recommended)

Automatically generate preview snapshots and comment on PRs.

#### Setup

**`.github/workflows/sruja-preview.yml`**:
```yaml
name: Sruja Architecture Preview

on:
  pull_request:
    paths:
      - '**/*.sruja'
      - 'changes/**'
      - 'architecture/**'

jobs:
  preview:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Setup Sruja
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
      
      - name: Generate Preview Snapshot
        id: preview
        run: |
          # Detect changes in PR
          CHANGES=$(git diff --name-only origin/${{ github.base_ref }}...HEAD | grep '\.sruja$' | grep '^changes/' | sed 's|changes/||;s|\.sruja||' | tr '\n' ',' | sed 's/,$//')
          
          if [ -n "$CHANGES" ]; then
            PREVIEW_NAME="pr-${{ github.event.pull_request.number }}"
            sruja snapshot preview "$PREVIEW_NAME" --changes "$CHANGES"
            
            # Export to HTML for viewing
            sruja export html "snapshots/preview/$PREVIEW_NAME.sruja" "preview.html"
            
            # Upload as artifact
            echo "preview_name=$PREVIEW_NAME" >> $GITHUB_OUTPUT
          fi
      
      - name: Upload Preview
        uses: actions/upload-artifact@v4
        if: steps.preview.outputs.preview_name
        with:
          name: architecture-preview
          path: preview.html
      
      - name: Comment on PR
        uses: actions/github-script@v7
        if: steps.preview.outputs.preview_name
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            const previewHtml = fs.readFileSync('preview.html', 'utf8');
            
            // Extract JSON from HTML (if embedded)
            // Or generate viewer URL
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## 🏗️ Architecture Preview
              
              Preview snapshot generated for this PR.
              
              **Changes included:**
              ${process.env.CHANGES || 'All changes in PR'}
              
              **View Preview:**
              - [Open in Studio](https://sruja.ai/studio?preview=pr-${{ github.event.pull_request.number }})
              - [Download HTML Preview](./preview.html)
              
              **Note:** This is a preview snapshot - changes may not be finalized.
              `
            });
```

#### Benefits

✅ **Automatic** - Runs on every PR with .sruja changes  
✅ **No manual steps** - Preview generated automatically  
✅ **PR comments** - Direct links in PR  
✅ **Artifacts** - Preview files available for download  

### Option 2: GitHub App / Bot

GitHub App that detects .sruja files and generates previews.

**Features**:
- Detects .sruja files in PR
- Generates preview snapshot
- Comments on PR with preview link
- Updates comment when PR changes

**Implementation** (Future - Cloud Studio):
- GitHub App installed on repo
- Webhook receives PR events
- Generates preview via Cloud Studio API
- Comments with preview link

### Option 3: CLI Command for PR Integration

Command to generate preview and prepare PR comment.

```bash
# Generate preview for PR and output markdown comment
sruja pr preview --changes <change1,change2> --output comment.md

# Example output (comment.md):
# ## 🏗️ Architecture Preview
# 
# Preview snapshot: `pr-123`
# 
# [View in Studio](https://sruja.ai/studio?preview=pr-123)
# [View Diff](https://sruja.ai/viewer?diff=main...pr-123)
```

**Usage**:
```bash
# Developer workflow
sruja pr preview --changes 003-add-analytics,004-add-payment > pr-comment.md
# Copy comment.md content to PR description or comment
```

### Option 4: PR Template with Instructions

Add to PR template to guide developers.

**`.github/pull_request_template.md`**:
```markdown
## Architecture Changes

If this PR includes architecture changes:

1. **Generate preview snapshot:**
   ```bash
   sruja snapshot preview "pr-$(gh pr view --json number -q .number)" --changes <change-ids>
   ```

2. **Add preview link to PR:**
   - View in Studio: `https://sruja.ai/studio?preview=pr-<number>`
   - Or attach preview HTML file

3. **List changes included:**
   - Change 001: Add analytics
   - Change 002: Add payment
```

### Option 5: Direct Viewer Links

URL parameters for Studio/Viewer to load previews directly.

**Studio URL Format**:
```
https://sruja.ai/studio?preview=pr-123
https://sruja.ai/studio?change=003-add-analytics
https://sruja.ai/studio?snapshot=v1.1.0
```

**Viewer URL Format**:
```
https://sruja.ai/viewer?preview=pr-123
https://sruja.ai/viewer?diff=main...pr-123
https://sruja.ai/viewer?change=003-add-analytics
```

**Implementation**:
- Studio/Viewer reads URL parameters
- Loads preview snapshot from GitHub Pages/CDN
- Or loads from GitHub API (if authenticated)

### Option 6: GitHub Pages Hosting

Host preview snapshots on GitHub Pages for easy access.

**Workflow**:
```yaml
- name: Deploy Preview to GitHub Pages
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./previews
    destination_dir: previews/pr-${{ github.event.pull_request.number }}
```

**Access**:
- `https://<org>.github.io/<repo>/previews/pr-123/index.html`
- Direct link in PR comments

## Recommended Approach

### Phase 1: Simple (Immediate)

1. **PR Template** - Guide developers to generate previews
2. **CLI Command** - `sruja pr preview` generates comment markdown
3. **Manual PR Comments** - Developers paste preview links
4. **GitHub Pages** - Customer hosts previews on their GitHub Pages
5. **Direct URLs** - Studio/Viewer loads from customer's GitHub Pages

### Phase 2: Automated (Short-term)

1. **GitHub Actions** - Auto-generate previews on PR
2. **GitHub Pages Deployment** - Auto-deploy previews to customer's GitHub Pages
3. **PR Comments** - Auto-comment with preview links (pointing to customer's GitHub Pages)
4. **URL Convention** - Support `github://org/repo/pr/123` format

### Phase 3: Advanced (Future)

1. **GitHub App** - Full integration with Cloud Studio (has GitHub API access)
2. **Status Checks** - Preview validation in PR checks
3. **Rich Previews** - Embedded diagrams in PR comments
4. **OAuth Integration** - Studio can access private repos via GitHub OAuth
5. **Private Repo Support** - Customer hosting or GitHub API integration

## Architecture: How Studio Accesses Customer PRs

### Public Repos: GitHub Pages (Recommended)

**Customer hosts previews on their GitHub Pages**:

```
Customer GitHub Repo (Public)
  └── GitHub Actions
      └── Generates preview.json
      └── Deploys to gh-pages branch
          └── previews/pr-123/preview.json
              └── Accessible at: https://<org>.github.io/<repo>/previews/pr-123/preview.json

sruja.ai Studio
  └── Receives URL: ?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json
  └── Fetches JSON from customer's GitHub Pages (public, no auth needed)
  └── Displays preview
```

**Benefits**:
- ✅ No auth needed (public GitHub Pages)
- ✅ Customer controls hosting
- ✅ No sruja.ai backend required
- ✅ Works for all customers

### Private Repos: Options

GitHub Pages are **public only**, so private repos need different approaches:

#### Option 1: Customer's Own Hosting (Recommended for Private)

**Customer hosts previews on their own domain/server**:

```
Customer GitHub Repo (Private)
  └── GitHub Actions
      └── Generates preview.json
      └── Uploads to customer's server/CDN
          └── previews/pr-123/preview.json
              └── Accessible at: https://<customer-domain>/previews/pr-123/preview.json

sruja.ai Studio
  └── Receives URL: ?preview=https://<customer-domain>/previews/pr-123/preview.json
  └── Fetches JSON from customer's server
  └── Displays preview
```

**Alternative: Self-Hosted Studio**:
Customer can deploy their own Studio instance (see [SELF_HOSTED_STUDIO.md](./SELF_HOSTED_STUDIO.md)):
- Deploy Studio on customer's infrastructure
- Studio stores previews internally
- Share preview links: `https://studio.customer.com/preview/pr-123`
- No need to host preview JSON files separately

**GitHub Actions Example**:
```yaml
- name: Upload Preview to Customer Server
  run: |
    # Upload to customer's S3, CDN, or internal server
    aws s3 cp preview.json s3://<bucket>/previews/pr-${{ github.event.pull_request.number }}/preview.json
    # Or use customer's API
    curl -X POST https://<customer-api>/previews \
      -H "Authorization: Bearer ${{ secrets.CUSTOMER_API_TOKEN }}" \
      -F "file=@preview.json" \
      -F "pr=${{ github.event.pull_request.number }}"
```

**PR Comment**:
```markdown
[🎨 View Preview](https://sruja.ai/studio?preview=https://<customer-domain>/previews/pr-123/preview.json)
```

**Benefits**:
- ✅ Works for private repos
- ✅ Customer controls access/auth
- ✅ Can use internal networks
- ✅ No sruja.ai backend needed
- ❌ Requires customer infrastructure

#### Option 2: GitHub API with OAuth (Future)

**Studio calls GitHub API** (requires user authentication):

```
Customer GitHub Repo (Private)
  └── Preview stored in repo: previews/pr-123/preview.json
      └── Or GitHub Artifacts

sruja.ai Studio
  └── User clicks: ?preview=github://<org>/<repo>/pr/123
  └── User authenticates with GitHub (OAuth)
  └── Studio receives GitHub token
  └── Studio calls GitHub API: GET /repos/<org>/<repo>/contents/previews/pr-123/preview.json
  └── GitHub API returns preview (if user has access)
  └── Studio displays preview
```

**Implementation**:
```typescript
// Studio OAuth flow
async function loadGitHubPreview(org: string, repo: string, prNumber: string) {
  // Check if user is authenticated
  let token = localStorage.getItem('github_token');
  
  if (!token) {
    // Redirect to GitHub OAuth
    const clientId = 'your-github-app-client-id';
    const redirectUri = encodeURIComponent(`${window.location.origin}/auth/callback`);
    const scope = 'repo'; // Access to private repos
    window.location.href = `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&scope=${scope}`;
    return;
  }
  
  // Fetch preview from GitHub API
  const apiUrl = `https://api.github.com/repos/${org}/${repo}/contents/previews/pr-${prNumber}/preview.json`;
  const response = await fetch(apiUrl, {
    headers: {
      'Authorization': `token ${token}`,
      'Accept': 'application/vnd.github.v3.raw'
    }
  });
  
  if (!response.ok) {
    if (response.status === 401) {
      // Token expired, re-authenticate
      localStorage.removeItem('github_token');
      loadGitHubPreview(org, repo, prNumber);
      return;
    }
    throw new Error(`Failed to load preview: ${response.statusText}`);
  }
  
  const json = await response.json();
  setArchitecture(json);
}
```

**PR Comment**:
```markdown
[🎨 View Preview](https://sruja.ai/studio?preview=github://<org>/<repo>/pr/123)
```

**Benefits**:
- ✅ Works for private repos
- ✅ Can access PR data directly
- ✅ No customer infrastructure needed
- ❌ Requires OAuth setup
- ❌ User must authenticate
- ❌ More complex

#### Option 3: GitHub Artifacts (Limited)

**GitHub Actions uploads preview as artifact**:

```
Customer GitHub Repo (Private)
  └── GitHub Actions
      └── Generates preview.json
      └── Uploads as GitHub Artifact
          └── Accessible via GitHub API (requires auth)

sruja.ai Studio
  └── User authenticates with GitHub
  └── Studio calls GitHub API to download artifact
  └── Displays preview
```

**Limitations**:
- ❌ Artifacts expire (90 days default)
- ❌ Requires GitHub API access
- ❌ Not ideal for long-term sharing

#### Option 4: Cloud Studio (Future)

**Cloud Studio has built-in GitHub integration**:

```
Customer GitHub Repo (Private)
  └── Cloud Studio detects PR
  └── Cloud Studio generates preview
  └── Cloud Studio stores preview (with GitHub App permissions)

Cloud Studio
  └── User opens: ?preview=github://<org>/<repo>/pr/123
  └── Cloud Studio has GitHub App token (no user auth needed)
  └── Fetches preview directly
  └── Displays preview
```

**Benefits**:
- ✅ Works for private repos
- ✅ No user authentication needed (GitHub App)
- ✅ Seamless experience
- ❌ Requires Cloud Studio infrastructure

## Implementation: GitHub Actions Workflow

### Complete Workflow

```yaml
name: Sruja Architecture Preview

on:
  pull_request:
    types: [opened, synchronize, reopened]
    paths:
      - '**/*.sruja'
      - 'changes/**'
      - 'architecture/**'

jobs:
  preview:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      artifacts: write
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Install Sruja
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Detect Changes
        id: changes
        run: |
          # Find all .sruja files changed in PR
          CHANGED_FILES=$(git diff --name-only origin/${{ github.base_ref }}...HEAD | grep '\.sruja$' || true)
          
          # Extract change IDs from changes/ directory
          CHANGE_IDS=$(echo "$CHANGED_FILES" | grep '^changes/' | sed 's|changes/||;s|\.sruja||' | tr '\n' ',' | sed 's/,$//' || echo "")
          
          if [ -n "$CHANGE_IDS" ]; then
            echo "change_ids=$CHANGE_IDS" >> $GITHUB_OUTPUT
            echo "has_changes=true" >> $GITHUB_OUTPUT
          else
            echo "has_changes=false" >> $GITHUB_OUTPUT
          fi
      
      - name: Generate Preview
        if: steps.changes.outputs.has_changes == 'true'
        id: preview
        run: |
          PR_NUMBER=${{ github.event.pull_request.number }}
          PREVIEW_NAME="pr-$PR_NUMBER"
          CHANGE_IDS="${{ steps.changes.outputs.change_ids }}"
          
          # Generate preview snapshot
          sruja snapshot preview "$PREVIEW_NAME" --changes "$CHANGE_IDS"
          
          # Export to HTML
          sruja export html "snapshots/preview/$PREVIEW_NAME.sruja" "preview.html"
          
          # Generate diff (if base exists)
          if [ -f "current.sruja" ]; then
            sruja diff current "snapshots/preview/$PREVIEW_NAME.sruja" > diff.txt || true
          fi
          
          echo "preview_name=$PREVIEW_NAME" >> $GITHUB_OUTPUT
          echo "preview_file=preview.html" >> $GITHUB_OUTPUT
      
      - name: Upload Preview Artifact
        if: steps.preview.outputs.preview_name
        uses: actions/upload-artifact@v4
        with:
          name: architecture-preview-pr-${{ github.event.pull_request.number }}
          path: preview.html
          retention-days: 30
      
      - name: Export Preview JSON
        if: steps.preview.outputs.preview_name
        run: |
          # Export preview snapshot to JSON for GitHub Pages
          sruja export json "snapshots/preview/${{ steps.preview.outputs.preview_name }}.sruja" "preview.json"
      
      - name: Deploy to GitHub Pages
        if: steps.preview.outputs.preview_name
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: .
          destination_dir: previews/pr-${{ github.event.pull_request.number }}
          keep_files: false
          exclude_assets: |
            !preview.json
            !preview.html
      
      - name: Comment on PR
        if: steps.preview.outputs.preview_name
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const fs = require('fs');
            const previewName = '${{ steps.preview.outputs.preview_name }}';
            const changeIds = '${{ steps.changes.outputs.change_ids }}'.split(',').filter(Boolean);
            
            // Find existing comment
            const comments = await github.rest.issues.listComments({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
            });
            
            const existingComment = comments.data.find(c => 
              c.user.type === 'Bot' && c.body.includes('🏗️ Architecture Preview')
            );
            
            const org = context.repo.owner;
            const repo = context.repo.repo;
            const previewUrl = `https://${org}.github.io/${repo}/previews/${previewName}/preview.json`;
            
            const commentBody = `## 🏗️ Architecture Preview
              
              Preview snapshot generated for this PR.
              
              **Changes included:**
              ${changeIds.map(id => `- Change \`${id}\``).join('\n')}
              
              **View Preview:**
              - 🎨 [Open in Studio](https://sruja.ai/studio?preview=${encodeURIComponent(previewUrl)})
              - 📊 [View in Viewer](https://sruja.ai/viewer?preview=${encodeURIComponent(previewUrl)})
              - 📦 [Download Preview JSON](${previewUrl})
              - 📄 [Download Preview HTML](./preview.html) (artifact)
              
              **Note:** This is a preview snapshot - changes may not be finalized.
              
              ---
              *Preview generated automatically by Sruja GitHub Action*`;
            
            if (existingComment) {
              // Update existing comment
              await github.rest.issues.updateComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                comment_id: existingComment.id,
                body: commentBody
              });
            } else {
              // Create new comment
              await github.rest.issues.createComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: context.issue.number,
                body: commentBody
              });
            }
```

## CLI Command: PR Preview

### New Command

```bash
# Generate preview and output PR comment markdown
sruja pr preview [--changes <change1,change2>] [--pr-number <number>]

# Example
sruja pr preview --changes 003-add-analytics,004-add-payment --pr-number 123
```

**Output** (markdown for PR comment):
```markdown
## 🏗️ Architecture Preview

Preview snapshot: `pr-123`

**Changes included:**
- Change `003-add-analytics`
- Change `004-add-payment`

**View Preview:**
- [Open in Studio](https://sruja.ai/studio?preview=pr-123)
- [Download Preview](./preview.html)

**Note:** This is a preview snapshot - changes may not be finalized.
```

### Implementation

```go
// cmd/sruja/pr.go
func runPRPreview(cmd *cobra.Command, args []string) error {
    changeIDs := getStringSlice(cmd, "changes")
    prNumber := getString(cmd, "pr-number")
    
    if len(changeIDs) == 0 {
        // Auto-detect changes from git diff
        changeIDs = detectChangesFromGit()
    }
    
    previewName := fmt.Sprintf("pr-%s", prNumber)
    
    // Generate preview
    err := generatePreviewSnapshot(previewName, changeIDs)
    if err != nil {
        return err
    }
    
    // Export to HTML
    err = exportToHTML(fmt.Sprintf("snapshots/preview/%s.sruja", previewName), "preview.html")
    if err != nil {
        return err
    }
    
    // Generate PR comment markdown
    comment := generatePRComment(previewName, changeIDs)
    fmt.Println(comment)
    
    return nil
}
```

## URL Parameters for Studio/Viewer

### Studio Parameters

```typescript
// Studio URL handling
const urlParams = new URLSearchParams(window.location.search);

if (urlParams.has('preview')) {
  const previewUrl = urlParams.get('preview');
  
  // Handle different URL formats
  if (previewUrl.startsWith('http://') || previewUrl.startsWith('https://')) {
    // Direct URL (e.g., GitHub Pages, customer domain)
    loadPreviewFromURL(previewUrl);
  } else if (previewUrl.startsWith('github://')) {
    // Convention: github://org/repo/pr/123
    const [, org, repo, , prNumber] = previewUrl.split('/');
    
    // Try GitHub Pages first (public repos)
    const githubPagesUrl = `https://${org}.github.io/${repo}/previews/pr-${prNumber}/preview.json`;
    
    fetch(githubPagesUrl)
      .then(response => {
        if (response.ok) {
          // Public repo - GitHub Pages works
          return loadPreviewFromURL(githubPagesUrl);
        } else {
          // Private repo - need GitHub API
          return loadGitHubPreview(org, repo, prNumber);
        }
      })
      .catch(() => {
        // Network error - try GitHub API
        loadGitHubPreview(org, repo, prNumber);
      });
  } else {
    // Legacy: just preview name (requires GitHub API or local)
    loadPreviewSnapshot(previewUrl);
  }
}

if (urlParams.has('change')) {
  const changeId = urlParams.get('change');
  // Load change file and show preview
  // Requires GitHub API or local file access
  loadChangePreview(changeId);
}

if (urlParams.has('pr')) {
  const prNumber = urlParams.get('pr');
  // Load preview for PR
  // Requires GitHub API or convention URL
  loadPRPreview(prNumber);
}

async function loadPreviewFromURL(url: string) {
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to load preview: ${response.statusText}`);
    }
    const json = await response.json();
    setArchitecture(json);
  } catch (error) {
    console.error('Failed to load preview:', error);
    // Show error message to user
    alert(`Failed to load preview from ${url}. Make sure the preview is accessible.`);
  }
}

async function loadGitHubPreview(org: string, repo: string, prNumber: string) {
  // Check if user is authenticated
  let token = localStorage.getItem('github_token');
  
  if (!token) {
    // Redirect to GitHub OAuth
    const clientId = process.env.GITHUB_CLIENT_ID;
    const redirectUri = encodeURIComponent(`${window.location.origin}/auth/callback`);
    const scope = 'repo'; // Access to private repos
    window.location.href = `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&scope=${scope}`;
    return;
  }
  
  // Try to fetch from repo contents
  const apiUrl = `https://api.github.com/repos/${org}/${repo}/contents/previews/pr-${prNumber}/preview.json`;
  const response = await fetch(apiUrl, {
    headers: {
      'Authorization': `token ${token}`,
      'Accept': 'application/vnd.github.v3.raw'
    }
  });
  
  if (!response.ok) {
    if (response.status === 401) {
      // Token expired, re-authenticate
      localStorage.removeItem('github_token');
      loadGitHubPreview(org, repo, prNumber);
      return;
    }
    throw new Error(`Failed to load preview: ${response.statusText}`);
  }
  
  const json = await response.json();
  setArchitecture(json);
}
```

### Viewer Parameters

```typescript
// Viewer URL handling
if (urlParams.has('diff')) {
  const diff = urlParams.get('diff'); // e.g., "main...pr-123" or full URLs
  const [base, target] = diff.split('...');
  // Show diff view
  // Load both snapshots and compare
  showDiff(base, target);
}

if (urlParams.has('preview')) {
  const previewUrl = urlParams.get('preview');
  
  // Handle URL formats (same as Studio)
  if (previewUrl.startsWith('http://') || previewUrl.startsWith('https://')) {
    loadPreviewFromURL(previewUrl);
  } else if (previewUrl.startsWith('github://')) {
    // Expand convention to GitHub Pages URL
    const [, org, repo, , prNumber] = previewUrl.split('/');
    const githubPagesUrl = `https://${org}.github.io/${repo}/previews/pr-${prNumber}/preview.json`;
    loadPreviewFromURL(githubPagesUrl);
  } else {
    // Legacy format (requires GitHub API or local)
    loadPreview(previewUrl);
  }
}
```

## Developer Workflow

### Option A: Manual (Simple)

1. Developer creates PR with .sruja changes
2. Developer runs: `sruja pr preview --pr-number 123`
3. Developer copies markdown output
4. Developer pastes into PR comment
5. Reviewers click link to view preview in Studio/Viewer

### Option B: Automated (Recommended)

1. Developer creates PR with .sruja changes
2. GitHub Action automatically runs
3. GitHub Action generates preview snapshot
4. GitHub Action uploads preview as artifact
5. GitHub Action comments on PR with preview link
6. Reviewers click link: `https://sruja.ai/studio?preview=pr-123`
7. Studio/Viewer loads preview automatically from URL

### Option C: Cloud Studio (Future)

1. Developer creates PR with .sruja changes
2. Cloud Studio detects PR
3. Cloud Studio generates preview
4. Cloud Studio embeds preview in PR (or comments)
5. Reviewers view preview directly in PR

## Quick Access from PR

### One-Click Preview

**From GitHub PR comment**:
```markdown
[🎨 View Preview in Studio](https://sruja.ai/studio?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json)
[📊 View Preview in Viewer](https://sruja.ai/viewer?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json)
```

**User clicks link** → Studio/Viewer opens → Loads preview from customer's GitHub Pages

**Alternative (shorter URL with convention)**:
```markdown
[🎨 View Preview](https://sruja.ai/studio?preview=github://<org>/<repo>/pr/123)
```
Studio expands `github://` convention to full GitHub Pages URL.

### Direct Links

**Important**: Studio/Viewer hosted on `sruja.ai` cannot directly access customer GitHub repos. Use one of these approaches:

#### Option A: GitHub Pages (Recommended)

Customer hosts previews on their GitHub Pages:

**Studio URLs**:
- `https://sruja.ai/studio?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json`
- `https://sruja.ai/studio?preview=github://<org>/<repo>/previews/pr-123/preview.json` (convention)

**Viewer URLs**:
- `https://sruja.ai/viewer?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json`

#### Option B: GitHub API (Requires Auth)

Studio calls GitHub API to fetch preview (requires GitHub token):

**Studio URLs**:
- `https://sruja.ai/studio?preview=github://<org>/<repo>/pr/123` (requires auth)
- User authenticates with GitHub → Studio fetches via API

#### Option C: Customer Domain

Customer hosts previews on their own domain:

**Studio URLs**:
- `https://sruja.ai/studio?preview=https://<customer-domain>/previews/pr-123/preview.json`

#### Option D: GitHub Artifacts (Public)

GitHub Actions uploads preview as public artifact:

**Studio URLs**:
- `https://sruja.ai/studio?preview=github-artifact://<org>/<repo>/pr/123` (requires GitHub API)

### GitHub Pages Hosting (Recommended)

**Best approach**: Customer hosts previews on their GitHub Pages.

**Workflow**:
1. GitHub Action generates preview
2. Exports preview to JSON: `preview.json`
3. Uploads to `gh-pages` branch: `previews/pr-123/preview.json`
4. Accessible at: `https://<org>.github.io/<repo>/previews/pr-123/preview.json`
5. Studio/Viewer loads from this public URL

**PR Comment**:
```markdown
## 🏗️ Architecture Preview

**View Preview:**
- 🎨 [Open in Studio](https://sruja.ai/studio?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json)
- 📊 [View in Viewer](https://sruja.ai/viewer?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json)
- 📦 [Direct Link](https://<org>.github.io/<repo>/previews/pr-123/preview.json)
```

**Benefits**:
- ✅ Public access (no auth needed)
- ✅ Customer controls hosting
- ✅ Versioned (tied to PR number)
- ✅ Fast (GitHub Pages CDN)
- ✅ Shareable (direct links)
- ✅ No sruja.ai backend needed

**Studio Implementation**:
```typescript
// Studio loads preview from customer's GitHub Pages
const urlParams = new URLSearchParams(window.location.search);
if (urlParams.has('preview')) {
  const previewUrl = urlParams.get('preview');
  // Load from customer's GitHub Pages
  const response = await fetch(previewUrl);
  const json = await response.json();
  setArchitecture(json);
}
```

## Benefits

✅ **Easy Access** - One click from PR to preview  
✅ **Automatic** - No manual steps needed  
✅ **Clear** - Visual preview of architecture changes  
✅ **Shareable** - Links work for all reviewers  
✅ **Versioned** - Preview tied to PR number  
✅ **Fast** - No need to clone and run locally  
✅ **Private Repo Support** - Multiple options (customer hosting, GitHub API, Cloud Studio)

## Summary: Public vs Private Repos

| Repo Type | Recommended Approach | URL Format |
|-----------|---------------------|------------|
| **Public** | GitHub Pages | `https://sruja.ai/studio?preview=https://<org>.github.io/<repo>/previews/pr-123/preview.json` |
| **Private** | Customer hosting | `https://sruja.ai/studio?preview=https://<customer-domain>/previews/pr-123/preview.json` |
| **Private** | GitHub API (OAuth) | `https://sruja.ai/studio?preview=github://<org>/<repo>/pr/123` (requires auth) |
| **Private** | Cloud Studio (future) | `https://studio.sruja.ai/preview?github://<org>/<repo>/pr/123` (no user auth) |  

## Implementation Priority

1. **Phase 1** (Immediate):
   - CLI command: `sruja pr preview`
   - PR template with instructions
   - Manual PR comments

2. **Phase 2** (Short-term):
   - GitHub Actions workflow
   - Auto-comment on PRs
   - GitHub Pages hosting

3. **Phase 3** (Future):
   - GitHub App integration
   - Cloud Studio automation
   - Embedded previews in PR

## Example PR Comment

```markdown
## 🏗️ Architecture Preview

Preview snapshot generated for this PR.

**Changes included:**
- Change `003-add-analytics` (in-progress)
- Change `004-add-payment` (in-progress)

**View Preview:**
- 📦 [Download Preview HTML](./preview.html) (artifact)
- 🎨 [Open in Studio](https://sruja.ai/studio?preview=pr-123)
- 📊 [View Diff](https://sruja.ai/viewer?diff=main...pr-123)

**Note:** This is a preview snapshot - changes may not be finalized.
Some ADRs may have pending decisions.

---
*Preview generated automatically by Sruja GitHub Action*
```

