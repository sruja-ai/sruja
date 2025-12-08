# Design System Usage

- Import `@sruja/ui/design-system/styles.css` in app entry.
- Wrap root with `ThemeProvider` from `@sruja/ui`.
- Use CSS variables (e.g., `var(--color-text-primary)`) and tokens from `packages/ui/src/design-system/tokens.ts`.
- Prefer `--color-brand-gradient` for brand fills and `--gradient-primary` for emphasis.
- Avoid hard-coded color hex values and inline RGBA; use `--accent-primary`/`--accent-dim` or existing variables.
- Respect dark/light via data attributes or `.dark` class; variables switch automatically.
