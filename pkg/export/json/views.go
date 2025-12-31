package json

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// convertViewsFromProgram converts Views to ViewDump
func (e *Exporter) convertViewsFromProgram(dump *SrujaModelDump, program *language.Program) {
	if program == nil {
		return
	}
	// Default index view with a wildcard include rule
	dump.Views["index"] = ViewDump{
		ID:    "index",
		Title: "Index",
		Rules: []ViewRule{
			{
				Include: &ViewRuleExpr{Wildcard: true},
			},
		},
		Nodes: []NodeDump{},
		Edges: []EdgeDump{},
	}

	// Add L1, L2, L3 default views if they don't exist
	// L1: System Landscape
	dump.Views["L1"] = ViewDump{
		ID:    "L1",
		Title: "Landscape View (L1)",
		Rules: []ViewRule{
			{
				Include: &ViewRuleExpr{Wildcard: true},
			},
		},
		Nodes: []NodeDump{},
		Edges: []EdgeDump{},
	}

	// L2: Container View
	dump.Views["L2"] = ViewDump{
		ID:    "L2",
		Title: "Container View (L2)",
		Rules: []ViewRule{
			{
				Include: &ViewRuleExpr{Wildcard: true},
			},
		},
		Nodes: []NodeDump{},
		Edges: []EdgeDump{},
	}

	// L3: Component View
	dump.Views["L3"] = ViewDump{
		ID:    "L3",
		Title: "Component View (L3)",
		Rules: []ViewRule{
			{
				Include: &ViewRuleExpr{Wildcard: true},
			},
		},
		Nodes: []NodeDump{},
		Edges: []EdgeDump{},
	}

	if program.Views != nil {
		for i, item := range program.Views.Items {
			if item.View == nil {
				continue
			}
			v := item.View
			viewID := "index"
			if v.Name != nil {
				viewID = *v.Name
			}
			if viewID == "" {
				viewID = fmt.Sprintf("view_%d", i)
			}
			viewTitle := viewID
			if v.Title != nil {
				viewTitle = *v.Title
			}
			viewOf := ""
			if v.Of != nil {
				viewOf = v.Of.String()
			}

			// Capture rules if body is present
			var rules []ViewRule
			var layoutPositions map[string]map[string]interface{}
			if v.Body != nil {
				for _, bitem := range v.Body.Items {
					if bitem.Include != nil {
						expr := &ViewRuleExpr{}
						for _, vexpr := range bitem.Include.Expressions {
							switch {
							case vexpr.Wildcard:
								expr.Wildcard = true
							case vexpr.Recursive:
								expr.Recursive = true
							case vexpr.Selector != nil:
								expr.Elements = append(expr.Elements, *vexpr.Selector)
							}
						}
						rules = append(rules, ViewRule{Include: expr})
					}
					if bitem.Exclude != nil {
						expr := &ViewRuleExpr{}
						for _, vexpr := range bitem.Exclude.Expressions {
							switch {
							case vexpr.Wildcard:
								expr.Wildcard = true
							case vexpr.Recursive:
								expr.Recursive = true
							case vexpr.Selector != nil:
								expr.Elements = append(expr.Elements, *vexpr.Selector)
							}
						}
						rules = append(rules, ViewRule{Exclude: expr})
					}
					// Capture layout positions if present
					if bitem.Layout != nil {
						if layoutPositions == nil {
							layoutPositions = make(map[string]map[string]interface{})
						}
						for _, elLayout := range bitem.Layout.Elements {
							if elLayout.Position != nil {
								elementID := elLayout.Element.String()
								layoutPositions[elementID] = map[string]interface{}{
									"x": elLayout.Position.X,
									"y": elLayout.Position.Y,
								}
							}
						}
					}
				}
			}

			// If no rules defined in body, add a default wildcard include
			if len(rules) == 0 {
				rules = append(rules, ViewRule{Include: &ViewRuleExpr{Wildcard: true}})
			}

			viewDump := ViewDump{
				ID:          viewID,
				Title:       viewTitle,
				Description: "", // Not capturing description in View struct currently
				ViewOf:      viewOf,
				Tags:        []string{},
				Rules:       rules,
				Nodes:       []NodeDump{}, // Critical: Initialize as empty slice to avoid null
				Edges:       []EdgeDump{}, // Critical: Initialize as empty slice to avoid null
			}

			// Add layout positions if present
			// Store in view metadata format that frontend expects
			// Frontend expects: views[viewKey].layout.positions[nodeId]
			if len(layoutPositions) > 0 {
				// Convert layoutPositions map to ViewLayoutDump format
				positions := make(map[string]ViewPositionDump)
				for elementID, posMap := range layoutPositions {
					// posMap is map[string]interface{} from the loop above
					var x, y float64
					if xVal, ok := posMap["x"].(float64); ok {
						x = xVal
					} else if xVal, ok := posMap["x"].(int); ok {
						x = float64(xVal)
					}
					if yVal, ok := posMap["y"].(float64); ok {
						y = yVal
					} else if yVal, ok := posMap["y"].(int); ok {
						y = float64(yVal)
					}
					positions[elementID] = ViewPositionDump{X: x, Y: y}
				}
				viewDump.Layout = &ViewLayoutDump{
					Positions: positions,
				}
			}

			dump.Views[viewID] = viewDump
		}
	}
}
