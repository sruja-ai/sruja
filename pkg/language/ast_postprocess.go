// pkg/language/ast_postprocess.go
// Package language provides DSL parsing and AST structures.
package language

const policyTypeString = "policy"

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
			a.Relations = append(a.Relations, item.Relation)
		}
		if item.Requirement != nil {
			a.Requirements = append(a.Requirements, item.Requirement)
		}
		if item.Policy != nil {
			item.Policy.PostProcess()
			policyType := policyTypeString
			desc := item.Policy.Description
			req := &Requirement{
				ID:          item.Policy.ID,
				Type:        &policyType,
				Description: desc,
				Rules:       item.Policy.Rules,
			}
			a.Requirements = append(a.Requirements, req)
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
		if item.Context != nil {
			item.Context.PostProcess()
			a.Contexts = append(a.Contexts, item.Context)
		}
		if item.Domain != nil {
			item.Domain.PostProcess()
			a.Domains = append(a.Domains, item.Domain)
			a.Contexts = append(a.Contexts, item.Domain.Contexts...)
		}
		if item.DeploymentNode != nil {
			item.DeploymentNode.PostProcess()
			a.DeploymentNodes = append(a.DeploymentNodes, item.DeploymentNode)
		}
		if item.Scenario != nil {
			item.Scenario.PostProcess()
			a.Scenarios = append(a.Scenarios, item.Scenario)
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
		if item.View != nil {
			a.Views = append(a.Views, item.View)
		}
		if item.Flow != nil {
			item.Flow.PostProcess()
			a.Flows = append(a.Flows, item.Flow)
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
		if item.Component != nil {
			item.Component.PostProcess()
			s.Components = append(s.Components, item.Component)
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
		if item.Policy != nil {
			item.Policy.PostProcess()
			policyType := policyTypeString
			desc := item.Policy.Description
			req := &Requirement{
				ID:          item.Policy.ID,
				Type:        &policyType,
				Description: desc,
				Rules:       item.Policy.Rules,
			}
			s.Requirements = append(s.Requirements, req)
		}
		if item.ADR != nil {
			s.ADRs = append(s.ADRs, item.ADR)
		}
		if item.Scenario != nil {
			item.Scenario.PostProcess()
			s.Scenarios = append(s.Scenarios, item.Scenario)
		}
		if item.Flow != nil {
			item.Flow.PostProcess()
			s.Flows = append(s.Flows, item.Flow)
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
		if item.Context != nil {
			item.Context.PostProcess()
			s.Contexts = append(s.Contexts, item.Context)
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
		if item.Aggregate != nil {
			item.Aggregate.PostProcess()
			c.Aggregates = append(c.Aggregates, item.Aggregate)
		}
		if item.Entity != nil {
			item.Entity.PostProcess()
			c.Entities = append(c.Entities, item.Entity)
		}
		if item.ValueObject != nil {
			item.ValueObject.PostProcess()
			c.ValueObjects = append(c.ValueObjects, item.ValueObject)
		}
		if item.DomainEvent != nil {
			item.DomainEvent.PostProcess()
			c.Events = append(c.Events, item.DomainEvent)
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

// PostProcess populates convenience fields from domain items.
func (d *DomainBlock) PostProcess() {
	for _, item := range d.Items {
		if item.Context != nil {
			item.Context.PostProcess()
			d.Contexts = append(d.Contexts, item.Context)
		}
		if item.Component != nil {
			item.Component.PostProcess()
			d.Components = append(d.Components, item.Component)
		}
		if item.Flow != nil {
			item.Flow.PostProcess()
			d.Flows = append(d.Flows, item.Flow)
		}
		if item.Metadata != nil {
			d.Metadata = append(d.Metadata, item.Metadata.Entries...)
		}
	}
}

// PostProcess populates convenience fields from context items.
func (c *ContextBlock) PostProcess() {
	for _, item := range c.Items {
		if item.Aggregate != nil {
			item.Aggregate.PostProcess()
			c.Aggregates = append(c.Aggregates, item.Aggregate)
		}
		if item.Entity != nil {
			item.Entity.PostProcess()
			c.Entities = append(c.Entities, item.Entity)
		}
		if item.ValueObject != nil {
			item.ValueObject.PostProcess()
			c.ValueObjects = append(c.ValueObjects, item.ValueObject)
		}
		if item.DomainEvent != nil {
			item.DomainEvent.PostProcess()
			c.Events = append(c.Events, item.DomainEvent)
		}
		if item.Component != nil {
			// Components might need their own PostProcess if they have nested items
			// item.Component.PostProcess() // Assuming Component has PostProcess, let's check
			c.Components = append(c.Components, item.Component)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.Requirement != nil {
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.Policy != nil {
			item.Policy.PostProcess()
			policyType := "policy"
			req := &Requirement{
				ID:          item.Policy.ID,
				Type:        &policyType,
				Description: item.Policy.Description,
				Rules:       item.Policy.Rules,
			}
			c.Requirements = append(c.Requirements, req)
		}
	}
}

// PostProcess populates convenience fields from requirement body.
func (r *Requirement) PostProcess() {
	if r.Body != nil {
		if r.Body.Description != nil {
			r.Description = r.Body.Description
		}
		if len(r.Body.Rules) > 0 {
			r.Rules = append(r.Rules, r.Body.Rules...)
		}
	}
}

// PostProcess populates convenience fields from aggregate items.
func (a *Aggregate) PostProcess() {
	for _, item := range a.Items {
		if item.Entity != nil {
			item.Entity.PostProcess()
			a.Entities = append(a.Entities, item.Entity)
		}
		if item.ValueObject != nil {
			item.ValueObject.PostProcess()
			a.ValueObjects = append(a.ValueObjects, item.ValueObject)
		}
		if item.Metadata != nil {
			a.Metadata = append(a.Metadata, item.Metadata.Entries...)
		}
	}
}

// PostProcess populates convenience fields from entity items.
func (e *Entity) PostProcess() {
	for _, item := range e.Items {
		if item.Field != nil {
			e.Fields = append(e.Fields, item.Field)
		}
		if item.ValueObject != nil {
			// Note: We don't have a specific field for nested VOs in Entity yet.
			// For now, we just ensure they are post-processed.
			// In a real implementation, we might want to add `NestedValueObjects` or similar to Entity.
			item.ValueObject.PostProcess()
		}
		if item.Metadata != nil {
			e.Metadata = append(e.Metadata, item.Metadata.Entries...)
		}
	}
}

// PostProcess populates convenience fields from value object items.
func (v *ValueObject) PostProcess() {
	for _, item := range v.Items {
		if item.Field != nil {
			v.Fields = append(v.Fields, item.Field)
		}
		if item.Metadata != nil {
			v.Metadata = append(v.Metadata, item.Metadata.Entries...)
		}
	}
}

// PostProcess populates convenience fields from domain event items.
func (d *DomainEvent) PostProcess() {
	for _, item := range d.Items {
		if item.Field != nil {
			d.Fields = append(d.Fields, item.Field)
		}
		if item.Metadata != nil {
			d.Metadata = append(d.Metadata, item.Metadata.Entries...)
		}
	}
}

func (p *Policy) PostProcess() {
	// No post-processing needed for now
}

func (f *Flow) PostProcess() {
	for _, item := range f.Items {
		if item.Step != nil {
			f.Steps = append(f.Steps, item.Step)
		}
	}
}
