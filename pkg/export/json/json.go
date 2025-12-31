// pkg/export/json/json.go
// Main JSON exporter for architecture AST
package json

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// Wrapper exports Architecture to JSON format
type Wrapper struct {
	Extended bool // If true, include pre-computed views in output
}

// NewWrapper creates a new JSON exporter wrapper
func NewWrapper() *Wrapper { return &Wrapper{} }

// Export converts Program to JSON string
func (e *Wrapper) Export(program *language.Program) (string, error) {
	jsonExporter := NewExporter()
	jsonExporter.Extended = e.Extended
	return jsonExporter.Export(program)
}

// ExportAsModelDump returns the structured model dump
func (e *Wrapper) ExportAsModelDump(program *language.Program) *SrujaModelDump {
	jsonExporter := NewExporter()
	jsonExporter.Extended = e.Extended
	return jsonExporter.ToModelDump(program)
}

// ExportCompact exports without indentation
func (e *Wrapper) ExportCompact(program *language.Program) ([]byte, error) {
	jsonExporter := NewExporter()
	jsonExporter.Extended = e.Extended
	return jsonExporter.ExportCompact(program)
}
