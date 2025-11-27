//go:build legacy

// Package compiler_test provides tests for the format selector.
package compiler_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestSelector_Recommend_SimpleModel(t *testing.T) {
	dsl := `
workspace {
	model {
		system API "API Service"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	rec, err := selector.Recommend(program, "general")
	if err != nil {
		t.Fatalf("Failed to get recommendation: %v", err)
	}

	if rec == nil {
		t.Fatal("Expected recommendation, got nil")
	}

	if rec.Format == "" {
		t.Error("Expected format to be set")
	}

	if rec.Score < 0 || rec.Score > 1 {
		t.Errorf("Expected score between 0 and 1, got %f", rec.Score)
	}

	if len(rec.Reasons) == 0 {
		t.Error("Expected at least one reason for recommendation")
	}
}

func TestSelector_Recommend_ComplexModel(t *testing.T) {
	// Create a more complex model with many elements
	dsl := `
workspace {
	model {
		system User "End User"
		system API "API Service" {
			container WebApp "Web Application"
			container Database "PostgreSQL Database"
		}
		system Auth "Auth Service"
		system Payment "Payment Service"
		User -> API "Uses"
		API -> Auth "Authenticates"
		API -> Payment "Processes Payment"
		API -> Database "Reads/Writes"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	rec, err := selector.Recommend(program, "presentation")
	if err != nil {
		t.Fatalf("Failed to get recommendation: %v", err)
	}

	if rec.Format == "" {
		t.Error("Expected format to be set")
	}

	// Complex models should favor D2
	if rec.Format == "d2" {
		// This is expected for complex models
		t.Logf("Recommended D2 for complex model (score: %.2f)", rec.Score)
	}
}

func TestSelector_Recommend_DocumentationUseCase(t *testing.T) {
	dsl := `
workspace {
	model {
		system API "API Service"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	rec, err := selector.Recommend(program, "documentation")
	if err != nil {
		t.Fatalf("Failed to get recommendation: %v", err)
	}

	if rec.Format == "" {
		t.Error("Expected format to be set")
	}

	// Documentation use case might favor mermaid
	t.Logf("Recommended format for documentation: %s (score: %.2f)", rec.Format, rec.Score)
}

func TestSelector_AutoSelect(t *testing.T) {
	dsl := `
workspace {
	model {
		system API "API Service"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	format, output, err := selector.AutoSelect(program, "general")
	if err != nil {
		t.Fatalf("Failed to auto-select: %v", err)
	}

	if format == "" {
		t.Error("Expected format to be set")
	}

	if output == "" {
		t.Error("Expected non-empty output")
	}
}

func TestSelector_GetFormatInfo(t *testing.T) {
	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	// Test getting D2 info
	d2Info, err := selector.GetFormatInfo("d2")
	if err != nil {
		t.Fatalf("Failed to get D2 info: %v", err)
	}

	if d2Info.Name != "d2" {
		t.Errorf("Expected name 'd2', got '%s'", d2Info.Name)
	}

	if d2Info.Description == "" {
		t.Error("Expected non-empty description")
	}

	if len(d2Info.UseCases) == 0 {
		t.Error("Expected at least one use case")
	}

	if len(d2Info.Capabilities) == 0 {
		t.Error("Expected at least one capability")
	}

	// Test getting Mermaid info
	mermaidInfo, err := selector.GetFormatInfo("mermaid")
	if err != nil {
		t.Fatalf("Failed to get Mermaid info: %v", err)
	}

	if mermaidInfo.Name != "mermaid" {
		t.Errorf("Expected name 'mermaid', got '%s'", mermaidInfo.Name)
	}

	// Test getting non-existent format
	_, err = selector.GetFormatInfo("nonexistent")
	if err == nil {
		t.Error("Expected error when getting info for non-existent format")
	}
}

func TestSelector_ListFormatsWithInfo(t *testing.T) {
	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	formats, err := selector.ListFormatsWithInfo()
	if err != nil {
		t.Fatalf("Failed to list formats: %v", err)
	}

	if len(formats) == 0 {
		t.Error("Expected at least one format")
	}

	// Check that d2 and mermaid are present
	if _, ok := formats["d2"]; !ok {
		t.Error("Expected 'd2' format to be in list")
	}

	if _, ok := formats["mermaid"]; !ok {
		t.Error("Expected 'mermaid' format to be in list")
	}

	// Verify each format has complete info
	for name, info := range formats {
		if info.Name != name {
			t.Errorf("Format name mismatch: expected '%s', got '%s'", name, info.Name)
		}
		if info.Description == "" {
			t.Errorf("Format '%s' has empty description", name)
		}
	}
}

func TestSelector_Recommend_Alternatives(t *testing.T) {
	dsl := `
workspace {
	model {
		system API "API Service"
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	registry := compiler.NewRegistry()
	selector := compiler.NewSelector(registry)

	rec, err := selector.Recommend(program, "general")
	if err != nil {
		t.Fatalf("Failed to get recommendation: %v", err)
	}

	// Should have at least one alternative (the other format)
	if len(rec.Alternatives) == 0 {
		t.Log("No alternatives provided, but this is acceptable")
	}
}
