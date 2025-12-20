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
		// LikeC4 Structure
		{"specification", language.TOKEN_SPECIFICATION},
		{"model", language.TOKEN_MODEL},
		{"views", language.TOKEN_VIEWS},
		{"view", language.TOKEN_VIEW},

		// LikeC4 View Predicates
		{"include", language.TOKEN_INCLUDE},
		{"exclude", language.TOKEN_EXCLUDE},
		{"of", language.TOKEN_OF},
		{"title", language.TOKEN_TITLE},

		// Element Types (now keywords)
		{"element", language.TOKEN_ELEMENT},
		{"person", language.TOKEN_PERSON},
		{"system", language.TOKEN_SYSTEM},
		{"container", language.TOKEN_CONTAINER},
		{"component", language.TOKEN_COMPONENT},
		{"database", language.TOKEN_DATABASE},
		{"queue", language.TOKEN_QUEUE},

		// Properties
		{"description", language.TOKEN_DESCRIPTION},
		{"technology", language.TOKEN_TECHNOLOGY},
		{"metadata", language.TOKEN_METADATA},

		// Booleans
		{"true", language.TOKEN_TRUE},
		{"false", language.TOKEN_FALSE},

		// Sruja-specific keywords (still in use)
		{"story", language.TOKEN_STORY},
		{"functional", language.TOKEN_FUNCTIONAL},
		{"nonfunctional", language.TOKEN_NONFUNCTIONAL},
		{"constraint", language.TOKEN_CONSTRAINT},
		// Legacy plural forms (not used in parser)
		{"requirements", language.TOKEN_REQUIREMENTS},
		{"adrs", language.TOKEN_ADRS},

		// Non-keywords (should return IDENT)
		{"workspace", language.TOKEN_IDENT}, // workspace removed - not used
		{"library", language.TOKEN_IDENT},   // library removed - not used with LikeC4
		{"relation", language.TOKEN_IDENT},
		{"architecture", language.TOKEN_IDENT},
		{"datastore", language.TOKEN_IDENT}, // Note: 'database' is keyword, 'datastore' is not
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
