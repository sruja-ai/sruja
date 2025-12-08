// pkg/language/ast_postprocess.go
// Package language provides DSL parsing and AST structures.
package language

// extractMetaEntries removed - MetadataBlock now contains MetaEntry directly

// ============================================================================
// Post-Processing Methods
// ============================================================================

// PostProcess populates convenience fields from parsed items.
//
//nolint:funlen,gocyclo // PostProcess is long and complex
func (a *Architecture) PostProcess() {
	for i := range a.Items {
		item := &a.Items[i]
		if item.Import != nil {
			a.Imports = append(a.Imports, item.Import)
		}
		if item.System != nil {
			item.System.PostProcess()
			a.Systems = append(a.Systems, item.System)
		}
		if item.Container != nil {
			item.Container.PostProcess()
			a.Containers = append(a.Containers, item.Container)
		}
		if item.Component != nil {
			item.Component.PostProcess()
			a.Components = append(a.Components, item.Component)
		}
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			a.DataStores = append(a.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			a.Queues = append(a.Queues, item.Queue)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			a.Persons = append(a.Persons, item.Person)
		}
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			a.Relations = append(a.Relations, item.Relation)
		}
		if item.Requirement != nil {
			item.Requirement.PostProcess()
			a.Requirements = append(a.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			a.ADRs = append(a.ADRs, item.ADR)
		}
		if item.SharedArtifact != nil {
			a.SharedArtifacts = append(a.SharedArtifacts, item.SharedArtifact)
		}
		if item.Library != nil {
			item.Library.PostProcess()
			a.Libraries = append(a.Libraries, item.Library)
		}
		if item.Metadata != nil {
			a.Metadata = append(a.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			a.Contracts = append(a.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			a.Constraints = append(a.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			a.Conventions = append(a.Conventions, item.ConventionsBlock.Entries...)
		}
		if item.DeploymentNode != nil {
			item.DeploymentNode.PostProcess()
			a.DeploymentNodes = append(a.DeploymentNodes, item.DeploymentNode)
		}
		if item.Scenario != nil {
			item.Scenario.PostProcess()
			a.Scenarios = append(a.Scenarios, item.Scenario)
		}
		if item.Policy != nil {
			item.Policy.PostProcess()
			a.Policies = append(a.Policies, item.Policy)
		}
		if item.Flow != nil {
			item.Flow.PostProcess()
			a.Flows = append(a.Flows, item.Flow)
		}
		if item.Views != nil {
			a.Views = item.Views
		}
		if item.Properties != nil {
			if a.Properties == nil {
				a.Properties = make(map[string]string)
			}
			for _, entry := range item.Properties.Entries {
				a.Properties[entry.Key] = entry.Value
			}
		}
		if item.Style != nil {
			if a.Style == nil {
				a.Style = make(map[string]string)
			}
			for _, entry := range item.Style.Entries {
				a.Style[entry.Key] = entry.Value
			}
		}
		if item.Description != nil {
			a.Description = item.Description
		}
		// View type removed - not in simplified plan
	}

	// Infer implied relationships (DRY principle)
	// If User -> System.Container exists, automatically infer User -> System
	a.inferImpliedRelationships()
}

// inferImpliedRelationships automatically infers parent relationships when child relationships exist.
// This follows the DRY principle: if A -> B.C exists, then A -> B is implied.
//
// Example: If "User -> System.Container" exists, then "User -> System" is automatically inferred.
// This reduces boilerplate while maintaining clarity.
func (a *Architecture) inferImpliedRelationships() {
	// Build a map of existing relationships to avoid duplicates
	existing := make(map[string]bool)
	for _, rel := range a.Relations {
		key := rel.From.String() + "->" + rel.To.String()
		existing[key] = true
	}

	// Collect all relationships from all scopes
	allRelations := []*Relation{}
	allRelations = append(allRelations, a.Relations...)
	for _, sys := range a.Systems {
		allRelations = append(allRelations, sys.Relations...)
		for _, cont := range sys.Containers {
			allRelations = append(allRelations, cont.Relations...)
			for _, comp := range cont.Components {
				allRelations = append(allRelations, comp.Relations...)
			}
		}
	}

	// For each relationship, infer parent relationships
	for _, rel := range allRelations {
		fromParts := rel.From.Parts
		toParts := rel.To.Parts

		// Rule: If A -> B.C exists (where B.C is nested), infer A -> B
		// This only applies when:
		//   1. The "To" side is nested (B.C)
		//   2. The "From" side (A) is not nested within the parent (B)
		// Example: User -> System.Container implies User -> System
		if len(toParts) > 1 {
			parentTo := QualifiedIdent{Parts: toParts[:len(toParts)-1]}

			// Only infer if "From" is not nested within parent of "To"
			// e.g., don't infer Shop -> Shop.API from Shop.WebApp -> Shop.API
			shouldInfer := true
			if len(fromParts) > 0 {
				// Check if From starts with parent of To
				if len(fromParts) >= len(parentTo.Parts) {
					matches := true
					for i := 0; i < len(parentTo.Parts); i++ {
						if fromParts[i] != parentTo.Parts[i] {
							matches = false
							break
						}
					}
					if matches {
						shouldInfer = false // From is nested within parent of To
					}
				}
			}

			if shouldInfer {
				key := rel.From.String() + "->" + parentTo.String()
				if !existing[key] {
					// Create implied relationship
					implied := &Relation{
						From:  rel.From,
						To:    parentTo,
						Label: rel.Label, // Inherit label from child relationship
						Tags:  rel.Tags,  // Inherit tags
						Pos:   rel.Pos,   // Use same position
						Verb:  rel.Verb,  // Inherit verb if present
					}
					a.Relations = append(a.Relations, implied)
					existing[key] = true
				}
			}
		}
	}
}

func normalizeRelation(r *Relation) {
	if r == nil {
		return
	}
	if r.Verb == nil && r.VerbRaw != nil {
		v := r.VerbRaw.Value
		r.Verb = &v
	}
}

// PostProcess populates convenience fields from system items.
//
//nolint:gocyclo // PostProcess is complex
func (s *System) PostProcess() {
	for i := range s.Items {
		item := &s.Items[i]
		if item.Description != nil {
			s.Description = item.Description
		}
		if item.Container != nil {
			item.Container.PostProcess()
			s.Containers = append(s.Containers, item.Container)
		}
		// Component not in SystemItem - removed
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			s.DataStores = append(s.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			s.Queues = append(s.Queues, item.Queue)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			s.Persons = append(s.Persons, item.Person)
		}
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			s.Relations = append(s.Relations, item.Relation)
		}
		if item.Requirement != nil {
			item.Requirement.PostProcess()
			s.Requirements = append(s.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			s.ADRs = append(s.ADRs, item.ADR)
		}
		if item.Metadata != nil {
			s.Metadata = append(s.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			s.Contracts = append(s.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			s.Constraints = append(s.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			s.Conventions = append(s.Conventions, item.ConventionsBlock.Entries...)
		}
		// Context removed - DDD feature, deferred to Phase 2
		if item.Properties != nil {
			if s.Properties == nil {
				s.Properties = make(map[string]string)
			}
			for _, prop := range item.Properties.Entries {
				s.Properties[prop.Key] = prop.Value
			}
		}
		if item.Style != nil {
			if s.Style == nil {
				s.Style = make(map[string]string)
			}
			for _, style := range item.Style.Entries {
				s.Style[style.Key] = style.Value
			}
		}
	}
}

// PostProcess populates convenience fields from container items.
//
//nolint:gocyclo // PostProcess is complex
func (c *Container) PostProcess() {
	for i := range c.Items {
		item := &c.Items[i]
		if item.Description != nil {
			c.Description = item.Description
		}
		if item.Component != nil {
			item.Component.PostProcess()
			c.Components = append(c.Components, item.Component)
		}
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			c.DataStores = append(c.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			c.Queues = append(c.Queues, item.Queue)
		}
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			c.Relations = append(c.Relations, item.Relation)
		}
		if item.Requirement != nil {
			item.Requirement.PostProcess()
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			c.Contracts = append(c.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			c.Constraints = append(c.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			c.Conventions = append(c.Conventions, item.ConventionsBlock.Entries...)
		}
		// Aggregate, ValueObject, Entity, DomainEvent removed - DDD features, deferred to Phase 2
		if item.Properties != nil {
			if c.Properties == nil {
				c.Properties = make(map[string]string)
			}
			for _, prop := range item.Properties.Entries {
				c.Properties[prop.Key] = prop.Value
			}
		}
		if item.Style != nil {
			if c.Style == nil {
				c.Style = make(map[string]string)
			}
			for _, style := range item.Style.Entries {
				c.Style[style.Key] = style.Value
			}
		}
		if item.Scale != nil {
			c.Scale = item.Scale
		}
		if item.Version != nil {
			c.Version = item.Version
		}
	}
}

// PostProcess populates convenience fields from component items.
func (c *Component) PostProcess() {
	for _, item := range c.Items {
		if item.Technology != nil {
			c.Technology = item.Technology
		}
		if item.Description != nil {
			c.Description = item.Description
		}
		if item.Requirement != nil {
			item.Requirement.PostProcess()
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			c.Relations = append(c.Relations, item.Relation)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.Properties != nil {
			if c.Properties == nil {
				c.Properties = make(map[string]string)
			}
			for _, prop := range item.Properties.Entries {
				c.Properties[prop.Key] = prop.Value
			}
		}
		if item.Style != nil {
			if c.Style == nil {
				c.Style = make(map[string]string)
			}
			for _, style := range item.Style.Entries {
				c.Style[style.Key] = style.Value
			}
		}
		if item.Scale != nil {
			c.Scale = item.Scale
		}
	}
}

// PostProcess populates metadata from inline blocks for DataStore.
func (d *DataStore) PostProcess() {
	for _, it := range d.Items {
		if it.Technology != nil {
			d.Technology = it.Technology
		}
		if it.Description != nil {
			d.Description = it.Description
		}
		if it.Metadata != nil {
			d.Metadata = append(d.Metadata, it.Metadata.Entries...)
		}
		if it.Properties != nil {
			if d.Properties == nil {
				d.Properties = make(map[string]string)
			}
			for _, prop := range it.Properties.Entries {
				d.Properties[prop.Key] = prop.Value
			}
		}
		if it.Style != nil {
			if d.Style == nil {
				d.Style = make(map[string]string)
			}
			for _, style := range it.Style.Entries {
				d.Style[style.Key] = style.Value
			}
		}
	}
}

// PostProcess populates metadata from inline blocks for Queue.
func (q *Queue) PostProcess() {
	for _, it := range q.Items {
		if it.Technology != nil {
			q.Technology = it.Technology
		}
		if it.Description != nil {
			q.Description = it.Description
		}
		if it.Metadata != nil {
			q.Metadata = append(q.Metadata, it.Metadata.Entries...)
		}
		if it.Properties != nil {
			if q.Properties == nil {
				q.Properties = make(map[string]string)
			}
			for _, prop := range it.Properties.Entries {
				q.Properties[prop.Key] = prop.Value
			}
		}
		if it.Style != nil {
			if q.Style == nil {
				q.Style = make(map[string]string)
			}
			for _, style := range it.Style.Entries {
				q.Style[style.Key] = style.Value
			}
		}
	}
}

// PostProcess populates metadata from inline blocks for Person.
func (p *Person) PostProcess() {
	for _, it := range p.Items {
		if it.Description != nil {
			p.Description = it.Description
		}
		if it.Metadata != nil {
			p.Metadata = append(p.Metadata, it.Metadata.Entries...)
		}
		if it.Properties != nil {
			if p.Properties == nil {
				p.Properties = make(map[string]string)
			}
			for _, prop := range it.Properties.Entries {
				p.Properties[prop.Key] = prop.Value
			}
		}
		if it.Style != nil {
			if p.Style == nil {
				p.Style = make(map[string]string)
			}
			for _, style := range it.Style.Entries {
				p.Style[style.Key] = style.Value
			}
		}
	}
}

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
			// Construct QualifiedIdent from parts
			item.Step.From = QualifiedIdent{Parts: item.Step.FromParts}
			item.Step.To = QualifiedIdent{Parts: item.Step.ToParts}
			s.Steps = append(s.Steps, item.Step)
		}
	}
}

// DomainBlock.PostProcess removed - DomainBlock is DDD feature, deferred to Phase 2
// DomainBlock structure doesn't match expected fields (no Items, Contexts, Components, Metadata fields)
// If DomainBlock is needed in future, implement proper structure then

// ContextBlock.PostProcess removed - ContextBlock is DDD feature, deferred to Phase 2
// If ContextBlock type is needed in future, implement it then

// Requirement.PostProcess removed - Requirement doesn't have Body field
// Requirement struct only has: ID, Type, Description
// If Requirement.Body is needed in future, add it to ast.go then

// Aggregate.PostProcess removed - Aggregate is DDD feature, deferred to Phase 2
// If Aggregate type is needed in future, implement it then

// Entity.PostProcess removed - Entity is DDD feature, deferred to Phase 2
// Entity structure doesn't match expected fields (no Items, Fields, Metadata fields)
// If Entity is needed in future, implement proper structure then

// ValueObject.PostProcess removed - ValueObject is DDD feature, deferred to Phase 2
// If ValueObject type is needed in future, implement it then

// DomainEvent.PostProcess removed - DomainEvent is DDD feature, deferred to Phase 2
// DomainEvent structure doesn't match expected fields (no Items, Fields, Metadata fields)
// If DomainEvent is needed in future, implement proper structure then

// Policy.PostProcess removed - Policy type not yet defined in ast.go
// TODO: Define Policy type in ast.go, then implement PostProcess
// Policy is architecture construct, should be implemented

// PostProcess extracts fields from PolicyBody or inline fields to top-level Policy fields.
func (p *Policy) PostProcess() {
	// Start with inline values
	p.Category = p.InlineCategory
	p.Enforcement = p.InlineEnforcement

	// Override with body values if present
	if p.Body != nil {
		if p.Body.Category != nil {
			p.Category = p.Body.Category
		}
		if p.Body.Enforcement != nil {
			p.Enforcement = p.Body.Enforcement
		}
		if p.Body.Description != nil {
			p.Description = *p.Body.Description
		}
		if p.Body.Metadata != nil {
			p.Metadata = append(p.Metadata, p.Body.Metadata.Entries...)
		}
	}
}

// PostProcess extracts steps and metadata from Flow.
func (f *Flow) PostProcess() {
	// Flow is an alias to Scenario - extract steps from ScenarioItems
	for _, item := range f.Items {
		if item.Step != nil {
			// Construct QualifiedIdent from parts
			item.Step.From = QualifiedIdent{Parts: item.Step.FromParts}
			item.Step.To = QualifiedIdent{Parts: item.Step.ToParts}
			f.Steps = append(f.Steps, item.Step)
		}
	}
}

// PostProcess populates convenience fields from requirement body.
func (r *Requirement) PostProcess() {
	if r.Body != nil {
		if r.Body.Type != nil {
			r.Type = r.Body.Type
		}
		if r.Body.Description != nil {
			r.Description = r.Body.Description
		}
		if r.Body.Metadata != nil {
			r.Metadata = append(r.Metadata, r.Body.Metadata.Entries...)
		}
	}
}
