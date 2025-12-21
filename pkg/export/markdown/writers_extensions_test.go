package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtr(s string) *string {
	return &s
}

func TestMarkdownExport_ScenariosAndFlows(t *testing.T) {
	// Manually construct program with scenarios and flows to bypass parser limitations
	// for specific fields like Description on Scenario which seemed to fail parsing

	desc := "Successful order flow"
	stepDesc := "Process Payment"
	scenario := &language.Scenario{
		ID:          "OrderProcessing",
		Title:       stringPtr("Order Processing"),
		Description: &desc,
		Steps: []*language.ScenarioStep{
			{
				From:        language.QualifiedIdent{Parts: []string{"OrderService"}},
				To:          language.QualifiedIdent{Parts: []string{"PaymentService"}},
				Description: &stepDesc,
				Tags:        []string{"fast", "secure"},
			},
		},
	}

	flowDesc := "Daily sync"
	flowStepDesc := "Sync Data"
	flow := &language.Flow{
		ID:          "DataSync",
		Title:       stringPtr("Data Sync"),
		Description: &flowDesc,
		Steps: []*language.ScenarioStep{
			{
				From:        language.QualifiedIdent{Parts: []string{"OrderService"}},
				To:          language.QualifiedIdent{Parts: []string{"PaymentService"}},
				Description: &flowStepDesc,
			},
		},
	}

	program := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{Scenario: scenario},
				{Flow: flow},
			},
		},
	}

	exporter := NewExporter(DefaultOptions())
	output := exporter.Export(program)

	// Check Scenarios
	if !strings.Contains(output, "## User Scenarios and Data Flows") {
		t.Error("Expected output to contain 'User Scenarios and Data Flows' section")
	}

	if !strings.Contains(output, "### User Scenarios") {
		t.Error("Expected output to contain 'User Scenarios' subsection")
	}

	if !strings.Contains(output, "#### Order Processing") {
		t.Error("Expected output to contain 'Order Processing' scenario")
	}

	if !strings.Contains(output, "Successful order flow") {
		t.Logf("Output: %s", output)
		t.Error("Expected output to contain scenario description")
	}

	// Check Sequence Diagram generation
	if !strings.Contains(output, "```mermaid") || !strings.Contains(output, "sequenceDiagram") {
		t.Error("Expected output to contain mermaid sequence diagram")
	}

	if !strings.Contains(output, "participant OrderService") {
		t.Error("Expected sequence diagram to contain participant OrderService")
	}

	if !strings.Contains(output, "OrderService->>PaymentService: Process Payment") {
		t.Error("Expected sequence diagram to contain interaction")
	}

	// Check Step details
	if !strings.Contains(output, "**Steps:**") {
		t.Error("Expected output to contain Steps list")
	}

	if !strings.Contains(output, "Tags: fast, secure") {
		t.Error("Expected output to contain step tags")
	}

	// Check Flows
	if !strings.Contains(output, "### Data Flows") {
		t.Error("Expected output to contain 'Data Flows' subsection")
	}

	if !strings.Contains(output, "#### Data Sync") {
		t.Error("Expected output to contain 'Data Sync' flow")
	}
}

func TestMarkdownExport_SequenceDiagram_Complex(t *testing.T) {
	dsl := `model {
		system A "System A"
		system B "System B"
		system C "System C"

		scenario "Complex Flow" {
			step A -> B
			step B -> C "Forward"
			step C -> B "Ack"
			step B -> A "Done"
		}
	}`

	program := parseDSL(t, dsl)
	exporter := NewExporter(DefaultOptions())
	output := exporter.Export(program)

	if !strings.Contains(output, "A->>B") { // No description
		t.Error("Expected A->>B interaction without description")
	}
	if !strings.Contains(output, "B->>C: Forward") {
		t.Error("Expected B->>C interaction with description")
	}
}
