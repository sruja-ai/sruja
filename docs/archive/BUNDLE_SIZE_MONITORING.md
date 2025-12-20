# Bundle Size Monitoring

Sruja uses `size-limit` to monitor and enforce bundle size limits for frontend packages.

## Overview

Bundle size monitoring helps:
- **Prevent bloat**: Catch size increases before they become problems
- **Track changes**: See how bundle size changes over time
- **Optimize**: Identify opportunities for code splitting and tree-shaking

## Current Limits

### `@sruja/shared`
- **Limit**: 50 KB (gzipped)
- **Config**: `packages/shared/package.json`

### Other Packages
- Add `size-limit` configuration to `package.json` as needed

## Usage

### Check Bundle Size
```bash
# Check all packages
npm run size

# Check specific package
cd packages/shared && npm run size
```

### Analyze Bundle Size
```bash
# See what's contributing to bundle size
npm run size:why
```

### Update Limits
Edit `package.json` in the relevant package:
```json
{
  "size-limit": [
    {
      "path": "dist/**/*.js",
      "limit": "50 KB"
    }
  ]
}
```

## CI Integration

Bundle size checks should be added to CI workflows:

```yaml
- name: Check bundle size
  run: npm run size
```

## Best Practices

1. **Set Realistic Limits**: Base limits on actual usage and performance requirements
2. **Monitor Regularly**: Check bundle size in PRs
3. **Use Tree-Shaking**: Ensure packages support tree-shaking
4. **Code Splitting**: Split large features into separate bundles
5. **Lazy Loading**: Load features on demand

## Reducing Bundle Size

### Strategies

1. **Remove Unused Dependencies**: Use `npm run check:unused:deps`
2. **Tree-Shaking**: Use ES modules and avoid side effects
3. **Code Splitting**: Split routes and features
4. **Lazy Loading**: Load components on demand
5. **Optimize Imports**: Import only what you need

### Tools

- **size-limit**: Bundle size monitoring (already configured)
- **webpack-bundle-analyzer**: Visualize bundle contents
- **rollup-plugin-visualizer**: Analyze Rollup bundles
- **source-map-explorer**: Analyze source maps

## Monitoring

### Regular Checks
- Run `npm run size` before releases
- Monitor bundle size trends over time
- Set up alerts for significant increases

### Reporting
- Include bundle size in PR descriptions
- Document size changes in CHANGELOG
- Track size metrics in analytics

## References

- [size-limit documentation](https://github.com/ai/size-limit)
- [Webpack Bundle Analyzer](https://github.com/webpack-contrib/webpack-bundle-analyzer)
- [Bundle Size Best Practices](https://web.dev/reduce-javascript-payloads-with-code-splitting/)

