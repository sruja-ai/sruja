package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type ValidReferenceRule struct{}

func (r *ValidReferenceRule) Name() string {
	return "Valid References"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *ValidReferenceRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil {
		return nil
	}

	// Work directly with LikeC4 Model AST
	if program.Model == nil {
		return nil
	}

	// Collect all defined elements and relations from Model block
	defined, relations := collectLikeC4Elements(program.Model)
	diags := make([]diagnostics.Diagnostic, 0, len(relations))

	resolve := func(ref, scope string) string {
		// 1. Try absolute/global match
		if defined[ref] {
			return ref
		}

		// 2. Try relative to scope, walking up
		if scope != "" {
			candidate := scope + "." + ref
			if defined[candidate] {
				return candidate
			}

			parts := strings.Split(scope, ".")
			for i := len(parts) - 1; i >= 0; i-- {
				prefix := strings.Join(parts[:i], ".")
				var candidate string
				if prefix == "" {
					candidate = ref
				} else {
					candidate = prefix + "." + ref
				}
				if defined[candidate] {
					return candidate
				}
			}
		}

		// 3. For architecture-level relations (scope=""), search all defined elements
		// to find matching IDs (e.g., "API" matches "Order.API")
		if scope == "" {
			for id := range defined {
				parts := strings.Split(id, ".")
				if len(parts) > 0 && parts[len(parts)-1] == ref {
					return id
				}
			}
		}

		return ""
	}

	checkRel := func(rel *language.Relation, scope string) {
		if rel == nil {
			return
		}
		if resolve(rel.From.String(), scope) == "" {
			loc := rel.Location()
			fromID := rel.From.String()

			// Build enhanced error message with suggestions
			var msgSb strings.Builder
			msgSb.Grow(len(fromID) + 100)
			msgSb.WriteString("Reference to undefined element '")
			msgSb.WriteString(fromID)
			msgSb.WriteString("'")
			if scope != "" {
				msgSb.WriteString(fmt.Sprintf(" in scope '%s'", scope))
			}

			var suggestions []string
			suggestions = append(suggestions, "Check if the element is defined before this relation")
			suggestions = append(suggestions, "Ensure the element ID matches exactly (case-sensitive)")
			if scope != "" {
				suggestions = append(suggestions, fmt.Sprintf("Try using the fully qualified name: '%s.%s'", scope, fromID))
			}
			// Try to find similar element names
			similar := r.findSimilarElements(fromID, defined)
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
				Code:     diagnostics.CodeReferenceNotFound,
				Severity: diagnostics.SeverityError,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
				Suggestions: suggestions,
			})
		}
		if resolve(rel.To.String(), scope) == "" {
			loc := rel.Location()
			toID := rel.To.String()

			// Build enhanced error message with suggestions
			var msgSb strings.Builder
			msgSb.Grow(len(toID) + 100)
			msgSb.WriteString("Reference to undefined element '")
			msgSb.WriteString(toID)
			msgSb.WriteString("'")
			if scope != "" {
				msgSb.WriteString(fmt.Sprintf(" in scope '%s'", scope))
			}

			var suggestions []string
			suggestions = append(suggestions, "Check if the element is defined before this relation")
			suggestions = append(suggestions, "Ensure the element ID matches exactly (case-sensitive)")
			if scope != "" {
				suggestions = append(suggestions, fmt.Sprintf("Try using the fully qualified name: '%s.%s'", scope, toID))
			}
			// Try to find similar element names
			similar := r.findSimilarElements(toID, defined)
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
				Code:     diagnostics.CodeReferenceNotFound,
				Severity: diagnostics.SeverityError,
				Message:  msgSb.String(),
				Location: diagnostics.SourceLocation{
					File:   loc.File,
					Line:   loc.Line,
					Column: loc.Column,
				},
				Suggestions: suggestions,
			})
		}
	}
	// Check all relations from Model block
	for _, rel := range relations {
		scope := getElementScope(program.Model, rel)
		checkRel(rel, scope)
	}

	return diags
}

// findSimilarElements finds element IDs similar to the given reference (for typo detection)
func (r *ValidReferenceRule) findSimilarElements(ref string, defined map[string]bool) []string {
	var similar []string
	refLower := strings.ToLower(ref)

	// Calculate similarity for each defined element
	type candidate struct {
		id    string
		score float64
	}
	candidates := make([]candidate, 0, len(defined))

	for id := range defined {
		score := r.calculateSimilarity(refLower, strings.ToLower(id))
		if score > 0.3 { // Threshold for similarity
			candidates = append(candidates, candidate{id: id, score: score})
		}
	}

	// Sort by score (simple selection sort for small lists)
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

// calculateSimilarity calculates a similarity score between two strings (0.0 to 1.0)
func (r *ValidReferenceRule) calculateSimilarity(s1, s2 string) float64 {
	if s1 == s2 {
		return 1.0
	}

	// Check if one contains the other
	if strings.Contains(s1, s2) || strings.Contains(s2, s1) {
		return 0.7
	}

	// Simple character-based similarity
	if len(s1) == 0 || len(s2) == 0 {
		return 0.0
	}

	// Count matching characters in same positions
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

	// Also check if last part matches (for qualified IDs)
	if strings.HasSuffix(s1, s2) || strings.HasSuffix(s2, s1) {
		return 0.6
	}

	// Return normalized score
	if minLen > 0 {
		return float64(matches) / float64(minLen)
	}
	return 0.0
}
