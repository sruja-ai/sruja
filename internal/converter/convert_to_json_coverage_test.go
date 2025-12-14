// internal/converter/convert_to_json_coverage_test.go
// Additional tests to improve coverage for convert_to_json functions
package converter

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertDataStore_AllFields(t *testing.T) {
	tech := "PostgreSQL"
	ds := &language.DataStore{
		ID:          "ds1",
		Label:       "Database",
		Description: mkStr("Main database"),
		Items: []language.DataStoreItem{
			{Technology: &tech},
			{Metadata: &language.MetadataBlock{
				Entries: []*language.MetaEntry{
					{Key: "region", Value: mkStr("us-east")},
				},
			}},
		},
	}

	result := convertDataStore(ds)

	if result.ID != "ds1" {
		t.Errorf("expected ID ds1, got %s", result.ID)
	}
	if result.Label != "Database" {
		t.Errorf("expected Label Database, got %s", result.Label)
	}
	if result.Description != "Main database" {
		t.Errorf("expected Description Main database, got %s", result.Description)
	}
	if result.Technology != "PostgreSQL" {
		t.Errorf("expected Technology PostgreSQL, got %s", result.Technology)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}
}

func TestConvertQueue_AllFields(t *testing.T) {
	tech := "RabbitMQ"
	q := &language.Queue{
		ID:          "q1",
		Label:       "Message Queue",
		Description: mkStr("Event queue"),
		Items: []language.QueueItem{
			{Technology: &tech},
			{Metadata: &language.MetadataBlock{
				Entries: []*language.MetaEntry{
					{Key: "durable", Value: mkStr("true")},
				},
			}},
		},
	}

	result := convertQueue(q)

	if result.ID != "q1" {
		t.Errorf("expected ID q1, got %s", result.ID)
	}
	if result.Label != "Message Queue" {
		t.Errorf("expected Label Message Queue, got %s", result.Label)
	}
	if result.Description != "Event queue" {
		t.Errorf("expected Description Event queue, got %s", result.Description)
	}
	if result.Technology != "RabbitMQ" {
		t.Errorf("expected Technology RabbitMQ, got %s", result.Technology)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}
}

func TestConvertPerson_AllFields(t *testing.T) {
	p := &language.Person{
		ID:    "p1",
		Label: "User",
		Items: []language.PersonItem{
			{Description: mkStr("End user")},
			{Metadata: &language.MetadataBlock{
				Entries: []*language.MetaEntry{
					{Key: "role", Value: mkStr("customer")},
				},
			}},
		},
	}

	result := convertPerson(p)

	if result.ID != "p1" {
		t.Errorf("expected ID p1, got %s", result.ID)
	}
	if result.Label != "User" {
		t.Errorf("expected Label User, got %s", result.Label)
	}
	if result.Description != "End user" {
		t.Errorf("expected Description End user, got %s", result.Description)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}
}

func TestConvertComponent_AllFields(t *testing.T) {
	tech := "Go"
	comp := &language.Component{
		ID:          "comp1",
		Label:       "Component",
		Description: mkStr("Description"),
		Technology:  &tech,
		Items: []language.ComponentItem{
			{Technology: &tech},
			{Metadata: &language.MetadataBlock{
				Entries: []*language.MetaEntry{
					{Key: "key1", Value: mkStr("value1")},
				},
			}},
		},
	}

	result := convertComponent(comp)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if result.Label != "Component" {
		t.Errorf("expected Label Component, got %s", result.Label)
	}
	if result.Description != "Description" {
		t.Errorf("expected Description, got %s", result.Description)
	}
	if result.Technology != "Go" {
		t.Errorf("expected Technology Go, got %s", result.Technology)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}
}

func TestConvertRequirement_AllFields(t *testing.T) {
	reqType := "functional"
	req := &language.Requirement{
		ID:          "req1",
		Type:        &reqType,
		Description: mkStr("Requirement description"),
		Body: &language.RequirementBody{
			Type:        &reqType,
			Description: mkStr("Body description"),
			Tags:        []string{"tag1", "tag2"},
		},
	}

	result := convertRequirement(req)

	if result.ID != "req1" {
		t.Errorf("expected ID req1, got %s", result.ID)
	}
	if result.Type != "functional" {
		t.Errorf("expected Type functional, got %s", result.Type)
	}
	if result.Title != "Body description" {
		t.Errorf("expected Title Body description, got %s", result.Title)
	}
	if result.Description != "Body description" {
		t.Errorf("expected Description Body description, got %s", result.Description)
	}
	if len(result.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(result.Tags))
	}
}

