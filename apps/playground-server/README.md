# Playground Server

Go HTTP server for the Sruja playground API.

## Development

### Hot Reloading with Air

The server supports hot reloading using [Air](https://github.com/cosmtrek/air).

**Install Air:**
```bash
go install github.com/air-verse/air@latest
```

**Run with hot reload:**
```bash
# Using turborepo (recommended)
bun dev

# Or directly
cd apps/playground-server
bun run dev
# or
air
```

**Run without hot reload:**
```bash
bun run dev:simple
# or
go run ./main.go
```

### Configuration

Air configuration is in `.air.toml`. It watches:
- All `.go` files in the current directory and parent `pkg/` directory
- Rebuilds and restarts the server on changes

### API Endpoints

- `POST /api/compile` - Compile Sruja code to D2
- `POST /api/validate` - Validate Sruja code
- `GET /health` - Health check

### Environment Variables

- `PORT` - Server port (default: 8080)

