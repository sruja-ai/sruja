// pkg/dx/explainer_description_test.go
package dx

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestBuildDescription_DataStore(t *testing.T) {
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
	if explanation.Description == "" {
		t.Error("Should build description for datastore")
	}
}

func TestBuildDescription_Queue(t *testing.T) {
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
	// Queue uses default case in buildDescription
	if explanation.Description == "" {
		t.Error("Should build description for queue")
	}
	if !strings.Contains(explanation.Description, "architecture element") {
		t.Error("Queue description should use default case")
	}
}

func TestBuildDescription_DefaultCase(t *testing.T) {
	// Test with an unknown element type (nil)
	explainer := NewExplainer(&language.Program{Architecture: &language.Architecture{Name: "Test"}})
	desc := explainer.buildDescription(nil)
	if !strings.Contains(desc, "architecture element") {
		t.Error("Default case should mention 'architecture element'")
	}
}

func TestBuildDescription_SystemWithDataStores(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID:    "Sys",
					Label: "System",
					DataStores: []*language.DataStore{
						{ID: "DB", Label: "Database"},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if !strings.Contains(explanation.Description, "data store") {
		t.Error("Should mention data stores in system description")
	}
}

func TestBuildDescription_ContainerWithComponents(t *testing.T) {
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
							Components: []*language.Component{
								{ID: "Comp", Label: "Component"},
							},
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
	if !strings.Contains(explanation.Description, "component") {
		t.Error("Should mention components in container description")
	}
}

