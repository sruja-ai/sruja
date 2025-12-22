# Sentry Quick Start Guide

## What You Need to Do (5 Minutes)

### Step 1: Create Sentry Account & Project (2 minutes)

1. **Sign up**: Go to https://sentry.io/signup/
   - Use the free Developer plan (5,000 errors/month - usually enough)
   
2. **Create Project**:
   - Click "Create Project"
   - Select **"JavaScript"** or **"Astro"** as platform
   - Name it: `sruja-website`
   - Click "Create Project"

3. **Copy Your DSN**:
   - After creating, Sentry shows you a DSN
   - It looks like: `https://xxxxx@xxxxx.ingest.sentry.io/xxxxx`
   - **Copy this entire DSN** (you'll need it in Step 2)

### Step 2: Add DSN to GitHub Secrets (1 minute)

1. Go to: `https://github.com/sruja-ai/sruja/settings/secrets/actions`
2. Click **"New repository secret"**
3. Add:
   - **Name**: `PUBLIC_SENTRY_DSN`
   - **Value**: Paste your DSN from Step 1
4. Click **"Add secret"**

**Note**: The same DSN is used for both staging and production. Events are automatically tagged with `environment: "staging"` or `environment: "production"` so you can filter them in Sentry. If you prefer separate DSNs, see the full guide.

### Step 3: Deploy (Automatic)

That's it! The next deployment will automatically:
- ✅ Include Sentry error tracking
- ✅ Tag errors with environment (staging/production)
- ✅ Track releases automatically
- ✅ Capture session replays on errors

### Step 4: Verify (1 minute)

After deployment:
1. Visit your staging or production site
2. Go to your Sentry dashboard: https://sentry.io/
3. You should see your project
4. Errors will appear here automatically when they occur

## Optional: Source Maps (Better Debugging)

For better error debugging with source maps, add 3 GitHub secrets:

1. **Create Auth Token**:
   - Go to: https://sentry.io/settings/account/api/auth-tokens/
   - Click "Create New Token"
   - Name: `sruja-website-sourcemaps`
   - Scope: Check `project:releases`
   - Copy the token (you won't see it again!)

2. **Find Org & Project Names**:
   - Look at your Sentry URL: `https://sentry.io/organizations/{ORG}/projects/{PROJECT}/`
   - `{ORG}` = your organization slug
   - `{PROJECT}` = your project slug

3. **Add 3 GitHub Secrets**:
   - `SENTRY_AUTH_TOKEN` = your auth token
   - `SENTRY_ORG` = your organization slug
   - `SENTRY_PROJECT` = your project slug

**That's it!** Source maps will be uploaded automatically on each deployment.

## What Gets Tracked

✅ **All JavaScript errors**  
✅ **Unhandled promise rejections**  
✅ **Performance metrics** (page loads, API calls)  
✅ **Session replays** (on errors + sampled sessions)  
✅ **Release tracking** (which deployment caused errors)

## Environment Configuration

- **Staging**: All errors tracked, 50% performance sampling
- **Production**: All errors tracked, 10% performance sampling (to reduce overhead)
- **Development**: Errors only if `PUBLIC_SENTRY_DEBUG=true` (to avoid noise)

## Cost

**Free Tier Includes**:
- 5,000 errors/month
- 10,000 performance units/month  
- 1,000 session replays/month

This is usually enough for small to medium projects. You can upgrade later if needed.

## Need Help?

See the full guide: `docs/SENTRY_SETUP.md`

