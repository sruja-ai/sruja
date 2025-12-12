// pkg/language/printer_test.go
package language_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtr(s string) *string {
	return &s
}

func TestPrinter_Print_Full(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	// Verify basic architecture printing
	if !strings.Contains(output, `architecture "Test"`) {
		t.Error("Should print architecture name")
	}
}

// TestPrinter_PrintImport removed - import feature removed
func TestPrinter_PrintImport_Removed(t *testing.T) {
	t.Skip("Import feature removed")
}

func TestPrinter_PrintRelation(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Relations: []*language.Relation{
			{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}},
			{From: language.QualifiedIdent{Parts: []string{"C"}}, To: language.QualifiedIdent{Parts: []string{"D"}}, Verb: stringPtr("calls")},
			{From: language.QualifiedIdent{Parts: []string{"E"}}, To: language.QualifiedIdent{Parts: []string{"F"}}, Verb: stringPtr("uses"), Label: stringPtr("HTTP")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, "A -> B") {
		t.Error("Should print simple relation")
	}
	if !strings.Contains(output, "C -> D calls") {
		t.Error("Should print relation with verb")
	}
	if !strings.Contains(output, `E -> F uses "HTTP"`) {
		t.Error("Should print relation with verb and label")
	}
}

func TestPrinter_PrintRequirement(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Requirements: []*language.Requirement{
			// Requirement.Type and Description are *string
			{ID: "R1", Type: stringPtr("performance"), Description: stringPtr("p95<200ms")},
			{ID: "R2", Type: stringPtr("security"), Description: stringPtr("TLS1.3")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `requirement R2 security "TLS1.3"`) {
		t.Error("Should print all requirements")
	}
}

func TestPrinter_PrintADR_Full(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		ADRs: []*language.ADR{
			{
				ID:    "ADR-001",
				Title: stringPtr("Use Go"),
				Body: &language.ADRBody{
					Status:       stringPtr("Accepted"),
					Context:      stringPtr("Need speed"),
					Decision:     stringPtr("Use Go"),
					Consequences: stringPtr("Good performance"),
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `adr ADR-001 "Use Go" {`) {
		t.Error("Should print ADR header")
	}
	if !strings.Contains(output, `status "Accepted"`) {
		t.Error("Should print status")
	}
	if !strings.Contains(output, `context "Need speed"`) {
		t.Error("Should print context")
	}
	if !strings.Contains(output, `decision "Use Go"`) {
		t.Error("Should print decision")
	}
	if !strings.Contains(output, `consequences "Good performance"`) {
		t.Error("Should print consequences")
	}
}

func TestPrinter_PrintSharedArtifact(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		SharedArtifacts: []*language.SharedArtifact{
			{ID: "SA1", Label: "Shared Lib"},
			{ID: "SA2", Label: "Shared Lib 2", Version: stringPtr("1.0")},
			{ID: "SA3", Label: "Shared Lib 3", Version: stringPtr("2.0"), Owner: stringPtr("team")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `sharedArtifact SA1 "Shared Lib"`) {
		t.Error("Should print shared artifact")
	}
	if !strings.Contains(output, `sharedArtifact SA2 "Shared Lib 2" version "1.0"`) {
		t.Error("Should print shared artifact with version")
	}
	if !strings.Contains(output, `sharedArtifact SA3 "Shared Lib 3" version "2.0" owner "team"`) {
		t.Error("Should print shared artifact with version and owner")
	}
}

func TestPrinter_PrintLibrary(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Libraries: []*language.Library{
			{ID: "Lib1", Label: "Library 1"},
			{ID: "Lib2", Label: "Library 2", Version: stringPtr("1.0")},
			{ID: "Lib3", Label: "Library 3", Version: stringPtr("2.0"), Owner: stringPtr("team")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `library Lib1 "Library 1"`) {
		t.Error("Should print library")
	}
	if !strings.Contains(output, `library Lib2 "Library 2" version "1.0"`) {
		t.Error("Should print library with version")
	}
	if !strings.Contains(output, `library Lib3 "Library 3" version "2.0" owner "team"`) {
		t.Error("Should print library with version and owner")
	}
}

func TestPrinter_PrintRelation_InSystem(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Systems: []*language.System{
			{
				ID:    "Sys",
				Label: "System",
				Items: []language.SystemItem{
					{
						Relation: &language.Relation{
							From: language.QualifiedIdent{Parts: []string{"Sys"}},
							To:   language.QualifiedIdent{Parts: []string{"Other"}},
							Verb: stringPtr("calls"),
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, "Sys -> Other calls") {
		t.Error("Should print relation within system")
	}
}

func TestPrinter_PrintRequirement_AtRoot(t *testing.T) {
    arch := &language.Architecture{
        Name: "Test",
        Systems: []*language.System{{ ID: "Sys", Label: "System" }},
        Requirements: []*language.Requirement{{ ID: "R1", Type: stringPtr("performance"), Description: stringPtr("Fast") }},
    }
    prog := &language.Program{Architecture: arch}
    printer := language.NewPrinter()
    output := printer.Print(prog)

    if !strings.Contains(output, `requirement R1 performance "Fast"`) {
        t.Error("Should print requirement at root")
    }
}

func TestPrinter_PrintADR_AtRoot(t *testing.T) {
    arch := &language.Architecture{
        Name: "Test",
        Systems: []*language.System{{ ID: "Sys", Label: "System" }},
        ADRs: []*language.ADR{{ ID: "ADR001", Title: stringPtr("Use JWT") }},
    }
    prog := &language.Program{Architecture: arch}
    printer := language.NewPrinter()
    output := printer.Print(prog)

    if !strings.Contains(output, `adr ADR001 "Use JWT"`) {
        t.Error("Should print ADR at root")
    }
}

// Removed: component-level requirement printing (root-only policy)

// Removed: component-level ADR printing (root-only policy)

func TestPrinter_PrintRelation_InComponent(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Systems: []*language.System{
			{
				ID:    "Sys",
				Label: "System",
				Items: []language.SystemItem{
					{
						Container: &language.Container{
							ID:    "Cont",
							Label: "Container",
							Items: []language.ContainerItem{
								{
									Component: &language.Component{
										ID:    "Comp",
										Label: "Component",
										Items: []language.ComponentItem{
											{
												Relation: &language.Relation{
													From: language.QualifiedIdent{Parts: []string{"Comp"}},
													To:   language.QualifiedIdent{Parts: []string{"Other"}},
												},
											},
										},
										// Add technology to trigger block printing
										Technology: stringPtr("Go"),
									},
								},
							},
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, "Comp -> Other") {
		t.Errorf("Should print relation within component. Output:\n%s", output)
	}
}
