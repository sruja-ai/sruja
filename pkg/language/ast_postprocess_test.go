// pkg/language/ast_postprocess_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func strPtr(s string) *string {
	return &s
}

func TestArchitecture_PostProcess_Imports(t *testing.T) {
	ds := &language.DataStore{
		ID:    "DB",
		Label: "Database",
		Items: []language.DataStoreItem{
			{
				Description: strPtr("Main database"),
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "engine", Value: strPtr("postgres")},
					},
				},
			},
		},
	}

	ds.PostProcess()

	if ds.Description == nil || *ds.Description != "Main database" {
		t.Error("Description should be populated from items")
	}
	if len(ds.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(ds.Metadata))
	}
	if ds.Metadata[0].Key != "engine" {
		t.Errorf("Expected metadata key 'engine', got %q", ds.Metadata[0].Key)
	}
}

func TestDataStore_PostProcess(t *testing.T) {
	ds := &language.DataStore{
		ID:    "DB",
		Label: "Database",
		Items: []language.DataStoreItem{
			{
				Description: strPtr("Main database"),
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "engine", Value: strPtr("postgres")},
					},
				},
			},
		},
	}

	ds.PostProcess()

	if ds.Description == nil || *ds.Description != "Main database" {
		t.Error("Description should be populated from items")
	}
	if len(ds.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(ds.Metadata))
	}
	if ds.Metadata[0].Key != "engine" {
		t.Errorf("Expected metadata key 'engine', got %q", ds.Metadata[0].Key)
	}
}

func TestQueue_PostProcess(t *testing.T) {
	q := &language.Queue{
		ID:    "Q",
		Label: "Queue",
		Items: []language.QueueItem{
			{
				Description: strPtr("Event queue"),
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "topic", Value: strPtr("events")},
					},
				},
			},
		},
	}

	q.PostProcess()

	if q.Description == nil || *q.Description != "Event queue" {
		t.Error("Description should be populated from items")
	}
	if len(q.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(q.Metadata))
	}
}

func TestPerson_PostProcess(t *testing.T) {
	p := &language.Person{
		ID:    "User",
		Label: "End User",
		Items: []language.PersonItem{
			{
				Description: strPtr("Customer"),
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "persona", Value: strPtr("customer")},
					},
				},
			},
		},
	}

	p.PostProcess()

	if p.Description == nil || *p.Description != "Customer" {
		t.Error("Description should be populated from items")
	}
	if len(p.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(p.Metadata))
	}
}

func TestPerson_PostProcess_PropertiesAndStyle(t *testing.T) {
	p := &language.Person{
		ID:    "User",
		Label: "End User",
		Items: []language.PersonItem{
			{

				Style: &language.StyleDecl{
					Body: &language.StyleBlock{
						Entries: []*language.StyleEntry{
							{Key: "color", Value: strPtr("blue")},
						},
					},
				},
			},
		},
	}

	p.PostProcess()

	if p.Style == nil {
		t.Fatal("Style should be initialized")
	}
	if p.Style["color"] != "blue" {
		t.Errorf("Expected color='blue', got %q", p.Style["color"])
	}
}

func TestQueue_PostProcess_PropertiesAndStyle(t *testing.T) {
	q := &language.Queue{
		ID:    "Q",
		Label: "Queue",
		Items: []language.QueueItem{
			{
				Technology: strPtr("RabbitMQ"),
			},
			{},
			{
				Style: &language.StyleDecl{
					Body: &language.StyleBlock{
						Entries: []*language.StyleEntry{
							{Key: "shape", Value: strPtr("cylinder")},
						},
					},
				},
			},
		},
	}

	q.PostProcess()

	if q.Technology == nil || *q.Technology != "RabbitMQ" {
		t.Error("Technology should be populated")
	}
	if q.Style == nil {
		t.Fatal("Style should be initialized")
	}
	if q.Style["shape"] != "cylinder" {
		t.Errorf("Expected shape='cylinder', got %q", q.Style["shape"])
	}
}

