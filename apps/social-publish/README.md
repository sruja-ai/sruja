# Social Publish

Automated social media posting scripts for Sruja releases.

## Overview

This app contains TypeScript scripts for posting release announcements to various social media platforms.

## Supported Platforms

- **Reddit** - Automated posting via snoowrap
- **LinkedIn** - Automated posting via API
- **Mastodon** - Automated posting via API
- **Bluesky** - Automated posting via AT Protocol
- **GitHub Discussions** - Automated discussion creation

## Usage

### From GitHub Actions

The scripts are automatically executed by the `social-publish.yml` workflow when a release is published.

### Local Development

```bash
# Install dependencies
cd apps/social-publish
npm install

# Set environment variables
export REDDIT_CLIENT_ID="..."
export REDDIT_CLIENT_SECRET="..."
# ... etc

# Run a script
npm run reddit
npm run linkedin
npm run mastodon
npm run bluesky
npm run github-discussion
```

## Environment Variables

Each script requires specific environment variables. See `docs/SOCIAL_PUBLISHING_SETUP.md` for detailed setup instructions.

## Dependencies

- `snoowrap` - Reddit API client
- `tsx` - TypeScript execution (dev dependency)
- `typescript` - TypeScript compiler (dev dependency)

All dependencies are managed in `package.json` and installed via the monorepo's workspace system.

