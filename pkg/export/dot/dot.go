// Package dot provides Graphviz DOT language export for Sruja diagrams.
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

	// Single source of truth for all elements with properties
	allElementsMap := e.extractAllElementsMap(prog)
	allRelations := extractRelationsFromModel(prog)
	lookup := buildElementLookup(prog)

	elements, relations := e.computeViewGraph(allElementsMap, allRelations, lookup)

	if len(elements) == 0 {
		return &ExportResult{}
	}

	// Extract positions from views if not already set in config
	if len(e.Config.ElementPositions) == 0 {
		e.Config.ElementPositions = e.extractPositionsFromViews(prog)
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

// computeViewGraph determines the visible elements and projected relations for the current view.
func (e *Exporter) computeViewGraph(allElements map[string]*Element, allRelations []*Relation, lookup *elementLookup) ([]*Element, []*Relation) {
	level := e.Config.ViewLevel
	focusID := e.Config.FocusNodeID

	visibleIDs := make(map[string]bool)
	var finalElements []*Element
	var finalRelations []*Relation
	seenRel := make(map[string]bool)

	// Helper to normalize element kind (shared logic)
	normalizeKind := func(kind string) string {
		normalized := strings.ToLower(kind)
		switch normalized {
		case "database", "db", "storage":
			return "datastore"
		case "mq":
			return "queue"
		case "actor":
			return "person"
		default:
			return normalized
		}
	}

	// Helper to ensure element is valid (normalize kind, ensure title, ensure dimensions)
	ensureElementValid := func(elem *Element) *Element {
		// Normalize kind (must be lowercase and valid)
		elem.Kind = normalizeKind(elem.Kind)
		// Validate kind is in allowed list
		validKinds := map[string]bool{
			"person": true, "system": true, "container": true,
			"component": true, "datastore": true, "queue": true,
		}
		if !validKinds[elem.Kind] {
			// Fallback to system if kind is invalid
			elem.Kind = "system"
		}
		// Ensure title is not empty (required field)
		if elem.Title == "" {
			elem.Title = elem.ID
		}
		// Ensure dimensions are valid (must be positive numbers)
		if elem.Width <= 0 {
			elem.Width = e.Config.DefaultNodeWidth
		}
		if elem.Height <= 0 {
			elem.Height = e.Config.DefaultNodeHeight
		}
		// Ensure ID is not empty (required field)
		if elem.ID == "" {
			// This shouldn't happen, but add safeguard
			elem.ID = "unknown"
			elem.Title = "Unknown Element"
		}
		return elem
	}

	// Helper to add element if not already added
	addElement := func(id string) {
		if visibleIDs[id] {
			return
		}
		if elem, ok := allElements[id]; ok {
			visibleIDs[id] = true
			// Ensure element is valid before adding
			ensureElementValid(elem)
			finalElements = append(finalElements, elem)
		} else if info, ok := lookup.elements[id]; ok {
			visibleIDs[id] = true
			// Normalize kind
			kind := normalizeKind(info.Kind)
			// Ensure title is not empty (fallback to ID) - must be set before MeasureNodeContent
			title := info.Label
			if title == "" {
				title = info.ID
			}
			// Create element with required fields
			newElem := &Element{
				ID:       info.ID,
				Kind:     kind,
				Title:    title,
				ParentID: info.ParentID,
				// Technology and Description are optional, leave as empty strings
				Technology:  "",
				Description: "",
				Width:       e.Config.DefaultNodeWidth,
				Height:      e.Config.DefaultNodeHeight,
			}
			// Measure actual content if possible (may improve sizing)
			// Title is already set, so measurement should work correctly
			width, height := MeasureNodeContent(newElem)
			newElem.Width = int(width)
			newElem.Height = int(height)
			// Ensure element is valid before adding (double-check all fields)
			ensureElementValid(newElem)
			finalElements = append(finalElements, newElem)
		}
	}

	// 1. Identify Core Set (Internals)
	isCore := func(id string) bool { return false }

	if level == 1 || level == 0 {
		for id, elem := range allElements {
			if elem.Kind == "person" || elem.Kind == "system" {
				addElement(id)
			}
		}
		isCore = func(id string) bool {
			kind := ""
			if el, ok := allElements[id]; ok {
				kind = el.Kind
			} else if info, ok := lookup.elements[id]; ok {
				kind = info.Kind
			}
			return kind == "person" || kind == "system"
		}
	} else if level == 2 {
		if focusID == "" {
			for id, elem := range allElements {
				k := elem.Kind
				if k == "container" || k == "datastore" || k == "queue" || k == "system" || k == "person" {
					addElement(id)
				}
			}
			isCore = func(id string) bool { return true }
		} else {
			addElement(focusID)
			internalPrefix := focusID + "."
			for id, elem := range allElements {
				if strings.HasPrefix(id, internalPrefix) {
					if elem.Kind == "container" || elem.Kind == "datastore" || elem.Kind == "queue" {
						addElement(id)
					}
				}
			}
			isCore = func(id string) bool {
				return id == focusID || strings.HasPrefix(id, internalPrefix)
			}
		}
	} else if level == 3 {
		if focusID == "" {
			for id := range allElements {
				addElement(id)
			}
			isCore = func(id string) bool { return true }
		} else {
			addElement(focusID)
			internalPrefix := focusID + "."
			for id := range allElements {
				if strings.HasPrefix(id, internalPrefix) {
					if elem, ok := allElements[id]; ok && elem.Kind == "component" {
						addElement(id)
					}
				}
			}
			isCore = func(id string) bool {
				return id == focusID || strings.HasPrefix(id, internalPrefix)
			}
		}
	}

	// Helper to resolve short name to FQN based on context
	resolveFQN := func(shortID, contextID string) string {
		// 1. Exact match in allElements
		if _, ok := allElements[shortID]; ok {
			return shortID
		}
		// 2. Context-aware suffix match
		var bestMatch string

		contextScope := ""
		if contextID != "" {
			parts := strings.Split(contextID, ".")
			if len(parts) > 1 {
				contextScope = strings.Join(parts[:len(parts)-1], ".")
			} else {
				contextScope = parts[0]
			}
		}

		for id := range allElements {
			if strings.HasSuffix(id, "."+shortID) {
				// Prefer match in same scope
				if contextScope != "" && strings.HasPrefix(id, contextScope+".") {
					bestMatch = id
					break
				}
				if bestMatch == "" {
					bestMatch = id
				}
			}
		}
		if bestMatch != "" {
			return bestMatch
		}

		// 3. Fallback to lookup check
		if _, ok := lookup.elements[shortID]; ok {
			return shortID
		}
		return shortID
	}

	// 2. Process Relations & Discover Neighbors
	for _, rel := range allRelations {
		// Resolve FQNs first
		fromFQN := resolveFQN(rel.From, "")
		toFQN := resolveFQN(rel.To, fromFQN)

		project := func(fqn string) string {
			if isCore(fqn) {
				return fqn
			}
			// External Projection
			if level == 2 {
				root, _ := lookup.getRoot(fqn)
				return root
			}
			if level == 3 {
				contID := lookup.getContainer(fqn)
				if contID != "" && contID != focusID {
					// Check if container shares same root?
					// Ideally we check if it is "sibling".
					return contID
				}
				root, _ := lookup.getRoot(fqn)
				return root
			}
			root, _ := lookup.getRoot(fqn)
			return root
		}

		source := project(fromFQN)
		target := project(toFQN)

		if source == "" || target == "" || source == target {
			continue
		}

		// Visibility Check: Must connect to Scope
		connectsToScope := false
		if isCore(source) {
			connectsToScope = true
		}
		if isCore(target) {
			connectsToScope = true
		}

		if !connectsToScope && (visibleIDs[source] || visibleIDs[target]) {
			connectsToScope = true
		}

		if !connectsToScope {
			continue
		}

		// Add external nodes
		if !visibleIDs[source] {
			addElement(source)
		}
		if !visibleIDs[target] {
			addElement(target)
		}

		key := fmt.Sprintf("%s->%s:%s", source, target, rel.Label)
		if !seenRel[key] {
			proj := Relation{
				From:  source,
				To:    target,
				Label: rel.Label,
			}
			finalRelations = append(finalRelations, &proj)
			seenRel[key] = true
		}
	}

	// Final validation pass: ensure all elements are valid before returning
	// This catches any elements that might have been missed in earlier validation
	// We validate in-place to maintain element order and indices
	for _, elem := range finalElements {
		// Re-validate each element (ensures all fields are correct)
		ensureElementValid(elem)

		// Double-check critical fields (defensive programming)
		if elem.ID == "" {
			// This should never happen after ensureElementValid, but add safeguard
			elem.ID = fmt.Sprintf("unknown_%d", len(finalElements))
		}
		if elem.Kind == "" {
			elem.Kind = "system" // Fallback
		}
		if elem.Title == "" {
			elem.Title = elem.ID // Fallback
		}
		if elem.Width <= 0 {
			elem.Width = e.Config.DefaultNodeWidth
		}
		if elem.Height <= 0 {
			elem.Height = e.Config.DefaultNodeHeight
		}
	}

	return finalElements, finalRelations
}

// extractAllElementsMap returns a map of all elements keyed by ID.
func (e *Exporter) extractAllElementsMap(prog *language.Program) map[string]*Element {
	list := e.extractAllElements(prog)
	m := make(map[string]*Element)
	for _, el := range list {
		m[el.ID] = el
	}
	return m
}

// getVisibleAncestor finds the closest visible parent for an ID.
// This is a convenience wrapper that calls getVisibleAncestorWithContext without context.
//
//nolint:unused // Kept for API compatibility
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
//
//nolint:unused // Kept for backward compatibility with old writers.go implementation
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
