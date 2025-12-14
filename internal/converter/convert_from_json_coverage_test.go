// internal/converter/convert_from_json_coverage_test.go
// Additional tests to improve coverage for convert_from_json functions
package converter

import (
	"testing"
)

func TestConvertRequirementFromJSON_AllFields(t *testing.T) {
	req := &Requirement{
		ID:          "req1",
		Type:        "functional",
		Title:       "Requirement Title",
		Description: "Requirement Description",
		Tags:        []string{"tag1", "tag2"},
	}

	result := convertRequirementFromJSON(req)

	if result.ID != "req1" {
		t.Errorf("expected ID req1, got %s", result.ID)
	}
	if result.Type == nil || *result.Type != "functional" {
		t.Errorf("expected Type functional, got %v", result.Type)
	}
	if result.Description == nil || *result.Description != "Requirement Description" {
		t.Errorf("expected Description Requirement Description, got %v", result.Description)
	}
	if result.Body == nil {
		t.Fatal("expected Body to be non-nil")
	}
	if result.Body.Type == nil || *result.Body.Type != "functional" {
		t.Errorf("expected Body.Type functional, got %v", result.Body.Type)
	}
	if result.Body.Description == nil || *result.Body.Description != "Requirement Description" {
		t.Errorf("expected Body.Description Requirement Description, got %v", result.Body.Description)
	}
}

func TestConvertRequirementFromJSON_TitleOnly(t *testing.T) {
	req := &Requirement{
		ID:    "req1",
		Title: "Requirement Title",
	}

	result := convertRequirementFromJSON(req)

	if result.ID != "req1" {
		t.Errorf("expected ID req1, got %s", result.ID)
	}
	if result.Description == nil || *result.Description != "Requirement Title" {
		t.Errorf("expected Description Requirement Title, got %v", result.Description)
	}
}

func TestConvertRequirementFromJSON_TypeOnly(t *testing.T) {
	req := &Requirement{
		ID:   "req1",
		Type: "functional",
	}

	result := convertRequirementFromJSON(req)

	if result.ID != "req1" {
		t.Errorf("expected ID req1, got %s", result.ID)
	}
	if result.Type == nil || *result.Type != "functional" {
		t.Errorf("expected Type functional, got %v", result.Type)
	}
	if result.Body == nil {
		t.Fatal("expected Body to be non-nil")
	}
}

func TestConvertComponentFromJSON_AllFields(t *testing.T) {
	comp := &Component{
		ID:          "comp1",
		Label:       "Component",
		Description: "Component description",
		Technology:  "Go",
		Metadata: []MetadataEntryJSON{
			{Key: "key1", Value: mkStr("value1")},
		},
	}

	result := convertComponentFromJSON(comp)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if result.Label != "Component" {
		t.Errorf("expected Label Component, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Component description" {
		t.Errorf("expected Description Component description, got %v", result.Description)
	}
	if len(result.Items) != 2 {
		t.Errorf("expected 2 items (technology + metadata), got %d", len(result.Items))
	}
}

func TestConvertComponentFromJSON_NoTechnology(t *testing.T) {
	comp := &Component{
		ID:    "comp1",
		Label: "Component",
		Metadata: []MetadataEntryJSON{
			{Key: "key1", Value: mkStr("value1")},
		},
	}

	result := convertComponentFromJSON(comp)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if len(result.Items) != 1 {
		t.Errorf("expected 1 item (metadata), got %d", len(result.Items))
	}
}

func TestConvertDataStoreFromJSON_AllFields(t *testing.T) {
	ds := &DataStore{
		ID:          "ds1",
		Label:       "Database",
		Description: "Main database",
		Technology:  "PostgreSQL",
		Metadata: []MetadataEntryJSON{
			{Key: "region", Value: mkStr("us-east")},
		},
	}

	result := convertDataStoreFromJSON(ds)

	if result.ID != "ds1" {
		t.Errorf("expected ID ds1, got %s", result.ID)
	}
	if result.Label != "Database" {
		t.Errorf("expected Label Database, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Main database" {
		t.Errorf("expected Description Main database, got %v", result.Description)
	}
	if len(result.Items) != 2 {
		t.Errorf("expected 2 items (technology + metadata), got %d", len(result.Items))
	}
}

func TestConvertQueueFromJSON_AllFields(t *testing.T) {
	q := &Queue{
		ID:          "q1",
		Label:       "Message Queue",
		Description: "Event queue",
		Technology:  "RabbitMQ",
		Metadata: []MetadataEntryJSON{
			{Key: "durable", Value: mkStr("true")},
		},
	}

	result := convertQueueFromJSON(q)

	if result.ID != "q1" {
		t.Errorf("expected ID q1, got %s", result.ID)
	}
	if result.Label != "Message Queue" {
		t.Errorf("expected Label Message Queue, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Event queue" {
		t.Errorf("expected Description Event queue, got %v", result.Description)
	}
	if len(result.Items) != 2 {
		t.Errorf("expected 2 items (technology + metadata), got %d", len(result.Items))
	}
}

func TestConvertPersonFromJSON_AllFields(t *testing.T) {
	p := &Person{
		ID:          "p1",
		Label:       "User",
		Description: "End user",
		Metadata: []MetadataEntryJSON{
			{Key: "role", Value: mkStr("customer")},
		},
	}

	result := convertPersonFromJSON(p)

	if result.ID != "p1" {
		t.Errorf("expected ID p1, got %s", result.ID)
	}
	if result.Label != "User" {
		t.Errorf("expected Label User, got %s", result.Label)
	}
	if len(result.Items) != 2 {
		t.Errorf("expected 2 items (description + metadata), got %d", len(result.Items))
	}
}

func TestConvertPersonFromJSON_NoDescription(t *testing.T) {
	p := &Person{
		ID:    "p1",
		Label: "User",
		Metadata: []MetadataEntryJSON{
			{Key: "role", Value: mkStr("customer")},
		},
	}

	result := convertPersonFromJSON(p)

	if result.ID != "p1" {
		t.Errorf("expected ID p1, got %s", result.ID)
	}
	if len(result.Items) != 1 {
		t.Errorf("expected 1 item (metadata), got %d", len(result.Items))
	}
}
