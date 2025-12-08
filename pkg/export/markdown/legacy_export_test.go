package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExportLegacy_BasicArchitecture(t *testing.T) {
	exporter := NewExporter()

	// Create a simple architecture
	arch := &language.Architecture{
		Name:        "Test System",
		Description: stringPtr("Test description"),
		Systems: []*language.System{
			{
				ID:          "TestSys",
				Label:       "Test System",
				Description: stringPtr("A test system"),
			},
		},
		Persons: []*language.Person{
			{
				ID:          "User",
				Label:       "User",
				Description: stringPtr("A test user"),
			},
		},
	}

	config := MermaidConfig{
		Direction: "TB",
		Theme:     "default",
	}

	result, err := exporter.exportLegacy(arch, config)
	if err != nil {
		t.Fatalf("exportLegacy failed: %v", err)
	}

	// Verify basic content is present
	if !strings.Contains(result, "# Test System") {
		t.Error("Expected architecture name as heading")
	}
	if !strings.Contains(result, "Test description") {
		t.Error("Expected architecture description")
	}
	if !strings.Contains(result, "## Systems") {
		t.Error("Expected Systems section")
	}
	if !strings.Contains(result, "## Persons") {
		t.Error("Expected Persons section")
	}
	if !strings.Contains(result, "## Architecture Overview") {
		t.Error("Expected Architecture Overview section")
	}
}

func TestExportLegacy_WithAllSections(t *testing.T) {
	exporter := NewExporter()

	status := "Accepted"
	context := "We need this"
	decision := "Do this way"
	consequences := "This will happen"

	arch := &language.Architecture{
		Name: "Complete System",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System One",
			},
		},
		Requirements: []*language.Requirement{
			{
				ID:          "REQ-1",
				Type:        str("functional"),
				Description: str("Must do something"),
			},
		},
		ADRs: []*language.ADR{
			{
				ID:    "ADR-1",
				Title: stringPtr("Important Decision"),
				Body: &language.ADRBody{
					Status:       &status,
					Context:      &context,
					Decision:     &decision,
					Consequences: &consequences,
				},
			},
		},
		Policies: []*language.Policy{
			{
				ID:          "POL-1",
				Description: "Security policy",
			},
		},
		Constraints: []*language.ConstraintEntry{
			{
				Key:   "Budget",
				Value: "$1M",
			},
		},
		Conventions: []*language.ConventionEntry{
			{
				Key:   "Naming",
				Value: "CamelCase",
			},
		},
	}

	config := MermaidConfig{}
	result, err := exporter.exportLegacy(arch, config)
	if err != nil {
		t.Fatalf("exportLegacy failed: %v", err)
	}

	// Check all sections are present
	expectedSections := []string{
		"## Requirements",
		"## Architecture Decision Records (ADRs)",
		"## Policies",
		"## Constraints",
		"## Conventions",
		"## Table of Contents",
	}

	for _, section := range expectedSections {
		if !strings.Contains(result, section) {
			t.Errorf("Expected section %q not found", section)
		}
	}

	// Check specific content
	if !strings.Contains(result, "REQ-1") {
		t.Error("Expected requirement ID")
	}
	if !strings.Contains(result, "ADR-1") {
		t.Error("Expected ADR ID")
	}
	if !strings.Contains(result, "Accepted") {
		t.Error("Expected ADR status")
	}
}

func TestWriteComponent(t *testing.T) {
	exporter := NewExporter()
	var sb strings.Builder

	comp := &language.Component{
		ID:          "TestComponent",
		Label:       "Test Component",
		Description: str("A test component"),
	}

	exporter.writeComponent(&sb, comp)
	result := sb.String()

	if !strings.Contains(result, "TestComponent") {
		t.Error("Expected component ID in output")
	}
	if !strings.Contains(result, "Test Component") {
		t.Error("Expected component label in output")
	}
	if !strings.Contains(result, "A test component") {
		t.Error("Expected component description in output")
	}
}

func TestWriteDataStore(t *testing.T) {
	exporter := NewExporter()
	var sb strings.Builder

	ds := &language.DataStore{
		ID:          "TestDB",
		Label:       "Test Database",
		Description: str("A test database"),
	}

	exporter.writeDataStore(&sb, ds)
	result := sb.String()

	if !strings.Contains(result, "TestDB") {
		t.Error("Expected datastore ID in output")
	}
	if !strings.Contains(result, "Test Database") {
		t.Error("Expected datastore label in output")
	}
}

func TestWriteQueue(t *testing.T) {
	exporter := NewExporter()
	var sb strings.Builder

	q := &language.Queue{
		ID:          "TestQueue",
		Label:       "Test Queue",
		Description: str("A test queue"),
	}

	exporter.writeQueue(&sb, q)
	result := sb.String()

	if !strings.Contains(result, "TestQueue") {
		t.Error("Expected queue ID in output")
	}
	if !strings.Contains(result, "Test Queue") {
		t.Error("Expected queue label in output")
	}
}

func TestExportLegacy_EmptyArchitecture(t *testing.T) {
	exporter := NewExporter()

	arch := &language.Architecture{
		Name: "Empty System",
	}

	config := MermaidConfig{}
	result, err := exporter.exportLegacy(arch, config)
	if err != nil {
		t.Fatalf("exportLegacy failed: %v", err)
	}

	// Should still have basic structure
	if !strings.Contains(result, "# Empty System") {
		t.Error("Expected architecture name")
	}
	if !strings.Contains(result, "## Table of Contents") {
		t.Error("Expected TOC even for empty architecture")
	}
}

// Helper function - renamed to avoid conflict
func str(s string) *string {
	return &s
}
