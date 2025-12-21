package json

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func convertSystem(s *language.System) SystemJSON {
	out := SystemJSON{ID: s.ID, Label: s.Label, Description: s.Description}

	if len(s.Containers) > 0 {
		containers := make([]ContainerJSON, 0, len(s.Containers))
		for _, c := range s.Containers {
			containers = append(containers, convertContainer(c))
		}
		out.Containers = containers
	}
	if len(s.Components) > 0 {
		comps := make([]ComponentJSON, 0, len(s.Components))
		for _, c := range s.Components {
			comps = append(comps, convertComponent(c))
		}
		out.Components = comps
	}
	if len(s.DataStores) > 0 {
		dss := make([]DataStoreJSON, 0, len(s.DataStores))
		for _, d := range s.DataStores {
			dss = append(dss, convertDataStore(d))
		}
		out.DataStores = dss
	}
	if len(s.Queues) > 0 {
		qs := make([]QueueJSON, 0, len(s.Queues))
		for _, q := range s.Queues {
			qs = append(qs, convertQueue(q))
		}
		out.Queues = qs
	}
	if len(s.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(s.Relations))
		for _, r := range s.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}

	if len(s.Properties) > 0 {
		out.Properties = s.Properties
	}
	if len(s.Style) > 0 {
		out.Style = s.Style
	}
	if s.SLO != nil {
		out.SLO = convertSLO(s.SLO)
	}
	if len(s.Contracts) > 0 {
		out.Contracts = convertContracts(s.Contracts)
	}
	if len(s.Constraints) > 0 {
		out.Constraints = convertConstraints(s.Constraints)
	}
	if len(s.Conventions) > 0 {
		out.Conventions = convertConventions(s.Conventions)
	}

	return out
}

//nolint:gocyclo // Conversion logic is complex but straightforward
func convertContainer(c *language.Container) ContainerJSON {
	out := ContainerJSON{
		ID:          c.ID,
		Label:       c.Label,
		Description: c.Description,
		Version:     c.Version,
	}

	for i := range c.Items {
		item := &c.Items[i]
		if item.Technology != nil {
			out.Technology = item.Technology
		}
		if len(item.Tags) > 0 {
			out.Tags = item.Tags
		}
	}

	if len(c.Components) > 0 {
		comps := make([]ComponentJSON, 0, len(c.Components))
		for _, comp := range c.Components {
			comps = append(comps, convertComponent(comp))
		}
		out.Components = comps
	}
	if len(c.DataStores) > 0 {
		dss := make([]DataStoreJSON, 0, len(c.DataStores))
		for _, d := range c.DataStores {
			dss = append(dss, convertDataStore(d))
		}
		out.DataStores = dss
	}
	if len(c.Queues) > 0 {
		qs := make([]QueueJSON, 0, len(c.Queues))
		for _, q := range c.Queues {
			qs = append(qs, convertQueue(q))
		}
		out.Queues = qs
	}
	if len(c.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(c.Relations))
		for _, r := range c.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}
	if len(c.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(c.Metadata))
		for _, m := range c.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}

	if len(c.Properties) > 0 {
		out.Properties = c.Properties
	}
	if len(c.Style) > 0 {
		out.Style = c.Style
	}
	if c.Scale != nil {
		out.Scale = convertScale(c.Scale)
	}
	if c.SLO != nil {
		out.SLO = convertSLO(c.SLO)
	}
	if len(c.Contracts) > 0 {
		out.Contracts = convertContracts(c.Contracts)
	}
	if len(c.Constraints) > 0 {
		out.Constraints = convertConstraints(c.Constraints)
	}
	if len(c.Conventions) > 0 {
		out.Conventions = convertConventions(c.Conventions)
	}

	return out
}

func convertComponent(c *language.Component) ComponentJSON {
	out := ComponentJSON{
		ID:          c.ID,
		Label:       c.Label,
		Description: c.Description,
		Technology:  c.Technology,
	}

	if len(c.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(c.Relations))
		for _, r := range c.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}
	if len(c.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(c.Metadata))
		for _, m := range c.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}

	if len(c.Properties) > 0 {
		out.Properties = c.Properties
	}
	if len(c.Style) > 0 {
		out.Style = c.Style
	}
	if c.Scale != nil {
		out.Scale = convertScale(c.Scale)
	}

	return out
}

func idOrLabel(id, label string) string {
	if strings.TrimSpace(id) != "" {
		return id
	}
	return label
}

func sanitize(s string) string {
	return strings.TrimSpace(s)
}

func strVal(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func convertPerson(p *language.Person) PersonJSON {
	out := PersonJSON{
		ID:          p.ID,
		Label:       p.Label,
		Description: p.Description,
	}
	if len(p.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(p.Metadata))
		for _, m := range p.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}
	if len(p.Properties) > 0 {
		out.Properties = p.Properties
	}
	if len(p.Style) > 0 {
		out.Style = p.Style
	}
	return out
}

func convertDataStore(d *language.DataStore) DataStoreJSON {
	out := DataStoreJSON{
		ID:          d.ID,
		Label:       d.Label,
		Description: d.Description,
		Technology:  d.Technology,
	}
	if len(d.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(d.Metadata))
		for _, m := range d.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}
	if len(d.Properties) > 0 {
		out.Properties = d.Properties
	}
	if len(d.Style) > 0 {
		out.Style = d.Style
	}
	return out
}

func convertQueue(q *language.Queue) QueueJSON {
	out := QueueJSON{
		ID:          q.ID,
		Label:       q.Label,
		Description: q.Description,
		Technology:  q.Technology,
	}
	if len(q.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(q.Metadata))
		for _, m := range q.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}
	if len(q.Properties) > 0 {
		out.Properties = q.Properties
	}
	if len(q.Style) > 0 {
		out.Style = q.Style
	}
	return out
}
