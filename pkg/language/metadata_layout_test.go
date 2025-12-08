package language

import (
	"testing"
)

// TestMetadata_LayoutPositions tests metadata with layout positions
func TestMetadata_LayoutPositions(t *testing.T) {
	dsl := `architecture "LayoutTest" {
		person Customer "Customer" {
			metadata {
				pos_x "100"
				pos_y "50"
			}
		}
		
		system Backend "Backend" {
			metadata {
				layout_x "200"
				layout_y "100"
				layout_w "300"
				layout_h "200"
			}
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Verify metadata was parsed
	if len(program.Architecture.Persons) != 1 {
		t.Fatalf("Expected 1 person, got %d", len(program.Architecture.Persons))
	}

	person := program.Architecture.Persons[0]
	if len(person.Metadata) == 0 {
		t.Error("Expected metadata on person")
	}

	// Check for pos_x and pos_y
	hasX, hasY := false, false
	for _, m := range person.Metadata {
		if m.Key == "pos_x" {
			hasX = true
			if m.Value == nil || *m.Value != "100" {
				t.Errorf("Expected pos_x value '100', got %v", m.Value)
			}
		}
		if m.Key == "pos_y" {
			hasY = true
			if m.Value == nil || *m.Value != "50" {
				t.Errorf("Expected pos_y value '50', got %v", m.Value)
			}
		}
	}

	if !hasX || !hasY {
		t.Error("Expected pos_x and pos_y in person metadata")
	}

	// Verify system metadata
	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	system := program.Architecture.Systems[0]
	if len(system.Metadata) == 0 {
		t.Error("Expected metadata on system")
	}
}

// TestMetadata_Arrays tests metadata with array values
func TestMetadata_Arrays(t *testing.T) {
	dsl := `architecture "ArrayTest" {
		system Backend "Backend" {
			container API "API" {
				metadata {
					tags ["api", "rest", "v2"]
					environments ["dev", "staging", "prod"]
				}
			}
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	if len(program.Architecture.Systems) == 0 {
		t.Fatal("Expected at least 1 system")
	}

	system := program.Architecture.Systems[0]
	if len(system.Containers) == 0 {
		t.Fatal("Expected at least 1 container")
	}

	container := system.Containers[0]
	if len(container.Metadata) == 0 {
		t.Error("Expected metadata on container")
	}
}

// TestMetadata_OnAllElementTypes tests metadata on various element types
func TestMetadata_OnAllElementTypes(t *testing.T) {
	dsl := `architecture "MetadataTest" {
		person Customer "Customer" {
			metadata {
				role "user"
			}
		}
		
		system Backend "Backend" {
			metadata {
				tier "critical"
			}
			
			container API "API" {
				metadata {
					port "8080"
				}
				
				component Auth "Auth" {
					metadata {
						lib "jwt"
					}
				}
				
				datastore Cache "Redis" {
					metadata {
						type "in-memory"
					}
				}
				
				queue Events "Events" {
					metadata {
						broker "rabbitmq"
					}
				}
			}
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Person metadata
	if len(program.Architecture.Persons) > 0 {
		person := program.Architecture.Persons[0]
		if len(person.Metadata) == 0 {
			t.Error("Expected metadata on person")
		}
	}

	// System metadata
	if len(program.Architecture.Systems) > 0 {
		system := program.Architecture.Systems[0]
		if len(system.Metadata) == 0 {
			t.Error("Expected metadata on system")
		}

		// Container metadata
		if len(system.Containers) > 0 {
			container := system.Containers[0]
			if len(container.Metadata) == 0 {
				t.Error("Expected metadata on container")
			}

			// Component metadata
			if len(container.Components) > 0 {
				component := container.Components[0]
				if len(component.Metadata) == 0 {
					t.Error("Expected metadata on component")
				}
			}

			// DataStore metadata
			if len(container.DataStores) > 0 {
				ds := container.DataStores[0]
				if len(ds.Metadata) == 0 {
					t.Error("Expected metadata on datastore")
				}
			}

			// Queue metadata
			if len(container.Queues) > 0 {
				queue := container.Queues[0]
				if len(queue.Metadata) == 0 {
					t.Error("Expected metadata on queue")
				}
			}
		}
	}
}

// TestMetadata_BrandAndLayout tests architecture-level brand and layout metadata
func TestMetadata_BrandAndLayout(t *testing.T) {
	dsl := `architecture "BrandTest" {
		metadata {
			brand_logo "logo.png"
			layout_engine "dagre"
			version "1.0.0"
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	if len(program.Architecture.Metadata) == 0 {
		t.Fatal("Expected architecture metadata")
	}

	// Check for brand_logo and layout_engine
	hasLogo, hasEngine := false, false
	for _, m := range program.Architecture.Metadata {
		if m.Key == "brand_logo" {
			hasLogo = true
		}
		if m.Key == "layout_engine" {
			hasEngine = true
		}
	}

	if !hasLogo {
		t.Error("Expected brand_logo in metadata")
	}
	if !hasEngine {
		t.Error("Expected layout_engine in metadata")
	}
}

// TestMetadata_NegativeCoordinates tests negative coordinates in layout
func TestMetadata_NegativeCoordinates(t *testing.T) {
	dsl := `architecture "NegativeTest" {
		person User "User" {
			metadata {
				pos_x "-50"
				pos_y "-100"
			}
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	if len(program.Architecture.Persons) == 0 {
		t.Fatal("Expected person")
	}

	person := program.Architecture.Persons[0]
	if len(person.Metadata) == 0 {
		t.Fatal("Expected metadata on person")
	}

	// Verify negative values are preserved
	for _, m := range person.Metadata {
		if m.Key == "pos_x" && (m.Value == nil || *m.Value != "-50") {
			t.Errorf("Expected pos_x '-50', got %v", m.Value)
		}
		if m.Key == "pos_y" && (m.Value == nil || *m.Value != "-100") {
			t.Errorf("Expected pos_y '-100', got %v", m.Value)
		}
	}
}
