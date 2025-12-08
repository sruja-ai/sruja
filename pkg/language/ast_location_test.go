// pkg/language/ast_location_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestLocation_Methods(t *testing.T) {
	// Test that all Location() methods return empty SourceLocation
	// This is a simple test to ensure the methods are callable

	tests := []struct {
		name string
		elem interface {
			Location() language.SourceLocation
		}
	}{
		{"MetaEntry", &language.MetaEntry{}},
		{"Architecture", &language.Architecture{}},
		{"MetadataBlock", &language.MetadataBlock{}},
		{"PropertiesBlock", &language.PropertiesBlock{}},
		{"StyleBlock", &language.StyleBlock{}},
		{"Contract", &language.Contract{}},
		{"SchemaBlock", &language.SchemaBlock{}},
		{"ConstraintEntry", &language.ConstraintEntry{}},
		{"ConventionEntry", &language.ConventionEntry{}},
		{"ImportSpec", &language.ImportSpec{}},
		{"System", &language.System{}},
		{"Container", &language.Container{}},
		{"Component", &language.Component{}},
		{"DataStore", &language.DataStore{}},
		{"Queue", &language.Queue{}},
		{"Person", &language.Person{}},
		{"Relation", &language.Relation{}},
		{"Requirement", &language.Requirement{}},
		{"ADR", &language.ADR{}},
		{"SharedArtifact", &language.SharedArtifact{}},
		{"Library", &language.Library{}},
		{"DeploymentNode", &language.DeploymentNode{}},
		{"ContainerInstance", &language.ContainerInstance{}},
		{"InfrastructureNode", &language.InfrastructureNode{}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			loc := tt.elem.Location()
			if loc.File != "" || loc.Line != 0 || loc.Column != 0 {
				t.Errorf("Expected empty SourceLocation, got %+v", loc)
			}
		})
	}
}

func TestSourceLocation_String(t *testing.T) {
	loc := language.SourceLocation{
		File:   "test.sruja",
		Line:   5,
		Column: 12,
	}

	str := loc.String()
	expected := "test.sruja:5:12"
	if str != expected {
		t.Errorf("Expected %q, got %q", expected, str)
	}
}

func TestSourceLocation_String_NoFile(t *testing.T) {
	loc := language.SourceLocation{
		Line:   10,
		Column: 5,
	}

	str := loc.String()
	expected := ":10:5"
	if str != expected {
		t.Errorf("Expected %q, got %q", expected, str)
	}
}
