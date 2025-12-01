# Studio Local API Server

## Overview

When `sruja studio` is run, it starts a **local Go API server** that:
1. Serves the Studio React app (static files)
2. Provides REST API endpoints for file operations
3. Handles DSL ↔ JSON conversions
4. Directly reads/writes `.sruja` files from the local Git repository

**Important**: This is different from **Self-Hosted Studio** (see [SELF_HOSTED_STUDIO.md](./SELF_HOSTED_STUDIO.md)):
- **CLI Studio Server** (`sruja studio`): Local server, connected to Git repo, reads/writes `.sruja` files directly
- **Self-Hosted Studio**: Standalone web app, no Git connection, for sharing previews and designing architectures

## Architecture

```
┌─────────────────┐
│  sruja studio   │
└────────┬────────┘
         │
         ├──────────────────┐
         │                  │
         ▼                  ▼
┌──────────────┐    ┌──────────────┐
│  Go API      │    │  React App   │
│  Server      │    │  (Static)    │
│  (Port 5173) │    │              │
└──────┬───────┘    └──────┬───────┘
       │                   │
       │  HTTP API         │
       │  (REST)           │
       │◄──────────────────┤
       │                   │
       ▼                   │
┌──────────────┐          │
│  File System │          │
│  (.sruja)    │          │
└──────────────┘          │
```

## API Endpoints

### 1. Load File

```http
GET /api/files/:path
```

**Response**:
```json
{
  "path": "architecture.sruja",
  "json": { /* ArchitectureJSON */ }
}
```

**Implementation**:
- Go reads `.sruja` file
- Parses DSL → AST → JSON
- Returns JSON to Studio

### 2. Save File

```http
POST /api/files/:path
Content-Type: application/json

{
  "json": { /* ArchitectureJSON */ }
}
```

**Response**:
```json
{
  "success": true,
  "path": "architecture.sruja"
}
```

**Implementation**:
- Go receives JSON
- Converts JSON → AST → DSL
- Writes `.sruja` file directly

### 3. List Files

```http
GET /api/files?dir=.
```

**Response**:
```json
{
  "files": [
    { "path": "architecture.sruja", "type": "file" },
    { "path": "requirements.sruja", "type": "file" },
    { "path": "decisions.sruja", "type": "file" }
  ]
}
```

### 4. Create File

```http
POST /api/files
Content-Type: application/json

{
  "path": "new-architecture.sruja",
  "json": { /* ArchitectureJSON */ }
}
```

### 5. Delete File

```http
DELETE /api/files/:path
```

## Workflow

### Opening Studio

```bash
sruja studio
# Starts:
# 1. Go API server on http://localhost:5173
# 2. Serves React app at http://localhost:5173
# 3. Opens browser automatically
```

### Loading File

1. User clicks "Open File" in Studio
2. Studio shows file picker (or list of `.sruja` files)
3. User selects `architecture.sruja`
4. Studio calls `GET /api/files/architecture.sruja`
5. Go reads file, converts to JSON, returns to Studio
6. Studio renders diagram

### Editing

1. User edits in Studio (adds/modifies elements)
2. Studio updates JSON in memory
3. Changes are reflected in diagram immediately

### Saving

1. User clicks "Save" (or auto-save)
2. Studio calls `POST /api/files/architecture.sruja` with JSON
3. Go converts JSON → DSL
4. Go writes `.sruja` file directly
5. Studio shows "Saved" confirmation

### Creating Change

1. User creates change in Studio
2. User clicks "Save as Change"
3. Studio calls `POST /api/changes` with change data
4. Go creates `changes/001-add-analytics.sruja`
5. File is written directly

## Implementation

### Bundling Studio App into Binary

The Studio React app is **embedded directly into the Go binary** using Go's `embed` directive. This means:
- ✅ **No source code needed** - Users just need the binary
- ✅ **Single binary** - Everything is self-contained
- ✅ **Works offline** - No external dependencies
- ✅ **Easy distribution** - Just download the binary

### Build Process

1. **Build Local Studio React app** (during development):
   ```bash
   cd local-studio
   npm run build
   # Outputs to: local-studio/dist/
   ```

2. **Embed in Go binary**:
   ```go
   // cmd/sruja/studio.go
   package main
   
   import (
       "embed"
       "io/fs"
       "net/http"
   )
   
   //go:embed studio-dist/*
   var studioFiles embed.FS
   
   func runStudio(cmd *cobra.Command, args []string) error {
       // Start HTTP server
       mux := http.NewServeMux()
       
       // Serve embedded static files (React app)
       studioFS, _ := fs.Sub(studioFiles, "studio-dist")
       mux.Handle("/", http.FileServer(http.FS(studioFS)))
       
       // API endpoints
       mux.HandleFunc("/api/files/", handleFiles)
       mux.HandleFunc("/api/migrations", handleMigrations)
       
       // Start server
       log.Printf("Studio running at http://localhost:5173")
       return http.ListenAndServe(":5173", mux)
   }
   ```

