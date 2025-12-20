# Task 4.3: Studio File Operations

**Priority**: 🟡 High (Core functionality)
**Technology**: TypeScript/JavaScript + Go API
**Estimated Time**: 2-3 days
**Dependencies**: Task 4.2, Task 1.2, Go API Server

## Features

* Load `.sruja` files directly via API
* Save `.sruja` files directly via API
* Real-time DSL preview (optional, for preview only)
* No export step needed - direct file operations

## Implementation

**Studio uses Go API server for all file operations**:
- Go API server handles DSL ↔ JSON conversions
- Studio calls API endpoints for read/write operations
- No WASM needed for file operations (only for preview if desired)

**Workflow**:
1. `sruja studio` starts Go API server + serves React app
2. Studio loads file via `GET /api/files/:path` (Go reads `.sruja`, converts to JSON)
3. User edits in Studio (JSON in memory)
4. Studio saves via `POST /api/files/:path` (Go converts JSON to DSL, writes file)

## Implementation

### File Operations via API

```typescript
// apps/studio/src/utils/studio-api.ts
export async function loadFile(path: string): Promise<ArchitectureJSON> {
  const response = await fetch(`/api/files/${path}`);
  const data = await response.json();
  return data.json;
}

export async function saveFile(path: string, json: ArchitectureJSON): Promise<void> {
  await fetch(`/api/files/${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ json }),
  });
}

export async function listFiles(dir: string = '.'): Promise<FileInfo[]> {
  const response = await fetch(`/api/files?dir=${dir}`);
  const data = await response.json();
  return data.files;
}
```

### Studio Component

```typescript
// local-studio/src/components/studio/Studio.tsx
export function Studio() {
  const [architecture, setArchitecture] = useState<ArchitectureJSON | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  
  const handleLoadFile = async (path: string) => {
    const json = await loadFile(path);
    setArchitecture(json);
    setCurrentFile(path);
  };
  
  const handleSave = async () => {
    if (!architecture || !currentFile) return;
    await saveFile(currentFile, architecture);
    // Show "Saved" notification
  };
  
  return (
    <div className="studio-container">
      <StudioToolbar 
        onLoad={handleLoadFile}
        onSave={handleSave}
        currentFile={currentFile}
      />
      {/* ... rest of Studio */}
    </div>
  );
}
```

### Real-time DSL Preview

```typescript
// local-studio/src/components/studio/DSLPreview.tsx
export function DSLPreview({ architecture }: DSLPreviewProps) {
  const [dsl, setDsl] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  
  useEffect(() => {
    if (!architecture) return;
    
    setIsLoading(true);
    exportJSONToDSL(architecture)
      .then(setDsl)
      .finally(() => setIsLoading(false));
  }, [architecture]);
  
  return (
    <div className="dsl-preview">
      {isLoading ? (
        <div>Generating DSL...</div>
      ) : (
        <pre><code>{dsl}</code></pre>
      )}
    </div>
  );
}
```

## Acceptance Criteria

* [ ] Can load `.sruja` files via API
* [ ] Can save `.sruja` files via API
* [ ] File operations work correctly (read/write)
* [ ] DSL ↔ JSON conversions handled by Go API
* [ ] Real-time DSL preview works (optional, for preview only)
* [ ] No WASM needed for file operations
