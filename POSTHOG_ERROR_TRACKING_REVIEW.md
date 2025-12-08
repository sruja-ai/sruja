# PostHog Error Tracking Review

## Current State

### ✅ What's Working
- PostHog is initialized in `StudioApp` via `PosthogProvider`
- Basic `capture` function exists in `packages/shared/src/analytics/posthog.ts`
- Some basic event tracking (e.g., `live.render_view` in playground)

### ❌ Missing Error Tracking

#### 1. WASM/Code Execution Errors
**Location**: `packages/shared/src/web/wasmAdapter.ts`
- ❌ WASM initialization failures (line 148-154)
- ❌ Missing WASM functions (line 142-154)
- ❌ WASM script loading failures (line 15, 40)
- ❌ WASM function execution errors (parseDslToJson, dslToSvg, etc.)

**Location**: `apps/website/src/features/playground/components/LiveSrujaBlock.tsx`
- ❌ No error handling in `renderDiagram` (line 14-28)
- ❌ No tracking of parse failures
- ❌ No tracking of viewer initialization failures

#### 2. Studio Errors
**Location**: `apps/studio-core/src/utils/viewerUtils.ts`
- ❌ Parse errors (line 199)
- ❌ DSL sync failures (line 257, 400)
- ❌ Viewer initialization failures (line 170)
- ❌ Diagnostics fetch failures (line 185)
- ❌ Container resize failures (line 121, 131)

**Location**: `apps/studio-core/src/utils/exportUtils.ts`
- ❌ SVG export failures (line 89)
- ❌ PDF export failures (line 130)
- ❌ PNG export failures (line 24, 94)
- ❌ WASM export fallbacks (line 50)

**Location**: `apps/studio-core/src/hooks/useViewer.ts`
- ❌ WASM init failures (line 142)

**Location**: `apps/studio-core/src/components/ErrorBoundary.tsx`
- ❌ React error boundary errors (line 25) - should track

**Location**: `apps/studio-core/src/App.tsx`
- ❌ Viewer reload failures (line 891)
- ❌ Export failures (line 1379)

#### 3. Viewer Errors
**Location**: `apps/viewer-core/app/embed.tsx`
- ❌ Container not found (line 52)
- ❌ Layout failures (line 518)
- ❌ View switch failures (line 471, 484)

**Location**: `apps/viewer-core/app/utils/exportUtils.ts`
- ❌ SVG export failures (line 158)
- ❌ PDF export failures (line 239)
- ❌ Markdown export failures (line 280)
- ❌ HTML export failures (line 356)
- ❌ Export fallbacks (line 79, 111, 125, 175)

**Location**: `apps/viewer-core/app/App.tsx`
- ❌ WASM init failures (line 160)
- ❌ Markdown preview errors (line 437)
- ❌ JSON to DSL conversion failures (line 484)

**Location**: `apps/viewer-core/app/index.tsx` & `main.tsx`
- ❌ Architecture data parse failures (line 23, 25)
- ❌ Root element not found (line 34)
- ❌ App load failures (line 38)

#### 4. Mermaid Rendering Errors
**Location**: `packages/ui/src/components/MermaidDiagram.tsx`
- ❌ Mermaid initialization failures
- ❌ Mermaid render failures (line 53)

**Location**: `apps/studio-core/src/components/MarkdownPreview.tsx`
- ❌ Mermaid init failures (line 18)

**Location**: `apps/viewer-core/app/components/MarkdownPreview.tsx`
- ❌ Mermaid init failures (line 18)

### ❌ Missing User Interaction Tracking

#### Warm Interactions (Common User Actions)
- ❌ Diagram render button clicks
- ❌ Export actions (SVG, PNG, PDF, JSON, HTML)
- ❌ View level changes (system, container, component)
- ❌ Node clicks/expansions
- ❌ Editor changes (DSL edits)
- ❌ Theme toggles
- ❌ Layout algorithm changes
- ❌ Zoom/pan interactions
- ❌ Search/filter actions
- ❌ File save/load actions

## Recommendations

### 1. Create Error Tracking Utility
Create a centralized error tracking function that:
- Captures error with context (component, action, error type)
- Includes error message, stack trace, and user context
- Handles sanitization (no sensitive data)
- Works with structured logger

### 2. Add Error Tracking to All Error Handlers
Wrap all `catch` blocks and error handlers with PostHog tracking:
- WASM errors
- Parse/validation errors
- Export errors
- Viewer errors
- Network errors

### 3. Add User Interaction Tracking
Track common user actions:
- Button clicks
- Form submissions
- View changes
- Export actions
- Settings changes

### 4. Error Context
Include useful context in error events:
- Component name
- Action being performed
- Error type/code
- User agent
- Screen size
- Feature flags
- WASM version
- Viewer version

## Implementation Plan

1. ✅ Review current state (this document)
2. ⏳ Create error tracking utility
3. ⏳ Add error tracking to WASM adapter
4. ⏳ Add error tracking to Studio
5. ⏳ Add error tracking to Viewer
6. ⏳ Add user interaction tracking
7. ⏳ Test error tracking in all scenarios

