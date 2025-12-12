package json

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// TestJSONExport_PreservesImports removed - import feature removed
func TestJSONExport_PreservesImports_Removed(t *testing.T) {
	t.Skip("Import feature removed")
	dsl := `architecture "Test" {
  person Customer "Customer"
  system API "API Service" {}
}`

	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser error: %v", err)
	}
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	e := NewExporter()
	out, err := e.Export(prog.Architecture)
	if err != nil {
		t.Fatalf("export error: %v", err)
	}

	var doc ArchitectureJSON
	if err := json.Unmarshal([]byte(out), &doc); err != nil {
		t.Fatalf("json unmarshal: %v", err)
	}

	// Import feature removed - no longer checking imports
	_ = doc
}

func TestJSONExport_Metadata(t *testing.T) {
	dsl := `architecture "MetadataTest" {
		metadata {
			brand_logo "logo.png"
			layout_engine "elk"
		}
		system S "System" {
			metadata {
				pos_x "100"
				pos_y "200"
			}
		}
	}`

	doc := parseAndExport(t, dsl)

	if doc.Metadata.BrandLogo != "logo.png" {
		t.Errorf("Expected BrandLogo 'logo.png', got '%s'", doc.Metadata.BrandLogo)
	}
	if doc.Metadata.LayoutEngine != "elk" {
		t.Errorf("Expected LayoutEngine 'elk', got '%s'", doc.Metadata.LayoutEngine)
	}

	layout, ok := doc.Metadata.Layout["S"]
	if !ok {
		t.Fatal("Expected layout data for system S")
	}
	if layout.X != 100 || layout.Y != 200 {
		t.Errorf("Expected pos 100,200, got %d,%d", layout.X, layout.Y)
	}
}

func TestJSONExport_Elements(t *testing.T) {
	dsl := `architecture "ElementsTest" {
		person User "End User" {
			description "A user of the system"
		}
		system Sys "System" {
			description "Main System"
			container Web "Web App" {
				description "React App"
				component Comp "Component" {
					description "Logic"
				}
			}
			datastore DB "Database" {
				description "Postgres"
			}
			queue Q "Queue" {
				description "Kafka"
			}
		}
	}`

	doc := parseAndExport(t, dsl)

	// Check Person
	if len(doc.Architecture.Persons) != 1 {
		t.Fatalf("Expected 1 person, got %d", len(doc.Architecture.Persons))
	}
	p := doc.Architecture.Persons[0]
	if p.ID != "User" || p.Label != "End User" || *p.Description != "A user of the system" {
		t.Errorf("Person mismatch: %+v", p)
	}

	// Check System
	if len(doc.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(doc.Architecture.Systems))
	}
	s := doc.Architecture.Systems[0]
	if s.ID != "Sys" || s.Label != "System" || *s.Description != "Main System" {
		t.Errorf("System mismatch: %+v", s)
	}

	// Check Container
	if len(s.Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(s.Containers))
	}
	c := s.Containers[0]
	if c.ID != "Web" || c.Label != "Web App" || *c.Description != "React App" {
		t.Errorf("Container mismatch: %+v", c)
	}

	// Check Component
	if len(c.Components) != 1 {
		t.Fatalf("Expected 1 component, got %d", len(c.Components))
	}
	comp := c.Components[0]
	if comp.ID != "Comp" || comp.Label != "Component" || *comp.Description != "Logic" {
		t.Errorf("Component mismatch: %+v", comp)
	}

	// Check DataStore
	if len(s.DataStores) != 1 {
		t.Fatalf("Expected 1 datastore, got %d", len(s.DataStores))
	}
	d := s.DataStores[0]
	if d.ID != "DB" || d.Label != "Database" {
		t.Errorf("DataStore mismatch: %+v", d)
	}

	// Check Queue
	if len(s.Queues) != 1 {
		t.Fatalf("Expected 1 queue, got %d", len(s.Queues))
	}
	q := s.Queues[0]
	if q.ID != "Q" || q.Label != "Queue" {
		t.Errorf("Queue mismatch: %+v", q)
	}
}

