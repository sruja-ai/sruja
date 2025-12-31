package dsl

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/json"
)

func TestPrint_NilModel(t *testing.T) {
	result := Print(nil)
	if result != "" {
		t.Errorf("Expected empty string for nil model, got: %s", result)
	}
}

func TestPrint_EmptyModel(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
	}

	result := Print(model)
	if result != "" && strings.TrimSpace(result) != "" {
		t.Errorf("Expected empty output for empty model, got: %q", result)
	}
}

func TestPrint_SingleElement(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"MySystem": {
				ID:          "MySystem",
				Kind:        "system",
				Title:       "My System",
				Description: "A test system",
				Technology:  "Go",
			},
		},
	}

	result := Print(model)

	// Check element appears
	if !strings.Contains(result, "MySystem = system") {
		t.Error("Expected element definition")
	}
	if !strings.Contains(result, `"My System"`) {
		t.Error("Expected element title")
	}
	if !strings.Contains(result, `description "A test system"`) {
		t.Error("Expected description")
	}
	if !strings.Contains(result, `technology "Go"`) {
		t.Error("Expected technology")
	}
}

func TestPrint_NestedElements(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"System1": {
				ID:    "System1",
				Kind:  "system",
				Title: "System One",
			},
			"System1.Container1": {
				ID:     "System1.Container1",
				Kind:   "container",
				Title:  "Container One",
				Parent: "System1",
			},
		},
	}

	result := Print(model)

	// Parent should appear
	if !strings.Contains(result, "System1 = system") {
		t.Error("Expected parent element")
	}
	// Child should use short name
	if !strings.Contains(result, "Container1 = container") {
		t.Error("Expected nested element with short name")
	}
}

func TestPrint_Relations(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"A": {ID: "A", Kind: "system", Title: "A"},
			"B": {ID: "B", Kind: "system", Title: "B"},
		},
		Relations: []json.RelationDump{
			{
				ID:         "A-B",
				Source:     json.FqnRefDump{Model: "A"},
				Target:     json.FqnRefDump{Model: "B"},
				Title:      "uses",
				Technology: "REST",
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, "A -> B") {
		t.Error("Expected relation from A to B")
	}
	if !strings.Contains(result, `"uses"`) {
		t.Error("Expected relation title")
	}
	if !strings.Contains(result, `technology "REST"`) {
		t.Error("Expected relation technology")
	}
}

func TestPrint_BidirectionalRelation(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"A": {ID: "A", Kind: "system", Title: "A"},
			"B": {ID: "B", Kind: "system", Title: "B"},
		},
		Relations: []json.RelationDump{
			{
				ID:     "A-B",
				Source: json.FqnRefDump{Model: "A"},
				Target: json.FqnRefDump{Model: "B"},
				Kind:   "bidirectional",
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, "A <-> B") {
		t.Error("Expected bidirectional relation")
	}
}

func TestPrint_Tags(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"MySystem": {
				ID:    "MySystem",
				Kind:  "system",
				Title: "My System",
				Tags:  []string{"critical", "legacy"},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `tags ["critical", "legacy"]`) {
		t.Error("Expected tags array")
	}
}

func TestPrint_Metadata(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"MySystem": {
				ID:       "MySystem",
				Kind:     "system",
				Title:    "My System",
				Metadata: map[string]string{"owner": "team-a"},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, "metadata {") {
		t.Error("Expected metadata block")
	}
	if !strings.Contains(result, `owner "team-a"`) {
		t.Error("Expected metadata entry")
	}
}

func TestPrint_Views(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"System1": {ID: "System1", Kind: "system", Title: "System One"},
		},
		Views: map[string]json.ViewDump{
			"landscape": {
				ID:    "landscape",
				Title: "System Landscape",
				Rules: []json.ViewRule{
					{Include: &json.ViewRuleExpr{Wildcard: true}},
				},
			},
		},
	}

	result := Print(model)

	if strings.Contains(result, "views {") {
		t.Error("Did not expect views block wrapper")
	}
	if !strings.Contains(result, "view landscape") {
		t.Error("Expected view definition")
	}
	if !strings.Contains(result, `title "System Landscape"`) {
		t.Error("Expected view title")
	}
	if !strings.Contains(result, "include *") {
		t.Error("Expected wildcard include")
	}
}

