package likec4

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func parseDSL(t *testing.T, dsl string) *language.Program {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}
	return prog
}

func TestExporter_NilArchitecture(t *testing.T) {
	exp := NewExporter()
	data, err := exp.Export(nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	var model Model
	if err := json.Unmarshal(data, &model); err != nil {
		t.Fatalf("invalid JSON: %v", err)
	}

	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements, got %d", len(model.Elements))
	}
}

func TestExporter_SimpleArchitecture(t *testing.T) {
	dsl := `model {
		user = person "User"
		app = system "Application" {
			web = container "Web Frontend"
			api = container "API Backend"
		}
		user -> app.web "uses"
	}`
	prog := parseDSL(t, dsl)

	exp := NewExporter()
	model := exp.ToModel(prog)

	// Should have: 1 person + 1 system + 2 containers = 4 elements
	if len(model.Elements) != 4 {
		t.Errorf("expected 4 elements, got %d", len(model.Elements))
	}

	// Expect 2 relations: 1 explicit (user -> app.web) + 1 implied (user -> app)
	if len(model.Relations) != 2 {
		t.Errorf("expected 2 relations, got %d", len(model.Relations))
	}

	if len(model.Views) != 1 {
		t.Errorf("expected 1 view, got %d", len(model.Views))
	}

	// Verify view exists
	if len(model.Views) == 0 {
		t.Error("expected at least one view")
	}
}

func TestExporter_NestedElements(t *testing.T) {
	dsl := `model {
		shop = system "Shop" {
			api = container "API" {
				handler = component "Handler"
				service = component "Service"
			}
			db = database "Database"
		}
	}`
	prog := parseDSL(t, dsl)

	exp := NewExporter()
	model := exp.ToModel(prog)

	// Should have: 1 system + 1 container + 2 components + 1 datastore = 5
	if len(model.Elements) != 5 {
		t.Errorf("expected 5 elements, got %d", len(model.Elements))
	}

	// Verify FQN for nested component
	found := false
	for _, elem := range model.Elements {
		if elem.ID == "shop.api.handler" {
			found = true
			if elem.Parent != "shop.api" {
				t.Errorf("expected parent 'shop.api', got %q", elem.Parent)
			}
		}
	}
	if !found {
		t.Error("nested component with FQN 'shop.api.handler' not found")
	}
}

func TestExporter_ElementKinds(t *testing.T) {
	dsl := `model {
		p1 = person "Person"
		s1 = system "System" {
			c1 = container "Container" {
				comp1 = component "Component"
			}
			db1 = database "Database"
			q1 = queue "Queue"
		}
	}`
	prog := parseDSL(t, dsl)

	exp := NewExporter()
	model := exp.ToModel(prog)

	kinds := make(map[string]string)
	for _, elem := range model.Elements {
		kinds[elem.ID] = elem.Kind
	}

	tests := []struct {
		id   string
		want string
	}{
		{"p1", "person"},
		{"s1", "system"},
		{"s1.c1", "container"},
		{"s1.c1.comp1", "component"},
		{"s1.db1", "database"},
		{"s1.q1", "queue"},
	}

	for _, tt := range tests {
		if got := kinds[tt.id]; got != tt.want {
			t.Errorf("element %q: expected kind %q, got %q", tt.id, tt.want, got)
		}
	}
}

func TestExporter_Metadata(t *testing.T) {
	dsl := `model {
		sys = system "System" {
			metadata {
				owner "team-a"
			}
			slo {
				availability {
					target "99.9%"
				}
			}
		}
	}`
	prog := parseDSL(t, dsl)

	exp := NewExporter()
	model := exp.ToModel(prog)

	if len(model.Elements) != 1 {
		t.Fatalf("expected 1 element, got %d", len(model.Elements))
	}

	meta := model.Elements[0].Metadata
	if meta == nil {
		t.Fatal("expected metadata to be set")
	}

	if owner, ok := meta["owner"].(string); !ok || owner != "team-a" {
		t.Errorf("expected owner 'team-a', got %v", meta["owner"])
	}

	slo, ok := meta["slo"].(map[string]interface{})
	if !ok {
		t.Fatalf("expected slo to be map, got %T", meta["slo"])
	}
	if slo["availability"] != "99.9%" {
		t.Errorf("expected availability '99.9%%', got %v", slo["availability"])
	}
}

