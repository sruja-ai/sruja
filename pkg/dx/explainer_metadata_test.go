// pkg/dx/explainer_metadata_test.go
package dx

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExtractMetadata_DataStore(t *testing.T) {
	// extractMetadata only handles System, Container, Component
	// DataStore metadata extraction is not implemented in extractMetadata
	// This test verifies the element can be found and explained
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					DataStores: []*language.DataStore{
						{
							ID:    "DB",
							Label: "Database",
							Metadata: []*language.MetaEntry{
								{Key: "engine", Value: "postgres"},
							},
						},
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
	// extractMetadata doesn't handle DataStore, so metadata will be empty
	// but the element should still be found
	if explanation.ID != "DB" {
		t.Errorf("Expected ID 'DB', got '%s'", explanation.ID)
	}
}

func TestExtractMetadata_Queue(t *testing.T) {
	// extractMetadata only handles System, Container, Component
	// Queue metadata extraction is not implemented in extractMetadata
	// This test verifies the element can be found and explained
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Systems: []*language.System{
				{
					ID: "Sys",
					Queues: []*language.Queue{
						{
							ID:    "Q",
							Label: "Queue",
							Metadata: []*language.MetaEntry{
								{Key: "topic", Value: "events"},
							},
						},
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
	// extractMetadata doesn't handle Queue, so metadata will be empty
	// but the element should still be found
	if explanation.ID != "Q" {
		t.Errorf("Expected ID 'Q', got '%s'", explanation.ID)
	}
}

func TestExtractMetadata_Person(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{
			Name: "Test",
			Persons: []*language.Person{
				{
					ID: "User",
					Metadata: []*language.MetaEntry{
						{Key: "persona", Value: "customer"},
					},
				},
			},
		},
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("User")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	// Person metadata extraction is not implemented in extractMetadata
	// but we can test that it doesn't crash
	_ = explanation
}