func TestPrint_Requirement(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			Requirements: []json.RequirementDump{
				{
					ID:          "REQ001",
					Title:       "Must be fast",
					Type:        "functional",
					Description: "Response time < 100ms",
					Priority:    "high",
				},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `requirement REQ001 functional "Must be fast"`) {
		t.Error("Expected requirement definition")
	}
	if !strings.Contains(result, `description "Response time < 100ms"`) {
		t.Error("Expected requirement description")
	}
	if !strings.Contains(result, `priority "high"`) {
		t.Error("Expected requirement priority")
	}
}

func TestPrint_ADR(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			ADRs: []json.ADRDump{
				{
					ID:       "ADR001",
					Title:    "Use Go",
					Status:   "accepted",
					Decision: "We will use Go",
				},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `adr ADR001 "Use Go"`) {
		t.Error("Expected ADR definition")
	}
	if !strings.Contains(result, `status "accepted"`) {
		t.Error("Expected ADR status")
	}
	if !strings.Contains(result, `decision "We will use Go"`) {
		t.Error("Expected ADR decision")
	}
}

func TestPrint_Scenario(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			Scenarios: []json.ScenarioDump{
				{
					ID:    "SC001",
					Title: "User Login",
					Steps: []json.StepDump{
						{Description: "User enters credentials", From: "User", To: "LoginPage"},
					},
				},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `scenario SC001 "User Login"`) {
		t.Error("Expected scenario definition")
	}
	if !strings.Contains(result, `step "User enters credentials" from User to LoginPage`) {
		t.Error("Expected step with from/to")
	}
}

func TestPrint_EscapeStrings(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"MySystem": {
				ID:          "MySystem",
				Kind:        "system",
				Title:       `System with "quotes"`,
				Description: "Line1\nLine2",
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `"System with \"quotes\""`) {
		t.Error("Expected escaped quotes in title")
	}
	if !strings.Contains(result, `"Line1\nLine2"`) {
		t.Error("Expected escaped newline in description")
	}
}

func TestPrint_Policy(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			Policies: []json.PolicyDump{
				{
					ID:          "POL001",
					Title:       "Encryption Policy",
					Description: "All data must be encrypted",
					Category:    "security",
					Enforcement: "mandatory",
				},
			},
		},
	}

	result := Print(model)

	// Policy format is: policy ID category "Title"
	if !strings.Contains(result, `policy POL001 security "Encryption Policy"`) {
		t.Errorf("Expected policy definition, got: %s", result)
	}
	if !strings.Contains(result, `description "All data must be encrypted"`) {
		t.Error("Expected policy description")
	}
	if !strings.Contains(result, `enforcement "mandatory"`) {
		t.Error("Expected policy enforcement")
	}
}

func TestPrint_Flow(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			Flows: []json.FlowDump{
				{
					ID:    "FL001",
					Title: "Order Flow",
					Steps: []json.StepDump{
						{Description: "Customer places order", From: "Customer", To: "OrderService"},
						{Description: "Order is validated", From: "OrderService", To: "Validator"},
					},
				},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `flow FL001 "Order Flow"`) {
		t.Error("Expected flow definition")
	}
	if !strings.Contains(result, `step "Customer places order" from Customer to OrderService`) {
		t.Error("Expected first step")
	}
	if !strings.Contains(result, `step "Order is validated" from OrderService to Validator`) {
		t.Error("Expected second step")
	}
}

func TestPrint_PrintFlat(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"System1": {
				ID:    "System1",
				Kind:  "system",
				Title: "System One",
			},
			"System1.Container1": {
				ID:     "System1.Container1",
				Kind:   "container",
				Title:  "Container One",
				Parent: "System1",
			},
		},
	}

	printer := NewModelPrinter()
	result := printer.PrintFlat(model)

	// PrintFlat should produce non-empty output
	if result == "" {
		t.Error("Expected non-empty output from PrintFlat")
	}
	// Should contain some element definition
	if !strings.Contains(result, "system") || !strings.Contains(result, "container") {
		t.Errorf("Expected element kinds in output, got: %s", result)
	}
}

// Benchmark
func BenchmarkPrint_LargeModel(b *testing.B) {
	// Create a model with 50 elements
	elements := make(map[string]json.ElementDump)
	for i := 0; i < 50; i++ {
		id := "System" + string(rune('A'+i%26))
		elements[id] = json.ElementDump{
			ID:          id,
			Kind:        "system",
			Title:       "System " + id,
			Description: "Description for " + id,
			Technology:  "Go",
		}
	}

	model := &json.SrujaModelDump{
		Elements: elements,
	}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_ = Print(model)
	}
}