func TestExportCompact(t *testing.T) {
	dsl := `model {
		s1 = system "System"
	}`
	prog := parseDSL(t, dsl)

	exp := NewExporter()
	compact, err := exp.ExportCompact(prog)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	regular, err := exp.Export(prog)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Compact should be smaller (no indentation)
	if len(compact) >= len(regular) {
		t.Errorf("compact (%d bytes) should be smaller than regular (%d bytes)",
			len(compact), len(regular))
	}
}
func TestExporter_Complete(t *testing.T) {
	dsl := `model {
		sys = system "System" {
			tech "Go"
			description "desc"
			comp = component "Component" {
				tech "React"
				description "ui"
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	if len(model.Elements) != 2 {
		t.Errorf("expected 2 elements, got %d", len(model.Elements))
	}

	foundComp := false
	for _, e := range model.Elements {
		if e.ID == "sys.comp" {
			foundComp = true
			if e.Kind != "component" {
				t.Errorf("expected kind component, got %q", e.Kind)
			}
			if e.Technology != "React" {
				t.Errorf("expected tech React, got %q", e.Technology)
			}
		}
	}
	if !foundComp {
		t.Error("component sys.comp not found")
	}
}

func TestConvertComponent(t *testing.T) {
	exporter := NewExporter()
	model := exporter.ToModel(&language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{ElementDef: &language.LikeC4ElementDef{
					Assignment: &language.LikeC4Assignment{
						Name: "Comp1",
						Kind: "component",
						Body: &language.LikeC4ElementDefBody{
							Items: []*language.LikeC4BodyItem{
								{Description: strPtr("Comp Description")},
								{Technology: strPtr("Go")},
							},
						},
					},
				}},
			},
		},
	})

	if model == nil {
		t.Fatal("model is nil")
	}

	// Verification logic would go here, but this at least executes the code
	// Actually we should test the internal converter directly if possible
}

func strPtr(s string) *string { return &s }

func TestExporter_ScaleMetadata(t *testing.T) {
	dsl := `model {
		sys = system "System" {
			scale {
				min 1
				max 10
				metric "requests"
			}
		}
		app = system "App" {
			container "Container" {
				scale {
					min 2
				}
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	foundSys := false
	for _, e := range model.Elements {
		if e.ID == "sys" {
			foundSys = true
			if e.Metadata == nil || e.Metadata["scale"] == nil {
				t.Fatal("expected scale metadata for sys")
			}
		}
	}
	if !foundSys {
		t.Error("system sys not found")
	}
}

func TestExporter_ToModel_NilProgram(t *testing.T) {
	exp := NewExporter()
	model := exp.ToModel(nil)
	if model == nil {
		t.Fatal("ToModel should return empty model, not nil")
	}
	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements, got %d", len(model.Elements))
	}
}

func TestExporter_ToModel_NilModel(t *testing.T) {
	exp := NewExporter()
	prog := &language.Program{Model: nil}
	model := exp.ToModel(prog)
	if model == nil {
		t.Fatal("ToModel should return empty model, not nil")
	}
	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements, got %d", len(model.Elements))
	}
}

func TestExporter_ToModel_EmptyElements(t *testing.T) {
	exp := NewExporter()
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{},
		},
	}
	model := exp.ToModel(prog)
	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements, got %d", len(model.Elements))
	}
	if len(model.Views) != 0 {
		t.Errorf("expected 0 views when no elements, got %d", len(model.Views))
	}
}

