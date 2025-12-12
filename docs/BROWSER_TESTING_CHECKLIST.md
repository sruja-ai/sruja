# Browser Testing Checklist - Migration Phase 2

This document provides a comprehensive testing checklist for verifying the TypeScript exporter migration works correctly in the browser.

## Prerequisites

1. **Start Development Servers**:
   ```bash
   # Terminal 1: Website
   cd apps/website
   npm run dev

   # Terminal 2: Playground
   cd apps/playground
   npm run dev
   ```

2. **Clear Browser Cache**: Clear browser cache or use incognito mode to ensure fresh load

3. **Browser Console**: Keep browser DevTools console open to catch any errors

---

## Website Testing (`apps/website`)

### Test 1: Code Block Mermaid Diagrams

**Location**: Any documentation page with ````sruja` code blocks

**Steps**:
1. Navigate to a documentation page (e.g., `/docs/getting-started`)
2. Find a code block with ````sruja` syntax
3. Click "Show Diagram" button
4. Verify diagram renders correctly

**Expected Results**:
- ✅ Diagram renders without errors
- ✅ Diagram matches expected structure
- ✅ No console errors
- ✅ Diagram is properly sized
- ✅ Zoom controls work (if applicable)

**Test Cases**:
- [ ] Simple architecture with 1-2 systems
- [ ] Complex architecture with multiple systems and containers
- [ ] Architecture with relations (arrows)
- [ ] Architecture with persons
- [ ] Architecture with deployments

### Test 2: Mermaid Diagram Theme Support

**Steps**:
1. Open a page with a rendered diagram
2. Toggle dark/light theme (if available)
3. Verify diagram theme updates

**Expected Results**:
- ✅ Diagram theme changes correctly
- ✅ No console errors during theme switch
- ✅ Diagram remains visible and readable

### Test 3: "Open in Playground" Functionality

**Steps**:
1. Find a code block with ````sruja` syntax
2. Click "Open in Playground" button
3. Verify playground opens with correct DSL

**Expected Results**:
- ✅ Playground opens in new tab/window
- ✅ DSL content is correctly loaded
- ✅ Diagram renders in playground
- ✅ No data loss or corruption

### Test 4: Code Block Actions UI

**Steps**:
1. Find multiple code blocks on a page
2. Verify toolbar appears on each block
3. Test all buttons (Show Diagram, Open in Playground)

**Expected Results**:
- ✅ Toolbar appears on all ````sruja` blocks
- ✅ Buttons are clickable and responsive
- ✅ No UI glitches or layout issues

### Test 5: Error Handling

**Steps**:
1. Create a test page with invalid DSL in a code block
2. Try to render diagram
3. Verify error handling

**Expected Results**:
- ✅ Error message displays clearly
- ✅ No uncaught exceptions in console
- ✅ Page doesn't crash
- ✅ User can still interact with other elements

---

## Playground Testing (`apps/playground`)

### Test 1: Markdown Panel Display

**Location**: Playground app, Markdown panel (right side)

**Steps**:
1. Open playground (`/playground`)
2. Load or enter DSL code
3. Check Markdown panel on the right

**Expected Results**:
- ✅ Markdown panel displays correctly
- ✅ Markdown content is properly formatted
- ✅ Headers, lists, code blocks render correctly
- ✅ Mermaid diagrams in markdown render (if any)

### Test 2: Preview/Raw Toggle

**Steps**:
1. Open Markdown panel
2. Click "Preview" button (should be active by default)
3. Click "Raw" button
4. Toggle back to "Preview"

**Expected Results**:
- ✅ Preview mode shows rendered markdown
- ✅ Raw mode shows markdown source code
- ✅ Toggle works smoothly
- ✅ No content loss when switching

### Test 3: Copy to Clipboard

**Steps**:
1. Open Markdown panel
2. Click "Copy" button
3. Paste into a text editor
4. Verify content matches

**Expected Results**:
- ✅ Copy button shows "Copied!" feedback
- ✅ Clipboard contains correct markdown
- ✅ No formatting issues in pasted content
- ✅ Works in both Preview and Raw modes

### Test 4: Markdown Updates on DSL Change

**Steps**:
1. Load DSL in playground
2. Wait for markdown to generate
3. Modify DSL code
4. Verify markdown updates

**Expected Results**:
- ✅ Markdown updates automatically when DSL changes
- ✅ Loading state shows during conversion
- ✅ No duplicate content
- ✅ Updates are smooth (no flickering)

### Test 5: Loading States

**Steps**:
1. Load a large/complex DSL file
2. Observe loading state
3. Verify it completes

**Expected Results**:
- ✅ Loading message displays: "Converting DSL to Markdown..."
- ✅ Loading state clears when complete
- ✅ No stuck loading states
- ✅ Error handling if conversion fails

