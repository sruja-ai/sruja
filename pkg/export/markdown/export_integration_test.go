package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Integration test for full markdown export workflow
func TestMarkdownExport_Integration_FullArchitecture(t *testing.T) {
	// Create a comprehensive architecture with all element types
	desc := "Test system description"
	adrTitle := "Use Microservices"
	const statusAccepted = "Accepted"
	adrStatus := statusAccepted
	adrContext := "Need scalability"
	adrDecision := "Use microservices architecture"
	adrConsequences := "Better scalability, more complexity"
	category := "security"
	enforcement := "automatic"

	arch := &language.Architecture{
		Name:        "E-Commerce Platform",
		Description: &desc,
		Systems: []*language.System{
			{
				ID:          "Frontend",
				Label:       "Frontend System",
				Description: stringPtr("Web frontend"),
				Containers: []*language.Container{
					{
						ID:          "WebUI",
						Label:       "Web UI",
						Description: stringPtr("React application"),
						Components: []*language.Component{
							{
								ID:          "HomePage",
								Label:       "Home Page",
								Description: stringPtr("Landing page"),
							},
						},
					},
				},
				DataStores: []*language.DataStore{
					{
						ID:          "SessionDB",
						Label:       "Session Store",
						Description: stringPtr("Redis session storage"),
					},
				},
				Queues: []*language.Queue{
					{
						ID:          "EventQueue",
						Label:       "Event Queue",
						Description: stringPtr("RabbitMQ queue"),
					},
				},
			},
		},
		Persons: []*language.Person{
			{
				ID:          "Customer",
				Label:       "Customer",
				Description: stringPtr("End user"),
			},
		},
		Requirements: []*language.Requirement{
			{
				ID:          "REQ-1",
				Type:        stringPtr("functional"),
				Description: stringPtr("System must handle 10k concurrent users"),
			},
			{
				ID:          "REQ-2",
				Type:        stringPtr("security"),
				Description: stringPtr("All data must be encrypted"),
			},
		},
		ADRs: []*language.ADR{
			{
				ID:    "ADR-001",
				Title: &adrTitle,
				Body: &language.ADRBody{
					Status:       &adrStatus,
					Context:      &adrContext,
					Decision:     &adrDecision,
					Consequences: &adrConsequences,
				},
			},
		},
		Policies: []*language.Policy{
			{
				ID:          "POL-1",
				Description: "All API calls must be authenticated",
				Category:    &category,
				Enforcement: &enforcement,
			},
		},
		Constraints: []*language.ConstraintEntry{
			{Key: "Budget", Value: "$500K"},
		},
		Conventions: []*language.ConventionEntry{
			{Key: "Naming", Value: "camelCase for variables"},
		},
		Relations: []*language.Relation{
			{
				From:  language.QualifiedIdent{Parts: []string{"Customer"}},
				To:    language.QualifiedIdent{Parts: []string{"Frontend"}},
				Verb:  stringPtr("uses"),
				Label: stringPtr("browses"),
			},
		},
	}

	// Create exporter and export
	exporter := NewExporter()
	result, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Verify all sections are present
	expectedSections := []string{
		"# E-Commerce Platform",
		"Test system description",
		"## Table of Contents",
		"## Architecture Overview",
		"## Systems",
		"## Persons",
		"## Requirements",
		"## Architecture Decision Records",
		"## Policies",
		"## Constraints",
		"## Conventions",
		"## Relations",
		"## Quality Attributes",
		"## Security",
		"## Glossary",
	}

	for _, section := range expectedSections {
		if !strings.Contains(result, section) {
			t.Errorf("Expected section %q not found in output", section)
		}
	}

	// Verify specific element details
	expectedContent := []string{
		"Frontend System",
		"Web UI",
		"Home Page",
		"Session Store",
		"Event Queue",
		"Customer",
		"REQ-1",
		"REQ-2",
		"ADR-001",
		"Use Microservices",
		"Accepted",
		"POL-1",
		"authenticated",
		"Budget",
		"$500K",
		"camelCase",
		"Customer → Frontend",
	}

	for _, content := range expectedContent {
		if !strings.Contains(result, content) {
			t.Errorf("Expected content %q not found in output", content)
		}
	}

	// Verify Mermaid diagrams are included
	if !strings.Contains(result, "```mermaid") {
		t.Error("Expected Mermaid diagram code blocks")
	}
}

