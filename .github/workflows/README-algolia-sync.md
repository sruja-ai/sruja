# Algolia Sync Workflow

This workflow automatically syncs your website content to Algolia search index.

## Setup

### 1. Add Algolia Write API Key Secret

1. Go to your GitHub repository
2. Navigate to **Settings** > **Secrets and variables** > **Actions**
3. Click **New repository secret**
4. Add the following secret:
   - **Name**: `ALGOLIA_WRITE_KEY`
   - **Value**: Your Algolia Admin API Key (get it from [Algolia Dashboard](https://www.algolia.com/account/api-keys/))

⚠️ **Important**: Use the **Admin API Key** (also called "Write API Key" in Algolia), not the search-only key. This key has write permissions needed to push data to the index.

### 2. Configure Algolia App ID (Optional)

The workflow uses a default App ID: `N6FUL0KI3V`

If you need to use a different App ID, you can set it as a repository variable:

1. Go to **Settings** > **Secrets and variables** > **Actions**
2. Click on the **Variables** tab
3. Click **New repository variable**
4. Add:
   - **Name**: `ALGOLIA_APP_ID`
   - **Value**: Your Algolia Application ID

If the variable is not set, the workflow will use the default value `N6FUL0KI3V`.

**Note**: The App ID is a public identifier and is safe to expose. It's the same across all environments.

## How It Works

1. **Triggers**:
   - Automatically on pushes to `main` branch when content files change
   - Manually via workflow dispatch (with environment selection)

2. **Process**:
   - Starts Astro dev server
   - Calls the `/generate-algolia.json` endpoint to generate data
   - Creates `algolia-data.json` file
   - Pushes data to Algolia using batch API
   - Uploads data file as artifact for debugging

3. **Environments**:
   - **Production**: Uses `sruja_docs` index, site URL `https://sruja.ai`
   - **Staging**: Uses `sruja_docs_staging` index, site URL `https://staging.sruja.ai`

## Manual Trigger

You can manually trigger the workflow:

1. Go to **Actions** tab in GitHub
2. Select **Sync Algolia Search Index**
3. Click **Run workflow**
4. Choose environment (production or staging)
5. Click **Run workflow**

## Monitoring

- Check the workflow run in the **Actions** tab
- View logs for each step
- Download the `algolia-data` artifact to inspect the generated data
- Check Algolia Dashboard to verify records were indexed

## Troubleshooting

### Workflow fails with "ALGOLIA_WRITE_KEY not set"

- Make sure you've added the secret in repository settings
- Verify the secret name is exactly `ALGOLIA_WRITE_KEY`

### Server fails to start

- Check Node.js version compatibility
- Verify all dependencies are installed
- Check workflow logs for specific errors

### Data generation fails

- Verify the `/generate-algolia.json` endpoint is accessible
- Check that content collections are properly configured
- Review Astro build logs

### Push to Algolia fails

- Verify the Write API Key (Admin API Key) is correct
- Check that the index name exists in Algolia
- Ensure the API key has write permissions (use Admin API Key, not search-only key)
- Review Algolia API response in workflow logs

## Index Configuration

Make sure your Algolia index is configured with appropriate settings:

- **Searchable attributes**: `title`, `content`, `summary`, `description`
- **Attributes for faceting**: `type`, `category`, `difficulty`, `tags`
- **Custom ranking**: Consider ranking by `weight` or `pubDate`

See [Algolia Documentation](https://www.algolia.com/doc/) for more configuration options.
