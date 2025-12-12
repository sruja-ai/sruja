package json

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// TestDSLToJSON_ComprehensiveParsing tests the full DSL parsing pipeline
// by parsing complex DSL and validating JSON output. This improves parser coverage.
func TestDSLToJSON_ComprehensiveParsing(t *testing.T) {
	dsl := `architecture "ComprehensiveTest" {
		metadata {
			version "1.0.0"
			owner "Tech Team"
			brand_logo "logo.png"
		}

        // imports removed

		person Customer "Customer" {
			description "End user"
		}

		person Admin "Administrator"

		system Backend "Backend System" {
			description "Core backend services"
			
			properties {
				"tier": "1"
				"criticality": "high"
			}
			style {
				shape: "rounded_box"
			}
			slo {
				availability {
					target "99.9%"
					window "30d"
				}
			}

			container API "REST API" {
				technology "Go"
				tags ["api", "rest"]
				version "2.0.0"
				description "Main API gateway"
				
				properties {
					"language": "go"
				}
				
				component Auth "Authentication" {
					technology "JWT"
					description "Handles authentication"
				}
				
				component Router "Request Router" {
					technology "Chi"
				}
				
				datastore Cache "Redis Cache" {
					description "Session cache"
				}
				
				queue Events "Event Queue" {
					description "Async events"
				}
				
				Auth -> Router "forwards to"
			}
			
			container Worker "Background Worker" {
				technology "Python"
				scale {
					min 1
					max 10
					metric "cpu > 80%"
				}
			}
			
			datastore DB "PostgreSQL" {
				description "Main database"
			}
			
			queue JobQueue "Job Queue"
			
			API -> DB "reads/writes"
			Worker -> DB "reads/writes"

            // root-only: requirements declared at architecture level
		}

		system Frontend "Frontend" {
			container Web "Web App" {
				technology "React"
			}
		}

		datastore Analytics "Analytics DB"
		queue Notifications "Notification Queue"

		Customer -> Frontend "uses"
		Frontend -> Backend "calls" "HTTPS"

		requirement FR1 functional "User authentication required"
		requirement NFR1 non_functional "99.9% uptime"

		adr ADR1 "Use microservices architecture" {
			status "Accepted"
			context "Need to scale independently"
			decision "Adopt microservices"
			consequences "Increased operational complexity"
		}

		adr ADR2 "Use PostgreSQL" {
			status "Proposed"
		}

		scenario Login "User Login Flow" "User logs into the system" {
			Customer -> Frontend "Opens app"
			Frontend -> Backend "POST /login"
			Backend -> Customer "Returns token"
		}

		scenario Checkout "Checkout Process" {
			Customer -> Frontend "Clicks checkout"
		}
	}`

	// Parse DSL
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Export to JSON
	exporter := NewExporter()
	jsonOutput, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Failed to export to JSON: %v", err)
	}

	// Parse JSON to validate structure
	var result map[string]interface{}
	if err := json.Unmarshal([]byte(jsonOutput), &result); err != nil {
		t.Fatalf("Failed to parse JSON output: %v", err)
	}

	// Validate metadata
	metadata, ok := result["metadata"].(map[string]interface{})
	if !ok {
		t.Fatal("Expected metadata object")
	}
	if metadata["name"] != "ComprehensiveTest" {
		t.Errorf("Expected name 'ComprehensiveTest', got %v", metadata["name"])
	}

	// Validate architecture structure
	arch, ok := result["architecture"].(map[string]interface{})
	if !ok {
		t.Fatal("Expected architecture object")
	}

    // Imports feature removed; skip validation

	// Validate persons
	persons, ok := arch["persons"].([]interface{})
	if !ok || len(persons) != 2 {
		t.Errorf("Expected 2 persons, got %v", persons)
	}

	// Validate systems
	systems, ok := arch["systems"].([]interface{})
	if !ok || len(systems) != 2 {
		t.Errorf("Expected 2 systems, got %v", systems)
	}

	// Validate Backend system structure
	backend := systems[0].(map[string]interface{})
	if backend["id"] != "Backend" {
		t.Error("Expected Backend system")
	}

	containers := backend["containers"].([]interface{})
	if len(containers) != 2 {
		t.Errorf("Expected 2 containers in Backend, got %d", len(containers))
	}

	// Validate API container
	api := containers[0].(map[string]interface{})
	if api["technology"] == nil {
		t.Error("Expected technology in API container")
	}

	components := api["components"].([]interface{})
	if len(components) != 2 {
		t.Errorf("Expected 2 components in API, got %d", len(components))
	}

	datastores := api["datastores"].([]interface{})
	if len(datastores) != 1 {
		t.Error("Expected 1 datastore in API container")
	}

	queues := api["queues"].([]interface{})
	if len(queues) != 1 {
		t.Error("Expected 1 queue in API container")
	}

	// Validate system-level datastores
	sysDatastores := backend["datastores"].([]interface{})
	if len(sysDatastores) != 1 {
		t.Error("Expected 1 datastore at system level")
	} else {
		db := sysDatastores[0].(map[string]interface{})
		if db["description"] != "Main database" {
			t.Errorf("Expected database description 'Main database', got %v", db["description"])
		}
	}

	// Validate top-level relations
	relations, ok := arch["relations"].([]interface{})
	if !ok || len(relations) != 2 {
		t.Errorf("Expected 2 top-level relations, got %v", relations)
	}

	// Validate requirements
	requirements, ok := arch["requirements"].([]interface{})
	if !ok || len(requirements) != 2 {
		t.Errorf("Expected 2 requirements, got %v", requirements)
	}

	// Validate ADRs
	adrs, ok := arch["adrs"].([]interface{})
	if !ok || len(adrs) != 2 {
		t.Errorf("Expected 2 ADRs, got %v", adrs)
	}

	adr1 := adrs[0].(map[string]interface{})
	if adr1["status"] == nil {
		t.Error("Expected status in ADR")
	}

	// Validate scenarios
	scenarios, ok := arch["scenarios"].([]interface{})
	if !ok || len(scenarios) != 2 {
		t.Errorf("Expected 2 scenarios, got %v", scenarios)
	}

	scenario1 := scenarios[0].(map[string]interface{})
	steps := scenario1["steps"].([]interface{})
	if len(steps) != 3 {
		t.Errorf("Expected 3 steps in Login scenario, got %d", len(steps))
	}

	// Validate Properties and Style on Backend
	if backend["properties"] == nil {
		t.Error("Expected properties on Backend")
	} else {
		props := backend["properties"].(map[string]interface{})
		if props["tier"] != "1" {
			t.Errorf("Expected properties.tier to be '1', got %v", props["tier"])
		}
	}
	if backend["style"] == nil {
		t.Error("Expected style on Backend")
	}

	// Validate SLO on Backend
	if backend["slo"] == nil {
		t.Error("Expected slo on Backend")
	} else {
		slo := backend["slo"].(map[string]interface{})
		avail := slo["availability"].(map[string]interface{})
		if avail["target"] != "99.9%" {
			t.Errorf("Expected slo.availability.target to be '99.9%%', got %v", avail["target"])
		}
	}

    // Validate Requirements at architecture root
    if arch["requirements"] == nil {
        t.Error("Expected requirements at architecture root")
    } else {
        reqs := arch["requirements"].([]interface{})
        if len(reqs) < 2 { // FR1 and NFR1
            t.Errorf("Expected at least 2 requirements at root, got %d", len(reqs))
        }
    }

	// Validate Scale on Worker container (second container in Backend)
	worker := containers[1].(map[string]interface{})
	if worker["scale"] == nil {
		t.Error("Expected scale on Worker container")
	} else {
		scale := worker["scale"].(map[string]interface{})
		if scale["min"] != float64(1) { // generic JSON numbers are floats
			t.Errorf("Expected scale.min to be 1, got %v", scale["min"])
		}
	}
}