func TestMarkdownExport_Integration_MinimalOptions(t *testing.T) {
	arch := &language.Architecture{
		Name: "Minimal System",
		Systems: []*language.System{
			{
				ID:    "API",
				Label: "API System",
			},
		},
		Requirements: []*language.Requirement{
			{
				ID:          "REQ-1",
				Type:        stringPtr("functional"),
				Description: stringPtr("Test requirement"),
			},
		},
	}

	// Create exporter with minimal options
	exporter := NewExporterWithOptions(MinimalExportOptions())
	result, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Should include essential sections
	if !strings.Contains(result, "## Systems") {
		t.Error("Expected Systems section even with minimal options")
	}
	if !strings.Contains(result, "## Table of Contents") {
		t.Error("Expected TOC even with minimal options")
	}

	// Should NOT include optional sections
	if strings.Contains(result, "## Requirements") {
		t.Error("Did not expect Requirements section with minimal options")
	}
}

func TestMarkdownExport_Integration_CustomMermaidConfig(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test System",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System One",
			},
		},
	}

	// Create exporter with custom Mermaid config
	opts := DefaultExportOptions()
	opts.MermaidConfig = MermaidConfig{
		Direction: "LR",
		Theme:     "dark",
		Layout:    "elk",
	}

	exporter := NewExporterWithOptions(opts)
	result, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Result should contain Mermaid diagrams
	if !strings.Contains(result, "```mermaid") {
		t.Error("Expected Mermaid diagrams in output")
	}
}

func TestMarkdownExport_Integration_NilDescription(t *testing.T) {
	arch := &language.Architecture{
		Name:        "System Without Description",
		Description: nil,
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System One",
			},
		},
	}

	exporter := NewExporter()
	result, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Should still have proper structure
	if !strings.Contains(result, "# System Without Description") {
		t.Error("Expected architecture name header")
	}
}

func TestMarkdownExport_Integration_ImpliedRelationships(t *testing.T) {
	// Test that implied relationships are included in markdown export
	dsl := `
architecture "Test" {
    person User "User"
    system API "API Service" {
        container WebApp "Web Application"
    }
    
    User -> API.WebApp "Uses"
}`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if program.Architecture == nil {
		t.Fatal("Architecture is nil")
	}

	// Verify implied relationship was created
	if len(program.Architecture.Relations) != 2 {
		t.Errorf("Expected 2 relationships (1 explicit + 1 implied), got %d", len(program.Architecture.Relations))
	}

	// Export to markdown
	exporter := NewExporter()
	result, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Verify both relationships appear in markdown
	if !strings.Contains(result, "User → API.WebApp") {
		t.Error("Expected explicit relationship 'User → API.WebApp' in markdown")
	}
	if !strings.Contains(result, "User → API") {
		t.Error("Expected implied relationship 'User → API' in markdown")
	}
}

func TestMarkdownExport_Integration_ViewsBlock(t *testing.T) {
	// Test that views block is used in markdown export
	dsl := `
architecture "Test" {
    system Shop "Shop" {
        container WebApp "Web Application"
        container API "API Gateway"
    }
    
    views {
        container Shop "Container View" {
            include *
        }
    }
}`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse: %v", err)
	}

	if program.Architecture == nil {
		t.Fatal("Architecture is nil")
	}

	// Verify views block was parsed
	if program.Architecture.Views == nil {
		t.Fatal("Expected views block to be parsed")
	}
	if len(program.Architecture.Views.Views) != 1 {
		t.Errorf("Expected 1 view, got %d", len(program.Architecture.Views.Views))
	}

	// Export to markdown
	exporter := NewExporter()
	result, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Verify custom view appears in markdown (should use custom view name)
	view := program.Architecture.Views.Views[0]
	if view.Name != "" {
		// If view has a name, it should appear in the markdown
		if !strings.Contains(result, strings.Trim(view.Name, "\"")) {
			t.Errorf("Expected view name '%s' in markdown", view.Name)
		}
	}
}

func stringPtr(s string) *string {
	return &s
}
