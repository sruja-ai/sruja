# Sruja Viewer

Unified viewer package with core engine and React UI, located in `apps/viewer`.

## Structure

```
apps/viewer/
  core/          # Cytoscape engine (viewer-core.js)
  app/           # React UI (viewer-app.js)
  dist/          # Build output
  index.html     # Dev server entry
```

## Development

```bash
cd apps/viewer
npm install
npm run dev
```

Opens `http://localhost:5173` with hot module reloading.

## Build

```bash
npm run build
```

Produces:
- `dist/viewer-core.js` - Core engine (includes Cytoscape)
- `dist/viewer-app.js` - React UI (requires React/ReactDOM from CDN)

## Usage in Go Exporter

The Go exporter uses these bundles in three modes:

1. **CDN Mode (default)**: `sruja export html file.sruja`
2. **Local Mode**: `sruja export html file.sruja --local -o dist/`
3. **Single-File Mode**: `sruja export html file.sruja --single-file > standalone.html`





