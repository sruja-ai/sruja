package markdown

import (
	"strings"

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
	m.Config.ViewLevel = 1
	return m.Export(prog)
}

func (e *Exporter) generateL2Diagram(sys *language.System, prog *language.Program) string {
	m := e.getMermaid()
	m.Config.ViewLevel = 2
	m.Config.TargetID = sys.ID
	return m.Export(prog)
}

func (e *Exporter) generateL3Diagram(cont *language.Container, systemID string, prog *language.Program) string {
	m := e.getMermaid()
	m.Config.ViewLevel = 3
	id := cont.ID
	if systemID != "" && !strings.Contains(cont.ID, systemID) {
		id = systemID + "." + cont.ID
	}
	m.Config.TargetID = id
	return m.Export(prog)
}
