// pkg/language/ast_postprocess.go
// Package language provides DSL parsing and AST structures.
package language

// ============================================================================
// Post-Processing Methods
// ============================================================================

// PostProcess populates convenience fields from parsed items.
func (a *Architecture) PostProcess() {
	for _, item := range a.Items {
		if item.Import != nil {
			a.Imports = append(a.Imports, item.Import)
		}
		if item.System != nil {
			item.System.PostProcess()
			a.Systems = append(a.Systems, item.System)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			a.Persons = append(a.Persons, item.Person)
		}
		if item.Relation != nil {
			a.Relations = append(a.Relations, item.Relation)
		}
		if item.Requirement != nil {
			a.Requirements = append(a.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			a.ADRs = append(a.ADRs, item.ADR)
		}
		if item.SharedArtifact != nil {
			a.SharedArtifacts = append(a.SharedArtifacts, item.SharedArtifact)
		}
		if item.Library != nil {
			a.Libraries = append(a.Libraries, item.Library)
		}
		if item.Journey != nil {
			item.Journey.PostProcess()
			a.Journeys = append(a.Journeys, item.Journey)
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
		if item.EntitiesBlock != nil {
			a.Entities = append(a.Entities, item.EntitiesBlock.Entities...)
		}
		if item.Domain != nil && item.Domain.EntitiesBlock != nil {
			a.Entities = append(a.Entities, item.Domain.EntitiesBlock.Entities...)
		}
		if item.Domain != nil && item.Domain.EventsBlock != nil {
			a.Events = append(a.Events, item.Domain.EventsBlock.Events...)
		}
		if item.DeploymentNode != nil {
			item.DeploymentNode.PostProcess()
			a.DeploymentNodes = append(a.DeploymentNodes, item.DeploymentNode)
		}
		if item.Scenario != nil {
			item.Scenario.PostProcess()
			a.Scenarios = append(a.Scenarios, item.Scenario)
		}
	}
}

// PostProcess populates convenience fields from system items.
func (s *System) PostProcess() {
	for _, item := range s.Items {
		if item.Description != nil {
			s.Description = item.Description
		}
		if item.Container != nil {
			item.Container.PostProcess()
			s.Containers = append(s.Containers, item.Container)
		}
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
		if item.Requirement != nil {
			s.Requirements = append(s.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			s.ADRs = append(s.ADRs, item.ADR)
		}
		if item.Relation != nil {
			s.Relations = append(s.Relations, item.Relation)
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
		if item.EntitiesBlock != nil {
			s.Entities = append(s.Entities, item.EntitiesBlock.Entities...)
		}
		if item.EventsBlock != nil {
			s.Events = append(s.Events, item.EventsBlock.Events...)
		}
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
func (c *Container) PostProcess() {
	for _, item := range c.Items {
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
		if item.Requirement != nil {
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Relation != nil {
			c.Relations = append(c.Relations, item.Relation)
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
		if item.EntitiesBlock != nil {
			c.Entities = append(c.Entities, item.EntitiesBlock.Entities...)
		}
		if item.EventsBlock != nil {
			c.Events = append(c.Events, item.EventsBlock.Events...)
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
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Relation != nil {
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

// PostProcess populates convenience fields from journey items.
func (j *Journey) PostProcess() {
	for _, item := range j.Items {
		if item.Title != nil {
			j.Title = *item.Title
		}
		if item.Steps != nil {
			j.Steps = append(j.Steps, item.Steps.Items...)
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
			s.Steps = append(s.Steps, item.Step)
		}
	}
}
