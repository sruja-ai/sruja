package json

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Exporter converts Program AST to Sruja JSON
type Exporter struct {
	Extended bool // Include computed views with layout
}

// NewExporter creates a new exporter
func NewExporter() *Exporter {
	return &Exporter{}
}

// Export converts Program (AST) to JSON string with 2-space indentation.
// Returns empty JSON "{}" for nil programs without error for backward compatibility.
//
// Example:
//
//	exporter := NewExporter()
//	json, err := exporter.Export(program)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(json)
func (e *Exporter) Export(program *language.Program) (string, error) {
	if program == nil {
		return "{}", nil
	}

	dump := e.ToModelDump(program)

	data, err := json.MarshalIndent(dump, "", "  ")
	if err != nil {
		elementCount := 0
		if program.Model != nil {
			elementCount = len(program.Model.Items)
		}
		name := "unknown"
		if program.Model != nil {
			name = "Model"
		}
		return "", fmt.Errorf("export failed for program %s with %d elements: %w", name, elementCount, err)
	}
	return string(data), nil
}

// ExportCompact exports without indentation for smaller payload size.
// Suitable for network transmission or storage.
// Returns empty JSON "{}" for nil programs without error for backward compatibility.
//
// Example:
//
//	exporter := NewExporter()
//	json, err := exporter.ExportCompact(program)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(json)
func (e *Exporter) ExportCompact(program *language.Program) ([]byte, error) {
	if program == nil {
		return []byte("{}"), nil
	}
	dump := e.ToModelDump(program)
	data, err := json.Marshal(dump)
	if err != nil {
		elementCount := 0
		if program.Model != nil {
			elementCount = len(program.Model.Items)
		}
		return nil, fmt.Errorf("export compact failed for program with %d elements: %w", elementCount, err)
	}
	return data, nil
}

// ToModelDump converts Program (AST) to SrujaModelDump.
// Handles nil programs gracefully by returning an empty model dump.
// Pre-allocates maps with reasonable capacity hints for performance.
//
// Example:
//
//	exporter := NewExporter()
//	dump := exporter.ToModelDump(program)
//	fmt.Printf("Elements: %d, Relations: %d\n", len(dump.Elements), len(dump.Relations))
func (e *Exporter) ToModelDump(program *language.Program) *SrujaModelDump {
	modelName := "Untitled"
	elementCount := 0
	if program != nil && program.Model != nil {
		elementCount = len(program.Model.Items)
		if elementCount > 0 {
			modelName = "Model"
		}
	}

	projectID := modelName
	projectDump := &ProjectDump{
		ID:   projectID,
		Name: modelName,
	}

	estimatedCapacity := max(16, elementCount*2)

	dump := &SrujaModelDump{
		Stage:     "parsed",
		ProjectID: projectID,
		Project:   projectDump,
		Globals:   &GlobalsDump{},
		Imports:   make(map[string][]ElementDump, estimatedCapacity/4),
		Deployments: &DeploymentsDump{
			Elements:  make(map[string]interface{}, estimatedCapacity/2),
			Relations: make(map[string]interface{}, estimatedCapacity/2),
		},
		Specification: e.buildSpecification(program),
		Elements:      make(map[string]ElementDump, estimatedCapacity),
		Relations:     make([]RelationDump, 0, estimatedCapacity),
		Views:         make(map[string]ViewDump, 4),
		Metadata: ModelMetadata{
			Name:      modelName,
			Version:   "1.0.0",
			Generated: time.Now().Format(time.RFC3339),
			SrujaVer:  "2.0.0",
		},
	}

	if program != nil && program.Model != nil {
		e.convertElementsFromModel(dump, program.Model)
		e.convertRelationsFromModel(dump, program.Model)
	}

	e.convertViewsFromProgram(dump, program)
	dump.Sruja = e.buildSrujaExtensionsFromProgram(program)

	return dump
}
