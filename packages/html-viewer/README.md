# HTML Viewer Components

TypeScript web components for the Sruja HTML export feature. These components provide interactive functionality for viewing architecture diagrams in exported HTML files.

## Components

- **SrujaNodeIndex** - Indexes and maps architecture nodes for search and filtering
- **SrujaSvgViewer** - Handles pan/zoom functionality for SVG diagrams
- **SrujaInfoPanel** - Displays node details when selected
- **SrujaViewer** - Main orchestrator component that coordinates all functionality

## Development

```bash
# Install dependencies
npm install

# Build TypeScript to JavaScript
npm run build

# Watch mode for development
npm run dev

# Clean build artifacts
npm run clean
```

## Build Process

1. TypeScript files in `src/` are compiled to `dist/`
2. Compiled files are bundled into a single `sruja-components.js` file
3. Bundle is copied to `pkg/export/html/embed/components/` for Go embedding
4. Go code embeds the bundle in generated HTML files

## Usage

Components are automatically initialized when included in the HTML template. The Go HTML exporter embeds the bundled components and initializes them on page load.
