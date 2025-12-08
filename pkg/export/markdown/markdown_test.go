// pkg/export/markdown/markdown_test.go
// Package markdown_test provides tests for the Markdown exporter.
package markdown_test

import (
	"os"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestMarkdownExporter_BasicSystem(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
		Systems: []*language.System{
			{
				ID:    "API",
				Label: "API Service",
			},
		},
	}

	exporter := markdown.NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export: %v", err)
	}

	if !strings.Contains(output, "# Test System") {
		t.Errorf("Expected header 'Test System', got:\n%s", output)
	}

	if !strings.Contains(output, "## Systems") {
		t.Errorf("Expected Systems section, got:\n%s", output)
	}

	if !strings.Contains(output, "API Service") {
		t.Errorf("Expected system label, got:\n%s", output)
	}
}

func TestMarkdownExporter_WithMermaidDiagram(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
		Systems: []*language.System{
			{
				ID:    "API",
				Label: "API Service",
			},
		},
		Persons: []*language.Person{
			{
				ID:    "User",
				Label: "End User",
			},
		},
	}

	exporter := markdown.NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export: %v", err)
	}

	if !strings.Contains(output, "```mermaid") {
		t.Errorf("Expected Mermaid diagram, got:\n%s", output)
	}

	if !strings.Contains(output, "graph ") {
		t.Errorf("Expected graph diagram, got:\n%s", output)
	}
}

func TestMarkdownExporter_TOC(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
		Systems: []*language.System{
			{
				ID:    "API",
				Label: "API Service",
			},
		},
		Persons: []*language.Person{
			{
				ID:    "User",
				Label: "End User",
			},
		},
	}

	exporter := markdown.NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export: %v", err)
	}

	checks := []string{
		"## Table of Contents",
		"[Architecture Overview (C4 L1)](#architecture-overview-c4-l1)",
		"[Systems (C4 L2/L3)](#systems)",
	}
	for _, check := range checks {
		if !strings.Contains(output, check) {
			t.Errorf("Expected output to contain '%s'", check)
		}
	}
}

func TestMarkdownExporter_ScenarioSequenceDiagram(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
		Scenarios: []*language.Scenario{
			{
				ID:    "S1",
				Title: "User Login",
				Steps: []*language.ScenarioStep{
					{
						From: language.QualifiedIdent{Parts: []string{"User"}},
						To:   language.QualifiedIdent{Parts: []string{"API"}},
					},
					{
						From:        language.QualifiedIdent{Parts: []string{"API"}},
						To:          language.QualifiedIdent{Parts: []string{"DB"}},
						Description: stringPtr("Validate credentials"),
					},
				},
			},
		},
	}

	exporter := markdown.NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export: %v", err)
	}

	if !strings.Contains(output, "sequenceDiagram") {
		t.Errorf("Expected sequence diagram, got:\n%s", output)
	}

	if !strings.Contains(output, "User Login") {
		t.Errorf("Expected scenario title, got:\n%s", output)
	}
}

func stringPtr(s string) *string {
	return &s
}