func TestComponent_PostProcess(t *testing.T) {
	comp := &language.Component{
		ID:    "Comp",
		Label: "Component",
		Items: []language.ComponentItem{
			{
				Technology:  strPtr("Go"),
				Description: strPtr("Main component"),
			},
			{
				Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"Comp"}},
					To:   language.QualifiedIdent{Parts: []string{"Other"}},
				},
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "critical", Value: strPtr("true")},
					},
				},
			},
		},
	}

	comp.PostProcess()

	if comp.Technology == nil || *comp.Technology != "Go" {
		t.Error("Technology should be populated")
	}
	if comp.Description == nil || *comp.Description != "Main component" {
		t.Error("Description should be populated")
	}
	// root-only policy: component does not collect requirements/ADRs
	if len(comp.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(comp.Relations))
	}
	if len(comp.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(comp.Metadata))
	}
}

func TestComponent_PostProcess_Style(t *testing.T) {
	comp := &language.Component{
		ID:    "Comp",
		Label: "Component",
		Items: []language.ComponentItem{
			{
				Style: &language.StyleDecl{
					Body: &language.StyleBlock{
						Entries: []*language.StyleEntry{
							{Key: "color", Value: strPtr("red")},
						},
					},
				},
			},
		},
	}

	comp.PostProcess()

	if comp.Style == nil {
		t.Fatal("Style should be initialized")
	}
	if comp.Style["color"] != "red" {
		t.Errorf("Expected color='red', got %q", comp.Style["color"])
	}
}

func TestContainer_PostProcess(t *testing.T) {
	cont := &language.Container{
		ID:    "Cont",
		Label: "Container",
		Items: []language.ContainerItem{
			{
				Description: strPtr("Container description"),
			},
			{
				Component: &language.Component{
					ID:    "Comp",
					Label: "Component",
				},
			},
			{
				DataStore: &language.DataStore{
					ID:    "DB",
					Label: "Database",
				},
			},
			{
				Queue: &language.Queue{
					ID:    "Q",
					Label: "Queue",
				},
			},
			{
				Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"Cont"}},
					To:   language.QualifiedIdent{Parts: []string{"Other"}},
				},
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "tier", Value: strPtr("gold")},
					},
				},
			},
		},
	}

	cont.PostProcess()

	if cont.Description == nil || *cont.Description != "Container description" {
		t.Error("Description should be populated")
	}
	if len(cont.Components) != 1 {
		t.Errorf("Expected 1 component, got %d", len(cont.Components))
	}
	if len(cont.DataStores) != 1 {
		t.Errorf("Expected 1 datastore, got %d", len(cont.DataStores))
	}
	if len(cont.Queues) != 1 {
		t.Errorf("Expected 1 queue, got %d", len(cont.Queues))
	}
	// root-only policy: container does not collect requirements/ADRs
	if len(cont.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(cont.Relations))
	}
	if len(cont.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(cont.Metadata))
	}
}

func TestSystem_PostProcess(t *testing.T) {
	sys := &language.System{
		ID:    "Sys",
		Label: "System",
		Items: []language.SystemItem{
			{
				Description: strPtr("System description"),
			},
			{
				Container: &language.Container{
					ID:    "Cont",
					Label: "Container",
				},
			},
			{
				DataStore: &language.DataStore{
					ID:    "DB",
					Label: "Database",
				},
			},
			{
				Queue: &language.Queue{
					ID:    "Q",
					Label: "Queue",
				},
			},
			{
				Person: &language.Person{
					ID:    "User",
					Label: "User",
				},
			},
			{
				Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"Sys"}},
					To:   language.QualifiedIdent{Parts: []string{"Other"}},
				},
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "owner", Value: strPtr("team")},
					},
				},
			},
		},
	}

	sys.PostProcess()

	if sys.Description == nil || *sys.Description != "System description" {
		t.Error("Description should be populated")
	}
	if len(sys.Containers) != 1 {
		t.Errorf("Expected 1 container, got %d", len(sys.Containers))
	}
	if len(sys.DataStores) != 1 {
		t.Errorf("Expected 1 datastore, got %d", len(sys.DataStores))
	}
	if len(sys.Queues) != 1 {
		t.Errorf("Expected 1 queue, got %d", len(sys.Queues))
	}
	if len(sys.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(sys.Persons))
	}
	// root-only policy: system does not collect requirements/ADRs
	if len(sys.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(sys.Relations))
	}
	if len(sys.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(sys.Metadata))
	}
}
