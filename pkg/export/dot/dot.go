// Package dot provides Graphviz DOT language export for Sruja diagrams.
package dot

import (
	"github.com/sruja-ai/sruja/pkg/export/views"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Config represents DOT generation configuration.
type Config struct {
	// RankDir specifies layout direction: TB (top-bottom), LR (left-right)
	RankDir string
	// NodeSep specifies minimum horizontal spacing between nodes (in pixels)
	NodeSep int
	// RankSep specifies minimum vertical spacing between ranks (in pixels)
	RankSep int
	// DefaultNodeWidth is the default node width (in pixels)
	DefaultNodeWidth int
	// DefaultNodeHeight is the default node height (in pixels)
	DefaultNodeHeight int
	// UseRankConstraints enables rank=same constraints for alignment
	UseRankConstraints bool
	// UseEdgeWeights enables edge weight attributes
	UseEdgeWeights bool
	// ViewLevel specifies the C4 view level (1=Context, 2=Container, 3=Component)
	ViewLevel int
	// FocusNodeID specifies the node to focus on for L2/L3 views (optional)
	FocusNodeID string
	// ViewID specifies a DSL view definition to use for filtering elements (optional)
	// When set, the view's include/exclude rules are applied instead of level-based filtering
	ViewID string
	// NodeSizes provides explicit size overrides for nodes (ID -> {W, H})
	NodeSizes map[string]struct{ Width, Height float64 }
	// ElementPositions provides explicit position overrides for nodes (ID -> {X, Y})
	// These are manual positions set by the user via layout blocks
	ElementPositions map[string]struct{ X, Y float64 }
	// LayoutStrategy specifies the layout strategy to use
	// Options: "auto" (default), "hierarchical", "radial", "grid"
	LayoutStrategy string
}

// LayoutStrategy constants
const (
	LayoutStrategyAuto         = "auto"
	LayoutStrategyHierarchical = "hierarchical"
	LayoutStrategyRadial       = "radial"
	LayoutStrategyGrid         = "grid"
	LayoutStrategyForce        = "force"
)

// DefaultConfig returns the default DOT configuration.
func DefaultConfig() Config {
	return Config{
		RankDir:            "TB",
		NodeSep:            DefaultNodeSep,
		RankSep:            DefaultRankSep,
		DefaultNodeWidth:   DefaultNodeWidth,
		DefaultNodeHeight:  DefaultNodeHeight,
		UseRankConstraints: true,
		UseEdgeWeights:     true,
		ViewLevel:          1, // Default to L1 (Context view)
		FocusNodeID:        "",
	}
}

// Exporter handles DOT language generation.
type Exporter struct {
	Config Config
}

// NewExporter creates a new DOT exporter.
func NewExporter(config Config) *Exporter {
	return &Exporter{Config: config}
}

// ExportResult contains the results of a DOT export.
type ExportResult struct {
	// DOT is the generated Graphviz DOT string.
	DOT string
	// Elements is the list of visible elements in the view.
	Elements []*views.Element
	// Relations is the list of projected relations in the view.
	Relations []*views.Relation
	// Constraints are the layout constraints used (for testing/debugging).
	Constraints *LayoutConstraints
}

// Export generates a Graphviz DOT result from a program.
func (e *Exporter) Export(prog *language.Program) *ExportResult {
	if prog == nil || prog.Model == nil {
		return &ExportResult{}
	}

	// Use unified ViewEngine
	engine := views.NewViewEngine(views.ViewConfig{
		ViewLevel:   e.Config.ViewLevel,
		FocusNodeID: e.Config.FocusNodeID,
		ViewID:      e.Config.ViewID,
	})
	res := engine.ComputeView(prog)

	if len(res.Elements) == 0 {
		return &ExportResult{}
	}

	// Extract positions from views if not already set in config
	if len(e.Config.ElementPositions) == 0 {
		e.Config.ElementPositions = e.extractPositionsFromViews(prog)
	}

	// Build constraints (FAANG-level constraint-based approach)
	constraints := BuildConstraints(res.Elements, res.Relations, e.Config.ViewLevel, e.Config)

	// Generate DOT from constraints
	dot := GenerateDOTFromConstraints(res.Elements, res.Relations, constraints)

	return &ExportResult{
		DOT:         dot,
		Elements:    res.Elements,
		Relations:   res.Relations,
		Constraints: &constraints,
	}
}

// extractPositionsFromViews extracts manual layout positions from all views in the program.
func (e *Exporter) extractPositionsFromViews(prog *language.Program) map[string]struct{ X, Y float64 } {
	positions := make(map[string]struct{ X, Y float64 })

	if prog.Views == nil || len(prog.Views.Items) == 0 {
		return positions
	}

	// Iterate through all views to find layout blocks
	for _, item := range prog.Views.Items {
		if item == nil || item.View == nil || item.View.Body == nil {
			continue
		}

		// Extract positions from LayoutBlock
		for _, bodyItem := range item.View.Body.Items {
			if bodyItem == nil || bodyItem.Layout == nil {
				continue
			}

			for _, elemLayout := range bodyItem.Layout.Elements {
				if elemLayout == nil || elemLayout.Position == nil {
					continue
				}

				// Get the element ID (FQN)
				elemID := elemLayout.Element.String()
				if elemID == "" {
					continue
				}

				positions[elemID] = struct{ X, Y float64 }{
					X: elemLayout.Position.X(),
					Y: elemLayout.Position.Y(),
				}
			}
		}
	}

	return positions
}
