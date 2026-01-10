package mermaid

import (
	"github.com/sruja-ai/sruja/pkg/export/views"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Config represents Mermaid diagram configuration.
type Config struct {
	Layout         string
	Theme          string
	Look           string
	Direction      string
	UseFrontmatter bool
	ViewLevel      int    // 1=Context, 2=Container, 3=Component
	TargetID       string // ID of the System (for L2) or Container (for L3) to focus on
}

// DefaultConfig returns the default Mermaid configuration.
func DefaultConfig() Config {
	return Config{
		Layout:    "elk",
		Theme:     "default",
		Direction: "LR",
		ViewLevel: 1,
	}
}

// Exporter handles Mermaid diagram generation.
type Exporter struct {
	Config Config
}

// NewExporter creates a new Mermaid exporter.
func NewExporter(config Config) *Exporter {
	return &Exporter{Config: config}
}

// Export generates a Mermaid diagram from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	// Use unified ViewEngine
	engine := views.NewViewEngine(views.ViewConfig{
		ViewLevel:   e.Config.ViewLevel,
		FocusNodeID: e.Config.TargetID,
	})
	res := engine.ComputeView(prog)

	if len(res.Elements) == 0 {
		return ""
	}

	return e.Generate(res.Elements, res.Relations)
}