func TestConvertRequirement_NoBody(t *testing.T) {
	reqType := "functional"
	req := &language.Requirement{
		ID:          "req1",
		Type:        &reqType,
		Description: mkStr("Requirement description"),
	}

	result := convertRequirement(req)

	if result.ID != "req1" {
		t.Errorf("expected ID req1, got %s", result.ID)
	}
	if result.Type != "functional" {
		t.Errorf("expected Type functional, got %s", result.Type)
	}
	if result.Title != "Requirement description" {
		t.Errorf("expected Title Requirement description, got %s", result.Title)
	}
}

func TestConvertFlow_AllFields(t *testing.T) {
	desc := "Flow description"
	flow := &language.Flow{
		ID:          "flow1",
		Title:       "Flow Title",
		Description: &desc,
		Steps: []*language.ScenarioStep{
			{
				From:        language.QualifiedIdent{Parts: []string{"A"}},
				To:          language.QualifiedIdent{Parts: []string{"B"}},
				Description: mkStr("Step 1"),
			},
			{
				From: language.QualifiedIdent{Parts: []string{"B"}},
				To:   language.QualifiedIdent{Parts: []string{"C"}},
			},
		},
	}

	result := convertFlow(flow)

	if result.ID != "flow1" {
		t.Errorf("expected ID flow1, got %s", result.ID)
	}
	if result.Title != "Flow Title" {
		t.Errorf("expected Title Flow Title, got %s", result.Title)
	}
	if result.Label != "Flow Title" {
		t.Errorf("expected Label Flow Title, got %s", result.Label)
	}
	if result.Description != "Flow description" {
		t.Errorf("expected Description Flow description, got %s", result.Description)
	}
	if len(result.Steps) != 2 {
		t.Errorf("expected 2 steps, got %d", len(result.Steps))
	}
	if result.Steps[0].Description != "Step 1" {
		t.Errorf("expected Step 1 description, got %s", result.Steps[0].Description)
	}
	if result.Steps[1].Description != "" {
		t.Errorf("expected empty description for step 2, got %s", result.Steps[1].Description)
	}
}

func TestConvertFlow_NoSteps(t *testing.T) {
	flow := &language.Flow{
		ID:    "flow1",
		Title: "Flow Title",
	}

	result := convertFlow(flow)

	if result.ID != "flow1" {
		t.Errorf("expected ID flow1, got %s", result.ID)
	}
	if len(result.Steps) != 0 {
		t.Errorf("expected 0 steps, got %d", len(result.Steps))
	}
}

func TestConvertPolicy_AllFields(t *testing.T) {
	category := "security"
	enforcement := "required"
	policy := &language.Policy{
		ID:                "pol1",
		Description:       "Policy description",
		InlineCategory:    &category,
		InlineEnforcement: &enforcement,
		Body: &language.PolicyBody{
			Category:    &category,
			Enforcement: &enforcement,
			Tags:        []string{"tag1", "tag2"},
		},
	}

	result := convertPolicy(policy)

	if result.ID != "pol1" {
		t.Errorf("expected ID pol1, got %s", result.ID)
	}
	if result.Description != "Policy description" {
		t.Errorf("expected Description Policy description, got %s", result.Description)
	}
	if result.Category != "security" {
		t.Errorf("expected Category security, got %s", result.Category)
	}
	if result.Enforcement != "required" {
		t.Errorf("expected Enforcement required, got %s", result.Enforcement)
	}
	if len(result.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(result.Tags))
	}
}

func TestConvertPolicy_InlineOnly(t *testing.T) {
	category := "security"
	enforcement := "required"
	policy := &language.Policy{
		ID:                "pol1",
		Description:       "Policy description",
		InlineCategory:    &category,
		InlineEnforcement: &enforcement,
	}

	result := convertPolicy(policy)

	if result.ID != "pol1" {
		t.Errorf("expected ID pol1, got %s", result.ID)
	}
	if result.Category != "security" {
		t.Errorf("expected Category security, got %s", result.Category)
	}
	if result.Enforcement != "required" {
		t.Errorf("expected Enforcement required, got %s", result.Enforcement)
	}
}

func TestConvertPolicy_BodyOnly(t *testing.T) {
	category := "security"
	enforcement := "required"
	policy := &language.Policy{
		ID:          "pol1",
		Description: "Policy description",
		Body: &language.PolicyBody{
			Category:    &category,
			Enforcement: &enforcement,
			Tags:        []string{"tag1"},
		},
	}

	result := convertPolicy(policy)

	if result.ID != "pol1" {
		t.Errorf("expected ID pol1, got %s", result.ID)
	}
	if result.Category != "security" {
		t.Errorf("expected Category security, got %s", result.Category)
	}
	if result.Enforcement != "required" {
		t.Errorf("expected Enforcement required, got %s", result.Enforcement)
	}
	if len(result.Tags) != 1 {
		t.Errorf("expected 1 tag, got %d", len(result.Tags))
	}
}

func mkStr(s string) *string {
	return &s
}
