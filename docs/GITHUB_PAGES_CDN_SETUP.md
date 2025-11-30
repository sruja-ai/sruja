# GitHub Pages CDN Setup for Sruja Viewer

## Overview

Use GitHub Pages with custom domain `sruja.ai` to host the Sruja Viewer JavaScript library as a CDN. This is free, easy to set up, and works well for static assets.

**Custom Domain**: `sruja.ai` (already configured via `learn/CNAME`)

## Setup Steps

### 1. Enable GitHub Pages

1. Go to repository settings
2. Navigate to **Pages** section
3. **Source**: Select branch (e.g., `main`) and folder (e.g., `/learn` or `/ (root)`)
4. Save

### 2. File Structure

Place JavaScript files in one of these locations:

**Option A: In `learn/static/js/` (if using Hugo site)**
```
learn/
  static/
    js/
      sruja-viewer.js
      sruja-viewer.css
```

**Option B: In root `static/js/` directory**
```
static/
  js/
    sruja-viewer.js
    sruja-viewer.css
```

### 3. GitHub Pages URL

Once enabled, files are accessible at (using custom domain):
```
https://sruja.ai/static/js/sruja-viewer.js
https://sruja.ai/static/js/sruja-viewer.css
```

**Note**: Custom domain `sruja.ai` is already configured via `learn/CNAME` file.

### 4. HTML Template Usage

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: My System</title>
  <!-- Sruja Viewer CSS from sruja.ai -->
  <link rel="stylesheet" href="https://sruja.ai/static/js/sruja-viewer.css">
</head>
<body>
  <div id="sruja-app"></div>
  
  <!-- Cytoscape.js from unpkg CDN -->
  <script src="https://unpkg.com/cytoscape@3.27.0/dist/cytoscape.min.js"></script>
  <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
  
  <!-- Sruja Viewer from sruja.ai -->
  <script src="https://sruja.ai/static/js/sruja-viewer.js"></script>
  
  <script>
    SrujaViewer.init({
      container: '#sruja-app',
      data: './architecture.json'
    });
  </script>
</body>
</html>
```

## Versioning Strategy

### Option 1: Latest Version (Simple)
- Always use `main` branch
- URL: `https://sruja.ai/static/js/sruja-viewer.js`
- Updates automatically on push

### Option 2: Versioned (Recommended)
- Use Git tags/releases
- Create versioned directories:
  ```
  static/
    js/
      v1.0.0/
        sruja-viewer.js
      v1.1.0/
        sruja-viewer.js
      latest/
        sruja-viewer.js -> v1.1.0/sruja-viewer.js
  ```
- URL: `https://sruja.ai/static/js/v1.0.0/sruja-viewer.js`

### Option 3: GitHub Releases + jsDelivr
- Publish releases on GitHub
- Use jsDelivr CDN:
  ```
  https://cdn.jsdelivr.net/gh/sruja-ai/sruja-lang@v1.0.0/static/js/sruja-viewer.js
  ```

## Deployment

### Automatic (Recommended)
- GitHub Pages automatically deploys on push to main branch
- No additional setup needed

### Manual
- Use GitHub Actions to build and deploy
- Example workflow:
  ```yaml
  name: Deploy to GitHub Pages
  on:
    push:
      branches: [main]
  jobs:
    deploy:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - name: Build JS
          run: npm run build
        - name: Deploy
          uses: peaceiris/actions-gh-pages@v3
          with:
            github_token: ${{ secrets.GITHUB_TOKEN }}
            publish_dir: ./static
  ```

## Benefits

✅ **Free** - No cost
✅ **Easy Setup** - Just enable in repo settings
✅ **Version Controlled** - Files tracked in Git
✅ **Automatic Deployment** - Updates on push
✅ **HTTPS** - Secure by default
✅ **Fast** - Served from GitHub's CDN

## Limitations

⚠️ **Custom Domain** - Requires CNAME file (already have `learn/CNAME`)
⚠️ **Build Process** - If using build step, need GitHub Actions
⚠️ **Rate Limits** - GitHub Pages has rate limits (usually not an issue)

## Future Migration

When ready to scale, can migrate to:
- **jsDelivr** - Free CDN with GitHub integration
- **Cloudflare Pages** - Free, fast, better performance
- **Self-hosted CDN** - For enterprise use

## Testing

Test the CDN URL:
```bash
# Check if file is accessible
curl -I https://sruja.ai/static/js/sruja-viewer.js

# Should return 200 OK
```

## Example: Complete Setup

1. **Create files:**
   ```bash
   mkdir -p learn/static/js
   touch learn/static/js/sruja-viewer.js
   touch learn/static/js/sruja-viewer.css
   ```

2. **Enable GitHub Pages:**
   - Settings → Pages → Source: `main` branch, `/learn` folder

3. **Commit and push:**
   ```bash
   git add learn/static/js/
   git commit -m "Add Sruja Viewer JS library"
   git push
   ```

4. **Verify:**
   - Visit: `https://sruja.ai/static/js/sruja-viewer.js`
   - Should see the JavaScript file

5. **Use in HTML:**
   ```html
   <script src="https://sruja.ai/static/js/sruja-viewer.js"></script>
   ```

## Troubleshooting

**404 Not Found:**
- Check GitHub Pages is enabled
- Verify custom domain `sruja.ai` is configured (check `learn/CNAME`)
- Verify file path matches URL
- Wait a few minutes after enabling (deployment takes time)

**CORS Issues:**
- GitHub Pages serves with proper CORS headers
- If issues, check file is in correct location

**Cache Issues:**
- Browser may cache old version
- Use versioned URLs or cache-busting query params: `?v=1.0.0`

