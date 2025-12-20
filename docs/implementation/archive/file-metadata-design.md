# File Metadata Design

## Problem Statement

JSON should be **self-contained** and **portable**. Having imports in JSON structure creates complications:
- Requires import resolution
- Makes JSON dependent on file system
- Complicates rendering (need to resolve imports)

## Solution: Metadata-Based File Tracking

Instead of imports in JSON structure, we preserve file information via **metadata annotations** on elements.

## JSON Structure

### No Imports Array

**Before** (with imports - problematic):
```json
{
  "architecture": {
    "imports": [
      { "path": "shared.sruja", "alias": null }
    ],
    "systems": [...]
  }
}
```

**After** (self-contained - correct):
```json
{
  "metadata": {
    "sourceFiles": [
      { "path": "main.sruja", "elements": ["System1"] },
      { "path": "shared.sruja", "elements": ["Person1"] }
    ]
  },
  "architecture": {
    "systems": [
      {
        "id": "System1",
        "metadata": {
          "sourceFile": "main.sruja"
        }
      }
    ]
  }
}
```

## Element Metadata

Each element has file source information:

```json
{
  "id": "Person1",
  "label": "User",
  "metadata": {
    "sourceFile": "shared.sruja",      // File where element is defined
    "imported": true,                   // Whether element was imported
    "importedFrom": "shared.sruja"      // Original file (for tracking)
  }
}
```

## DSL → JSON Conversion

1. **Load all files**: Main file + all imported files
2. **Resolve imports**: Merge all elements into single architecture
3. **Flatten elements**: Put all elements in JSON arrays
4. **Add metadata**: Each element gets `metadata.sourceFile`
5. **Track files**: Build `metadata.sourceFiles` mapping

## JSON → DSL Conversion

1. **Read metadata**: Extract file information from element metadata
2. **Group by file**: Group elements by `metadata.sourceFile`
3. **Reconstruct files**: Create file boundaries
4. **Generate imports**: Create import statements based on file grouping
5. **Write files**: Generate separate `.sruja` files

## Studio Visualization

Studio can visualize file boundaries using metadata:

1. **File construct**: Visual grouping by file (optional overlay)
2. **Color coding**: Elements colored by source file
3. **File panel**: Sidebar showing all source files
4. **Import indicators**: Visual marks on imported elements

The "File" construct in Studio is **visual only** - it doesn't affect JSON structure.

## Benefits

✅ **Self-contained JSON**: No external dependencies
✅ **Portable**: JSON can be shared without file system
✅ **Simpler rendering**: No import resolution needed
✅ **File information preserved**: Can reconstruct file organization
✅ **Round-trip safe**: DSL → JSON → DSL preserves file boundaries
