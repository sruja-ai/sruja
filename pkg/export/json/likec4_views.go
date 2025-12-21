package json

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// convertViewsFromProgram converts LikeC4 Views to ViewDump
func (e *LikeC4Exporter) convertViewsFromProgram(dump *SrujaModelDump, program *language.Program) {
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
			if v.Body != nil {
				for _, bitem := range v.Body.Items {
					if bitem.Include != nil {
						expr := &ViewRuleExpr{}
						for _, vexpr := range bitem.Include.Expressions {
							if vexpr.Wildcard {
								expr.Wildcard = true
							} else if vexpr.Recursive {
								expr.Recursive = true
							} else if vexpr.Selector != nil {
								expr.Elements = append(expr.Elements, *vexpr.Selector)
							}
						}
						rules = append(rules, ViewRule{Include: expr})
					}
					if bitem.Exclude != nil {
						expr := &ViewRuleExpr{}
						for _, vexpr := range bitem.Exclude.Expressions {
							if vexpr.Wildcard {
								expr.Wildcard = true
							} else if vexpr.Recursive {
								expr.Recursive = true
							} else if vexpr.Selector != nil {
								expr.Elements = append(expr.Elements, *vexpr.Selector)
							}
						}
						rules = append(rules, ViewRule{Exclude: expr})
					}
				}
			}

			// If no rules defined in body, add a default wildcard include
			if len(rules) == 0 {
				rules = append(rules, ViewRule{Include: &ViewRuleExpr{Wildcard: true}})
			}

			dump.Views[viewID] = ViewDump{
				ID:          viewID,
				Title:       viewTitle,
				Description: "", // Not capturing description in View struct currently
				ViewOf:      viewOf,
				Tags:        []string{},
				Rules:       rules,
				Nodes:       []NodeDump{}, // Critical: Initialize as empty slice to avoid null
				Edges:       []EdgeDump{}, // Critical: Initialize as empty slice to avoid null
			}
		}
	}
}
