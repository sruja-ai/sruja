// pkg/import/json/json_test.go
package json

import (
	"testing"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConverter_ToArchitecture_Basic(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "API",
					"label": "API Service",
					"containers": [
						{
							"id": "WebApp",
							"label": "Web Application"
						}
					]
				}
			],
			"persons": [
				{
					"id": "User",
					"label": "End User"
				}
			],
			"relations": [
				{
					"from": "User",
					"to": "API",
					"verb": "uses"
				}
			]
		},
		"navigation": {
			"levels": ["level1", "level2", "level3"]
		}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert JSON to AST: %v", err)
	}

	if arch.Name != "Test System" {
		t.Errorf("Expected architecture name 'Test System', got '%s'", arch.Name)
	}

	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}

	if len(arch.Persons) != 1 {
		t.Errorf("Expected 1 person, got %d", len(arch.Persons))
	}

	if len(arch.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(arch.Relations))
	}

	if arch.Systems[0].ID != "API" {
		t.Errorf("Expected system ID 'API', got '%s'", arch.Systems[0].ID)
	}

	if len(arch.Systems[0].Containers) != 1 {
		t.Errorf("Expected 1 container in system, got %d", len(arch.Systems[0].Containers))
	}

	if arch.Systems[0].Containers[0].ID != "WebApp" {
		t.Errorf("Expected container ID 'WebApp', got '%s'", arch.Systems[0].Containers[0].ID)
	}
}

func TestConverter_RoundTrip_Basic(t *testing.T) {
	// Create a simple architecture
	arch := &language.Architecture{
		Name: "Test System",
	}
	arch.Items = append(arch.Items,
		language.ArchitectureItem{
			Person: &language.Person{
				ID:    "User",
				Label: "End User",
			},
		},
		language.ArchitectureItem{
			System: &language.System{
				ID:    "API",
				Label: "API Service",
			},
		},
	)
	arch.PostProcess()

	// Export to JSON
	exporter := jsonexport.NewExporter()
	jsonStr, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export to JSON: %v", err)
	}

	// Convert back to AST
	converter := NewConverter()
	arch2, err := converter.ToArchitecture([]byte(jsonStr))
	if err != nil {
		t.Fatalf("Failed to convert JSON back to AST: %v", err)
	}

	// Verify round-trip
	if arch2.Name != arch.Name {
		t.Errorf("Round-trip failed: expected name '%s', got '%s'", arch.Name, arch2.Name)
	}

	if len(arch2.Systems) != len(arch.Systems) {
		t.Errorf("Round-trip failed: expected %d systems, got %d", len(arch.Systems), len(arch2.Systems))
	}

	if len(arch2.Persons) != len(arch.Persons) {
		t.Errorf("Round-trip failed: expected %d persons, got %d", len(arch.Persons), len(arch2.Persons))
	}
}

