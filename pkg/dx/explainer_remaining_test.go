// pkg/dx/explainer_remaining_test.go
package dx

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestFindRelations_ArchitectureLevel(t *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
				{ID: "Sys2", Label: "System 2"},
			},
			Relations: []*language.Relation{
				{From: "Sys1", To: "Sys2"},
			},
		},
	}

	explainer := NewExplainer(program)
	info := explainer.findRelations("Sys1")

	if len(info.Outgoing) != 1 {
		t.Errorf("Expected 1 outgoing relation, got %d", len(info.Outgoing))
	}
	if info.Outgoing[0].To != "Sys2" {
		t.Errorf("Expected relation to Sys2, got %s", info.Outgoing[0].To)
	}
}

func TestFindRelations_ArchitectureLevel_Incoming(t *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{ID: "Sys1", Label: "System 1"},
				{ID: "Sys2", Label: "System 2"},
			},
			Relations: []*language.Relation{
				{From: "Sys1", To: "Sys2"},
			},
		},
	}

	explainer := NewExplainer(program)
	info := explainer.findRelations("Sys2")

	if len(info.Incoming) != 1 {
		t.Errorf("Expected 1 incoming relation, got %d", len(info.Incoming))
	}
	if info.Incoming[0].From != "Sys1" {
		t.Errorf("Expected relation from Sys1, got %s", info.Incoming[0].From)
	}
}

func TestExtractMetadata_OtherTypes(t *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{},
	}
	explainer := NewExplainer(program)

	// Test with non-System/Container/Component type
	person := &language.Person{ID: "User"}
	metadata := explainer.extractMetadata(person)

	if len(metadata) != 0 {
		t.Errorf("Expected 0 metadata entries for Person, got %d", len(metadata))
	}
}

func TestExtractMetadata_EmptyMetadata(t *testing.T) {
	sys := &language.System{
		ID:       "Sys1",
		Metadata: []*language.MetaEntry{},
	}

	program := &language.Program{
		Architecture: &language.Architecture{},
	}
	explainer := NewExplainer(program)
	metadata := explainer.extractMetadata(sys)

	if len(metadata) != 0 {
		t.Errorf("Expected 0 metadata entries, got %d", len(metadata))
	}
}
