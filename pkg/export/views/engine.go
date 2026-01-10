package views

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// ViewConfig represents the configuration for filtering a view.
type ViewConfig struct {
	ViewLevel   int    // 1=Context, 2=Container, 3=Component
	FocusNodeID string // Optional node to focus on
	ViewID      string // Optional DSL view definition ID
}

// ViewResult contains the filtered elements and relations for a view.
type ViewResult struct {
	Elements  []*Element
	Relations []*Relation
}

// ViewEngine handles filtering and projection of architectural models.
type ViewEngine struct {
	Config ViewConfig
}

// NewViewEngine creates a new ViewEngine.
func NewViewEngine(config ViewConfig) *ViewEngine {
	return &ViewEngine{Config: config}
}

// ComputeView computes the filtered elements and relations for the given program.
func (e *ViewEngine) ComputeView(prog *language.Program) *ViewResult {
	if prog == nil || prog.Model == nil {
		return &ViewResult{}
	}

	allElements := make(map[string]*Element)
	for _, el := range ExtractAllElements(prog) {
		allElements[el.ID] = el
	}
	allRelations := ExtractRelationsFromModel(prog)
	lookup := BuildElementLookup(prog)

	elements, relations := e.computeViewGraph(prog, allElements, allRelations, lookup)

	return &ViewResult{
		Elements:  elements,
		Relations: relations,
	}
}

