// Package dot provides Graphviz DOT language export for Sruja diagrams.
//
// This package generates DOT language output from Sruja's Program AST,
// suitable for layout by Graphviz. The output includes:
// - Hierarchical node structure (persons, systems, containers, components)
// - Relationship edges with labels
// - Rank constraints for proper alignment
// - Layout hints for optimal positioning
package dot

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
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
}

// DefaultConfig returns the default DOT configuration.
func DefaultConfig() Config {
	return Config{
		RankDir:            "TB",
		NodeSep:            80,
		RankSep:            90,
		DefaultNodeWidth:   200,
		DefaultNodeHeight:  120,
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
	Elements []*Element
	// Relations is the list of projected relations in the view.
	Relations []*Relation
}

// Export generates a Graphviz DOT result from a program.
func (e *Exporter) Export(prog *language.Program) *ExportResult {
	if prog == nil || prog.Model == nil {
		return &ExportResult{}
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Extract all elements
	allElements := e.extractAllElements(prog)
	allRelations := extractRelationsFromModel(prog)

	// Apply view projection filtering
	elements := e.filterByView(allElements)
	relations := e.filterRelationsByView(allRelations, elements)

	if len(elements) == 0 {
		return &ExportResult{}
	}

	// Write DOT structure
	e.writeGraphHeader(sb)
	e.writeGlobalNodeAttributes(sb)
	e.writeGlobalEdgeAttributes(sb)

	// Group elements by parent for cluster generation
	rootElements, clusters := e.groupByParent(elements)

	// Write root-level nodes
	for _, elem := range rootElements {
		e.writeNode(sb, elem, "  ")
	}

	// Write clusters (subgraphs)
	for parentID, children := range clusters {
		e.writeCluster(sb, parentID, children, elements)
	}

	// Write rank constraints
	if e.Config.UseRankConstraints {
		e.writeRankConstraints(sb, elements)
	}

	// Write edges
	for _, rel := range relations {
		e.writeEdge(sb, rel)
	}

	sb.WriteString("}\n")

	return &ExportResult{
		DOT:       sb.String(),
		Elements:  elements,
		Relations: relations,
	}
}

// filterByView filters elements based on ViewLevel and FocusNodeID.
func (e *Exporter) filterByView(elements []*Element) []*Element {
	level := e.Config.ViewLevel
	focusID := e.Config.FocusNodeID

	// L1 (Context): Show persons and systems only
	if level == 1 || level == 0 {
		var result []*Element
		for _, elem := range elements {
			if elem.Kind == "person" || elem.Kind == "system" {
				result = append(result, elem)
			}
		}
		return result
	}

	// L2 (Container): Show containers, datastores, queues within focused system
	if level == 2 {
		if focusID == "" {
			// No focus - show all L2 elements
			var result []*Element
			for _, elem := range elements {
				if elem.Kind == "container" || elem.Kind == "datastore" || elem.Kind == "queue" || elem.Kind == "system" || elem.Kind == "person" {
					result = append(result, elem)
				}
			}
			return result
		}
		// Filter to elements within focused system + external systems/persons
		var result []*Element
		for _, elem := range elements {
			// Include elements inside the focused system
			if strings.HasPrefix(elem.ID, focusID+".") {
				if elem.Kind == "container" || elem.Kind == "datastore" || elem.Kind == "queue" {
					result = append(result, elem)
				}
			} else if elem.ID == focusID {
				// Include the focused system itself (often as a cluster, but good to have in list)
				result = append(result, elem)
			} else if elem.Kind == "system" || elem.Kind == "person" {
				// Include external systems and persons
				result = append(result, elem)
			}
		}
		return result
	}

	// L3 (Component): Show components within focused container
	if level == 3 {
		if focusID == "" {
			// No focus - show all components
			var result []*Element
			for _, elem := range elements {
				if elem.Kind == "component" || elem.Kind == "container" || elem.Kind == "system" || elem.Kind == "person" {
					result = append(result, elem)
				}
			}
			return result
		}
		// Filter to elements within focused container + parent containers/systems
		var result []*Element
		for _, elem := range elements {
			if strings.HasPrefix(elem.ID, focusID+".") {
				if elem.Kind == "component" {
					result = append(result, elem)
				}
			} else if elem.ID == focusID {
				result = append(result, elem)
			} else if elem.Kind == "container" || elem.Kind == "system" || elem.Kind == "person" {
				// Include external things
				result = append(result, elem)
			}
		}
		return result
	}

	return elements
}

// filterRelationsByView filters relations to only include those between visible elements (projected).
func (e *Exporter) filterRelationsByView(relations []*Relation, visibleElements []*Element) []*Relation {
	// Build set of visible element IDs
	visible := make(map[string]bool)
	for _, elem := range visibleElements {
		visible[elem.ID] = true
	}

	var result []*Relation
	seen := make(map[string]bool)

	for _, rel := range relations {
		// Find visible ancestors for source and target
		source := e.getVisibleAncestor(rel.From, visible)
		target := e.getVisibleAncestor(rel.To, visible)

		// Create projected relation if both found and different
		if source != "" && target != "" && source != target {
			// Avoid duplicate edges between same nodes with same label
			key := fmt.Sprintf("%s->%s:%s", source, target, rel.Label)
			if !seen[key] {
				projected := *rel
				projected.From = source
				projected.To = target
				result = append(result, &projected)
				seen[key] = true
			}
		}
	}
	return result
}

// getVisibleAncestor finds the closest visible parent for an ID.
func (e *Exporter) getVisibleAncestor(id string, visible map[string]bool) string {
	if visible[id] {
		return id
	}

	parts := strings.Split(id, ".")
	for i := len(parts) - 1; i > 0; i-- {
		parentID := strings.Join(parts[:i], ".")
		if visible[parentID] {
			return parentID
		}
	}

	return ""
}

// Element represents a flattened element for DOT generation.
type Element struct {
	ID          string
	Kind        string // person, system, container, component, datastore, queue
	Title       string
	Technology  string
	Description string
	ParentID    string
	Width       int
	Height      int
}

// pxToInch converts pixels to inches using 72 DPI (Graphviz default).
func pxToInch(px int) float64 {
	return float64(px) / 72.0
}

// escapeID escapes an ID for DOT format.
func escapeID(id string) string {
	return strings.ReplaceAll(id, "\"", "\\\"")
}

// escapeLabel escapes a label string for DOT format.
func escapeLabel(label string) string {
	return strings.ReplaceAll(label, "\"", "\\\"")
}
