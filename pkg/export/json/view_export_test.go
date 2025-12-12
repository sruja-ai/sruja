package json

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestGenerateViews_Basic(t *testing.T) {
	// Create a simple architecture
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "System1",
				Label: "System One",
				Containers: []*language.Container{
					{
						ID:    "Container1",
						Label: "Container One",
						Components: []*language.Component{
							{ID: "Comp1", Label: "Component One"},
						},
					},
				},
			},
		},
		Persons: []*language.Person{
			{ID: "User1", Label: "End User"},
		},
	}

	views := GenerateViews(arch)

	// Check L1 view
	if views.L1.Nodes == nil {
		t.Fatal("L1 nodes should not be nil")
	}
	if len(views.L1.Nodes) != 2 {
		t.Errorf("Expected 2 L1 nodes (1 system + 1 person), got %d", len(views.L1.Nodes))
	}

	// Check L2 views
	if len(views.L2) != 1 {
		t.Errorf("Expected 1 L2 view (for System1), got %d", len(views.L2))
	}
	l2View, ok := views.L2["System1"]
	if !ok {
		t.Error("L2 view for System1 should exist")
	}
	if len(l2View.Nodes) != 1 {
		t.Errorf("Expected 1 L2 node (Container1), got %d", len(l2View.Nodes))
	}

	// Check L3 views
	if len(views.L3) != 1 {
		t.Errorf("Expected 1 L3 view (for System1.Container1), got %d", len(views.L3))
	}
	l3View, ok := views.L3["System1.Container1"]
	if !ok {
		t.Error("L3 view for System1.Container1 should exist")
	}
	if len(l3View.Nodes) != 1 {
		t.Errorf("Expected 1 L3 node (Comp1), got %d", len(l3View.Nodes))
	}
}

func TestGenerateViews_WithRelations(t *testing.T) {
	desc := "Test Description"
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{ID: "System1", Label: "System One"},
			{ID: "System2", Label: "System Two"},
		},
		Persons: []*language.Person{
			{ID: "User1", Label: "User", Description: &desc},
		},
		Relations: []*language.Relation{
			{
				From:  language.QualifiedIdent{Parts: []string{"User1"}},
				To:    language.QualifiedIdent{Parts: []string{"System1"}},
				Label: strPtr("uses"),
			},
			{
				From:  language.QualifiedIdent{Parts: []string{"System1"}},
				To:    language.QualifiedIdent{Parts: []string{"System2"}},
				Label: strPtr("calls"),
			},
		},
	}

	views := GenerateViews(arch)

	// Check L1 edges
	if len(views.L1.Edges) != 2 {
		t.Errorf("Expected 2 L1 edges, got %d", len(views.L1.Edges))
	}

	// Verify edge labels
	foundUses := false
	foundCalls := false
	for _, e := range views.L1.Edges {
		if e.Label == "uses" {
			foundUses = true
		}
		if e.Label == "calls" {
			foundCalls = true
		}
	}
	if !foundUses {
		t.Error("Expected edge with label 'uses'")
	}
	if !foundCalls {
		t.Error("Expected edge with label 'calls'")
	}
}

func TestGenerateViews_NodeTypes(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "System1",
				Label: "System One",
				Containers: []*language.Container{
					{ID: "Container1", Label: "Container One"},
				},
				DataStores: []*language.DataStore{
					{ID: "DB1", Label: "Database"},
				},
				Queues: []*language.Queue{
					{ID: "Queue1", Label: "Message Queue"},
				},
			},
		},
		Persons: []*language.Person{
			{ID: "User1", Label: "User"},
		},
	}

	views := GenerateViews(arch)

	// Check L1 node types
	foundPerson := false
	foundSystem := false
	for _, n := range views.L1.Nodes {
		if n.Type == "person" {
			foundPerson = true
		}
		if n.Type == "system" {
			foundSystem = true
		}
	}
	if !foundPerson {
		t.Error("Expected person node in L1")
	}
	if !foundSystem {
		t.Error("Expected system node in L1")
	}

	// Check L2 node types
	l2View := views.L2["System1"]
	foundContainer := false
	foundDatastore := false
	foundQueue := false
	for _, n := range l2View.Nodes {
		switch n.Type {
		case "container":
			foundContainer = true
		case "datastore":
			foundDatastore = true
		case "queue":
			foundQueue = true
		}
	}
	if !foundContainer {
		t.Error("Expected container node in L2")
	}
	if !foundDatastore {
		t.Error("Expected datastore node in L2")
	}
	if !foundQueue {
		t.Error("Expected queue node in L2")
	}
}

func TestExporter_Extended(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{ID: "System1", Label: "System One"},
		},
	}

	// Without extended
	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}
	if containsViews(output) {
		t.Error("Standard export should not contain views")
	}

	// With extended
	exporter.Extended = true
	output, err = exporter.Export(arch)
	if err != nil {
		t.Fatalf("Extended export failed: %v", err)
	}
	if !containsViews(output) {
		t.Error("Extended export should contain views")
	}
}

func containsViews(json string) bool {
    return json != "" && strings.Contains(json, `"views":`)
}

func strPtr(s string) *string {
	return &s
}
