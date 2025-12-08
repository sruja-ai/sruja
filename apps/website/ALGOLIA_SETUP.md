# Algolia Search Configuration

This project uses Algolia for search functionality with separate indices for different environments.

## Environment-Specific Indices

The search automatically uses different Algolia indices based on the environment:

- **Development**: `sruja_docs_dev`
- **Staging**: `sruja_docs_staging`
- **Production**: `sruja_docs`

## Environment Detection

The environment is determined in the following order:

1. `PUBLIC_ENV` environment variable (explicit override)
2. `SITE_URL` containing "staging", "stage", or "stg" → staging
3. `NODE_ENV=production` → production
4. Default → development

## Configuration

### Local Development Setup

**Application ID is hardcoded** in `src/config/env.ts` (safe to expose, same across all environments).

**You only need to set the Search API Key** in `.env.local`:

1. **Create `.env.local` file** in `apps/website/` directory:

```bash
# Required: Algolia Search API Key
PUBLIC_ALGOLIA_SEARCH_API_KEY=966344a0eccf1872c34d1fee70ff7f7d

# Optional: Override default index name
# If not set, uses sruja_docs_dev for development
# PUBLIC_ALGOLIA_INDEX_NAME=sruja_docs_dev

# Optional: Explicitly set environment
# If not set, defaults to development
# PUBLIC_ENV=development
```

**Note:** The Application ID (`N6FUL0KI3V`) is already hardcoded in the code, so you don't need to set it in `.env.local`.

### Security Notes

**What's Safe to Expose:**
- ✅ **Application ID** - Public by design, safe to hardcode or expose
- ✅ **Search-Only API Key** - Designed for client-side use, read-only access

**What's NOT Safe:**
- ❌ **Admin API Key** - Never expose this! Only use server-side

**Best Practice:**
- Application ID can be hardcoded if it's the same across all environments
- Search API Key should be in environment variables (even though it's safe to expose, it's better practice)
- This allows different keys for different environments if needed

### Getting Your Credentials

1. Go to [Algolia Dashboard](https://www.algolia.com/account/api-keys/)
2. Copy your **Application ID** (safe to hardcode)
3. Create a **Search-Only API Key** (read-only, safe for client-side)
4. Add to `.env.local` or hardcode App ID in `env.ts`

### Production/Deployment Environment Variables

For deployment, set these in your CI/CD environment or hosting platform:

```bash
PUBLIC_ALGOLIA_APP_ID=your_app_id
PUBLIC_ALGOLIA_SEARCH_API_KEY=your_search_api_key
PUBLIC_ENV=production|staging|development  # Optional, auto-detected
```

### Build Commands

The build scripts automatically set the correct environment:

```bash
# Development build (uses sruja_docs_dev)
npm run build:dev

# Staging build (uses sruja_docs_staging)
npm run build:staging

# Production build (uses sruja_docs)
npm run build:prod
```

## Setting Up Algolia Indices

1. **Create indices in Algolia Dashboard:**
   - `sruja_docs_dev` - for development
   - `sruja_docs_staging` - for staging
   - `sruja_docs` - for production

2. **Configure index settings** (same for all indices):
   - Add searchable attributes: `title`, `description`, `content`
   - Configure ranking and relevance settings
   - Set up synonyms if needed

3. **Index your content:**
   - Use Algolia's indexing API or CLI
   - Ensure URLs are environment-specific (e.g., `http://localhost:4321/docs/...` for dev)

## Debugging

In development mode, the console will log which index is being used:

```
[Algolia] Using index: sruja_docs_dev (env: development)
[Algolia Search] Initialized with index: sruja_docs_dev
```

## Open Source Branding

This project uses Algolia's open source support, which requires displaying "Search by Algolia" branding. The branding is automatically shown when the search dialog is open.

