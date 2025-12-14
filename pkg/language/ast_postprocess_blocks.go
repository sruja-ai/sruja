// pkg/language/ast_postprocess_blocks.go
// Post-processing methods for blocks (DeploymentNode, Scenario, Policy, Flow, etc.)
package language

// PostProcess populates convenience fields from deployment node items.
func (d *DeploymentNode) PostProcess() {
	for _, item := range d.Items {
		if item.Node != nil {
			item.Node.PostProcess()
			d.Children = append(d.Children, item.Node)
		}
		if item.ContainerInstance != nil {
			d.ContainerInstances = append(d.ContainerInstances, item.ContainerInstance)
		}
		if item.Infrastructure != nil {
			d.Infrastructure = append(d.Infrastructure, item.Infrastructure)
		}
	}
}

// PostProcess populates convenience fields from scenario items.
func (s *Scenario) PostProcess() {
	for _, item := range s.Items {
		if item.Step != nil {
			item.Step.From = QualifiedIdent{Parts: item.Step.FromParts}
			item.Step.To = QualifiedIdent{Parts: item.Step.ToParts}
			s.Steps = append(s.Steps, item.Step)
		}
	}
}

// PostProcess extracts fields from PolicyBody or inline fields to top-level Policy fields.
func (p *Policy) PostProcess() {
	p.Category = p.InlineCategory
	p.Enforcement = p.InlineEnforcement

	if p.Body != nil {
		for _, prop := range p.Body.Properties {
			if prop.Category != nil {
				p.Category = prop.Category
				p.Body.Category = prop.Category
			}
			if prop.Enforcement != nil {
				p.Enforcement = prop.Enforcement
				p.Body.Enforcement = prop.Enforcement
			}
			if prop.Description != nil {
				p.Description = *prop.Description
				p.Body.Description = prop.Description
			}
			if len(prop.Tags) > 0 {
				p.Body.Tags = append(p.Body.Tags, prop.Tags...)
			}
			if prop.Metadata != nil {
				p.Metadata = append(p.Metadata, prop.Metadata.Entries...)
			}
		}
	}
}

// PostProcess extracts steps and metadata from Flow.
func (f *Flow) PostProcess() {
	for _, item := range f.Items {
		if item.Step != nil {
			item.Step.From = QualifiedIdent{Parts: item.Step.FromParts}
			item.Step.To = QualifiedIdent{Parts: item.Step.ToParts}
			f.Steps = append(f.Steps, item.Step)
		}
	}
}

// PostProcess populates convenience fields from requirement body.
func (r *Requirement) PostProcess() {
	if r.Body != nil {
		for _, prop := range r.Body.Properties {
			if prop.Type != nil {
				r.Type = prop.Type
				r.Body.Type = prop.Type
			}
			if prop.Description != nil {
				r.Description = prop.Description
				r.Body.Description = prop.Description
			}
			if len(prop.Tags) > 0 {
				r.Body.Tags = append(r.Body.Tags, prop.Tags...)
			}
			if prop.Metadata != nil {
				r.Metadata = append(r.Metadata, prop.Metadata.Entries...)
			}
		}
		if r.Body.Type != nil {
			r.Type = r.Body.Type
		}
		if r.Body.Description != nil {
			r.Description = r.Body.Description
		}
	}
}

// PostProcess populates items for SharedArtifact.
func (s *SharedArtifact) PostProcess() {
	// SharedArtifact doesn't have a PostProcess method signature requirement but good for consistency
}

// PostProcess populates ADR fields from unordered properties.
func (a *ADR) PostProcess() {
	if a.Body != nil {
		for _, prop := range a.Body.Properties {
			if prop.Status != nil {
				a.Body.Status = prop.Status
			}
			if prop.Context != nil {
				a.Body.Context = prop.Context
			}
			if prop.Decision != nil {
				a.Body.Decision = prop.Decision
			}
			if prop.Consequences != nil {
				a.Body.Consequences = prop.Consequences
			}
			if len(prop.Tags) > 0 {
				a.Body.Tags = append(a.Body.Tags, prop.Tags...)
			}
		}
	}
}

// PostProcess populates fields for ScaleBlock.
func (s *ScaleBlock) PostProcess() {
	for _, item := range s.Items {
		if item.Min != nil {
			val := item.Min.Val
			s.Min = &val
		}
		if item.Max != nil {
			val := item.Max.Val
			s.Max = &val
		}
		if item.Metric != nil {
			val := item.Metric.Val
			s.Metric = &val
		}
	}
}
