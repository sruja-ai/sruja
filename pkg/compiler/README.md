# Sruja Compiler Package

The compiler package provides robust compilation from Sruja DSL to various diagram formats with automatic format selection and rendering capabilities.

## Features

- **Multiple Format Support**: D2, Mermaid (with extensible registry)
- **Automatic Format Selection**: Intelligently chooses the best format based on model characteristics
- **D2 Rendering**: Export D2 diagrams to SVG, PNG, PDF using official D2 libraries
- **Format Recommendations**: Get format suggestions with reasoning
- **Flexible Configuration**: Themes, layouts, and export options

## Usage

### Basic Compilation

```go
import "github.com/sruja-ai/sruja/pkg/compiler"

// Create registry and compile
registry := compiler.NewRegistry()
output, err := registry.Compile("d2", program)
```

### Automatic Format Selection

```go
selector := compiler.NewSelector(registry)

// Get recommendation
rec, err := selector.Recommend(program, "presentation")
// rec.Format = "d2"
// rec.Score = 0.85
// rec.Reasons = ["D2 handles large diagrams well", ...]

// Auto-compile with best format
format, output, err := selector.AutoSelect(program, "documentation")
```

### D2 Rendering (SVG/PNG/PDF)

**Prerequisites**: Install D2 rendering libraries:

```bash
go get github.com/terrastruct/d2/d2lib
go get github.com/terrastruct/d2/d2renderers/d2svg
go get github.com/terrastruct/d2/d2renderers/d2png
go get github.com/terrastruct/d2/d2themes/d2themescatalog
```

Build with rendering support:

```bash
go build -tags d2render
```

**Usage**:

```go
renderer := compiler.NewD2RendererWithOptions(compiler.RenderOptions{
    Theme:  "gruvbox-dark",
    Layout: "dagre",
})

// Render to SVG
svgBytes, err := renderer.RenderToSVG(d2Code)

// Render to PNG
pngBytes, err := renderer.RenderToPNG(d2Code)

// Render to file
err := renderer.RenderToFile(d2Code, "output.svg", "svg")
```

## CLI Usage

### List Available Formats

```bash
sruja compile --format list example.sruja
```

### Automatic Format Selection

```bash
# Auto-select based on model
sruja compile --format auto example.sruja

# Auto-select for specific use case
sruja compile --format auto --use-case presentation example.sruja
```

### Get Format Recommendations

```bash
sruja compile --info example.sruja
```

### Export D2 to SVG/PNG

```bash
# Compile to D2 and export as SVG
sruja compile --format d2 --export svg example.sruja

# With custom theme and layout
sruja compile --format d2 --export svg --theme gruvbox-dark --layout elk example.sruja
```

## Format Selection Logic

The automatic format selector analyzes:

- **Model Complexity**: Number of elements, relations, hierarchical depth
- **Use Case**: presentation, documentation, version-control, export, github, markdown
- **Format Capabilities**: Themes, animations, Git-friendliness, journey support

### D2 is Recommended For:
- Large/complex diagrams (>20 elements)
- Presentation use cases
- Export to images (SVG, PNG, PDF)
- Beautiful themes and animations
- Complex hierarchical structures

### Mermaid is Recommended For:
- Documentation and version control
- GitHub/markdown rendering
- Journey/sequence diagrams
- Moderate-sized diagrams (<30 elements)
- Text-based, Git-friendly format

## Available D2 Themes

- `neutral-default` (default)
- `gruvbox-dark`
- `gruvbox-light`
- `neon`
- `terminal`
- `violet`
- `warm-neon`

## Available D2 Layout Engines

- `dagre` (default) - Fast, hierarchical layouts (free/open-source)
- `elk` - ELK layout engine for complex graphs (free/open-source)

**Note**: TALA layout engine is not supported as it requires a commercial license.

## Extending the Compiler

### Adding a New Format

1. Implement the `Compiler` interface:

```go
type MyCompiler struct{}

func (c *MyCompiler) Name() string { return "myformat" }
func (c *MyCompiler) Compile(program *language.Program) (string, error) {
    // Implementation
}
```

2. Register it:

```go
registry := compiler.NewRegistry()
registry.Register(compiler.NewMyCompiler())
```

### Adding Format Selection Logic

Update `selector.go` in the `recommendFromModel` function to add scoring logic for your format.

## Architecture

```
Compiler Package
├── interface.go      - Compiler interface
├── registry.go       - Format registry and management
├── selector.go       - Automatic format selection
├── transformer.go    - AST to Model transformation
├── d2.go            - D2 compiler implementation
├── d2renderer.go    - D2 rendering interface
├── d2renderer_impl.go - D2 rendering implementation (build tag: d2render)
└── mermaid.go       - Mermaid compiler implementation
```

