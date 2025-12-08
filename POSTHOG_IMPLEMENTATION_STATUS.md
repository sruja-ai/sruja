# PostHog Error Tracking Implementation Status

## ✅ Completed

### 1. Error Tracking Utility
**File**: `packages/shared/src/analytics/errorTracking.ts`
- ✅ `trackError()` - Captures errors with context to PostHog
- ✅ `trackInteraction()` - Tracks user interactions
- ✅ `trackPerformance()` - Tracks performance metrics
- ✅ Error sanitization (removes sensitive data)
- ✅ Automatic context collection (user agent, screen size, URL)
- ✅ Integration with structured logger

### 2. Logger Enhancement
**File**: `packages/shared/src/utils/logger.ts`
- ✅ Automatic PostHog integration for all `logger.error()` calls
- ✅ Sends errors with component, action, and browser context
- ✅ Event naming: `error.{component}.{action}`

### 3. WASM Adapter Error Tracking
**File**: `packages/shared/src/web/wasmAdapter.ts`
- ✅ Script loading failures
- ✅ Go runtime loading failures
- ✅ WASM module loading failures
- ✅ Missing WASM functions
- ✅ Parse DSL errors
- ✅ Print JSON errors
- ✅ Export errors (Markdown, SVG, HTML)

### 4. Playground Error Tracking
**File**: `apps/website/src/features/playground/components/LiveSrujaBlock.tsx`
- ✅ Error handling in `renderDiagram()`
- ✅ Track parse failures
- ✅ Track viewer initialization failures
- ✅ Track render button clicks (interaction)
- ✅ Track successful renders

### 5. Studio Error Tracking
**Files updated**:
- ✅ `apps/studio-core/src/utils/viewerUtils.ts` - Parse errors, sync failures, viewer errors, diagnostics failures
- ✅ `apps/studio-core/src/utils/exportUtils.ts` - Export failures (SVG, PDF, PNG, JSON) with interaction tracking
- ✅ `apps/studio-core/src/hooks/useViewer.ts` - WASM init failures
- ✅ `apps/studio-core/src/components/ErrorBoundary.tsx` - React error boundary
- ✅ `apps/studio-core/src/components/MarkdownPreview.tsx` - Mermaid init failures
- ✅ `apps/studio-core/src/App.tsx` - Viewer reload, export failures

### 6. Viewer Error Tracking
**Files updated**:
- ✅ `apps/viewer-core/app/embed.tsx` - Container not found, layout failures, view switching errors
- ✅ `apps/viewer-core/app/utils/exportUtils.ts` - All export failures (SVG, PNG, PDF, Markdown, HTML) with interaction tracking
- ✅ `apps/viewer-core/app/components/MarkdownPreview.tsx` - Mermaid init failures

### 7. Mermaid Error Tracking
**Files updated**:
- ✅ `packages/ui/src/components/MermaidDiagram.tsx` - Render failures
- ✅ `apps/studio-core/src/components/MarkdownPreview.tsx` - Init failures
- ✅ `apps/viewer-core/app/components/MarkdownPreview.tsx` - Init failures

### 8. User Interaction Tracking
**Areas covered**:
- ✅ Diagram render button clicks (Playground)
- ✅ Export actions (SVG, PNG, PDF, JSON) in Studio
- ✅ Export actions (SVG, PNG, PDF, Markdown, HTML) in Viewer
- ✅ Successful export tracking

## Summary

All error tracking and interaction tracking has been implemented across the codebase:

1. **Logger Enhancement**: Logger now automatically sends errors to PostHog with context
2. **Playground**: Error tracking for render failures, interaction tracking for render button
3. **Studio**: Error tracking in export utils, viewer utils, ErrorBoundary, useViewer, App.tsx, MarkdownPreview; interaction tracking for exports
4. **Viewer**: Error tracking in embed.tsx, exportUtils.ts, MarkdownPreview; interaction tracking for exports
5. **Mermaid**: Error tracking in MermaidDiagram.tsx and MarkdownPreview components (studio & viewer)

All errors are now captured with:
- Component context (playground, studio, viewer, mermaid, wasm)
- Action context (render, export, parse, etc.)
- Error type and message
- Browser context (user agent, screen size, URL)

## Usage Example

```typescript
import { logger, trackInteraction } from '@sruja/shared'

// Errors are automatically tracked via logger.error()
logger.error('Failed to render diagram', {
  component: 'playground',
  action: 'render',
  errorType: 'parse_failure',
  error: error.message,
})

// Track user interactions
trackInteraction('export', 'studio', {
  format: 'svg',
  scale: 2,
})
```

## Event Naming Convention

- **Errors**: `error.{component}.{action}` (e.g., `error.playground.render`)
- **Interactions**: `interaction.{component}.{action}` (e.g., `interaction.studio.export`)
- **Performance**: `performance.{metric}` (e.g., `performance.render_time`)

## Next Steps

1. ✅ Monitor PostHog dashboard for error patterns
2. ✅ Review error rates and identify common issues
3. ✅ Use interaction data to understand user behavior
4. ✅ Add performance tracking for critical operations (optional)
