// pkg/language/ast_location_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

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
