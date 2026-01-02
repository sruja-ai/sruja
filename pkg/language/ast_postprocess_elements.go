// pkg/language/ast_postprocess_elements.go
// Post-processing methods for architecture elements (System, Container, Component, etc.)
package language

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
		if item.Metadata != nil {
			s.Metadata = append(s.Metadata, item.Metadata.Entries...)
		}
		if item.ConstraintsBlock != nil {
			s.Constraints = append(s.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			s.Conventions = append(s.Conventions, item.ConventionsBlock.Entries...)
		}

		if item.Style != nil && item.Style.Body != nil {
			if s.Style == nil {
				s.Style = make(map[string]string)
			}
			for _, style := range item.Style.Body.Entries {
				if style.Value != nil {
					s.Style[style.Key] = *style.Value
				}
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
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.ConstraintsBlock != nil {
			c.Constraints = append(c.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			c.Conventions = append(c.Conventions, item.ConventionsBlock.Entries...)
		}

		if item.Style != nil && item.Style.Body != nil {
			if c.Style == nil {
				c.Style = make(map[string]string)
			}
			for _, style := range item.Style.Body.Entries {
				if style.Value != nil {
					c.Style[style.Key] = *style.Value
				}
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
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			c.Relations = append(c.Relations, item.Relation)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}

		if item.Style != nil && item.Style.Body != nil {
			if c.Style == nil {
				c.Style = make(map[string]string)
			}
			for _, style := range item.Style.Body.Entries {
				if style.Value != nil {
					c.Style[style.Key] = *style.Value
				}
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

		if it.Style != nil && it.Style.Body != nil {
			if d.Style == nil {
				d.Style = make(map[string]string)
			}
			for _, style := range it.Style.Body.Entries {
				if style.Value != nil {
					d.Style[style.Key] = *style.Value
				}
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

		if it.Style != nil && it.Style.Body != nil {
			if q.Style == nil {
				q.Style = make(map[string]string)
			}
			for _, style := range it.Style.Body.Entries {
				if style.Value != nil {
					q.Style[style.Key] = *style.Value
				}
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

		if it.Style != nil && it.Style.Body != nil {
			if p.Style == nil {
				p.Style = make(map[string]string)
			}
			for _, style := range it.Style.Body.Entries {
				if style.Value != nil {
					p.Style[style.Key] = *style.Value
				}
			}
		}
	}
}
