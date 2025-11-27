# Debugging Extension Activation

## If Extension Shows "Not Yet Activated"

### Step 1: Check Extension Files

```bash
# Verify extension is installed
ls -la ~/.cursor/extensions/sruja/

# Check key files exist
ls -la ~/.cursor/extensions/sruja/package.json
ls -la ~/.cursor/extensions/sruja/out/extension.js
```

### Step 2: Check Output Panel

1. Open **Output** panel: `View` → `Output`
2. Select **"Log (Extension Host)"** from dropdown
3. Look for:
   - `🚀🚀🚀 Sruja extension ACTIVATING NOW! 🚀🚀🚀`
   - If you see this, activation IS happening
   - If you DON'T see this, extension isn't activating at all

### Step 3: Check for Errors

In the Output panel, look for:
- Red error messages
- `❌❌❌ CRITICAL ERROR`
- Any stack traces

### Step 4: Check Extension Status

**In Cursor:**
1. `Cmd+Shift+P` → `Developer: Show Running Extensions`
2. Look for "sruja" in the list
3. Check status:
   - **Activated** = Good!
   - **Not yet activated** = Problem
   - **Error** = Check error message

### Step 5: Try Manual Activation

**In Cursor Developer Console:**
1. `Cmd+Shift+P` → `Developer: Toggle Developer Tools`
2. Go to Console tab
3. Type:
   ```javascript
   vscode.extensions.getExtension('sruja-ai.sruja')
   ```
4. Check what it returns

### Step 6: Check package.json

```bash
cat ~/.cursor/extensions/sruja/package.json | grep -A 2 "main"
cat ~/.cursor/extensions/sruja/package.json | grep -A 3 "activationEvents"
```

Should show:
- `"main": "./out/extension.js"`
- `"activationEvents": ["*", "onStartupFinished"]`

### Step 7: Verify Extension ID

The extension ID might need to match the publisher.name format. Check:
```bash
cat ~/.cursor/extensions/sruja/package.json | grep -E "(name|publisher)"
```

If the folder name doesn't match, it might not load.

## Common Issues

### Issue 1: Extension Folder Name Mismatch

The extension folder should match the extension ID. Try:
```bash
# Check current name
ls -d ~/.cursor/extensions/sruja

# If needed, rename to match publisher.name
mv ~/.cursor/extensions/sruja ~/.cursor/extensions/sruja-ai.sruja
```

### Issue 2: Missing Dependencies

Check if node_modules exists:
```bash
ls -la ~/.cursor/extensions/sruja/node_modules/
```

If missing, the extension might fail to load.

### Issue 3: Syntax Error in extension.js

Try loading it:
```bash
node -c ~/.cursor/extensions/sruja/out/extension.js
```

Should return no errors.

### Issue 4: Activation Events Not Supported

Some editors don't support `"*"` activation. Try:
- Open a `.sruja` file (triggers `onLanguage:sruja`)
- Or use `onStartupFinished`

## Quick Test

After restarting Cursor, you should see:
1. **Notification**: "Sruja extension is activating!"
2. **In Output**: `🚀🚀🚀 Sruja extension ACTIVATING NOW! 🚀🚀🚀`

If you see these, activation is working!
If you don't see these, the extension isn't loading at all.