func TestConverter_ToDSL_SingleFile(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "API",
					"label": "API Service"
				}
			]
		},
		"navigation": {
			"levels": ["level1", "level2", "level3"]
		}
	}`

	converter := NewConverter()
	files, err := converter.ToDSL([]byte(jsonData), OutputFormatSingleFile)
	if err != nil {
		t.Fatalf("Failed to convert JSON to DSL: %v", err)
	}

	if len(files) != 1 {
		t.Errorf("Expected 1 file, got %d", len(files))
	}

	if files[0].Path != "Test_System.sruja" {
		t.Errorf("Expected file path 'Test_System.sruja', got '%s'", files[0].Path)
	}

	if files[0].Content == "" {
		t.Error("Expected non-empty file content")
	}
}

func TestConverter_ToDSL_MultipleFiles(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "API",
					"label": "API Service"
				}
			],
			"requirements": [
				{
					"id": "REQ001",
					"title": "Must handle 10k users"
				}
			],
			"adrs": [
				{
					"id": "ADR001",
					"title": "Use microservices"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	files, err := converter.ToDSL([]byte(jsonData), OutputFormatMultipleFiles)
	if err != nil {
		t.Fatalf("Failed to convert JSON to DSL: %v", err)
	}

	// Should have at least architecture file
	if len(files) < 1 {
		t.Fatalf("Expected at least 1 file, got %d", len(files))
	}

	// Check for architecture file
	foundArch := false
	for _, f := range files {
		if f.Path == "architecture.sruja" {
			foundArch = true
			if f.Content == "" {
				t.Error("Expected non-empty architecture file content")
			}
		}
	}
	if !foundArch {
		t.Error("Expected architecture.sruja file")
	}
}

func TestConverter_ToArchitecture_EmptyJSON(t *testing.T) {
	converter := NewConverter()
	_, err := converter.ToArchitecture([]byte(""))
	if err == nil {
		t.Error("Expected error for empty JSON data")
	}
	if err != nil && err.Error() != "empty JSON data" {
		t.Errorf("Expected 'empty JSON data' error, got: %v", err)
	}
}

func TestConverter_ToArchitecture_InvalidJSON(t *testing.T) {
	converter := NewConverter()
	_, err := converter.ToArchitecture([]byte("{ invalid json }"))
	if err == nil {
		t.Error("Expected error for invalid JSON")
	}
	if err != nil && err.Error() == "" {
		t.Error("Expected non-empty error message")
	}
}

func TestConverter_ToArchitecture_MissingName(t *testing.T) {
	jsonData := `{
		"metadata": {
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {},
		"navigation": {}
	}`

	converter := NewConverter()
	_, err := converter.ToArchitecture([]byte(jsonData))
	if err == nil {
		t.Error("Expected error for missing metadata.name")
	}
	if err != nil && err.Error() == "" {
		t.Error("Expected non-empty error message")
	}
}

func TestConverter_ToArchitecture_NullFields(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "API",
					"label": "",
					"description": null,
					"containers": []
				}
			],
			"persons": [
				{
					"id": "User",
					"label": "",
					"description": null
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert JSON with null fields: %v", err)
	}

	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}

	if arch.Systems[0].ID != "API" {
		t.Errorf("Expected system ID 'API', got '%s'", arch.Systems[0].ID)
	}

	if arch.Systems[0].Description != nil {
		t.Error("Expected nil description")
	}
}

func TestConverter_ToArchitecture_MissingIDs(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "",
					"label": "API Service"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert JSON with missing IDs: %v", err)
	}

	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}

	// ID should fallback to label
	if arch.Systems[0].ID != "API Service" {
		t.Errorf("Expected system ID to fallback to label 'API Service', got '%s'", arch.Systems[0].ID)
	}
}

func TestConverter_ToArchitecture_ComplexSystem(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Complex System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "ECommerce",
					"label": "E-Commerce Platform",
					"description": "Main e-commerce system",
					"containers": [
						{
							"id": "WebApp",
							"label": "Web Application",
							"components": [
								{
									"id": "ProductCatalog",
									"label": "Product Catalog"
								}
							]
						},
						{
							"id": "API",
							"label": "REST API"
						}
					],
					"datastores": [
						{
							"id": "Database",
							"label": "PostgreSQL Database"
						}
					],
					"queues": [
						{
							"id": "EventQueue",
							"label": "Event Queue"
						}
					],
					"relations": [
						{
							"from": "WebApp",
							"to": "API",
							"verb": "calls"
						}
					]
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert complex system: %v", err)
	}

	if len(arch.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(arch.Systems))
	}

	sys := arch.Systems[0]
	if len(sys.Containers) != 2 {
		t.Errorf("Expected 2 containers, got %d", len(sys.Containers))
	}

	if len(sys.Containers[0].Components) != 1 {
		t.Errorf("Expected 1 component in WebApp, got %d", len(sys.Containers[0].Components))
	}

	if len(sys.DataStores) != 1 {
		t.Errorf("Expected 1 datastore, got %d", len(sys.DataStores))
	}

	if len(sys.Queues) != 1 {
		t.Errorf("Expected 1 queue, got %d", len(sys.Queues))
	}

	if len(sys.Relations) != 1 {
		t.Errorf("Expected 1 relation, got %d", len(sys.Relations))
	}
}

func TestConverter_RoundTrip_Complex(t *testing.T) {
	// Create a complex architecture
	arch := &language.Architecture{
		Name: "Complex System",
	}

	// Add system with containers
	sys := &language.System{
		ID:    "ECommerce",
		Label: "E-Commerce Platform",
	}
	sys.Items = append(sys.Items,
		language.SystemItem{
			Container: &language.Container{
				ID:    "WebApp",
				Label: "Web Application",
			},
		},
		language.SystemItem{
			DataStore: &language.DataStore{
				ID:    "Database",
				Label: "PostgreSQL Database",
			},
		},
	)
	arch.Items = append(arch.Items, language.ArchitectureItem{System: sys})

	// Add person
	arch.Items = append(arch.Items,
		language.ArchitectureItem{
			Person: &language.Person{
				ID:    "User",
				Label: "End User",
			},
		},
		language.ArchitectureItem{
			Relation: &language.Relation{
				From:  language.QualifiedIdent{Parts: []string{"User"}},
				Arrow: "->",
				To:    language.QualifiedIdent{Parts: []string{"ECommerce"}},
				Verb:  stringPtr("uses"),
			},
		},
	)

	arch.PostProcess()

	// Export to JSON
	exporter := jsonexport.NewExporter()
	jsonStr, err := exporter.Export(arch)
	if err != nil {
		t.Fatalf("Failed to export to JSON: %v", err)
	}

	// Convert back to AST
	converter := NewConverter()
	arch2, err := converter.ToArchitecture([]byte(jsonStr))
	if err != nil {
		t.Fatalf("Failed to convert JSON back to AST: %v", err)
	}

	// Verify round-trip
	if arch2.Name != arch.Name {
		t.Errorf("Round-trip failed: expected name '%s', got '%s'", arch.Name, arch2.Name)
	}

	if len(arch2.Systems) != len(arch.Systems) {
		t.Errorf("Round-trip failed: expected %d systems, got %d", len(arch.Systems), len(arch2.Systems))
	}

	if len(arch2.Persons) != len(arch.Persons) {
		t.Errorf("Round-trip failed: expected %d persons, got %d", len(arch.Persons), len(arch2.Persons))
	}

	if len(arch2.Relations) != len(arch.Relations) {
		t.Errorf("Round-trip failed: expected %d relations, got %d", len(arch.Relations), len(arch2.Relations))
	}

	if len(arch2.Systems) > 0 && len(arch2.Systems[0].Containers) != len(arch.Systems[0].Containers) {
		t.Errorf("Round-trip failed: expected %d containers, got %d", len(arch.Systems[0].Containers), len(arch2.Systems[0].Containers))
	}
}

func TestConverter_ToArchitecture_InvalidRelations(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"relations": [
				{
					"from": "",
					"to": "API"
				},
				{
					"from": "User",
					"to": ""
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	_, err := converter.ToArchitecture([]byte(jsonData))
	if err == nil {
		t.Error("Expected error for invalid relations")
	}
}

func TestConverter_ToArchitecture_InvalidRelationsInSystem(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Test System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"systems": [
				{
					"id": "API",
					"label": "API Service",
					"relations": [
						{
							"from": "",
							"to": "Database"
						},
						{
							"from": "API",
							"to": "Database"
						}
					]
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Should handle invalid relations gracefully: %v", err)
	}

	// Invalid relation should be skipped, valid one should remain
	if len(arch.Systems) == 0 {
		t.Fatal("Expected at least one system")
	}

	// After post-processing, relations are in sys.Relations
	// The invalid relation should be skipped, so we should have 1 valid relation
	if len(arch.Systems[0].Relations) != 1 {
		t.Errorf("Expected 1 valid relation (invalid one skipped), but found %d relations", len(arch.Systems[0].Relations))
	}
}

func TestSanitizeFileName(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"Test System", "Test_System"},
		{"My-Architecture", "My-Architecture"},
		{"System@123", "System123"},
		{"", "architecture"},
		{"   ", "architecture"},
		{"E-Commerce Platform", "E-Commerce_Platform"},
	}

	for _, tt := range tests {
		result := sanitizeFileName(tt.input)
		if result != tt.expected {
			t.Errorf("sanitizeFileName(%q) = %q, expected %q", tt.input, result, tt.expected)
		}
	}
}

func stringPtr(s string) *string {
	return &s
}

func TestConverter_ToArchitecture_Scenarios(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Scenario System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"scenarios": [
				{
					"id": "S1",
					"label": "Login Flow",
					"description": "User logs in"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert scenario: %v", err)
	}

	if len(arch.Scenarios) != 1 {
		t.Fatalf("Expected 1 scenario, got %d", len(arch.Scenarios))
	}

	s := arch.Scenarios[0]
	if s.ID != "S1" || s.Title != "Login Flow" {
		t.Errorf("Scenario mismatch: %+v", s)
	}
}

func TestConverter_ToArchitecture_Contracts(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Contract System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"contracts": [
				{
					"id": "C1",
					"label": "API Contract"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert contract: %v", err)
	}

	// Contracts are stored in a ContractsBlock
	found := false
	for _, item := range arch.Items {
		if item.ContractsBlock != nil {
			if len(item.ContractsBlock.Contracts) == 1 {
				c := item.ContractsBlock.Contracts[0]
				if c.ID == "C1" {
					found = true
					break
				}
			}
		}
	}
	if !found {
		t.Error("Expected contract C1 not found")
	}
}

func TestConverter_ToArchitecture_Deployment(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Deployment System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"deployment": [
				{
					"id": "Node1",
					"label": "Server"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert deployment: %v", err)
	}

	if len(arch.DeploymentNodes) != 1 {
		t.Fatalf("Expected 1 deployment node, got %d", len(arch.DeploymentNodes))
	}
	d := arch.DeploymentNodes[0]
	if d.ID != "Node1" || d.Label != "Server" {
		t.Errorf("Deployment node mismatch: %+v", d)
	}
}

func TestConverter_ToArchitecture_SharedArtifacts(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Shared System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"sharedArtifacts": [
				{
					"id": "SA1",
					"label": "Common Lib"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert shared artifact: %v", err)
	}

	// SharedArtifacts are stored in Items directly (no convenience field on Architecture yet?)
	// Checking Items
	found := false
	for _, item := range arch.Items {
		if item.SharedArtifact != nil {
			if item.SharedArtifact.ID == "SA1" && item.SharedArtifact.Label == "Common Lib" {
				found = true
				break
			}
		}
	}
	if !found {
		t.Error("Expected shared artifact SA1 not found")
	}
}

func TestConverter_ToArchitecture_Libraries(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Lib System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"libraries": [
				{
					"id": "Lib1",
					"label": "Core"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert library: %v", err)
	}

	found := false
	for _, item := range arch.Items {
		if item.Library != nil {
			if item.Library.ID == "Lib1" && item.Library.Label == "Core" {
				found = true
				break
			}
		}
	}
	if !found {
		t.Error("Expected library Lib1 not found")
	}
}

func TestConverter_ToArchitecture_ConstraintsConventions(t *testing.T) {
	jsonData := `{
		"metadata": {
			"name": "Rules System",
			"version": "1.0.0",
			"generated": "2025-12-01T20:30:00+05:30"
		},
		"architecture": {
			"constraints": [
				{
					"key": "C1",
					"value": "Must be secure"
				}
			],
			"conventions": [
				{
					"key": "Conv1",
					"value": "Use camelCase"
				}
			]
		},
		"navigation": {}
	}`

	converter := NewConverter()
	arch, err := converter.ToArchitecture([]byte(jsonData))
	if err != nil {
		t.Fatalf("Failed to convert constraints/conventions: %v", err)
	}

	// Check Constraints
	foundC := false
	for _, item := range arch.Items {
		if item.ConstraintsBlock != nil {
			if len(item.ConstraintsBlock.Entries) == 1 {
				e := item.ConstraintsBlock.Entries[0]
				if e.Key == "C1" && e.Value == "Must be secure" {
					foundC = true
				}
			}
		}
	}
	if !foundC {
		t.Error("Expected constraint C1 not found")
	}

	// Check Conventions
	foundConv := false
	for _, item := range arch.Items {
		if item.ConventionsBlock != nil {
			if len(item.ConventionsBlock.Entries) == 1 {
				e := item.ConventionsBlock.Entries[0]
				if e.Key == "Conv1" && e.Value == "Use camelCase" {
					foundConv = true
				}
			}
		}
	}
	if !foundConv {
		t.Error("Expected convention Conv1 not found")
	}
}
