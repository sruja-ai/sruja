// pkg/import/json/coverage_test.go
// Additional tests to improve coverage
package json

import (
	"testing"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertComponentJSONToAST_AllFields(t *testing.T) {
	compJSON := &jsonexport.ComponentJSON{
		ID:          "comp1",
		Label:       "Component",
		Description: mkStr("Component description"),
		Technology:  mkStr("Go"),
		Relations: []jsonexport.RelationJSON{
			{From: "A", To: "B", Verb: mkStr("calls"), Label: mkStr("API call")},
		},
	}

	result := convertComponentJSONToAST(compJSON)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if result.Label != "Component" {
		t.Errorf("expected Label Component, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Component description" {
		t.Errorf("expected Description Component description, got %v", result.Description)
	}
	if len(result.Items) != 1 {
		t.Errorf("expected 1 item (relation), got %d", len(result.Items))
	}
	if result.Items[0].Relation == nil {
		t.Fatal("expected relation in items")
	}
	if result.Items[0].Relation.From.String() != "A" {
		t.Errorf("expected relation From A, got %s", result.Items[0].Relation.From.String())
	}
}

func TestConvertComponentJSONToAST_NoID(t *testing.T) {
	compJSON := &jsonexport.ComponentJSON{
		Label: "Component",
	}

	result := convertComponentJSONToAST(compJSON)

	if result.ID != "Component" {
		t.Errorf("expected ID Component (from label), got %s", result.ID)
	}
}

func TestConvertComponentJSONToAST_InvalidRelations(t *testing.T) {
	compJSON := &jsonexport.ComponentJSON{
		ID:    "comp1",
		Label: "Component",
		Relations: []jsonexport.RelationJSON{
			{From: "", To: "B"},
			{From: "A", To: ""},
			{From: "A", To: "B"},
		},
	}

	result := convertComponentJSONToAST(compJSON)

	if len(result.Items) != 1 {
		t.Errorf("expected 1 valid relation, got %d", len(result.Items))
	}
}

func TestExtractScenariosOnly(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{System: &language.System{ID: "S1", Label: "System"}},
			{Scenario: &language.Scenario{ID: "Sc1", Title: "Scenario 1"}},
			{Scenario: &language.Scenario{ID: "Sc2", Title: "Scenario 2"}},
		},
	}

	result := extractScenariosOnly(arch)

	if result.Name != "Test - Scenarios" {
		t.Errorf("expected name Test - Scenarios, got %s", result.Name)
	}
	if len(result.Items) != 2 {
		t.Errorf("expected 2 scenarios, got %d", len(result.Items))
	}
}

func TestExtractScenariosOnly_NoScenarios(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{System: &language.System{ID: "S1", Label: "System"}},
		},
	}

	result := extractScenariosOnly(arch)

	if len(result.Items) != 0 {
		t.Errorf("expected 0 scenarios, got %d", len(result.Items))
	}
}

func mkStr(s string) *string {
	return &s
}
