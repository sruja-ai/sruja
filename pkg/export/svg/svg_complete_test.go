// pkg/export/svg/svg_complete_test.go
package svg

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_FullArchitecture(t *testing.T) {
	arch := &language.Architecture{
		Name:        "FullArch",
		Description: stringPtr("A complete architecture example"),
		Persons: []*language.Person{
			{
				ID:          "User",
				Label:       "End User",
				Description: stringPtr("Application user"),
			},
		},
		Systems: []*language.System{
			{
				ID:          "API",
				Label:       "API Service",
				Description: stringPtr("Main API service"),
				Containers: []*language.Container{
					{
						ID:          "Web",
						Label:       "Web Server",
						Description: stringPtr("Web container"),
						Components: []*language.Component{
							{
								ID:         "Handler",
								Label:      "Request Handler",
								Technology: stringPtr("Go"),
							},
						},
					},
				},
				DataStores: []*language.DataStore{
					{
						ID:         "DB",
						Label:      "Database",
						Technology: stringPtr("PostgreSQL"),
					},
				},
				Queues: []*language.Queue{
					{
						ID:         "MQ",
						Label:      "Message Queue",
						Technology: stringPtr("RabbitMQ"),
					},
				},
				Requirements: []*language.Requirement{
					{
						ID:          "R1",
						Type:        stringPtr("performance"),
						Description: stringPtr("Response time < 100ms"),
					},
				},
				ADRs: []*language.ADR{
					{
						ID:    "ADR001",
						Title: stringPtr("Use REST API"),
						Body: &language.ADRBody{
							Status:   stringPtr("Accepted"),
							Decision: stringPtr("We will use REST for API communication"),
						},
					},
				},
			},
		},
		Requirements: []*language.Requirement{
			{
				ID:          "R0",
				Type:        stringPtr("functional"),
				Description: stringPtr("System must be scalable"),
			},
		},
		ADRs: []*language.ADR{
			{
				ID:    "ADR000",
				Title: stringPtr("Microservices Architecture"),
				Body: &language.ADRBody{
					Status: stringPtr("Accepted"),
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Verify SVG structure
	svgChecks := []string{
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>",
		"<svg",
		"xmlns=\"http://www.w3.org/2000/svg\"",
		"</svg>",
	}

	// Verify architecture elements (check for labels and view structure)
	elementChecks := []string{
		"FullArch Architecture",
		"End User",        // Person label
		"API Service",     // System label
		"Web Server",      // Container label
		"Request Handler", // Component label
		"Database",        // DataStore label
		"level1",          // Level 1 view
		"view-API",        // Level 2 view for system
		"view-Web",        // Level 3 view for container
	}

	// Verify documentation (check for data store IDs and view structure)
	docChecks := []string{
		"Architecture Overview",
		"All Requirements",
		"All Architectural Decision Records",
		"data-content-id",   // Interactive elements
		"view-requirements", // Requirements view
		"view-adrs",         // ADRs view
	}

	// Verify interactivity
	interactiveChecks := []string{
		"<script",
		"loadContent",
		"switchLevel",
		"data-content-id",
		"interactive",
		"Level 1: System Context",
		"Level 2: Containers",
		"Level 3: Components",
	}

	allChecks := append(append(append(svgChecks, elementChecks...), docChecks...), interactiveChecks...)

	for _, check := range allChecks {
		if !strings.Contains(output, check) {
			t.Errorf("Expected output to contain %q, but it didn't", check)
		}
	}
}

func TestExport_QuickAccessButtons(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Requirements: []*language.Requirement{
			{ID: "R1", Description: stringPtr("Req 1")},
		},
		ADRs: []*language.ADR{
			{ID: "ADR1", Title: stringPtr("Decision 1")},
		},
		Systems: []*language.System{
			{
				ID: "Sys1",
				Containers: []*language.Container{
					{
						ID: "Cont1",
						Components: []*language.Component{
							{
								ID:         "Comp1",
								Technology: stringPtr("Go"),
							},
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"All Requirements",
		"All ADRs",
		"Technology Stack",
		"btnQuickReq",
		"btnQuickADR",
		"btnQuickTech",
		"requirements_summary",
		"adrs_summary",
		"technology_summary",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain %q, but it didn't", exp)
		}
	}
}

func TestExport_NoQuickAccessButtons_WhenEmpty(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID: "Sys1",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	// Should not include quick access buttons when there's no content
	if strings.Contains(output, "btnQuickReq") {
		t.Error("Should not include requirements button when no requirements exist")
	}
	if strings.Contains(output, "btnQuickADR") {
		t.Error("Should not include ADRs button when no ADRs exist")
	}
	if strings.Contains(output, "btnQuickTech") {
		t.Error("Should not include technology button when no technology is specified")
	}
}

func TestExport_JavaScript_Interactivity(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
		Systems: []*language.System{
			{
				ID:    "Sys1",
				Label: "System 1",
			},
		},
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	jsChecks := []string{
		"<script type=\"text/ecmascript\">",
		"<![CDATA[",
		"function loadContent",
		"function switchLevel",
		"function selectElement",
		"function clearSelection",
		"function init",
		"document.addEventListener",
		"DOMContentLoaded",
	}

	for _, check := range jsChecks {
		if !strings.Contains(output, check) {
			t.Errorf("Expected JavaScript to contain %q, but it didn't", check)
		}
	}
}

func TestExport_Styles(t *testing.T) {
	arch := &language.Architecture{
		Name: "TestArch",
	}

	exporter := NewExporter()
	output, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	styleChecks := []string{
		"<style type=\"text/css\">",
		".title",
		".subtitle",
		".system-box",
		".container-box",
		".component-box",
		".person-box",
		".db-box",
		".queue-box",
		".interactive",
		".button",
		".selected",
	}

	for _, check := range styleChecks {
		if !strings.Contains(output, check) {
			t.Errorf("Expected styles to contain %q, but it didn't", check)
		}
	}
}

func stringPtr(s string) *string {
	return &s
}
