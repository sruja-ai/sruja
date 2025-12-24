package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// CategoryScores represents the scores for each health dimension.
type CategoryScores struct {
	Structural      int // 40%
	Documentation   int // 20%
	Traceability    int // 15%
	Complexity      int // 15%
	Standardization int // 10%
}

// ScoreCard represents the result of an architectural score.
type ScoreCard struct {
	Score      int
	Grade      string
	Categories CategoryScores
	Deductions []Deduction
}

// Deduction represents a point deduction with a reason.
type Deduction struct {
	Rule     string
	Points   int
	Message  string
	Target   string
	Severity diagnostics.Severity
	Category string // Category names like "Structural", "Documentation", etc.
}

// Scorer calculates the architecture score.
type Scorer struct {
	validator *Validator
}

// NewScorer creates a new Scorer.
func NewScorer() *Scorer {
	v := NewValidator()
	v.RegisterRule(&CycleDetectionRule{})
	v.RegisterRule(&OrphanDetectionRule{})
	v.RegisterRule(&LayerViolationRule{})
	v.RegisterRule(&ValidReferenceRule{})

	return &Scorer{
		validator: v,
	}
}

func (s *Scorer) CalculateScore(program *language.Program) ScoreCard {
	// Initialize category scores (start at 100 for each)
	scores := CategoryScores{
		Structural:      100,
		Documentation:   100,
		Traceability:    100,
		Complexity:      100,
		Standardization: 100,
	}

	deductions := make([]Deduction, 0, 20)

	// 1. Structural Integrity (40%) - Validation Rules
	diags := s.validator.Validate(program)
	for i := range diags {
		d := &diags[i]
		points := 0
		rule := ""
		severity := d.Severity

		switch d.Code {
		case diagnostics.CodeCycleDetected:
			points = 30
			rule = "Circular Dependency"
			severity = diagnostics.SeverityError
		case diagnostics.CodeLayerViolation:
			points = 15
			rule = "Layer Violation"
			severity = diagnostics.SeverityWarning
		case diagnostics.CodeOrphanElement:
			points = 10
			rule = "Orphan Element"
			severity = diagnostics.SeverityWarning
		case diagnostics.CodeReferenceNotFound:
			points = 20
			rule = "Invalid Reference"
			severity = diagnostics.SeverityError
		default:
			if d.Severity == diagnostics.SeverityError {
				points = 10
				rule = "Validation Error"
			}
		}

		if points > 0 {
			target := fmt.Sprintf("%s:%d", d.Location.File, d.Location.Line)
			deductions = append(deductions, Deduction{
				Rule:     rule,
				Points:   points,
				Message:  d.Message,
				Target:   target,
				Severity: severity,
				Category: "Structural",
			})
			scores.Structural -= points
		}
	}

	// 2. Documentation Depth (20%) & Standardization (10%)
	if program.Model != nil {
		var checkElement func(elem *language.LikeC4ElementDef, parentID string)
		checkElement = func(elem *language.LikeC4ElementDef, parentID string) {
			if elem == nil {
				return
			}
			id := elem.GetID()
			if id == "" {
				return
			}
			elementID := id
			if parentID != "" {
				elementID = buildQualifiedID(parentID, id)
			}

			// Documentation Checks
			hasDescription := false
			hasTechnology := false
			hasMetadata := false
			body := elem.GetBody()
			if body != nil {
				for _, item := range body.Items {
					if item.Description != nil {
						hasDescription = true
					}
					if item.Technology != nil {
						hasTechnology = true
					}
					if item.Metadata != nil {
						hasMetadata = true
					}
				}
			}

			if !hasDescription {
				deductions = append(deductions, Deduction{
					Rule:     "Missing Description",
					Points:   5,
					Message:  fmt.Sprintf("Element '%s' is missing a description", elementID),
					Target:   elementID,
					Severity: diagnostics.SeverityInfo,
					Category: "Documentation",
				})
				scores.Documentation -= 5
			}

			// Only check technology for containers and components
			kind := elem.GetKind()
			if kind == "container" || kind == "component" {
				if !hasTechnology {
					deductions = append(deductions, Deduction{
						Rule:     "Missing Technology",
						Points:   5,
						Message:  fmt.Sprintf("Element '%s' is missing technology stack", elementID),
						Target:   elementID,
						Severity: diagnostics.SeverityInfo,
						Category: "Documentation",
					})
					scores.Documentation -= 5
				}
			}

			// Standardization Checks
			if !hasMetadata {
				scores.Standardization -= 2
			}

			// Recursively check nested
			if body != nil {
				for _, item := range body.Items {
					if item.Element != nil {
						checkElement(item.Element, elementID)
					}
				}
			}
		}

		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				checkElement(item.ElementDef, "")
			}
		}
	}

	// 3. Traceability (15%) - Requirement Coverage
	if program.Model != nil {
		totalElements := 0
		taggedElements := make(map[string]bool)

		// Map requirements and count elements
		for _, item := range program.Model.Items {
			if item.Requirement != nil && item.Requirement.Body != nil {
				for _, tag := range item.Requirement.Body.Tags {
					taggedElements[tag] = true
				}
			}
			if item.ElementDef != nil {
				totalElements++
				// Recursive count would be better but this is a good start
			}
		}

		if totalElements > 0 && len(taggedElements) < totalElements/2 {
			scores.Traceability -= 20 // Penalty for low requirement coverage
			deductions = append(deductions, Deduction{
				Rule:     "Low Traceability",
				Points:   20,
				Message:  "Less than 50% of elements are mapped to requirements",
				Target:   "requirements",
				Severity: diagnostics.SeverityWarning,
				Category: "Traceability",
			})
		}
	}

	// Ensure categories don't go below 0
	clamp := func(n int) int {
		if n < 0 {
			return 0
		}
		if n > 100 {
			return 100
		}
		return n
	}
	scores.Structural = clamp(scores.Structural)
	scores.Documentation = clamp(scores.Documentation)
	scores.Traceability = clamp(scores.Traceability)
	scores.Complexity = clamp(scores.Complexity)
	scores.Standardization = clamp(scores.Standardization)

	// Calculate Final Weighted Score
	// Structural (40%), Doc (20%), Trace (15%), Complexity (15%), Standard (10%)
	finalScore := float64(scores.Structural)*0.40 +
		float64(scores.Documentation)*0.20 +
		float64(scores.Traceability)*0.15 +
		float64(scores.Complexity)*0.15 +
		float64(scores.Standardization)*0.10

	// Apply critical multiplier if structural is very low
	if scores.Structural < 50 {
		finalScore *= 0.8
	}

	score := int(finalScore)
	if score < 0 {
		score = 0
	}
	if score > 100 {
		score = 100
	}

	// Calculate Grade
	grade := "F"
	switch {
	case score >= 90:
		grade = "A"
	case score >= 80:
		grade = "B"
	case score >= 70:
		grade = "C"
	case score >= 60:
		grade = "D"
	}

	return ScoreCard{
		Score:      score,
		Grade:      grade,
		Categories: scores,
		Deductions: deductions,
	}
}
