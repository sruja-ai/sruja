package lsp

import (
	"testing"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func TestConvertDiagnosticsToLSP(t *testing.T) {
	s := NewServer()

	inputDiags := []diagnostics.Diagnostic{
		{
			Code:     diagnostics.CodeSyntaxError,
			Severity: diagnostics.SeverityError,
			Message:  "unexpected token",
			Location: diagnostics.SourceLocation{
				File:   "test.sruja",
				Line:   5,
				Column: 10,
			},
		},
	}

	diags := s.convertDiagnosticsToLSP(inputDiags)

	if len(diags) != 1 {
		t.Fatalf("Expected 1 diagnostic, got %d", len(diags))
	}

	d := diags[0]
	if d.Message != "unexpected token" {
		t.Errorf("Expected message 'unexpected token', got '%s'", d.Message)
	}
	if d.Severity != lsp.Error {
		t.Errorf("Expected severity Error, got %v", d.Severity)
	}
	// Line/Column are 0-indexed in LSP, 1-indexed in diagnostics
	if d.Range.Start.Line != 4 || d.Range.Start.Character != 9 {
		t.Errorf("Expected range start 4:9, got %d:%d", d.Range.Start.Line, d.Range.Start.Character)
	}
}
