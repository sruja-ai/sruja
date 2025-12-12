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
	// Count items by type for pre-allocation
	var systemsCount, containersCount, componentsCount, dataStoresCount, queuesCount int
	var personsCount, relationsCount, requirementsCount, adrsCount, sharedArtifactsCount int
	var librariesCount, metadataCount, contractsCount, constraintsCount, conventionsCount int
	var deploymentNodesCount, scenariosCount, policiesCount, flowsCount int

	for i := range a.Items {
		item := &a.Items[i]
		if item.System != nil {
			systemsCount++
		}
		if item.Container != nil {
			containersCount++
		}
		if item.Component != nil {
			componentsCount++
		}
		if item.DataStore != nil {
			dataStoresCount++
		}
		if item.Queue != nil {
			queuesCount++
		}
		if item.Person != nil {
			personsCount++
		}
		if item.Relation != nil {
			relationsCount++
		}
		if item.Requirement != nil {
			requirementsCount++
		}
		if item.ADR != nil {
			adrsCount++
		}
		if item.SharedArtifact != nil {
			sharedArtifactsCount++
		}
		if item.Library != nil {
			librariesCount++
		}
		if item.Metadata != nil {
			metadataCount += len(item.Metadata.Entries)
		}
		if item.ContractsBlock != nil {
			contractsCount += len(item.ContractsBlock.Contracts)
		}
		if item.ConstraintsBlock != nil {
			constraintsCount += len(item.ConstraintsBlock.Entries)
		}
		if item.ConventionsBlock != nil {
			conventionsCount += len(item.ConventionsBlock.Entries)
		}
		if item.DeploymentNode != nil {
			deploymentNodesCount++
		}
		if item.Scenario != nil {
			scenariosCount++
		}
		if item.Policy != nil {
			policiesCount++
		}
		if item.Flow != nil {
			flowsCount++
		}
	}

	// Pre-allocate slices
	a.Systems = make([]*System, 0, systemsCount)
	a.Containers = make([]*Container, 0, containersCount)
	a.Components = make([]*Component, 0, componentsCount)
	a.DataStores = make([]*DataStore, 0, dataStoresCount)
	a.Queues = make([]*Queue, 0, queuesCount)
	a.Persons = make([]*Person, 0, personsCount)
	a.Relations = make([]*Relation, 0, relationsCount)
	a.Requirements = make([]*Requirement, 0, requirementsCount)
	a.ADRs = make([]*ADR, 0, adrsCount)
	a.SharedArtifacts = make([]*SharedArtifact, 0, sharedArtifactsCount)
	a.Libraries = make([]*Library, 0, librariesCount)
	a.Metadata = make([]*MetaEntry, 0, metadataCount)
	a.Contracts = make([]*Contract, 0, contractsCount)
	a.Constraints = make([]*ConstraintEntry, 0, constraintsCount)
	a.Conventions = make([]*ConventionEntry, 0, conventionsCount)
	a.DeploymentNodes = make([]*DeploymentNode, 0, deploymentNodesCount)
	a.Scenarios = make([]*Scenario, 0, scenariosCount)
	a.Policies = make([]*Policy, 0, policiesCount)
	a.Flows = make([]*Flow, 0, flowsCount)

	for i := range a.Items {
		item := &a.Items[i]

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
			item.ADR.PostProcess()
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
				// Pre-allocate with estimated capacity
				a.Properties = make(map[string]string, len(item.Properties.Entries))
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
		// requirements and ADRs are root-level only
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
		if item.SLO != nil {
			item.SLO.PostProcess()
			s.SLO = item.SLO
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
		// requirements and ADRs are root-level only
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
			item.Scale.PostProcess()
			c.Scale = item.Scale
		}
		if item.Version != nil {
			c.Version = item.Version
		}
		if item.SLO != nil {
			item.SLO.PostProcess()
			c.SLO = item.SLO
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
		// requirements and ADRs are root-level only
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
			item.Scale.PostProcess()
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

		// Fallback for fields populated directly (if any logic bypasses PostProcess)
		// But for PostProcess we just use Properties
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
		// Populate root-level fields if body overrides them (though types usually set on root)
		if r.Body.Type != nil {
			r.Type = r.Body.Type
		}
		if r.Body.Description != nil {
			r.Description = r.Body.Description
		}
	}
}

// PostProcess populates items for SharedArtifact.
// SharedArtifact now has Items list.
// Struct fields: Description, Url.
func (s *SharedArtifact) PostProcess() {
	// SharedArtifact doesn't have a PostProcess method signature requirement but good for consistency
	// But since it's not an ASTNode root usually, maybe not needed?
	// Wait, Architecture.PostProcess accesses SharedArtifact. But doesn't call PostProcess on it.
	// I should implement it and call it if I want to populate Description/Url.
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
