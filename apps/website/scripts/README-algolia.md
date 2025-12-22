# Algolia Data Generation

This script generates an Algolia import data file from all content collections.

## Usage

### Option 1: Run as Astro Script (Recommended)

Since this script uses Astro's `getCollection` API, it needs to run in the Astro context. The easiest way is to create a temporary Astro page:

1. Create a file `src/pages/generate-algolia.json.ts`:

```typescript
import type { APIRoute } from "astro";
import { generateAlgoliaData } from "../../scripts/generate-algolia-data.mts";

export const GET: APIRoute = async () => {
  await generateAlgoliaData();
  return new Response(JSON.stringify({ success: true }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
};
```

2. Run the dev server: `npm run dev`
3. Visit: `http://localhost:4321/generate-algolia.json`
4. The file `algolia-data.json` will be created in the website root

### Option 2: Use Astro Build Integration

Create `src/integrations/algolia-generator.ts`:

```typescript
import type { AstroIntegration } from "astro";
import { generateAlgoliaData } from "../../scripts/generate-algolia-data.mts";

export default function algoliaGenerator(): AstroIntegration {
  return {
    name: "algolia-generator",
    hooks: {
      "build:done": async () => {
        await generateAlgoliaData();
      },
    },
  };
}
```

Then add to `astro.config.mjs`:

```javascript
import algoliaGenerator from "./src/integrations/algolia-generator";

export default defineConfig({
  integrations: [algoliaGenerator()],
  // ... rest of config
});
```

### Option 3: Manual Import (Alternative)

If the above don't work, you can manually import and run:

```typescript
// In an Astro page or component
import { generateAlgoliaData } from "../scripts/generate-algolia-data.mts";

// Call during page load or build
await generateAlgoliaData();
```

## Importing to Algolia

Once you have `algolia-data.json`:

1. **Using Algolia CLI:**

   ```bash
   algolia import -a <APP_ID> -k <ADMIN_API_KEY> -n <INDEX_NAME> -f algolia-data.json
   ```

2. **Using Algolia Dashboard:**
   - Go to Algolia Dashboard > Search > Indices
   - Select or create your index
   - Click "Import" and upload `algolia-data.json`

3. **Using Algolia API:**
   ```bash
   curl -X POST \
     "https://<APP_ID>-dsn.algolia.net/1/indexes/<INDEX_NAME>/batch" \
     -H "X-Algolia-Application-Id: <APP_ID>" \
     -H "X-Algolia-API-Key: <ADMIN_API_KEY>" \
     -H "Content-Type: application/json" \
     --data-binary @algolia-data.json
   ```

## Output Format

The generated JSON file contains an array of records with:

- `objectID`: Unique identifier (required by Algolia)
- `title`: Page title
- `content`: Plain text content (markdown stripped)
- `url`: Full URL to the page
- `type`: Content type (documentation, blog, course, tutorial, challenge)
- `category`: Sub-category (e.g., "concepts", course name)
- `summary`: Short summary
- `description`: Full description
- `difficulty`: Difficulty level (beginner, intermediate, advanced)
- `tags`: Array of tags
- `weight`: Sort weight
- `pubDate`: Publication date (for blog posts)
- `authors`: Author information (for blog posts)

## Customization

You can customize the script by:

- Changing `SITE_URL` via environment variable
- Modifying the `stripMarkdown` function to preserve certain formatting
- Adding additional metadata fields
- Filtering which collections to include
