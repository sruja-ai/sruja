package json

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
)

// LikeC4Exporter converts LikeC4 Program AST to LikeC4-compatible JSON
type LikeC4Exporter struct {
	Extended bool // Include computed views with layout
}

// NewLikeC4Exporter creates a new exporter
func NewLikeC4Exporter() *LikeC4Exporter {
	return &LikeC4Exporter{}
}

// Export converts Program (LikeC4 AST) to LikeC4-compatible JSON string
func (e *LikeC4Exporter) Export(program *language.Program) (string, error) {
	if program == nil {
		return "{}", nil
	}

	dump := e.ToModelDump(program)

	data, err := json.MarshalIndent(dump, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal JSON: %w", err)
	}
	return string(data), nil
}

// ExportCompact exports without indentation
func (e *LikeC4Exporter) ExportCompact(program *language.Program) ([]byte, error) {
	if program == nil {
		return []byte("{}"), nil
	}
	dump := e.ToModelDump(program)
	return json.Marshal(dump)
}

// ToModelDump converts Program (LikeC4 AST) to SrujaModelDump
func (e *LikeC4Exporter) ToModelDump(program *language.Program) *SrujaModelDump {
	modelName := "Untitled"
	if program != nil && program.Model != nil {
		// Try to extract name from filename or use default
		modelName = "Model"
	}

	projectID := modelName
	projectDump := &ProjectDump{
		ID:   projectID,
		Name: modelName,
	}

	dump := &SrujaModelDump{
		Stage:         "parsed", // "parsed" stage - LikeC4 will compute view layouts at runtime
		ProjectID:     projectID,
		Project:       projectDump,
		Globals:       &GlobalsDump{},                 // Empty globals by default
		Imports:       make(map[string][]ElementDump), // Empty imports by default
		Deployments:   &DeploymentsDump{Elements: make(map[string]interface{}), Relations: make(map[string]interface{})},
		Specification: e.buildSpecification(program),
		Elements:      make(map[string]ElementDump),
		Relations:     []RelationDump{},
		Views:         make(map[string]ViewDump),
		Metadata: ModelMetadata{
			Name:      modelName,
			Version:   "1.0.0",
			Generated: time.Now().Format(time.RFC3339),
			SrujaVer:  "2.0.0",
		},
	}

	if program != nil && program.Model != nil {
		// Convert elements (flat with FQN)
		e.convertElementsFromModel(dump, program.Model)

		// Convert relations
		e.convertRelationsFromModel(dump, program.Model)
	}

	// Convert views
	e.convertViewsFromProgram(dump, program)

	// Add Sruja extensions (governance)
	dump.Sruja = e.buildSrujaExtensionsFromProgram(program)

	return dump
}

func (e *LikeC4Exporter) buildSpecification(program *language.Program) SpecificationDump {
	spec := SpecificationDump{
		Elements: map[string]ElementKindDump{
			"person":    {Title: "Person"},
			"system":    {Title: "System"},
			"container": {Title: "Container"},
			"component": {Title: "Component"},
			"database":  {Title: "Database"},
			"queue":     {Title: "Queue"},
		},
	}
	// Add project to specification if available
	if program != nil {
		modelName := "sruja-project"
		if program.Model != nil {
			modelName = "Model"
		}
		spec.Project = &ProjectDump{
			ID:   modelName,
			Name: modelName,
		}
	}
	return spec
}

