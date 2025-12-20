// pkg/export/json/json.go
// Main JSON exporter for architecture AST - LikeC4 compatible format
package json

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// Exporter exports Architecture to LikeC4-compatible JSON format
type Exporter struct {
	Extended bool // If true, include pre-computed views in output
}

// NewExporter creates a new JSON exporter
func NewExporter() *Exporter { return &Exporter{} }

// Export converts Program (LikeC4 AST) to LikeC4-compatible JSON string
func (e *Exporter) Export(program *language.Program) (string, error) {
	likec4 := NewLikeC4Exporter()
	likec4.Extended = e.Extended
	return likec4.Export(program)
}

// ExportAsModelDump returns the structured model dump
func (e *Exporter) ExportAsModelDump(program *language.Program) *SrujaModelDump {
	likec4 := NewLikeC4Exporter()
	likec4.Extended = e.Extended
	return likec4.ToModelDump(program)
}

// ExportCompact exports without indentation
func (e *Exporter) ExportCompact(program *language.Program) ([]byte, error) {
	likec4 := NewLikeC4Exporter()
	likec4.Extended = e.Extended
	return likec4.ExportCompact(program)
}
