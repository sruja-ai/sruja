// internal/converter/convert_to_json.go
// AST to JSON conversion functions
package converter

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

func convertSystem(src *language.System) System {
	sys := System{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		sys.Description = *src.Description
	}

	if src.Items != nil {
		for _, item := range src.Items {
			switch {
			case item.Container != nil:
				sys.Containers = append(sys.Containers, convertContainer(item.Container))
			case item.DataStore != nil:
				sys.DataStores = append(sys.DataStores, convertDataStore(item.DataStore))
			case item.Queue != nil:
				sys.Queues = append(sys.Queues, convertQueue(item.Queue))
			case item.Metadata != nil:
				sys.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}

	return sys
}

func convertContainer(src *language.Container) Container {
	c := Container{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		c.Description = *src.Description
	}
	if src.Items != nil {
		for _, item := range src.Items {
			if item.Technology != nil {
				c.Technology = *item.Technology
			}
			if item.Component != nil {
				c.Components = append(c.Components, convertComponent(item.Component))
			}
			if item.Metadata != nil {
				c.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}
	return c
}

func convertComponent(src *language.Component) Component {
	c := Component{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		c.Description = *src.Description
	}
	if src.Items != nil {
		for _, item := range src.Items {
			if item.Technology != nil {
				c.Technology = *item.Technology
			}
			if item.Metadata != nil {
				c.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}
	return c
}

func convertDataStore(src *language.DataStore) DataStore {
	d := DataStore{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		d.Description = *src.Description
	}
	if src.Items != nil {
		for _, item := range src.Items {
			if item.Technology != nil {
				d.Technology = *item.Technology
			}
			if item.Metadata != nil {
				d.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}
	return d
}

func convertQueue(src *language.Queue) Queue {
	q := Queue{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		q.Description = *src.Description
	}
	if src.Items != nil {
		for _, item := range src.Items {
			if item.Technology != nil {
				q.Technology = *item.Technology
			}
			if item.Metadata != nil {
				q.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}
	return q
}

func convertPerson(src *language.Person) Person {
	p := Person{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Items != nil {
		for _, item := range src.Items {
			if item.Description != nil {
				p.Description = *item.Description
			}
			if item.Metadata != nil {
				p.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}
	return p
}

func convertRelation(src *language.Relation) Relation {
	r := Relation{
		From: src.From.String(),
		To:   src.To.String(),
		Tags: src.Tags,
	}
	if src.Verb != nil {
		r.Verb = *src.Verb
	}
	if src.Label != nil {
		r.Label = *src.Label
	}
	return r
}

func convertADR(src *language.ADR) ADR {
	adr := ADR{
		ID: src.ID,
	}
	if src.Title != nil {
		adr.Title = *src.Title
	}
	if src.Body != nil {
		if src.Body.Status != nil {
			adr.Status = *src.Body.Status
		}
		if src.Body.Context != nil {
			adr.Context = *src.Body.Context
		}
		if src.Body.Decision != nil {
			adr.Decision = *src.Body.Decision
		}
		if src.Body.Consequences != nil {
			adr.Consequences = *src.Body.Consequences
		}
	}
	return adr
}

func convertRequirement(src *language.Requirement) Requirement {
	r := Requirement{ID: src.ID}
	if src.Type != nil {
		r.Type = *src.Type
	}
	if src.Description != nil {
		r.Title = *src.Description
		r.Description = *src.Description
	}
	if src.Body != nil {
		if src.Body.Type != nil {
			r.Type = *src.Body.Type
		}
		if src.Body.Description != nil {
			r.Title = *src.Body.Description
			r.Description = *src.Body.Description
		}
		if len(src.Body.Tags) > 0 {
			r.Tags = src.Body.Tags
		}
	}
	return r
}

func convertFlow(src *language.Flow) Flow {
	title := ""
	if src.Title != nil {
		title = *src.Title
	}
	f := Flow{
		ID:    src.ID,
		Title: title,
		Label: title,
	}
	if src.Description != nil {
		f.Description = *src.Description
	}
	if len(src.Steps) > 0 {
		steps := make([]FlowStep, 0, len(src.Steps))
		for _, step := range src.Steps {
			desc := ""
			if step.Description != nil {
				desc = *step.Description
			}
			steps = append(steps, FlowStep{
				From:        step.From.String(),
				To:          step.To.String(),
				Description: desc,
			})
		}
		f.Steps = steps
	}
	return f
}

func convertPolicy(src *language.Policy) Policy {
	p := Policy{
		ID:          src.ID,
		Description: src.Description,
	}
	if src.InlineCategory != nil {
		p.Category = *src.InlineCategory
	} else if src.Body != nil && src.Body.Category != nil {
		p.Category = *src.Body.Category
	}
	if src.InlineEnforcement != nil {
		p.Enforcement = *src.InlineEnforcement
	} else if src.Body != nil && src.Body.Enforcement != nil {
		p.Enforcement = *src.Body.Enforcement
	}
	if src.Body != nil && len(src.Body.Tags) > 0 {
		p.Tags = src.Body.Tags
	}
	return p
}
