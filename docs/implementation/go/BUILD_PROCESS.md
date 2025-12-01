# Studio Build Process: Embedding React App in Binary

## Overview

The Studio React app is embedded directly into the Go binary using Go's `embed` directive. This allows users to run `sruja studio` without needing source code or external files.

## Build Steps

### 1. Build Studio React App

```bash
cd local-studio
npm install
npm run build
# Outputs to: local-studio/dist/
```

### 2. Copy to Embed Location

```bash
# Copy built files to Go embed location
mkdir -p cmd/sruja/studio-dist
cp -r local-studio/dist/* cmd/sruja/studio-dist/
```

### 3. Build Go Binary

```bash
# Build binary (embeds studio-dist/)
go build -o bin/sruja cmd/sruja/*.go
```

The binary now contains the entire Studio app embedded.

## Implementation

### Go Code with Embed

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
    mux := http.NewServeMux()
    
    // Serve embedded files
    studioFS, err := fs.Sub(studioFiles, "studio-dist")
    if err != nil {
        return fmt.Errorf("failed to load studio: %w", err)
    }
    mux.Handle("/", http.FileServer(http.FS(studioFS)))
    
    // API endpoints
    mux.HandleFunc("/api/files/", handleFiles)
    
    return http.ListenAndServe(":5173", mux)
}
```

### Makefile Integration

```makefile
# Build Studio app
.PHONY: build-studio
build-studio:
	@echo "Building Local Studio app..."
	cd local-studio && npm run build
	@echo "Copying Studio files..."
	@mkdir -p cmd/sruja/studio-dist
	@cp -r local-studio/dist/* cmd/sruja/studio-dist/
	@echo "Studio app ready for embedding"

# Build binary with embedded Studio
.PHONY: build
build: build-studio
	@echo "Building Go binary..."
	go build -o bin/sruja cmd/sruja/*.go
	@echo "Binary built: bin/sruja"

# Clean build artifacts
.PHONY: clean-studio
clean-studio:
	rm -rf cmd/sruja/studio-dist
	rm -rf local-studio/dist
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/build.yml
jobs:
  build:
    steps:
      - name: Build Local Studio
        run: |
          cd local-studio
          npm install
          npm run build
          mkdir -p ../cmd/sruja/studio-dist
          cp -r dist/* ../cmd/sruja/studio-dist/
      
      - name: Build Binary
        run: |
          go build -o bin/sruja cmd/sruja/*.go
      
      - name: Test Binary
        run: |
          ./bin/sruja studio --help
```

### Release Process

1. Build Studio app
2. Copy to `cmd/sruja/studio-dist/`
3. Build Go binary (embeds Studio)
4. Release binary (contains everything)

## Directory Structure

```
sruja-lang/
├── cmd/
│   └── sruja/
│       ├── studio.go          # Studio command
│       ├── studio-dist/       # Embedded files (generated, gitignored)
│       │   ├── index.html
│       │   └── assets/
│       └── ...
├── local-studio/
│   ├── package.json
│   ├── dist/                 # Build output (gitignored)
│   └── ...
└── .gitignore
    ├── cmd/sruja/studio-dist/
    └── local-studio/dist/
```

## Verification

### Test Embedded Studio

```bash
# Build binary
make build

# Run Studio (should work without source files)
./bin/sruja studio

# Verify Studio loads
curl http://localhost:5173
# Should return index.html

# Verify API works
curl http://localhost:5173/api/files
```

### Test Binary Distribution

1. Copy binary to clean directory (no source)
2. Run `./sruja studio`
3. Verify Studio loads in browser
4. Verify file operations work

## Benefits

✅ **Single binary** - Everything included  
✅ **No source code needed** - Users just download binary  
✅ **Works offline** - No external dependencies  
✅ **Easy distribution** - Just one file  
✅ **Version consistency** - Studio matches CLI version  
✅ **No file system dependencies** - Works from any directory  

## Troubleshooting

### Studio Not Loading

- Check `studio-dist/` exists and has files
- Verify `embed` directive path matches directory
- Check build process copied files correctly

### Build Fails

- Ensure `npm run build` succeeds first
- Check `studio-dist/` directory exists
- Verify file paths in `embed` directive

### Binary Too Large

- Use compression in React build
- Consider code splitting
- Minify JavaScript/CSS
- Use tree-shaking

## Next Steps

1. Set up build process in Makefile
2. Add CI/CD integration
3. Test binary distribution
4. Document for users


