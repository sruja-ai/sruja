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
				Description: stringPtr("Main database"),
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
				Description: stringPtr("Main database"),
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
				Description: stringPtr("Event queue"),
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
				Description: stringPtr("Customer"),
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
				Properties: &language.PropertiesBlock{
					Entries: []*language.PropertyEntry{
						{Key: "role", Value: "admin"},
					},
				},
			},
			{
				Style: &language.StyleBlock{
					Entries: []*language.StyleEntry{
						{Key: "color", Value: "blue"},
					},
				},
			},
		},
	}

	p.PostProcess()

	if p.Properties == nil {
		t.Fatal("Properties should be initialized")
	}
	if p.Properties["role"] != "admin" {
		t.Errorf("Expected role='admin', got %q", p.Properties["role"])
	}
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
				Technology: stringPtr("RabbitMQ"),
			},
			{
				Properties: &language.PropertiesBlock{
					Entries: []*language.PropertyEntry{
						{Key: "durable", Value: "true"},
					},
				},
			},
			{
				Style: &language.StyleBlock{
					Entries: []*language.StyleEntry{
						{Key: "shape", Value: "cylinder"},
					},
				},
			},
		},
	}

	q.PostProcess()

	if q.Technology == nil || *q.Technology != "RabbitMQ" {
		t.Error("Technology should be populated")
	}
	if q.Properties == nil {
		t.Fatal("Properties should be initialized")
	}
	if q.Properties["durable"] != "true" {
		t.Errorf("Expected durable='true', got %q", q.Properties["durable"])
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
				Technology:  stringPtr("Go"),
				Description: stringPtr("Main component"),
			},
			{
				Requirement: &language.Requirement{
					ID:          "R1",
					Type:        strPtr("performance"),
					Description: strPtr("Fast"),
				},
			},
			{
				ADR: &language.ADR{
					ID:    "ADR001",
					Title: stringPtr("Use JWT"),
				},
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
	if len(comp.Requirements) != 1 {
		t.Errorf("Expected 1 requirement, got %d", len(comp.Requirements))
	}
	if len(comp.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(comp.ADRs))
	}
	if len(comp.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(comp.Relations))
	}
	if len(comp.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(comp.Metadata))
	}
}

func TestComponent_PostProcess_Properties(t *testing.T) {
	comp := &language.Component{
		ID:    "Comp",
		Label: "Component",
		Items: []language.ComponentItem{
			{
				Properties: &language.PropertiesBlock{
					Entries: []*language.PropertyEntry{
						{Key: "prop1", Value: "value1"},
						{Key: "prop2", Value: "value2"},
					},
				},
			},
		},
	}

	comp.PostProcess()

	if comp.Properties == nil {
		t.Fatal("Properties should be initialized")
	}
	if comp.Properties["prop1"] != "value1" {
		t.Errorf("Expected prop1='value1', got %q", comp.Properties["prop1"])
	}
	if comp.Properties["prop2"] != "value2" {
		t.Errorf("Expected prop2='value2', got %q", comp.Properties["prop2"])
	}
}

func TestComponent_PostProcess_Style(t *testing.T) {
	comp := &language.Component{
		ID:    "Comp",
		Label: "Component",
		Items: []language.ComponentItem{
			{
				Style: &language.StyleBlock{
					Entries: []*language.StyleEntry{
						{Key: "color", Value: "red"},
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
				Description: stringPtr("Container description"),
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
				Requirement: &language.Requirement{
					ID:          "R1",
					Type:        strPtr("performance"),
					Description: strPtr("Fast"),
				},
			},
			{
				ADR: &language.ADR{
					ID:    "ADR001",
					Title: stringPtr("Use JWT"),
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
	if len(cont.Requirements) != 1 {
		t.Errorf("Expected 1 requirement, got %d", len(cont.Requirements))
	}
	if len(cont.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(cont.ADRs))
	}
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
				Description: stringPtr("System description"),
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
				Requirement: &language.Requirement{
					ID:          "R1",
					Type:        strPtr("security"),
					Description: strPtr("Must be secure"),
				},
			},
			{
				ADR: &language.ADR{
					ID:    "ADR001",
					Title: stringPtr("Use JWT"),
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
	if len(sys.Requirements) != 1 {
		t.Errorf("Expected 1 requirement, got %d", len(sys.Requirements))
	}
	if len(sys.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(sys.ADRs))
	}
	if len(sys.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(sys.Relations))
	}
	if len(sys.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(sys.Metadata))
	}
}

func TestArchitecture_PostProcess(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{
				Import: &language.ImportSpec{
					Path: "other.sruja",
				},
			},
			{
				System: &language.System{
					ID:    "Sys",
					Label: "System",
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
					From: language.QualifiedIdent{Parts: []string{"User"}},
					To:   language.QualifiedIdent{Parts: []string{"Sys"}},
				},
			},
			{
				Requirement: &language.Requirement{
					ID:          "R1",
					Type:        strPtr("performance"),
					Description: strPtr("Fast"),
				},
			},
			{
				ADR: &language.ADR{
					ID:    "ADR001",
					Title: stringPtr("Use JWT"),
				},
			},
			{
				SharedArtifact: &language.SharedArtifact{
					ID:    "SA1",
					Label: "Shared Lib",
				},
			},
			{
				Library: &language.Library{
					ID:    "Lib1",
					Label: "Library",
				},
			},
			{
				Metadata: &language.MetadataBlock{
					Entries: []*language.MetaEntry{
						{Key: "level", Value: strPtr("arch")},
					},
				},
			},
		},
	}

	arch.PostProcess()

	if len(arch.Imports) != 1 {
		t.Errorf("Expected 1 import, got %d", len(arch.Imports))
	}
	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}
	if len(arch.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(arch.Persons))
	}
	if len(arch.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(arch.Relations))
	}
	if len(arch.Requirements) != 1 {
		t.Errorf("Expected 1 requirement, got %d", len(arch.Requirements))
	}
	if len(arch.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(arch.ADRs))
	}
	if len(arch.SharedArtifacts) != 1 {
		t.Errorf("Expected 1 shared artifact, got %d", len(arch.SharedArtifacts))
	}
	if len(arch.Libraries) != 1 {
		t.Errorf("Expected 1 library, got %d", len(arch.Libraries))
	}
	if len(arch.Metadata) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(arch.Metadata))
	}
}

