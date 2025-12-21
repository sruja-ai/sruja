package markdown

import (
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Helper to get a mermaid exporter
func (e *Exporter) getMermaid() *mermaid.Exporter {
	cfg := mermaid.DefaultConfig()
	// Map markdown options to mermaid config if needed
	return mermaid.NewExporter(cfg)
}

func (e *Exporter) generateL1Diagram(prog *language.Program) string {
	m := e.getMermaid()
	return m.GenerateL1(prog)
}

func (e *Exporter) generateL2Diagram(sys *language.System, prog *language.Program) string {
	m := e.getMermaid()
	return m.GenerateL2(sys, prog)
}

func (e *Exporter) generateL3Diagram(cont *language.Container, systemID string, prog *language.Program) string {
	m := e.getMermaid()
	return m.GenerateL3(cont, systemID, prog)
}
