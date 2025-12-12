# Quick Browser Testing Guide

## Quick Start

### 1. Start Development Servers

```bash
# Terminal 1: Website
cd apps/website
npm run dev
# Should start on http://localhost:4321

# Terminal 2: Playground  
cd apps/playground
npm run dev
# Should start on http://localhost:5173 (or next available port)
```

### 2. Quick Verification Tests

#### Test A: Website Mermaid Diagrams (2 minutes)

1. Open http://localhost:4321/docs/getting-started (or any docs page)
2. Scroll to find a code block with ````sruja` syntax
3. Click "Show Diagram" button
4. ✅ **Expected**: Diagram renders without errors

**If it works**: ✅ Migration successful for Website
**If it fails**: Check browser console for errors

#### Test B: Playground Markdown Panel (2 minutes)

1. Open http://localhost:5173 (or playground URL)
2. Load any example or enter DSL code
3. Look at the right panel - should show "Markdown Representation"
4. ✅ **Expected**: Markdown content displays in preview mode

**If it works**: ✅ Migration successful for Playground
**If it fails**: Check browser console for errors

#### Test C: Markdown Panel Features (3 minutes)

1. In Playground markdown panel:
   - Click "Raw" button → Should show markdown source
   - Click "Preview" button → Should show rendered markdown
   - Click "Copy" button → Should show "Copied!" feedback
2. ✅ **Expected**: All buttons work correctly

**If it works**: ✅ Shared component working correctly

#### Test D: Theme Toggle (1 minute)

1. In Playground, toggle dark/light theme (if available)
2. Check markdown panel styling
3. ✅ **Expected**: Theme applies correctly, no visual glitches

---

## Common Issues & Solutions

### Issue: "Module not found" errors

**Solution**: Rebuild packages
```bash
cd packages/ui && npm run build
cd packages/shared && npm run build  # if needed
```

### Issue: Diagram doesn't render

**Check**:
1. Browser console for errors
2. Network tab - is WASM loading?
3. Is DSL syntax valid?

### Issue: Markdown panel empty

**Check**:
1. Is DSL loaded in playground?
2. Browser console for conversion errors
3. Check `architectureStore` state in React DevTools

### Issue: Copy button doesn't work

**Check**:
1. Browser permissions for clipboard
2. Console for errors
3. Try in different browser

---

## Detailed Testing

For comprehensive testing, see: `BROWSER_TESTING_CHECKLIST.md`

---

## Test Results Log

```
Date: ___________

Quick Tests:
- [ ] Test A: Website Mermaid Diagrams
- [ ] Test B: Playground Markdown Panel  
- [ ] Test C: Markdown Panel Features
- [ ] Test D: Theme Toggle

Issues:
1. 
2. 

Notes:
```
