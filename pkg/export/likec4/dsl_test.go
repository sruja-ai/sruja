package likec4

import (
	"strings"
	"testing"
)

func TestDSLExporter_ExportDSL(t *testing.T) {
	dsl := `specification {
		element component
		element database
	}
	model {
		user = person "User"
		app = system "App" {
			api = container "API"
		}
		user -> app.api "uses"
	}
	views {
		view index {
			title "Index"
			include *
		}
	}`
	prog := parseDSL(t, dsl)

	exp := NewDSLExporter()
	got := exp.ExportDSL(prog)

	required := []string{
		"specification {",
		"element component",
		"model {",
		"user = person \"User\"",
		"app = system \"App\" {",
		"user -> app.api \"uses\"",
		"views {",
		"view index {",
	}

	for _, req := range required {
		if !strings.Contains(got, req) {
			t.Errorf("expected DSL to contain %q, but it didn't.\nGot:\n%s", req, got)
		}
	}
}

func TestDSLExporter_Empty(t *testing.T) {
	exp := NewDSLExporter()
	if got := exp.ExportDSL(nil); got != "" {
		t.Errorf("expected empty string for nil program, got %q", got)
	}
}

func TestHelpers(t *testing.T) {
	if got := sanitizeID("my-id!"); got != "my_id_" {
		t.Errorf("sanitizeID failed: %q", got)
	}
	if got := sanitizeRef("A.B.C-D"); got != "A.B.C_D" {
		t.Errorf("sanitizeRef failed: %q", got)
	}
}
