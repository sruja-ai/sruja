// pkg/dx/explainer_test.go
// This file is kept for backward compatibility.
// Tests have been split into separate files:
//   - explainer_element_test.go
//   - explainer_relations_test.go
//   - explainer_metadata_test.go
//   - explainer_description_test.go
//   - explainer_format_test.go
package dx

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// stringPtr is a helper function used across test files
func stringPtr(s string) *string {
	return &s
}

// TestNewExplainer is kept here for backward compatibility
func TestNewExplainer(t *testing.T) {
	prog := &language.Program{
		Architecture: &language.Architecture{Name: "Test"},
	}
	explainer := NewExplainer(prog)
	if explainer == nil {
		t.Fatal("NewExplainer should not return nil")
	}
	if explainer.program != prog {
		t.Error("Explainer should store the program")
	}
}
