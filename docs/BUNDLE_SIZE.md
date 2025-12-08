# Bundle Size Monitoring

## Overview

Bundle size monitoring is configured using `size-limit` to track and prevent bundle bloat in TypeScript packages.

## Current Limits

### @sruja/shared
- **Limit**: 100 KB
- **Monitored**: All TypeScript source files
- **Purpose**: Keep shared utilities lightweight

### @sruja/ui
- **Limit**: 500 KB
- **Monitored**: All TypeScript/TSX source files
- **Purpose**: Keep UI component library reasonable

## Running Bundle Size Checks

```bash
# Check bundle sizes
npm run size

# See detailed breakdown
npm run size:why
```

## CI Integration

Bundle size checks run automatically in CI:
1. Packages are built
2. Size limits are checked
3. CI continues even if limits exceeded (for now)
4. Results are visible in CI logs

## Adjusting Limits

Edit `.size-limit.json` or package-specific `size-limit` config in `package.json`:

```json
{
  "size-limit": [
    {
      "path": "dist/**/*.js",
      "limit": "200 KB"
    }
  ]
}
```

## Best Practices

1. **Monitor regularly**: Check sizes before major releases
2. **Set realistic limits**: Based on current size + 20% buffer
3. **Investigate increases**: Use `size:why` to understand what's growing
4. **Optimize imports**: Use tree-shaking friendly imports
5. **Code splitting**: Split large features into separate bundles

## Troubleshooting

If bundle size exceeds limits:
1. Run `npm run size:why` to see what's large
2. Check for unnecessary dependencies
3. Look for duplicate code
4. Consider code splitting
5. Review imports for tree-shaking opportunities




