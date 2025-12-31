// pkg/dx/explainer_description_test.go
package dx

import (
	"strings"
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

func TestBuildDescription_DataStore(t *testing.T) {
	dsl := `
		Sys = system "System" {
			DB = database "Database"
		}
	`
	prog := parseDSL(t, dsl)

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
	dsl := `
		Sys = system "System" {
			Q = queue "Queue"
		}
	`
	prog := parseDSL(t, dsl)

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
	prog := &language.Program{
		Model: &language.Model{},
	}
	explainer := NewExplainer(prog)
	desc := explainer.buildDescription(nil)
	if !strings.Contains(desc, "architecture element") {
		t.Error("Default case should mention 'architecture element'")
	}
}

func TestBuildDescription_SystemWithDataStores(t *testing.T) {
	dsl := `
		Sys = system "System" {
			DB = database "Database"
		}
	`
	prog := parseDSL(t, dsl)

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
	dsl := `
		Sys = system "System" {
			Cont = container "Container" {
				Comp = component "Component"
			}
		}
	`
	prog := parseDSL(t, dsl)

	explainer := NewExplainer(prog)
	explanation, err := explainer.ExplainElement("Cont")
	if err != nil {
		t.Fatalf("ExplainElement failed: %v", err)
	}
	if !strings.Contains(explanation.Description, "component") {
		t.Error("Should mention components in container description")
	}
}