func (e *ViewEngine) computeViewGraph(prog *language.Program, allElements map[string]*Element, allRelations []*Relation, lookup *ElementLookup) ([]*Element, []*Relation) {
	level := e.Config.ViewLevel
	focusID := e.Config.FocusNodeID
	viewID := e.Config.ViewID

	if viewID != "" {
		return e.computeViewGraphFromViewDef(prog, viewID, allElements, allRelations, lookup)
	}

	// Auto-detect L2 for single-system diagrams in L1
	if level <= 1 && focusID == "" {
		var systemsWithChildren []string
		var totalSystems int
		for id, elem := range allElements {
			if elem.Kind == "system" {
				totalSystems++
				// Check if this system has any containers, datastores or queues
				hasChildren := false
				for _, other := range allElements {
					if other.ParentID == id {
						hasChildren = true
						break
					}
				}
				if hasChildren {
					systemsWithChildren = append(systemsWithChildren, id)
				}
			}
		}

		// Only auto-switch to L2 if there's exactly one system total (not just one with children)
		// This ensures all systems are shown in L1 when there are multiple systems
		if totalSystems == 1 && len(systemsWithChildren) == 1 {
			e.Config.ViewLevel = 2
			e.Config.FocusNodeID = systemsWithChildren[0]
			// Re-run with new config
			return e.computeViewGraph(prog, allElements, allRelations, lookup)
		}
	}

	visibleIDs := make(map[string]bool)
	var finalElements []*Element
	var finalRelations []*Relation

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

	ensureElementValid := func(elem *Element) *Element {
		elem.Kind = normalizeKind(elem.Kind)
		validKinds := map[string]bool{
			"person": true, "system": true, "container": true,
			"component": true, "datastore": true, "queue": true,
		}
		if !validKinds[elem.Kind] {
			elem.Kind = "system"
		}
		if elem.Title == "" {
			elem.Title = elem.ID
		}
		if elem.Width <= 0 || elem.Height <= 0 {
			w, h := MeasureNodeContent(elem)
			elem.Width = int(w)
			elem.Height = int(h)
		}
		return elem
	}

	addElement := func(id string) {
		if visibleIDs[id] {
			return
		}
		if elem, ok := allElements[id]; ok {
			visibleIDs[id] = true
			ensureElementValid(elem)
			finalElements = append(finalElements, elem)
		} else if info, ok := lookup.Elements[id]; ok {
			visibleIDs[id] = true
			newElem := &Element{
				ID:          info.ID,
				Kind:        normalizeKind(info.Kind),
				Title:       info.Label,
				ParentID:    info.ParentID,
				Technology:  "",
				Description: "",
			}
			ensureElementValid(newElem)
			finalElements = append(finalElements, newElem)
		}
	}

	isCore := func(id string) bool { return false }

	if level <= 1 {
		for id, elem := range allElements {
			if elem.Kind == "person" || elem.Kind == "system" {
				addElement(id)
			}
		}
		isCore = func(id string) bool {
			kind := ""
			if el, ok := allElements[id]; ok {
				kind = el.Kind
			} else if info, ok := lookup.Elements[id]; ok {
				kind = info.Kind
			}
			kind = normalizeKind(kind)
			return kind == "person" || kind == "system"
		}
	} else if level == 2 {
		isL2Element := func(id string) bool {
			kind := ""
			if elem, ok := allElements[id]; ok {
				kind = elem.Kind
			} else if info, ok := lookup.Elements[id]; ok {
				kind = info.Kind
			}
			kind = normalizeKind(kind)
			return kind == "container" || kind == "datastore" || kind == "queue" || kind == "system" || kind == "person"
		}

		if focusID == "" {
			for id := range allElements {
				if isL2Element(id) {
					addElement(id)
				}
			}
			isCore = isL2Element
		} else {
			addElement(focusID)
			internalPrefix := focusID + "."
			for id := range allElements {
				if strings.HasPrefix(id, internalPrefix) && isL2Element(id) {
					addElement(id)
				}
			}
			isCore = func(id string) bool {
				return id == focusID || (strings.HasPrefix(id, internalPrefix) && isL2Element(id))
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

	var projectedRelations []*Relation
	for _, rel := range allRelations {
		fromFQN := lookup.ResolveFQN(rel.From, "")
		toFQN := lookup.ResolveFQN(rel.To, fromFQN)

		project := func(fqn string) string {
			if isCore(fqn) {
				return fqn
			}
			if level == 2 {
				if contID := lookup.GetContainer(fqn); contID != "" {
					return contID
				}
				root, _ := lookup.GetRoot(fqn)
				return root
			}
			if level == 3 {
				contID := lookup.GetContainer(fqn)
				if contID != "" && contID != focusID {
					return contID
				}
				root, _ := lookup.GetRoot(fqn)
				return root
			}
			root, _ := lookup.GetRoot(fqn)
			return root
		}

		source := project(fromFQN)
		target := project(toFQN)

		if source == "" || target == "" || source == target {
			continue
		}

		if strings.HasPrefix(target, source+".") || strings.HasPrefix(source, target+".") {
			continue
		}

		if !isCore(source) && !isCore(target) {
			continue
		}

		addElement(source)
		addElement(target)

		projectedRelations = append(projectedRelations, &Relation{
			From:  source,
			To:    target,
			Label: rel.Label,
		})
	}

	groupedRels := make(map[string][]*Relation)
	for _, rel := range projectedRelations {
		key := rel.From + "->" + rel.To
		groupedRels[key] = append(groupedRels[key], rel)
	}

	for _, group := range groupedRels {
		if len(group) == 0 {
			continue
		}

		if level == 2 && len(group) > 1 {
			first := group[0]
			labels := make([]string, 0, len(group))
			seenLabels := make(map[string]bool)
			for _, r := range group {
				if r.Label != "" && !seenLabels[r.Label] {
					labels = append(labels, r.Label)
					seenLabels[r.Label] = true
				}
			}

			var summaryLabel string
			if len(labels) <= 3 {
				summaryLabel = strings.Join(labels, ", ")
			} else {
				summaryLabel = fmt.Sprintf("%d relations", len(group))
			}

			finalRelations = append(finalRelations, &Relation{
				From:  first.From,
				To:    first.To,
				Label: summaryLabel,
			})
		} else {
			seenExact := make(map[string]bool)
			for _, rel := range group {
				key := fmt.Sprintf("%s->%s:%s", rel.From, rel.To, rel.Label)
				if !seenExact[key] {
					finalRelations = append(finalRelations, rel)
					seenExact[key] = true
				}
			}
		}
	}

	if level == 2 {
		edgeSet := make(map[string]bool)
		for _, rel := range finalRelations {
			edgeSet[rel.From+"->"+rel.To] = true
		}

		filteredRelations := make([]*Relation, 0, len(finalRelations))
		for _, rel := range finalRelations {
			hasMoreSpecificEdge := false
			for edgeKey := range edgeSet {
				if edgeKey != rel.From+"->"+rel.To {
					prefix := rel.From + "->" + rel.To + "."
					if strings.HasPrefix(edgeKey, prefix) {
						hasMoreSpecificEdge = true
						break
					}
					targetSuffix := "->" + rel.To
					if strings.HasSuffix(edgeKey, targetSuffix) {
						parts := strings.Split(edgeKey, "->")
						if len(parts) == 2 && strings.HasPrefix(parts[0], rel.From+".") {
							hasMoreSpecificEdge = true
							break
						}
					}
				}
			}
			if !hasMoreSpecificEdge {
				filteredRelations = append(filteredRelations, rel)
			}
		}
		finalRelations = filteredRelations
	}

	return finalElements, finalRelations
}

func (e *ViewEngine) computeViewGraphFromViewDef(prog *language.Program, viewID string, allElements map[string]*Element, allRelations []*Relation, lookup *ElementLookup) ([]*Element, []*Relation) {
	var viewDef *language.ViewDef
	if prog.Views != nil {
		for _, item := range prog.Views.Items {
			if item != nil && item.View != nil && item.View.Name != nil && *item.View.Name == viewID {
				viewDef = item.View
				break
			}
		}
	}

	if viewDef == nil {
		var finalElements []*Element
		for _, elem := range allElements {
			if elem.Kind == "person" || elem.Kind == "system" {
				finalElements = append(finalElements, elem)
			}
		}
		return finalElements, nil
	}

	includedIDs := make(map[string]bool)
	excludedIDs := make(map[string]bool)

	var scopePrefix string
	if viewDef.Of != nil {
		scopePrefix = viewDef.Of.String()
	}

	if viewDef.Body != nil {
		for _, item := range viewDef.Body.Items {
			if item == nil {
				continue
			}

			if item.Include != nil {
				for _, expr := range item.Include.Expressions {
					if expr.Wildcard {
						if scopePrefix != "" {
							includedIDs[scopePrefix] = true
							prefix := scopePrefix + "."
							for id := range allElements {
								if strings.HasPrefix(id, prefix) {
									includedIDs[id] = true
								}
							}
						} else {
							for id := range allElements {
								includedIDs[id] = true
							}
						}
					} else if expr.Selector != nil {
						selector := expr.String()
						if expr.Sub != nil && (expr.Sub.Wildcard || expr.Sub.Recursive) {
							prefix := *expr.Selector
							includedIDs[prefix] = true
							childPrefix := prefix + "."
							for id := range allElements {
								if strings.HasPrefix(id, childPrefix) {
									includedIDs[id] = true
								}
							}
						} else {
							if _, ok := allElements[selector]; ok {
								includedIDs[selector] = true
							} else {
								if scopePrefix != "" {
									fullID := scopePrefix + "." + selector
									if _, ok := allElements[fullID]; ok {
										includedIDs[fullID] = true
									}
								}
								for id := range allElements {
									if strings.HasSuffix(id, "."+selector) || id == selector {
										includedIDs[id] = true
									}
								}
							}
						}
					}
				}
			}

			if item.Exclude != nil {
				for _, expr := range item.Exclude.Expressions {
					if expr.Wildcard {
						for id := range includedIDs {
							excludedIDs[id] = true
						}
					} else if expr.Selector != nil {
						selector := expr.String()
						excludedIDs[selector] = true
						childPrefix := selector + "."
						for id := range allElements {
							if strings.HasPrefix(id, childPrefix) {
								excludedIDs[id] = true
							}
						}
					}
				}
			}
		}
	}

	if len(includedIDs) == 0 {
		for id, elem := range allElements {
			if elem.Kind == "person" || elem.Kind == "system" {
				includedIDs[id] = true
			}
		}
	}

	for id := range excludedIDs {
		delete(includedIDs, id)
	}

	validKinds := map[string]bool{
		"person": true, "system": true, "container": true,
		"component": true, "datastore": true, "queue": true,
	}

	var finalElements []*Element
	for id := range includedIDs {
		if elem, ok := allElements[id]; ok {
			if !validKinds[elem.Kind] {
				continue
			}
			if elem.Title == "" {
				elem.Title = elem.ID
			}
			if elem.Width <= 0 || elem.Height <= 0 {
				w, h := MeasureNodeContent(elem)
				elem.Width = int(w)
				elem.Height = int(h)
			}
			finalElements = append(finalElements, elem)
		}
	}

	var finalRelations []*Relation
	for _, rel := range allRelations {
		fromFQN := lookup.ResolveFQN(rel.From, "")
		toFQN := lookup.ResolveFQN(rel.To, fromFQN)

		sourceVisible := includedIDs[fromFQN]
		targetVisible := includedIDs[toFQN]

		if !sourceVisible {
			for id := range includedIDs {
				if strings.HasPrefix(fromFQN, id+".") {
					sourceVisible = true
					fromFQN = id
					break
				}
			}
		}
		if !targetVisible {
			for id := range includedIDs {
				if strings.HasPrefix(toFQN, id+".") {
					targetVisible = true
					toFQN = id
					break
				}
			}
		}

		if sourceVisible && targetVisible && fromFQN != toFQN {
			finalRelations = append(finalRelations, &Relation{
				From:  fromFQN,
				To:    toFQN,
				Label: rel.Label,
			})
		}
	}

	seenRelations := make(map[string]bool)
	var uniqueRelations []*Relation
	for _, rel := range finalRelations {
		key := rel.From + "->" + rel.To
		if !seenRelations[key] {
			seenRelations[key] = true
			uniqueRelations = append(uniqueRelations, rel)
		}
	}

	return finalElements, uniqueRelations
}
