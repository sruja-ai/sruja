# Sentry Setup Guide

This guide explains how to set up Sentry for error tracking and monitoring in the Sruja website.

## What You Need to Do

### 1. Create Sentry Account and Project

1. **Sign up for Sentry** (if you don't have an account)
   - Go to https://sentry.io/signup/
   - Choose the free Developer plan (sufficient for most use cases)
   - Or use Team/Pro plan for more features

2. **Create a Project**
   - After signing up, create a new project
   - **Platform**: Select "Astro" or "JavaScript"
   - **Project Name**: `sruja-website` (or your preferred name)
   - **Organization**: Create or select your organization

3. **Get Your DSN**
   - After creating the project, Sentry will show you a DSN (Data Source Name)
   - It looks like: `https://xxxxx@xxxxx.ingest.sentry.io/xxxxx`
   - **Copy this DSN** - you'll need it for the next steps

### 2. Configure GitHub Secrets

**Option A: Same DSN for Staging and Production (Recommended)**

Using the same DSN is simpler and recommended because:
- Events are automatically tagged with `environment: "staging"` or `environment: "production"`
- You can filter by environment in the Sentry UI
- One secret to manage
- Unified view of all errors

1. Go to your GitHub repository: `https://github.com/sruja-ai/sruja`
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add the following secret:
   - **Name**: `PUBLIC_SENTRY_DSN`
   - **Value**: Your Sentry DSN (from step 1.3)
   - Click **Add secret**

**Option B: Separate DSNs for Staging and Production**

If you prefer complete isolation between environments:

1. Create **two separate Sentry projects** (one for staging, one for production)
2. Get DSNs for both projects
3. Add two GitHub secrets:
   - **Name**: `PUBLIC_SENTRY_DSN_STAGING`
   - **Value**: Staging project DSN
   - **Name**: `PUBLIC_SENTRY_DSN_PRODUCTION`
   - **Value**: Production project DSN
4. Update the workflows to use environment-specific secrets (contact maintainers if needed)

### 3. (Optional) Configure Source Maps

For better error debugging with source maps, you need to add 3 GitHub secrets:

#### Step 1: Create Sentry Auth Token

1. **Go to Sentry Auth Tokens**:
   - Visit: https://sentry.io/settings/account/api/auth-tokens/
   - Or: Sentry Dashboard → Settings (gear icon) → Account → Auth Tokens

2. **Create New Token**:
   - Click **"Create New Token"**
   - **Name**: `sruja-website-sourcemaps` (or any name you prefer)
   - **Scopes**: Check `project:releases` (required for source maps)
   - Click **"Create Token"**
   - **⚠️ IMPORTANT**: Copy the token immediately - you won't be able to see it again!

#### Step 2: Find Your Sentry Organization and Project Names

1. **Find Organization Name**:
   - Look at your Sentry URL: `https://sentry.io/organizations/{ORG_NAME}/`
   - The `{ORG_NAME}` is your organization slug
   - Or: Go to Settings → Organization Settings → General → Organization Slug

2. **Find Project Name**:
   - Look at your Sentry URL: `https://sentry.io/organizations/{ORG_NAME}/projects/{PROJECT_NAME}/`
   - The `{PROJECT_NAME}` is your project slug
   - Or: Go to your project → Settings → General → Project Slug

#### Step 3: Add GitHub Secrets

Go to: `https://github.com/sruja-ai/sruja/settings/secrets/actions`

Add these 3 secrets:

1. **SENTRY_AUTH_TOKEN**
   - **Name**: `SENTRY_AUTH_TOKEN`
   - **Value**: The auth token you created in Step 1

2. **SENTRY_ORG**
   - **Name**: `SENTRY_ORG`
   - **Value**: Your organization slug (from Step 2.1)

3. **SENTRY_PROJECT**
   - **Name**: `SENTRY_PROJECT`
   - **Value**: Your project slug (from Step 2.2)

**Example:**
- If your Sentry URL is: `https://sentry.io/organizations/my-org/projects/sruja-website/`
- Then:
  - `SENTRY_ORG` = `my-org`
  - `SENTRY_PROJECT` = `sruja-website`

#### What This Enables

Once these secrets are added, the deployment workflows will automatically:
- ✅ Upload source maps to Sentry on each deployment
- ✅ Enable better error debugging (see original source code in error stack traces)
- ✅ Track releases with source maps attached
- ✅ Staging releases: `staging-{commit-sha}`
- ✅ Production releases: `{version-tag}` (e.g., `v1.2.3`)

### 4. (Optional) Set Up Alerts

1. **Go to Sentry Project Settings** → **Alerts**
2. **Create Alert Rules**:
   - **New Issue Alert**: Get notified when new errors occur
   - **Regression Alert**: Get notified when resolved issues reappear
   - **Spike Alert**: Get notified when error rate spikes

3. **Configure Notification Channels**:
   - Email notifications (default)
   - Slack integration (optional)
   - PagerDuty integration (optional)

### 5. Verify Setup

After deploying:

1. **Check Sentry Dashboard**:
   - Go to your Sentry project dashboard
   - You should see events appearing (if any errors occur)

2. **Test Error Tracking**:
   - Visit your staging/production site
   - Trigger a test error (if you have a test endpoint)
   - Check Sentry dashboard to see if the error appears

## Configuration Details

### Environment-Specific Settings

The Sentry integration is configured with environment-specific settings:

- **Development**: 
  - Errors only sent if `PUBLIC_SENTRY_DEBUG=true`
  - 100% trace sampling
  - 100% session replay sampling

- **Staging**:
  - All errors sent
  - 50% trace sampling
  - 50% session replay sampling
  - 100% error replay sampling

- **Production**:
  - All errors sent
  - 10% trace sampling (to reduce overhead)
  - 10% session replay sampling
  - 100% error replay sampling

### What Gets Tracked

- **Errors**: All JavaScript errors, unhandled promise rejections
- **Performance**: Page load times, API calls, user interactions
- **Session Replay**: User sessions (sampled) and all error sessions
- **Release Tracking**: Automatic release tracking tied to deployments

### What Gets Filtered

The following errors are automatically filtered out:
- Browser extension errors
- Network errors (often not actionable)
- ResizeObserver errors (common, usually harmless)
- Development errors (unless `PUBLIC_SENTRY_DEBUG=true`)

## Troubleshooting

### Errors Not Appearing in Sentry

1. **Check DSN is set correctly**:
   - Verify `PUBLIC_SENTRY_DSN` secret is set in GitHub
   - Check build logs to ensure DSN is being passed

2. **Check Sentry Dashboard**:
   - Go to Settings → Projects → Your Project → Client Keys (DSN)
   - Verify the DSN matches what you set in GitHub secrets

3. **Check Browser Console**:
   - Open browser DevTools → Console
   - Look for Sentry initialization messages
   - Check for any Sentry-related errors

### Source Maps Not Working

1. **Verify Sentry Auth Token**:
   - Check `SENTRY_AUTH_TOKEN` is set in GitHub secrets
   - Verify token has `project:releases` scope

2. **Check Release Tracking**:
   - Verify `SENTRY_RELEASE` is set during build
   - Check Sentry → Releases to see if releases are being created

## Cost Considerations

### Free Tier (Developer Plan)
- **5,000 errors/month**
- **10,000 performance units/month**
- **1,000 session replays/month**
- Usually sufficient for small to medium projects

### Paid Plans
- **Team Plan**: $26/month - More errors, performance units, and replays
- **Business Plan**: $80/month - Advanced features, custom integrations

## Best Practices

1. **Use Same DSN for Staging and Production (Recommended)**
   - Events are automatically tagged with `environment: "staging"` or `environment: "production"`
   - Filter by environment in Sentry UI: `environment:production` or `environment:staging`
   - Easier to compare and debug across environments
   - One secret to manage (`PUBLIC_SENTRY_DSN`)
   - If you need separate DSNs, use `PUBLIC_SENTRY_DSN_STAGING` and `PUBLIC_SENTRY_DSN_PRODUCTION` secrets

2. **Set Up Alert Rules**
   - Get notified of critical errors immediately
   - Set up spike alerts for sudden error increases

3. **Review and Resolve Issues Regularly**
   - Check Sentry dashboard weekly
   - Prioritize high-impact errors
   - Use Sentry's issue grouping to manage similar errors

4. **Use Release Tracking**
   - Already configured in workflows
   - Helps identify which deployment introduced errors

## Integration with PostHog

Sentry and PostHog work together:
- **Sentry**: Error tracking, performance monitoring, debugging
- **PostHog**: Product analytics, user behavior, feature flags

Both are configured to use the same environment tags, so you can correlate errors with user behavior.

