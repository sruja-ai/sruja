package views

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestApplyStyles_DefaultSystemTagAndDatastoreTags(t *testing.T) {
	dsl := `model {
        S1 = system "System1" {
            DB = database "Database" #storage
        }
    }`
	prog := parseDSL(t, dsl)

	vb := &language.ViewBlock{Styles: &language.StylesBlock{Styles: []*language.ElementStyle{
		{Target: "element", Tag: "System", Properties: []*language.StyleProperty{{Key: "color", Value: mkStrV("\"blue\"")}}},
		{Target: "element", Tag: "storage", Properties: []*language.StyleProperty{{Key: "shape", Value: mkStrV("\"cylinder\"")}}},
	}}}

	styles := ApplyStyles(prog, vb)
	// Default tag System should apply to system id
	if styles["S1"]["color"] != "blue" {
		t.Fatalf("expected system style color blue, got %q", styles["S1"]["color"])
	}
	// Metadata tag "storage" should apply to S1.DB
	if styles["S1.DB"]["shape"] != "cylinder" {
		t.Fatalf("expected datastore shape cylinder, got %q", styles["S1.DB"]["shape"])
	}
}
