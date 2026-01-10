package json

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// buildSrujaExtensionsFromProgram extracts Sruja extensions from Program
func (e *Exporter) buildSrujaExtensionsFromProgram(program *language.Program) *SrujaExtensions {
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
		// Governance elements are now parsed through ElementDef with specific Kinds
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			switch a.Kind {
			case "requirement", "Requirement":
				hasContent = true
				ext.Requirements = append(ext.Requirements, RequirementDump{
					ID:          a.Name,
					Title:       ptrToString(a.Title),
					Type:        ptrToString(a.SubKind),
					Description: ptrToString(a.Title),
				})
			case "adr", "Adr", "ADR":
				hasContent = true
				adr := ADRDump{
					ID:    a.Name,
					Title: ptrToString(a.Title),
				}
				// Extract body fields if present
				if a.Body != nil {
					for _, bodyItem := range a.Body.Items {
						if bodyItem.Metadata != nil {
							for _, entry := range bodyItem.Metadata.Entries {
								switch entry.Key {
								case "status":
									adr.Status = ptrToString(entry.Value)
								case "context":
									adr.Context = ptrToString(entry.Value)
								case "decision":
									adr.Decision = ptrToString(entry.Value)
								case "consequences":
									adr.Consequences = ptrToString(entry.Value)
								}
							}
						}
					}
				}
				ext.ADRs = append(ext.ADRs, adr)
			case "policy", "Policy":
				hasContent = true
				pol := PolicyDump{
					ID:    a.Name,
					Title: ptrToString(a.Title),
				}
				// Extract category and enforcement from body
				if a.Body != nil {
					for _, bodyItem := range a.Body.Items {
						if bodyItem.Metadata != nil {
							for _, entry := range bodyItem.Metadata.Entries {
								switch entry.Key {
								case "category":
									pol.Category = ptrToString(entry.Value)
								case "enforcement":
									pol.Enforcement = ptrToString(entry.Value)
								}
							}
						}
					}
				}
				ext.Policies = append(ext.Policies, pol)
			case "scenario", "Scenario", "story", "Story":
				hasContent = true
				sc := ScenarioDump{
					ID:          a.Name,
					Title:       ptrToString(a.Title),
					Description: "", // Description can be extracted from body if needed
				}
				// Extract steps from body if present (for inline scenarios)
				if a.Body != nil {
					for _, bodyItem := range a.Body.Items {
						if bodyItem.Step != nil {
							sc.Steps = append(sc.Steps, convertScenarioStepToStepDump(bodyItem.Step))
						}
					}
				}
				ext.Scenarios = append(ext.Scenarios, sc)
			case "flow", "Flow":
				hasContent = true
				flow := FlowDump{
					ID:          a.Name,
					Title:       ptrToString(a.Title),
					Description: "", // Description can be extracted from body if needed
				}
				// Extract steps from body if present (for inline flows)
				if a.Body != nil {
					for _, bodyItem := range a.Body.Items {
						if bodyItem.Step != nil {
							flow.Steps = append(flow.Steps, convertScenarioStepToStepDump(bodyItem.Step))
						}
					}
				}
				ext.Flows = append(ext.Flows, flow)
			}
		}

		// Handle top-level scenarios
		if item.Scenario != nil {
			hasContent = true
			sc := ScenarioDump{
				ID:          item.Scenario.ID,
				Title:       ptrToString(item.Scenario.Title),
				Description: ptrToString(item.Scenario.Description),
			}
			// Extract steps from scenario.Steps (populated during PostProcess)
			for _, step := range item.Scenario.Steps {
				sc.Steps = append(sc.Steps, convertScenarioStepToStepDump(step))
			}
			ext.Scenarios = append(ext.Scenarios, sc)
		}

		// Handle top-level flows
		if item.Flow != nil {
			hasContent = true
			flow := FlowDump{
				ID:          item.Flow.ID,
				Title:       ptrToString(item.Flow.Title),
				Description: ptrToString(item.Flow.Description),
			}
			// Extract steps from flow.Steps (populated during PostProcess)
			for _, step := range item.Flow.Steps {
				flow.Steps = append(flow.Steps, convertScenarioStepToStepDump(step))
			}
			ext.Flows = append(ext.Flows, flow)
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

func (e *Exporter) buildSpecification(program *language.Program) SpecificationDump {
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

// convertScenarioStepToStepDump converts a language.ScenarioStep to StepDump format
func convertScenarioStepToStepDump(step *language.ScenarioStep) StepDump {
	stepDump := StepDump{
		Description: ptrToString(step.Description),
	}

	// Convert From QualifiedIdent to string
	if len(step.From.Parts) > 0 {
		stepDump.From = step.From.String()
	} else if len(step.FromParts) > 0 {
		// Fallback to FromParts if From is not populated (shouldn't happen after PostProcess, but be safe)
		fromQualified := language.QualifiedIdent{Parts: step.FromParts}
		stepDump.From = fromQualified.String()
	}

	// Convert To QualifiedIdent to string
	if len(step.To.Parts) > 0 {
		stepDump.To = step.To.String()
	} else if len(step.ToParts) > 0 {
		// Fallback to ToParts if To is not populated (shouldn't happen after PostProcess, but be safe)
		toQualified := language.QualifiedIdent{Parts: step.ToParts}
		stepDump.To = toQualified.String()
	}

	return stepDump
}
