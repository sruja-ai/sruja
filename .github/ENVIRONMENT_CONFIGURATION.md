# Environment-Specific Configuration

This document outlines what should be environment-specific across staging and production.

## ✅ Already Configured

### 1. **PostHog Analytics**
- ✅ Environment automatically tagged on all events
- ✅ Single project with environment filtering
- **Status**: Implemented - all events include `environment: "staging"` or `environment: "production"`

### 2. **Algolia Search**
- ✅ Staging: `sruja_docs_staging` index
- ✅ Production: `sruja_docs` index
- ✅ Same Algolia project, different indices
- **Status**: Correctly configured

### 3. **Site URLs**
- ✅ Staging: `https://staging.sruja.ai`
- ✅ Production: `https://sruja.ai`
- **Status**: Correctly configured in workflows

### 4. **Build Configuration**
- ✅ `PUBLIC_ENV` set correctly in workflows
- ✅ `NODE_ENV=production` for both staging and production
- **Status**: Correctly configured

## 🔍 Should Be Environment-Specific (Recommendations)

### 1. **Console Logging** ⚠️
**Current**: Console logs appear in all environments  
**Recommendation**: 
- Disable `console.log`, `console.debug`, `console.info` in production
- Keep `console.warn` and `console.error` in all environments
- Use environment-aware logger utility

**Files to update**:
- `apps/website/src/config/env.ts` - Add logging configuration
- `packages/shared/src/utils/logger.ts` - Already has some environment awareness

### 2. **Debug Information** ⚠️
**Current**: Debug logs appear in production builds  
**Recommendation**:
- Disable debug logs in production
- Keep debug logs in staging for troubleshooting
- Use `envConfig.env === 'development'` checks

**Files with debug logs**:
- `apps/website/src/features/search/components/AlgoliaSearch.tsx` - Already checks for development
- `apps/website/src/config/env.ts` - Already checks for development

### 3. **Error Tracking Verbosity** ✅
**Current**: Errors are tracked to PostHog in all environments  
**Recommendation**: 
- Keep error tracking in all environments (current behavior is correct)
- Environment is automatically included (already implemented)

### 4. **Performance Monitoring**
**Current**: Not explicitly configured  
**Recommendation**:
- Same PostHog project (already using)
- Environment automatically tagged (already implemented)
- Consider different sampling rates if needed

### 5. **Feature Flags** (Future)
**Current**: No feature flags system  
**Recommendation**:
- If implementing feature flags, use PostHog feature flags
- Environment-specific flags can be set in PostHog dashboard
- No code changes needed if using PostHog feature flags

## 📋 Summary

### What's Already Good ✅
1. PostHog environment tagging - ✅ Implemented
2. Algolia index separation - ✅ Correct
3. Site URL configuration - ✅ Correct
4. Build environment variables - ✅ Correct

### What Could Be Improved 🔧
1. **Console logging** - Should be environment-aware
   - Production: Only errors/warnings
   - Staging: All logs (for debugging)
   - Development: All logs

2. **Debug information** - Mostly handled, but could be more consistent
   - Most debug logs already check for development
   - Some console.info could be environment-aware

### Not Needed ❌
- Separate PostHog projects (using single project with environment filtering)
- Separate Algolia projects (using separate indices in same project)
- Different API keys (same keys work for all environments)
- Different error tracking (same PostHog project with environment tags)

## 🎯 Action Items

### High Priority
1. ✅ **PostHog environment tagging** - DONE
2. ⚠️ **Console logging** - Consider implementing environment-aware console wrapper

### Low Priority
1. **Performance monitoring** - Already handled via PostHog
2. **Feature flags** - Can be added later using PostHog feature flags

## Implementation Notes

The current setup is **production-ready**. The main improvement would be to make console logging environment-aware, but this is not critical since:
- Most console logs are already behind development checks
- Production builds are minified, reducing console output impact
- Error tracking works correctly in all environments

