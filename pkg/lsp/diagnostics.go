package lsp

import (
	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func (s *Server) convertDiagnosticsToLSP(diags []diagnostics.Diagnostic) []lsp.Diagnostic {
	lspDiags := make([]lsp.Diagnostic, 0, len(diags))
	//nolint:gocritic // copying small struct
	for _, d := range diags {
		severity := lsp.Error
		if d.Severity == diagnostics.SeverityWarning {
			severity = lsp.Warning
		} else if d.Severity == diagnostics.SeverityInfo {
			severity = lsp.Info
		}

		lspDiags = append(lspDiags, lsp.Diagnostic{
			Range: lsp.Range{
				Start: lsp.Position{Line: max0(d.Location.Line - 1), Character: max0(d.Location.Column - 1)},
				End:   lsp.Position{Line: max0(d.Location.Line - 1), Character: max0(d.Location.Column - 1)},
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
