# LikeC4 Version Information

## Current Versions

Sruja uses the latest versions of LikeC4 packages:

- **@likec4/diagram**: `^1.46.0` (for React diagram rendering components)
- **@likec4/core**: `^1.46.0` (dependency of @likec4/diagram, provides types/utilities)

**Note**: We don't use LikeC4's parser - we use our own Sruja parser that supports LikeC4 syntax and generate LikeC4-compatible JSON for `@likec4/diagram` to render.

These versions were current as of January 2025.

## Installation

After updating package.json, run:

```bash
npm install
```

This will install the latest compatible versions of LikeC4 packages (^1.46.0 means >=1.46.0 <2.0.0).

## Usage

We use LikeC4 packages only for:
- **@likec4/diagram**: React components to render our LikeC4-compatible JSON
- **@likec4/core**: Types and utilities (dependency of diagram package)

We **don't use**:
- LikeC4's parser (we have our own)
- LikeC4's CLI (we use our own export commands)

## Checking for Updates

To check for newer versions:

```bash
npm outdated @likec4/diagram @likec4/core
```

Or visit:
- https://www.npmjs.com/package/@likec4/diagram
- https://www.npmjs.com/package/@likec4/core

## Compatibility

These versions are compatible with:
- React 19.x
- Node.js 18+ 
- TypeScript 5.x

Make sure your project uses compatible versions of these dependencies.
