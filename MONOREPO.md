# Sruja Monorepo

This repository is organized as a monorepo using [Turborepo](https://turbo.build/repo) for managing multiple packages and applications.

## Structure

```
sruja-lang/
├── apps/
│   ├── website/        # Astro-based website (docs, courses, tutorials, blog)
│   ├── architecture-visualizer/ # The architecture visualization tool
│   ├── storybook/      # Storybook for component documentation
│   └── social-publish/ # Social media publishing scripts
├── packages/
│   ├── shared/         # Shared TypeScript utilities and types
│   ├── ui/             # UI component library
│   ├── viewer/         # Viewer library
│   └── html-viewer/   # HTML viewer components
├── cmd/                # Go CLI commands
├── pkg/                # Go packages (language, engine, export, etc.)
├── package.json        # Root package.json with Turborepo
└── turbo.json         # Turborepo configuration
```

## Getting Started

### Prerequisites

- Node.js >= 18.0.0
- Go >= 1.25
- npm >= 10.2.4

### Installation

```bash
# Install all dependencies
npm install

# This will install dependencies for all apps and packages
```

### Development

```bash
# Run all apps in development mode
npm run dev

# Run a specific app
npm run dev --filter=@sruja/website
npm run dev --filter=@sruja/studio-core
npm run dev --filter=@sruja/viewer-core

# Build all packages and apps
npm run build

# Run linting across all packages
npm run lint

# Run tests
npm run test
```

## Apps

### `@sruja/website`

Astro-based website with documentation, courses, tutorials, and blog posts.

- **Location**: `apps/website/`
- **Tech**: Astro, React, TypeScript, Tailwind CSS
- **Dev**: `cd apps/website && npm run dev`
- **Build**: `cd apps/website && npm run build`

### `@sruja/studio-core`

TypeScript/React application for visualizing and editing Sruja architecture files.

- **Location**: `apps/studio-core/`
- **Tech**: React, TypeScript, Vite
- **Dev**: `cd apps/studio-core && npm run dev`
- **Build**: `cd apps/studio-core && npm run build`

### `@sruja/viewer-core`

Viewer application for rendering architecture diagrams.

- **Location**: `apps/viewer-core/`
- **Tech**: React, TypeScript, Vite, Cytoscape.js
- **Dev**: `cd apps/viewer-core && npm run dev`
- **Build**: `cd apps/viewer-core && npm run build`

### `@sruja/vscode-extension`

VS Code extension providing language support for Sruja.

- **Location**: `apps/vscode-extension/`
- **Tech**: TypeScript, VS Code API
- **Build**: `cd apps/vscode-extension && npm run build`

## Packages

### `@sruja/shared`

Shared TypeScript utilities, types, and helpers used across frontend applications.

- **Location**: `packages/shared/`
- **Usage**: Import with `@sruja/shared` in any app
- **Build**: `cd packages/shared && npm run build`

### `@sruja/ui`

Shared UI component library with design system.

- **Location**: `packages/ui/`
- **Usage**: Import with `@sruja/ui` in any app
- **Build**: `cd packages/ui && npm run build`
- **Tech**: React, TypeScript, Tailwind CSS

### `@sruja/viewer`

Core viewer library for rendering Sruja architecture diagrams using Cytoscape.js.

- **Location**: `packages/viewer/`
- **Usage**: Import with `@sruja/viewer` in any app
- **Build**: `cd packages/viewer && npm run build`
- **Tech**: TypeScript, Cytoscape.js
- **Purpose**: Shared visualization library used by website and studio apps

### `@sruja/html-viewer`

HTML viewer components for exported HTML files.

- **Location**: `packages/html-viewer/`
- **Usage**: Bundled components for HTML export
- **Build**: `cd packages/html-viewer && npm run build`

## Go CLI

The Go CLI remains at the repository root and is not part of the Turborepo workspace. It can be built and run independently:

```bash
# Build CLI
go build -o ./bin/sruja ./cmd/sruja

# Run CLI
./bin/sruja --help
```

## Deployment

The website is deployed to multiple environments:
- **Production**: `https://sruja.ai/` (via `deploy-astro-website.yml`)
- **Staging**: `https://staging.sruja.ai/` (via `deploy-astro-website.yml`)
- **Dev**: `https://dev.sruja.ai/` (via `dev-website.yml`)

Deployment workflows automatically:
1. Build the Astro website
2. Deploy to appropriate GitHub Pages repository
3. Trigger on push to `main` branch or manual workflow dispatch

## Turborepo Features

- **Parallel Execution**: Tasks run in parallel across packages
- **Caching**: Build outputs are cached for faster subsequent builds
- **Task Dependencies**: Automatically handles dependencies between packages
- **Filtering**: Run tasks on specific packages with `--filter`

## Adding New Packages

1. Create directory in `apps/` or `packages/`
2. Add `package.json` with appropriate name (`@sruja/package-name`)
3. Add scripts: `build`, `dev`, `lint`, `test`, `clean`
4. Turborepo will automatically detect it

## Workspace Management

Turborepo uses npm workspaces. All packages are linked automatically when you run `npm install` at the root.

