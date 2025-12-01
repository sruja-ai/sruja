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
		Imports: []*language.ImportSpec{
			{Path: "other.sruja"},
			{Path: "billing.sruja", Alias: stringPtr("Billing")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `import "other.sruja"`) {
		t.Error("Should print import without alias")
	}
	if !strings.Contains(output, `import "billing.sruja" as Billing`) {
		t.Error("Should print import with alias")
	}
}

func TestPrinter_PrintImport(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Imports: []*language.ImportSpec{
			{Path: "other.sruja"},
			{Path: "billing.sruja", Alias: stringPtr("Billing")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `import "other.sruja"`) {
		t.Error("Should print import without alias")
	}
	if !strings.Contains(output, `import "billing.sruja" as Billing`) {
		t.Error("Should print import with alias")
	}
}

func TestPrinter_PrintRelation(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Relations: []*language.Relation{
			{From: "A", To: "B"},
			{From: "C", To: "D", Verb: stringPtr("calls")},
			{From: "E", To: "F", Verb: stringPtr("uses"), Label: stringPtr("HTTP")},
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
			{ID: "R1", Type: stringPtr("performance"), Description: stringPtr("p95<200ms")},
			{ID: "R2", Type: stringPtr("security"), Description: stringPtr("TLS1.3")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `requirement R1 performance "p95<200ms"`) {
		t.Error("Should print requirement")
	}
	if !strings.Contains(output, `requirement R2 security "TLS1.3"`) {
		t.Error("Should print all requirements")
	}
}

func TestPrinter_PrintADR(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		ADRs: []*language.ADR{
			{ID: "ADR001", Title: stringPtr("Use JWT")},
			{ID: "ADR002", Title: stringPtr("Use PostgreSQL")},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `adr ADR001 "Use JWT"`) {
		t.Error("Should print ADR")
	}
	if !strings.Contains(output, `adr ADR002 "Use PostgreSQL"`) {
		t.Error("Should print all ADRs")
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
							From: "Sys",
							To:   "Other",
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

func TestPrinter_PrintRequirement_InSystem(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Systems: []*language.System{
			{
				ID:    "Sys",
				Label: "System",
				Items: []language.SystemItem{
					{
						Requirement: &language.Requirement{
							ID:          "R1",
							Type:        stringPtr("performance"),
							Description: stringPtr("Fast"),
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `requirement R1 performance "Fast"`) {
		t.Error("Should print requirement within system")
	}
}

func TestPrinter_PrintADR_InSystem(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Systems: []*language.System{
			{
				ID:    "Sys",
				Label: "System",
				Items: []language.SystemItem{
					{
						ADR: &language.ADR{
							ID:    "ADR001",
							Title: stringPtr("Use JWT"),
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `adr ADR001 "Use JWT"`) {
		t.Error("Should print ADR within system")
	}
}

func TestPrinter_PrintRequirement_InComponent(t *testing.T) {
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
												Requirement: &language.Requirement{
													ID:          "R1",
													Type:        stringPtr("security"),
													Description: stringPtr("Secure"),
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

	if !strings.Contains(output, `requirement R1 security "Secure"`) {
		t.Logf("Actual output:\n%s", output)
		t.Error("Should print requirement within component")
	}
}

func TestPrinter_PrintADR_InComponent(t *testing.T) {
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
												ADR: &language.ADR{
													ID:    "ADR001",
													Title: stringPtr("Component ADR"),
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

	if !strings.Contains(output, `adr ADR001 "Component ADR"`) {
		t.Errorf("Should print ADR within component. Output:\n%s", output)
	}
}

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
													From: "Comp",
													To:   "Other",
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