func TestJSONExport_Relations(t *testing.T) {
	dsl := `architecture "RelationsTest" {
		person User "User"
		system Sys "System"
		User -> Sys "Uses" "HTTP"
	}`

	doc := parseAndExport(t, dsl)

	if len(doc.Architecture.Relations) != 1 {
		t.Fatalf("Expected 1 relation, got %d", len(doc.Architecture.Relations))
	}
	r := doc.Architecture.Relations[0]
	if r.From != "User" || r.To != "Sys" {
		t.Errorf("Relation endpoints mismatch: %s -> %s", r.From, r.To)
	}
	if r.Verb == nil || *r.Verb != "Uses" {
		t.Errorf("Relation verb mismatch: %v", r.Verb)
	}
	if r.Label == nil || *r.Label != "HTTP" {
		t.Errorf("Relation label mismatch: %v", r.Label)
	}
}

func TestJSONExport_RequirementsADRs(t *testing.T) {
	dsl := `architecture "ReqADRTest" {
		requirement R1 functional "Must be fast"
		adr ADR1 "Use Go" {
			status "Accepted"
			context "Need speed"
			decision "Go"
			consequences "Fast"
		}
	}`

	doc := parseAndExport(t, dsl)

	// Check Requirement
	if len(doc.Architecture.Requirements) != 1 {
		t.Fatalf("Expected 1 requirement, got %d", len(doc.Architecture.Requirements))
	}
	req := doc.Architecture.Requirements[0]
	if req.ID != "R1" || req.Type != "functional" || req.Description != "Must be fast" {
		t.Errorf("Requirement mismatch: %+v", req)
	}

	// Check ADR
	if len(doc.Architecture.ADRs) != 1 {
		t.Fatalf("Expected 1 ADR, got %d", len(doc.Architecture.ADRs))
	}
	adr := doc.Architecture.ADRs[0]
	if adr.ID != "ADR1" || adr.Title != "Use Go" {
		t.Errorf("ADR header mismatch: %+v", adr)
	}
	if adr.Status == nil || *adr.Status != "Accepted" {
		t.Errorf("ADR status mismatch: %v", adr.Status)
	}
}

func TestJSONExport_Scenarios(t *testing.T) {
	dsl := `architecture "ScenarioTest" {
		system A "A"
		system B "B"
		scenario S1 "Login Flow" "User logs in" {
			A -> B "Auth Request"
		}
	}`

	doc := parseAndExport(t, dsl)

	if len(doc.Architecture.Scenarios) != 1 {
		t.Fatalf("Expected 1 scenario, got %d", len(doc.Architecture.Scenarios))
	}
	s := doc.Architecture.Scenarios[0]
	if s.ID != "S1" || s.Title != "Login Flow" || *s.Description != "User logs in" {
		t.Errorf("Scenario header mismatch: %+v", s)
	}

	if len(s.Steps) != 1 {
		t.Fatalf("Expected 1 step, got %d", len(s.Steps))
	}
	step := s.Steps[0]
	if step.From != "A" || step.To != "B" || step.Description == nil || *step.Description != "Auth Request" {
		t.Errorf("Step mismatch: %+v", step)
	}
}

