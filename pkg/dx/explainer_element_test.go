// pkg/dx/explainer_element_test.go
package dx

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExplainElement_System(t *testing.T) {
	dsl := `
		API = system "API Service" {
			description "Main API"
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
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
	dsl := `
		API = system "API Service"
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	_, err = explainer.ExplainElement("NonExistent")
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
		t.Fatal("ExplainElement should error when no model")
	}
}

func TestExplainElement_Container(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container"
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys.Cont")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Sys.Cont" {
		t.Errorf("Expected ID 'Sys.Cont', got '%s'", explanation.ID)
	}
}

func TestExplainElement_Component(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				Comp = component "Component" {
					technology "Go"
				}
			}
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys.Cont.Comp")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Sys.Cont.Comp" {
		t.Errorf("Expected ID 'Sys.Cont.Comp', got '%s'", explanation.ID)
	}
	if !strings.Contains(explanation.Description, "Go") {
		t.Error("Description should mention technology")
	}
}

func TestExplainElement_Person(t *testing.T) {
	dsl := `
		User = person "End User"
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
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
	dsl := `
		Sys = system "System" {
			DB = database "Database"
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys.DB")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Sys.DB" {
		t.Errorf("Expected ID 'Sys.DB', got '%s'", explanation.ID)
	}
}

func TestFindElement_Queue(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Q = queue "Queue"
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys.Q")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Sys.Q" {
		t.Errorf("Expected ID 'Sys.Q', got '%s'", explanation.ID)
	}
}

func TestFindElement_ComponentInSystem(t *testing.T) {
	dsl := `
		Sys = system "System" {
			Comp = component "Component"
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Sys.Comp")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if explanation.ID != "Sys.Comp" {
		t.Errorf("Expected ID 'Sys.Comp', got '%s'", explanation.ID)
	}
}
