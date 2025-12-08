package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// TestMarkdownExport_BasicArchitecture tests basic architecture export
func TestMarkdownExport_BasicArchitecture(t *testing.T) {
	dsl := `architecture "Test System" {
		person Customer "Customer"
		system Backend "Backend System"
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	if !strings.Contains(output, "Test System") {
		t.Error("Expected architecture name in markdown output")
	}
}

// TestMarkdownExport_SystemsContainersComponents tests hierarchical export
func TestMarkdownExport_SystemsContainersComponents(t *testing.T) {
	dsl := `architecture "Hierarchy Test" {
		system Backend "Backend System" {
			container API "REST API" {
				component Auth "Authentication"
			}
		}
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	expectedStrings := []string{"Backend System", "REST API", "Authentication"}
	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("Expected %q in markdown output", expected)
		}
	}
}

// TestMarkdownExport_RequirementsAndADRs tests requirements and ADRs export
func TestMarkdownExport_RequirementsAndADRs(t *testing.T) {
	dsl := `architecture "Requirements Test" {
		requirement R1 functional "Support user authentication"
		requirement R2 performance "Handle 10k concurrent users"
		
		adr ADR1 "Use microservices" {
			status "Accepted"
			context "Need to scale independently"
			decision "Adopt microservices architecture"
			consequences "Increased operational complexity"
		}
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	expectedStrings := []string{"R1", "R2", "ADR1", "microservices"}
	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("Expected %q in markdown output", expected)
		}
	}
}

// TestMarkdownExport_Scenarios tests scenario export
func TestMarkdownExport_Scenarios(t *testing.T) {
	dsl := `architecture "Scenario Test" {
		person User "User"
		system App "Application"
		
		scenario Login "User Login" "User logs into the app" {
			User -> App "Opens app"
			App -> User "Shows login screen"
		}
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	if !strings.Contains(output, "Login") || !strings.Contains(output, "User Login") {
		t.Error("Expected scenario in markdown output")
	}
}

// TestMarkdownExport_Relations tests relation export
func TestMarkdownExport_Relations(t *testing.T) {
	dsl := `architecture "Relations Test" {
		person Customer "Customer"
		system Frontend "Frontend"
		system Backend "Backend"
		
		Customer -> Frontend "uses"
		Frontend -> Backend "calls" "HTTPS"
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	// Should have some representation of relations
	if output == "" {
		t.Error("Expected non-empty markdown output")
	}
}

// TestMarkdownExport_EmptyArchitecture tests empty architecture
func TestMarkdownExport_EmptyArchitecture(t *testing.T) {
	dsl := `architecture "Empty" {}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	if !strings.Contains(output, "Empty") {
		t.Error("Expected architecture name in markdown output")
	}
}

// TestMarkdownExport_CompleteExample tests a complete example
func TestMarkdownExport_CompleteExample(t *testing.T) {
	dsl := `architecture "E-Commerce Platform" {
		person Customer "Online Customer"
		
		system Frontend "Web Application" {
			container WebUI "React App" {
				technology "React"
				component Shop "Shopping Cart"
			}
		}
		
		system Backend "API Services" {
			container API "REST API" {
				technology "Go"
			}
			datastore DB "PostgreSQL"
		}
		
		Customer -> Frontend "browses products"
		Frontend -> Backend "fetches data" "REST API"
		
		requirement FR1 functional "Support online shopping"
		requirement NFR1 performance "99.9% uptime"
		
		adr ADR1 "Use React for frontend" {
			status "Accepted"
			decision "React provides best UX"
		}
		
		scenario Purchase "Product Purchase" {
			Customer -> Frontend "Adds item to cart"
			Frontend -> Backend "Creates order"
			Backend -> Customer "Confirms order"
		}
	}`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to markdown: %v", err)
	}

	// Check for key elements
	expectedStrings := []string{
		"E-Commerce Platform",
		"Customer",
		"Frontend",
		"Backend",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("Expected %q in markdown output", expected)
		}
	}

	// Output should be reasonably sized
	if len(output) < 50 {
		t.Error("Expected more substantial markdown output")
	}
}
