// pkg/export/json/json_test.go
package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertDataStore_WithMetadata(t *testing.T) {
	ds := &language.DataStore{
		ID:          "db1",
		Label:       "Database",
		Description: strPtr("Main DB"),
		Technology:  strPtr("PostgreSQL"),
		Metadata: []*language.MetaEntry{
			{Key: "version", Value: strPtr("14")},
		},
	}
	result := dataStoreToJSON(ds)
	if result.ID != "db1" || result.Label != "Database" {
		t.Fatalf("basic fields not converted correctly: %+v", result)
	}
	if len(result.Metadata) != 1 || result.Metadata[0].Key != "version" {
		t.Fatalf("metadata not converted correctly: %+v", result.Metadata)
	}
}

func TestConvertQueue_WithProperties(t *testing.T) {
	q := &language.Queue{
		ID:         "queue1",
		Label:      "Event Queue",
		Technology: strPtr("RabbitMQ"),
	}
	result := queueToJSON(q)
	if result.ID != "queue1" {
		t.Fatalf("queue ID not converted: %+v", result)
	}
}

func TestConvertPerson_Complete(t *testing.T) {
	p := &language.Person{
		ID:          "user1",
		Label:       "End User",
		Description: strPtr("System user"),
		Style:       map[string]string{"color": "blue"},
	}
	result := personToJSON(p)
	if result.ID != "user1" || result.Label != "End User" {
		t.Fatalf("person fields not converted: %+v", result)
	}
}

func TestConvertConstraints_Multiple(t *testing.T) {
	constraints := []*language.ConstraintEntry{
		{Key: "security", Value: "TLS required"},
		{Key: "performance", Value: "< 100ms"},
	}
	result := convertConstraints(constraints)
	if len(result) != 2 {
		t.Fatalf("expected 2 constraints, got %d", len(result))
	}
}

func TestConvertConventions_Multiple(t *testing.T) {
	conventions := []*language.ConventionEntry{
		{Key: "naming", Value: "camelCase"},
		{Key: "versioning", Value: "semver"},
	}
	result := convertConventions(conventions)
	if len(result) != 2 {
		t.Fatalf("expected 2 conventions, got %d", len(result))
	}
}

func TestConvertPolicies_Multiple(t *testing.T) {
	policies := []*language.Policy{
		{ID: "pol1", Description: "Security policy"},
		{ID: "pol2", Description: "Data policy"},
	}
	result := convertPolicies(policies)
	if len(result) != 2 {
		t.Fatalf("expected 2 policies, got %d", len(result))
	}
	if result[0].ID != "pol1" || result[1].ID != "pol2" {
		t.Fatalf("policies not converted correctly: %+v", result)
	}
}

func TestExporter(t *testing.T) {
	program := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{},
		},
	}
	exporter := NewExporter()

	// Test Export
	_, err := exporter.Export(program)
	if err != nil {
		t.Errorf("Export failed: %v", err)
	}

	// Test ToModelDump
	dump := exporter.ToModelDump(program)
	if dump == nil {
		t.Error("ExportAsModelDump returned nil")
	}

	// Test ExportCompact
	_, err = exporter.ExportCompact(program)
	if err != nil {
		t.Errorf("ExportCompact failed: %v", err)
	}
}

func TestNewMetadata(t *testing.T) {
	m := NewMetadata("test-app")
	if m.Name != "test-app" {
		t.Errorf("expected name test-app, got %s", m.Name)
	}
}

func TestConvertSystem_Complete(t *testing.T) {
	s := &language.System{
		ID:          "sys1",
		Label:       "System 1",
		Description: strPtr("Main system"),
		Containers: []*language.Container{
			{ID: "cont1", Label: "Container 1"},
		},
		Components: []*language.Component{
			{ID: "comp1", Label: "Component 1"},
		},
		DataStores: []*language.DataStore{
			{ID: "ds1", Label: "DataStore 1"},
		},
		Queues: []*language.Queue{
			{ID: "q1", Label: "Queue 1"},
		},

		Style: map[string]string{"color": "red"},
	}
	result := systemToJSON(s)
	if result.ID != "sys1" || len(result.Containers) != 1 || len(result.Components) != 1 {
		t.Fatalf("system conversion failed: %+v", result)
	}
	if len(result.DataStores) != 1 || len(result.Queues) != 1 {
		t.Fatalf("system nested elements conversion failed: %+v", result)
	}
	if result.Style["color"] != "red" {
		t.Fatalf("system properties/style conversion failed: %+v", result)
	}
}

func TestWrapper(t *testing.T) {
	program := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "system",
							Name:  "SystemA",
							Title: strPtr("System A"),
						},
					},
				},
			},
		},
	}

	wrapper := NewWrapper()

	if wrapper == nil {
		t.Fatal("NewWrapper returned nil")
	}

	wrapper.Extended = true

	_, err := wrapper.Export(program)
	if err != nil {
		t.Errorf("Wrapper.Export failed: %v", err)
	}

	dump := wrapper.ExportAsModelDump(program)
	if dump == nil {
		t.Error("ExportAsModelDump returned nil")
	}

	_, err = wrapper.ExportCompact(program)
	if err != nil {
		t.Errorf("Wrapper.ExportCompact failed: %v", err)
	}
}

func TestWrapper_NoExtended(t *testing.T) {
	program := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{},
		},
	}

	wrapper := NewWrapper()
	wrapper.Extended = false

	_, err := wrapper.Export(program)
	if err != nil {
		t.Errorf("Wrapper.Export with Extended=false failed: %v", err)
	}

	dump := wrapper.ExportAsModelDump(program)
	if dump == nil {
		t.Error("ExportAsModelDump returned nil")
	}

	_, err = wrapper.ExportCompact(program)
	if err != nil {
		t.Errorf("Wrapper.ExportCompact with Extended=false failed: %v", err)
	}
}

func TestWrapper_EmptyProgram(t *testing.T) {
	program := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{},
		},
	}

	wrapper := NewWrapper()

	_, err := wrapper.Export(program)
	if err != nil {
		t.Errorf("Wrapper.Export with empty program failed: %v", err)
	}

	dump := wrapper.ExportAsModelDump(program)
	if dump == nil {
		t.Error("ExportAsModelDump returned nil for empty program")
	}

	_, err = wrapper.ExportCompact(program)
	if err != nil {
		t.Errorf("Wrapper.ExportCompact with empty program failed: %v", err)
	}
}

func TestExporter_ExtendedFlag(t *testing.T) {
	program := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "person",
							Name:  "User",
							Title: strPtr("End User"),
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()
	exporter.Extended = true

	result, err := exporter.Export(program)
	if err != nil {
		t.Errorf("Exporter.Export with Extended=true failed: %v", err)
	}

	if result == "" {
		t.Error("Expected non-empty result")
	}

	dump := exporter.ToModelDump(program)
	if dump == nil {
		t.Error("ToModelDump returned nil")
		return
	}

	if dump.Elements == nil {
		t.Error("Expected Elements map in dump")
	}
}

func TestExporter_NilModel(t *testing.T) {
	program := &language.Program{
		Model: nil,
	}

	exporter := NewExporter()
	_, err := exporter.Export(program)
	if err != nil {
		t.Errorf("Exporter.Export with nil model failed: %v", err)
	}

	dump := exporter.ToModelDump(program)
	if dump == nil {
		t.Error("ToModelDump returned nil for nil model")
	}
}