func TestExporter_RelationWithVerb(t *testing.T) {
	dsl := `model {
		a = system "A"
		b = system "B"
		a -> b "calls"
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	if len(model.Relations) == 0 {
		t.Fatal("expected at least one relation")
	}
	rel := model.Relations[0]
	if rel.Title == "" {
		t.Error("relation should have title from verb")
	}
}

func TestExporter_RelationWithLabel(t *testing.T) {
	dsl := `model {
		a = system "A"
		b = system "B"
		a -> b "custom label"
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	if len(model.Relations) == 0 {
		t.Fatal("expected at least one relation")
	}
	rel := model.Relations[0]
	if rel.Title != "custom label" {
		t.Errorf("expected relation title 'custom label', got %q", rel.Title)
	}
}

func TestExporter_SystemWithErrorRate(t *testing.T) {
	// Test that systems with errorRate SLO are handled
	dsl := `model {
		sys = system "System" {
			slo {
				errorRate {
					target "0.1%"
				}
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	// Just verify the system exists and model is created
	if len(model.Elements) == 0 {
		t.Error("expected at least one element")
	}
}

func TestExporter_ContainerWithCost(t *testing.T) {
	// Test that containers with cost SLO are handled
	dsl := `model {
		sys = system "System" {
			cont = container "Container" {
				slo {
					cost {
						target "$100"
					}
				}
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	// Just verify the container exists
	found := false
	for _, e := range model.Elements {
		if e.ID == "sys.cont" {
			found = true
			break
		}
	}
	if !found {
		t.Error("container sys.cont not found")
	}
}

func TestExporter_SystemWithConstraints(t *testing.T) {
	// Test that systems with constraints are handled
	dsl := `model {
		sys = system "System" {
			constraints "no external deps"
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	// Just verify the system exists
	found := false
	for _, e := range model.Elements {
		if e.ID == "sys" {
			found = true
			break
		}
	}
	if !found {
		t.Error("system sys not found")
	}
}

func TestExporter_ElementWithEmptyID(t *testing.T) {
	// Test that elements with empty IDs are skipped
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{
					ElementDef: &language.LikeC4ElementDef{
						Assignment: &language.LikeC4Assignment{
							Name: "",
							Kind: "system",
						},
					},
				},
			},
		},
	}
	exp := NewExporter()
	model := exp.ToModel(prog)
	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements (empty ID should be skipped), got %d", len(model.Elements))
	}
}

func TestExporter_BuildFQN_EdgeCases(t *testing.T) {
	// Test buildFQN helper through actual usage
	dsl := `model {
		parent = system "Parent" {
			child = container "Child"
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	found := false
	for _, e := range model.Elements {
		if e.ID == "parent.child" {
			found = true
			if e.Parent != "parent" {
				t.Errorf("expected parent 'parent', got %q", e.Parent)
			}
		}
	}
	if !found {
		t.Error("child element not found")
	}
}

func TestExporter_MetadataWithArray(t *testing.T) {
	dsl := `model {
		sys = system "System" {
			metadata {
				tags ["tag1", "tag2"]
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	found := false
	for _, e := range model.Elements {
		if e.ID == "sys" {
			found = true
			if e.Metadata == nil {
				t.Fatal("expected metadata")
			}
			tags, ok := e.Metadata["tags"].([]string)
			if !ok {
				t.Fatalf("expected tags as []string, got %T", e.Metadata["tags"])
			}
			if len(tags) != 2 {
				t.Errorf("expected 2 tags, got %d", len(tags))
			}
		}
	}
	if !found {
		t.Error("system sys not found")
	}
}

func TestExporter_TechnologyFromProperties(t *testing.T) {
	dsl := `model {
		cont = container "Container" {
			technology "Go"
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	found := false
	for _, e := range model.Elements {
		if e.ID == "cont" {
			found = true
			if e.Technology != "Go" {
				t.Errorf("expected technology 'Go', got %q", e.Technology)
			}
		}
	}
	if !found {
		t.Error("container cont not found")
	}
}

func TestExporter_ExportCompact_Nil(t *testing.T) {
	exp := NewExporter()
	data, err := exp.ExportCompact(nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	var model Model
	if err := json.Unmarshal(data, &model); err != nil {
		t.Fatalf("invalid JSON: %v", err)
	}
	if len(model.Elements) != 0 {
		t.Errorf("expected 0 elements, got %d", len(model.Elements))
	}
}

func TestExporter_QueueElements(t *testing.T) {
	dsl := `model {
		sys = system "Sys" {
			q = queue "Q" {
				technology "Kafka"
				description "Messages"
			}
		}
		container = container "Cont" {
			q2 = queue "Q2"
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	foundQ := false
	foundQ2 := false
	for _, e := range model.Elements {
		if e.ID == "sys.q" {
			foundQ = true
			if e.Kind != "queue" {
				t.Errorf("expected kind queue, got %q", e.Kind)
			}
			if e.Technology != "Kafka" {
				t.Errorf("expected technology Kafka, got %q", e.Technology)
			}
			if e.Description != "Messages" {
				t.Errorf("expected description Messages, got %q", e.Description)
			}
		}
		if e.ID == "container.q2" {
			foundQ2 = true
			if e.Kind != "queue" {
				t.Errorf("expected kind queue, got %q", e.Kind)
			}
		}
	}
	if !foundQ {
		t.Error("queue sys.q not found")
	}
	if !foundQ2 {
		t.Error("queue container.q2 not found")
	}
}

func TestExporter_DataStoreElements(t *testing.T) {
	dsl := `model {
		sys = system "Sys" {
			ds = database "DB" {
				technology "Postgres"
				description "Main DB"
			}
		}
		container = container "Cont" {
			ds2 = database "DB2"
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	foundDS := false
	foundDS2 := false
	for _, e := range model.Elements {
		if e.ID == "sys.ds" {
			foundDS = true
			if e.Kind != "database" {
				t.Errorf("expected kind database, got %q", e.Kind)
			}
			if e.Technology != "Postgres" {
				t.Errorf("expected technology Postgres, got %q", e.Technology)
			}
			if e.Description != "Main DB" {
				t.Errorf("expected description Main DB, got %q", e.Description)
			}
		}
		if e.ID == "container.ds2" {
			foundDS2 = true
			if e.Kind != "database" {
				t.Errorf("expected kind database, got %q", e.Kind)
			}
		}
	}
	if !foundDS {
		t.Error("database sys.ds not found")
	}
	if !foundDS2 {
		t.Error("database container.ds2 not found")
	}
}

func TestExporter_EdgeCase_NilProperties(t *testing.T) {
	// Manually construct element with nil properties to test helpers
	// Note: It's hard to trigger nil pointer dereferences from DSL because parser handles initializing fields
	// So we test helper functions directly where possible or create edge case models

	// Test getTechnology with nil map
	if tech := getTechnology(nil); tech != "" {
		t.Errorf("getTechnology(nil) = %q, want empty", tech)
	}

	// Test convertMetadata with nil/empty
	if m := convertMetadata(nil); m != nil {
		t.Errorf("convertMetadata(nil) = %v, want nil", m)
	}

	// Test getString with nil
	if s := getString(nil); s != "" {
		t.Errorf("getString(nil) = %q, want empty", s)
	}
}

func TestExporter_ScaleBlock_Full(t *testing.T) {
	dsl := `model {
		sys = system "System" {
			scale {
				min 1
				max 10
				metric "cpu"
			}
		}
	}`
	prog := parseDSL(t, dsl)
	exp := NewExporter()
	model := exp.ToModel(prog)

	found := false
	for _, e := range model.Elements {
		if e.ID == "sys" {
			found = true
			scale, ok := e.Metadata["scale"].(map[string]interface{})
			if !ok {
				t.Fatal("expected scale map")
			}
			if scale["min"] != 1 {
				t.Errorf("expected min 1, got %v", scale["min"])
			}
			if scale["max"] != 10 {
				t.Errorf("expected max 10, got %v", scale["max"])
			}
			if scale["metric"] != "cpu" {
				t.Errorf("expected metric cpu, got %v", scale["metric"])
			}
		}
	}
	if !found {
		t.Error("system not found")
	}
}