// Helper function to parse DSL and export to JSON struct
func parseAndExport(t *testing.T, dsl string) ArchitectureJSON {
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("parser error: %v", err)
	}
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	// PostProcess is already called by Parse

	e := NewExporter()
	out, err := e.Export(prog.Architecture)
	if err != nil {
		t.Fatalf("export error: %v", err)
	}

	var doc ArchitectureJSON
	if err := json.Unmarshal([]byte(out), &doc); err != nil {
		t.Fatalf("json unmarshal: %v", err)
	}
	return doc
}
func TestJSONExport_ComplexContainerComponent(t *testing.T) {
	dsl := `architecture "ComplexTest" {
		system Sys "System" {
			container Web "Web App" {
				technology "React"
				tags ["frontend", "spa"]
				version "1.0.0"
                metadata {
                    key "value"
                }
                
                component Comp "Component" {
                    technology "Redux"
                    metadata {
                        key2 "value2"
                    }
                }
			}
		}
	}`

	doc := parseAndExport(t, dsl)

	if len(doc.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system")
	}
	c := doc.Architecture.Systems[0].Containers[0]

	if c.Technology == nil || *c.Technology != "React" {
		t.Error("Container technology mismatch")
	}
	if len(c.Tags) != 2 {
		t.Error("Container tags mismatch")
	}
	if c.Version == nil || *c.Version != "1.0.0" {
		t.Error("Container version mismatch")
	}
    // Root-only policy: no container-level requirements/ADRs in export
	if len(c.Metadata) != 1 {
		t.Error("Container metadata mismatch")
	}

	comp := c.Components[0]
	if comp.Technology == nil || *comp.Technology != "Redux" {
		t.Error("Component technology mismatch")
	}
    // Root-only policy: no component-level requirements/ADRs in export
	if len(comp.Metadata) != 1 {
		t.Error("Component metadata mismatch")
	}
}

func TestJSONExport_MetadataParsing(t *testing.T) {
	dsl := `architecture "MetaParseTest" {
		metadata {
			layout_x "10"
			layout_y "20"
			layout_w "100"
			layout_h "50"
		}
		system S "System" {
			metadata {
				pos_x "invalid"
				pos_y "200"
			}
		}
		container C "Container" {
			metadata {
				pos_x "-30"
				pos_y "40"
			}
		}
	}`

	doc := parseAndExport(t, dsl)

	// Check architecture layout (implicitly uses "MetaParseTest" ID or similar? No, architecture layout is usually for elements)
	// Actually populateMetadataFromDSL iterates over elements.
	// The architecture metadata block sets top-level metadata, but layout map is keyed by element ID.
	// Wait, populateMetadataFromDSL iterates arch.Persons, arch.Systems, etc.
	// It doesn't seem to process architecture-level layout_x/y for the architecture itself in the layout map?
	// Let's check the code:
	// It iterates Persons, Systems, Containers (top-level), DataStores, Queues.
	// So architecture-level metadata is only for brand_logo, layout_engine.

	// Let's check S (invalid int)
	if _, ok := doc.Metadata.Layout["S"]; ok {
		t.Error("Expected no layout for S due to invalid x")
	}

	// Let's check C (negative int)
	lC, ok := doc.Metadata.Layout["C"]
	if !ok {
		t.Error("Expected layout for C")
	}
	if lC.X != -30 || lC.Y != 40 {
		t.Errorf("Expected -30, 40, got %d, %d", lC.X, lC.Y)
	}
}

func TestJSONExport_NestedElements(t *testing.T) {
	dsl := `architecture "NestedTest" {
		system Sys "System" {
			container Web "Web" {
				datastore DB "DB"
				queue Q "Queue"
				Web -> DB "Reads"
			}
			datastore Cache "Cache"
			queue Topic "Topic"
			Web -> Cache "Writes"
		}
	}`

	doc := parseAndExport(t, dsl)

	sys := doc.Architecture.Systems[0]

	// Check System-level nested
	if len(sys.DataStores) != 1 || sys.DataStores[0].ID != "Cache" {
		t.Error("System DataStore mismatch")
	}
	if len(sys.Queues) != 1 || sys.Queues[0].ID != "Topic" {
		t.Error("System Queue mismatch")
	}
	if len(sys.Relations) != 1 {
		t.Error("System Relation mismatch")
	}

	// Check Container-level nested
	cont := sys.Containers[0]
	if len(cont.DataStores) != 1 || cont.DataStores[0].ID != "DB" {
		t.Error("Container DataStore mismatch")
	}
	if len(cont.Queues) != 1 || cont.Queues[0].ID != "Q" {
		t.Error("Container Queue mismatch")
	}
	if len(cont.Relations) != 1 {
		t.Error("Container Relation mismatch")
	}
}
