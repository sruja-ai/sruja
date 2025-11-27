// Package compiler provides compilation from model to various diagram formats.
package compiler

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

// FormatRecommendation represents a format recommendation with reasoning.
type FormatRecommendation struct {
	Format       string   // Recommended format name
	Score        float64  // Recommendation score (0-1)
	Reasons      []string // Reasons for recommendation
	Alternatives []string // Alternative formats to consider
}

// Selector automatically selects the best diagram format based on model characteristics.
//
// The selector analyzes:
//   - Model complexity (number of elements, relations)
//   - Model structure (hierarchical depth, patterns)
//   - Use case (presentation, documentation, version control)
//   - Format capabilities (themes, animations, export options)
type Selector struct {
	registry *Registry
}

// NewSelector creates a new format selector.
func NewSelector(registry *Registry) *Selector {
	return &Selector{registry: registry}
}

// Recommend analyzes a program and recommends the best format.
func (s *Selector) Recommend(program *language.Program, useCase string) (*FormatRecommendation, error) {
	// Transform to model for analysis
	transformer := NewTransformer()
	m, err := transformer.Transform(program)
	if err != nil {
		return nil, fmt.Errorf("transformation error: %w", err)
	}

	return s.recommendFromModel(m, useCase), nil
}

// recommendFromModel recommends format based on model characteristics.
func (s *Selector) recommendFromModel(m *model.Model, useCase string) *FormatRecommendation {
	scores := make(map[string]float64)
	reasons := make(map[string][]string)

	// Analyze model characteristics
	elementCount := len(m.Architecture.Elements)
	relationCount := len(m.Architecture.Relations)
	hasJourneys := len(m.Architecture.Journeys) > 0
	complexity := s.calculateComplexity(m)

	// Score each available format
	for _, format := range s.registry.List() {
		score := 0.0
		formatReasons := []string{}

		switch format {
		case "d2":
			// D2 excels at:
			// - Beautiful visualizations (presentation)
			// - Complex hierarchical structures
			// - Animations and themes
			// - Large diagrams with many elements
			if elementCount > 20 {
				score += 0.3
				formatReasons = append(formatReasons, "D2 handles large diagrams well")
			}
			if complexity > 0.7 {
				score += 0.2
				formatReasons = append(formatReasons, "D2 supports complex hierarchical structures")
			}
			if useCase == "presentation" || useCase == "export" {
				score += 0.4
				formatReasons = append(formatReasons, "D2 provides beautiful themes and export options")
			}
			if relationCount > 30 {
				score += 0.1
				formatReasons = append(formatReasons, "D2 handles many relationships efficiently")
			}
			// D2 is good for general use
			score += 0.2
			formatReasons = append(formatReasons, "D2 is versatile and production-ready")

		case "mermaid":
			// Mermaid excels at:
			// - Version control (text-based, Git-friendly)
			// - Documentation (GitHub, markdown)
			// - Journey/sequence diagrams
			// - Simple to moderate complexity
			if useCase == "documentation" || useCase == "version-control" {
				score += 0.5
				formatReasons = append(formatReasons, "Mermaid is text-based and Git-friendly")
			}
			if hasJourneys {
				score += 0.3
				formatReasons = append(formatReasons, "Mermaid has excellent journey/sequence diagram support")
			}
			if elementCount < 30 {
				score += 0.2
				formatReasons = append(formatReasons, "Mermaid works well for moderate-sized diagrams")
			}
			if useCase == "github" || useCase == "markdown" {
				score += 0.4
				formatReasons = append(formatReasons, "Mermaid renders natively in GitHub and markdown")
			}
			// Mermaid is good for documentation
			score += 0.2
			formatReasons = append(formatReasons, "Mermaid is widely supported in documentation tools")
		}

		scores[format] = score
		reasons[format] = formatReasons
	}

	// Find best format
	bestFormat := ""
	bestScore := 0.0
	for format, score := range scores {
		if score > bestScore {
			bestScore = score
			bestFormat = format
		}
	}

	// If no format scored well, default to D2
	if bestFormat == "" || bestScore < 0.3 {
		bestFormat = "d2"
		bestScore = 0.5
		reasons[bestFormat] = []string{"D2 is the default robust format"}
	}

	// Get alternatives (other formats with decent scores)
	alternatives := []string{}
	for format, score := range scores {
		if format != bestFormat && score > 0.3 {
			alternatives = append(alternatives, format)
		}
	}

	return &FormatRecommendation{
		Format:       bestFormat,
		Score:        bestScore,
		Reasons:      reasons[bestFormat],
		Alternatives: alternatives,
	}
}

