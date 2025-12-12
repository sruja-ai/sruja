# Debugging WASM LSP in VS Code Extension

This guide helps you debug issues with the WASM-based LSP implementation.

## Quick Start

1. **Open Output Channel**: The extension automatically opens the "Sruja WASM LSP" output channel when it initializes. You can also open it manually:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type "Output: Show Output Channels"
   - Select "Sruja WASM LSP"

2. **Run Debug Command**:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type "Sruja: Debug WASM LSP"
   - This will test all LSP functions with the current file

## Common Issues

### 1. WASM Files Not Found

**Symptoms:**

- Error: "WASM file not found" or "wasm_exec.js not found"
- Output channel shows missing file paths

**Solution:**

- Ensure WASM files are copied during build:
  ```bash
  npm run copy-wasm
  ```
- Check that `wasm/` directory exists in extension root with:
  - `sruja.wasm.gz` or `sruja.wasm`
  - `wasm_exec.js`

### 2. WASM Functions Not Available

**Symptoms:**

- Output channel shows "No WASM LSP functions found"
- Specific functions fail with "function not available"

**Solution:**

- Verify WASM module loaded correctly (check output channel)
- Check that Go WASM exports match expected function names:
  - `sruja_get_diagnostics`
  - `sruja_hover`
  - `sruja_completion`
  - `sruja_go_to_definition`
  - `sruja_format`

### 3. Functions Return Errors

**Symptoms:**

- Functions are available but return errors
- Output channel shows function test failures

**Debugging Steps:**

1. Check the output channel for detailed error messages
2. Run the debug command to see which functions fail
3. Verify the input format matches what Go WASM expects
4. Check if there are issues with line/column number conversion (0-based vs 1-based)

### 4. Functions Work But No Results

**Symptoms:**

- Functions execute without errors
- But hover/completion/definition return empty results

**Debugging Steps:**

1. Check if the cursor position is correct (line/column conversion)
2. Verify the document text is being passed correctly
3. Test with a simple file first:
   ```sruja
   architecture "Test" {
       system App "Application" {}
   }
   ```

## Debugging Workflow

### Step 1: Check Initialization

When the extension activates, check the output channel for:

- ✅ WASM files found
- ✅ WASM API initialized
- ✅ Function tests passed

### Step 2: Test Individual Features

1. **Diagnostics**: Open a .sruja file with errors - you should see red squiggles
2. **Hover**: Hover over a symbol - check output channel for hover requests
3. **Completion**: Type `Ctrl+Space` - check output channel for completion requests
4. **GoToDefinition**: `F12` or `Cmd+Click` - check output channel for definition requests
5. **Format**: `Shift+Alt+F` - check output channel for format requests

### Step 3: Use Debug Command

The debug command (`Sruja: Debug WASM LSP`) will:

- Test all LSP functions with the current file
- Show detailed results in the output channel
- Help identify which specific function is failing

## Output Channel Logs

The output channel shows:

- **Timestamps**: When each operation occurred
- **Log Levels**: INFO, WARN, ERROR
- **Function Calls**: Which WASM function was called with what parameters
- **Results**: What was returned (success/failure, data preview)
- **Errors**: Full error messages and stack traces

## VS Code Developer Tools

For advanced debugging:

1. **Open Developer Tools**:
   - `Help > Toggle Developer Tools`
   - Or `Cmd+Option+I` (Mac) / `Ctrl+Shift+I` (Windows/Linux)

2. **Check Console**:
   - All `console.log/error/warn` calls also appear in Developer Tools console
   - Useful for seeing logs that might not be in output channel

3. **Network Tab**:
   - Check if WASM files are being loaded correctly
   - Look for 404 errors on WASM resources

## Rebuilding WASM

If WASM functions are missing or incorrect:

1. **Rebuild WASM from Go**:

   ```bash
   # From project root
   make wasm
   ```

2. **Copy to Extension**:

   ```bash
   cd apps/vscode-extension
   npm run copy-wasm
   ```

3. **Rebuild Extension**:

   ```bash
   npm run build
   ```

4. **Reload VS Code**:
   - `Cmd+Shift+P` > "Developer: Reload Window"

## Testing Checklist

- [ ] Extension activates without errors
- [ ] Output channel shows WASM files found
- [ ] All function tests pass during initialization
- [ ] Diagnostics appear on files with errors
- [ ] Hover works on symbols
- [ ] Completion shows suggestions
- [ ] GoToDefinition navigates to definitions
- [ ] Format formats the document

## Getting Help

If issues persist:

1. Check the output channel for specific error messages
2. Run the debug command and share the output
3. Verify WASM files are up to date
4. Check that Go WASM exports match TypeScript expectations
