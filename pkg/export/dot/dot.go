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
	// NodeSizes provides explicit size overrides for nodes (ID -> {W, H})
	NodeSizes map[string]struct{ Width, Height float64 }
}

// DefaultConfig returns the default DOT configuration.
func DefaultConfig() Config {
	return Config{
		RankDir:            "TB",
		NodeSep:            150, // Increased for better horizontal spacing (was 120, increased to prevent overlaps)
		RankSep:            180, // Increased for better vertical spacing (was 130, increased to prevent overlaps)
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
	// Constraints are the layout constraints used (for testing/debugging).
	Constraints *LayoutConstraints
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

	// Build constraints (FAANG-level constraint-based approach)
	constraints := BuildConstraints(elements, relations, e.Config.ViewLevel, e.Config)

	// Generate DOT from constraints
	dot := GenerateDOTFromConstraints(elements, relations, constraints)

	return &ExportResult{
		DOT:         dot,
		Elements:    elements,
		Relations:   relations,
		Constraints: &constraints,
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
				// Include containers, datastores, queues, and nested systems
				if elem.Kind == "container" || elem.Kind == "datastore" || elem.Kind == "queue" || elem.Kind == "system" {
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
		// Pass the source ID to help resolve short names in the same scope
		source := e.getVisibleAncestorWithContext(rel.From, visible, "")
		// Use the resolved source as context for target resolution (if available)
		// This helps resolve short names in the same scope as the resolved source
		contextForTarget := source
		if contextForTarget == "" {
			contextForTarget = rel.From
		}
		target := e.getVisibleAncestorWithContext(rel.To, visible, contextForTarget)

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
// This is a convenience wrapper that calls getVisibleAncestorWithContext without context.
func (e *Exporter) getVisibleAncestor(id string, visible map[string]bool) string {
	return e.getVisibleAncestorWithContext(id, visible, "")
}

// getVisibleAncestorWithContext finds the closest visible parent for an ID.
// Handles both FQN (e.g., "ragPlatform.llm") and short names (e.g., "llm").
// contextID helps resolve short names by preferring matches within the same scope.
func (e *Exporter) getVisibleAncestorWithContext(id string, visible map[string]bool, contextID string) string {
	// First, try exact match
	if visible[id] {
		return id
	}

	// Try walking up the path (for FQNs like "ragPlatform.gateway.llm" -> "ragPlatform.gateway" -> "ragPlatform")
	parts := strings.Split(id, ".")
	for i := len(parts) - 1; i > 0; i-- {
		parentID := strings.Join(parts[:i], ".")
		if visible[parentID] {
			return parentID
		}
	}

	// If ID has no dots (short name like "llm"), search for visible elements
	// This handles cases where relation uses "llm" but element is "ragPlatform.llm"
	if len(parts) == 1 {
		shortName := id

		// Determine context scope (parent of contextID if available)
		var contextScope string
		if contextID != "" {
			contextParts := strings.Split(contextID, ".")
			if len(contextParts) > 1 {
				contextScope = strings.Join(contextParts[:len(contextParts)-1], ".")
			} else if len(contextParts) == 1 {
				contextScope = contextParts[0]
			}
		}

		// Prefer matches within the same scope
		var sameScopeMatch string
		var otherMatches []string

		for visibleID := range visible {
			if strings.HasSuffix(visibleID, "."+shortName) {
				if contextScope != "" && strings.HasPrefix(visibleID, contextScope+".") {
					sameScopeMatch = visibleID
					break // Prefer first same-scope match
				}
				otherMatches = append(otherMatches, visibleID)
			}
		}

		if sameScopeMatch != "" {
			return sameScopeMatch
		}
		if len(otherMatches) > 0 {
			return otherMatches[0] // Return first match if no same-scope match found
		}

		// Also try exact match of short name if it exists at root level
		if visible[shortName] {
			return shortName
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
