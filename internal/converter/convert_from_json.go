// internal/converter/convert_from_json.go
// JSON to AST conversion functions
package converter

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:gocritic // JSON is large struct, passed by value for simplicity
func ConvertFromJSON(json ArchitectureJSON) *language.Architecture {
	arch := &language.Architecture{
		Name:  json.Metadata.Name,
		Items: []language.ArchitectureItem{},
	}

	for _, sys := range json.Architecture.Systems {
		s := convertSystemFromJSON(&sys)
		arch.Items = append(arch.Items, language.ArchitectureItem{System: s})
		arch.Systems = append(arch.Systems, s)
	}
	for _, p := range json.Architecture.Persons {
		per := convertPersonFromJSON(&p)
		arch.Items = append(arch.Items, language.ArchitectureItem{Person: per})
		arch.Persons = append(arch.Persons, per)
	}
	for _, r := range json.Architecture.Relations {
		rel := convertRelationFromJSON(&r)
		arch.Items = append(arch.Items, language.ArchitectureItem{Relation: rel})
		arch.Relations = append(arch.Relations, rel)
	}
	for _, adr := range json.Architecture.ADRs {
		a := convertADRFromJSON(&adr)
		arch.Items = append(arch.Items, language.ArchitectureItem{ADR: a})
		arch.ADRs = append(arch.ADRs, a)
	}

	for _, req := range json.Architecture.Requirements {
		r := convertRequirementFromJSON(&req)
		arch.Items = append(arch.Items, language.ArchitectureItem{Requirement: r})
		arch.Requirements = append(arch.Requirements, r)
	}

	return arch
}

func convertSystemFromJSON(src *System) *language.System {
	sys := &language.System{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		sys.Description = &src.Description
	}

	for _, c := range src.Containers {
		sys.Items = append(sys.Items, language.SystemItem{Container: convertContainerFromJSON(&c)})
	}
	for _, d := range src.DataStores {
		sys.Items = append(sys.Items, language.SystemItem{DataStore: convertDataStoreFromJSON(&d)})
	}
	for _, q := range src.Queues {
		sys.Items = append(sys.Items, language.SystemItem{Queue: convertQueueFromJSON(&q)})
	}
	if len(src.Metadata) > 0 {
		sys.Items = append(sys.Items, language.SystemItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return sys
}

func convertContainerFromJSON(src *Container) *language.Container {
	c := &language.Container{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		c.Description = &src.Description
	}
	if src.Technology != "" {
		c.Items = append(c.Items, language.ContainerItem{Technology: &src.Technology})
	}
	for _, comp := range src.Components {
		c.Items = append(c.Items, language.ContainerItem{Component: convertComponentFromJSON(&comp)})
	}
	if len(src.Metadata) > 0 {
		c.Items = append(c.Items, language.ContainerItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return c
}

func convertComponentFromJSON(src *Component) *language.Component {
	c := &language.Component{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		c.Description = &src.Description
	}
	if src.Technology != "" {
		c.Items = append(c.Items, language.ComponentItem{Technology: &src.Technology})
	}
	if len(src.Metadata) > 0 {
		c.Items = append(c.Items, language.ComponentItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return c
}

func convertDataStoreFromJSON(src *DataStore) *language.DataStore {
	d := &language.DataStore{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		d.Description = &src.Description
	}
	if src.Technology != "" {
		d.Items = append(d.Items, language.DataStoreItem{Technology: &src.Technology})
	}
	if len(src.Metadata) > 0 {
		d.Items = append(d.Items, language.DataStoreItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return d
}

func convertQueueFromJSON(src *Queue) *language.Queue {
	q := &language.Queue{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		q.Description = &src.Description
	}
	if src.Technology != "" {
		q.Items = append(q.Items, language.QueueItem{Technology: &src.Technology})
	}
	if len(src.Metadata) > 0 {
		q.Items = append(q.Items, language.QueueItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return q
}

func convertPersonFromJSON(src *Person) *language.Person {
	p := &language.Person{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != "" {
		p.Items = append(p.Items, language.PersonItem{Description: &src.Description})
	}
	if len(src.Metadata) > 0 {
		p.Items = append(p.Items, language.PersonItem{Metadata: convertMetadataFromJSON(src.Metadata)})
	}
	return p
}

func convertRelationFromJSON(src *Relation) *language.Relation {
	r := &language.Relation{
		From: language.QualifiedIdent{Parts: strings.Split(src.From, ".")},
		To:   language.QualifiedIdent{Parts: strings.Split(src.To, ".")},
		Tags: src.Tags,
	}
	if src.Verb != "" {
		r.Verb = &src.Verb
	}
	if src.Label != "" {
		r.Label = &src.Label
	}
	return r
}

func convertADRFromJSON(src *ADR) *language.ADR {
	adr := &language.ADR{
		ID:    src.ID,
		Title: &src.Title,
		Body:  &language.ADRBody{},
	}
	if src.Status != "" {
		adr.Body.Status = &src.Status
	}
	if src.Context != "" {
		adr.Body.Context = &src.Context
	}
	if src.Decision != "" {
		adr.Body.Decision = &src.Decision
	}
	if src.Consequences != "" {
		adr.Body.Consequences = &src.Consequences
	}
	return adr
}

func convertRequirementFromJSON(src *Requirement) *language.Requirement {
	r := &language.Requirement{ID: src.ID}
	if src.Type != "" {
		r.Type = &src.Type
	}
	if src.Title != "" {
		r.Description = &src.Title
	}
	r.Body = &language.RequirementBody{}
	if src.Type != "" {
		r.Body.Type = &src.Type
	}
	if src.Description != "" {
		r.Body.Description = &src.Description
		r.Description = &src.Description
	}
	return r
}