3. **Build binary**:
   ```bash
   # Build Studio app first
   cd learn && npm run build && cd ..
   
   # Build Go binary (includes embedded Studio)
   go build -o sruja cmd/sruja/*.go
   ```

### Go API Server

```go
// cmd/sruja/studio.go
package main

import (
    "embed"
    "io/fs"
    "net/http"
    "log"
)

//go:embed studio-dist/*
var studioFiles embed.FS

func runStudio(cmd *cobra.Command, args []string) error {
    // Start HTTP server
    mux := http.NewServeMux()
    
    // Serve embedded static files (React app)
    studioFS, err := fs.Sub(studioFiles, "studio-dist")
    if err != nil {
        return fmt.Errorf("failed to load studio files: %w", err)
    }
    mux.Handle("/", http.FileServer(http.FS(studioFS)))
    
    // API endpoints
    mux.HandleFunc("/api/files/", handleFiles)
    mux.HandleFunc("/api/changes", handleChanges)
    
    // Start server
    log.Printf("Studio running at http://localhost:5173")
    log.Printf("Open http://localhost:5173 in your browser")
    return http.ListenAndServe(":5173", mux)
}

func handleFiles(w http.ResponseWriter, r *http.Request) {
    path := r.URL.Path[len("/api/files/"):]
    
    switch r.Method {
    case "GET":
        // Read .sruja file, convert to JSON
        json := loadAndConvertToJSON(path)
        json.NewEncoder(w).Encode(json)
    case "POST":
        // Receive JSON, convert to DSL, write file
        var req struct {
            JSON ArchitectureJSON `json:"json"`
        }
        json.NewDecoder(r.Body).Decode(&req)
        saveJSONToFile(path, req.JSON)
    }
}
```

### Studio Frontend

```typescript
// components/studio/Studio.tsx
const loadFile = async (path: string) => {
  const response = await fetch(`/api/files/${path}`);
  const data = await response.json();
  setArchitecture(data.json);
};

const saveFile = async (path: string, json: ArchitectureJSON) => {
  await fetch(`/api/files/${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ json }),
  });
};
```

## Benefits

✅ **No export step** - Direct file operations  
✅ **Real-time save** - Changes saved immediately  
✅ **Simple workflow** - Open, edit, save (like any editor)  
✅ **No WASM needed** - Go handles conversions server-side  
✅ **Familiar UX** - Works like VS Code or any file editor  

## File Operations

All file operations go through the Go API:
- **Read**: Go reads `.sruja`, converts to JSON
- **Write**: Go receives JSON, converts to DSL, writes file
- **List**: Go scans directory for `.sruja` files
- **Create**: Go creates new `.sruja` file
- **Delete**: Go deletes `.sruja` file

Studio never directly accesses the file system - all operations go through the API.

## Security

- Only serves files in current directory (or specified workspace)
- No access to files outside workspace
- Validates file paths (no `../` traversal)
- Only accepts `.sruja` files

## Build Integration

### Makefile Target

```makefile
# Build Studio app and embed in binary
.PHONY: build-studio
build-studio:
	cd learn && npm run build
	@echo "Studio app built and ready for embedding"

# Build binary with embedded Studio
.PHONY: build
build: build-studio
	go build -o bin/sruja cmd/sruja/*.go
```

### CI/CD Integration

The build process in CI/CD:
1. Build Studio React app (`npm run build`)
2. Copy built files to `cmd/sruja/studio-dist/`
3. Build Go binary (embeds `studio-dist/`)
4. Binary contains everything - no external files needed

### Directory Structure

```
cmd/sruja/
├── studio.go          # Studio command + embedded files
├── studio-dist/       # Built React app (gitignored, generated)
│   ├── index.html
│   ├── assets/
│   │   ├── main.js
│   │   └── main.css
│   └── ...
└── ...
```

**Note**: `studio-dist/` is generated during build, not committed to git.

## Benefits of Embedding

✅ **Single binary** - Everything included  
✅ **No source code needed** - Users just download binary  
✅ **Works offline** - No external dependencies  
✅ **Easy distribution** - Just one file to download  
✅ **Version consistency** - Studio version matches CLI version  
✅ **No file system dependencies** - Works from any directory  

## Next Steps

1. Set up build process to generate `studio-dist/`
2. Implement Go API server with embedded files
3. Add API endpoints for file operations
4. Update Studio frontend to use API
5. Test binary distribution (verify Studio works without source)

