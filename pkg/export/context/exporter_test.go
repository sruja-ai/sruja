package context

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export(t *testing.T) {
	// Helper to create string pointer
	strPtr := func(s string) *string { return &s }

	// Construct a sample AST
	// model {
	//   element system sys1 {
	//     technology "Go"
	//     container cont1
	//   }
	//   sys1 -> sys2
	// }
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind: "system",
							Name: "sys1",
							Body: &language.ElementDefBody{
								Items: []*language.BodyItem{
									{
										Technology: strPtr("Go"),
									},
									{
										Element: &language.ElementDef{
											Assignment: &language.ElementAssignment{
												Kind: "container",
												Name: "cont1",
											},
										},
									},
								},
							},
						},
					},
				},
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind: "system",
							Name: "sys2",
						},
					},
				},
				{
					Relation: &language.Relation{
						From:  language.QualifiedIdent{Parts: []string{"sys1"}},
						To:    language.QualifiedIdent{Parts: []string{"sys2"}},
						Label: strPtr("calls"),
					},
				},
			},
		},
	}

	opts := Options{
		Template: "proposal",
		Scope:    "",
	}
	exporter := NewExporter(opts)
	output := exporter.Export(prog)

	// Verify Header
	if !strings.Contains(output, "Architecture Model") {
		t.Errorf("Expected output to contain header 'Architecture Model'")
	}
	if !strings.Contains(output, "Deployment Proposal") {
		t.Errorf("Expected output to contain template text 'Deployment Proposal'")
	}

	// Verify Elements
	if !strings.Contains(output, "- **sys1** (system)") {
		t.Errorf("Expected output to contain system 'sys1'")
	}
	if !strings.Contains(output, "technology: Go") {
		t.Errorf("Expected output to contain metadata 'technology: Go'")
	}
	if !strings.Contains(output, "- **cont1** (container)") {
		t.Errorf("Expected output to contain container 'cont1'")
	}

	// Verify Relationships
	if !strings.Contains(output, "# Relationships") {
		t.Errorf("Expected output to contain '# Relationships' section")
	}
	if !strings.Contains(output, "[sys1] -> [sys2]: calls") {
		t.Errorf("Expected output to contain relationship '[sys1] -> [sys2]: calls', got: %s", output)
	}
}

func TestIsRelevantMetadata(t *testing.T) {
	tests := []struct {
		key  string
		want bool
	}{
		{"technology", true},
		{"Technology", true},
		{"cost", true},
		{"tier", true},
		{"random", false},
	}

	for _, tt := range tests {
		if got := isRelevantMetadata(tt.key); got != tt.want {
			t.Errorf("isRelevantMetadata(%q) = %v, want %v", tt.key, got, tt.want)
		}
	}
}