// TestMarkdownExporter_ComplexExamples tests the exporter with real-world complex examples
func TestMarkdownExporter_ComplexExamples(t *testing.T) {
	// Test with c4_full.sruja - includes systems, containers, persons, relations, deployment
	t.Run("c4_full", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		content, err := os.ReadFile("../../../examples/c4_full.sruja")
		if err != nil {
			t.Fatalf("Failed to read example file: %v", err)
		}

		program, diags, err := parser.Parse("c4_full.sruja", string(content))
		if err != nil {
			t.Fatalf("Failed to parse example: %v", err)
		}
		if len(diags) > 0 {
			t.Fatalf("Parser returned diagnostics: %v", diags)
		}

		if program.Architecture == nil {
			t.Fatal("Expected architecture to be parsed")
		}

		exporter := markdown.NewExporter()
		output, err := exporter.Export(program.Architecture)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}

		// Verify key sections are present
		checks := []string{
			"# C4 Complete Example",
			"## Architecture Overview",
			"## Systems",
			"## Deployment",
			"```mermaid",
			"graph ",
		}

		for _, check := range checks {
			if !strings.Contains(output, check) {
				t.Errorf("Expected output to contain '%s', got:\n%s", check, output)
			}
		}

		// Verify Mermaid diagrams are valid (contain classDef)
		if !strings.Contains(output, "classDef") {
			t.Errorf("Expected Mermaid diagrams with classDef styling")
		}
	})

	// Test with full_features.sruja - includes requirements, ADRs
	t.Run("full_features", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		content, err := os.ReadFile("../../../examples/full_features.sruja")
		if err != nil {
			t.Fatalf("Failed to read example file: %v", err)
		}

		program, diags, err := parser.Parse("full_features.sruja", string(content))
		if err != nil {
			t.Fatalf("Failed to parse example: %v", err)
		}
		if len(diags) > 0 {
			t.Fatalf("Parser returned diagnostics: %v", diags)
		}

		if program.Architecture == nil {
			t.Fatal("Expected architecture to be parsed")
		}

		exporter := markdown.NewExporter()
		output, err := exporter.Export(program.Architecture)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}

		// Verify requirements and ADRs are exported
		checks := []string{
			"## Requirements",
			"## Architecture Decision Records",
		}

		for _, check := range checks {
			if !strings.Contains(output, check) {
				t.Errorf("Expected output to contain '%s', got:\n%s", check, output)
			}
		}
	})

	// Test with full_mvp.sruja - comprehensive e-commerce example
	t.Run("full_mvp", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		content, err := os.ReadFile("../../../examples/full_mvp.sruja")
		if err != nil {
			t.Fatalf("Failed to read example file: %v", err)
		}

		program, diags, err := parser.Parse("full_mvp.sruja", string(content))
		if err != nil {
			t.Fatalf("Failed to parse example: %v", err)
		}
		if len(diags) > 0 {
			t.Fatalf("Parser returned diagnostics: %v", diags)
		}

		if program.Architecture == nil {
			t.Fatal("Expected architecture to be parsed")
		}

		exporter := markdown.NewExporter()
		output, err := exporter.Export(program.Architecture)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}

		// Verify comprehensive output
		checks := []string{
			"## Systems",
			"## Persons",
			"```mermaid",
		}

		for _, check := range checks {
			if !strings.Contains(output, check) {
				t.Errorf("Expected output to contain '%s'", check)
			}
		}

		// Verify output is substantial (complex example should produce substantial output)
		if len(output) < 500 {
			t.Errorf("Expected substantial output for complex example, got %d bytes", len(output))
		}
	})

	// Test with ecommerce_platform.sruja - comprehensive real-world example with requirements, ADRs, scenarios
	t.Run("ecommerce_platform", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		content, err := os.ReadFile("../../../examples/ecommerce_platform.sruja")
		if err != nil {
			t.Fatalf("Failed to read example file: %v", err)
		}

		program, diags, err := parser.Parse("ecommerce_platform.sruja", string(content))
		if err != nil {
			t.Fatalf("Failed to parse example: %v", err)
		}
		if len(diags) > 0 {
			t.Fatalf("Parser returned diagnostics: %v", diags)
		}

		if program.Architecture == nil {
			t.Fatal("Expected architecture to be parsed")
		}

		exporter := markdown.NewExporter()
		output, err := exporter.Export(program.Architecture)
		if err != nil {
			t.Fatalf("Failed to export: %v", err)
		}

		// Verify comprehensive output
		checks := []string{
			"# E-Commerce Platform",
			"## Requirements",
			"## Architecture Decision Records",
			"## Flows",
		}

		for _, check := range checks {
			if !strings.Contains(output, check) {
				t.Errorf("Expected output to contain '%s'", check)
			}
		}

		// Verify substantial output (real-world example should be large)
		if len(output) < 1000 {
			t.Errorf("Expected substantial output for real-world example, got %d bytes", len(output))
		}

		// Verify requirements section is present (check for requirement IDs or descriptions)
		if !strings.Contains(output, "REQ") && !strings.Contains(output, "functional") && !strings.Contains(output, "constraint") {
			t.Errorf("Expected requirements section with requirement details")
		}

		// Verify ADRs section is present
		if !strings.Contains(output, "ADR") && !strings.Contains(output, "Microservices") {
			t.Errorf("Expected ADRs section with ADR details")
		}

		// Verify flows are present
		if !strings.Contains(output, "PaymentFlow") && !strings.Contains(output, "OrderFulfillmentFlow") {
			t.Errorf("Expected flows to be present")
		}
	})
}
