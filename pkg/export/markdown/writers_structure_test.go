package markdown

import (
	"strings"
	"testing"
)

func TestMarkdownExport_SystemsStructure(t *testing.T) {
	dsl := `model {
		system OrderService "Order Service" {
			description "Handles orders"
			
			container API "API Layer" {
				description "REST API"
				technology "Go"

				component Handler "Handler" {
					description "Request handler"
					technology "Gin"
				}
			}

			container DB "Database" {
				technology "PostgreSQL"
			}
		}
	}`

	program := parseDSL(t, dsl)
	exporter := NewExporter(DefaultOptions())
	output := exporter.Export(program)

	// Check System
	if !strings.Contains(output, "### Order Service") {
		t.Error("Expected output to contain 'Order Service' section")
	}

	// Check L2 Diagram
	if !strings.Contains(output, "#### Container Diagram (Level 2)") {
		t.Error("Expected output to contain Container Diagram")
	}

	// Check L3 Diagram
	if !strings.Contains(output, "#### Component Diagrams (Level 3)") {
		t.Error("Expected output to contain Component Diagrams section")
	}

	if !strings.Contains(output, "##### API Layer Component Diagram") {
		t.Error("Expected output to contain API Layer component diagram")
	}
}

func TestMarkdownExport_Relationships_Prioritization(t *testing.T) {
	dsl := `model {
		system A "System A"
		system B "System B"
		system C "System C"

		A -> B "Calls"
		B -> C "Forwards"
		C -> A "Reports"
	}`

	program := parseDSL(t, dsl)

	options := DefaultOptions()
	// options.TokenLimit = 500 // Removed to ensure output is generated
	options.Context = ContextAnalysis // Enable analysis context to ensure relationships are emphasized/included
	exporter := NewExporter(options)
	output := exporter.Export(program)

	if !strings.Contains(output, "## System Relationships and Data Flow") {
		t.Error("Expected output to contain Relationships section")
	}

	if !strings.Contains(output, "**A** → **B**: Calls") {
		t.Logf("Output: %s", output)
		t.Error("Expected A -> B relationship")
	}
}

func TestMarkdownExport_Relationships_WithTokenLimit(t *testing.T) {
	dsl := `model {
		system A "System A" {
			description "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
		}
		system B "System B"
		system C "System C"
		system D "System D"

		A -> B "Rel1"
		B -> C "Rel2"
		C -> D "Rel3"
		D -> A "Rel4"
	}`

	program := parseDSL(t, dsl)

	options := DefaultOptions()
	// Disable other sections to focus on relationships
	options.IncludeTOC = false
	options.IncludeOverview = false
	options.IncludeSystems = true // Enable systems so description uses up token budget
	options.IncludePersons = false
	options.IncludeRequirements = false
	options.IncludeADRs = false
	options.Context = ContextAnalysis // Enable relationships section

	// Set limit.
	// Logic: available = 2000/4 = 500.
	// Used (Header + System A Desc) > 500.
	// So relationships truncated.
	// Global limit = 2000 * 4 = 8000.
	// Total length < 8000.
	options.TokenLimit = 2000
	exporter := NewExporter(options)
	output := exporter.Export(program)

	if !strings.Contains(output, "more relationships") {
		t.Logf("Output: %s", output)
		t.Error("Expected output to contain truncation message '... and X more relationships'")
	}
}
