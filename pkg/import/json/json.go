// pkg/import/json/json.go
// Package json provides JSON to AST conversion functionality.
package json

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// OutputFormat specifies how DSL files should be organized
type OutputFormat string

const (
	OutputFormatSingleFile    OutputFormat = "single"   // All in one file
	OutputFormatMultipleFiles OutputFormat = "multiple" // Concept-based files
)

// FileOutput represents a generated DSL file
type FileOutput struct {
	Path    string
	Content string
}

// stringToQualifiedIdent converts a string to a QualifiedIdent
func stringToQualifiedIdent(s string) language.QualifiedIdent {
	// Count dots to estimate capacity
	dotCount := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '.' {
			dotCount++
		}
	}
	parts := make([]string, 0, dotCount+1)
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '.' {
			if i > start {
				parts = append(parts, s[start:i])
			}
			start = i + 1
		}
	}
	if start < len(s) {
		parts = append(parts, s[start:])
	} else if len(parts) == 0 {
		// Empty string case
		parts = append(parts, "")
	}
	return language.QualifiedIdent{Parts: parts}
}

// Converter converts JSON to AST
type Converter struct{}

// NewConverter creates a new JSON to AST converter
func NewConverter() *Converter {
	return &Converter{}
}

// ToArchitecture removed - Architecture struct removed (old syntax no longer supported)
// Use LikeC4 importers instead
func (c *Converter) ToArchitecture(_ []byte) (interface{}, error) {
	return nil, fmt.Errorf("ToArchitecture is no longer supported - Architecture struct removed. Use LikeC4 importers instead")
}

// ToDSL removed - Architecture struct removed (old syntax no longer supported)
// Use LikeC4 importers instead
func (c *Converter) ToDSL(_ []byte, _ OutputFormat) ([]FileOutput, error) {
	return nil, fmt.Errorf("ToDSL is no longer supported - Architecture struct removed. Use LikeC4 importers instead")
}
