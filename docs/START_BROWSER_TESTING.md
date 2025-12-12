# Start Browser Testing - Step by Step

## ✅ Pre-Flight Check Complete

All packages are built and exports verified. Ready to test!

---

## Step 1: Start Development Servers

Open **two terminal windows**:

### Terminal 1: Website
```bash
cd apps/website
npm run dev
```

**Expected Output**:
```
  ➜  Local:   http://localhost:4321/
  ➜  Network: use --host to expose
```

**Wait for**: "ready in X ms" message

### Terminal 2: Playground
```bash
cd apps/playground
npm run dev
```

**Expected Output**:
```
  VITE vX.X.X  ready in XXX ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

**Wait for**: "ready in X ms" message

---

## Step 2: Open Browser DevTools

**Important**: Keep browser console open to catch any errors!

1. Open your browser (Chrome/Edge recommended)
2. Press `F12` or `Cmd+Option+I` (Mac) to open DevTools
3. Go to **Console** tab
4. Clear console (right-click → Clear console)

---

## Step 3: Quick Verification Tests

### Test 1: Website Mermaid Diagram (2 minutes)

1. **Navigate**: Open http://localhost:4321/docs/getting-started
   - (Or any docs page with ````sruja` code blocks)

2. **Find Code Block**: Scroll to find a code block with ````sruja` syntax
   - Look for code blocks with "Show Diagram" button

3. **Click "Show Diagram"**: Click the button above the code block

4. **Verify**:
   - ✅ Diagram renders below the code block
   - ✅ No errors in browser console
   - ✅ Diagram is visible and properly sized

**If it works**: ✅ Website migration successful!
**If it fails**: Check console for errors, see troubleshooting below

---

### Test 2: Playground Markdown Panel (2 minutes)

1. **Navigate**: Open http://localhost:5173
   - (Or the port shown in your terminal)

2. **Load Example**: 
   - Click on an example from the dropdown (if available)
   - OR paste DSL code into the editor

3. **Check Right Panel**: Look at the right side panel
   - Should show "Markdown Representation" header
   - Should display formatted markdown content

4. **Verify**:
   - ✅ Markdown panel is visible
   - ✅ Content is properly formatted
   - ✅ No errors in browser console

**If it works**: ✅ Playground migration successful!
**If it fails**: Check console for errors, see troubleshooting below

---

### Test 3: Markdown Panel Features (3 minutes)

1. **In Playground Markdown Panel**:

   a. **Preview/Raw Toggle**:
      - Click "Raw" button → Should show markdown source code
      - Click "Preview" button → Should show rendered markdown
      - ✅ Toggle works smoothly

   b. **Copy Button**:
      - Click "Copy" button
      - Should show "Copied!" feedback
      - Paste into a text editor
      - ✅ Content matches what's displayed

2. **Verify**:
   - ✅ All buttons work
   - ✅ No console errors
   - ✅ UI is responsive

**If it works**: ✅ Shared component working correctly!

---

### Test 4: Theme Toggle (1 minute)

1. **In Playground**: Look for theme toggle button (if available)
2. **Toggle Theme**: Switch between light/dark
3. **Check Markdown Panel**: Verify styling updates correctly

**Expected**:
- ✅ Theme changes apply to markdown panel
- ✅ Text remains readable
- ✅ No visual glitches

---

## Step 4: Check Console for Errors

After running tests, check browser console:

**Good Signs** ✅:
- No red errors
- Only normal info/warning messages
- Everything loads successfully

**Bad Signs** ❌:
- Red error messages
- "Module not found" errors
- "Cannot read property" errors
- Network errors (404, 500, etc.)

**If you see errors**: Note them down and see troubleshooting section

---

## Step 5: Document Results

Open `docs/TEST_RESULTS.md` and fill in:
- Date
- Browser used
- Test results (Pass/Fail for each test)
- Any issues found
- Notes

---

## Troubleshooting

### Issue: "Module not found" or import errors

**Solution**:
```bash
# Rebuild packages
cd packages/ui && npm run build
cd ../../apps/website && npm run dev  # Restart
```

### Issue: Diagram doesn't render

**Check**:
1. Browser console for errors
2. Network tab - is WASM loading?
3. Is DSL syntax valid?

**Try**:
- Hard refresh (Ctrl+Shift+R or Cmd+Shift+R)
- Clear browser cache
- Check if WASM files are loading in Network tab

### Issue: Markdown panel empty

**Check**:
1. Is DSL loaded in playground?
2. Browser console for conversion errors
3. Check React DevTools - is `convertedMarkdown` in state?

**Try**:
- Load a different example
- Enter simple DSL manually
- Check console for errors

### Issue: Copy button doesn't work

**Check**:
1. Browser permissions for clipboard
2. Console for errors
3. Try in different browser

**Note**: Some browsers require HTTPS or user interaction for clipboard API

### Issue: Server won't start

**Check**:
1. Port already in use? Try different port
2. Dependencies installed? Run `npm install`
3. Check terminal for error messages

---

## What to Test Next

If quick tests pass, continue with:

1. **Detailed Tests**: See `BROWSER_TESTING_CHECKLIST.md`
   - Test different DSL scenarios
   - Test error handling
   - Test edge cases

2. **Cross-App Testing**:
   - Use same DSL in website and playground
   - Compare outputs
   - Verify consistency

3. **Performance Testing**:
   - Load large DSL files
   - Measure render time
   - Check for memory issues

---

## Success Criteria

Migration is successful if:
- ✅ All quick tests pass
- ✅ No console errors
- ✅ Diagrams render correctly
- ✅ Markdown displays correctly
- ✅ All UI features work
- ✅ No visual regressions

---

## Need Help?

If you encounter issues:
1. Check browser console for errors
2. Check terminal output for server errors
3. Review troubleshooting section above
4. Check `BROWSER_TESTING_CHECKLIST.md` for detailed scenarios

---

## Next Steps After Testing

Once testing is complete:
1. Document results in `TEST_RESULTS.md`
2. Fix any issues found
3. Update roadmap with test results
4. Mark Phase 2 as fully complete

Good luck! 🚀