// TestDSLToJSON_EdgeCases tests edge cases in DSL parsing
func TestDSLToJSON_EdgeCases(t *testing.T) {
	testCases := []struct {
		name     string
		dsl      string
		validate func(*testing.T, string)
	}{
		{
			name: "Empty architecture",
			dsl:  `architecture "Empty" {}`,
			validate: func(t *testing.T, jsonOutput string) {
				var result map[string]interface{}
				json.Unmarshal([]byte(jsonOutput), &result)
				arch := result["architecture"].(map[string]interface{})
				if arch["systems"] != nil {
					t.Error("Expected no systems")
				}
			},
		},
		{
			name: "Nested qualified names",
			dsl: `architecture "Qualified" {
				system S1 "S1" {
					container C1 "C1" {
						component Comp1 "Comp1"
					}
				}
				S1.C1.Comp1 -> S1 "uses"
			}`,
			validate: func(t *testing.T, jsonOutput string) {
				var result map[string]interface{}
				json.Unmarshal([]byte(jsonOutput), &result)
				arch := result["architecture"].(map[string]interface{})
				relations := arch["relations"].([]interface{})
				if len(relations) == 0 {
					t.Error("Expected qualified name relation")
				}
			},
		},
		{
			name: "Multiple metadata entries",
			dsl: `architecture "Meta" {
				person P "Person" {
					metadata {
						role "admin"
						team "engineering"
						location "remote"
					}
				}
			}`,
			validate: func(t *testing.T, jsonOutput string) {
				// Just validate it parses
				var result map[string]interface{}
				if err := json.Unmarshal([]byte(jsonOutput), &result); err != nil {
					t.Error("Failed to parse JSON with multiple metadata")
				}
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			parser, _ := language.NewParser()
			program, _, err := parser.Parse("test.sruja", tc.dsl)
			if err != nil {
				t.Fatalf("Failed to parse: %v", err)
			}

			exporter := NewExporter()
			jsonOutput, err := exporter.Export(program.Architecture)
			if err != nil {
				t.Fatalf("Failed to export: %v", err)
			}

			tc.validate(t, jsonOutput)
		})
	}
}
