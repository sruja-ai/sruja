//nolint:gocritic // rangeValCopy, ifElseChain acceptable here
package converter

import (
	"strings"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
)

// JSON Structures matching viewer expectations

type ArchitectureJSON struct {
	Metadata     MetadataJSON `json:"metadata"`
	Architecture Architecture `json:"architecture"`
}

type MetadataJSON struct {
	Name      string `json:"name"`
	Version   string `json:"version"`
	Generated string `json:"generated"`
}

type MetadataEntryJSON struct {
	Key   string   `json:"key"`
	Value *string  `json:"value,omitempty"`
	Array []string `json:"array,omitempty"`
}

type Architecture struct {
	Systems    []System    `json:"systems,omitempty"`
	Persons    []Person    `json:"persons,omitempty"`
	Containers []Container `json:"containers,omitempty"` // Top-level containers
	Relations  []Relation  `json:"relations,omitempty"`
	ADRs       []ADR       `json:"adrs,omitempty"`
}

type System struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Containers  []Container         `json:"containers,omitempty"`
	DataStores  []DataStore         `json:"datastores,omitempty"`
	Queues      []Queue             `json:"queues,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Container struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Components  []Component         `json:"components,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Component struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type DataStore struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Queue struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Person struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Relation struct {
	From  string   `json:"from"`
	To    string   `json:"to"`
	Tags  []string `json:"tags,omitempty"`
	Verb  string   `json:"verb,omitempty"`
	Label string   `json:"label,omitempty"`
}

type ADR struct {
	ID           string `json:"id"`
	Title        string `json:"title"`
	Status       string `json:"status,omitempty"`
	Context      string `json:"context,omitempty"`
	Decision     string `json:"decision,omitempty"`
	Consequences string `json:"consequences,omitempty"`
}

func ConvertToJSON(arch *language.Architecture) ArchitectureJSON {
	destArch := Architecture{}

	// Handle items from Architecture
	processArchitecture(arch, &destArch)

	return ArchitectureJSON{
		Metadata: MetadataJSON{
			Name:      "Sruja Architecture",
			Version:   "1.0.0",
			Generated: time.Now().Format(time.RFC3339),
		},
		Architecture: destArch,
	}
}

func processArchitecture(src *language.Architecture, dest *Architecture) {
	for _, item := range src.Items {
		if item.System != nil {
			dest.Systems = append(dest.Systems, convertSystem(item.System))
		} else if item.Person != nil {
			dest.Persons = append(dest.Persons, convertPerson(item.Person))
		} else if item.Relation != nil {
			dest.Relations = append(dest.Relations, convertRelation(item.Relation))
		} else if item.ADR != nil {
			dest.ADRs = append(dest.ADRs, convertADR(item.ADR))
		}
		// Add other items as needed
	}
}

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
			if item.Container != nil {
				sys.Containers = append(sys.Containers, convertContainer(item.Container))
			} else if item.DataStore != nil {
				sys.DataStores = append(sys.DataStores, convertDataStore(item.DataStore))
			} else if item.Queue != nil {
				sys.Queues = append(sys.Queues, convertQueue(item.Queue))
			} else if item.Metadata != nil {
				sys.Metadata = convertMetadataToJSON(item.Metadata)
			}
		}
	}

	return sys
}

func convertMetadataToJSON(src *language.MetadataBlock) []MetadataEntryJSON {
	var entries []MetadataEntryJSON
	for _, entry := range src.Entries {
		if entry.Value != nil {
			val := *entry.Value
			entries = append(entries, MetadataEntryJSON{
				Key:   entry.Key,
				Value: &val,
			})
		} else if len(entry.Array) > 0 {
			entries = append(entries, MetadataEntryJSON{
				Key:   entry.Key,
				Array: entry.Array,
			})
		}
	}
	return entries
}

func convertMetadataFromJSON(src []MetadataEntryJSON) *language.MetadataBlock {
	if len(src) == 0 {
		return nil
	}
	block := &language.MetadataBlock{}
	for _, entry := range src {
		if entry.Value != nil {
			block.Entries = append(block.Entries, &language.MetaEntry{
				Key:   entry.Key,
				Value: entry.Value,
			})
		} else if len(entry.Array) > 0 {
			block.Entries = append(block.Entries, &language.MetaEntry{
				Key:   entry.Key,
				Array: entry.Array,
			})
		}
	}
	return block
}

func convertContainer(src *language.Container) Container {
	c := Container{
		ID:    src.ID,
		Label: src.Label,
	}
	if src.Description != nil {
		c.Description = *src.Description
	}
	// Extract technology from items if present
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
	// Description is in Items or post-processed
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
