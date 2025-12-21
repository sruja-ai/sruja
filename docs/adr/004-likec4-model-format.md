# ADR 004: LikeC4 Model Format

## Status

Accepted

## Context

Sruja needed a model format for architecture diagrams that:
- Supports C4 model concepts
- Is extensible for Sruja-specific features
- Has good tooling support
- Can be rendered in browsers
- Supports programmatic manipulation

We evaluated several options including custom formats, Mermaid, PlantUML, and LikeC4.

## Decision

We use LikeC4 model format as the foundation, extending it with Sruja-specific features.

Format:
- **Base**: LikeC4 model format (`LikeC4ModelDump`)
- **Extensions**: Sruja-specific fields (`sruja` field in model)
- **Compatibility**: Can be consumed by LikeC4 tools
- **Rendering**: Uses `@likec4/diagram` for rendering

## Consequences

### Positive

- **Tooling**: Leverages existing LikeC4 ecosystem
- **Rendering**: High-quality diagram rendering out of the box
- **Compatibility**: Can use LikeC4 tools and viewers
- **Extensibility**: Can add Sruja features without breaking compatibility
- **Community**: Benefits from LikeC4 community improvements

### Negative

- **Dependency**: Depends on LikeC4 library and format
- **Migration**: If LikeC4 changes, we may need to adapt
- **Complexity**: Need to understand LikeC4 format

### Neutral

- **Format Evolution**: LikeC4 format is stable but may evolve
- **Features**: Some Sruja features may not map directly to LikeC4

## Alternatives Considered

1. **Custom Format**: Rejected - too much work, no ecosystem
2. **Mermaid**: Rejected - less flexible, limited programmatic API
3. **PlantUML**: Rejected - text-based, less modern tooling
4. **LikeC4**: Accepted - best balance of features and ecosystem

## References

- LikeC4: https://likec4.dev/
- Model format: `packages/shared/src/types/core.ts`
- Rendering: `apps/designer/src/components/Canvas/LikeC4Canvas.tsx`

