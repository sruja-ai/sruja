# Hot Reload Setup for Playground Server

The playground server supports hot reloading using [Air](https://github.com/cosmtrek/air), which automatically rebuilds and restarts the server when Go files change.

## Quick Start

1. **Install Air:**
   ```bash
   go install github.com/air-verse/air@latest
   
   # Add Go bin to PATH (add to ~/.zshrc or ~/.bashrc for persistence)
   export PATH=$PATH:$(go env GOPATH)/bin
   
   # Or for zsh specifically, add to ~/.zshrc:
   echo 'export PATH=$PATH:$(go env GOPATH)/bin' >> ~/.zshrc
   source ~/.zshrc
   ```
   
   **Note**: The `dev.sh` script will automatically find air even if it's not in PATH, but adding it to PATH is recommended.

2. **Run with hot reload:**
   ```bash
   # Using turborepo (recommended)
   bun dev
   
   # Or directly
   cd apps/playground-server
   bun run dev
   ```

3. **The server will automatically:**
   - Watch for changes in `apps/playground-server/*.go`
   - Watch for changes in `pkg/**/*.go` (shared packages)
   - Rebuild and restart on file changes
   - Show build errors in the terminal

## Configuration

The Air configuration is in `.air.toml`. Key settings:

- **Watches**: Current directory and `../../pkg` for shared code
- **Build command**: `go build -o ./tmp/main ./main.go`
- **Excludes**: Test files, node_modules, build artifacts
- **Delay**: 1000ms before rebuilding (prevents rapid rebuilds)

## Troubleshooting

### Air not found
```bash
# Install air
go install github.com/air-verse/air@latest

# Add to PATH (add to ~/.zshrc or ~/.bashrc)
export PATH=$PATH:$(go env GOPATH)/bin
```

### Not watching pkg/ changes
If changes in `pkg/` don't trigger rebuilds, ensure `follow_symlink = true` in `.air.toml` and that the `include_dir` includes `../../pkg`.

### Fallback to simple mode
If air isn't available, the `dev.sh` script will fall back to `go run`:
```bash
bun run dev:simple
# or
go run ./main.go
```

## Integration with Turborepo

The `dev` task in `package.json` runs the hot reload script. Turborepo will:
- Run this task in parallel with other `dev` tasks
- Keep it running (persistent task)
- Not cache it (cache: false in turbo.json)

## Manual Testing

To test hot reload:
1. Start the server: `bun run dev`
2. Make a change to `main.go` or any file in `pkg/`
3. Watch the terminal - it should rebuild and restart automatically

