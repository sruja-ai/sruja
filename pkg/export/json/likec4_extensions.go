package json

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

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
