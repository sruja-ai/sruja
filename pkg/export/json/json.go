// pkg/export/json/json.go
// Main JSON exporter for architecture AST
package json

import (
	"encoding/json"

	"github.com/sruja-ai/sruja/pkg/language"
)

type Exporter struct {
	Extended bool // If true, include pre-computed views in output
}

func NewExporter() *Exporter { return &Exporter{} }

func (e *Exporter) Export(arch *language.Architecture) (string, error) {
	doc := ArchitectureJSON{
		Metadata:     NewMetadata(arch.Name),
		Architecture: convertArchitectureToJSON(arch),
		Navigation:   buildNavigation(arch),
	}

	populateMetadataFromDSL(&doc.Metadata, arch)

	if e.Extended {
		doc.Views = GenerateViews(arch)
	}

	b, err := json.MarshalIndent(doc, "", "  ")
	if err != nil {
		return "", err
	}
	return string(b), nil
}
