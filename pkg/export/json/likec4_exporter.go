package json

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/sruja-ai/sruja/pkg/language"
)

// LikeC4Exporter converts LikeC4 Program AST to LikeC4-compatible JSON
type LikeC4Exporter struct {
	Extended bool // Include computed views with layout
}

// NewLikeC4Exporter creates a new exporter
func NewLikeC4Exporter() *LikeC4Exporter {
	return &LikeC4Exporter{}
}

// Export converts Program (LikeC4 AST) to LikeC4-compatible JSON string
func (e *LikeC4Exporter) Export(program *language.Program) (string, error) {
	if program == nil {
		return "{}", nil
	}

	dump := e.ToModelDump(program)

	data, err := json.MarshalIndent(dump, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal JSON: %w", err)
	}
	return string(data), nil
}

// ExportCompact exports without indentation
func (e *LikeC4Exporter) ExportCompact(program *language.Program) ([]byte, error) {
	if program == nil {
		return []byte("{}"), nil
	}
	dump := e.ToModelDump(program)
	return json.Marshal(dump)
}

// ToModelDump converts Program (LikeC4 AST) to SrujaModelDump
func (e *LikeC4Exporter) ToModelDump(program *language.Program) *SrujaModelDump {
	modelName := "Untitled"
	if program != nil && program.Model != nil {
		// Try to extract name from filename or use default
		modelName = "Model"
	}

	projectID := modelName
	projectDump := &ProjectDump{
		ID:   projectID,
		Name: modelName,
	}

	dump := &SrujaModelDump{
		Stage:         "parsed", // "parsed" stage - LikeC4 will compute view layouts at runtime
		ProjectID:     projectID,
		Project:       projectDump,
		Globals:       &GlobalsDump{},                 // Empty globals by default
		Imports:       make(map[string][]ElementDump), // Empty imports by default
		Deployments:   &DeploymentsDump{Elements: make(map[string]interface{}), Relations: make(map[string]interface{})},
		Specification: e.buildSpecification(program),
		Elements:      make(map[string]ElementDump),
		Relations:     []RelationDump{},
		Views:         make(map[string]ViewDump),
		Metadata: ModelMetadata{
			Name:      modelName,
			Version:   "1.0.0",
			Generated: time.Now().Format(time.RFC3339),
			SrujaVer:  "2.0.0",
		},
	}

	if program != nil && program.Model != nil {
		// Convert elements (flat with FQN)
		e.convertElementsFromModel(dump, program.Model)

		// Convert relations
		e.convertRelationsFromModel(dump, program.Model)
	}

	// Convert views
	e.convertViewsFromProgram(dump, program)

	// Add Sruja extensions (governance)
	dump.Sruja = e.buildSrujaExtensionsFromProgram(program)

	return dump
}
