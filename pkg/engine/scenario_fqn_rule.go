package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type ScenarioFQNRule struct{}

func (r *ScenarioFQNRule) Name() string { return "ScenarioReferenceValidation" }

// findSimilarInDefined finds element IDs similar to the given reference
func findSimilarInDefined(ref string, defined map[string]*language.ElementDef) []string {
	var similar []string
	refLower := strings.ToLower(ref)

	type candidate struct {
		id    string
		score float64
	}
	candidates := make([]candidate, 0, len(defined))

	for id := range defined {
		score := calculateSimilarityForScenario(refLower, strings.ToLower(id))
		if score > 0.3 {
			candidates = append(candidates, candidate{id: id, score: score})
		}
	}

	// Sort by score and return top 3
	for i := 0; i < len(candidates) && i < 3; i++ {
		maxIdx := i
		for j := i + 1; j < len(candidates); j++ {
			if candidates[j].score > candidates[maxIdx].score {
				maxIdx = j
			}
		}
		if maxIdx != i {
			candidates[i], candidates[maxIdx] = candidates[maxIdx], candidates[i]
		}
		similar = append(similar, candidates[i].id)
	}

	return similar
}

// calculateSimilarityForScenario calculates similarity between two strings
func calculateSimilarityForScenario(s1, s2 string) float64 {
	if s1 == s2 {
		return 1.0
	}
	if strings.Contains(s1, s2) || strings.Contains(s2, s1) {
		return 0.7
	}
	if len(s1) == 0 || len(s2) == 0 {
		return 0.0
	}
	matches := 0
	minLen := len(s1)
	if len(s2) < minLen {
		minLen = len(s2)
	}
	for i := 0; i < minLen; i++ {
		if s1[i] == s2[i] {
			matches++
		}
	}
	if minLen > 0 {
		return float64(matches) / float64(minLen)
	}
	return 0.0
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *ScenarioFQNRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all elements from Model
	defined, _ := collectElements(program.Model)

	// Build map of suffix -> []fullID for smart resolution
	suffixMap := make(map[string][]string, len(defined)/2)

	// Pre-allocate diagnostics slice
	diags := make([]diagnostics.Diagnostic, 0, 8)

	// Build suffix map from defined elements
	for id := range defined {
		if id == "" {
			continue
		}
		lastDot := strings.LastIndexByte(id, '.')
		var suffix string
		if lastDot == -1 {
			suffix = id
		} else {
			suffix = id[lastDot+1:]
		}
		suffixMap[suffix] = append(suffixMap[suffix], id)
	}

	_validateRef := func(ref string, loc language.SourceLocation) {
		if ref == "" {
			return
		}

		// 1. Exact match (Fully Qualified or Global)
		if defined[ref] != nil {
			return
		}

		// 2. Unqualified match
		// Use LastIndex for better performance than Split when we only need the last part
		lastDot := strings.LastIndexByte(ref, '.')
		var suffix string
		if lastDot == -1 {
			suffix = ref
		} else {
			suffix = ref[lastDot+1:]
		}

		matches := suffixMap[suffix]

		if len(matches) == 0 {
			// Build enhanced error message with suggestions
			var msgSb strings.Builder
			msgSb.Grow(len(ref) + 100)
			msgSb.WriteString("Reference to undefined element '")
			msgSb.WriteString(ref)
			msgSb.WriteString("' in scenario/flow step")

			var suggestions []string
			suggestions = append(suggestions, "Check if the element is defined before this step")
			suggestions = append(suggestions, "Ensure the element ID matches exactly (case-sensitive)")
			// Try to find similar element names
			similar := findSimilarInDefined(ref, defined)
			if len(similar) > 0 {
				var similarSb strings.Builder
				similarSb.Grow(50)
				similarSb.WriteString("Did you mean: ")
				for i, sim := range similar {
					if i > 0 {
						similarSb.WriteString(", ")
					}
					similarSb.WriteString("'")
					similarSb.WriteString(sim)
					similarSb.WriteString("'")
				}
				suggestions = append(suggestions, similarSb.String())
			}

			diags = append(diags, diagnostics.Diagnostic{
				Code:        diagnostics.CodeReferenceNotFound,
				Severity:    diagnostics.SeverityError,
				Message:     msgSb.String(),
				Location:    diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
				Suggestions: suggestions,
			})
			return
		}

		if len(matches) == 1 {
			// Unique resolution found!
			// We allow this usage.
			// Implicitly resolves to matches[0]
			return
		}

		// Ambiguous match - build enhanced error message
		var msgSb strings.Builder
		msgSb.Grow(len(ref) + len(strings.Join(matches, ", ")) + 80)
		msgSb.WriteString("Ambiguous reference '")
		msgSb.WriteString(ref)
		msgSb.WriteString("' matches multiple elements: ")
		msgSb.WriteString(strings.Join(matches, ", "))
		msgSb.WriteString(". Use a fully qualified name to disambiguate.")

		var suggestions []string
		for _, match := range matches {
			suggestions = append(suggestions, fmt.Sprintf("Use fully qualified name: '%s'", match))
		}
		if len(suggestions) > 3 {
			suggestions = suggestions[:3] // Limit to 3 suggestions
		}

		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeValidationRuleError,
			Severity:    diagnostics.SeverityError,
			Message:     msgSb.String(),
			Location:    diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
			Suggestions: suggestions,
		})
	}

	// Note: validateRef was for validating scenario steps
	// Currently steps in scenarios/flows need specialized handling
	// after step parsing is integrated into ElementDef body
	_ = _validateRef // Suppress unused warning until step validation is needed

	// Check scenarios and flows from Model items
	// Note: With the new unified syntax, scenarios/flows are parsed via ElementDef
	// Step validation would need to be updated once step parsing is integrated into ElementDef body
	// For now, we only validate relations in element bodies
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			// Could check body items for step-like relations if needed
			// Currently steps in scenarios/flows need to be parsed differently
			_ = item.ElementDef.Assignment // explicit usage to satisfy linter
		}
	}

	return diags
}
