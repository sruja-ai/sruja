# Sruja Astro Website

This is the new Astro-based website for Sruja, replacing the Docusaurus setup.

## Features

- **Astro** for static site generation
- **React Islands** for interactive components (Studio)
- **UI Components** from `@sruja/ui` package
- **Content Collections** for docs, blog, courses, tutorials
- **Studio Core** as a separate package for the diagram editor

## Structure

```
apps/website/
├── src/
│   ├── pages/          # Astro pages (routes)
│   ├── layouts/        # Page layouts
│   ├── components/     # React components (islands)
│   ├── content/        # Content Collections (docs, blog, etc.)
│   ├── config/         # Configuration
│   └── utils/          # Utilities
├── public/             # Static assets
└── astro.config.mjs   # Astro configuration
```

## Development

```bash
cd apps/website
npm install

# Create .env.local file with Algolia credentials (see ALGOLIA_SETUP.md)
# PUBLIC_ALGOLIA_APP_ID=your_app_id
# PUBLIC_ALGOLIA_SEARCH_API_KEY=your_search_api_key

npm run dev
```

**Note:** Search functionality requires Algolia credentials in `.env.local`. See [ALGOLIA_SETUP.md](./ALGOLIA_SETUP.md) for details.

## Building

```bash
npm run build          # Production build
npm run build:dev      # Development build
npm run build:staging  # Staging build
npm run build:prod     # Production build
```

## UI Components

All UI components from `@sruja/ui` are available as React islands:

```astro
---
import { Button } from '@sruja/ui';
---

<Button client:load variant="primary">Click me</Button>
```

## Studio Integration

The Studio is integrated as a React island at `/studio`:

```astro
---
import StudioApp from '../components/StudioApp';
---

<StudioApp client:load />
```

This ensures React and heavy dependencies only load on the Studio page, keeping other pages fast.