// calculateComplexity calculates model complexity (0-1).
func (s *Selector) calculateComplexity(m *model.Model) float64 {
	if m.Architecture == nil {
		return 0.0
	}

	elementCount := float64(len(m.Architecture.Elements))
	relationCount := float64(len(m.Architecture.Relations))

	// Normalize complexity (assume 100 elements and 200 relations is max complexity)
	elementComplexity := elementCount / 100.0
	if elementComplexity > 1.0 {
		elementComplexity = 1.0
	}

	relationComplexity := relationCount / 200.0
	if relationComplexity > 1.0 {
		relationComplexity = 1.0
	}

	// Weighted average
	complexity := (elementComplexity*0.4 + relationComplexity*0.6)

	// Check for hierarchical depth (systems containing containers containing components)
	depth := s.calculateDepth(m)
	depthComplexity := float64(depth) / 3.0
	if depthComplexity > 1.0 {
		depthComplexity = 1.0
	}

	// Combine
	return (complexity*0.7 + depthComplexity*0.3)
}

// calculateDepth estimates hierarchical depth of the model.
func (s *Selector) calculateDepth(m *model.Model) int {
	if m.Architecture == nil {
		return 0
	}

	hasSystems := false
	hasContainers := false
	hasComponents := false

	for _, e := range m.Architecture.Elements {
		switch e.Type {
		case model.ElementTypeSystem:
			hasSystems = true
		case model.ElementTypeContainer:
			hasContainers = true
		case model.ElementTypeComponent:
			hasComponents = true
		}
	}

	depth := 0
	if hasSystems {
		depth++
	}
	if hasContainers {
		depth++
	}
	if hasComponents {
		depth++
	}

	return depth
}

// AutoSelect automatically selects and compiles using the best format.
func (s *Selector) AutoSelect(program *language.Program, useCase string) (string, string, error) {
	rec, err := s.Recommend(program, useCase)
	if err != nil {
		return "", "", err
	}

	output, err := s.registry.Compile(rec.Format, program)
	if err != nil {
		return "", "", err
	}

	return rec.Format, output, nil
}

// FormatInfo provides information about a format.
type FormatInfo struct {
	Name          string   // Format name
	Description   string   // Format description
	UseCases      []string // Best use cases
	Capabilities  []string // Format capabilities
	ExportFormats []string // Supported export formats
}

// GetFormatInfo returns information about a format.
func (s *Selector) GetFormatInfo(format string) (*FormatInfo, error) {
	switch format {
	case "d2":
		return &FormatInfo{
			Name:          "d2",
			Description:   "Modern declarative diagramming language with beautiful themes",
			UseCases:      []string{"presentation", "export", "complex-diagrams", "animations"},
			Capabilities:  []string{"themes", "animations", "sketch-mode", "custom-styling", "containers", "layouts"},
			ExportFormats: []string{"svg", "png", "pdf"},
		}, nil
	case "mermaid":
		return &FormatInfo{
			Name:          "mermaid",
			Description:   "Text-based diagram format, Git-friendly and widely supported",
			UseCases:      []string{"documentation", "version-control", "github", "markdown", "journeys"},
			Capabilities:  []string{"c4-model", "journey-diagrams", "sequence-diagrams", "text-based"},
			ExportFormats: []string{"svg", "png", "pdf"},
		}, nil
	default:
		return nil, fmt.Errorf("format '%s' not found", format)
	}
}

// ListFormatsWithInfo returns all formats with their information.
func (s *Selector) ListFormatsWithInfo() (map[string]*FormatInfo, error) {
	formats := make(map[string]*FormatInfo)
	for _, formatName := range s.registry.List() {
		info, err := s.GetFormatInfo(formatName)
		if err != nil {
			return nil, err
		}
		formats[formatName] = info
	}
	return formats, nil
}