### Test 6: Empty States

**Steps**:
1. Open playground with no DSL loaded
2. Check Markdown panel

**Expected Results**:
- ✅ Empty state message displays: "No architecture loaded"
- ✅ UI is clean and informative
- ✅ No errors in console

### Test 7: Markdown Content Verification

**Steps**:
1. Load a known DSL file
2. Compare markdown output with expected format
3. Verify all sections are present

**Expected Results**:
- ✅ Table of Contents (if enabled)
- ✅ Executive Summary
- ✅ Architecture Overview
- ✅ Systems section
- ✅ All other sections as expected
- ✅ Formatting is correct

---

## Cross-App Testing

### Test 1: Consistency Check

**Steps**:
1. Use same DSL in both Website code block and Playground
2. Compare markdown output
3. Compare mermaid diagram output

**Expected Results**:
- ✅ Markdown output is identical (or functionally equivalent)
- ✅ Mermaid diagrams match
- ✅ No discrepancies in content

### Test 2: Performance

**Steps**:
1. Load large DSL files (1000+ lines)
2. Measure render time
3. Check for performance issues

**Expected Results**:
- ✅ Rendering completes in reasonable time (< 5 seconds)
- ✅ No UI freezing
- ✅ Memory usage is reasonable
- ✅ No memory leaks

---

## Browser Compatibility

Test in multiple browsers:

- [ ] Chrome/Edge (Chromium)
- [ ] Firefox
- [ ] Safari (if on Mac)
- [ ] Mobile browser (optional)

**Expected Results**:
- ✅ All features work in all browsers
- ✅ No browser-specific errors
- ✅ UI renders correctly in all browsers

---

## Error Scenarios

### Test 1: Network Issues

**Steps**:
1. Disable network after page load
2. Try to render diagrams
3. Re-enable network

**Expected Results**:
- ✅ Graceful error handling
- ✅ User-friendly error messages
- ✅ No crashes

### Test 2: Invalid DSL

**Steps**:
1. Enter invalid DSL syntax
2. Try to generate markdown/mermaid

**Expected Results**:
- ✅ Error messages are clear
- ✅ No uncaught exceptions
- ✅ User can correct and retry

### Test 3: Empty DSL

**Steps**:
1. Enter empty DSL
2. Check markdown panel

**Expected Results**:
- ✅ Empty state displays correctly
- ✅ No errors
- ✅ UI remains functional

---

## Regression Testing

Verify existing functionality still works:

- [ ] DSL parsing still works
- [ ] Diagram visualization still works
- [ ] All existing features function correctly
- [ ] No visual regressions
- [ ] No performance regressions

---

## Test Results Template

```
Date: ___________
Tester: ___________
Browser: ___________
Version: ___________

Website Tests:
- [ ] Test 1: Code Block Mermaid Diagrams
- [ ] Test 2: Mermaid Diagram Theme Support
- [ ] Test 3: "Open in Playground" Functionality
- [ ] Test 4: Code Block Actions UI
- [ ] Test 5: Error Handling

Playground Tests:
- [ ] Test 1: Markdown Panel Display
- [ ] Test 2: Preview/Raw Toggle
- [ ] Test 3: Copy to Clipboard
- [ ] Test 4: Markdown Updates on DSL Change
- [ ] Test 5: Loading States
- [ ] Test 6: Empty States
- [ ] Test 7: Markdown Content Verification

Cross-App Tests:
- [ ] Test 1: Consistency Check
- [ ] Test 2: Performance

Browser Compatibility:
- [ ] Chrome/Edge
- [ ] Firefox
- [ ] Safari

Error Scenarios:
- [ ] Network Issues
- [ ] Invalid DSL
- [ ] Empty DSL

Regression:
- [ ] All existing features work

Issues Found:
1. 
2. 
3. 

Notes:
```

---

## Quick Test Script

For quick verification, test these key scenarios:

1. **Website - Simple Diagram**:
   - Go to any docs page
   - Find ````sruja` block
   - Click "Show Diagram"
   - ✅ Should render

2. **Playground - Markdown Panel**:
   - Open `/playground`
   - Load any example
   - Check right panel
   - ✅ Should show markdown

3. **Toggle Test**:
   - In Playground markdown panel
   - Click Preview/Raw toggle
   - ✅ Should switch views

4. **Copy Test**:
   - In Playground markdown panel
   - Click Copy
   - Paste somewhere
   - ✅ Should match content

---

## Success Criteria

Migration is successful if:
- ✅ All tests pass
- ✅ No console errors
- ✅ Output matches WASM version (or is functionally equivalent)
- ✅ Performance is acceptable
- ✅ No visual regressions
- ✅ Error handling works correctly
