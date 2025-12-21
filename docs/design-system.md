# Design System Usage

This guide covers using the Sruja design system in TypeScript/React applications.

## Setup

1. **Import styles** in your app entry:
   ```typescript
   import '@sruja/ui/design-system/styles.css'
   ```

2. **Wrap root with ThemeProvider**:
   ```typescript
   import { ThemeProvider } from '@sruja/ui'
   
   <ThemeProvider>
     <App />
   </ThemeProvider>
   ```

## Using Design Tokens

### CSS Variables

Use CSS variables for colors, spacing, and other design tokens:

```css
.my-component {
  color: var(--color-text-primary);
  background: var(--color-background);
  padding: var(--spacing-md);
}
```

### Available Tokens

Tokens are defined in `packages/ui/src/design-system/tokens.ts`. Common variables:

- **Colors**: `--color-text-primary`, `--color-text-secondary`, `--color-background`, `--color-border`
- **Spacing**: `--spacing-xs`, `--spacing-sm`, `--spacing-md`, `--spacing-lg`
- **Brand**: `--color-brand-gradient`, `--gradient-primary`
- **Accents**: `--accent-primary`, `--accent-dim`

### Best Practices

1. **Use CSS variables**: Avoid hard-coded hex values or inline RGBA
2. **Prefer brand gradients**: Use `--color-brand-gradient` for brand fills, `--gradient-primary` for emphasis
3. **Respect dark mode**: Variables switch automatically based on `.dark` class or data attributes
4. **Use semantic tokens**: Prefer semantic names (`--color-text-primary`) over specific colors

## Dark Mode

The design system automatically handles dark mode via CSS variables. Ensure your app respects the dark mode class:

```html
<html class="dark">
  <!-- Dark mode styles applied automatically -->
</html>
```

Or use data attributes:
```html
<html data-theme="dark">
```

## TypeScript Tokens

For TypeScript usage, import tokens directly:

```typescript
import { tokens } from '@sruja/ui/design-system/tokens'

const primaryColor = tokens.colors.primary
```
