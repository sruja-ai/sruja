// Package compiler provides compilation from model to various diagram formats.
package compiler

import (
	"fmt"
	"os"

	"github.com/sruja-ai/sruja/pkg/language"
)

// D2Renderer renders D2 code to various output formats (SVG, PNG, PDF).
//
// This uses the official terrastruct/d2 libraries to render D2 diagrams.
// Only free/open-source layout engines are supported (dagre, elk).
type D2Renderer struct {
	Theme  string // Theme ID (e.g., "neutral-default", "gruvbox-dark")
	Layout string // Layout engine (dagre, elk) - only free engines supported
}

// RenderOptions configures D2 rendering.
type RenderOptions struct {
	Theme  string // Theme ID
	Layout string // Layout engine
	Format string // Output format: svg, png, pdf
}

// NewD2Renderer creates a new D2 renderer with default options.
func NewD2Renderer() *D2Renderer {
	return &D2Renderer{
		Theme:  "neutral-default",
		Layout: "dagre",
	}
}

// NewD2RendererWithOptions creates a new D2 renderer with custom options.
func NewD2RendererWithOptions(opts RenderOptions) *D2Renderer {
	theme := opts.Theme
	if theme == "" {
		theme = "neutral-default"
	}
	layout := opts.Layout
	if layout == "" {
		layout = "dagre"
	}
	// Validate layout - only free engines supported
	if layout == "tala" {
		layout = "dagre" // Fallback to dagre if TALA is requested
	}
	return &D2Renderer{
		Theme:  theme,
		Layout: layout,
	}
}

// RenderToSVG renders D2 code to SVG bytes.
//
// This function compiles D2 code and renders it to SVG format.
// It requires the terrastruct/d2 packages to be available.
func (r *D2Renderer) RenderToSVG(d2Code string) ([]byte, error) {
	// Check if D2 libraries are available
	if !r.hasD2Libraries() {
		return nil, fmt.Errorf("D2 rendering libraries not available. Install with: go get github.com/terrastruct/d2/d2lib github.com/terrastruct/d2/d2renderers/d2svg github.com/terrastruct/d2/d2themes/d2themescatalog")
	}

	// Use build tags or conditional compilation to include D2 rendering
	// For now, return an error with instructions
	return nil, fmt.Errorf("D2 rendering requires additional dependencies. See pkg/compiler/d2renderer_impl.go for implementation")
}

// RenderToPNG renders D2 code to PNG bytes.
func (r *D2Renderer) RenderToPNG(d2Code string) ([]byte, error) {
	if !r.hasD2Libraries() {
		return nil, fmt.Errorf("D2 rendering libraries not available")
	}
	return nil, fmt.Errorf("D2 PNG rendering requires additional dependencies")
}

// RenderToPDF renders D2 code to PDF bytes.
func (r *D2Renderer) RenderToPDF(d2Code string) ([]byte, error) {
	if !r.hasD2Libraries() {
		return nil, fmt.Errorf("D2 rendering libraries not available")
	}
	return nil, fmt.Errorf("D2 PDF rendering requires additional dependencies")
}

// RenderToFile renders D2 code to a file in the specified format.
func (r *D2Renderer) RenderToFile(d2Code, outputPath, format string) error {
	var data []byte
	var err error

	switch format {
	case "svg":
		data, err = r.RenderToSVG(d2Code)
	case "png":
		data, err = r.RenderToPNG(d2Code)
	case "pdf":
		data, err = r.RenderToPDF(d2Code)
	default:
		return fmt.Errorf("unsupported format: %s (supported: svg, png, pdf)", format)
	}

	if err != nil {
		return err
	}

	return os.WriteFile(outputPath, data, 0644)
}

// hasD2Libraries checks if D2 rendering libraries are available.
func (r *D2Renderer) hasD2Libraries() bool {
	// This will be implemented with build tags
	// For now, return false to indicate libraries need to be installed
	return false
}

// CompileAndRender compiles a Sruja program to D2 and renders it to the specified format.
func (r *D2Renderer) CompileAndRender(program *language.Program, format string) ([]byte, error) {
	// Compile to D2
	compiler := NewD2Compiler()
	d2Code, err := compiler.Compile(program)
	if err != nil {
		return nil, fmt.Errorf("compilation error: %w", err)
	}

	// Render based on format
	switch format {
	case "svg":
		return r.RenderToSVG(d2Code)
	case "png":
		return r.RenderToPNG(d2Code)
	case "pdf":
		return r.RenderToPDF(d2Code)
	default:
		return nil, fmt.Errorf("unsupported format: %s", format)
	}
}