// convertElementsFromModel converts LikeC4 Model elements to ElementDump
func (e *LikeC4Exporter) convertElementsFromModel(dump *SrujaModelDump, model *language.ModelBlock) {
	if model == nil {
		return
	}

	var convertElement func(elem *language.LikeC4ElementDef, parentFQN string)
	convertElement = func(elem *language.LikeC4ElementDef, parentFQN string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentFQN != "" {
			fqn = parentFQN + "." + id
		}

		// Extract title, description, technology from element
		title := ""
		t := elem.GetTitle()
		if t != nil {
			title = *t
		}
		description := ""
		technology := ""
		var metadata []*language.MetaEntry

		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
				}
				if item.Technology != nil {
					technology = *item.Technology
				}
				if item.Metadata != nil {
					metadata = item.Metadata.Entries
				}
			}
		}

		kind := elem.GetKind()
		elementDump := ElementDump{
			ID:          fqn,
			Kind:        kind,
			Title:       title,
			Description: description,
			Technology:  technology,
			Metadata:    metaToMap(metadata),
			Parent:      parentFQN,
		}

		dump.Elements[fqn] = elementDump

		// Recursively process nested elements
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					convertElement(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			convertElement(item.ElementDef, "")
		}
	}
}

// convertRelationsFromModel converts LikeC4 Model relations to RelationDump
func (e *LikeC4Exporter) convertRelationsFromModel(dump *SrujaModelDump, model *language.ModelBlock) {
	if model == nil {
		return
	}

	relIndex := 0

	// Helper to resolve a QualifiedIdent to FQN based on context
	resolveFQN := func(qid language.QualifiedIdent, contextFQN string) string {
		if contextFQN == "" {
			return qid.String()
		}

		if len(qid.Parts) > 1 {
			firstPart := qid.Parts[0]
			if firstPart == contextFQN || strings.HasPrefix(contextFQN, firstPart+".") {
				return qid.String()
			}
		}

		// Fast path for joining context and qid
		var sb strings.Builder
		sb.Grow(len(contextFQN) + 1 + len(qid.Parts)*8) // Estimate
		sb.WriteString(contextFQN)
		sb.WriteByte('.')
		for i, part := range qid.Parts {
			if i > 0 {
				sb.WriteByte('.')
			}
			sb.WriteString(part)
		}
		return sb.String()
	}

	var collectRelations func(elem *language.LikeC4ElementDef, contextFQN string)
	collectRelations = func(elem *language.LikeC4ElementDef, contextFQN string) {
		if elem == nil {
			return
		}

		// Get the FQN for this element to use as context for nested relations
		var elemFQN string
		id := elem.GetID()
		if id != "" {
			if contextFQN != "" {
				elemFQN = contextFQN + "." + id
			} else {
				elemFQN = id
			}
		} else {
			elemFQN = contextFQN
		}

		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Relation != nil {
					rel := bodyItem.Relation
					fromFQN := resolveFQN(rel.From, elemFQN)
					toFQN := resolveFQN(rel.To, elemFQN)
					dump.Relations = append(dump.Relations, RelationDump{
						ID:     fmt.Sprintf("rel-%d", relIndex),
						Source: NewFqnRef(fromFQN),
						Target: NewFqnRef(toFQN),
						Title:  ptrToString(rel.Label),
					})
					relIndex++
				}
				if bodyItem.Element != nil {
					collectRelations(bodyItem.Element, elemFQN)
				}
			}
		}
	}

	// Pre-allocate relations slice based on model items as a starting capacity
	if len(model.Items) > 0 && len(dump.Relations) == 0 {
		dump.Relations = make([]RelationDump, 0, len(model.Items))
	}

	// Collect top-level relations and relations from elements
	for _, item := range model.Items {
		if item.Relation != nil {
			// Top-level relations are already fully qualified or relative to root
			fromFQN := item.Relation.From.String()
			toFQN := item.Relation.To.String()
			dump.Relations = append(dump.Relations, RelationDump{
				ID:     fmt.Sprintf("rel-%d", relIndex),
				Source: NewFqnRef(fromFQN),
				Target: NewFqnRef(toFQN),
				Title:  ptrToString(item.Relation.Label),
			})
			relIndex++
		}
		if item.ElementDef != nil {
			// Start with empty context for top-level elements
			collectRelations(item.ElementDef, "")
		}
	}
}

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

