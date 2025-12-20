# Task 2.1: HTML Exporter (JSON → HTML)

**Priority**: 🟡 High (User-facing)
**Technology**: Go
**Estimated Time**: 1-2 days
**Dependencies**: Task 1.1 (needs JSON structure)

## Files to Create

* `pkg/export/html/html.go` - HTML exporter
* `pkg/export/html/template.go` - HTML template

## Implementation

```go
// pkg/export/html/html.go
type Exporter struct {
    JSONPath string // Path to JSON file (relative or absolute)
    CDN      bool   // Use CDN for JS/CSS
}

func (e *Exporter) Export(jsonData []byte, outputPath string) error {
    // Generate minimal HTML
    html := generateHTML(e.JSONPath, e.CDN)
    return os.WriteFile(outputPath, []byte(html), 0644)
}
```

## HTML Template

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Architecture: {{.Name}}</title>
  <!-- Sruja Viewer from sruja.ai (GitHub Pages with custom domain) -->
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
      data: './{{.JSONFile}}'
    });
  </script>
</body>
</html>
```

**Note**: Using `sruja.ai` custom domain (GitHub Pages):
* Files in `learn/static/js/` are served via GitHub Pages
* **URL**: `https://sruja.ai/static/js/sruja-viewer.js` (custom domain)
* Custom domain `sruja.ai` already configured (see `learn/CNAME`)

## Acceptance Criteria

* [ ] Generates valid HTML
* [ ] Links to JSON file correctly
* [ ] Supports CDN mode
* [ ] Supports embedded JSON mode (optional)
