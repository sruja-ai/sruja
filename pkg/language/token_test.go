// pkg/language/token_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestLookupIdent(t *testing.T) {
	tests := []struct {
		ident    string
		expected language.TokenType
	}{
		{"workspace", language.TOKEN_WORKSPACE},
		{"model", language.TOKEN_MODEL},
		{"views", language.TOKEN_VIEWS},
		{"system", language.TOKEN_SYSTEM},
		{"container", language.TOKEN_CONTAINER},
		{"component", language.TOKEN_COMPONENT},
		{"true", language.TOKEN_TRUE},
		{"false", language.TOKEN_FALSE},
		{"requirements", language.TOKEN_REQUIREMENTS},
		{"adrs", language.TOKEN_ADRS},
		{"functional", language.TOKEN_FUNCTIONAL},
		{"nonfunctional", language.TOKEN_NONFUNCTIONAL},
		{"constraint", language.TOKEN_CONSTRAINT},
		{"relation", language.TOKEN_IDENT},
		{"architecture", language.TOKEN_IDENT},
		{"person", language.TOKEN_IDENT},
		{"datastore", language.TOKEN_IDENT},
		{"queue", language.TOKEN_IDENT},
		{"unknown", language.TOKEN_IDENT},
		{"customId", language.TOKEN_IDENT},
		{"", language.TOKEN_IDENT},
	}

	for _, tt := range tests {
		result := language.LookupIdent(tt.ident)
		if result != tt.expected {
			t.Errorf("LookupIdent(%q) = %v, want %v", tt.ident, result, tt.expected)
		}
	}
}
