# Sentry Setup Checklist

## ✅ What's Already Done (Code Implementation)

- ✅ Sentry SDK installed (`@sentry/react` v10.32.1)
- ✅ Sentry initialization component created
- ✅ Environment-specific configuration (staging vs production)
- ✅ Error tracking integration (errors sent to both PostHog and Sentry)
- ✅ Performance monitoring configured
- ✅ Session replay configured
- ✅ Release tracking configured
- ✅ Workflow updated to include Sentry DSN
- ✅ Error filtering (browser extensions, network errors, etc.)

## 📋 What YOU Need to Do

### 1. Create Sentry Account & Project (5 minutes)

1. **Sign up at**: https://sentry.io/signup/
   - Choose **Developer Plan** (free, 5,000 errors/month)
   - Or choose Team/Pro if you need more

2. **Create a Project**:
   - Platform: **JavaScript** or **Astro**
   - Project Name: `sruja-website` (or your choice)
   - Organization: Create or select one

3. **Get Your DSN**:
   - After creating project, Sentry shows you a DSN
   - Format: `https://xxxxx@xxxxx.ingest.sentry.io/xxxxx`
   - **Copy this DSN** - you'll need it next

### 2. Add DSN to GitHub Secrets (2 minutes)

1. Go to: `https://github.com/sruja-ai/sruja/settings/secrets/actions`

2. Click **"New repository secret"**

3. Add:
   - **Name**: `PUBLIC_SENTRY_DSN`
   - **Value**: Your DSN from step 1
   - Click **"Add secret"**

### 3. (Optional) Source Maps Setup (5 minutes)

For better error debugging:

1. **Create Sentry Auth Token**:
   - Sentry → Settings → Account → Auth Tokens
   - Click "Create New Token"
   - Name: `github-actions`
   - Scopes: Check `project:releases`
   - Click "Create Token"
   - **Copy the token** (you won't see it again!)

2. **Get Your Organization Slug**:
   - Sentry → Settings → Organization Settings
   - Copy your organization slug (e.g., `my-org`)

3. **Add to GitHub Secrets**:
   - Go to GitHub Secrets
   - Add `SENTRY_AUTH_TOKEN`: Your auth token
   - Add `SENTRY_ORG`: Your organization slug
   - Add `SENTRY_PROJECT`: Your project slug (usually `sruja-website`)
   - Click "Add secret" for each

### 4. Verify Setup

After the next deployment:

1. **Check Sentry Dashboard**: https://sentry.io/
2. **Visit your site** (staging or production)
3. **Check Sentry** - you should see your project
4. Errors will automatically appear when they occur

## 🎯 Quick Reference

**GitHub Secrets Needed**:
- ✅ `PUBLIC_SENTRY_DSN` - **REQUIRED** (your Sentry DSN)
- ⚠️ `SENTRY_AUTH_TOKEN` - **OPTIONAL** (for source maps upload)
- ⚠️ `SENTRY_ORG` - **OPTIONAL** (your Sentry organization slug, needed for source maps)
- ⚠️ `SENTRY_PROJECT` - **OPTIONAL** (your Sentry project slug, defaults to `sruja-website`)

**Sentry Dashboard**: https://sentry.io/

**Documentation**: 
- Quick Start: `docs/SENTRY_QUICK_START.md`
- Full Guide: `docs/SENTRY_SETUP.md`

## 🔍 How It Works

Once you add the DSN:

1. **Staging deployments** will:
   - Track all errors to Sentry
   - Tag with `environment: staging`
   - Release: `staging-{commit-sha}`

2. **Production deployments** will:
   - Track all errors to Sentry
   - Tag with `environment: production`
   - Release: `{version-tag}` (e.g., `v1.2.3`)

3. **Errors are automatically**:
   - Sent to both PostHog (analytics) and Sentry (error tracking)
   - Tagged with environment
   - Grouped and deduplicated by Sentry
   - Include stack traces and context

## 💰 Cost

**Free Tier (Developer Plan)**:
- 5,000 errors/month
- 10,000 performance units/month
- 1,000 session replays/month

**Usually sufficient** for small to medium projects. Upgrade if you exceed limits.

## 🚀 Next Steps After Setup

1. **Set up alerts** (optional):
   - Sentry → Alerts → Create Alert Rules
   - Get notified of new errors, regressions, spikes

2. **Configure integrations** (optional):
   - Slack notifications
   - Email alerts
   - PagerDuty (for critical errors)

3. **Review errors regularly**:
   - Check Sentry dashboard weekly
   - Prioritize high-impact errors
   - Use issue grouping to manage similar errors

---

**That's it!** Once you add the DSN secret, Sentry will start working automatically on the next deployment.

