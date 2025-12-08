# Structured Logging Migration

## Overview

Migrated from `console.log` to structured logging using the enhanced `@sruja/shared` logger utility.

## Enhanced Logger Features

### Structured Logging
- **Timestamps**: ISO 8601 format
- **Service Context**: Optional service name for all logs
- **Structured Context**: JSON-formatted context objects
- **Production Mode**: JSON output in production, human-readable in development
- **Log Levels**: `debug`, `info`, `warn`, `error`

### Example Usage

```typescript
import { logger } from '@sruja/shared';

// Set service name (optional)
logger.setService('my-service');

// Simple message
logger.info('Operation completed');

// With context
logger.info('Post published successfully', {
  postId: '123',
  platform: 'bluesky',
  timestamp: new Date().toISOString(),
});

// Error with context
logger.error('Failed to publish', {
  platform: 'github',
  error: sanitizedError,
  retryCount: 3,
});
```

### Output Format

**Development:**
```
2024-12-07T12:00:00.000Z [INFO] [bluesky-publish] Post published successfully
```

**Production (JSON):**
```json
{
  "timestamp": "2024-12-07T12:00:00.000Z",
  "level": "info",
  "service": "bluesky-publish",
  "message": "Post published successfully",
  "context": {
    "postId": "123",
    "platform": "bluesky"
  }
}
```

## Migration Summary

### ✅ Migrated to Structured Logging

**Apps:**
- `apps/social-publish/src/bluesky_post.ts` - Uses logger with context
- `apps/social-publish/src/github_discussion.ts` - Uses logger with context
- `apps/social-publish/src/linkedin_post.ts` - Uses logger with context
- `apps/social-publish/src/reddit_post.ts` - Uses logger with context
- `apps/social-publish/src/mastodon_post.ts` - Uses logger with context
- `apps/social-publish/src/utils/error-handler.ts` - Enhanced with structured logging

**Packages:**
- `packages/shared/src/utils/logger.ts` - Enhanced with structured logging features

### ✅ Kept console.log (Appropriate)

**Scripts (CLI tools):**
- `scripts/*.mts` - Console output is appropriate for CLI scripts
- `apps/vscode-extension/scripts/*.mts` - Console output for CLI tools
- `packages/html-viewer/scripts/bundle.ts` - Build script output

**Reason:** Scripts are CLI tools where console output is the expected interface.

## ESLint Rules

- **Application Code**: `no-console` error (must use logger)
- **Scripts**: `no-console` off (console is appropriate for CLI tools)
- **Allowed**: `console.warn`, `console.error`, `console.info`, `console.debug` (for logger implementation)

## Benefits

1. **Structured Data**: JSON output in production for log aggregation
2. **Context**: Rich context objects for better debugging
3. **Service Identification**: Service names in logs for multi-service debugging
4. **Timestamps**: Consistent ISO 8601 timestamps
5. **Production Ready**: Automatic JSON formatting in production
6. **Debug Control**: Environment-based debug logging

## Next Steps

1. ✅ Enhanced logger with structured features
2. ✅ Migrated social-publish apps
3. ✅ Updated error handler
4. ⚠️ Consider migrating other apps (optional)
5. ⚠️ Add log aggregation integration (optional - e.g., Datadog, CloudWatch)