// buildSrujaExtensionsFromProgram extracts Sruja extensions from Program
func (e *LikeC4Exporter) buildSrujaExtensionsFromProgram(program *language.Program) *SrujaExtensions {
	if program == nil {
		return nil
	}
	ext := &SrujaExtensions{}
	hasContent := false

	if program.Model == nil {
		return nil
	}

	// Extract Requirements, ADRs, Policies, etc. from Model block
	for _, item := range program.Model.Items {
		if item.Requirement != nil {
			hasContent = true
			r := item.Requirement
			ext.Requirements = append(ext.Requirements, RequirementDump{
				ID:          r.ID,
				Title:       ptrToString(r.Description), // Use description as title
				Type:        ptrToString(r.Type),
				Description: ptrToString(r.Description),
			})
		}

		if item.ADR != nil {
			hasContent = true
			a := item.ADR
			adr := ADRDump{
				ID:    a.ID,
				Title: ptrToString(a.Title),
			}
			// Get fields from Body if present
			if a.Body != nil {
				adr.Status = ptrToString(a.Body.Status)
				adr.Context = ptrToString(a.Body.Context)
				adr.Decision = ptrToString(a.Body.Decision)
				adr.Consequences = ptrToString(a.Body.Consequences)
			}
			ext.ADRs = append(ext.ADRs, adr)
		}

		if item.Policy != nil {
			hasContent = true
			p := item.Policy
			ext.Policies = append(ext.Policies, PolicyDump{
				ID:          p.ID,
				Title:       p.Description, // Policy uses Description as main text
				Category:    ptrToString(p.Category),
				Enforcement: ptrToString(p.Enforcement),
			})
		}

		if item.Scenario != nil {
			hasContent = true
			s := item.Scenario
			sc := ScenarioDump{
				ID:    s.ID,
				Title: ptrToString(s.Title),
			}
			for _, step := range s.Steps {
				sc.Steps = append(sc.Steps, StepDump{
					Description: ptrToString(step.Description),
				})
			}
			ext.Scenarios = append(ext.Scenarios, sc)
		}

		if item.Flow != nil {
			hasContent = true
			f := item.Flow
			ext.Flows = append(ext.Flows, FlowDump{
				ID:    f.ID,
				Title: ptrToString(f.Title),
			})
		}

		if item.DeploymentNode != nil {
			hasContent = true
			d := item.DeploymentNode
			ext.Deployments = append(ext.Deployments, DeploymentDump{
				ID:    d.ID,
				Title: d.Label,
			})
		}

		if item.ConstraintsBlock != nil {
			hasContent = true
			for _, c := range item.ConstraintsBlock.Entries {
				ext.Constraints = append(ext.Constraints, ConstraintDump{
					ID:          c.Key,
					Description: c.Value,
				})
			}
		}

		if item.ConventionsBlock != nil {
			hasContent = true
			for _, c := range item.ConventionsBlock.Entries {
				ext.Conventions = append(ext.Conventions, ConventionDump{
					ID:          c.Key,
					Description: c.Value,
				})
			}
		}

		if item.ContractsBlock != nil {
			hasContent = true
			for _, c := range item.ContractsBlock.Contracts {
				ext.Contracts = append(ext.Contracts, ContractDump{
					ID:   c.ID,
					Type: c.Kind,
				})
			}
		}

		if item.Import != nil {
			hasContent = true
			ext.Imports = append(ext.Imports, ImportDump{
				Elements: item.Import.Elements,
				From:     item.Import.From,
			})
		}
	}

	if !hasContent {
		return nil
	}
	return ext
}

// Helper functions
func ptrToString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func metaToMap(meta []*language.MetaEntry) map[string]string {
	if len(meta) == 0 {
		return nil
	}
	m := make(map[string]string)
	for _, e := range meta {
		if e.Value != nil {
			m[e.Key] = *e.Value
		} else if len(e.Array) > 0 {
			m[e.Key] = strings.Join(e.Array, ",")
		}
	}
	return m
}

func extractTechnology(c *language.Container) *string {
	// Look for technology in items
	for _, item := range c.Items {
		if item.Technology != nil {
			return item.Technology
		}
	}
	return nil
}