func TestDeploymentNode_PostProcess(t *testing.T) {
	dn := &language.DeploymentNode{
		ID:    "Prod",
		Label: "Production",
		Items: []language.DeploymentNodeItem{
			{
				Node: &language.DeploymentNode{
					ID:    "Server",
					Label: "Server",
				},
			},
			{
				ContainerInstance: &language.ContainerInstance{
					ContainerID: "Sys.Cont",
				},
			},
			{
				Infrastructure: &language.InfrastructureNode{
					ID:    "LB",
					Label: "Load Balancer",
				},
			},
		},
	}

	dn.PostProcess()

	if len(dn.Children) != 1 {
		t.Errorf("Expected 1 child node, got %d", len(dn.Children))
	}
	if len(dn.ContainerInstances) != 1 {
		t.Errorf("Expected 1 container instance, got %d", len(dn.ContainerInstances))
	}
	if len(dn.Infrastructure) != 1 {
		t.Errorf("Expected 1 infrastructure node, got %d", len(dn.Infrastructure))
	}
}

func TestArchitecture_PostProcess_DomainEntitiesAndEvents(t *testing.T) {
	// DDD features (DomainBlock, DomainItem, ContextBlock, ContextItem) removed - deferred to Phase 2
	t.Skip("DDD features removed - deferred to Phase 2")
	// 							Items: []language.ContextItem{
	// 								{
	// 									Entity: &language.Entity{
	// 										ID: "E1",
	// 									},
	// 								},
	// 							},
	// 						},
	// 					},
	// 				},
	// 			},
	// 		},
	// 		{
	// 			Domain: &language.DomainBlock{
	// 				ID: "EventDomain",
	// 				Items: []language.DomainItem{
	// 					{
	// 						Context: &language.ContextBlock{
	// 							ID: "EventContext",
	// 							Items: []language.ContextItem{
	// 								{
	// 									DomainEvent: &language.DomainEvent{
	// 										ID: "Ev1",
	// 									},
	// 								},
	// 							},
	// 						},
	// 					},
	// 				},
	// 			},
	// 		},
	// 	},
	// }
	// arch.PostProcess()
	// if len(arch.Contexts) != 2 {
	// 	t.Errorf("Expected 2 contexts, got %d", len(arch.Contexts))
	// }
	// if len(arch.Contexts[0].Entities) != 1 {
	// 	t.Errorf("Expected 1 entity in first context, got %d", len(arch.Contexts[0].Entities))
	// }
	// if len(arch.Contexts[1].Events) != 1 {
	// 	t.Errorf("Expected 1 event in second context, got %d", len(arch.Contexts[1].Events))
	// }
}

func TestArchitecture_PostProcess_ContractsAndConstraints(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{
				ContractsBlock: &language.ContractsBlock{
					Contracts: []*language.Contract{
						{Kind: "api", ID: "C1"},
					},
				},
			},
			{
				ConstraintsBlock: &language.ConstraintsBlock{
					Entries: []*language.ConstraintEntry{
						{Key: "constraint1", Value: "value1"},
					},
				},
			},
			{
				ConventionsBlock: &language.ConventionsBlock{
					Entries: []*language.ConventionEntry{
						{Key: "convention1", Value: "value1"},
					},
				},
			},
			// ContextBlock removed - DDD feature, deferred to Phase 2
			// {
			// 	Context: &language.ContextBlock{
			// 		ID: "Ctx1",
			// 		Items: []language.ContextItem{
			// 			{
			// 				Entity: &language.Entity{
			// 					Name: "E1",
			// 				},
			// 			},
			// 		},
			// 	},
			// },
		},
	}

	arch.PostProcess()

	if len(arch.Contracts) != 1 {
		t.Errorf("Expected 1 contract, got %d", len(arch.Contracts))
	}
	if len(arch.Constraints) != 1 {
		t.Errorf("Expected 1 constraint, got %d", len(arch.Constraints))
	}
	if len(arch.Conventions) != 1 {
		t.Errorf("Expected 1 convention, got %d", len(arch.Conventions))
	}
	// Contexts removed - DDD feature, deferred to Phase 2
	// if len(arch.Contexts) != 1 {
	// 	t.Errorf("Expected 1 context, got %d", len(arch.Contexts))
	// }
	// if len(arch.Contexts[0].Entities) != 1 {
	// 	t.Errorf("Expected 1 entity, got %d", len(arch.Contexts[0].Entities))
	// }
}
