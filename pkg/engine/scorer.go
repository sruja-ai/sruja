package engine

import (
	"fmt"
	"strconv"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ScoreCard represents the result of an architectural score.
type ScoreCard struct {
	Score      int
	Grade      string
	Deductions []Deduction
}

// Deduction represents a point deduction with a reason.
type Deduction struct {
	Rule    string
	Points  int
	Message string
	Target  string
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

// CalculateScore calculates the score for a given program.
//
//nolint:funlen,gocyclo // Score calculation is long and complex
func (s *Scorer) CalculateScore(program *language.Program) ScoreCard {
	score := 100
	// Pre-allocate deductions slice with estimated capacity
	estimatedDeductions := 20
	deductions := make([]Deduction, 0, estimatedDeductions)

	// 1. Run Validation Rules (Correctness)
	diags := s.validator.Validate(program)
	for i := range diags {
		d := &diags[i]
		points := 0
		rule := ""
		switch d.Code {
		case diagnostics.CodeCycleDetected:
			points = 20
			rule = "Circular Dependency"
		case diagnostics.CodeLayerViolation:
			points = 10
			rule = "Layer Violation"
		case diagnostics.CodeOrphanElement:
			points = 5
			rule = "Orphan Element"
		case diagnostics.CodeReferenceNotFound:
			points = 10
			rule = "Invalid Reference"
		default:
			if d.Severity == diagnostics.SeverityError {
				points = 5
				rule = "Validation Error"
			}
		}

		if points > 0 {
			// Build target efficiently using pooled builder
			targetSb := GetStringBuilder()
			targetSb.Grow(len(d.Location.File) + 20)
			targetSb.WriteString(d.Location.File)
			targetSb.WriteByte(':')
			targetSb.WriteString(strconv.Itoa(d.Location.Line))
			target := targetSb.String()
			PutStringBuilder(targetSb)
			deductions = append(deductions, Deduction{
				Rule:    rule,
				Points:  points,
				Message: d.Message,
				Target:  target,
			})
			score -= points
		}
	}

	// 2. Check Completeness (Missing Metadata) - work with LikeC4 Model AST
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

			// Check if element has description
			hasDescription := false
			body := elem.GetBody()
			if body != nil {
				for _, bodyItem := range body.Items {
					if bodyItem.Description != nil {
						hasDescription = true
						break
					}
				}
			}

			if !hasDescription {
				elementID := id
				if parentID != "" {
					elementID = buildQualifiedID(parentID, id)
				}
				deductions = append(deductions, Deduction{
					Rule:    "Missing Description",
					Points:  2,
					Message: fmt.Sprintf("Element '%s' is missing a description", elementID),
					Target:  elementID,
				})
				score -= 2
			}

			// Recursively check nested elements
			body = elem.GetBody()
			if body != nil {
				qualifiedID := id
				if parentID != "" {
					qualifiedID = buildQualifiedID(parentID, id)
				}
				for _, bodyItem := range body.Items {
					if bodyItem.Element != nil {
						checkElement(bodyItem.Element, qualifiedID)
					}
				}
			}
		}

		// Check all top-level elements
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				checkElement(item.ElementDef, "")
			}
		}
	}

	// Cap score at 0
	if score < 0 {
		score = 0
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
		Deductions: deductions,
	}
}
