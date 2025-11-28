// pkg/dx/explainer_element_test.go
package dx

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExplainElement_System(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID:          "API",
					Label:       "API Service",
					Description: stringPtr("Main API"),
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("API")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation == nil {
		t.Fatal("ExplainElement should return explanation")
	}
	if explanation.ID != "API" {
		t.Errorf("Expected ID 'API', got '%s'", explanation.ID)
	}
	if !strings.Contains(explanation.Description, "API Service") {
		t.Error("Description should contain system label")
	}
}

func TestExplainElement_NotFound(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{Name: "Test"},
	}

	explainer := NewExplainer(prog)
	_, err := explainer.ExplainElement("NonExistent")
	if err == nil {
		t.Fatal("ExplainElement should error for non-existent element")
	}
	if !strings.Contains(err.Error(), "not found") {
		t.Errorf("Error should mention 'not found', got: %v", err)
	}
}

func TestExplainElement_NoArchitecture(t *testing.T) {
	prog := &language.Program{}
	explainer := NewExplainer(prog)
	_, err := explainer.ExplainElement("API")
	if err == nil {
		t.Fatal("ExplainElement should error when no architecture")
	}
}

func TestExplainElement_Container(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Containers: []*language.Container{
						{
							ID:    "Cont",
							Label: "Container",
						},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Cont")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Cont" {
		t.Errorf("Expected ID 'Cont', got '%s'", explanation.ID)
	}
}

func TestExplainElement_Component(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Containers: []*language.Container{
						{
							ID: "Cont",
							Components: []*language.Component{
								{
									ID:         "Comp",
									Label:      "Component",
									Technology: stringPtr("Go"),
								},
							},
						},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Comp")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Comp" {
		t.Errorf("Expected ID 'Comp', got '%s'", explanation.ID)
	}
	if !strings.Contains(explanation.Description, "Go") {
		t.Error("Description should mention technology")
	}
}

func TestExplainElement_Person(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Persons: []*language.Person{
				{
					ID:    "User",
					Label: "End User",
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("User")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "User" {
		t.Errorf("Expected ID 'User', got '%s'", explanation.ID)
	}
}

func TestFindElement_DataStore(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					DataStores: []*language.DataStore{
						{ID: "DB", Label: "Database"},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("DB")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "DB" {
		t.Errorf("Expected ID 'DB', got '%s'", explanation.ID)
	}
}

func TestFindElement_Queue(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Queues: []*language.Queue{
						{ID: "Q", Label: "Queue"},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Q")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Q" {
		t.Errorf("Expected ID 'Q', got '%s'", explanation.ID)
	}
}

func TestFindElement_ComponentInSystem(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Components: []*language.Component{
						{ID: "Comp", Label: "Component"},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Comp")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Comp" {
		t.Errorf("Expected ID 'Comp', got '%s'", explanation.ID)
	}
}

