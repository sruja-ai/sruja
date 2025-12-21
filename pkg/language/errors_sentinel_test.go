package language

import (
	"errors"
	"testing"
)

func TestSentinelErrors(t *testing.T) {
	t.Run("ParseError wraps ErrParse", func(t *testing.T) {
		err := NewParseError(SourceLocation{File: "test.sruja", Line: 1, Column: 1}, "test error")
		if !errors.Is(err, ErrParse) {
			t.Error("ParseError should wrap ErrParse")
		}
	})

	t.Run("ValidationError wraps ErrValidation", func(t *testing.T) {
		err := NewValidationError(SourceLocation{File: "test.sruja", Line: 1, Column: 1}, "rule1", "test error")
		if !errors.Is(err, ErrValidation) {
			t.Error("ValidationError should wrap ErrValidation")
		}
	})

	t.Run("CompilationError wraps ErrCompilation", func(t *testing.T) {
		err := NewCompilationError(SourceLocation{File: "test.sruja", Line: 1, Column: 1}, "test error")
		if !errors.Is(err, ErrCompilation) {
			t.Error("CompilationError should wrap ErrCompilation")
		}
	})
}

func TestErrorsAs(t *testing.T) {
	t.Run("errors.As with ParseError", func(t *testing.T) {
		err := NewParseError(SourceLocation{File: "test.sruja", Line: 5, Column: 10}, "syntax error")

		var parseErr *ParseError
		if !errors.As(err, &parseErr) {
			t.Error("errors.As should work with ParseError")
		}

		if parseErr.Location.Line != 5 {
			t.Errorf("Expected line 5, got %d", parseErr.Location.Line)
		}
	})

	t.Run("errors.As with ValidationError", func(t *testing.T) {
		err := NewValidationError(SourceLocation{File: "test.sruja", Line: 10, Column: 1}, "duplicate-id", "duplicate")

		var valErr *ValidationError
		if !errors.As(err, &valErr) {
			t.Error("errors.As should work with ValidationError")
		}

		if valErr.RuleID != "duplicate-id" {
			t.Errorf("Expected rule 'duplicate-id', got %s", valErr.RuleID)
		}
	})
}

func TestUnwrap(t *testing.T) {
	t.Run("ParseError.Unwrap", func(t *testing.T) {
		err := NewParseError(SourceLocation{}, "test")
		if err.Unwrap() != ErrParse {
			t.Error("Unwrap should return ErrParse")
		}
	})

	t.Run("ValidationError.Unwrap", func(t *testing.T) {
		err := NewValidationError(SourceLocation{}, "rule", "test")
		if err.Unwrap() != ErrValidation {
			t.Error("Unwrap should return ErrValidation")
		}
	})

	t.Run("CompilationError.Unwrap", func(t *testing.T) {
		err := NewCompilationError(SourceLocation{}, "test")
		if err.Unwrap() != ErrCompilation {
			t.Error("Unwrap should return ErrCompilation")
		}
	})
}
