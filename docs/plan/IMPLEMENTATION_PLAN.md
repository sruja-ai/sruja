# Implementation Plan - Sruja Language

## Overview
This document links all implementation components for the Sruja language project, a specialized programming language for creating interactive language learning applications.

## Linked Documentation

### Core Components
1. **[JSON_SCHEMA.md](./JSON_SCHEMA.md)** - Exact field mapping from AST with examples
2. **[EXPORT_HTML.md](./EXPORT_HTML.md)** - Template specifications, CLI commands, and CDN links
3. **[SRUJA_VIEWER_SPEC.md](./SRUJA_VIEWER_SPEC.md)** - JavaScript API, layouts, styling, performance, and tests
4. **[STUDIO_SPEC.md](./STUDIO_SPEC.md)** - Learn app integration with drag-and-drop and DSL export
5. **[TIMELINE.md](./TIMELINE.md)** - Development sprints and acceptance criteria

## Architecture Overview

### Frontend Layer
- **React-based viewer** for rendering Sruja language content
- **Studio interface** for visual language learning creation
- **Export system** for HTML generation

### Backend Layer
- **AST processing** for language parsing
- **JSON schema validation** for data integrity
- **Template rendering** for HTML export

### Integration Points
- **Learn app integration** via static paths under `/learn/static`
- **Studio pages** under `/learn/studio`
- **Drag-and-drop interface** for visual programming
- **DSL export** for code generation

## Implementation Flow

```
User Input → AST Parser → JSON Schema Validation → Viewer Rendering → HTML Export
                    ↓
            Studio Interface ← Drag-and-Drop ← Template System
```

## Key Features to Implement

### Phase 1: Core Language
- [ ] JSON schema definition
- [ ] AST field mapping
- [ ] Basic viewer functionality

### Phase 2: Studio Interface
- [ ] Drag-and-drop components
- [ ] Visual programming blocks
- [ ] DSL export functionality

### Phase 3: Export System
- [ ] HTML template system
- [ ] CLI commands
- [ ] CDN integration

### Phase 4: Integration
- [ ] Learn app compatibility
- [ ] Static asset management
- [ ] Performance optimization

## Success Criteria
- Complete JSON schema with all AST field mappings
- Functional viewer with JavaScript API
- Working studio interface with drag-and-drop
- HTML export capability with templates
- Integration with existing learn app infrastructure
- Performance benchmarks met
- Test coverage above 80%

## Next Steps
Refer to individual documentation files for detailed specifications and implementation details.