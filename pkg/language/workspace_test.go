package language

import (
	"path/filepath"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func TestWorkspace_New(t *testing.T) {
	ws := NewWorkspace()
	if ws == nil {
		t.Fatal("Expected workspace to be non-nil")
	}
	if ws.Programs == nil {
		t.Error("Expected Programs map to be initialized")
	}
	if len(ws.Aliases) == 0 {
		t.Error("Expected default aliases")
	}
}

func TestWorkspace_AddProgram(t *testing.T) {
	ws := NewWorkspace()
	prog := &Program{
		Model: &Model{},
	}
	diags := []diagnostics.Diagnostic{{Message: "test"}}

	ws.AddProgram("test.sruja", prog, diags)

	if len(ws.Programs) != 1 {
		t.Errorf("Expected 1 program, got %d", len(ws.Programs))
	}
	if len(ws.Diags) != 1 {
		t.Errorf("Expected 1 diagnostic, got %d", len(ws.Diags))
	}
}

func TestWorkspace_MergedProgram(t *testing.T) {
	ws := NewWorkspace()

	prog1 := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{{Element: &ElementKindDef{Name: "system"}}},
		},
		Model: &Model{
			Items: []ModelItem{{ElementDef: &ElementDef{Assignment: &ElementAssignment{Kind: "system", Name: "sys1"}}}},
		},
	}

	prog2 := &Program{
		Model: &Model{
			Items: []ModelItem{{ElementDef: &ElementDef{Assignment: &ElementAssignment{Kind: "system", Name: "sys2"}}}},
		},
		Views: &Views{
			Items: []*ViewsItem{{View: &ViewDef{Name: sPtrW("v1")}}},
		},
	}

	ws.AddProgram("p1.sruja", prog1, nil)
	ws.AddProgram("p2.sruja", prog2, nil)
	ws.AddProgram("empty.sruja", nil, nil) // Should be ignored

	merged := ws.MergedProgram()

	if merged == nil {
		t.Fatal("Expected merged program")
	}

	if len(merged.Specification.Items) != 1 {
		t.Errorf("Expected 1 spec item, got %d", len(merged.Specification.Items))
	}
	if len(merged.Model.Items) != 2 {
		t.Errorf("Expected 2 model items, got %d", len(merged.Model.Items))
	}
	if len(merged.Views.Items) != 1 {
		t.Errorf("Expected 1 view item, got %d", len(merged.Views.Items))
	}
}

func TestWorkspace_MergedProgram_Empty(t *testing.T) {
	ws := NewWorkspace()
	merged := ws.MergedProgram()

	if merged.Specification != nil {
		t.Error("Expected nil Specification")
	}
	if merged.Model != nil {
		t.Error("Expected nil Model")
	}
	if merged.Views != nil {
		t.Error("Expected nil Views")
	}
}

func TestWorkspace_Aliases(t *testing.T) {
	ws := NewWorkspace()

	path := ws.GetAliasPath("sruja.ai/stdlib")
	if path == "" {
		t.Error("Expected path for stdlib alias")
	}

	if ws.GetAliasPath("unknown") != "" {
		t.Error("Expected empty string for unknown alias")
	}

	// Test nil aliases
	ws.Aliases = nil
	if ws.GetAliasPath("foo") != "" {
		t.Error("Expected empty string for nil aliases")
	}
}

func TestWorkspace_IsStdLib(t *testing.T) {
	ws := NewWorkspace()
	if !ws.IsStdLib("sruja.ai/stdlib") {
		t.Error("Expected true for stdlib alias")
	}
	if !ws.IsStdLib("/path/to/pkg/stdlib/foo.sruja") {
		t.Error("Expected true for stdlib path")
	}
	if ws.IsStdLib("other/path") {
		t.Error("Expected false for other path")
	}
}

func TestWorkspace_ResolveRelativePath(t *testing.T) {
	ws := NewWorkspace()

	current := "/abs/path/main.sruja"

	// Absolute or alias import
	if got := ws.ResolveRelativePath(current, "lib"); got != "lib" {
		t.Errorf("Expected 'lib', got %q", got)
	}

	// Relative path
	expected := filepath.Join("/abs/path", "./sub/mod.sruja")
	if got := ws.ResolveRelativePath(current, "./sub/mod.sruja"); got != expected {
		t.Errorf("Expected %q, got %q", expected, got)
	}
}

func sPtrW(s string) *string {
	return &s
}
