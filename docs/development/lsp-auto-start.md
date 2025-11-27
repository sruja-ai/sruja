# Language Server Auto-Start

## How It Works

The Sruja extension **automatically starts the language server** when you open a `.sruja` file. You don't need to run anything manually.

## Automatic Startup Process

1. **Extension Activates**: When you open a `.sruja` file, the extension activates
2. **Finds LSP Binary**: The extension looks for `sruja-lsp` in this order:
   - Workspace root: `./sruja-lsp` (your current project)
   - System PATH: `sruja-lsp` (if installed globally)
   - Development fallback: `go run ./cmd/sruja-lsp/main.go` (if source code exists)
3. **Starts LSP Server**: Automatically launches the server process
4. **Connects**: The extension connects to the server via stdio

## What You Need

### Option 1: Binary in Workspace (Recommended)

Just have the `sruja-lsp` binary in your workspace root:

```bash
# Build it once
go build -o sruja-lsp ./cmd/sruja-lsp

# That's it! Extension will find it automatically
```

### Option 2: Binary in PATH

Install globally:

```bash
go build -o sruja-lsp ./cmd/sruja-lsp
sudo cp sruja-lsp /usr/local/bin/
```

### Option 3: Development Mode

If you have the source code, the extension will automatically use:

```bash
go run ./cmd/sruja-lsp/main.go
```

No manual setup needed!

## Verify LSP is Running

### Method 1: Check Status Bar

1. Open a `.sruja` file
2. Look at the bottom status bar
3. You should see "Sruja LSP" or similar indicator
4. If it shows an error, click it to see details

### Method 2: Check Output Panel

1. Open a `.sruja` file
2. Go to: View → Output
3. Select "Sruja LSP" from the dropdown
4. You should see LSP initialization messages

### Method 3: Test LSP Features

If LSP is running, you should see:
- ✅ Syntax error highlighting (red squiggles)
- ✅ Autocomplete when typing
- ✅ Hover information on identifiers
- ✅ Go to definition (Cmd+Click)

## Troubleshooting

### LSP Not Starting

**Symptom**: No error highlighting, no autocomplete, status bar shows error

**Check**:
1. Is `sruja-lsp` binary present?
   ```bash
   ls -la sruja-lsp
   ```

2. Is it executable?
   ```bash
   chmod +x sruja-lsp
   ```

3. Check extension output:
   - View → Output → Select "Sruja LSP"
   - Look for error messages

4. Check Developer Tools:
   - `Cmd+Shift+P` → "Developer: Toggle Developer Tools"
   - Console tab → Look for LSP errors

### LSP Starts But Features Don't Work

**Possible causes**:
1. **Parser errors**: Check if your `.sruja` file has syntax errors
2. **Extension not activated**: Make sure extension is enabled
3. **LSP version mismatch**: Rebuild the binary

**Fix**:
```bash
# Rebuild LSP
go build -o sruja-lsp ./cmd/sruja-lsp

# Reload extension
# Cmd+Shift+P → "Developer: Reload Window"
```

### Manual Test (If Needed)

If you want to test the LSP manually:

```bash
# Test that binary works
./sruja-lsp
# (It will wait for stdio input - this is normal)

# Send a test message (Ctrl+C to exit)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' | ./sruja-lsp
```

## Summary

✅ **No manual setup needed** - Extension handles everything  
✅ **Automatic startup** - Starts when you open `.sruja` files  
✅ **Smart detection** - Finds binary in workspace, PATH, or uses `go run`  
✅ **Just build once** - `go build -o sruja-lsp ./cmd/sruja-lsp`

The language server runs in the background automatically. You only need to ensure the `sruja-lsp` binary exists and is executable.

