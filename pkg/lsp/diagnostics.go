package lsp

import (
	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func (s *Server) convertDiagnosticsToLSP(diags []diagnostics.Diagnostic) []lsp.Diagnostic {
	lspDiags := make([]lsp.Diagnostic, 0, len(diags))
	//nolint:gocritic // copying small struct
	for _, d := range diags {
		var severity lsp.DiagnosticSeverity
		switch d.Severity {
		case diagnostics.SeverityWarning:
			severity = lsp.Warning
		case diagnostics.SeverityInfo:
			severity = lsp.Info
		default:
			severity = lsp.Error
		}

		// Calculate end position for better range highlighting
		startLine := max0(d.Location.Line - 1)
		startCol := max0(d.Location.Column - 1)
		endLine := startLine
		endCol := startCol + 1 // Default to single character

		// Try to estimate token length for better highlighting using context
		if len(d.Context) > 0 {
			// Use the error line (usually index 1 if we have previous line)
			lineIdx := 0
			if len(d.Context) > 1 {
				lineIdx = 1 // Error line is usually after previous line
			}
			if lineIdx < len(d.Context) {
				lineText := d.Context[lineIdx]
				if startCol < len(lineText) {
					estimatedEnd := startCol
					// Find word boundary or next delimiter
					for estimatedEnd < len(lineText) && estimatedEnd < startCol+50 {
						c := lineText[estimatedEnd]
						if c == ' ' || c == '\t' || c == '{' || c == '}' ||
							c == ':' || c == ',' || c == ';' || c == '\n' ||
							c == '[' || c == ']' || c == '(' || c == ')' {
							break
						}
						estimatedEnd++
					}
					if estimatedEnd > startCol {
						endCol = estimatedEnd
					}
				}
			}
		}

		lspDiags = append(lspDiags, lsp.Diagnostic{
			Range: lsp.Range{
				Start: lsp.Position{Line: startLine, Character: startCol},
				End:   lsp.Position{Line: endLine, Character: endCol},
			},
			Severity: severity,
			Code:     d.Code,
			Source:   "sruja",
			Message:  d.Message,
		})
	}
	return lspDiags
}

func max0(n int) int {
	if n < 0 {
		return 0
	}
	return n
}
