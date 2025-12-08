# @sruja/ui

Shared UI components and design system for Sruja applications (Studio, Learn, etc.).

## Philosophy

This package provides:
1. **Core components** not available in Headless UI (Button, Header, Footer, Logo)
2. **Theme system** for dark/light mode
3. **Styled wrappers** around Headless UI components (optional convenience)
4. **Direct access** to Headless UI primitives (for advanced usage)
5. **Tailwind CSS** for styling with CSS variables for theming

You can use either:
- **Styled wrappers** (easier, matches design system)
- **Headless UI directly** (more control, requires styling)

## Installation

This package is part of the Sruja monorepo and is automatically linked via workspace configuration.

Add to your app's `package.json`:

```json
{
  "dependencies": {
    "@sruja/ui": "*"
  }
}
```

**Important**: You must import the design system CSS in your app:

```tsx
import '@sruja/ui/design-system/styles.css';
```

## Usage

### Theme Provider (Required for Dark/Light Mode)

Wrap your app with the ThemeProvider:

```tsx
import { ThemeProvider } from '@sruja/ui';
import '@sruja/ui/design-system/styles.css';

function App() {
  return (
    <ThemeProvider defaultMode="system">
      {/* Your app content */}
    </ThemeProvider>
  );
}
```

### Core Components

#### Button

```tsx
import { Button } from '@sruja/ui';

<Button variant="primary" size="md">Click me</Button>
<Button variant="outline" isLoading={true}>Loading...</Button>
```

#### Header & Footer

```tsx
import { Header, Footer, ThemeToggle } from '@sruja/ui';

<Header
  title="Sruja Studio"
  subtitle="Architecture Visualization Tool"
  rightContent={<ThemeToggle iconOnly />}
/>

<Footer
  leftContent={<span>© 2024 Sruja</span>}
  centerContent={<span>Architecture as Code</span>}
/>
```

#### Dialog, Menu, Popover

All components use Tailwind CSS classes with CSS variables for theme support:

```tsx
import { Dialog, Menu, Button } from '@sruja/ui';

<Dialog
  isOpen={isOpen}
  onClose={() => setIsOpen(false)}
  title="Confirm Action"
  footer={<Button onClick={handleConfirm}>OK</Button>}
>
  Content here
</Dialog>
```

### Styling with Tailwind CSS

All components use **Tailwind CSS** with **CSS variables** for theming:

- Components use Tailwind utility classes
- Colors come from CSS variables (`--color-primary`, `--color-background`, etc.)
- Dark mode works automatically via CSS variables
- You can override with custom classes using the `cn()` utility

```tsx
import { Button, cn } from '@sruja/ui';

<Button className={cn("custom-class", "another-class")}>
  Custom Styled Button
</Button>
```

### Available Headless UI Components

All Headless UI components are re-exported:

- **Dialog**: `Dialog`, `DialogPanel`, `DialogTitle`, `DialogDescription`
- **Menu**: `Menu`, `MenuButton`, `MenuItems`, `MenuItem`
- **Popover**: `Popover`, `PopoverButton`, `PopoverPanel`
- **Listbox**: `Listbox`, `ListboxButton`, `ListboxOptions`, `ListboxOption`
- **Disclosure**: `Disclosure`, `DisclosureButton`, `DisclosurePanel`
- **RadioGroup**: `RadioGroup`, `Radio`
- **Switch**: `Switch`, `SwitchGroup`, `SwitchLabel`
- **Transition**: `Transition`, `TransitionChild`

## Design System

### CSS Variables

All theme colors are available as CSS variables:

```css
--color-background
--color-surface
--color-border
--color-text-primary
--color-text-secondary
--color-primary
--color-error-500
/* ... and more */
```

### Dark Mode

Dark mode is automatically supported via CSS variables. The ThemeProvider toggles the `.dark` class on the root element.

## Development

```bash
# Build the package
npm run build

# Watch mode
npm run dev

# Lint
npm run lint
```

## Migration Notes

Components have been migrated from inline styles to Tailwind CSS:
- All styling uses Tailwind utility classes
- CSS variables provide theme support
- Dark mode works via CSS variable changes
- Components are more maintainable and consistent
